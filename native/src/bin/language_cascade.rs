//! Source-word Deep Wave 93,312-point native execution/search-lane acceptance.
//! This is an exact word/route witness, not the trained GNN forward pass.
#![forbid(unsafe_code)]

use pnp_native::behcs_glyph_language;
use pnp_native::prism;
use pnp_native::sha256::hash_bytes;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;

const N: usize = 20;
const WAVES: usize = 12;
const TOTAL: usize = 6 * 6 * 6 * 6 * 6 * WAVES;
const BUCKETS: usize = 6 * 6 * WAVES;
const SOURCE_BLOB: &str = "b9acdaff9b4537def53eca3d0fdb87a9c0dcab89";
const THIRD_BLOB: &str = "a3e4485ca4a60d1b12cf6ed398c2f886ba358ea7";
const CODEX_BLOB: &str = "8d4a3298576ce2c9d4a1683ffab667c8a743fc42";
const WORDS_BLOB: &str = "2bf81fbc21a3851c4dbfcddeb227b0226e303836";
const ORIGIN: &str = "818cc235a6db815e3b2591339cf9b42aad3e85a0";

// Exact dimension terms from packages/deep-wave/src/cascade.ts, frozen source.
// Words are typed terms in a 6-axis language, not anonymous byte letters.
const ACTORS: [&str; 6] = ["acer", "liris", "falcon", "felipe", "beast", "dan"];
const VERBS: [&str; 6] = ["scan-dispatch", "scan-result", "drift-detected",
    "cosign-append", "migration-intent", "heartbeat"];
const TARGETS: [&str; 6] = ["acer", "liris", "falcon", "federation", "phantom", "witness"];
const STATES: [&str; 6] = ["pending", "in-flight", "dispatched",
    "settled", "failed", "observed"];

