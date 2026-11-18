//! Automatic post-GULP compaction and physical disposal of owned, temporary
//! event batches, after durable HBI/HBP/SHA/SH/HASH summary acceptance.
//! Original source files, trained weights and prior archive receipts are fixed.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::{fabric_gc, sha256::hash_bytes};

const INPUT_DIR: &str = "PNP_GULP_EPHEMERAL";
const INPUT_FILE: &str = "pending.events.hbp";
const HBP_NAME: &str = "PNP_GULP.hbp";
const HBI_NAME: &str = "PNP_GULP.hbi";
const ANCHOR_NAME: &str = "PNP_GULP.anchor";
const DISCARD_NAME: &str = "PNP_GULP_DISCARD.hbp";
const INPUT_HEADER: &str =
    "HBPHEADER|schema=pnp.gulp.owned-events.v1|seal=HBI-HBP-SHA-SH-HASH|source=OMNI_GULP_GC|json=0\n";
const OUTPUT_HEADER: &str =
    "HBPHEADER|schema=pnp.gulp.compact.v1|policy=AUTO_POST_GULP|seal=HBI-HBP-SHA-SH-HASH|raw=DISPOSABLE|manual_approval=NOT_REQUIRED|json=0\n";
const INDEX_HEADER: &str =
    "HBIHEADER|schema=pnp.gulp.compact.index.v1|for=PNP_GULP.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const MAX_BATCH: usize = 50 * 1024 * 1024;
const MAX_EVENTS: usize = 100_000;

