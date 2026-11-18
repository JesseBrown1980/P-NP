//! Read source-authenticated PyTorch checkpoint ZIP containers as inert bytes.
//! No pickle execution, neural inference, runtime dependencies, or float ops.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::sha256::hash_bytes;
use crate::source;

const PIN_INDEX: &str = include_str!("../../policy/TRAINED_SOURCE_PINS.hbi");
const HBP_HEADER: &str = "HBPHEADER|schema=pnp.checkpoint.storage.v1|seal=HBI-HBP-SHA-SH-HASH|forward_parity=HOLD|json=0\n";
const HBI_HEADER: &str = "HBIHEADER|schema=pnp.checkpoint.index.v1|for=PNP_CHECKPOINT.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const MODELS: [&str; 9] = [
    "baseline_model.pt", "contrastive_model.pt",
    "contrastive_model_run2.pt", "contrastive_model_run3.pt",
    "gslgnn_w9_3_seq47.pt", "gslgnn_w9_3_seq47_v2.pt",
    "prototype_model.pt", "prototype_model_run2.pt",
    "prototype_model_run3.pt",
];

fn read16(data: &[u8], pos: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(pos..pos.checked_add(2)?)?.try_into().ok()?))
}
fn read32(data: &[u8], pos: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(pos..pos.checked_add(4)?)?.try_into().ok()?))
}
fn read64(data: &[u8], pos: usize) -> Option<u64> {
    Some(u64::from_le_bytes(data.get(pos..pos.checked_add(8)?)?.try_into().ok()?))
}
fn checksum_crc32(data: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[derive(Clone, Debug)]
struct Entry<'a> {
    name: &'a str,
    payload: &'a [u8],
}

