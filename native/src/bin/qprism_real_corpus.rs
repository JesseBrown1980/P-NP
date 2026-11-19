//! Retained public BEHCS word language and trained GNN checkpoints:
//! exact byte-preserving Q-PRISM / loopback HTTP / GitRAM artifact comparison.
//! Trained checkpoint transport is not neural inference.
#![forbid(unsafe_code)]

use pnp_native::{prism, sha256::hash_bytes};
use std::{
    collections::BTreeSet,
    env,
    fs,
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

const WORD_COMMIT: &str = "818cc235a6db815e3b2591339cf9b42aad3e85a0";
const GNN_COMMIT: &str = "c4b2458c88358c46155d3b308c594f9991720847";
const N: usize = 20;
const LIMIT: usize = 16 * 1024 * 1024;
const PREFIX: &[u8] = b"QP_RETAINED_CORPUS_V1\0";

struct Entry {
    source: usize,
    name: &'static str,
    blob: &'static str,
}
const FILES: [Entry; 14] = [
    Entry { source: 0, name: "data/behcs/codex/alphabet.json", blob: "8d4a3298576ce2c9d4a1683ffab667c8a743fc42" },
    Entry { source: 0, name: "data/behcs/codex/catalogs.json", blob: "51af0b536c45e8e769066bfd886bf6f08daff75d" },
    Entry { source: 0, name: "packages/deep-wave/src/cascade.ts", blob: "b9acdaff9b4537def53eca3d0fdb87a9c0dcab89" },
    Entry { source: 0, name: "packages/deep-wave/src/third-cascade.ts", blob: "a3e4485ca4a60d1b12cf6ed398c2f886ba358ea7" },
    Entry { source: 0, name: "tools/behcs/generate-falcon-glyph-language.js", blob: "2bf81fbc21a3851c4dbfcddeb227b0226e303836" },
    Entry { source: 1, name: "models/gslgnn_w9_3_seq47_v2.pt", blob: "e3a9712b50f8e7b31590b0aa944e6a9ba6ea565b" },
    Entry { source: 1, name: "models/gslgnn_w9_3_seq47.pt", blob: "6cddadc91ff4e6fa2bbfc315d90fa1bab0117c6b" },
    Entry { source: 1, name: "models/prototype_model.pt", blob: "1d1be27874e5fa4a7ef88026561033a0e0aac3fe" },
    Entry { source: 1, name: "models/contrastive_model.pt", blob: "76b7ec50d57e1a70cb9d755edffc75afc15e7a4b" },
    Entry { source: 1, name: "models/baseline_model.pt", blob: "26caf5216a4df3a43250e40e1dd421180ca3c901" },
    Entry { source: 1, name: "models/prototype_model_run2.pt", blob: "3660b469f648aa4287b9f6a19732543a22e72ccc" },
    Entry { source: 1, name: "models/contrastive_model_run2.pt", blob: "6ab8743e0e3dca29de7387443a618272a2c0507f" },
    Entry { source: 1, name: "manifests/gslgnn_w9_3_seq47_v2_manifest.json", blob: "f1eb3675ab5e8822d1511ce4b3062d51ee50b2dd" },
    Entry { source: 1, name: "src/gsl_gnn.py", blob: "886b3b0c0cdbddba983fa8c3ae083c4520d38f0e" },
];

fn require(ok: bool, reason: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(reason.to_owned()) }
}
fn run_id(run: &str) -> Result<(), String> {
    require(!run.is_empty() && run.len() <= 128 && run.bytes().all(|b|
        b.is_ascii_alphanumeric() || b"._-".contains(&b)),
        "invalid run identity")
}
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let result = Command::new("git").arg("-C").arg(root).args(args).output()
        .map_err(|e| format!("source git is unavailable: {e}"))?;
    require(result.status.success(), "Git source validation failed")?;
    String::from_utf8(result.stdout)
        .map(|s| s.trim().to_owned()).map_err(|_| "invalid git output".to_owned())
}
fn authenticated_corpus(words: &Path, models: &Path) -> Result<Vec<u8>, String> {
    let roots = [words, models];
    require(git(words, &["rev-parse", "HEAD"])? == WORD_COMMIT,
        "word language commit drift")?;
    require(git(models, &["rev-parse", "HEAD"])? == GNN_COMMIT,
        "trained model commit drift")?;
    let mut raw = Vec::new();
    raw.extend_from_slice(PREFIX);
    raw.extend_from_slice(&(FILES.len() as u32).to_le_bytes());
    for entry in FILES {
        let root = roots[entry.source];
        require(git(root, &["hash-object", entry.name])? == entry.blob,
            "retained artifact Git blob drift")?;
        let body = fs::read(root.join(entry.name))
            .map_err(|e| format!("source read failure: {e}"))?;
        require(!body.is_empty() && body.len() <= LIMIT,
            "source file out of accepted range")?;
        let path = format!("{}:{}", entry.source, entry.name);
        require(path.len() <= usize::from(u16::MAX), "source identity too long")?;
        raw.extend_from_slice(&(path.len() as u16).to_le_bytes());
        raw.extend_from_slice(path.as_bytes());
        raw.extend_from_slice(&(body.len() as u64).to_le_bytes());
        raw.extend_from_slice(&body);
    }
    require(raw.len() <= LIMIT, "retained corpus exceeds authenticated capacity")?;
    Ok(raw)
}
fn symbol_bytes(values: &[u16]) -> Vec<u8> {
    let mut output = Vec::with_capacity(values.len() * 2);
    for v in values { output.extend_from_slice(&v.to_le_bytes()); }
    output
}
fn parse_symbols(wire: &[u8], nbytes: usize) -> Result<Vec<u16>, String> {
    let expected = nbytes.checked_mul(8).ok_or("size overflow")?.div_ceil(10);
    require(wire.len() == expected * 2, "incorrect encoded transport length")?;
    let mut symbols = Vec::with_capacity(expected);
    for pair in wire.chunks_exact(2) {
        let glyph = u16::from_le_bytes([pair[0], pair[1]]);
        require(glyph < 1024, "out-of-range BEHCS-1024 symbol")?;
        symbols.push(glyph);
    }
    Ok(symbols)
}
fn independent_inverse(symbols: &[u16], nbytes: usize) -> Result<Vec<u8>, String> {
    require(symbols.len() == nbytes.checked_mul(8).ok_or("length overflow")?.div_ceil(10),
        "independent inverse framing failed")?;
    let mut recovered = Vec::with_capacity(nbytes);
    let mut remaining = nbytes;
    for block in symbols.chunks(4) {
        let batch_bytes = remaining.min(5);
        let expected_symbols = (batch_bytes * 8).div_ceil(10);
        require(block.len() == expected_symbols, "truncated ten-bit block")?;
        let mut bits = 0_u64;
        for &glyph in block {
            require(glyph < 1024, "independent inverse rejected symbol")?;
            bits = (bits << 10) | u64::from(glyph);
        }
        let padding = expected_symbols * 10 - batch_bytes * 8;
        require(bits & ((1_u64 << padding) - 1) == 0,
            "noncanonical final-symbol padding")?;
        let value = bits >> padding;
        for shift in (0..batch_bytes).rev() {
            recovered.push(((value >> (shift * 8)) & 255) as u8);
        }
        remaining -= batch_bytes;
    }
    require(remaining == 0, "independent inverse did not reconstruct all bytes")?;
    Ok(recovered)
}
fn decode_wire(wire: &[u8], source: &[u8]) -> Result<(), String> {
    let symbols = parse_symbols(wire, source.len())?;
    require(prism::decode(&symbols, source.len()).map_err(str::to_owned)? == source,
        "native 1024 to 256 inversion failed")?;
    require(independent_inverse(&symbols, source.len())? == source,
        "independent 40-bit decoder failed")?;
    require(symbol_bytes(&prism::encode(source)) == wire,
        "noncanonical glyph transport representation")?;
    Ok(())
}
fn expected_forward(run: &str, leaf: usize, source: &[u8], wire: &[u8]) -> String {
    format!(
        "QPR_REAL_FORWARD|schema=1|run={run}|leaf={leaf}|to={}|word_commit={WORD_COMMIT}|gnn_commit={GNN_COMMIT}|files=14|raw_len={}|glyphs={}|wire_len={}|raw_sha={}|wire_sha={}|source=PUBLIC_RETAINED_FILE_BYTES|model_inference=NOT_RUN|status=PASS|json=0\n",
        (leaf + 1) % N, source.len(), prism::encode(source).len(), wire.len(),
        hash_bytes(source), hash_bytes(wire)
    )
}
fn wire_path(dir: &Path, leaf: usize) -> PathBuf {
    dir.join(format!("real-symbol-{leaf:02}.bin"))
}
fn forward_path(dir: &Path, leaf: usize) -> PathBuf {
    dir.join(format!("real-forward-{leaf:02}.hbp"))
}
fn reverse_path(dir: &Path, leaf: usize) -> PathBuf {
    dir.join(format!("real-reverse-{leaf:02}.hbp"))
}
fn forward(run: &str, leaf: usize, source: &[u8], dir: &Path) -> Result<String, String> {
    run_id(run)?;
    require(leaf < N, "forward leaf out of range")?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let wire = symbol_bytes(&prism::encode(source));
    decode_wire(&wire, source)?;
    fs::write(wire_path(dir, leaf), &wire).map_err(|e| e.to_string())?;
    let receipt = expected_forward(run, leaf, source, &wire);
    fs::write(forward_path(dir, leaf), &receipt).map_err(|e| e.to_string())?;
    Ok(receipt)
}
fn verify_forward(run: &str, leaf: usize, source: &[u8], dir: &Path)
    -> Result<(String, String), String>
{
    let wire = fs::read(wire_path(dir, leaf)).map_err(|e| e.to_string())?;
    let recorded = fs::read_to_string(forward_path(dir, leaf))
        .map_err(|e| e.to_string())?;
    decode_wire(&wire, source)?;
    require(recorded == expected_forward(run, leaf, source, &wire),
        "foreign or modified forward source receipt")?;
    Ok((hash_bytes(&wire), hash_bytes(recorded.as_bytes())))
}
fn roster(dir: &Path, phase: &str) -> Result<(), String> {
    let mut present = BTreeSet::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let e = entry.map_err(|e| e.to_string())?;
        require(e.file_type().map_err(|e| e.to_string())?.is_file(),
            "non-file in accepted transport roster")?;
        present.insert(e.file_name().into_string()
            .map_err(|_| "non-UTF8 transport name".to_owned())?);
    }
    let expected: BTreeSet<String> = if phase == "forward" {
        (0..N).flat_map(|leaf| [
            format!("real-symbol-{leaf:02}.bin"),
            format!("real-forward-{leaf:02}.hbp"),
        ]).collect()
    } else {
        (0..N).map(|leaf| format!("real-reverse-{leaf:02}.hbp")).collect()
    };
    require(present == expected, "unexpected, missing or duplicated transport artifact")
}
fn expected_root(run: &str, source: &[u8]) -> String {
    let wire = symbol_bytes(&prism::encode(source));
    let wire_sha = hash_bytes(&wire);
    let mut body = format!(
        "QPR_REAL_ROOT|schema=1|run={run}|word_commit={WORD_COMMIT}|gnn_commit={GNN_COMMIT}|source_sha={}|wire_sha={wire_sha}|raw_len={}|wire_len={}|leaves=20|replica_semantics=IDENTICAL_PINNED_CORPUS|status=PASS|json=0\n",
        hash_bytes(source), source.len(), wire.len()
    );
    for leaf in 0..N {
        let receipt = expected_forward(run, leaf, source, &wire);
        body.push_str(&format!(
            "LEAF|id={leaf}|raw_sha={}|wire_sha={wire_sha}|receipt_sha={}|json=0\n",
            hash_bytes(source), hash_bytes(receipt.as_bytes())
        ));
    }
    body
}
fn fanin(run: &str, source: &[u8], dir: &Path) -> Result<String, String> {
    roster(dir, "forward")?;
    for leaf in 0..N { verify_forward(run, leaf, source, dir)?; }
    Ok(expected_root(run, source))
}
fn validated_root(run: &str, source: &[u8], root: &Path) -> Result<String, String> {
    let body = fs::read_to_string(root).map_err(|e| e.to_string())?;
    require(body == expected_root(run, source), "source-root not canonical")?;
    Ok(hash_bytes(body.as_bytes()))
}
fn expected_reverse(run: &str, leaf: usize, source: &[u8], root_sha: &str) -> String {
    let sender = (leaf + N - 1) % N;
    let wire = symbol_bytes(&prism::encode(source));
    format!(
        "QPR_REAL_REVERSE|schema=1|run={run}|receiver={leaf}|sender={sender}|root_sha={root_sha}|source_sha={}|payload_sha={}|raw_len={}|glyphs={}|decode=NATIVE_AND_INDEPENDENT_40BIT|restored=EXACT|status=PASS|json=0\n",
        hash_bytes(source), hash_bytes(&wire), source.len(), prism::encode(source).len()
    )
}
fn reverse(run: &str, leaf: usize, source: &[u8], dir: &Path, root: &Path)
    -> Result<String, String>
{
    require(leaf < N, "reverse receiver out of range")?;
    let root_sha = validated_root(run, source, root)?;
    let sender = (leaf + N - 1) % N;
    verify_forward(run, sender, source, dir)?;
    Ok(expected_reverse(run, leaf, source, &root_sha))
}
fn final_receipt(run: &str, source: &[u8], dir: &Path, root: &Path,
                 reverse_dir: &Path, benchmark: &Path) -> Result<String, String>
{
    roster(dir, "forward")?;
    roster(reverse_dir, "reverse")?;
    let expected = fanin(run, source, dir)?;
    require(fs::read_to_string(root).map_err(|e| e.to_string())? == expected,
        "final fan-in root mismatch")?;
    let root_sha = validated_root(run, source, root)?;
    let bench_body = fs::read_to_string(benchmark).map_err(|e| e.to_string())?;
    require(bench_body.contains(&format!("source_sha={}", hash_bytes(source))) &&
        bench_body.contains(&format!("wire_sha={}", hash_bytes(&symbol_bytes(&prism::encode(source))))) &&
        bench_body.contains("|http_loopback=PASS|") &&
        bench_body.contains("|native=PASS|"), "baseline is missing or from another corpus")?;
    let mut reverse_digests = String::new();
    for leaf in 0..N {
        let actual = fs::read_to_string(reverse_path(reverse_dir, leaf))
            .map_err(|e| e.to_string())?;
        require(actual == expected_reverse(run, leaf, source, &root_sha),
            "independent reverse receipt changed")?;
        reverse_digests.push_str(&hash_bytes(actual.as_bytes()));
        reverse_digests.push('\n');
    }
    let reverse_sha = hash_bytes(reverse_digests.as_bytes());
    let wire_len = symbol_bytes(&prism::encode(source)).len();
    Ok(format!(
        "QPR_REAL_FINAL|schema=1|run={run}|source_sha={}|wire_sha={}|root_sha={root_sha}|reverse_root={reverse_sha}|input_files=14|input_len={}|wire_len={wire_len}|forward_workers=20|reverse_workers=20|replicas=20|source_unique_bytes={}|transported_bytes={}|independent_inversion=PASS|native=PASS|http_loopback=PASS|gitram_artifacts=PASS|model_inference=NOT_RUN|status=PASS|json=0\n",
        hash_bytes(source), hash_bytes(&symbol_bytes(&prism::encode(source))),
        source.len(), source.len(), wire_len * N
    ))
}
fn http_loopback(wire: &[u8], source: &[u8]) -> Result<u128, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let raw_len = source.len();
    let digest = hash_bytes(source);
    let wire_len = wire.len();
    let expected_digest = digest.clone();
    let task = thread::spawn(move || -> Result<(), String> {
        let (mut stream, _) = listener.accept().map_err(|e| e.to_string())?;
        stream.set_read_timeout(Some(Duration::from_secs(20)))
            .map_err(|e| e.to_string())?;
        stream.set_write_timeout(Some(Duration::from_secs(20)))
            .map_err(|e| e.to_string())?;
        let mut header = Vec::new();
        while !header.ends_with(b"\r\n\r\n") {
            require(header.len() <= 512, "HTTP request header too long")?;
            let mut next = [0_u8; 1];
            stream.read_exact(&mut next).map_err(|e| e.to_string())?;
            header.push(next[0]);
        }
        let expect = format!("POST /qprism HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {wire_len}\r\nConnection: close\r\n\r\n");
        require(header == expect.as_bytes(), "HTTP0-style loopback header changed")?;
        let mut body = vec![0_u8; wire_len];
        stream.read_exact(&mut body).map_err(|e| e.to_string())?;
        let symbols = parse_symbols(&body, raw_len)?;
        let native = prism::decode(&symbols, raw_len).map_err(str::to_owned)?;
        let alternate = independent_inverse(&symbols, raw_len)?;
        require(native == alternate && hash_bytes(&native) == expected_digest,
            "loopback receiver changed the retained source bytes")?;
        let response = format!("HTTP/1.1 200 OK\r\nContent-Length: 64\r\nConnection: close\r\n\r\n{expected_digest}");
        stream.write_all(response.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    });
    let start = Instant::now();
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(20))).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(20))).map_err(|e| e.to_string())?;
    let head = format!("POST /qprism HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", wire.len());
    stream.write_all(head.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(wire).map_err(|e| e.to_string())?;
    stream.shutdown(Shutdown::Write).map_err(|e| e.to_string())?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(|e| e.to_string())?;
    require(response == format!("HTTP/1.1 200 OK\r\nContent-Length: 64\r\nConnection: close\r\n\r\n{digest}").as_bytes(),
        "HTTP loopback reply identity changed")?;
    task.join().map_err(|_| "loopback server thread failed".to_owned())??;
    Ok(start.elapsed().as_nanos())
}
fn bench(source: &[u8]) -> Result<String, String> {
    let start = Instant::now();
    let symbols = prism::encode(source);
    let wire = symbol_bytes(&symbols);
    let encode_ns = start.elapsed().as_nanos();
    let start = Instant::now();
    decode_wire(&wire, source)?;
    let native_decode_ns = start.elapsed().as_nanos();
    let loopback_ns = http_loopback(&wire, source)?;
    Ok(format!(
        "QPR_REAL_BENCH|schema=1|source_sha={}|wire_sha={}|files=14|source_len={}|wire_len={}|encode_ns={encode_ns}|native_decode_ns={native_decode_ns}|http_loopback_ns={loopback_ns}|http_loopback=PASS|native=PASS|source_mode=RETAINED_PUBLIC_FILES|status=PASS|json=0\n",
        hash_bytes(source), hash_bytes(&wire), source.len(), wire.len()
    ))
}
fn arg(args: &mut impl Iterator<Item = String>, what: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("missing {what}"))
}
fn entry() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = arg(&mut args, "command")?;
    let run = arg(&mut args, "run key")?;
    run_id(&run)?;
    let words = arg(&mut args, "source word checkout")?;
    let models = arg(&mut args, "source GNN checkout")?;
    let source = authenticated_corpus(Path::new(&words), Path::new(&models))?;
    match command.as_str() {
        "bench" => {
            let output = arg(&mut args, "benchmark receipt path")?;
            let report = bench(&source)?;
            fs::write(output, &report).map_err(|e| e.to_string())?;
            print!("{report}");
        }
        "forward" => {
            let leaf: usize = arg(&mut args, "leaf")?.parse()
                .map_err(|_| "invalid leaf id".to_owned())?;
            let directory = arg(&mut args, "artifact directory")?;
            let receipt = forward(&run, leaf, &source, Path::new(&directory))?;
            print!("{receipt}");
        }
        "fanin" => {
            let directory = arg(&mut args, "forward directory")?;
            let output = arg(&mut args, "root output")?;
            let report = fanin(&run, &source, Path::new(&directory))?;
            fs::write(output, &report).map_err(|e| e.to_string())?;
            print!("{}", report.lines().next().unwrap_or(""));
        }
        "reverse" => {
            let leaf: usize = arg(&mut args, "receiver")?.parse()
                .map_err(|_| "invalid receiver".to_owned())?;
            let dir = arg(&mut args, "forward directory")?;
            let root = arg(&mut args, "root")?;
            let output = arg(&mut args, "reverse receipt")?;
            let report = reverse(&run, leaf, &source, Path::new(&dir), Path::new(&root))?;
            fs::write(output, &report).map_err(|e| e.to_string())?;
            print!("{report}");
        }
        "final" => {
            let forward_dir = arg(&mut args, "forward directory")?;
            let root = arg(&mut args, "root")?;
            let reverse_dir = arg(&mut args, "reverse directory")?;
            let benchmark = arg(&mut args, "baseline benchmark")?;
            let output = arg(&mut args, "final receipt")?;
            let result = final_receipt(&run, &source, Path::new(&forward_dir),
                Path::new(&root), Path::new(&reverse_dir), Path::new(&benchmark))?;
            fs::write(output, &result).map_err(|e| e.to_string())?;
            print!("{result}");
        }
        _ => return Err("commands: bench, forward, fanin, reverse, final".to_owned()),
    }
    require(args.next().is_none(), "unexpected extra arguments")
}
fn main() {
    if let Err(reason) = entry() {
        eprintln!("QPR_REAL_HOLD|reason={reason}|status=HOLD|json=0");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_qprism_golden_and_non_aligned_input() {
        let bytes = b"actual retained\x00word and weights\xff";
        let symbols = prism::encode(bytes);
        let wire = symbol_bytes(&symbols);
        decode_wire(&wire, bytes).expect("bitstream inverse");
        assert_eq!(independent_inverse(&symbols, bytes.len()).unwrap(), bytes);
        assert!(decode_wire(&wire, b"changed").is_err());
        let mut damaged = wire;
        damaged[0] ^= 1;
        assert!(decode_wire(&damaged, bytes).is_err());
    }
    #[test]
    fn wrong_padding_and_truncated_symbols_fail() {
        assert!(independent_inverse(&[1021], 1).is_err());
        assert!(independent_inverse(&[1024], 1).is_err());
        assert!(independent_inverse(&[0], 2).is_err());
        assert!(parse_symbols(&[0,0,1], 1).is_err());
    }
}
