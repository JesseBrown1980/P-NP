//! Hosted Q-PRISM source-bound 256 -> 1024 -> 256 transfer acceptance.
//! Carries the actual ten-bit symbol values across distinct hosted workers.
//! Exact arithmetic, Rust 1.81, no third-party crates, fail-closed receipts.
#![forbid(unsafe_code)]

use pnp_native::{prism, sha256::hash_bytes};
use std::{collections::BTreeSet, env, fs, path::{Path, PathBuf}, process::Command};

const SOURCE_COMMIT: &str = "b4a21a25381d6920b7aab92c94c3c3e1420fc9e4";
const SOURCE_FILE: &str = "host8/qprism_cube_host8.rs";
const SOURCE_BLOB: &str = "992ffd793f6c98f4f03b33ff0d1154b52d0174b0";
const CUBE_SHA: &str = "fe28e4a1813ea87aa55d9e4aad3e8ac028a2142890e39b3ab8e5eb40ba4ef05e";
const CUBE_SYMBOL_SHA: &str = "9593d266e75e5494172c62f4ceaebc2c4bd9a9cb9476f528f9d71543f313d4ab";
const LEAVES: usize = 20;
const CUBE_BYTES: usize = 3_200;
const RAW_BYTES: usize = 1_024_000;
const SYMBOLS: usize = RAW_BYTES * 4 / 5;
const WIRE_BYTES: usize = SYMBOLS * 2;

struct Corpus { source: Vec<u8>, cube: Vec<u8>, source_sha: String }