fn sha(raw: &[u8]) -> String {
    hash_bytes(raw)
}
fn fields(line: &str) -> Result<BTreeMap<&str, &str>, String> {
    let mut out = BTreeMap::new();
    for part in line.split('|').skip(1) {
        let (key, value) = part.split_once('=').ok_or("invalid GULP field")?;
        if key.is_empty() || value.is_empty() || out.insert(key, value).is_some() {
            return Err("GULP duplicate or empty field".to_owned());
        }
    }
    Ok(out)
}
fn value<'a>(map: &BTreeMap<&'a str, &'a str>, key: &str) -> Result<&'a str, String> {
    map.get(key).copied().ok_or_else(|| format!("missing GULP field: {key}"))
}
fn checked_token(word: &str) -> bool {
    !word.is_empty() && word.len() <= 64 && word.bytes().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-')
    })
}
fn digest_valid(raw: &str) -> bool {
    raw.len() == 64 && raw.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn usize_field(map: &BTreeMap<&str, &str>, key: &str) -> Result<usize, String> {
    value(map, key)?.parse().map_err(|_| format!("invalid integer: {key}"))
}

#[derive(Debug)]
struct Compact {
    raw_digest: String,
    raw_bytes: usize,
    events: usize,
    by_kind: BTreeMap<String, usize>,
    by_glyph: BTreeMap<String, usize>,
}
fn summarize(raw: &[u8]) -> Result<Compact, String> {
    if raw.len() > MAX_BATCH {
        return Err("owned GULP input exceeds original 50 MiB rotation boundary".to_owned());
    }
    let txt = std::str::from_utf8(raw).map_err(|_| "GULP input must be UTF8")?;
    let rest = txt.strip_prefix(INPUT_HEADER).ok_or("unknown GULP event schema")?;
    if !rest.ends_with('\n') {
        return Err("unterminated GULP event input".to_owned());
    }
    let mut count = 0_usize;
    let (mut kind, mut glyph) = (BTreeMap::new(), BTreeMap::new());
    for line in rest.lines() {
        if !line.starts_with("EVENT|") || line.len() > 512 {
            return Err("malformed or oversized GULP event".to_owned());
        }
        let record = fields(line)?;
        if record.len() != 4 || value(&record, "json")? != "0" {
            return Err("unhandled GULP event fields".to_owned());
        }
        count = count.checked_add(1).ok_or("GULP count overflow")?;
        if count > MAX_EVENTS || value(&record, "seq")? != count.to_string() {
            return Err("GULP missing/reordered/oversized event sequence".to_owned());
        }
        let event_kind = value(&record, "kind")?;
        let event_glyph = value(&record, "glyph")?;
        if !checked_token(event_kind) || !checked_token(event_glyph) {
            return Err("invalid GULP event/glyph token".to_owned());
        }
        *kind.entry(event_kind.to_owned()).or_insert(0_usize) += 1;
        *glyph.entry(event_glyph.to_owned()).or_insert(0_usize) += 1;
    }
    if count == 0 {
        return Err("empty GULP event input: HOLD".to_owned());
    }
    Ok(Compact {
        raw_digest: sha(raw),
        raw_bytes: raw.len(),
        events: count,
        by_kind: kind,
        by_glyph: glyph,
    })
}

type CompactedReceipts = (String, String, String);
fn render(data: &Compact, upstream: &str) -> Result<CompactedReceipts, String> {
    if !digest_valid(upstream) {
        return Err("GAC/GC source HBP digest invalid".to_owned());
    }
    let mut hbp = OUTPUT_HEADER.to_owned();
    let mut hbi = INDEX_HEADER.to_owned();
    let mut previous = sha(b"PNP-GULP-RECLAIM-GENESIS-v1");
    let mut seq = 0_usize;
    let mut append = |label: &str, id: &str, body: &str| {
        seq += 1;
        let payload = format!(
            "{label}|seq={seq}|id={id}|{body}|prev_hash={previous}"
        );
        let receipt = sha(payload.as_bytes());
        let chain = sha(format!("{previous}|{receipt}").as_bytes());
        let row = format!(
            "{payload}|receipt_sha={receipt}|hash={chain}|json=0\n"
        );
        hbi.push_str(&format!(
            "HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={chain}|status=COMMITTED|json=0\n",
            hbp.len(), row.len(), sha(row.as_bytes()), &receipt[..16]
        ));
        hbp.push_str(&row);
        previous = chain;
    };
    append(
        "GULP_BATCH", "batch",
        &format!(
            "upstream_hbp_sha256={upstream}|raw_sha256={}|raw_bytes={}|events={}|kinds={}|glyphs={}|processed=ALL|raw_policy=DISCARD_AFTER_COMMIT",
            data.raw_digest, data.raw_bytes, data.events,
            data.by_kind.len(), data.by_glyph.len()
        ),
    );
    for (kind, amount) in &data.by_kind {
        append("GULP_KIND", kind, &format!("kind={kind}|count={amount}"));
    }
    for (glyph, amount) in &data.by_glyph {
        append("GULP_GLYPH", glyph, &format!("glyph={glyph}|count={amount}"));
    }
    append("GULP_GC", "decision", &format!(
        "raw_sha256={}|preserved_kinds={}|preserved_glyphs={}|delete_scope=OWNED_EPHEMERAL_ONLY|delete_eligible=1|manual_approval=NOT_REQUIRED",
        data.raw_digest, data.by_kind.len(), data.by_glyph.len()
    ));
    let anchor_payload = format!(
        "GULP_ANCHOR|rows={seq}|events={}|raw_sha256={}|upstream_hbp_sha256={upstream}|root_hbp_sha256={}|root_hbi_sha256={}|terminal_hash={previous}|seal=HBI-HBP-SHA-SH-HASH",
        data.events, data.raw_digest, sha(hbp.as_bytes()), sha(hbi.as_bytes())
    );
    let anchor = format!(
        "{anchor_payload}|anchor_checksum={}|json=0\n",
        sha(anchor_payload.as_bytes())
    );
    Ok((hbp,hbi,anchor))
}
fn write_exclusive(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|e| format!("cannot create unique GULP receipt {}: {e}",path.display()))?;
    file.write_all(bytes).map_err(|e| format!("GULP receipt write failed: {e}"))?;
    file.sync_all().map_err(|e| format!("GULP receipt flush failed: {e}"))?;
    Ok(())
}
fn sync_dir(dir: &Path) -> Result<(), String> {
    fs::File::open(dir).and_then(|file| file.sync_all())
        .map_err(|e| format!("GULP directory sync failure: {e}"))
}
fn upstream_root(gac: &Path, neural: &Path, omni: &Path, evidence: &Path)
    -> Result<String, String>
{
    if fabric_gc::verify(gac, neural, omni, evidence)? != 16 {
        return Err("GAC neural GC source audit not accepted".to_owned());
    }
    let raw = fs::read(evidence.join("PNP_FABRIC_GC.hbp"))
        .map_err(|e| format!("GAC source root missing: {e}"))?;
    Ok(sha(&raw))
}
fn controlled_raw_path(output: &Path) -> Result<std::path::PathBuf, String> {
    let meta = fs::symlink_metadata(output).map_err(|e| format!("output unavailable: {e}"))?;
    if !meta.file_type().is_dir() {
        return Err("GC output must be a directory".to_owned());
    }
    let scratch = output.join(INPUT_DIR);
    let root = fs::symlink_metadata(&scratch)
        .map_err(|e| format!("owned GULP input directory missing: {e}"))?;
    if !root.file_type().is_dir() {
        return Err("owned GULP directory is not an ordinary directory".to_owned());
    }
    let path = scratch.join(INPUT_FILE);
    let file = fs::symlink_metadata(&path)
        .map_err(|e| format!("owned GULP input missing: {e}"))?;
    if !file.file_type().is_file() || file.len() > MAX_BATCH as u64 {
        return Err("only bounded owned regular GULP input can be discarded".to_owned());
    }
    let names: Vec<_> = fs::read_dir(&scratch)
        .map_err(|e| format!("cannot inspect owned directory: {e}"))?
        .map(|entry| {
            entry.map(|dirent| dirent.file_name()).map_err(|e| e.to_string())
        }).collect::<Result<_,_>>()?;
    if names.len() != 1 || names[0] != INPUT_FILE {
        return Err("owned ephemeral scope includes other files: HOLD".to_owned());
    }
    Ok(path)
}
fn reconstruct_from_committed(hbp: &str, upstream: &str) -> Result<Compact, String> {
    let text = hbp.strip_prefix(OUTPUT_HEADER).ok_or("unknown GULP receipt header")?;
    let mut lines = text.lines();
    let first = lines.next().ok_or("GULP batch root absent")?;
    if !first.starts_with("GULP_BATCH|") {
        return Err("GULP root record missing".to_owned());
    }
    let f = fields(first)?;
    let raw_digest = value(&f, "raw_sha256")?;
    let original_upstream = value(&f,"upstream_hbp_sha256")?;
    if !digest_valid(raw_digest) || original_upstream != upstream
        || value(&f,"processed")? != "ALL"
        || value(&f,"raw_policy")? != "DISCARD_AFTER_COMMIT"
        || value(&f,"id")? != "batch"
    {
        return Err("GULP batch commitment invalid".to_owned());
    }
    let raw_bytes = usize_field(&f,"raw_bytes")?;
    let events = usize_field(&f,"events")?;
    let nkinds = usize_field(&f,"kinds")?;
    let nglyphs = usize_field(&f,"glyphs")?;
    if raw_bytes > MAX_BATCH || events == 0 || events > MAX_EVENTS
        || nkinds == 0 || nkinds > events || nglyphs == 0 || nglyphs > events
    {
        return Err("invalid GULP batch cardinalities".to_owned());
    }
    let (mut kinds,mut glyphs) = (BTreeMap::new(),BTreeMap::new());
    for line in lines {
        let f = fields(line)?;
        if line.starts_with("GULP_KIND|") {
            let id = value(&f,"id")?;
            if !checked_token(id) || value(&f,"kind")? != id
                || kinds.insert(id.to_owned(),usize_field(&f,"count")?).is_some()
            {
                return Err("duplicate or malformed retained kind".to_owned());
            }
        } else if line.starts_with("GULP_GLYPH|") {
            let id = value(&f,"id")?;
            if !checked_token(id) || value(&f,"glyph")? != id
                || glyphs.insert(id.to_owned(),usize_field(&f,"count")?).is_some()
            {
                return Err("duplicate or malformed retained glyph".to_owned());
            }
        } else if line.starts_with("GULP_GC|") {
            if value(&f,"id")? != "decision"
                || value(&f,"raw_sha256")? != raw_digest
                || value(&f,"delete_scope")? != "OWNED_EPHEMERAL_ONLY"
                || value(&f,"delete_eligible")? != "1"
                || value(&f,"manual_approval")? != "NOT_REQUIRED"
            {
                return Err("unverified GULP disposal policy".to_owned());
            }
        } else {
            return Err("unexpected GULP audit record".to_owned());
        }
    }
    if kinds.len() != nkinds || glyphs.len() != nglyphs
        || kinds.values().sum::<usize>() != events
        || glyphs.values().sum::<usize>() != events
        || kinds.values().any(|&c| c == 0)
        || glyphs.values().any(|&c| c == 0)
    {
        return Err("GULP grouping coverage mismatch".to_owned());
    }
    Ok(Compact {
        raw_digest: raw_digest.to_owned(),
        raw_bytes,
        events,
        by_kind:kinds,
        by_glyph:glyphs,
    })
}
fn verify_summary(output: &Path, upstream: &str) -> Result<Compact, String> {
    let hbp = fs::read_to_string(output.join(HBP_NAME))
        .map_err(|e| format!("GULP durable summary absent: {e}"))?;
    let saved_hbi = fs::read(output.join(HBI_NAME))
        .map_err(|e| format!("GULP HBI absent: {e}"))?;
    let saved_anchor = fs::read(output.join(ANCHOR_NAME))
        .map_err(|e| format!("GULP anchor absent: {e}"))?;
    let data = reconstruct_from_committed(&hbp,upstream)?;
    let (expected_hbp,expected_hbi,expected_anchor) = render(&data,upstream)?;
    if hbp != expected_hbp
        || saved_hbi != expected_hbi.as_bytes()
        || saved_anchor != expected_anchor.as_bytes()
    {
        return Err("GULP committed SHA/SH/HASH/HBI/HBP chain differs".to_owned());
    }
    Ok(data)
}
fn marker(accepted: &Compact, upstream: &str, evidence: &Path)
    -> Result<String, String>
{
    let hbp = fs::read(evidence.join(HBP_NAME)).map_err(|e| format!("GULP HBP: {e}"))?;
    let hbi = fs::read(evidence.join(HBI_NAME)).map_err(|e| format!("GULP HBI: {e}"))?;
    let anchor = fs::read(evidence.join(ANCHOR_NAME))
        .map_err(|e| format!("GULP anchor: {e}"))?;
    let line = format!(
        "GULP_DISCARD|schema=pnp.gulp.discard.v1|source_hbp_sha256={upstream}|raw_sha256={}|raw_bytes={}|events={}|hbp_sha256={}|hbi_sha256={}|anchor_sha256={}|removed_path={INPUT_DIR}/{INPUT_FILE}|removed=1|manual_approval=NOT_REQUIRED|status=DISCARDED|seal=HBI-HBP-SHA-SH-HASH",
        accepted.raw_digest, accepted.raw_bytes,accepted.events,
        sha(&hbp),sha(&hbi),sha(&anchor)
    );
    Ok(format!(
        "{line}|checksum={}|json=0\n",
        sha(line.as_bytes())
    ))
}

