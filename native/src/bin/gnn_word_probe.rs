//! Frozen BEHCS-256 word-graph probe for external original PyTorch sidecar.
//! Rust 1.81 source identity and exact 432-word held-out wave fixture.
//! The model's floating-point inference is executed by its original Python
//! implementation, not by the native exact-integer transport kernel.
#![forbid(unsafe_code)]

use pnp_native::{behcs_glyph_language, sha256::hash_bytes};
use std::{env, fs, path::Path, process::Command};

const WORD_COMMIT: &str = "818cc235a6db815e3b2591339cf9b42aad3e85a0";
const CASCADE_BLOB: &str = "b9acdaff9b4537def53eca3d0fdb87a9c0dcab89";
const THIRD_BLOB: &str = "a3e4485ca4a60d1b12cf6ed398c2f886ba358ea7";
const CODEX_BLOB: &str = "8d4a3298576ce2c9d4a1683ffab667c8a743fc42";
const ACTORS: [&str;6] = ["acer","liris","falcon","felipe","beast","dan"];
const VERBS: [&str;6] = ["scan-dispatch","scan-result","drift-detected","cosign-append","migration-intent","heartbeat"];
const TARGETS: [&str;6] = ["acer","liris","falcon","federation","phantom","witness"];
const STATES: [&str;6] = ["pending","in-flight","dispatched","settled","failed","observed"];
const WAVES: usize = 12;
const PAIRS: usize = 36;

fn require(accept: bool, reason: &str) -> Result<(), String> {
    if accept { Ok(()) } else { Err(reason.to_owned()) }
}
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git").arg("-C").arg(root).args(args)
        .output().map_err(|e| format!("original Git query failed: {e}"))?;
    require(output.status.success(),"source Git command failed")?;
    String::from_utf8(output.stdout).map(|s| s.trim().to_owned())
        .map_err(|_| "source Git result is not UTF8".to_owned())
}
fn authenticated(root: &Path) -> Result<(), String> {
    require(git(root,&["rev-parse","HEAD"])? == WORD_COMMIT,
        "BEHCS word-language source commit drift")?;
    for (path, expected) in [
        ("packages/deep-wave/src/cascade.ts",CASCADE_BLOB),
        ("packages/deep-wave/src/third-cascade.ts",THIRD_BLOB),
        ("data/behcs/codex/alphabet.json",CODEX_BLOB),
    ] {
        require(git(root,&["hash-object",path])? == expected,
            "BEHCS original cascade or alphabet blob changed")?;
    }
    Ok(())
}
fn materialize() -> Result<String, String> {
    let mut rows = String::new();
    for wave in 0..WAVES {
        for actor in 0..6 {
            for target in 0..6 {
                // Deterministic stratified source sample: every row is an
                // actual coordinate of the original 93,312-point cube.
                // No training labels, checkpoint weights or source language
                // semantics are created by these integer selectors.
                let verb = (actor + 3*target + wave) % 6;
                let risk = (3*actor + 5*target + wave) % 6;
                let state = (actor + 2*target + wave) % 6;
                let word = format!(
                    "D1:{}/D2:{}/D3:{}/D4:{risk}/D5:{}/D6:w{wave}",
                    ACTORS[actor],VERBS[verb],TARGETS[target],STATES[state],
                );
                let glyph = behcs_glyph_language::address(&word)
                    .ok_or("unable to map original word to 256-glyph address")?;
                require(behcs_glyph_language::verify_word(&word,&glyph),
                    "original word-to-glyph SHA identity failed")?;
                let id = wave*PAIRS + actor*6 + target;
                rows.push_str(&format!(
                    "WORD|id={id}|wave={wave}|actor={}|verb={}|target={}|risk={risk}|state={}|word_sha={}|glyph_sha={}|json=0\n",
                    ACTORS[actor],VERBS[verb],TARGETS[target],STATES[state],
                    hash_bytes(word.as_bytes()),hash_bytes(glyph.as_bytes()),
                ));
            }
        }
    }
    require(rows.lines().count()==432,"source wave fixture count changed")?;
    let body_hash=hash_bytes(rows.as_bytes());
    Ok(format!(
        "BEHCS_GNN_WORD_PROBE|schema=1|source_commit={WORD_COMMIT}|cascade_blob={CASCADE_BLOB}|third_blob={THIRD_BLOB}|alphabet_blob={CODEX_BLOB}|all_source_events=93312|sampled_words=432|waves=12|edges_per_wave=36|context_waves=0_7|heldout_waves=8_11|word_sha={body_hash}|label_status=NO_ORIGINAL_BINARY_LABELS|status=PASS|json=0\n{rows}"
    ))
}
fn entry() -> Result<(), String> {
    let mut args=env::args().skip(1);
    let op=args.next().ok_or("missing command")?;
    let source=args.next().ok_or("missing pinned word checkout")?;
    let path=args.next().ok_or("missing fixture file")?;
    require(args.next().is_none(),"unexpected extra arguments")?;
    authenticated(Path::new(&source))?;
    let original=materialize()?;
    match op.as_str() {
        "emit"=>{
            fs::write(&path,&original).map_err(|e|e.to_string())?;
            println!("BEHCS_WORD_PROBE|records=432|waves=12|sha256={}|status=PASS|json=0",
                hash_bytes(original.as_bytes()));
        }
        "verify"=>{
            require(fs::read(&path).map_err(|e|e.to_string())?==original.as_bytes(),
                "word-graph fixture source or record mismatch")?;
            println!("BEHCS_WORD_PROBE_VERIFY|records=432|status=PASS|json=0");
        }
        _=>return Err("expected emit or verify".to_owned()),
    }
    Ok(())
}
fn main() {
    if let Err(err)=entry() {
        eprintln!("BEHCS_WORD_PROBE_HOLD|reason={err}|status=HOLD|json=0");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_sampled_words_with_original_glyphs() {
        let fixture=materialize().expect("source word materialization");
        let lines:Vec<_>=fixture.lines().collect();
        assert_eq!(lines.len(),433);
        assert!(lines[0].contains("|sampled_words=432|waves=12|"));
        assert!(lines[1].contains("|wave=0|actor=acer|verb=scan-dispatch|target=acer|risk=0|state=pending|"));
        assert!(lines[432].contains("|wave=11|actor=dan|"));
        assert_eq!(hash_bytes(fixture.as_bytes()).len(),64);
        let mut corrupted=fixture.into_bytes();
        corrupted[100]^=1;
        assert_ne!(hash_bytes(&corrupted),hash_bytes(&materialize().expect("fixture")));
    }
}
