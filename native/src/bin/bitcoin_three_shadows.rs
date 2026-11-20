//! Apply the operator's three-light/three-shadow 3D→2D method to
//! every byte of an already source-verified streamed Bitcoin RimeSphere.
//! Three exact orthogonal 3x3 signed projections plus eight independent
//! residual trits reconstruct each full 27-cell room, byte-for-byte.
#![forbid(unsafe_code)]

use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use pnp_native::bitcoin_gguf::{freeze, thaw};
use pnp_native::sha256::hash_bytes;
use pnp_native::ternary::{Cnf, Trit};
use pnp_native::three_shadow::{freeze_stream, play_stream, EXPECTED_RANK, RESIDUALS, SHADOW_BYTES};

const NONCE32: &str =
    "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";
const MERKLE288: &str =
    "601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b";

type Result<T> = std::result::Result<T, String>;

fn authentic(case: &str, source: &[u8]) -> Result<Cnf> {
    let (sha, variables, clauses) = match case {
        "nonce32" => (NONCE32, 239_112, 877_879),
        "merkle_nonce288" => (MERKLE288, 361_188, 1_325_357),
        _ => return Err("unknown original Bitcoin SHA256d CNF case".to_owned()),
    };
    if hash_bytes(source) != sha {
        return Err("three-shadow experiment requires the full original Bitcoin CNF".to_owned());
    }
    let text = std::str::from_utf8(source).map_err(|_| "original Bitcoin source UTF8")?;
    let cnf = Cnf::parse(text)?;
    if cnf.variables != variables || cnf.clauses.len() != clauses {
        return Err("source variable/clause dimensions changed".to_owned());
    }
    Ok(cnf)
}

