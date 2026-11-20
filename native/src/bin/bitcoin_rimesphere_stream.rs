//! Original Bitcoin SHA256d CNF streamed through nested 27/81-cell RimeSpheres.
//! A finished stream is independently read back, checked against original
//! clauses, and frozen to standard GGUF v3 for later Fischer SAT play.
#![forbid(unsafe_code)]

use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use pnp_native::bitcoin_gguf::{freeze, play_variable, thaw};
use pnp_native::bitcoin_rimesphere_stream::{play, stream, BYTES_PER_SLICE, ROOMS_PER_SPHERE};
use pnp_native::sha256::hash_bytes;
use pnp_native::ternary::{Cnf, Trit};

const NONCE32: &str =
    "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";
const MERKLE288: &str =
    "601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b";
type Result<T> = std::result::Result<T, String>;

fn original(case: &str, data: &[u8]) -> Result<Cnf> {
    let (digest, variables, clauses) = match case {
        "nonce32" => (NONCE32, 239_112, 877_879),
        "merkle_nonce288" => (MERKLE288, 361_188, 1_325_357),
        _ => return Err("unknown authentic full Bitcoin SHA256d CNF".to_owned()),
    };
    if hash_bytes(data) != digest {
        return Err("original Bitcoin full-CNF source SHA256 differs".to_owned());
    }
    let text = std::str::from_utf8(data).map_err(|_| "Bitcoin DIMACS is not valid UTF8")?;
    let cnf = Cnf::parse(text)?;
    if cnf.variables != variables || cnf.clauses.len() != clauses
        || cnf.clauses.iter().any(Vec::is_empty)
    {
        return Err("original Bitcoin SAT variable/clause dimensions changed".to_owned());
    }
    Ok(cnf)
}