fn read_zip(data: &[u8]) -> Result<Vec<Entry<'_>>, String> {
    if data.len() < 22 || data.len() > 20_000_000 {
        return Err("checkpoint ZIP byte range invalid".to_owned());
    }
    // End-of-central-directory may have an optional 65535-byte comment.
    let first = data.len().saturating_sub(22 + 65_535);
    let end = data.len() - 22;
    let eocd = (first..=end).rev().find(|&position| {
        read32(data, position) == Some(0x0605_4b50)
            && read16(data, position + 20).is_some_and(|comment| {
                position.checked_add(22)
                    .and_then(|p| p.checked_add(usize::from(comment))) == Some(data.len())
            })
    }).ok_or("checkpoint central-directory terminator absent")?;
    let (cd_start, cd_len, total_entries, directory_end) =
        if eocd >= 20 && read32(data, eocd - 20) == Some(0x0706_4b50) {
            // Original PyTorch serialization commonly emits ZIP64 metadata
            // even when the archive itself is only a few kilobytes.
            let locator = eocd - 20;
            if read32(data, locator + 4) != Some(0)
                || read32(data, locator + 16) != Some(1)
            {
                return Err("invalid ZIP64 locator".to_owned());
            }
            let zoff = usize::try_from(read64(data, locator + 8)
                .ok_or("ZIP64 directory offset")?)
                .map_err(|_| "ZIP64 location overflow")?;
            if read32(data, zoff) != Some(0x0606_4b50) {
                return Err("ZIP64 directory missing".to_owned());
            }
            let zlen = usize::try_from(read64(data, zoff + 4)
                .ok_or("ZIP64 record size")?).map_err(|_| "ZIP64 size overflow")?;
            if zlen < 44 || zoff.checked_add(12).and_then(|p| p.checked_add(zlen))
                != Some(locator)
                || read32(data, zoff + 16) != Some(0)
                || read32(data, zoff + 20) != Some(0)
            {
                return Err("invalid ZIP64 directory header".to_owned());
            }
            let on_disk = read64(data, zoff + 24).ok_or("ZIP64 entries on disk")?;
            let total = read64(data, zoff + 32).ok_or("ZIP64 total entries")?;
            if on_disk != total || total == 0 || total > 1000 {
                return Err("unsupported ZIP64 checkpoint count".to_owned());
            }
            (
                usize::try_from(read64(data, zoff + 48).ok_or("ZIP64 start")?)
                    .map_err(|_| "ZIP64 directory offset overflow")?,
                usize::try_from(read64(data, zoff + 40).ok_or("ZIP64 length")?)
                    .map_err(|_| "ZIP64 directory length overflow")?,
                usize::try_from(total).map_err(|_| "ZIP64 entry count overflow")?,
                zoff,
            )
        } else {
            let disk = read16(data, eocd + 4).ok_or("disk count")?;
            let cd_disk = read16(data, eocd + 6).ok_or("central disk")?;
            let this_entries = read16(data, eocd + 8).ok_or("local entries")?;
            let total_entries = read16(data, eocd + 10).ok_or("total entries")?;
            if disk != 0 || cd_disk != 0 || this_entries != total_entries
                || total_entries == 0 || total_entries > 1000
            {
                return Err("unsupported split checkpoint".to_owned());
            }
            (
                usize::try_from(read32(data, eocd + 16).ok_or("directory offset")?)
                    .map_err(|_| "directory position overflow")?,
                usize::try_from(read32(data, eocd + 12).ok_or("directory length")?)
                    .map_err(|_| "directory length overflow")?,
                usize::from(total_entries),
                eocd,
            )
        };
    if cd_start.checked_add(cd_len) != Some(directory_end) {
        return Err("checkpoint central-directory extent mismatch".to_owned());
    }

    let mut pos = cd_start;
    let mut entries = Vec::with_capacity(total_entries);
    let mut seen = BTreeSet::new();
    for _ in 0..total_entries {
        if read32(data, pos) != Some(0x0201_4b50) {
            return Err("malformed ZIP central record".to_owned());
        }
        let flags = read16(data, pos + 8).ok_or("ZIP flags")?;
        let method = read16(data, pos + 10).ok_or("ZIP compression method")?;
        let expected_crc = read32(data, pos + 16).ok_or("ZIP CRC")?;
        let compressed32 = read32(data, pos + 20).ok_or("ZIP length")?;
        let original32 = read32(data, pos + 24).ok_or("ZIP original length")?;
        let name_len = usize::from(read16(data, pos + 28).ok_or("ZIP filename length")?);
        let extra_len = usize::from(read16(data, pos + 30).ok_or("ZIP extra length")?);
        let comment_len = usize::from(read16(data, pos + 32).ok_or("ZIP comment length")?);
        let local32 = read32(data, pos + 42).ok_or("ZIP local offset")?;
        let name_start = pos.checked_add(46).ok_or("ZIP offset")?;
        let name_end = name_start.checked_add(name_len).ok_or("ZIP offset")?;
        let extra_end = name_end.checked_add(extra_len).ok_or("ZIP extra extent")?;
        let extra = data.get(name_end..extra_end).ok_or("ZIP extra absent")?;
        let mut offset = 0_usize;
        let mut zip64_extra = None;
        while offset < extra.len() {
            let tag = read16(extra, offset).ok_or("ZIP extra header")?;
            let len = usize::from(read16(extra, offset + 2).ok_or("ZIP extra size")?);
            let begin = offset.checked_add(4).ok_or("ZIP extra overflow")?;
            let finish = begin.checked_add(len).ok_or("ZIP extra overflow")?;
            let contents = extra.get(begin..finish).ok_or("ZIP extra truncated")?;
            if tag == 1 && zip64_extra.replace(contents).is_some() {
                return Err("duplicate ZIP64 extra".to_owned());
            }
            offset = finish;
        }
        let mut extra_pos = 0_usize;
        let mut expand = |raw: u32| -> Result<usize, String> {
            if raw != u32::MAX {
                return usize::try_from(raw).map_err(|_| "ZIP offset out of range".to_owned());
            }
            let more = zip64_extra.ok_or("ZIP64 extra required")?;
            let value = read64(more, extra_pos).ok_or("ZIP64 value truncated")?;
            extra_pos += 8;
            usize::try_from(value).map_err(|_| "ZIP64 value overflow".to_owned())
        };
        // ZIP64 values occur in exactly this order when the 32-bit
        // central-directory fields contain sentinel values.
        let original = expand(original32)?;
        let compressed = expand(compressed32)?;
        let local = expand(local32)?;
        if flags & 1 != 0 || method != 0 || compressed != original
            || read32(data, local) != Some(0x0403_4b50)
            || read16(data, local + 8) != Some(method)
        {
            return Err("unsupported encrypted/compressed/broken checkpoint ZIP".to_owned());
        }
        let name_raw = data.get(name_start..name_end).ok_or("ZIP path")?;
        let name = std::str::from_utf8(name_raw).map_err(|_| "non-UTF8 ZIP path")?;
        if name.is_empty() || name.starts_with('/')
            || name.split('/').any(|piece| piece.is_empty() || piece == "..")
            || !seen.insert(name)
        {
            return Err("invalid or duplicate checkpoint ZIP name".to_owned());
        }
        let local_len = usize::from(read16(data, local + 26).ok_or("local path")?);
        let local_extra = usize::from(read16(data, local + 28).ok_or("local extra")?);
        let local_start = local.checked_add(30).ok_or("ZIP local offset")?;
        let local_name_end = local_start.checked_add(local_len).ok_or("ZIP local offset")?;
        if data.get(local_start..local_name_end) != Some(name_raw) {
            return Err("checkpoint local and central names differ".to_owned());
        }
        let payload_start = local_name_end.checked_add(local_extra).ok_or("ZIP data start")?;
        let payload_end = payload_start.checked_add(compressed).ok_or("ZIP data end")?;
        let payload = data.get(payload_start..payload_end).ok_or("ZIP data missing")?;
        if checksum_crc32(payload) != expected_crc {
            return Err("checkpoint ZIP CRC mismatch".to_owned());
        }
        entries.push(Entry { name, payload });
        pos = name_end.checked_add(extra_len).and_then(|p| p.checked_add(comment_len))
            .ok_or("ZIP record overflow")?;
    }
    if pos != directory_end || entries.is_empty() {
        return Err("checkpoint ZIP directory length mismatch".to_owned());
    }
    Ok(entries)
}

