//! Original trained-source inventory, native HBI/HBP receipts, and
//! SHA-256 / SH / linked HASH verification. Sources are read, never executed.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::sha256::hash_bytes;

const PINS: &str = include_str!("../../policy/TRAINED_SOURCE_PINS.hbi");
const HEADER_HBP: &str = "HBPHEADER|schema=pnp.gnn.source.v1|seal=HBI-HBP-SHA-SH-HASH|traversal=HBI->HBP->SHA->SH->HASH|json=0\n";
const HEADER_HBI: &str = "HBIHEADER|schema=pnp.gnn.offset.v1|for=PNP_TRAINED_SOURCE.hbp|seal=HBI-HBP-SHA-SH-HASH|traversal=HBI->HBP->SHA->SH->HASH|json=0\n";

struct Pin<'a> {
    id: &'a str,
    repo: &'a str,
    commit: &'a str,
    path: &'a str,
    blob: &'a str,
    expected_sha: &'a str,
}

fn fields(line: &str) -> Result<BTreeMap<&str, &str>, String> {
    let mut out = BTreeMap::new();
    for part in line.split('|').skip(1) {
        let (key, value) = part.split_once('=').ok_or("invalid HBI/HBP field")?;
        if key.is_empty() || value.is_empty() || out.insert(key, value).is_some() {
            return Err("empty or duplicate HBI/HBP field".to_owned());
        }
    }
    Ok(out)
}

fn field<'a>(map: &'a BTreeMap<&str, &str>, key: &str) -> Result<&'a str, String> {
    map.get(key).copied().ok_or_else(|| format!("missing field {key}"))
}

fn is_digest(value: &str, length: usize) -> bool {
    value.len() == length
        && value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn pins() -> Result<Vec<Pin<'static>>, String> {
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    let mut lines = PINS.lines();
    if lines.next() != Some("HBIHEADER|schema=pnp.gnn.source-pins.v1|seal=HBI-HBP-SHA-SH-HASH|traversal=HBI->HBP->SHA->SH->HASH|status=SOURCE_PINNED|json=0") {
        return Err("unexpected original source HBI schema".to_owned());
    }
    for line in lines {
        if !line.starts_with("HBIPIN|") {
            return Err("unexpected original source record".to_owned());
        }
        let f = fields(line)?;
        let pin = Pin {
            id: field(&f, "id")?,
            repo: field(&f, "repo")?,
            commit: field(&f, "commit")?,
            path: field(&f, "path")?,
            blob: field(&f, "git_blob")?,
            expected_sha: field(&f, "expected_sha256")?,
        };
        if !matches!(pin.repo, "trained" | "stage")
            || pin.path.starts_with('/')
            || pin.path.split('/').any(|part| part == ".." || part.is_empty())
            || !pin.path.bytes().all(|b| b.is_ascii_alphanumeric()
                || matches!(b, b'/' | b'.' | b'_' | b'-'))
            || !is_digest(pin.commit, 40) || !is_digest(pin.blob, 40)
            || (pin.expected_sha != "none" && !is_digest(pin.expected_sha, 64))
            || !seen.insert(pin.id)
        {
            return Err("invalid or duplicate original source pin".to_owned());
        }
        result.push(pin);
    }
    if result.len() != 23 {
        return Err("trained model provenance inventory must include 23 pins".to_owned());
    }
    Ok(result)
}

fn git(root: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C").arg(root).args(arguments)
        .output().map_err(|e| format!("git unavailable: {e}"))?;
    if !output.status.success() {
        return Err(format!("git source check failed: {}", arguments.join(" ")));
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_owned())
        .map_err(|_| "non-utf8 git output".to_owned())
}

fn sha(source: &str) -> String {
    hash_bytes(source.as_bytes())
}