fn run(case: &str, source: &Path, output: &Path) -> Result<()> {
    fs::create_dir_all(output).map_err(|e| format!("output directory: {e}"))?;
    let started = Instant::now();
    // The pump holds only 729 unencoded source bytes at a time. The complete
    // archived CNF is authenticated after source-ordered RimeSphere replay.
    let mut original_file = BufReader::new(
        File::open(source).map_err(|e| format!("original input: {e}"))?
    );
    let sphere_path = output.join("BITCOIN.rime");
    let index_path = output.join("BITCOIN.hbi");
    let mut sphere_file = BufWriter::new(
        File::create(&sphere_path).map_err(|e| format!("sphere destination: {e}"))?
    );
    let mut index_file = BufWriter::new(
        File::create(&index_path).map_err(|e| format!("RimeSphere index destination: {e}"))?
    );
    let stream_started = Instant::now();
    let generated = stream(&mut original_file, &mut sphere_file, &mut index_file)?;
    sphere_file.flush().map_err(|e| format!("sphere flush: {e}"))?;
    index_file.flush().map_err(|e| format!("index flush: {e}"))?;
    drop(sphere_file);
    drop(index_file);
    let stream_ns = stream_started.elapsed().as_nanos();

    let replay_started = Instant::now();
    let mut frozen_reader = BufReader::new(
        File::open(&sphere_path).map_err(|e| format!("frozen sphere read: {e}"))?
    );
    let restored_path = output.join("BITCOIN.replayed.cnf");
    let mut recovered_writer = BufWriter::new(
        File::create(&restored_path).map_err(|e| format!("replay destination: {e}"))?
    );
    let played = play(&mut frozen_reader, &mut recovered_writer)?;
    recovered_writer.flush().map_err(|e| format!("replay flush: {e}"))?;
    drop(recovered_writer);
    if played != generated {
        return Err("streaming RimeSphere playback counters and root differ".to_owned());
    }
    let replay_ns = replay_started.elapsed().as_nanos();

    let original_data = fs::read(source).map_err(|e| format!("original source readback: {e}"))?;
    let recovered_data = fs::read(&restored_path)
        .map_err(|e| format!("reconstructed source readback: {e}"))?;
    if recovered_data != original_data {
        return Err("RimeSphere replay differs from authentic Bitcoin source bytes".to_owned());
    }
    let cnf = original(case, &original_data)?;
    let original_sha = hash_bytes(&original_data);
    let source_bytes = original_data.len();
    if generated.bytes != source_bytes as u64 {
        return Err("stream counter differs from original Bitcoin bytes".to_owned());
    }
    let gguf_started = Instant::now();
    let initial = vec![Trit::Zero; cnf.variables];
    let archive = freeze(&recovered_data, &initial)?;
    let gguf_sha = hash_bytes(&archive);
    let gguf_path = output.join("BITCOIN.rimesphere.gguf");
    fs::write(&gguf_path, &archive).map_err(|e| format!("GGUF checkpoint write: {e}"))?;
    let on_disk = fs::read(&gguf_path).map_err(|e| format!("GGUF readback: {e}"))?;
    let restored = thaw(&on_disk)?;
    if restored.source != original_data || restored.states != initial
        || restored.source_sha256 != original_sha
    {
        return Err("GGUF checkpoint did not preserve exact streamed CNF".to_owned());
    }
    for variable in [1, 2, 27, 28, 81, cnf.variables / 2, cnf.variables] {
        let (_, trit) = play_variable(&restored, variable)?;
        if trit != Trit::Zero {
            return Err("Fischer playback injected an unsourced SAT assignment".to_owned());
        }
    }
    let gguf_ns = gguf_started.elapsed().as_nanos();
    let total_ns = started.elapsed().as_nanos();
    let receipt = format!(
        "BITCOIN_RIMESPHERE_FINAL|schema=1|case={case}|source_sha256={original_sha}|\
         variables={}|clauses={}|source_bytes={source_bytes}|\
         slices={}|trits={}|rooms={}|sphere_arity=3-27-81|\
         source_bytes_per_gulp={BYTES_PER_SLICE}|rooms_per_sphere={ROOMS_PER_SPHERE}|\
         fischer_reads={}|sphere_bytes={}|sphere_root={}|\
         index_sha256={}|gguf_bytes={}|gguf_sha256={gguf_sha}|\
         streamed_source_sha256={}|stream_wall_ns={stream_ns}|\
         replay_wall_ns={replay_ns}|gguf_wall_ns={gguf_ns}|\
         total_wall_ns={total_ns}|reconstructed_source=BYTE_EXACT|\
         original_cnf=AUTHENTIC|known_nonce_injected=NO|\
         sat_search=NOT_RUN|checkpoint=READY|status=PASS|json=0\n",
        cnf.variables, cnf.clauses.len(), generated.slices,
        generated.trits, generated.rooms, generated.fischer_reads,
        generated.stored_bytes, generated.root,
        hash_bytes(&fs::read(&index_path).map_err(|e| format!("index readback: {e}"))?),
        archive.len(), hash_bytes(&recovered_data)
    );
    let receipt_path = output.join("BITCOIN_RIMESPHERE.hbp");
    fs::write(&receipt_path, &receipt).map_err(|e| format!("receipt write: {e}"))?;
    print!("{receipt}");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 || args[1] != "stream" {
        eprintln!("BITCOIN_RIME_FAIL|usage=bitcoin_rimesphere_stream stream <nonce32|merkle_nonce288> <original.cnf> <output-dir>|json=0");
        std::process::exit(1);
    }
    if let Err(error) = run(&args[2], Path::new(&args[3]), Path::new(&args[4])) {
        eprintln!("BITCOIN_RIME_FAIL|reason={error}|status=FAIL|json=0");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pnp_native::bitcoin_rimesphere_stream::{play, stream};
    use std::io::Cursor;

    #[test]
    fn actual_source_bytes_stream_to_nested_fischer_rooms_and_gguf() {
        let sample = b"p cnf 4 2\n1 -2 0\n3 4 0\n";
        let mut source = Cursor::new(sample);
        let mut stream_file = Vec::new();
        let mut lines = Vec::new();
        let stats = stream(&mut source, &mut stream_file, &mut lines)
            .expect("stream");
        let mut decoded = Vec::new();
        assert_eq!(play(&mut Cursor::new(&stream_file), &mut decoded)
                   .expect("played"), stats);
        assert_eq!(decoded, sample);
        let snap = freeze(sample, &[Trit::Zero; 4]).expect("checkpoint");
        let replayed = thaw(&snap).expect("gguf thaw");
        assert_eq!(replayed.source, decoded);
        assert_eq!(replayed.states, [Trit::Zero; 4]);
    }
}
