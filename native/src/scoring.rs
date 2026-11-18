//! Source-matched original SHA baseline and reverse-gain primitive.
//! Trained neural logits remain HOLD; this native integer plane exactly
//! replays the deterministic parts of the pinned historical score source.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::sha256::hash_bytes;
use crate::source;

const SCORE_BLOB: &str = "b101c0989de3af0ca41db0eac1293c758d7e5296";
const SOURCE_PINS: &str = include_str!("../../policy/TRAINED_SOURCE_PINS.hbi");
const HEADER_INPUT: &str = "HBIHEADER|schema=pnp.score.cases.v1|source_blob=b101c0989de3af0ca41db0eac1293c758d7e5296|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HEADER_HBP: &str = "HBPHEADER|schema=pnp.source.score.v1|seal=HBI-HBP-SHA-SH-HASH|mode=sha_baseline_reverse_gain|trained=HOLD|json=0\n";
const HEADER_HBI: &str = "HBIHEADER|schema=pnp.source.score.index.v1|for=PNP_SCORE.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Baseline {
    pub combined_sha256: String,
    pub score_q10000: u16,
    pub reverse_risk_q10000: u16,
    pub promoted: bool,
}

pub fn sha_baseline(pid: &str, content: &str) -> Baseline {
    // Original asolaria-score.mjs: sha16(`${pid}|${content}`);
    // parseInt(first 4 hex chars, 16) % 1000 / 1000.
    let digest = hash_bytes(format!("{pid}|{content}").as_bytes());
    let first = u16::from_str_radix(&digest[..4], 16).expect("hex SHA prefix");
    let score_q10000 = (first % 1000) * 10;
    let reverse_risk_q10000 = 10_000 - score_q10000;
    Baseline {
        combined_sha256: digest,
        score_q10000,
        reverse_risk_q10000,
        promoted: score_q10000 >= 7_200 && reverse_risk_q10000 <= 2_800,
    }
}

fn fields(line: &str) -> Result<BTreeMap<&str, &str>, String> {
    let mut result = BTreeMap::new();
    for pair in line.split('|').skip(1) {
        let (key, value) = pair.split_once('=').ok_or("noncanonical input field")?;
        if key.is_empty() || value.is_empty() || result.insert(key, value).is_some() {
            return Err("empty or duplicate score HBI field".to_owned());
        }
    }
    Ok(result)
}

fn field<'a>(map: &'a BTreeMap<&str, &str>, key: &str) -> Result<&'a str, String> {
    map.get(key).copied().ok_or_else(|| format!("missing field {key}"))
}

fn decode_hex(source: &str) -> Result<String, String> {
    if source == "none" {
        return Ok(String::new());
    }
    if source.is_empty() || source.len() % 2 != 0 || source.len() > 131_072
        || !source.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("invalid canonical hex source".to_owned());
    }
    let mut bytes = Vec::with_capacity(source.len() / 2);
    for pair in source.as_bytes().chunks_exact(2) {
        let part = std::str::from_utf8(pair).map_err(|_| "invalid hex bytes")?;
        bytes.push(u8::from_str_radix(part, 16).map_err(|_| "invalid hex digit")?);
    }
    String::from_utf8(bytes).map_err(|_| "input is not UTF-8 text".to_owned())
}

#[derive(Clone, Debug)]
struct Case {
    id: String,
    pid: String,
    content: String,
    original_record: String,
}