fn ensure(test: bool, message: &str) -> Result<(), String> {
    if test { Ok(()) } else { Err(message.to_owned()) }
}
fn valid(run: &str) -> Result<(), String> {
    ensure(!run.is_empty() && run.len() <= 128 &&
        run.bytes().all(|v| v.is_ascii_alphanumeric() || b"-._".contains(&v)),
        "run identifier changed or contains delimiters")
}
fn path(dir: &Path, kind: &str, leaf: usize) -> std::path::PathBuf {
    dir.join(format!("{kind}-{leaf:02}.hbp"))
}
fn point(index: usize) -> Result<(String, usize), String> {
    ensure(index < TOTAL, "word coordinate outside 93,312-point grid")?;
    let mut v = index;
    let wave = v % WAVES; v /= WAVES;
    let state = v % 6; v /= 6;
    let risk = v % 6; v /= 6;
    let target = v % 6; v /= 6;
    let verb = v % 6; v /= 6;
    let actor = v;
    ensure(actor < 6, "actor beyond source dimension")?;
    let word = format!(
        "D1:{}/D2:{}/D3:{}/D4:{risk}/D5:{}/D6:w{wave}",
        ACTORS[actor], VERBS[verb], TARGETS[target], STATES[state],
    );
    Ok((word, (verb * 6 + target) * WAVES + wave))
}
fn bytes_glyph_roundtrip(word: &str) -> Result<usize, String> {
    let bytes = word.as_bytes();
    let symbols = prism::encode(bytes);
    let restored = prism::decode(&symbols, bytes.len()).map_err(str::to_owned)?;
    ensure(restored == bytes, "word-level Q-PRISM 256-to-1024 reconstruction changed")?;
    ensure(prism::encode(&restored) == symbols, "noncanonical glyph representation")?;
    Ok(symbols.len())
}
fn counts_line(counts: &[u32; BUCKETS]) -> String {
    counts.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
}
fn shard(run: &str, leaf: usize) -> Result<String, String> {
    valid(run)?;
    ensure(leaf < N, "leaf outside twenty-runner range")?;
    let mut buckets = [0_u32; BUCKETS];
    let mut words = 0_usize;
    let mut bytes = 0_usize;
    let mut glyphs = 0_usize;
    let mut fingerprint = format!("WAVE_WORDS_V2|{run}|{leaf}|{SOURCE_BLOB}|{ORIGIN}\n");
    for index in (leaf..TOTAL).step_by(N) {
        let (word, bucket) = point(index)?;
        glyphs += bytes_glyph_roundtrip(&word)?;
        let address = behcs_glyph_language::address(&word)
            .ok_or("missing original 256-symbol word address")?;
        ensure(behcs_glyph_language::verify_word(&word, &address),
            "source word/glyph address mapping altered")?;
        bytes += word.len();
        words += 1;
        buckets[bucket] += 1;
        let address_sha = hash_bytes(address.as_bytes());
        fingerprint.push_str(&format!("{index}|{word}|glyph_sha256={address_sha}\n"));
    }
    let counts = counts_line(&buckets);
    let source_digest = hash_bytes(fingerprint.as_bytes());
    let bins_sha = hash_bytes(counts.as_bytes());
    Ok(format!(
        "CW_SHARD|schema=1|run={run}|leaf={leaf}|source_commit={ORIGIN}|source_blob={SOURCE_BLOB}|codex_blob={CODEX_BLOB}|third_blob={THIRD_BLOB}|words_blob={WORDS_BLOB}|events={words}|word_bytes={bytes}|glyphs={glyphs}|word_sha={source_digest}|buckets_sha={bins_sha}|status=PASS|json=0\nBUCKETS|counts={counts}|json=0\n",
    ))
}
fn recheck_shard(run: &str, leaf: usize, body: &str) -> Result<[u32; BUCKETS], String> {
    ensure(body == shard(run, leaf)?, "source-word checkpoint not canonical")?;
    let counts = body.lines().nth(1).ok_or("missing search histogram")?;
    let str_counts = counts.strip_prefix("BUCKETS|counts=")
        .and_then(|s| s.strip_suffix("|json=0")).ok_or("invalid bucket row")?;
    let v = str_counts.split(',').map(|s| s.parse::<u32>()
        .map_err(|_| "bad bucket tally".to_owned())).collect::<Result<Vec<_>,_>>()?;
    ensure(v.len() == BUCKETS, "search bucket count mismatch")?;
    let mut out = [0_u32; BUCKETS];
    out.copy_from_slice(&v);
    Ok(out)
}
fn fanin_root(run: &str, folder: &Path) -> Result<String, String> {
    valid(run)?;
    ensure(fs::read_dir(folder).map_err(|e| e.to_string())?.count() == N,
        "missing, duplicated or unexpected leaf file")?;
    let mut totals = [0_u32; BUCKETS];
    let mut commits = vec![format!("CW_ROOT_V2|{run}|{ORIGIN}|{SOURCE_BLOB}")];
    let mut point_count = 0_usize;
    for leaf in 0..N {
        let body = fs::read_to_string(path(folder, "language", leaf)).map_err(|e| e.to_string())?;
        let bins = recheck_shard(run, leaf, &body)?;
        point_count += (leaf..TOTAL).step_by(N).count();
        for (total, count) in totals.iter_mut().zip(bins) {
            *total += count;
        }
        commits.push(hash_bytes(body.as_bytes()));
    }
    ensure(point_count == TOTAL, "execution lanes did not cover all events")?;
    ensure(totals.iter().all(|&x| x == 216), "word search convergence changed")?;
    let mut address_witness = BTreeSet::new();
    for index in 0..TOTAL {
        let (word, _) = point(index)?;
        let address = behcs_glyph_language::address(&word)
            .ok_or("source word has no address")?;
        ensure(address_witness.insert(address), "original 64-bit glyph address alias")?;
    }
    ensure(address_witness.len() == TOTAL, "source-word glyph address coverage changed")?;
    // A 216:1 projection is intentionally many-to-one. Full source-word
    // identities remain recoverable from their pinned coordinate/corpus.
    Ok(hash_bytes(commits.join("|").as_bytes()))
}
fn root_line(run: &str, root: &str) -> String {
    format!("CW_ROOT|schema=1|run={run}|source={ORIGIN}|leaves=20|events=93312|buckets=432|hits_per_bucket=216|root={root}|status=PASS|json=0\n")
}
fn verify_root(run: &str, dir: &Path, root_file: &Path) -> Result<String, String> {
    let root = fanin_root(run, dir)?;
    ensure(fs::read_to_string(root_file).map_err(|e| e.to_string())? == root_line(run, &root),
        "language wave root is not the accepted source set")?;
    Ok(root)
}
fn reverse_row(run: &str, leaf: usize, dir: &Path, root_file: &Path) -> Result<String, String> {
    ensure(leaf < N, "invalid reverse leaf")?;
    let root = verify_root(run, dir, root_file)?;
    let from = (leaf + N - 1) % N;
    let body = fs::read_to_string(path(dir, "language", from)).map_err(|e| e.to_string())?;
    recheck_shard(run, from, &body)?;
    let expected = shard(run, from)?;
    ensure(body == expected, "word-level reverse reconstruction disagrees")?;
    let count = (from..TOTAL).step_by(N).count();
    Ok(format!(
        "CW_REVERSE|schema=1|run={run}|leaf={leaf}|from={from}|source={ORIGIN}|parent_root={root}|source_receipt_sha={}|restored_words={count}|restoration=EXACT_FROM_PINNED_WORDS|status=PASS|json=0\n",
        hash_bytes(body.as_bytes()),
    ))
}
fn final_row(run: &str, dir: &Path, root_file: &Path, reverse_dir: &Path) -> Result<String, String> {
    let root = verify_root(run, dir, root_file)?;
    ensure(fs::read_dir(reverse_dir).map_err(|e| e.to_string())?.count() == N,
        "missing or extra reverse word receipts")?;
    let mut receipts = vec![format!("CW_REVERSE_ROOT_V2|{run}|{root}")];
    for leaf in 0..N {
        let actual = fs::read_to_string(path(reverse_dir, "language-reverse", leaf))
            .map_err(|e| e.to_string())?;
        ensure(actual == reverse_row(run, leaf, dir, root_file)?, "reverse word provenance diverged")?;
        receipts.push(hash_bytes(actual.as_bytes()));
    }
    let reverse = hash_bytes(receipts.join("|").as_bytes());
    Ok(format!(
        "CW_FINAL|schema=1|run={run}|source={ORIGIN}|forward_root={root}|reverse_root={reverse}|events=93312|word_dimension=ACTOR_VERB_TARGET_RISK_STATE_WAVE|glyph_language=ORIGINAL_256_GLYPHS|unique_execution=93312|unique_glyph_addresses=93312|search_buckets=432|collisions_per_bucket=216|reverse_leaves=20|word_reconstruction=EXACT_FROM_PINNED_WORDS|prism=ROUNDTRIP|trained_gnn=SEPARATE_MODEL_GATE|status=PASS|json=0\n"
    ))
}
fn take(args: &mut impl Iterator<Item=String>, what: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("missing {what}"))
}
fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let cmd = take(&mut args, "command")?;
    match cmd.as_str() {
        "provenance" => {
            let (word, _) = point(0)?;
            ensure(word == "D1:acer/D2:scan-dispatch/D3:acer/D4:0/D5:pending/D6:w0", "original source-word first point changed")?;
            println!("CW_SOURCE|commit={ORIGIN}|second_blob={SOURCE_BLOB}|third_blob={THIRD_BLOB}|codex_blob={CODEX_BLOB}|glyph_words_blob={WORDS_BLOB}|shape=6x6x6x6x6x12|points=93312|status=PASS|json=0");
        }
        "shard" => {
            let run = take(&mut args, "run")?;
            let leaf: usize = take(&mut args, "leaf")?.parse().map_err(|_| "invalid leaf".to_owned())?;
            let out = take(&mut args, "output")?;
            fs::write(out, shard(&run, leaf)?).map_err(|e| e.to_string())?;
            println!("CW_SHARD_PASS|leaf={leaf}|status=PASS|json=0");
        }
        "fanin" => {
            let run = take(&mut args, "run")?;
            let dir = take(&mut args, "directory")?;
            let output = take(&mut args, "root")?;
            let root = fanin_root(&run, Path::new(&dir))?;
            fs::write(output, root_line(&run, &root)).map_err(|e| e.to_string())?;
            println!("CW_FANIN_PASS|root={root}|status=PASS|json=0");
        }
        "reverse" => {
            let run = take(&mut args, "run")?;
            let leaf: usize = take(&mut args, "leaf")?.parse().map_err(|_| "invalid leaf".to_owned())?;
            let dir = take(&mut args, "directory")?;
            let root = take(&mut args, "root")?;
            let output = take(&mut args, "output")?;
            fs::write(output, reverse_row(&run, leaf, Path::new(&dir), Path::new(&root))?)
                .map_err(|e| e.to_string())?;
            println!("CW_REVERSE_PASS|leaf={leaf}|status=PASS|json=0");
        }
        "final" => {
            let run = take(&mut args, "run")?;
            let dir = take(&mut args, "directory")?;
            let root = take(&mut args, "root")?;
            let reverse_dir = take(&mut args, "reverse dir")?;
            let output = take(&mut args, "output")?;
            let receipt = final_row(&run, Path::new(&dir), Path::new(&root), Path::new(&reverse_dir))?;
            fs::write(output, &receipt).map_err(|e| e.to_string())?;
            print!("{receipt}");
        }
        _ => return Err("unsupported wave language command".to_owned()),
    }
    ensure(args.next().is_none(), "unexpected arguments")
}
fn main() {
    if let Err(err) = run() {
        eprintln!("CW_HOLD|reason={err}|status=HOLD|json=0");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn word_level_source_geometry_and_failure_controls() {
        assert_eq!(TOTAL, 93_312);
        let (first, bucket) = point(0).expect("first word");
        assert_eq!(first, "D1:acer/D2:scan-dispatch/D3:acer/D4:0/D5:pending/D6:w0");
        assert_eq!(bucket, 0);
        let (last, b) = point(TOTAL - 1).expect("last word");
        assert_eq!(last, "D1:dan/D2:heartbeat/D3:witness/D4:5/D5:observed/D6:w11");
        assert_eq!(b, 431);
        let a = shard("test", 0).expect("first shard");
        assert!(recheck_shard("test", 0, &a).is_ok());
        assert!(recheck_shard("other", 0, &a).is_err());
        assert!(recheck_shard("test", 1, &a).is_err());
        let mut corrupt = a.into_bytes();
        let n = corrupt.len() - 12;
        corrupt[n] ^= 1;
        assert!(recheck_shard("test", 0, &String::from_utf8(corrupt).expect("ASCII")).is_err());
    }
}
