//! Native Q-PRISM 256 <-> 1024 reversible representation rung.
//! Exact Rust 1.81, integer-only, backed by original source Git and HBI/HBP.
#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::sha256::hash_bytes;

const ORIGINAL_REPO: &str = "JesseBrown1980/Q-PRISM-human-organoid-neural-stream-as-a-high-dimensional-control";
const ORIGINAL_COMMIT: &str = "b4a21a25381d6920b7aab92c94c3c3e1420fc9e4";
const ORIGINAL_FILE: &str = "host8/qprism_cube_host8.rs";
const ORIGINAL_GIT_BLOB: &str = "992ffd793f6c98f4f03b33ff0d1154b52d0174b0";
const PIN: &str = include_str!("../../policy/PRISM_SOURCE_PIN.hbi");
const HEADER: &str = "HBPHEADER|schema=pnp.prism2561024.v1|seal=HBI-HBP-SHA-SH-HASH|operator=HELD|json=0\n";
const INDEX: &str = "HBIHEADER|schema=pnp.prism2561024.index.v1|for=PNP_PRISM.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const CUBE_SHA: &str = "fe28e4a1813ea87aa55d9e4aad3e8ac028a2142890e39b3ab8e5eb40ba4ef05e";
const CUBE_SYMBOL_SHA: &str = "9593d266e75e5494172c62f4ceaebc2c4bd9a9cb9476f528f9d71543f313d4ab";
const CASES: [&str; 15] = [
    "empty", "zero", "ones", "seq2", "seq3",
    "seq4", "seq5", "seq7", "seq10", "seq16",
    "seq31", "seq40", "seq63", "tuple480", "cube3200",
];

/// Canonical big-endian 8-bit digits to big-endian 10-bit glyphs.
pub fn encode(bytes: &[u8]) -> Vec<u16> {
    let mut result = Vec::with_capacity(bytes.len().saturating_mul(4).div_ceil(5));
    let (mut buffer, mut available) = (0_u32, 0_u32);
    for &byte in bytes {
        buffer = (buffer << 8) | u32::from(byte);
        available += 8;
        while available >= 10 {
            available -= 10;
            result.push(((buffer >> available) & 1023) as u16);
        }
        buffer &= (1_u32 << available) - 1;
    }
    if available > 0 {
        result.push((buffer << (10 - available)) as u16);
    }
    result
}

/// Checked inverse. Rejects bad glyphs, wrong lengths and nonzero padding.
pub fn decode(glyphs: &[u16], nbytes: usize) -> Result<Vec<u8>, &'static str> {
    let expected = nbytes.checked_mul(8)
        .and_then(|v| v.checked_add(9)).ok_or("size overflow")? / 10;
    if glyphs.len() != expected {
        return Err("invalid glyph count for original byte length");
    }
    let mut bytes = Vec::with_capacity(nbytes);
    let (mut buffer, mut available) = (0_u32, 0_u32);
    for &glyph in glyphs {
        if glyph > 1023 {
            return Err("glyph outside BEHCS-1024 range");
        }
        buffer = (buffer << 10) | u32::from(glyph);
        available += 10;
        while available >= 8 && bytes.len() < nbytes {
            available -= 8;
            bytes.push(((buffer >> available) & 255) as u8);
        }
        buffer &= (1_u32 << available) - 1;
    }
    if bytes.len() != nbytes || buffer != 0 {
        return Err("noncanonical PRISM zero padding");
    }
    Ok(bytes)
}
fn encode_words(words: &[u16]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words.len() * 2);
    for &word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}