/// Freeze all original sources against full Git commit + blob, plus the
/// manifest-declared checkpoint digests where available.
pub fn attest(trained: &Path, stage: &Path, out: &Path) -> Result<usize, String> {
    let pins = pins()?;
    fs::create_dir_all(out).map_err(|e| format!("archive directory: {e}"))?;
    let mut hbp = HEADER_HBP.to_owned();
    let mut hbi = HEADER_HBI.to_owned();
    let mut predecessor = sha("PNP-GNN-SOURCE-GENESIS-v1");

    for (position, pin) in pins.iter().enumerate() {
        let root = if pin.repo == "trained" { trained } else { stage };
        if git(root, &["rev-parse", "HEAD"])? != pin.commit {
            return Err(format!("original source commit mismatch: {}", pin.id));
        }
        let blob = git(root, &["hash-object", pin.path])?;
        if blob != pin.blob {
            return Err(format!("original Git blob mismatch: {}", pin.id));
        }
        let raw = fs::read(root.join(pin.path))
            .map_err(|e| format!("original source absent {}: {e}", pin.id))?;
        let digest = hash_bytes(&raw);
        if pin.expected_sha != "none" && pin.expected_sha != digest {
            return Err(format!("checkpoint SHA-256 mismatch: {}", pin.id));
        }
        let sha_status = if pin.expected_sha == "none" {
            "COMPUTED_GIT_PINNED"
        } else {
            "MATCHES_MANIFEST"
        };
        let sh = &digest[..16];
        let payload = format!(
            "MODEL_SOURCE|seq={}|id={}|repo={}|commit={}|path={}|git_blob={}|sha256={}|sh={}|sha_status={}|trained_port=HOLD|prev_hash={}",
            position + 1, pin.id, pin.repo, pin.commit, pin.path, pin.blob,
            digest, sh, sha_status, predecessor
        );
        let receipt_sha = sha(&payload);
        let chain_hash = sha(&format!("{}|{}", predecessor, receipt_sha));
        let row = format!(
            "{payload}|receipt_sha={receipt_sha}|hash={chain_hash}|json=0\n"
        );
        hbi.push_str(&format!(
            "HBI|seq={}|id={}|offset={}|len={}|hbp_sha256={}|sh={}|hash={}|status=VERIFIED|json=0\n",
            position + 1, pin.id, hbp.len(), row.len(), sha(&row), sh, chain_hash
        ));
        hbp.push_str(&row);
        predecessor = chain_hash;
    }

    let root_hbp = sha(&hbp);
    let root_hbi = sha(&hbi);
    let pins_sha = sha(PINS);
    let anchor_payload = format!(
        "SOURCE_ANCHOR|count={}|root_hbp_sha256={}|root_hbi_sha256={}|pins_sha256={}|chain_hash={}|seal=HBI-HBP-SHA-SH-HASH",
        pins.len(), root_hbp, root_hbi, pins_sha, predecessor
    );
    let anchor = format!(
        "{anchor_payload}|anchor_checksum={}|json=0\n", sha(&anchor_payload)
    );
    fs::write(out.join("PNP_TRAINED_SOURCE.hbp"), hbp)
        .map_err(|e| format!("write HBP: {e}"))?;
    fs::write(out.join("PNP_TRAINED_SOURCE.hbi"), hbi)
        .map_err(|e| format!("write HBI: {e}"))?;
    fs::write(out.join("PNP_TRAINED_SOURCE.anchor"), anchor)
        .map_err(|e| format!("write anchor: {e}"))?;
    Ok(pins.len())
}

