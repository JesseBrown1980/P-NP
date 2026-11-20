//! Fail-closed, source-pinned cutover gate. Historical implementations stay
//! byte-for-byte immutable and receive FAIL_NATIVE_ACCEPTANCE, never promotion.
//! New executable implementation is allowed only in the Rust 1.81 native tree.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

const MANIFEST: &str = include_str!("../../policy/LEGACY_PINNED.hbp");
const ACCEPTED_WORKFLOWS: [&str; 16] = [
    ".github/workflows/pnp-native-certification.yml",
    ".github/workflows/pnp-chariot-twenty-leaf.yml",
    ".github/workflows/pnp-qprism-hosted-transport.yml",
    ".github/workflows/pnp-real-corpus-parity.yml",
    ".github/workflows/pnp-gnn-trained-wave-reference.yml",
    ".github/workflows/pnp-original-gnn-family-comparison.yml",
    ".github/workflows/pnp-stone-moss-source-recovery.yml",
    ".github/workflows/pnp-bitcoin-300s-source-reference.yml",
    ".github/workflows/pnp-bitcoin-matrix-funnel.yml",
    ".github/workflows/pnp-bitcoin-300s-research-extended.yml",
    ".github/workflows/pnp-bitcoin-300s-matrix-search.yml",
    ".github/workflows/pnp-bitcoin-cnf-orchestration-compare.yml",
    ".github/workflows/pnp-bitcoin-live-state-vs-direct.yml",
    ".github/workflows/pnp-bitcoin-matrix-guided-selector.yml",
    ".github/workflows/pnp-bitcoin-fischer-frozen-gguf.yml",
    ".github/workflows/pnp-bitcoin-gguf-sat-resume.yml",
];
// Explicit research-only model analysis, executed by pinned hosted workflow.
// Native production and transport still accept only Rust implementations.
const ACCEPTED_RESEARCH_SOURCE: [&str; 13] = [
    "research/gnn_original_checkpoint_compare.py",
    "research/stone_moss_recovery_verify.py",
    "research/stone_moss_checkpoint_forward.py",
    "research/stone_moss_provenance_check.py",
    "research/bitcoin_300s_reference.py",
    "research/bitcoin_300s_research.py",
    "research/bitcoin_300s_matrix_search.py",
    "research/bitcoin_300s_matrix_verify.py",
    "research/bitcoin_cnf_pump_compare.py",
    "research/bitcoin_live_sat_referee.py",
    "research/bitcoin_matrix_guided_referee.py",
    "research/bitcoin_gguf_fischer_referee.py",
    "research/bitcoin_gguf_sat_resume_referee.py",
];
const EXPECTED_LEGACY: usize = 54;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Audit {
    pub legacy_failed: usize,
    pub pinned_workflows: usize,
    pub pinned_scripts: usize,
    pub accepted_native_sources: usize,
}

fn exec_git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("unable to run git: {e}"))?;
    if !result.status.success() {
        return Err(format!("git command failed: {}", args.join(" ")));
    }
    Ok(result.stdout)
}

fn pinned_legacy() -> Result<BTreeMap<String, String>, String> {
    let mut found = BTreeMap::new();
    for line in MANIFEST.lines().filter(|line| line.starts_with("LEGACY|")) {
        let mut path = None;
        let mut blob = None;
        let mut failed = false;
        for field in line.split('|').skip(1) {
            if let Some(value) = field.strip_prefix("path=") {
                path = Some(value.to_owned());
            }
            if let Some(value) = field.strip_prefix("blob=") {
                blob = Some(value.to_owned());
            }
            if field == "software_status=FAIL_NATIVE_ACCEPTANCE" {
                failed = true;
            }
        }
        let path = path.ok_or("missing manifest path")?;
        let blob = blob.ok_or("missing manifest hash")?;
        if !failed || blob.len() != 40 || !blob.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("malformed pinned legacy record: {path}"));
        }
        if found.insert(path.clone(), blob).is_some() {
            return Err(format!("duplicate legacy path: {path}"));
        }
    }
    if found.len() != EXPECTED_LEGACY {
        return Err(format!("legacy inventory count must remain {EXPECTED_LEGACY}"));
    }
    Ok(found)
}

fn executable_legacy(path: &str) -> bool {
    path.starts_with(".github/workflows/")
        || [".py", ".js", ".cjs", ".mjs", ".jsx", ".ts", ".tsx", ".json"]
            .iter()
            .any(|suffix| path.ends_with(suffix))
}

fn forbidden_native_identifiers(source: &str) -> Result<(), String> {
    // Lexical identifier gate on executable source lines. Compiler tests and
    // offline locked Clippy provide an independent second acceptance layer.
    for (line_number, line) in source.lines().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        for token in code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            if matches!(token, "f32" | "f64" | "unsafe" | "serde" | "serde_json"
                            | "JavaScript" | "v8" | "nodejs") {
                return Err(format!("forbidden identifier '{token}' on line {}",
                                   line_number + 1));
            }
        }
    }
    Ok(())
}