fn run(case: &str, source: &Path, prior: &Path, output: &Path) -> Result<()> {
    let started = Instant::now();
    let raw = fs::read(source).map_err(|e| format!("read original Bitcoin source: {e}"))?;
    let cnf = authentic(case, &raw)?;
    let input = fs::read(prior.join("BITCOIN.replayed.cnf"))
        .map_err(|e| format!("read prior source reconstruction: {e}"))?;
    if input != raw {
        return Err("accepted prior RimeSphere replay differs from original CNF".to_owned());
    }
    let prior_gguf = fs::read(prior.join("BITCOIN.rimesphere.gguf"))
        .map_err(|e| format!("read prior original Bitcoin GGUF: {e}"))?;
    let prior_state = thaw(&prior_gguf)?;
    if prior_state.source != raw || prior_state.states != vec![Trit::Zero; cnf.variables] {
        return Err("prior frozen original Bitcoin CNF/unknown SAT root changed".to_owned());
    }
    fs::create_dir_all(output).map_err(|e| format!("output folder: {e}"))?;
    let shadow_path = output.join("BITCOIN.three_shadows.bin");
    let index_path = output.join("BITCOIN.three_shadows.hbi");
    let began = Instant::now();
    let mut original_stream = BufReader::new(
        File::open(prior.join("BITCOIN.rime"))
            .map_err(|e| format!("prior original RimeSphere open: {e}"))?
    );
    let mut shadow_writer = BufWriter::new(
        File::create(&shadow_path).map_err(|e| format!("three-shadow stream open: {e}"))?
    );
    let mut index_writer = BufWriter::new(
        File::create(&index_path).map_err(|e| format!("three-shadow index open: {e}"))?
    );
    let frozen = freeze_stream(&mut original_stream, &mut shadow_writer, &mut index_writer)?;
    shadow_writer.flush().map_err(|e| format!("shadow stream flush: {e}"))?;
    index_writer.flush().map_err(|e| format!("shadow index flush: {e}"))?;
    drop(shadow_writer);
    drop(index_writer);
    let freeze_ns = began.elapsed().as_nanos();

    let began = Instant::now();
    let mut encoded = BufReader::new(
        File::open(&shadow_path).map_err(|e| format!("shadow replay open: {e}"))?
    );
    let mut index = BufReader::new(
        File::open(&index_path).map_err(|e| format!("shadow HBI replay open: {e}"))?
    );
    let recovered_path = output.join("BITCOIN.three_shadows.replayed.cnf");
    let mut recovered_writer = BufWriter::new(
        File::create(&recovered_path).map_err(|e| format!("three-shadow replay open: {e}"))?
    );
    let played = play_stream(&mut encoded, &mut index, &mut recovered_writer)?;
    recovered_writer.flush().map_err(|e| format!("three-shadow replay flush: {e}"))?;
    drop(recovered_writer);
    if played != frozen {
        return Err("three-shadow projection and playback transcript differed".to_owned());
    }
    let recovered = fs::read(&recovered_path)
        .map_err(|e| format!("three-shadow original source reread: {e}"))?;
    if recovered != raw {
        return Err("three-shadow 2D reconstruction differs from every original Bitcoin source byte".to_owned());
    }
    let play_ns = began.elapsed().as_nanos();

    let began = Instant::now();
    let gguf = freeze(&recovered, &vec![Trit::Zero; cnf.variables])?;
    let gguf_path = output.join("BITCOIN.three_shadows.gguf");
    fs::write(&gguf_path, &gguf).map_err(|e| format!("reconstructed GGUF save: {e}"))?;
    let checked_gguf = fs::read(&gguf_path)
        .map_err(|e| format!("reconstructed GGUF reopen: {e}"))?;
    let gguf_state = thaw(&checked_gguf)?;
    if gguf_state.source != raw || gguf_state.states != prior_state.states
        || gguf != prior_gguf
    {
        return Err("three-shadow-replayed Bitcoin GGUF differs byte-for-byte from prior accepted checkpoint".to_owned());
    }
    let gguf_ns = began.elapsed().as_nanos();
    if frozen.shadow_bytes != frozen.rooms * SHADOW_BYTES as u64
        || frozen.source_bytes != raw.len() as u64
    {
        return Err("three-shadow exact source/room data accounting differs".to_owned());
    }

    let total_ns = started.elapsed().as_nanos();
    let report = format!(
        "BITCOIN_THREE_SHADOW_FINAL|schema=1|case={case}|source_sha256={}|\
         variables={}|clauses={}|source_bytes={}|\
         slices={}|rooms={}|three_lights=3|shadows_per_room=3|\
         pixels_per_shadow=9|projector_rank={EXPECTED_RANK}|\
         independent_residual_trits={RESIDUALS}|shadow_record_bytes={SHADOW_BYTES}|\
         shadow_data_bytes={}|shadow_hbi_sha256={}|shadow_root={}|\
         original_rime_root={}|gguf_sha256={}|gguf_bytes={}|\
         parent_gguf_sha256={}|shadow_stream_wall_ns={freeze_ns}|\
         shadow_play_wall_ns={play_ns}|gguf_wall_ns={gguf_ns}|\
         total_wall_ns={total_ns}|original_source=BYTE_EXACT|\
         3d_reconstruction=EXACT|gguf_identity=BYTE_EXACT|\
         SAT_search=NOT_RUN|status=PASS|json=0\n",
        hash_bytes(&raw), cnf.variables, cnf.clauses.len(), raw.len(),
        frozen.slices, frozen.rooms, frozen.shadow_bytes,
        hash_bytes(&fs::read(&index_path)
            .map_err(|e| format!("HBI digest read: {e}"))?),
        frozen.shadow_root, frozen.original_rime_root,
        hash_bytes(&checked_gguf), gguf.len(), hash_bytes(&prior_gguf)
    );
    fs::write(output.join("BITCOIN_THREE_SHADOW.hbp"), &report)
        .map_err(|e| format!("three-shadow receipt write: {e}"))?;
    print!("{report}");
    Ok(())
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 || args[1] != "project" {
        eprintln!("BITCOIN_THREE_SHADOW_FAIL|usage=bitcoin_three_shadows project <nonce32|merkle_nonce288> <original.cnf> <prior-stream-folder> <output-folder>|json=0");
        std::process::exit(1);
    }
    if let Err(error) = run(
        &args[2], Path::new(&args[3]),
        Path::new(&args[4]), Path::new(&args[5])
    ) {
        eprintln!("BITCOIN_THREE_SHADOW_FAIL|reason={error}|status=FAIL|json=0");
        std::process::exit(1);
    }
}