fn cases(source: &str) -> Result<Vec<Case>, String> {
    let rest = source.strip_prefix(HEADER_INPUT).ok_or("invalid score input header")?;
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for raw in rest.split_inclusive('\n') {
        let line = raw.strip_suffix('\n').ok_or("unterminated scoring input")?;
        if !line.starts_with("CASE|") {
            return Err("unknown score input row".to_owned());
        }
        let f = fields(line)?;
        let id = field(&f, "id")?;
        if id.is_empty()
            || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || !seen.insert(id.to_owned())
            || field(&f, "json")? != "0"
        {
            return Err("invalid or duplicated score input id".to_owned());
        }
        let pid = decode_hex(field(&f, "pid_hex")?)?;
        let content = decode_hex(field(&f, "content_hex")?)?;
        let out = sha_baseline(&pid, &content);
        let expected_baseline = field(&f, "baseline_q")?;
        let expected_reverse = field(&f, "reverse_q")?;
        let expected_mark = field(&f, "mark")?;
        if (expected_baseline != "none" && expected_baseline != out.score_q10000.to_string())
            || (expected_reverse != "none" && expected_reverse != out.reverse_risk_q10000.to_string())
            || (expected_mark != "none"
                && expected_mark != if out.promoted { "GENIUS" } else { "MISTAKE" })
        {
            return Err(format!("golden reference comparison failed at {id}"));
        }
        result.push(Case {
            id: id.to_owned(),
            pid,
            content,
            original_record: raw.to_owned(),
        });
    }
    if result.is_empty() || result.len() > 1024 {
        return Err("invalid scoring fixture size".to_owned());
    }
    Ok(result)
}

fn source_root(source_archive: &Path) -> Result<String, String> {
    if source::verify(source_archive)? != 23 {
        return Err("source HBI/HBP archive does not contain 23 pins".to_owned());
    }
    let expected = "HBIPIN|id=stage_score|repo=stage|commit=32f9670a648dbbd02d64524e91150efeec8e4d2c|path=pipeline/asolaria-score.mjs|git_blob=b101c0989de3af0ca41db0eac1293c758d7e5296|expected_sha256=none|kind=source|json=0";
    if !SOURCE_PINS.lines().any(|line| line == expected) || !expected.contains(SCORE_BLOB) {
        return Err("original scoring source blob pin has drifted".to_owned());
    }
    let source_hbp = fs::read(source_archive.join("PNP_TRAINED_SOURCE.hbp"))
        .map_err(|e| format!("read source bundle: {e}"))?;
    Ok(hash_bytes(&source_hbp))
}

fn digest(value: &str) -> String {
    hash_bytes(value.as_bytes())
}

fn render(input: &str, source_digest: &str) -> Result<(String, String, String, usize), String> {
    let rows = cases(input)?;
    let mut hbp = HEADER_HBP.to_owned();
    let mut hbi = HEADER_HBI.to_owned();
    let mut predecessor = digest("PNP-TERNARY-SCORE-GENESIS-v1");
    for (index, case) in rows.iter().enumerate() {
        let out = sha_baseline(&case.pid, &case.content);
        let mark = if out.promoted { "GENIUS" } else { "MISTAKE" };
        let request_sha = digest(&case.original_record);
        let p_sha = digest(&case.pid);
        let c_sha = digest(&case.content);
        let sh = &out.combined_sha256[..16];
        let data = format!(
            "SCORE|seq={}|id={}|source_blob={}|source_hbp_sha256={}|request_sha256={}|pid_sha256={}|content_sha256={}|baseline_sha256={}|sh={}|baseline_q={}|reverse_q={}|promoted={}|mark={}|trained_gnn=HOLD|prev_hash={}",
            index + 1, case.id, SCORE_BLOB, source_digest, request_sha, p_sha,
            c_sha, out.combined_sha256, sh, out.score_q10000,
            out.reverse_risk_q10000, u8::from(out.promoted), mark, predecessor
        );
        let receipt_sha = digest(&data);
        let chain_hash = digest(&format!("{}|{}", predecessor, receipt_sha));
        let final_row = format!(
            "{data}|receipt_sha={receipt_sha}|hash={chain_hash}|json=0\n"
        );
        hbi.push_str(&format!(
            "HBI|seq={}|id={}|offset={}|len={}|hbp_sha256={}|sh={}|hash={}|status=VERIFIED|json=0\n",
            index + 1, case.id, hbp.len(), final_row.len(),
            digest(&final_row), sh, chain_hash
        ));
        hbp.push_str(&final_row);
        predecessor = chain_hash;
    }
    let prefix = format!(
        "SCORE_ANCHOR|count={}|source_hbp_sha256={}|input_sha256={}|root_hbp_sha256={}|root_hbi_sha256={}|chain_hash={}|seal=HBI-HBP-SHA-SH-HASH",
        rows.len(), source_digest, digest(input), digest(&hbp), digest(&hbi), predecessor
    );
    let anchor = format!(
        "{prefix}|anchor_checksum={}|json=0\n",
        digest(&prefix)
    );
    Ok((hbp, hbi, anchor, rows.len()))
}