pub fn check_repository(root: &Path) -> Result<Audit, String> {
    let original = pinned_legacy()?;
    let tracked = exec_git(root, &["ls-files", "-z"])?;
    let filenames: Vec<String> = tracked.split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| String::from_utf8(name.to_vec()).map_err(|_| "non-UTF8 file"))
        .collect::<Result<_, _>>()?;
    let current: BTreeSet<String> = filenames.iter().cloned().collect();
    let mut scripts = 0;
    let mut workflows = 0;
    for (file, sha) in &original {
        if !current.contains(file) {
            return Err(format!("legacy evidence removed: {file}"));
        }
        let hash = exec_git(root, &["hash-object", file])?;
        let actual = String::from_utf8(hash).map_err(|_| "invalid git hash")?;
        if actual.trim() != sha {
            return Err(format!("historical source changed without native recertification: {file}"));
        }
        if file.starts_with(".github/workflows/") {
            workflows += 1;
        } else {
            scripts += 1;
        }
    }
    let mut native = 0;
    for file in &filenames {
        if executable_legacy(file) && !ACCEPTED_WORKFLOWS.contains(&file.as_str())
            && !original.contains_key(file)
            && !ACCEPTED_RESEARCH_SOURCE.contains(&file.as_str())
        {
            return Err(format!("non-native executable is not accepted: {file}"));
        }
        if file.ends_with(".rs") {
            if !file.starts_with("native/src/") {
                return Err(format!("new Rust implementation outside native crate: {file}"));
            }
            native += 1;
            // The auditor's own literal fixture strings have names of
            // forbidden tokens; scan every OTHER executable Rust module.
            if file != "native/src/policy.rs" {
                let body = std::fs::read_to_string(root.join(file))
                    .map_err(|e| format!("cannot inspect {file}: {e}"))?;
                forbidden_native_identifiers(&body)
                    .map_err(|e| format!("{file}: {e}"))?;
            }
        }
    }
    let cargo = std::fs::read_to_string(root.join("native/Cargo.toml"))
        .map_err(|e| format!("Cargo manifest missing: {e}"))?;
    if !cargo.contains("rust-version = \"1.81.0\"")
        || !cargo.contains("edition = \"2021\"")
        || !cargo.contains("[dependencies]\n\n[profile.release]")
    {
        return Err("Rust 1.81, no third-party crates, and locked exact arithmetic required"
                   .to_owned());
    }
    let toolchain = std::fs::read_to_string(root.join("native/rust-toolchain.toml"))
        .map_err(|e| format!("toolchain missing: {e}"))?;
    if !toolchain.contains("channel = \"1.81.0\"")
        || !toolchain.contains("\"clippy\"") {
        return Err("pinned Rust 1.81 and Clippy are mandatory".to_owned());
    }
    if native < 5 {
        return Err("native implementation absent".to_owned());
    }
    Ok(Audit {
        legacy_failed: original.len(),
        pinned_workflows: workflows,
        pinned_scripts: scripts,
        accepted_native_sources: native,
    })
}

pub fn audit_source_text_for_tests(source: &str) -> Result<(), String> {
    forbidden_native_identifiers(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_and_hostile_numeric_sources_fail() {
        for forbidden in [
            "let value: f32 = 1.0;",
            "let score: f64 = 2.0;",
            "unsafe { operation() }",
            "serde_json::from_slice(bytes)",
            "let runtime = v8;",
            "let runtime = nodejs;",
        ] {
            assert!(audit_source_text_for_tests(forbidden).is_err());
        }
        assert!(audit_source_text_for_tests(
            "let total = 3_i128.checked_mul(27).expect(\"bounded\");"
        ).is_ok());
    }

    #[test]
    fn legacy_inventory_is_complete_and_failed() {
        let inventory = pinned_legacy().expect("pinned");
        assert_eq!(inventory.len(), 54);
        assert!(inventory.contains_key("sat/engine.py"));
        assert!(inventory.contains_key("experiments/mtp_nnest/hybrid.py"));
        assert!(inventory.contains_key(".github/workflows/bitcoin-genesis-7.yml"));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[0]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[1]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[2]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[3]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[4]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[5]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[6]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[7]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[8]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[9]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[10]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[11]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[12]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[13]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[14]));
        assert!(!inventory.contains_key(ACCEPTED_WORKFLOWS[15]));
        assert!(ACCEPTED_RESEARCH_SOURCE.iter().all(|p| p.starts_with("research/")));
        assert!(!ACCEPTED_RESEARCH_SOURCE.contains(&"native/research/gnn_original_checkpoint_compare.py"));
    }
}