/// Round exact IEEE binary32 bits to signed Q20 by integer operations only.
/// Reject infinities, NaNs and overflows. Finite accepted values have at
/// most 1/2 Q20 quantum of conversion error, with ties to even.
pub fn binary32_to_q20(bits: u32) -> Result<i64, &'static str> {
    let neg = bits >> 31 != 0;
    let exp = (bits >> 23) & 255;
    let fraction = bits & 0x7f_ffff;
    if exp == 255 { return Err("non-finite binary32"); }
    let (significand, shift): (u128, i32) = if exp == 0 {
        (u128::from(fraction), -129)
    } else {
        (u128::from((1_u32 << 23) | fraction), (exp as i32) - 130)
    };
    let magnitude = if shift >= 0 {
        let left = shift as u32;
        let limit = (i64::MAX as u128) + u128::from(neg);
        if left >= 64 || significand > (limit >> left) {
            return Err("Q20 overflow");
        }
        significand << left
    } else {
        let right = (-shift) as u32;
        if right >= 128 {
            0
        } else {
            let whole = significand >> right;
            let mask = (1_u128 << right) - 1;
            let remainder = significand & mask;
            let half = 1_u128 << (right - 1);
            whole + u128::from(remainder > half || (remainder == half && whole & 1 == 1))
        }
    };
    if neg {
        if magnitude > (i64::MAX as u128) + 1 { return Err("Q20 overflow"); }
        if magnitude == (i64::MAX as u128) + 1 { return Ok(i64::MIN); }
        Ok(-(magnitude as i64))
    } else {
        i64::try_from(magnitude).map_err(|_| "Q20 overflow")
    }
}