pub fn attest(input: &Path, sources: &Path, output: &Path) -> Result<usize, String> {
    let original = fs::read_to_string(input)
        .map_err(|e| format!("read source score input: {e}"))?;
    let source_digest = source_root(sources)?;
    let (hbp, hbi, anchor, count) = render(&original, &source_digest)?;
    fs::create_dir_all(output).map_err(|e| format!("scoring output directory: {e}"))?;
    fs::write(output.join("PNP_SCORE.hbp"), hbp)
        .map_err(|e| format!("write scoring HBP: {e}"))?;
    fs::write(output.join("PNP_SCORE.hbi"), hbi)
        .map_err(|e| format!("write scoring HBI: {e}"))?;
    fs::write(output.join("PNP_SCORE.anchor"), anchor)
        .map_err(|e| format!("write scoring anchor: {e}"))?;
    Ok(count)
}

pub fn verify(input: &Path, sources: &Path, output: &Path) -> Result<usize, String> {
    let original = fs::read_to_string(input)
        .map_err(|e| format!("read score reference: {e}"))?;
    let source_digest = source_root(sources)?;
    let (hbp, hbi, anchor, count) = render(&original, &source_digest)?;
    for (name, expected) in [
        ("PNP_SCORE.hbp", hbp),
        ("PNP_SCORE.hbi", hbi),
        ("PNP_SCORE.anchor", anchor),
    ] {
        let actual = fs::read(output.join(name))
            .map_err(|e| format!("read score archive {name}: {e}"))?;
        if actual != expected.as_bytes() {
            return Err(format!("scoring archive hash/offset mismatch: {name}"));
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_original_scoring_reference_vectors_match_integer_port() {
        let source = include_str!("../fixtures/SOURCE_SCORE_VECTORS.hbi");
        assert_eq!(cases(source).expect("independent SHA oracle").len(), 9);
        let out = sha_baseline("worker:-1", "-");
        assert_eq!(out.score_q10000, 8190);
        assert_eq!(out.reverse_risk_q10000, 1810);
        assert!(out.promoted);
        let no = sha_baseline("", "");
        assert_eq!(no.score_q10000, 1970);
        assert!(!no.promoted);
    }

    #[test]
    fn malformed_golden_reference_and_duplicate_ids_fail_closed() {
        let source = include_str!("../fixtures/SOURCE_SCORE_VECTORS.hbi");
        assert!(cases(&source.replace("baseline_q=8190", "baseline_q=8191")).is_err());
        assert!(cases(&source.replace("mark=GENIUS", "mark=MISTAKE")).is_err());
        assert!(cases(&(source.to_owned() + &source[HEADER_INPUT.len()..])).is_err());
        assert!(cases(&source.replace("pid_hex=6167", "pid_hex=xx67")).is_err());
    }

    #[test]
    fn new_unseen_source_inputs_are_scored_without_oracle_outputs() {
        let source = format!(
            "{HEADER_INPUT}CASE|id=unseen|pid_hex=6162|content_hex=630a|baseline_q=none|reverse_q=none|mark=none|json=0\\n"
        ).replace("\\n", "\n");
        let records = cases(&source).expect("unseen input");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].pid, "ab");
        assert_eq!(records[0].content, "c\n");
        let calculated = sha_baseline(&records[0].pid, &records[0].content);
        assert!(calculated.score_q10000 <= 9990);
    }

    #[test]
    fn baseline_input_serialization_ambiguity_is_bound_by_two_source_hashes() {
        let a = sha_baseline("a|b", "c");
        let b = sha_baseline("a", "b|c");
        assert_eq!(a.combined_sha256, b.combined_sha256);
        assert_ne!(digest("a|b"), digest("a"));
        assert_ne!(digest("c"), digest("b|c"));
    }
}