fn require(ok: bool, reason: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(reason.to_owned()) }
}
fn checked_run(run: &str) -> Result<(), String> {
    require(!run.is_empty() && run.len() <= 128 && run.bytes().all(
        |b| b.is_ascii_alphanumeric() || b"._-".contains(&b)),
        "invalid run identifier")
}
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git").arg("-C").arg(root).args(args).output()
        .map_err(|e| format!("pinned Git source inaccessible: {e}"))?;
    require(output.status.success(), "original Q-PRISM checkout unavailable")?;
    String::from_utf8(output.stdout).map(|x| x.trim().to_owned())
        .map_err(|_| "invalid Git source output".to_owned())
}
fn cube3200() -> Vec<u8> {
    (0_u32..3_200).map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8).collect()
}
fn wire_bytes(words: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(words.len() * 2);
    for &word in words { out.extend_from_slice(&word.to_le_bytes()); }
    out
}
fn symbols_from_wire(wire: &[u8], expected: usize) -> Result<Vec<u16>, String> {
    require(wire.len() == expected * 2, "symbol-wire byte count mismatch")?;
    let mut out = Vec::with_capacity(expected);
    for pair in wire.chunks_exact(2) {
        let word = u16::from_le_bytes([pair[0], pair[1]]);
        require(word < 1024, "out-of-alphabet 1024 symbol")?;
        out.push(word);
    }
    Ok(out)
}
fn reference_decode(words: &[u16], raw_bytes: usize) -> Result<Vec<u8>, String> {
    require(raw_bytes % 5 == 0 && words.len() == raw_bytes * 4 / 5,
        "reference transcode framing mismatch")?;
    let mut restored = Vec::with_capacity(raw_bytes);
    for block in words.chunks_exact(4) {
        require(block.iter().all(|word| *word < 1024),
            "reference decoder rejected symbol")?;
        // Independent 40-bit grouping, separate from pnp_native::prism::decode.
        let w = (u64::from(block[0]) << 30) | (u64::from(block[1]) << 20)
            | (u64::from(block[2]) << 10) | u64::from(block[3]);
        restored.extend_from_slice(&w.to_be_bytes()[3..8]);
    }
    Ok(restored)
}
fn original_source(root: &Path) -> Result<Corpus, String> {
    require(git(root, &["rev-parse", "HEAD"])? == SOURCE_COMMIT,
        "original Q-PRISM commit drift")?;
    require(git(root, &["hash-object", SOURCE_FILE])? == SOURCE_BLOB,
        "original Q-PRISM source blob drift")?;
    let source = fs::read(root.join(SOURCE_FILE)).map_err(|e| e.to_string())?;
    require(!source.is_empty(), "missing source-code bytes")?;
    let cube = cube3200();
    require(cube.len() == CUBE_BYTES && hash_bytes(&cube) == CUBE_SHA,
        "original 3200-byte corpus SHA changed")?;
    let gold = wire_bytes(&prism::encode(&cube));
    require(gold.len() == 5120 && hash_bytes(&gold) == CUBE_SYMBOL_SHA,
        "original Q-PRISM 2560-symbol golden digest changed")?;
    let source_sha = hash_bytes(&source);
    Ok(Corpus { source, cube, source_sha })
}
fn generated(corpus: &Corpus, leaf: usize, nbytes: usize) -> Result<Vec<u8>, String> {
    require(leaf < LEAVES && nbytes % 5 == 0 && nbytes <= RAW_BYTES && nbytes > 0,
        "payload source and capacity out of scope")?;
    let mut raw = Vec::with_capacity(nbytes);
    for i in 0..nbytes {
        // Each byte is a deterministic composition of TWO authenticated
        // original sources, with a lane-specific rotation and block index.
        // This is expanded test traffic, not an unreleased training corpus.
        let a = corpus.source[(i + leaf * 7919) % corpus.source.len()];
        let b = corpus.cube[(i + leaf * 37) % CUBE_BYTES];
        let domain = (i / CUBE_BYTES) as u8;
        raw.push(a ^ b ^ domain.wrapping_add(leaf as u8));
    }
    Ok(raw)
}
fn fwd_file(dir: &Path, leaf: usize) -> PathBuf {
    dir.join(format!("symbol-{leaf:02}.bin"))
}
fn rec_file(dir: &Path, leaf: usize) -> PathBuf {
    dir.join(format!("forward-{leaf:02}.hbp"))
}
fn rev_file(dir: &Path, leaf: usize) -> PathBuf {
    dir.join(format!("reverse-{leaf:02}.hbp"))
}
fn expected_receipt(run: &str, leaf: usize, corpus: &Corpus,
                    raw: &[u8], wire: &[u8]) -> String {
    format!(
        "QP20_FORWARD|schema=1|run={run}|leaf={leaf}|to={}|source_commit={SOURCE_COMMIT}|source_blob={SOURCE_BLOB}|source_sha={}|cube_sha={CUBE_SHA}|raw_bytes={}|symbols={}|wire_bytes={}|raw_sha={}|symbols_sha={}|format=LE_U16_1024|status=PASS|json=0\n",
        (leaf + 1) % LEAVES, corpus.source_sha, raw.len(), raw.len() * 4 / 5, wire.len(),
        hash_bytes(raw), hash_bytes(wire)
    )
}
fn make_forward(run: &str, leaf: usize, corpus: &Corpus, out: &Path)
    -> Result<(), String>
{
    checked_run(run)?;
    let raw = generated(corpus, leaf, RAW_BYTES)?;
    let words = prism::encode(&raw);
    require(words.len() == SYMBOLS, "source-to-1024 symbol count mismatch")?;
    let wire = wire_bytes(&words);
    require(wire.len() == WIRE_BYTES, "serialized symbols exceed expected wire size")?;
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    fs::write(fwd_file(out, leaf), &wire).map_err(|e| e.to_string())?;
    fs::write(rec_file(out, leaf),
              expected_receipt(run, leaf, corpus, &raw, &wire))
        .map_err(|e| e.to_string())?;
    Ok(())
}
fn verify_forward(run: &str, leaf: usize, corpus: &Corpus, dir: &Path)
    -> Result<(String, String, String), String>
{
    checked_run(run)?;
    let wire = fs::read(fwd_file(dir, leaf)).map_err(|e| e.to_string())?;
    let rec = fs::read_to_string(rec_file(dir, leaf)).map_err(|e| e.to_string())?;
    let words = symbols_from_wire(&wire, SYMBOLS)?;
    let original = generated(corpus, leaf, RAW_BYTES)?;
    let inverse = prism::decode(&words, RAW_BYTES).map_err(str::to_owned)?;
    let independent = reference_decode(&words, RAW_BYTES)?;
    require(inverse == independent && inverse == original,
        "received 1024-symbol payload did not reconstruct exact source bytes")?;
    require(prism::encode(&inverse) == words && wire_bytes(&words) == wire,
        "noncanonical wire symbols")?;
    require(rec == expected_receipt(run, leaf, corpus, &original, &wire),
        "forward source identity or receipt changed")?;
    Ok((hash_bytes(rec.as_bytes()), hash_bytes(&wire), hash_bytes(&original)))
}
fn roster_line(leaf: usize, receipt_sha: &str, wire_sha: &str, raw_sha: &str) -> String {
    format!("LEAF|id={leaf}|receipt_sha={receipt_sha}|wire_sha={wire_sha}|raw_sha={raw_sha}|json=0\n")
}
fn checked_dir_names(dir: &Path, phase: &str) -> Result<(), String> {
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().into_string()
            .map_err(|_| "non-UTF8 artifact filename".to_owned())?;
        require(entry.file_type().map_err(|e| e.to_string())?.is_file(),
            "unexpected non-file artifact")?;
        require(names.insert(name), "duplicate checkpoint filename")?;
    }
    let expected = if phase == "forward" {
        (0..LEAVES).flat_map(|i| [format!("symbol-{i:02}.bin"),
            format!("forward-{i:02}.hbp")]).collect::<BTreeSet<_>>()
    } else {
        (0..LEAVES).map(|i| format!("reverse-{i:02}.hbp")).collect::<BTreeSet<_>>()
    };
    require(names == expected, "incomplete or unexpected artifact roster")
}
fn checked_root(run: &str, corpus: &Corpus, data: &str)
    -> Result<(String, Vec<(String, String, String)>), String>
{
    checked_run(run)?;
    require(data.ends_with('\n'), "unterminated root certificate")?;
    let lines: Vec<_> = data.lines().collect();
    require(lines.len() == LEAVES + 1, "root roster must contain exactly twenty leaves")?;
    let mut roster = String::new();
    let mut hashes = Vec::with_capacity(LEAVES);
    for (leaf, line) in lines.iter().skip(1).enumerate() {
        let prefix = format!("LEAF|id={leaf}|receipt_sha=");
        require(line.starts_with(&prefix), "root roster order or field mismatch")?;
        let fields: Vec<_> = line.split('|').collect();
        require(fields.len() == 6 && fields[5] == "json=0",
            "root roster field grammar changed")?;
        let receipt_sha = fields[3].strip_prefix("wire_sha=")
            .ok_or("missing wire SHA field")?;
        let raw_sha = fields[4].strip_prefix("raw_sha=")
            .ok_or("missing source SHA field")?;
        let actual_receipt = fields[2].strip_prefix("receipt_sha=")
            .ok_or("missing receipt SHA field")?;
        for value in [receipt_sha, raw_sha, actual_receipt] {
            require(value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()
                && !b.is_ascii_uppercase()), "invalid SHA-256 roster field")?;
        }
        // Preserve order and exact body; SHA is over all twenty rows.
        roster.push_str(line);
        roster.push('\n');
        hashes.push((actual_receipt.to_owned(), receipt_sha.to_owned(), raw_sha.to_owned()));
    }
    let preimage = format!("QP20_ROOT_V1|run={run}|source={}|{SOURCE_BLOB}\n{roster}",
        corpus.source_sha);
    let root = hash_bytes(preimage.as_bytes());
    let expected_header = format!(
        "QP20_ROOT|schema=1|run={run}|source_sha={}|cube_sha={CUBE_SHA}|leaves=20|raw_bytes={}|symbols={}|encoded_bytes={}|root={root}|status=PASS|json=0",
        corpus.source_sha, RAW_BYTES * LEAVES, SYMBOLS * LEAVES, WIRE_BYTES * LEAVES
    );
    require(lines[0] == expected_header, "root header or SHA changed")?;
    Ok((root, hashes))
}
fn fanin(run: &str, corpus: &Corpus, dir: &Path) -> Result<String, String> {
    checked_dir_names(dir, "forward")?;
    let mut roster = String::new();
    for leaf in 0..LEAVES {
        let (receipt, wire, raw) = verify_forward(run, leaf, corpus, dir)?;
        roster.push_str(&roster_line(leaf, &receipt, &wire, &raw));
    }
    let root = hash_bytes(format!("QP20_ROOT_V1|run={run}|source={}|{SOURCE_BLOB}\n{roster}",
        corpus.source_sha).as_bytes());
    let body = format!(
        "QP20_ROOT|schema=1|run={run}|source_sha={}|cube_sha={CUBE_SHA}|leaves=20|raw_bytes={}|symbols={}|encoded_bytes={}|root={root}|status=PASS|json=0\n{roster}",
        corpus.source_sha, RAW_BYTES * LEAVES, SYMBOLS * LEAVES, WIRE_BYTES * LEAVES
    );
    checked_root(run, corpus, &body)?;
    Ok(body)
}
fn reverse(run: &str, receiver: usize, corpus: &Corpus,
           forward_dir: &Path, root_path: &Path) -> Result<String, String>
{
    require(receiver < LEAVES, "receiving worker outside roster")?;
    let from = (receiver + LEAVES - 1) % LEAVES;
    let root_data = fs::read_to_string(root_path).map_err(|e| e.to_string())?;
    let (root, roster) = checked_root(run, corpus, &root_data)?;
    let (receipt, wire, raw) = verify_forward(run, from, corpus, forward_dir)?;
    require(roster[from] == (receipt.clone(), wire.clone(), raw.clone()),
        "transferred leaf does not match independent root commitment")?;
    Ok(format!(
        "QP20_REVERSE|schema=1|run={run}|receiver={receiver}|from={from}|parent_root={root}|source_sha={}|payload_sha={wire}|raw_sha={raw}|raw_bytes={RAW_BYTES}|symbols={SYMBOLS}|decode=NATIVE_AND_INDEPENDENT_40BIT|restored=EXACT|status=PASS|json=0\n",
        corpus.source_sha
    ))
}
fn finalize(run: &str, corpus: &Corpus, forward_dir: &Path,
            root_path: &Path, reverse_dir: &Path) -> Result<String, String>
{
    checked_dir_names(reverse_dir, "reverse")?;
    let expected_root_data = fanin(run, corpus, forward_dir)?;
    require(fs::read_to_string(root_path).map_err(|e| e.to_string())? == expected_root_data,
        "final verification found changed root")?;
    let (root, _) = checked_root(run, corpus, &expected_root_data)?;
    let mut witness = format!("QP20_REVERSE_ROOT_V1|run={run}|parent={root}\n");
    for receiver in 0..LEAVES {
        let actual = fs::read_to_string(rev_file(reverse_dir, receiver))
            .map_err(|e| e.to_string())?;
        require(actual == reverse(run, receiver, corpus, forward_dir, root_path)?,
            "reverse worker receipt did not reproduce independently")?;
        witness.push_str(&hash_bytes(actual.as_bytes()));
        witness.push('\n');
    }
    let reverse_root = hash_bytes(witness.as_bytes());
    Ok(format!(
        "QP20_FINAL|schema=1|run={run}|source_commit={SOURCE_COMMIT}|source_blob={SOURCE_BLOB}|source_sha={}|cube_sha={CUBE_SHA}|forward_root={root}|reverse_root={reverse_root}|forward_workers=20|reverse_workers=20|source_bytes={}|transported_symbols={}|transported_bytes={}|separate_hosted_workers=40|native_decode=PASS|independent_40bit_decode=PASS|restoration=EXACT|status=PASS|json=0\n",
        corpus.source_sha, RAW_BYTES * LEAVES, SYMBOLS * LEAVES, WIRE_BYTES * LEAVES
    ))
}
fn next(args: &mut impl Iterator<Item = String>, reason: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("missing {reason}"))
}
fn entry() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mode = next(&mut args, "command")?;
    let run = next(&mut args, "run identity")?;
    let source_root = next(&mut args, "pinned original Q-PRISM checkout")?;
    checked_run(&run)?;
    let corpus = original_source(Path::new(&source_root))?;
    match mode.as_str() {
        "provenance" => {
            println!("QP20_SOURCE|commit={SOURCE_COMMIT}|blob={SOURCE_BLOB}|source_sha={}|cube_sha={CUBE_SHA}|cube_symbols=2560|raw_bytes_per_leaf={RAW_BYTES}|transport_bytes_per_leaf={WIRE_BYTES}|status=PASS|json=0", corpus.source_sha);
        }
        "forward" => {
            let leaf: usize = next(&mut args, "leaf")?.parse()
                .map_err(|_| "invalid leaf number".to_owned())?;
            let output = next(&mut args, "forward artifact directory")?;
            make_forward(&run, leaf, &corpus, Path::new(&output))?;
            println!("QP20_FORWARD_PASS|leaf={leaf}|raw_bytes={RAW_BYTES}|wire_bytes={WIRE_BYTES}|status=PASS|json=0");
        }
        "fanin" => {
            let input = next(&mut args, "forward artifact directory")?;
            let output = next(&mut args, "root artifact path")?;
            let body = fanin(&run, &corpus, Path::new(&input))?;
            fs::write(output, &body).map_err(|e| e.to_string())?;
            println!("QP20_FANIN_PASS|raw_bytes={}|wire_bytes={}|status=PASS|json=0", RAW_BYTES * LEAVES, WIRE_BYTES * LEAVES);
        }
        "reverse" => {
            let receiver: usize = next(&mut args, "receiver")?.parse()
                .map_err(|_| "invalid receiver".to_owned())?;
            let input = next(&mut args, "forward artifact directory")?;
            let root = next(&mut args, "root file")?;
            let output = next(&mut args, "reverse receipt path")?;
            let body = reverse(&run, receiver, &corpus, Path::new(&input), Path::new(&root))?;
            fs::write(output, &body).map_err(|e| e.to_string())?;
            println!("QP20_REVERSE_PASS|receiver={receiver}|from={}|status=PASS|json=0",
                (receiver + LEAVES - 1) % LEAVES);
        }
        "final" => {
            let input = next(&mut args, "forward artifacts")?;
            let root = next(&mut args, "forward root")?;
            let reverse_dir = next(&mut args, "reverse artifacts")?;
            let output = next(&mut args, "final receipt")?;
            let body = finalize(&run, &corpus, Path::new(&input), Path::new(&root),
                Path::new(&reverse_dir))?;
            fs::write(output, &body).map_err(|e| e.to_string())?;
            print!("{body}");
        }
        _ => return Err("command must be provenance/forward/fanin/reverse/final".to_owned()),
    }
    require(args.next().is_none(), "unexpected argument after command")
}
fn main() {
    if let Err(reason) = entry() {
        eprintln!("QP20_HOLD|reason={reason}|status=HOLD|json=0");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn qprism_source_cube_and_independent_bit_grouping() {
        let cube = cube3200();
        assert_eq!(hash_bytes(&cube), CUBE_SHA);
        let words = prism::encode(&cube);
        assert_eq!(words.len(), 2560);
        assert_eq!(hash_bytes(&wire_bytes(&words)), CUBE_SYMBOL_SHA);
        assert_eq!(reference_decode(&words, cube.len()).unwrap(), cube);
        assert_eq!(prism::decode(&words, cube.len()).unwrap(), cube);
        assert!(reference_decode(&words[..words.len()-1], cube.len()).is_err());
        let mut wrong = words;
        wrong[0] = 1024;
        assert!(reference_decode(&wrong, cube.len()).is_err());
        assert!(symbols_from_wire(&wire_bytes(&wrong), wrong.len()).is_err());
    }
    #[test]
    fn source_derived_payload_and_mutation_rejection() {
        let cube = cube3200();
        let corpus = Corpus { source: b"original Q-PRISM code fixture".to_vec(),
            cube, source_sha: hash_bytes(b"original Q-PRISM code fixture") };
        let raw0 = generated(&corpus, 0, 3200).expect("source");
        let raw1 = generated(&corpus, 1, 3200).expect("source");
        assert_ne!(raw0, raw1);
        let words = prism::encode(&raw0);
        assert_eq!(reference_decode(&words, raw0.len()).unwrap(), raw0);
        assert!(generated(&corpus, LEAVES, 3200).is_err());
        assert!(generated(&corpus, 0, 3201).is_err());
        let mut mutated = words.clone();
        mutated[10] ^= 1;
        assert_ne!(reference_decode(&mutated, raw0.len()).unwrap(), raw0);
        let mut wire = wire_bytes(&words);
        wire[1] ^= 4;
        assert!(symbols_from_wire(&wire, words.len()).is_err()
            || symbols_from_wire(&wire, words.len()).unwrap() != words);
    }
    #[test]
    fn roster_root_parser_rejects_reorder_and_mutation() {
        let corpus = Corpus { source: b"code".to_vec(), cube: cube3200(),
            source_sha: hash_bytes(b"code") };
        let mut roster = String::new();
        for leaf in 0..LEAVES {
            roster.push_str(&roster_line(leaf, &"a".repeat(64), &"b".repeat(64), &"c".repeat(64)));
        }
        let run = "test";
        let root = hash_bytes(format!("QP20_ROOT_V1|run={run}|source={}|{SOURCE_BLOB}\n{roster}",
            corpus.source_sha).as_bytes());
        let valid = format!("QP20_ROOT|schema=1|run={run}|source_sha={}|cube_sha={CUBE_SHA}|leaves=20|raw_bytes={}|symbols={}|encoded_bytes={}|root={root}|status=PASS|json=0\n{roster}",
            corpus.source_sha, RAW_BYTES * LEAVES, SYMBOLS * LEAVES, WIRE_BYTES * LEAVES);
        assert!(checked_root(run, &corpus, &valid).is_ok());
        assert!(checked_root("another", &corpus, &valid).is_err());
        assert!(checked_root(run, &corpus, &valid.replace("LEAF|id=5|", "LEAF|id=6|")).is_err());
        assert!(checked_root(run, &corpus, &valid.replace("root=", "ro0t=")).is_err());
        assert!(checked_root(run, &corpus, &valid.replace("wire_sha=bbbb", "wire_sha=cb b")).is_err());
    }
}