fn contains(data: &[u8], term: &[u8]) -> bool {
    data.windows(term.len()).any(|window| window == term)
}
#[derive(Clone, Debug)]
struct StorageReport {
    entries: usize,
    storages: usize,
    storage_bytes: usize,
    quantized_words: usize,
    q20_sha: String,
    q20_bytes: Vec<u8>,
    dtype: &'static str,
}
fn inspect(data: &[u8]) -> Result<StorageReport, String> {
    let entries = read_zip(data)?;
    let pickle = entries.iter().find(|entry| entry.name.ends_with("/data.pkl"))
        .ok_or("PyTorch tensor manifest missing")?;
    let has_float = contains(pickle.payload, b"FloatStorage");
    let unsupported = [
        b"HalfStorage".as_slice(), b"DoubleStorage".as_slice(),
        b"LongStorage".as_slice(), b"IntStorage".as_slice(),
        b"ByteStorage".as_slice(), b"BoolStorage".as_slice(),
        b"BFloat16Storage".as_slice(), b"ComplexFloatStorage".as_slice(),
        b"ComplexDoubleStorage".as_slice()
    ].iter().any(|kind| contains(pickle.payload, kind));
    let mut storages: Vec<&Entry<'_>> = entries.iter()
        .filter(|entry| {
            entry.name.rsplit_once("/data/").is_some_and(|(_, tail)| {
                !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit())
            })
        }).collect();
    if storages.is_empty() {
        return Err("PyTorch tensor storage absent".to_owned());
    }
    storages.sort_by(|a, b| a.name.cmp(b.name));
    let total_bytes: usize = storages.iter().try_fold(0_usize, |count, entry| {
        count.checked_add(entry.payload.len())
            .ok_or("tensor size overflow")
    })?;
    let float32_only = has_float && !unsupported
        && storages.iter().all(|entry| entry.payload.len() % 4 == 0);
    let mut bytes = Vec::new();
    let mut words = 0_usize;
    if float32_only {
        bytes.extend_from_slice(b"PNP-Q20-TENSOR-STORAGE-v1\0");
        for item in &storages {
            let length = u32::try_from(item.name.len())
                .map_err(|_| "storage name too long")?;
            bytes.extend_from_slice(&length.to_le_bytes());
            bytes.extend_from_slice(item.name.as_bytes());
            for raw in item.payload.chunks_exact(4) {
                let bits = u32::from_le_bytes(raw.try_into().map_err(|_| "tensor word")?);
                let q = binary32_to_q20(bits).map_err(str::to_owned)?;
                bytes.extend_from_slice(&q.to_le_bytes());
                words = words.checked_add(1).ok_or("tensor count overflow")?;
            }
        }
    }
    let q20_sha = if float32_only { hash_bytes(&bytes) } else { "none".to_owned() };
    Ok(StorageReport {
        entries: entries.len(),
        storages: storages.len(),
        storage_bytes: total_bytes,
        quantized_words: words,
        q20_sha,
        q20_bytes: bytes,
        dtype: if float32_only { "FLOAT32_ONLY_Q20" } else { "HOLD_DTYPE" },
    })
}

fn source_digest(archive: &Path) -> Result<String, String> {
    if source::verify(archive)? != 23 { return Err("source bundle not verified".to_owned()); }
    let source_hbp = fs::read(archive.join("PNP_TRAINED_SOURCE.hbp"))
        .map_err(|e| format!("source HBP absent: {e}"))?;
    Ok(hash_bytes(&source_hbp))
}
fn find_pin(filename: &str) -> Result<(&'static str, &'static str), String> {
    for line in PIN_INDEX.lines().filter(|line| line.starts_with("HBIPIN|")) {
        if line.contains(&format!("|path=models/{filename}|")) {
            let mut id = None;
            let mut git_blob = None;
            for part in line.split('|').skip(1) {
                if let Some(value) = part.strip_prefix("id=") { id = Some(value); }
                if let Some(value) = part.strip_prefix("git_blob=") { git_blob = Some(value); }
            }
            return Ok((id.ok_or("checkpoint pin ID missing")?,
                       git_blob.ok_or("checkpoint Git blob missing")?));
        }
    }
    Err(format!("unknown published checkpoint: {filename}"))
}
fn original_checkpoint_sha(archive: &Path, pin_id: &str) -> Result<String, String> {
    let text = fs::read_to_string(archive.join("PNP_TRAINED_SOURCE.hbp"))
        .map_err(|e| format!("source HBP unreadable: {e}"))?;
    let selected: Vec<&str> = text.lines()
        .filter(|line| {
            line.starts_with("MODEL_SOURCE|")
                && line.contains(&format!("|id={pin_id}|"))
        }).collect();
    if selected.len() != 1 {
        return Err(format!("missing unique original checkpoint: {pin_id}"));
    }
    let match_line = selected[0];
    let sha = match_line.split('|')
        .find_map(|piece| piece.strip_prefix("sha256="))
        .ok_or("original checkpoint digest absent")?;
    if sha.len() != 64 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid original checkpoint SHA".to_owned());
    }
    Ok(sha.to_owned())
}