/// Check HBI byte offsets, every HBP digest and chained predecessor, source
/// pin identity, and an archive root checksum against the compiled source pins.
pub fn verify(out: &Path) -> Result<usize, String> {
    let hbp = fs::read_to_string(out.join("PNP_TRAINED_SOURCE.hbp"))
        .map_err(|e| format!("read HBP: {e}"))?;
    let hbi = fs::read_to_string(out.join("PNP_TRAINED_SOURCE.hbi"))
        .map_err(|e| format!("read HBI: {e}"))?;
    let anchor = fs::read_to_string(out.join("PNP_TRAINED_SOURCE.anchor"))
        .map_err(|e| format!("read anchor: {e}"))?;
    if !hbp.starts_with(HEADER_HBP) || !hbi.starts_with(HEADER_HBI)
        || !hbp.ends_with('\n') || !hbi.ends_with('\n')
        || !anchor.ends_with("\n")
    {
        return Err("invalid source receipt header".to_owned());
    }
    let a_line = anchor.strip_suffix('\n').ok_or("unterminated anchor")?;
    if a_line.contains('\n') || a_line.contains('\r') {
        return Err("invalid anchor lines".to_owned());
    }
    let a = fields(a_line)?;
    let anchor_prefix = a_line.split_once("|anchor_checksum=")
        .ok_or("missing anchor digest")?.0;
    if field(&a, "anchor_checksum")? != sha(anchor_prefix)
        || field(&a, "root_hbp_sha256")? != sha(&hbp)
        || field(&a, "root_hbi_sha256")? != sha(&hbi)
        || field(&a, "pins_sha256")? != sha(PINS)
        || field(&a, "seal")? != "HBI-HBP-SHA-SH-HASH"
    {
        return Err("source archive SHA/anchor mismatch".to_owned());
    }
    let pin_list = pins()?;
    if field(&a, "count")? != pin_list.len().to_string() {
        return Err("source archive row count mismatch".to_owned());
    }
    let mut offset = HEADER_HBP.len();
    let mut predecessor = sha("PNP-GNN-SOURCE-GENESIS-v1");
    let hbp_lines: Vec<&str> = hbp[HEADER_HBP.len()..].split_inclusive('\n').collect();
    let hbi_lines: Vec<&str> = hbi[HEADER_HBI.len()..].split_inclusive('\n').collect();
    if hbp_lines.len() != pin_list.len() || hbi_lines.len() != pin_list.len() {
        return Err("HBI/HBP source record count mismatch".to_owned());
    }
    for (index, ((raw, index_row), pin)) in hbp_lines.iter()
        .zip(hbi_lines.iter()).zip(pin_list.iter()).enumerate()
    {
        if !raw.ends_with('\n') || !index_row.ends_with('\n') {
            return Err("unterminated HBI/HBP source line".to_owned());
        }
        let hbp_line = raw.strip_suffix('\n').ok_or("HBP newline")?;
        let hbi_line = index_row.strip_suffix('\n').ok_or("HBI newline")?;
        let b = fields(hbp_line)?;
        let i = fields(hbi_line)?;
        let payload = hbp_line.split_once("|receipt_sha=")
            .ok_or("missing HBP receipt")?.0;
        let receipt_sha = sha(payload);
        let expected_hash = sha(&format!("{}|{}", predecessor, receipt_sha));
        let seq = (index + 1).to_string();
        if !hbp_line.starts_with("MODEL_SOURCE|")
            || !hbi_line.starts_with("HBI|")
            || field(&b, "seq")? != seq
            || field(&i, "seq")? != seq
            || field(&b, "id")? != pin.id || field(&i, "id")? != pin.id
            || field(&b, "repo")? != pin.repo
            || field(&b, "commit")? != pin.commit
            || field(&b, "path")? != pin.path
            || field(&b, "git_blob")? != pin.blob
            || !is_digest(field(&b, "sha256")?, 64)
            || field(&b, "sh")? != &field(&b, "sha256")?[..16]
            || field(&i, "sh")? != field(&b, "sh")?
            || field(&b, "prev_hash")? != predecessor
            || field(&b, "receipt_sha")? != receipt_sha
            || field(&b, "hash")? != expected_hash
            || field(&i, "hash")? != expected_hash
            || field(&i, "hbp_sha256")? != sha(raw)
            || field(&i, "offset")? != offset.to_string()
            || field(&i, "len")? != raw.len().to_string()
            || field(&i, "status")? != "VERIFIED"
            || field(&b, "trained_port")? != "HOLD"
            || field(&b, "json")? != "0" || field(&i, "json")? != "0"
        {
            return Err(format!("source receipt chain rejected at {}", pin.id));
        }
        if pin.expected_sha != "none" && field(&b, "sha256")? != pin.expected_sha {
            return Err(format!("source manifest digest mismatch at {}", pin.id));
        }
        predecessor = expected_hash;
        offset += raw.len();
    }
    if field(&a, "chain_hash")? != predecessor {
        return Err("source receipt terminal HASH mismatch".to_owned());
    }
    Ok(pin_list.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entire_source_pin_inventory_is_well_formed() {
        let all = pins().expect("source pin ledger");
        assert_eq!(all.len(), 23);
        assert_eq!(all.iter().filter(|p| p.expected_sha != "none").count(), 2);
        assert_eq!(all.iter().filter(|p| p.path.ends_with(".pt")).count(), 9);
    }

    #[test]
    fn chain_witness_is_order_sensitive() {
        let first = sha("x|y");
        let second = sha("y|x");
        assert_ne!(first, second);
        assert!(is_digest(&first, 64));
    }
}