fn hex(data: &[u8]) -> String {
    const ALPH: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(data.len() * 2);
    for &b in data {
        result.push(char::from(ALPH[usize::from(b >> 4)]));
        result.push(char::from(ALPH[usize::from(b & 15)]));
    }
    result
}
fn case_data(name: &str) -> Option<Vec<u8>> {
    match name {
        "empty" => Some(Vec::new()),
        "zero" => Some(vec![0]),
        "ones" => Some(vec![255]),
        "cube3200" => Some((0_u32..3200).map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8).collect()),
        other => {
            let count = match other {
                "seq2" => 2, "seq3" => 3, "seq4" => 4, "seq5" => 5,
                "seq7" => 7, "seq10" => 10, "seq16" => 16,
                "seq31" => 31, "seq40" => 40, "seq63" => 63,
                "tuple480" => 480, _ => return None,
            };
            Some((0..count).map(|i| i as u8).collect())
        }
    }
}
fn source_sha(root: &Path) -> Result<String, String> {
    let pin_row = format!(
        "PRISM_PIN|id=qprism_host8|repo={ORIGINAL_REPO}|commit={ORIGINAL_COMMIT}|path={ORIGINAL_FILE}|git_blob={ORIGINAL_GIT_BLOB}|json=0\n"
    );
    if PIN != format!("HBIHEADER|schema=pnp.prism.original.v1|seal=HBI-HBP-SHA-SH-HASH|json=0\n{pin_row}") {
        return Err("original PRISM HBI pin changed".to_owned());
    }
    let git = |args: &[&str]| -> Result<String, String> {
        let output = Command::new("git").arg("-C").arg(root).args(args)
            .output().map_err(|e| format!("PRISM source Git unavailable: {e}"))?;
        if !output.status.success() {
            return Err("original PRISM checkout or blob rejected".to_owned());
        }
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_owned())
            .map_err(|_| "invalid Git output".to_owned())
    };
    if git(&["rev-parse","HEAD"])? != ORIGINAL_COMMIT
        || git(&["hash-object",ORIGINAL_FILE])? != ORIGINAL_GIT_BLOB
    {
        return Err("original PRISM commit/blob mismatch".to_owned());
    }
    let raw = fs::read(root.join(ORIGINAL_FILE)).map_err(|e| format!("read PRISM source: {e}"))?;
    Ok(hash_bytes(&raw))
}

