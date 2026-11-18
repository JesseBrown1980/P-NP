//! Durable, lossless GC archive staging for original Omni files.
//! The source and archive both remain intact; live rotation requires authority.
#![forbid(unsafe_code)]

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::{fabric_gc, sha256::hash_bytes};

const HBP_HEADER: &str =
    "HBPHEADER|schema=pnp.gc.archive.v1|mode=STAGE_ONLY|seal=HBI-HBP-SHA-SH-HASH|operator=HELD|json=0\n";
const HBI_HEADER: &str =
    "HBIHEADER|schema=pnp.gc.archive.index.v1|for=PNP_GC_ARCHIVE.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const ARCHIVE_DIR: &str = "PNP_GC_BODIES";
const ORIGINALS: [(&str, &str, &str); 2] = [
    (
        "omni_gc",
        "packages/omni-gulp-gc/src/gc.ts",
        "original_gc.ts",
    ),
    (
        "omni_gulp",
        "packages/omni-gulp-gc/src/gulp.ts",
        "original_gulp.ts",
    ),
];

fn fingerprint(bytes: &[u8]) -> String {
    hash_bytes(bytes)
}
fn ensure_regular_file(path: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(path)
        .map_err(|e| format!("archive source/target metadata unavailable: {e}"))?;
    if !meta.file_type().is_file() {
        return Err("archive body must be an ordinary file".to_owned());
    }
    if meta.len() > fabric_gc::SANITY_BYTES {
        return Err("archive source exceeds original 500 MiB sanity cap".to_owned());
    }
    Ok(())
}
fn sources_verified(gac: &Path, neural: &Path, omni: &Path, proof: &Path)
    -> Result<String, String>
{
    if fabric_gc::verify(gac, neural, omni, proof)? != 16 {
        return Err("original fabric source proof missing".to_owned());
    }
    let evidence = fs::read(proof.join("PNP_FABRIC_GC.hbp"))
        .map_err(|e| format!("original source HBP unavailable: {e}"))?;
    Ok(fingerprint(&evidence))
}

/// Copy original bytes to a new archival body and sync them before receipt
/// generation. Any existing destination is a HOLD and is never overwritten.
fn copy_original_to_new_archive(original: &Path, destination: &Path) -> Result<(), String> {
    ensure_regular_file(original)?;
    let source_before = fs::read(original)
        .map_err(|e| format!("source byte stream missing: {e}"))?;
    let mut archive = OpenOptions::new()
        .write(true).create_new(true).open(destination)
        .map_err(|e| format!("existing or blocked archive destination: {e}"))?;
    archive.write_all(&source_before)
        .map_err(|e| format!("archive write interrupted: {e}"))?;
    archive.sync_all().map_err(|e| format!("archive flush failed: {e}"))?;
    let source_after = fs::read(original)
        .map_err(|e| format!("source vanished after archival: {e}"))?;
    let stored = fs::read(destination)
        .map_err(|e| format!("archive not readable after flush: {e}"))?;
    if source_before != source_after || source_before != stored {
        return Err("source changed or archived bytes differ: HOLD".to_owned());
    }
    Ok(())
}

type ArchiveReceipts = (String, String, String);
fn build_receipts(omni: &Path, out: &Path, upstream_sha: &str)
    -> Result<ArchiveReceipts, String>
{
    let mut hbp = HBP_HEADER.to_owned();
    let mut hbi = HBI_HEADER.to_owned();
    let mut predecessor = fingerprint(b"PNP-GAC-ARCHIVE-PRESERVATION-GENESIS-v1");
    for (index, (id, source_rel, archive_name)) in ORIGINALS.iter().enumerate() {
        let source = omni.join(source_rel);
        let archived = out.join(ARCHIVE_DIR).join(archive_name);
        ensure_regular_file(&source)?;
        ensure_regular_file(&archived)?;
        let original = fs::read(&source).map_err(|e| format!("original missing: {e}"))?;
        let backup = fs::read(&archived).map_err(|e| format!("archive missing: {e}"))?;
        if original != backup {
            return Err(format!("archived body differs from original source: {id}"));
        }
        let original_sha = fingerprint(&original);
        let archive_sha = fingerprint(&backup);
        if original_sha != archive_sha {
            return Err(format!("GC body SHA changed: {id}"));
        }
        let sh = &original_sha[..16];
        let entry = format!(
            "GC_ARCHIVE|seq={}|id={id}|source_path={source_rel}|archive_path={ARCHIVE_DIR}/{archive_name}|source_hbp_sha256={upstream_sha}|source_sha256={original_sha}|archive_sha256={archive_sha}|bytes={}|sh={sh}|source_intact=1|archive_intact=1|operator_approval=HELD|live_rotation=HOLD|gc=NEVER_DELETE|gulp_feedback=HELD|prev_hash={predecessor}",
            index + 1, original.len()
        );
        let receipt_sha = fingerprint(entry.as_bytes());
        let chain = fingerprint(format!("{predecessor}|{receipt_sha}").as_bytes());
        let row = format!("{entry}|receipt_sha={receipt_sha}|hash={chain}|json=0\n");
        hbi.push_str(&format!(
            "HBI|seq={}|id={id}|offset={}|len={}|hbp_sha256={}|sh={sh}|hash={chain}|status=STAGED_VERIFIED|json=0\n",
            index + 1, hbp.len(), row.len(), fingerprint(row.as_bytes())
        ));
        hbp.push_str(&row);
        predecessor = chain;
    }
    let anchor_payload = format!(
        "GC_ARCHIVE_ANCHOR|count=2|upstream_hbp_sha256={upstream_sha}|root_hbp_sha256={}|root_hbi_sha256={}|hash={predecessor}|source_intact=1|archive_intact=1|live_rotation=HOLD|seal=HBI-HBP-SHA-SH-HASH",
        fingerprint(hbp.as_bytes()), fingerprint(hbi.as_bytes())
    );
    let anchor = format!(
        "{anchor_payload}|anchor_checksum={}|json=0\n",
        fingerprint(anchor_payload.as_bytes())
    );
    Ok((hbp, hbi, anchor))
}

