//! One-versus-three orthogonal 2D shadow selection on the genuine original
//! Bitcoin SHA256d CNF. Same frozen GGUF and Fischer order in all modes.
//! Pilot is stopped by exact visited-node work, without an algorithmic timer.
#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

use pnp_native::bitcoin_gguf::thaw;
use pnp_native::guidance::{Guidance, Strategy};
use pnp_native::sha256::hash_bytes;
use pnp_native::shadow_selector::{ShadowMetrics, ShadowMode};
use pnp_native::solver::{solve_from_assignment, solve_shadow_from_assignment, Status};
use pnp_native::ternary::{Cnf, Trit};

const NONCE32: &str =
    "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";
const MERKLE288: &str =
    "601412f3d918c401b00df22a1b2799e50811d7fb1fa6bb4b";
const NODE_BUDGET: u64 = 2;
type Result<T> = std::result::Result<T, String>;

fn source_dimensions(case: &str) -> Result<(&'static str, usize, usize)> {
    match case {
        "nonce32" => Ok((NONCE32, 239_112, 877_879)),
        "merkle_nonce288" => Ok((MERKLE288, 361_188, 1_325_357)),
        _ => Err("unknown authentic Bitcoin full-SHA256d CNF case".to_owned()),
    }
}

fn run(mode: &str, case: &str, cnf_path: &Path, archive: &Path, out: &Path) -> Result<()> {
    if !matches!(mode, "direct" | "one" | "three") {
        return Err("mode must be direct, one or three".to_owned());
    }
    let start = Instant::now();
    let raw = fs::read(cnf_path).map_err(|e| format!("original source read: {e}"))?;
    let (expected, variables, clauses) = source_dimensions(case)?;
    if hash_bytes(&raw) != expected {
        return Err("original Bitcoin CNF source fingerprint changed".to_owned());
    }
    let source = std::str::from_utf8(&raw).map_err(|_| "original DIMACS not UTF8")?;
    let cnf = Cnf::parse(source)?;
    if cnf.variables != variables || cnf.clauses.len() != clauses {
        return Err("original Bitcoin clause/variable dimensions differ".to_owned());
    }
    let parse_ns = start.elapsed().as_nanos();
    let before_gguf = Instant::now();
    let on_disk = fs::read(archive).map_err(|e| format!("accepted 3-shadow GGUF read: {e}"))?;
    let gguf_sha = hash_bytes(&on_disk);
    let frozen = thaw(&on_disk)?;
    if frozen.source != raw || frozen.source_sha256 != expected
        || frozen.states.len() != variables
        || frozen.states.iter().any(|state| *state != Trit::Zero)
    {
        return Err("accepted original Bitcoin 3-shadow GGUF altered source or SAT seed".to_owned());
    }
    let gguf_ns = before_gguf.elapsed().as_nanos();
    let before_route = Instant::now();
    let guidance = Guidance::for_source(&cnf, Strategy::Fischer)?;
    let route_ns = before_route.elapsed().as_nanos();
    let before_search = Instant::now();
    let (solution, observer, shadows) = match mode {
        "direct" => (
            solve_from_assignment(&cnf, &guidance, Some(NODE_BUDGET), &frozen.states)?,
            None,
            None
        ),
        "one" | "three" => {
            let selector = if mode == "one" { ShadowMode::One } else { ShadowMode::Three };
            let (solution, live, guide) = solve_shadow_from_assignment(
                &cnf, &guidance, Some(NODE_BUDGET),
                &frozen.states, expected, selector,
            )?;
            (solution, Some(live), Some(guide))
        }
        _ => return Err("unsupported source-verified shadow mode".to_owned())
    };
    let solve_ns = before_search.elapsed().as_nanos();
    if solution.nodes > NODE_BUDGET || solution.time_exhausted
        || solution.stack_exhausted
    {
        return Err("timer-free shadow search exceeded exact pilot node work".to_owned());
    }
    fs::create_dir_all(out).map_err(|e| format!("result output directory: {e}"))?;
    let status = match solution.status {
        Status::Sat => "SAT",
        Status::Unsat => return Err(
            "a genuine original Bitcoin source must not be certified UNSAT".to_owned()
        ),
        Status::Hold => "HOLD_NODE",
    };
    let model_sha = if let Some(model) = &solution.model {
        if !cnf.valid_complete_model(model) {
            return Err("complete Bitcoin model failed original full CNF".to_owned());
        }
        let model_text = model.iter().map(|v| v.symbol()).collect::<String>() + "\n";
        fs::write(out.join("model.trits"), &model_text)
            .map_err(|e| format!("SAT model save: {e}"))?;
        hash_bytes(model_text.as_bytes())
    } else {
        "NONE".to_owned()
    };
    let (live_frames, live_frozen_bytes, live_wall_ns, live_root) = match observer {
        Some(meter) => (
            meter.freeze_play_pairs, meter.frozen_bytes,
            meter.matrix_wall_ns, meter.event_root,
        ),
        None => (0,0,0,"NONE".to_owned()),
    };
    let default = ShadowMetrics {
        mode: ShadowMode::One, source_sha256: expected.to_owned(), choices: 0,
        non_first_choices: 0, first_order_candidate: 0,
        first_shadow_candidate: 0, first_shadow_score: 0,
        room_projections: 0, pixel_reads: 0, source_trit_reads: 0,
        stored_projection_bytes: 0, reconstructed_rooms: 0,
        source_index_bytes: 0, index_wall_ns: 0, choice_wall_ns: 0,
        root: "NONE".to_owned(),
    };
    let guided = shadows.unwrap_or(default);
    let total_ns = start.elapsed().as_nanos();
    let receipt = format!(
        "BTC_THREE_SHADOW_SEARCH|schema=1|case={case}|mode={mode}|\
         status={status}|source_sha256={expected}|gguf_sha256={gguf_sha}|\
         variables={variables}|clauses={clauses}|source_bytes={}|\
         gguf_bytes={}|node_budget={NODE_BUDGET}|nodes={}|\
         propagations={}|literal_checks={}|\
         first_order_candidate={}|first_shadow_candidate={}|\
         first_shadow_score={}|projection_choices={}|projection_changes={}|\
         projected_rooms={}|source_trit_reads={}|shadow_pixel_reads={}|\
         projected_bytes={}|reconstructed_rooms={}|\
         shadow_index_bytes={}|shadow_index_ns={}|shadow_choice_ns={}|\
         shadow_decision_root={}|live_frames={live_frames}|\
         live_frozen_bytes={live_frozen_bytes}|live_wall_ns={live_wall_ns}|\
         live_root={live_root}|cnf_parse_ns={parse_ns}|gguf_read_ns={gguf_ns}|\
         fischer_route_ns={route_ns}|solver_ns={solve_ns}|\
         total_ns={total_ns}|model_sha256={model_sha}|\
         shadow_policy=SOURCE_SIGNED_OCCURRENCE_PLUS_LIVE_ASSIGNMENT|\
         artificial_timer=NO|known_nonce_injected=NO|\
         SAT_proof=ONLY_IF_SOURCE_MODEL_VALIDATED|json=0\n",
        raw.len(), on_disk.len(),
        solution.nodes, solution.propagations, solution.literal_checks,
        guided.first_order_candidate, guided.first_shadow_candidate,
        guided.first_shadow_score, guided.choices, guided.non_first_choices,
        guided.room_projections, guided.source_trit_reads, guided.pixel_reads,
        guided.stored_projection_bytes, guided.reconstructed_rooms,
        guided.source_index_bytes, guided.index_wall_ns, guided.choice_wall_ns,
        guided.root,
    );
    fs::write(out.join("receipt.hbp"), &receipt)
        .map_err(|e| format!("source-bound shadow receipt save: {e}"))?;
    print!("{receipt}");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 7 || args[1] != "run" {
        eprintln!("BTC_SHADOW_FAIL|usage=bitcoin_shadow_compare run <direct|one|three> <nonce32|merkle_nonce288> <source.cnf> <accepted.gguf> <outdir>|json=0");
        std::process::exit(1);
    }
    if let Err(error) = run(
        &args[2], &args[3], Path::new(&args[4]),
        Path::new(&args[5]), Path::new(&args[6]),
    ) {
        eprintln!("BTC_SHADOW_FAIL|reason={error}|status=FAIL|json=0");
        std::process::exit(1);
    }
}