/// Autonomous post-GULP: accepts only an explicitly owned disposable batch
/// in the receipt workspace, first commits/flushes summaries, then unlinks it.
pub fn run(gac: &Path, neural: &Path, omni: &Path, out: &Path)
    -> Result<usize, String>
{
    let upstream = upstream_root(gac,neural,omni,out)?;
    let raw_path = controlled_raw_path(out)?;
    let original = fs::read(&raw_path).map_err(|e| format!("GULP input unreadable: {e}"))?;
    let data = summarize(&original)?;
    let (hbp,hbi,anchor) = render(&data,&upstream)?;
    for (name,contents) in [
        (HBP_NAME,hbp),(HBI_NAME,hbi),(ANCHOR_NAME,anchor)
    ] {
        write_exclusive(&out.join(name),contents.as_bytes())?;
    }
    sync_dir(out)?;
    let verified = verify_summary(out,&upstream)?;
    if verified.raw_digest != data.raw_digest || verified.events != data.events {
        return Err("GULP third-seat summary disagreed: HOLD".to_owned());
    }
    let second = fs::read(&raw_path).map_err(|e| format!("input disappeared: {e}"))?;
    if second != original {
        return Err("owned raw batch changed between GULP and deletion: HOLD".to_owned());
    }
    // Actual reclamation: source Git files and permanent audit remain untouched.
    fs::remove_file(&raw_path)
        .map_err(|e| format!("GC could not reclaim owned batch: {e}"))?;
    fs::remove_dir(out.join(INPUT_DIR))
        .map_err(|e| format!("GC owned staging directory not empty: {e}"))?;
    sync_dir(out)?;
    if raw_path.exists() || out.join(INPUT_DIR).exists() {
        return Err("GC did not reclaim ephemeral data: HOLD".to_owned());
    }
    let disposal_receipt = marker(&verified,&upstream,out)?;
    write_exclusive(&out.join(DISCARD_NAME),disposal_receipt.as_bytes())?;
    sync_dir(out)?;
    if verify(gac,neural,omni,out)? != verified.events {
        return Err("GULP post-discard independent replay differs".to_owned());
    }
    Ok(verified.events)
}