pub fn attest(gac: &Path, neural: &Path, omni: &Path, output: &Path)
    -> Result<usize, String>
{
    let upstream_sha = sources_verified(gac, neural, omni, output)?;
    let bodies_dir = output.join(ARCHIVE_DIR);
    // Never rotate, overwrite, or delete existing original/archived bodies.
    fs::create_dir(&bodies_dir)
        .map_err(|e| format!("GC staging path already exists or blocked: {e}"))?;
    for (_id, original, archived) in ORIGINALS {
        copy_original_to_new_archive(&omni.join(original), &bodies_dir.join(archived))?;
    }
    let (hbp, hbi, anchor) = build_receipts(omni, output, &upstream_sha)?;
    for (name, record) in [
        ("PNP_GC_ARCHIVE.hbp", hbp),
        ("PNP_GC_ARCHIVE.hbi", hbi),
        ("PNP_GC_ARCHIVE.anchor", anchor),
    ] {
        let mut file = OpenOptions::new()
            .write(true).create_new(true).open(output.join(name))
            .map_err(|e| format!("GC receipt destination exists or blocked: {e}"))?;
        file.write_all(record.as_bytes())
            .map_err(|e| format!("write GC source receipt: {e}"))?;
        file.sync_all().map_err(|e| format!("GC source receipt flush: {e}"))?;
    }
    Ok(ORIGINALS.len())
}

pub fn verify(gac: &Path, neural: &Path, omni: &Path, output: &Path)
    -> Result<usize, String>
{
    let upstream_sha = sources_verified(gac, neural, omni, output)?;
    let (hbp, hbi, anchor) = build_receipts(omni, output, &upstream_sha)?;
    for (name, expected) in [
        ("PNP_GC_ARCHIVE.hbp", hbp),
        ("PNP_GC_ARCHIVE.hbi", hbi),
        ("PNP_GC_ARCHIVE.anchor", anchor),
    ] {
        let got = fs::read(output.join(name))
            .map_err(|e| format!("incomplete archive/journal after restart: {e}"))?;
        if got != expected.as_bytes() {
            return Err(format!("archive HBI/HBP/anchor mismatch: {name}"));
        }
    }
    Ok(ORIGINALS.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lossless_recovery_hash_is_domain_separated_and_ordered() {
        let original = b"GC-MISTAKE|room=27|state=HELD\nGULP|genius=2|mistake=1\n";
        assert_eq!(fingerprint(original), hash_bytes(original));
        let a = fingerprint(format!("A|{}",fingerprint(original)).as_bytes());
        let b = fingerprint(format!("B|{}",fingerprint(original)).as_bytes());
        assert_ne!(a,b);
    }
    #[test]
    fn archive_identities_are_distinct_and_relpaths_stable() {
        assert_eq!(ORIGINALS.len(), 2);
        assert_ne!(ORIGINALS[0].0, ORIGINALS[1].0);
        assert_ne!(ORIGINALS[0].2, ORIGINALS[1].2);
        assert!(ORIGINALS.iter().all(|entry| {
            entry.1.starts_with("packages/omni-gulp-gc/src/")
                && !entry.1.contains("..")
                && entry.2.starts_with("original_")
        }));
    }
}
