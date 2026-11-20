//! Unbounded-lifetime Bitcoin SAT search in exact, restartable node quanta.
//! Each invocation loads the authentic full source + original accepted GGUF,
//! independently verifies the prior complete DPLL frontier (when supplied),
//! executes exactly up to the requested new node-work quota, and freezes
//! all pending branch decisions/assignments for the next compute session.
#![forbid(unsafe_code)]

use pnp_native::bitcoin_gguf::thaw;
use pnp_native::continuous_frontier::{Frontier, FrontierMode, FrontierState};
use pnp_native::guidance::{Guidance, Strategy};
use pnp_native::sha256::hash_bytes;
use pnp_native::ternary::{Cnf, Trit};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

const NONCE32: &str =
    "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";
const MERKLE288: &str =
    "601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b";
type Result<T> = std::result::Result<T, String>;

fn original(case: &str) -> Result<(&'static str, usize, usize)> {
    match case {
        "nonce32" => Ok((NONCE32, 239_112, 877_879)),
        "merkle_nonce288" => Ok((MERKLE288, 361_188, 1_325_357)),
        _ => Err("unknown full original Bitcoin SHA256d SAT source".to_owned()),
    }
}

fn step(args: &[String]) -> Result<()> {
    let mode = match args[2].as_str() {
        "direct" => FrontierMode::Direct,
        "one" => FrontierMode::One,
        "three" => FrontierMode::Three,
        _ => return Err("mode must be direct/one/three".to_owned()),
    };
    let (source_sha, variables, clauses) = original(&args[3])?;
    let quota = args[8].parse::<u64>()
        .map_err(|_| "invalid positive work quantum")?;
    if quota == 0 || quota > 1_000_000 {
        return Err("work quantum outside supported 1..1000000 range".to_owned());
    }
    let began = Instant::now();
    let raw = fs::read(&args[4]).map_err(|e| format!("source CNF read: {e}"))?;
    if hash_bytes(&raw) != source_sha {
        return Err("original Bitcoin full CNF fingerprint differs".to_owned());
    }
    let cnf = Cnf::parse(std::str::from_utf8(&raw)
        .map_err(|_| "Bitcoin original DIMACS invalid UTF8")?)?;
    if cnf.variables != variables || cnf.clauses.len() != clauses {
        return Err("original Bitcoin CNF variable or clause count changed".to_owned());
    }
    let original_read_ns = began.elapsed().as_nanos();

    let start_gguf = Instant::now();
    let gguf = fs::read(&args[5]).map_err(|e| format!("source-frozen GGUF read: {e}"))?;
    let gguf_sha = hash_bytes(&gguf);
    let frozen = thaw(&gguf)?;
    if frozen.source != raw || frozen.source_sha256 != source_sha
        || frozen.states.len() != variables
        || frozen.states.iter().any(|state| *state != Trit::Zero)
    {
        return Err("original three-shadow GGUF is not exact unmodified Bitcoin CNF".to_owned());
    }
    let gguf_read_ns = start_gguf.elapsed().as_nanos();
    let started_route = Instant::now();
    let guidance = Guidance::for_source(&cnf, Strategy::Fischer)?;
    let route_ns = started_route.elapsed().as_nanos();

    let started_replay = Instant::now();
    let predecessor = args[6].as_str();
    let mut previous_checkpoint_sha = "NEW".to_owned();
    let mut state = if predecessor == "NEW" {
        Frontier::new(&cnf, &guidance, &frozen.states, source_sha, mode)?
    } else {
        let bytes = fs::read(predecessor)
            .map_err(|e| format!("prior full-frontier checkpoint read: {e}"))?;
        previous_checkpoint_sha = hash_bytes(&bytes);
        Frontier::thaw(&bytes, &cnf, &guidance, &frozen.states, source_sha, mode)?
    };
    let replay_ns = started_replay.elapsed().as_nanos();
    let previous_nodes = state.nodes;
    let previous_trace = state.trace_sha256.clone();

    let work_began = Instant::now();
    let actual = state.advance(&cnf, &guidance, quota)?;
    let work_ns = work_began.elapsed().as_nanos();
    if state.nodes != previous_nodes + actual.visited {
        return Err("new-node accounting changed while resuming frontier".to_owned());
    }
    if state.state == FrontierState::Unsat {
        return Err("full original known-satisfiable Bitcoin CNF cannot end UNSAT".to_owned());
    }
    let checkpoint_began = Instant::now();
    let bytes = state.freeze()?;
    let verified = Frontier::thaw(
        &bytes, &cnf, &guidance, &frozen.states, source_sha, mode
    )?;
    if verified.freeze()? != bytes
        || verified.trace_sha256 != state.trace_sha256
        || verified.nodes != state.nodes
    {
        return Err("complete DPLL frontier cannot be restored byte-identically".to_owned());
    }
    let out = Path::new(&args[7]);
    fs::create_dir_all(out).map_err(|e| format!("checkpoint folder creation: {e}"))?;
    let checkpoint_path = out.join("FRONTIER.frt");
    let temp_path = out.join("FRONTIER.frt.tmp");
    {
        let mut file = File::create(&temp_path)
            .map_err(|e| format!("open atomic full frontier temp file: {e}"))?;
        file.write_all(&bytes)
            .map_err(|e| format!("write full frontier data: {e}"))?;
        file.sync_all().map_err(|e| format!("sync full frontier data: {e}"))?;
    }
    fs::rename(&temp_path, &checkpoint_path)
        .map_err(|e| format!("publish atomic full frontier checkpoint: {e}"))?;
    if fs::read(&checkpoint_path)
        .map_err(|e| format!("reread frozen frontier: {e}"))? != bytes
    {
        return Err("on-disk frontier snapshot differs from verified native bytes".to_owned());
    }
    let freeze_ns = checkpoint_began.elapsed().as_nanos();
    let checkpoint_sha = hash_bytes(&bytes);
    let model_sha = if let Some(model) = verified.model() {
        if !cnf.valid_complete_model(model) {
            return Err("completed SAT model fails original full Bitcoin CNF".to_owned());
        }
        let result = model.iter().map(|s| s.symbol()).collect::<String>() + "\n";
        fs::write(out.join("MODEL.trits"), &result)
            .map_err(|e| format!("write exact SAT model: {e}"))?;
        hash_bytes(result.as_bytes())
    } else {
        "NONE".to_owned()
    };
    let total_ns = began.elapsed().as_nanos();
    let receipt = format!(
        "BTC_FRONTIER_SESSION|schema=1|case={}|mode={}|state={}|\
         original_source_sha256={source_sha}|original_variables={variables}|\
         original_clauses={clauses}|original_gguf_sha256={gguf_sha}|\
         original_gguf_bytes={}|session_node_quota={quota}|\
         previous_checkpoint_sha256={previous_checkpoint_sha}|\
         previous_nodes={previous_nodes}|new_nodes={}|nodes={}|\
         branch_decisions={}|propagations={}|literal_checks={}|\
         pending_frames={}|active_trail={}|first_fischer={}|first_selected={}|\
         prev_trace_sha256={previous_trace}|trace_sha256={}|\
         projected_rooms={}|projected_bytes={}|shadow_pixel_reads={}|\
         reconstructed_rooms={}|shadow_source_reads={}|shadow_index_bytes={}|\
         session_projected_rooms={}|session_projected_bytes={}|\
         session_reconstructed_rooms={}|session_shadow_index_bytes={}|\
         cnf_read_ns={original_read_ns}|gguf_read_ns={gguf_read_ns}|\
         source_route_ns={route_ns}|frontier_replay_ns={replay_ns}|\
         solver_work_ns={work_ns}|frontier_freeze_ns={freeze_ns}|\
         session_total_ns={total_ns}|frontier_bytes={}|\
         frontier_sha256={checkpoint_sha}|model_sha256={model_sha}|\
         known_nonce_injected=NO|original_cnf=EXACT|\
         algorithmic_timer=NONE|work_quantum=CHECKPOINT_BOUNDARY|json=0\n",
        args[3], mode.label(), state.state.label(),
        gguf.len(), actual.visited, state.nodes, state.decisions,
        state.propagations, state.literal_checks,
        state.stack_len(), state.trail_len(), state.first_order, state.first_selected,
        state.trace_sha256, state.projected_rooms, state.projected_bytes,
        state.pixel_reads, state.reconstructed_rooms, state.source_trit_reads,
        state.source_index_bytes, actual.projected_rooms, actual.projected_bytes,
        actual.reconstructed_rooms, actual.source_index_bytes,
        bytes.len()
    );
    fs::write(out.join("SESSION.hbp"), &receipt)
        .map_err(|e| format!("write node-work quantum receipt: {e}"))?;
    print!("{receipt}");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 9 || args[1] != "step" {
        eprintln!("BTC_FRONTIER_FAIL|usage=bitcoin_frontier_quantum step <direct|one|three> <nonce32|merkle_nonce288> <original.cnf> <accepted.gguf> <NEW|previous.FRONTIER.frt> <output-folder> <new-node-quantum>|json=0");
        std::process::exit(1);
    }
    if let Err(error) = step(&args) {
        eprintln!("BTC_FRONTIER_FAIL|reason={error}|status=FAIL|json=0");
        std::process::exit(1);
    }
}