/// Restart verifier: validates the persistent compacted records, source root,
/// complete HBI/HBP chain and successful physical reclamation evidence.
pub fn verify(gac: &Path, neural: &Path, omni: &Path, out: &Path)
    -> Result<usize, String>
{
    let upstream = upstream_root(gac,neural,omni,out)?;
    let retained = verify_summary(out,&upstream)?;
    if out.join(INPUT_DIR).exists() {
        return Err("owned GULP raw data still present: HOLD".to_owned());
    }
    let evidence = fs::read(out.join(DISCARD_NAME))
        .map_err(|e| format!("GC discard receipt absent: {e}"))?;
    if evidence != marker(&retained,&upstream,out)?.as_bytes() {
        return Err("GC actual reclamation receipt mismatch".to_owned());
    }
    Ok(retained.events)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_ten_event_pattern_counts_are_preserved() {
        let data = summarize(include_bytes!("../fixtures/GULP_OWNED_EVENTS.hbp"))
            .expect("valid signed event fixture");
        assert_eq!(data.events,10);
        assert_eq!(data.by_kind.get("MISTAKE"),Some(&4));
        assert_eq!(data.by_kind.get("GENIUS"),Some(&3));
        assert_eq!(data.by_kind.get("ROUTE"),Some(&2));
        assert_eq!(data.by_kind.get("NEUTRAL"),Some(&1));
        assert_eq!(data.by_glyph.get("MINUS"),Some(&4));
        assert_eq!(data.by_glyph.get("PLUS"),Some(&3));
        assert_eq!(data.by_glyph.get("ZERO"),Some(&3));
        let upstream = sha(b"frozen-original-GAC-source");
        let (hbp,hbi,anchor)=render(&data,&upstream).expect("committed groups");
        assert!(hbp.contains("|raw_policy=DISCARD_AFTER_COMMIT|"));
        assert_eq!(hbi.lines().filter(|line| line.starts_with("HBI|")).count(),9);
        assert!(anchor.contains("seal=HBI-HBP-SHA-SH-HASH"));
        assert_eq!(
            reconstruct_from_committed(&hbp,&upstream).expect("recount").events,10
        );
    }
    #[test]
    fn partial_batch_and_incomplete_gulp_must_hold_and_preserve_input() {
        let original = include_str!("../fixtures/GULP_OWNED_EVENTS.hbp");
        assert!(summarize(original.trim_end_matches('\n').as_bytes()).is_err());
        assert!(summarize(original.replace("seq=5","seq=6").as_bytes()).is_err());
        assert!(summarize(original.replace("kind=GENIUS","kind=").as_bytes()).is_err());
        assert!(summarize(INPUT_HEADER.as_bytes()).is_err());
    }
    #[test]
    fn corrupt_compacted_summary_cannot_pass_independent_recount() {
        let data = summarize(include_bytes!("../fixtures/GULP_OWNED_EVENTS.hbp"))
            .expect("batch");
        let upstream=sha(b"frozen-original-source");
        let (hbp,_,_)=render(&data,&upstream).expect("root");
        assert!(reconstruct_from_committed(
            &hbp.replace("count=4","count=5"),&upstream
        ).is_err());
        assert!(reconstruct_from_committed(
            &hbp.replace("DISCARD_AFTER_COMMIT","RETAIN_FOREVER"),&upstream
        ).is_err());
    }
}
