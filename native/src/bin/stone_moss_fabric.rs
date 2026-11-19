//! Stone-to-Moss B3 A/B/P exact source archive and 2-of-3 recovery.
//! Rust 1.81; integer-only; pinned upstream source is independently enforced by CI.
//! Source-content SHA proves bytes against an expected root, not physical authorship.
#![forbid(unsafe_code)]

use pnp_native::sha256::hash_bytes;
use std::{env, fs, path::Path, process};

const MAGIC: &[u8] = b"SMF_ARCHIVE_V1\n";
const SHARD_MAGIC: &[u8] = b"SMF_SHARD_V1\n";
const WIDTH: usize = 256;
const STRIPE: usize = WIDTH * 2;
const NAMES: [&str; 6] = [
    "behcs/alphabet.json",
    "behcs/cascade.ts",
    "behcs/third-cascade.ts",
    "behcs/word-graph.hbp",
    "gnn/gslgnn_w9_3_seq47_v2.pt",
    "gnn/gslgnn_w9_3_seq47_v2_manifest.json",
];

type Result<T> = std::result::Result<T, String>;

fn require(ok: bool, reason: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(reason.to_owned()) }
}
fn take<'a>(raw: &'a [u8], offset: &mut usize, n: usize) -> Result<&'a [u8]> {
    let end = offset.checked_add(n).ok_or("offset overflow")?;
    require(end <= raw.len(), "truncated archive")?;
    let piece = &raw[*offset..end];
    *offset = end;
    Ok(piece)
}
fn num16(raw: &[u8], pos: &mut usize) -> Result<usize> {
    let v = take(raw, pos, 2)?;
    Ok(usize::from(u16::from_be_bytes([v[0], v[1]])))
}
fn num64(raw: &[u8], pos: &mut usize) -> Result<usize> {
    let v = take(raw, pos, 8)?;
    usize::try_from(u64::from_be_bytes(v.try_into().map_err(|_| "bad integer")?))
        .map_err(|_| "length overflows usize".to_owned())
}
fn archive(records: &[(&str, Vec<u8>)]) -> Result<Vec<u8>> {
    require(records.len() == NAMES.len(), "record count")?;
    let mut data = Vec::new();
    data.extend_from_slice(MAGIC);
    data.extend_from_slice(&(records.len() as u16).to_be_bytes());
    for (i, (name, raw)) in records.iter().enumerate() {
        require(*name == NAMES[i], "noncanonical record order")?;
        require(name.len() <= u16::MAX as usize, "name too long")?;
        require(raw.len() <= 2_000_000, "bounded source record exceeded")?;
        let digest = hash_bytes(raw);
        data.extend_from_slice(&(name.len() as u16).to_be_bytes());
        data.extend_from_slice(&(raw.len() as u64).to_be_bytes());
        data.extend_from_slice(digest.as_bytes());
        data.extend_from_slice(name.as_bytes());
        data.extend_from_slice(raw);
    }
    Ok(data)
}
fn verify_archive(data: &[u8]) -> Result<usize> {
    let mut pos = 0;
    require(take(data, &mut pos, MAGIC.len())? == MAGIC, "archive magic")?;
    require(num16(data, &mut pos)? == NAMES.len(), "archive record count")?;
    for expected in NAMES {
        let len = num16(data, &mut pos)?;
        let size = num64(data, &mut pos)?;
        require(size <= 2_000_000, "oversized source record")?;
        let digest = take(data, &mut pos, 64)?;
        let name = take(data, &mut pos, len)?;
        require(name == expected.as_bytes(), "record name/order changed")?;
        let raw = take(data, &mut pos, size)?;
        require(hash_bytes(raw).as_bytes() == digest, "record SHA mismatch")?;
    }
    require(pos == data.len(), "unregistered trailing source bytes")?;
    Ok(pos)
}
fn produce(words: &Path, models: &Path, graph: &Path) -> Result<Vec<u8>> {
    let files = [
        words.join("data/behcs/codex/alphabet.json"),
        words.join("packages/deep-wave/src/cascade.ts"),
        words.join("packages/deep-wave/src/third-cascade.ts"),
        graph.to_path_buf(),
        models.join("models/gslgnn_w9_3_seq47_v2.pt"),
        models.join("manifests/gslgnn_w9_3_seq47_v2_manifest.json"),
    ];
    let records = NAMES.iter().zip(files.iter())
        .map(|(&name, p)| fs::read(p).map(|raw| (name, raw))
            .map_err(|e| format!("source {}: {e}", p.display())))
        .collect::<Result<Vec<_>>>()?;
    let result = archive(&records)?;
    verify_archive(&result)?;
    Ok(result)
}
fn stripes(len: usize) -> Result<usize> {
    require(len > 0 && len <= 12_000_000, "invalid bounded archive length")?;
    Ok(len.div_ceil(STRIPE))
}
fn key_ok(s: &str) -> bool {
    !s.is_empty() && s.len() <= 200 &&
        s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
#[derive(Debug, Clone)]
struct Shard {
    lane: u8,
    key: String,
    root: String,
    len: usize,
    body: Vec<u8>,
}
fn shard(data: &[u8], lane: u8, key: &str) -> Result<Vec<u8>> {
    verify_archive(data)?;
    require(key_ok(key), "invalid run identity")?;
    require(matches!(lane, b'A' | b'B' | b'P'), "invalid lane")?;
    let count = stripes(data.len())?;
    let mut body = vec![0_u8; count * WIDTH];
    for i in 0..count {
        let start = i * STRIPE;
        for j in 0..WIDTH {
            let a = data.get(start + j).copied().unwrap_or(0);
            let b = data.get(start + WIDTH + j).copied().unwrap_or(0);
            body[i * WIDTH + j] = match lane {
                b'A' => a, b'B' => b, b'P' => a ^ b,
                _ => return Err("unknown lane".into()),
            };
        }
    }
    let root = hash_bytes(data);
    let head = format!(
        "RUN|key={key}|lane={}|root={root}|bytes={}|stripes={count}|width=256\n",
        char::from(lane), data.len(),
    );
    let mut out = Vec::new();
    out.extend_from_slice(SHARD_MAGIC);
    out.extend_from_slice(head.as_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}
fn parse_shard(bytes: &[u8]) -> Result<Shard> {
    require(bytes.starts_with(SHARD_MAGIC), "shard magic")?;
    let rest = &bytes[SHARD_MAGIC.len()..];
    let end = rest.iter().position(|b| *b == b'\n').ok_or("missing shard header")?;
    require(end <= 400, "shard header too long")?;
    let text = std::str::from_utf8(&rest[..end]).map_err(|_| "shard header UTF8")?;
    let fields = text.split('|').collect::<Vec<_>>();
    require(fields.len() == 7 && fields[0] == "RUN", "shard header fields")?;
    let val = |index: usize, prefix: &str| -> Result<&str> {
        fields[index].strip_prefix(prefix).ok_or("wrong shard field")
            .map_err(str::to_owned)
    };
    let key = val(1, "key=")?;
    require(key_ok(key), "invalid shard run key")?;
    let lane = val(2, "lane=")?.as_bytes();
    require(lane.len() == 1 && matches!(lane[0], b'A' | b'B' | b'P'), "invalid shard lane")?;
    let root = val(3, "root=")?;
    require(root.len() == 64 && root.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "invalid source root")?;
    let len = val(4, "bytes=")?.parse::<usize>().map_err(|_| "invalid archive bytes")?;
    let count = val(5, "stripes=")?.parse::<usize>().map_err(|_| "invalid stripes")?;
    require(val(6, "width=")? == "256", "invalid shard width")?;
    require(count == stripes(len)?, "incorrect stripe count")?;
    let body = &rest[end + 1..];
    require(body.len() == count * WIDTH, "shard payload truncated or extended")?;
    Ok(Shard { lane: lane[0], key: key.into(), root: root.into(), len, body: body.to_vec() })
}
fn recover(a: &[u8], b: &[u8], expected_key: &str) -> Result<Vec<u8>> {
    let s = parse_shard(a)?;
    let t = parse_shard(b)?;
    require(key_ok(expected_key), "invalid expected run key")?;
    require(s.lane != t.lane, "duplicate shard lane")?;
    require(s.key == t.key && s.key == expected_key, "cross-run or replayed shard")?;
    require(s.root == t.root && s.len == t.len, "inconsistent bundle source root")?;
    require(s.body.len() == t.body.len(), "shard length disagreement")?;
    let mut data = Vec::with_capacity(stripes(s.len)? * STRIPE);
    for i in 0..stripes(s.len)? {
        let mut half_a = [0_u8; WIDTH];
        let mut half_b = [0_u8; WIDTH];
        for j in 0..WIDTH {
            let x = s.body[i * WIDTH + j];
            let y = t.body[i * WIDTH + j];
            match (s.lane, t.lane) {
                (b'A', b'B') => { half_a[j] = x; half_b[j] = y; }
                (b'B', b'A') => { half_a[j] = y; half_b[j] = x; }
                (b'A', b'P') => { half_a[j] = x; half_b[j] = x ^ y; }
                (b'P', b'A') => { half_a[j] = y; half_b[j] = y ^ x; }
                (b'B', b'P') => { half_b[j] = x; half_a[j] = x ^ y; }
                (b'P', b'B') => { half_b[j] = y; half_a[j] = y ^ x; }
                _ => return Err("invalid recovery combination".into()),
            }
        }
        data.extend_from_slice(&half_a);
        data.extend_from_slice(&half_b);
    }
    data.truncate(s.len);
    require(hash_bytes(&data) == s.root, "recovered archive source SHA mismatch")?;
    verify_archive(&data)?;
    Ok(data)
}
fn execute() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("pack") if args.len() == 5 => {
            let raw = produce(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3]))?;
            fs::write(&args[4], &raw).map_err(|e| e.to_string())?;
            println!("SMF_PACK|records=6|bytes={}|sha256={}|status=PASS|json=0", raw.len(), hash_bytes(&raw));
        }
        Some("shard") if args.len() == 5 => {
            let raw = fs::read(&args[1]).map_err(|e| e.to_string())?;
            let bytes = shard(&raw, args[2].as_bytes().first().copied().ok_or("missing lane")?, &args[3])?;
            fs::write(&args[4], &bytes).map_err(|e| e.to_string())?;
            println!("SMF_SHARD|lane={}|archive_sha256={}|artifact_sha256={}|bytes={}|status=PASS|json=0",
                args[2], hash_bytes(&raw), hash_bytes(&bytes), bytes.len());
        }
        Some("restore") if args.len() == 5 => {
            let first = fs::read(&args[1]).map_err(|e| e.to_string())?;
            let second = fs::read(&args[2]).map_err(|e| e.to_string())?;
            let restored = recover(&first, &second, &args[3])?;
            fs::write(&args[4], &restored).map_err(|e| e.to_string())?;
            println!("SMF_RESTORE|records=6|bytes={}|sha256={}|status=PASS|json=0",
                restored.len(), hash_bytes(&restored));
        }
        _ => return Err("usage: pack WORDS MODELS WORD_GRAPH OUT | shard ARCHIVE A/B/P RUN_KEY OUT | restore SHARD1 SHARD2 RUN_KEY OUT".into()),
    }
    Ok(())
}
fn main() {
    if let Err(e) = execute() {
        eprintln!("SMF_HOLD|reason={e}|status=HOLD|json=0");
        process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn mock() -> Vec<u8> {
        let entries: Vec<_> = NAMES.iter().enumerate()
            .map(|(i, &name)| (name, (0..(1000+i*23))
                .map(|j| ((j * 7 + i) % 256) as u8).collect::<Vec<u8>>()))
            .collect();
        archive(&entries).unwrap()
    }
    #[test]
    fn all_two_of_three_restore_and_one_is_insufficient() {
        let src = mock();
        let parts = [b'A', b'B', b'P'].map(|p| shard(&src, p, "run-42").unwrap());
        for a in 0..3 {
            for b in a+1..3 {
                assert_eq!(recover(&parts[a], &parts[b], "run-42").unwrap(), src);
            }
            assert!(recover(&parts[a], &parts[a], "run-42").is_err());
        }
    }
    #[test]
    fn fail_closed_against_canonical_and_epoch_faults() {
        let src = mock();
        let a = shard(&src, b'A', "run-42").unwrap();
        let b = shard(&src, b'B', "run-42").unwrap();
        let stale = shard(&src, b'P', "run-41").unwrap();
        assert!(recover(&a, &stale, "run-42").is_err());
        let mut corrupted = b.clone();
        *corrupted.last_mut().unwrap() ^= 1;
        assert!(recover(&a, &corrupted, "run-42").is_err());
        let mut extra = b.clone();
        extra.push(0);
        assert!(recover(&a, &extra, "run-42").is_err());
        let mut bad_record = src.clone();
        *bad_record.last_mut().unwrap() ^= 1;
        assert!(verify_archive(&bad_record).is_err());
        let forged = shard(&src, b'B', "run-11").unwrap();
        assert!(recover(&a, &forged, "run-42").is_err());
        let mut changed_root = b;
        let i = changed_root.windows(5).position(|w| w == b"root=").unwrap() + 5;
        changed_root[i] = b'f';
        assert!(recover(&a, &changed_root, "run-42").is_err());
    }
    #[test]
    fn source_archive_rejects_extra_bytes_and_changed_order() {
        let src = mock();
        assert_eq!(verify_archive(&src).unwrap(), src.len());
        let mut more = src.clone();
        more.push(0);
        assert!(verify_archive(&more).is_err());
        let mut corrupt = src;
        corrupt[MAGIC.len()] = 0;
        assert!(verify_archive(&corrupt).is_err());
    }
}