fn create(models: &Path, source_root: &str, source_archive: &Path)
    -> Result<(String, String, String, Vec<(String, Vec<u8>)>), String>
{
    let mut hbp = HBP_HEADER.to_owned();
    let mut hbi = HBI_HEADER.to_owned();
    let mut prev = hash_bytes(b"PNP-ORIGINAL-TENSOR-STORAGE-GENESIS-v1");
    let mut converted_files = Vec::new();
    for (index, filename) in MODELS.iter().enumerate() {
        let (id, blob) = find_pin(filename)?;
        let raw = fs::read(models.join(filename))
            .map_err(|e| format!("checkpoint file missing {filename}: {e}"))?;
        let original_sha = hash_bytes(&raw);
        if original_sha != original_checkpoint_sha(source_archive, id)? {
            return Err(format!("real checkpoint differs from source-attested original: {id}"));
        }
        let storage = inspect(&raw).map_err(|error| format!("{filename}: {error}"))?;
        let sh = &original_sha[..16];
        let payload = format!(
            "CHECKPOINT|seq={}|id={id}|filename={filename}|git_blob={blob}|source_hbp_sha256={source_root}|original_sha256={original_sha}|sh={sh}|zip_entries={}|storage_count={}|storage_bytes={}|quantized_words={}|dtype={}|q20_sha256={}|q20_error_bound=HALF_QUANTUM_IF_ACCEPTED|forward_parity=HOLD|prev_hash={prev}",
            index + 1, storage.entries, storage.storages, storage.storage_bytes,
            storage.quantized_words, storage.dtype, storage.q20_sha
        );
        let receipt = hash_bytes(payload.as_bytes());
        let linked = hash_bytes(format!("{prev}|{receipt}").as_bytes());
        let row = format!("{payload}|receipt_sha={receipt}|hash={linked}|json=0\n");
        hbi.push_str(&format!(
            "HBI|seq={}|id={id}|offset={}|len={}|hbp_sha256={}|sh={sh}|hash={linked}|status=VERIFIED|json=0\n",
            index + 1, hbp.len(), row.len(), hash_bytes(row.as_bytes())
        ));
        hbp.push_str(&row);
        prev = linked;
        if storage.dtype == "FLOAT32_ONLY_Q20" {
            converted_files.push((format!("PNP_Q20_{id}.bin"), storage.q20_bytes));
        }
    }
    let head = format!(
        "CHECKPOINT_ANCHOR|count={}|source_hbp_sha256={source_root}|root_hbp_sha256={}|root_hbi_sha256={}|chain_hash={prev}|seal=HBI-HBP-SHA-SH-HASH",
        MODELS.len(), hash_bytes(hbp.as_bytes()), hash_bytes(hbi.as_bytes())
    );
    let anchor = format!("{head}|anchor_checksum={}|json=0\n",hash_bytes(head.as_bytes()));
    Ok((hbp,hbi,anchor,converted_files))
}
pub fn attest(models: &Path, source_archive: &Path, output: &Path) -> Result<usize, String> {
    let root = source_digest(source_archive)?;
    let (hbp,hbi,anchor,converted_files) = create(models, &root, source_archive)?;
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    for (name,text) in [
        ("PNP_CHECKPOINT.hbp",hbp),("PNP_CHECKPOINT.hbi",hbi),("PNP_CHECKPOINT.anchor",anchor)
    ] {
        fs::write(output.join(name),text)
            .map_err(|e| format!("write checkpoint {name}: {e}"))?;
    }
    for (name, bytes) in converted_files {
        fs::write(output.join(&name), bytes)
            .map_err(|e| format!("write converted weights {name}: {e}"))?;
    }
    Ok(MODELS.len())
}
pub fn verify(models: &Path, source_archive: &Path, output: &Path) -> Result<usize, String> {
    let root = source_digest(source_archive)?;
    let (hbp,hbi,anchor,converted_files) = create(models, &root, source_archive)?;
    for (name,text) in [
        ("PNP_CHECKPOINT.hbp",hbp),("PNP_CHECKPOINT.hbi",hbi),("PNP_CHECKPOINT.anchor",anchor)
    ] {
        if fs::read(output.join(name)).map_err(|e| format!("checkpoint read: {e}"))?
            != text.as_bytes()
        {
            return Err(format!("checkpoint evidence mismatch: {name}"));
        }
    }
    for (name, expected) in converted_files {
        if fs::read(output.join(&name))
            .map_err(|e| format!("converted weight file absent {name}: {e}"))?
            != expected
        {
            return Err(format!("converted weight bytes differ: {name}"));
        }
    }
    Ok(MODELS.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary32_conversion_is_exact_for_integer_halves_and_rejects_nonfinite() {
        assert_eq!(binary32_to_q20(0x3f80_0000), Ok(1_048_576));
        assert_eq!(binary32_to_q20(0xbf80_0000), Ok(-1_048_576));
        assert_eq!(binary32_to_q20(0x3f00_0000), Ok(524_288));
        assert_eq!(binary32_to_q20(0x0000_0000), Ok(0));
        assert_eq!(binary32_to_q20(0x8000_0000), Ok(0));
        assert!(binary32_to_q20(0x7f80_0000).is_err());
        assert!(binary32_to_q20(0x7fc0_0000).is_err());
        assert!(binary32_to_q20(0x7f7f_ffff).is_err());
        assert!(binary32_to_q20(0x7f00_0000).is_err());
        // Quarter Q20 unit rounds to zero, exact half unit uses even tie.
        assert_eq!(binary32_to_q20(0x3480_0000), Ok(0));
        assert_eq!(binary32_to_q20(0x3500_0000), Ok(0));
        assert_eq!(binary32_to_q20(0x3580_0000), Ok(1));
    }
    #[test]
    fn crc32_has_independent_standard_vector() {
        assert_eq!(checksum_crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(checksum_crc32(b""), 0);
    }
    #[test]
    fn independent_standard_zip_round_trip_and_crc_tamper_rejection() {
        fn write16(bytes: &mut Vec<u8>, value: u16) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        fn write32(bytes: &mut Vec<u8>, value: u32) {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let name = b"sample/data.pkl";
        let payload = b"sample-original-weights";
        let crc = checksum_crc32(payload);
        let mut archive = Vec::new();
        write32(&mut archive, 0x0403_4b50);
        write16(&mut archive, 20);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write32(&mut archive, crc);
        write32(&mut archive, payload.len() as u32);
        write32(&mut archive, payload.len() as u32);
        write16(&mut archive, name.len() as u16);
        write16(&mut archive, 0);
        archive.extend_from_slice(name);
        let payload_index = archive.len();
        archive.extend_from_slice(payload);
        let central_start = archive.len();
        write32(&mut archive, 0x0201_4b50);
        write16(&mut archive, 20);
        write16(&mut archive, 20);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write32(&mut archive, crc);
        write32(&mut archive, payload.len() as u32);
        write32(&mut archive, payload.len() as u32);
        write16(&mut archive, name.len() as u16);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write32(&mut archive, 0);
        write32(&mut archive, 0);
        archive.extend_from_slice(name);
        let central_len = archive.len() - central_start;
        write32(&mut archive, 0x0605_4b50);
        write16(&mut archive, 0);
        write16(&mut archive, 0);
        write16(&mut archive, 1);
        write16(&mut archive, 1);
        write32(&mut archive, central_len as u32);
        write32(&mut archive, central_start as u32);
        write16(&mut archive, 0);
        let decoded = read_zip(&archive).expect("independent ZIP");
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].payload, payload);
        archive[payload_index] ^= 1;
        assert!(read_zip(&archive).is_err());
    }

    #[test]
    fn nine_distinct_checkpoint_pins_are_in_source_ledger() {
        let mut ids = BTreeSet::new();
        for model in MODELS {
            let (id, blob) = find_pin(model).expect("Git-pinned model");
            assert!(ids.insert(id));
            assert_eq!(blob.len(), 40);
        }
        assert_eq!(ids.len(), 9);
    }
}