type ReceiptBundle = (String, String, String);
fn make_bundle(root: &Path) -> Result<ReceiptBundle, String> {
    let original_sha = source_sha(root)?;
    let mut body = HEADER.to_owned();
    let mut index = INDEX.to_owned();
    let mut prev = hash_bytes(b"PNP-PRISM-256-1024-GENESIS-v1");
    for (i, name) in CASES.iter().enumerate() {
        let bytes = case_data(name).ok_or("unknown source test case")?;
        let symbols = encode(&bytes);
        if decode(&symbols,bytes.len()).map_err(str::to_owned)? != bytes {
            return Err("PRISM byte roundtrip mismatch".to_owned());
        }
        if symbols.len() != bytes.len().saturating_mul(8).div_ceil(10) {
            return Err("PRISM glyph length mismatch".to_owned());
        }
        let digest = hash_bytes(&bytes);
        let symbol_sha = hash_bytes(&encode_words(&symbols));
        if *name == "cube3200" && (digest != CUBE_SHA || symbol_sha != CUBE_SYMBOL_SHA) {
            return Err("original 3,200-byte PRISM cube golden digest mismatch".to_owned());
        }
        let symbols_preview = if symbols.len() <= 10 {
            symbols.iter().map(u16::to_string).collect::<Vec<_>>().join(",")
        } else {
            "large".to_owned()
        };
        let row_data = format!(
            "PRISM|seq={}|case={name}|original_commit={ORIGINAL_COMMIT}|original_blob={ORIGINAL_GIT_BLOB}|original_sha256={original_sha}|input_len={}|glyphs={}|input_sha256={digest}|symbols_sha256={symbol_sha}|sh={}|preview={symbols_preview}|roundtrip=EXACT|source_equivalence=BITSTREAM|operator=HELD|prev_hash={prev}",
            i + 1, bytes.len(), symbols.len(), &digest[..16]
        );
        let receipt_sha = hash_bytes(row_data.as_bytes());
        let chain = hash_bytes(format!("{prev}|{receipt_sha}").as_bytes());
        let row = format!("{row_data}|receipt_sha={receipt_sha}|hash={chain}|json=0\n");
        index.push_str(&format!(
            "HBI|seq={}|id={name}|offset={}|len={}|hbp_sha256={}|sh={}|hash={chain}|json=0\n",
            i + 1, body.len(), row.len(), hash_bytes(row.as_bytes()), &digest[..16]
        ));
        body.push_str(&row);
        prev = chain;
    }
    let anchor_data = format!(
        "PRISM_ANCHOR|cases={}|original_commit={ORIGINAL_COMMIT}|original_blob={ORIGINAL_GIT_BLOB}|original_sha256={original_sha}|source_hbi_sha256={}|hbp_sha256={}|hbi_sha256={}|hash={prev}|seal=HBI-HBP-SHA-SH-HASH",
        CASES.len(), hash_bytes(PIN.as_bytes()),
        hash_bytes(body.as_bytes()), hash_bytes(index.as_bytes())
    );
    let anchor = format!(
        "{anchor_data}|checksum={}|json=0\n",
        hash_bytes(anchor_data.as_bytes())
    );
    Ok((body, index, anchor))
}
pub fn attest(root: &Path, output: &Path) -> Result<usize, String> {
    let (hbp,hbi,anchor) = make_bundle(root)?;
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    for (name,contents) in [
        ("PNP_PRISM.hbp",hbp),("PNP_PRISM.hbi",hbi),("PNP_PRISM.anchor",anchor)
    ] {
        fs::write(output.join(name),contents)
            .map_err(|e| format!("write original PRISM receipt: {e}"))?;
    }
    Ok(CASES.len())
}
pub fn verify(root: &Path, output: &Path) -> Result<usize, String> {
    let (hbp,hbi,anchor) = make_bundle(root)?;
    for (name,expected) in [
        ("PNP_PRISM.hbp",hbp),("PNP_PRISM.hbi",hbi),("PNP_PRISM.anchor",anchor)
    ] {
        if fs::read(output.join(name)).map_err(|e| format!("read PRISM {name}: {e}"))?
            != expected.as_bytes()
        {
            return Err(format!("changed PRISM evidence: {name}"));
        }
    }
    Ok(CASES.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independently_known_original_bitstream_examples() {
        assert_eq!(encode(&[]), Vec::<u16>::new());
        assert_eq!(encode(&[0]), [0]);
        assert_eq!(encode(&[255]), [1020]);
        assert_eq!(encode(&[1,2,3,4,5]), [4,32,193,5]);
        assert_eq!(encode(&[0,1,2,3,4,5,6,7,8,9]), [0,16,128,772,20,96,450,9]);
        assert_eq!(encode(&[255;5]), [1023;4]);
        assert_eq!(decode(&[4,32,193,5],5), Ok(vec![1,2,3,4,5]));
        assert_eq!(decode(&[1020],1), Ok(vec![255]));
    }
    #[test]
    fn original_3200_byte_cube_sha_and_glyph_digest_match() {
        let original = case_data("cube3200").expect("cube");
        let encoded = encode(&original);
        assert_eq!(original.len(), 3200);
        assert_eq!(encoded.len(), 2560);
        assert_eq!(hash_bytes(&original), CUBE_SHA);
        assert_eq!(hash_bytes(&encode_words(&encoded)), CUBE_SYMBOL_SHA);
        assert_eq!(decode(&encoded,3200), Ok(original));
    }
    #[test]
    fn all_two_byte_patterns_and_padding_controls() {
        for value in 0_u32..=65535 {
            let bytes = (value as u16).to_be_bytes();
            let encoded = encode(&bytes);
            assert_eq!(decode(&encoded,2), Ok(bytes.to_vec()));
        }
        assert!(decode(&[1024],1).is_err());
        assert!(decode(&[1021],1).is_err()); // unused padding changed
        assert!(decode(&[4,32,193,5],4).is_err()); // wrong output length
        assert!(decode(&[],1).is_err());
        assert!(decode(&[0],0).is_err());
        assert!(decode(&[0],usize::MAX).is_err());
    }
    #[test]
    fn all_partial_block_sizes_and_invariant_lengths() {
        for n in 0..=101 {
            let input: Vec<u8> = (0..n).map(|i| (i * 17) as u8).collect();
            let encoded = encode(&input);
            assert_eq!(encoded.len(), n * 8_usize / 10 + usize::from(n * 8 % 10 != 0));
            assert_eq!(decode(&encoded,n), Ok(input));
        }
    }
}
