//! Direct versus live 27-cell-matrix SAT search on the immutable original
//! Bitcoin genesis CNF. Both modes use the same source-only Fischer order
//! and exact DPLL implementation with an identical search-node ceiling.
//! Only the live mode freezes and independently replays actual solver state.
#![forbid(unsafe_code)]

use pnp_native::guidance::{Guidance, Strategy};
use pnp_native::sha256::hash_bytes;
use pnp_native::solver::{solve, solve_with_live_matrix, Status};
use pnp_native::ternary::{Cnf, Trit};
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

const NONCE32: &str =
    "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";
const MERKLE288: &str =
    "601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b";
const MAX_NODES: u64 = 128;

fn ensure(condition: bool, why: &str) -> Result<(), String> {
    if condition { Ok(()) } else { Err(why.to_owned()) }
}

fn source(case: &str, bytes: &[u8]) -> Result<(Cnf, String), String> {
    let (expected, count, clauses) = match case {
        "nonce32" => (NONCE32, 239_112, 877_879),
        "merkle_nonce288" => (MERKLE288, 361_188, 1_325_357),
        _ => return Err("unknown original genesis SHA256d case".to_owned()),
    };
    let sha = hash_bytes(bytes);
    ensure(sha == expected, "original Bitcoin full-SHA256d CNF source digest mismatch")?;
    let text = std::str::from_utf8(bytes).map_err(|_| "source DIMACS is not UTF8")?;
    let cnf = Cnf::parse(text)?;
    ensure(cnf.variables == count && cnf.clauses.len() == clauses,
           "original Bitcoin DIMACS variable/constraint counts changed")?;
    Ok((cnf, sha))
}

fn run(mode: &str, case: &str, source_path: &Path, result_root: &Path) -> Result<(), String> {
    ensure(mode == "direct" || mode == "live", "mode must be direct or live")?;
    let all_started = Instant::now();
    let original = fs::read(source_path).map_err(|e| format!("read original CNF: {e}"))?;
    let (cnf, sha) = source(case, &original)?;
    let parsed_ns = all_started.elapsed().as_nanos();
    let started_route = Instant::now();
    let route = Guidance::for_source(&cnf, Strategy::Fischer)?;
    let route_ns = started_route.elapsed().as_nanos();
    let started_solve = Instant::now();
    let (solution, live) = if mode == "live" {
        let (solution, meters) =
            solve_with_live_matrix(&cnf, &route, Some(MAX_NODES), &sha)?;
        (solution, Some(meters))
    } else {
        (solve(&cnf, &route, Some(MAX_NODES)), None)
    };
    let solve_ns = started_solve.elapsed().as_nanos();
    ensure(
        solution.status != Status::Sat || solution.model.as_ref()
            .is_some_and(|model| cnf.valid_complete_model(model)),
        "SAT assignment fails exact original Bitcoin CNF"
    )?;
    fs::create_dir_all(result_root).map_err(|e| format!("mkdir: {e}"))?;
    let model_sha = if let Some(model) = solution.model {
        let states: String = model.iter().map(|state| state.symbol()).collect();
        let payload = format!("{states}\n");
        fs::write(result_root.join("model.trits"), &payload)
            .map_err(|e| format!("write solved model: {e}"))?;
        hash_bytes(payload.as_bytes())
    } else {
        "NONE".to_owned()
    };
    let (decision, unit, frames, watcher, frozen_bytes, metadata_bytes, matrix_ns, root) =
        if let Some(stats) = live {
            (stats.decision_frames, stats.propagation_frames,
             stats.freeze_play_pairs, stats.watcher_reads, stats.frozen_bytes,
             stats.digest_metadata_bytes, stats.matrix_wall_ns, stats.event_root)
        } else {
            (0, 0, 0, 0, 0, 0, 0, "NONE".to_owned())
        };
    let status = match solution.status {
        Status::Sat => "SAT",
        Status::Unsat => return Err("frozen known-SAT genesis source falsely deemed UNSAT".to_owned()),
        Status::Hold => "HOLD_NODE",
    };
    let total_ns = all_started.elapsed().as_nanos();
    let evidence = format!(
        "PNP_BTC_LIVE_SEARCH|schema=1|case={case}|mode={mode}|source_sha256={sha}|\
         variables={}|clauses={}|routing=FISCHER_SOURCE_ONLY|\
         budget_nodes={MAX_NODES}|budget_wall_s=300|status={status}|\
         nodes={}|propagations={}|literal_checks={}|\
         branch_frames={decision}|unit_frames={unit}|frames={frames}|watcher_reads={watcher}|\
         paid_frozen_bytes={frozen_bytes}|paid_metadata_bytes={metadata_bytes}|\
         matrix_wall_ns={matrix_ns}|source_parse_ns={parsed_ns}|\
         route_ns={route_ns}|search_ns={solve_ns}|total_ns={total_ns}|\
         frame_root={root}|model_sha256={model_sha}|known_nonce_injected=NO|\
         actual_solver_state=YES|json=0\n",
         cnf.variables, cnf.clauses.len(),
         solution.nodes, solution.propagations, solution.literal_checks
    );
    fs::write(result_root.join("receipt.hbp"), &evidence)
        .map_err(|e| format!("write receipt: {e}"))?;
    print!("{evidence}");
    Ok(())
}

fn verify(mode: &str, case: &str, source_path: &Path, result_root: &Path) -> Result<(), String> {
    let bytes = fs::read(source_path).map_err(|e| format!("read original source: {e}"))?;
    let (cnf, sha) = source(case, &bytes)?;
    let record = fs::read_to_string(result_root.join("receipt.hbp"))
        .map_err(|e| format!("read published search receipt: {e}"))?;
    ensure(record.lines().count() == 1, "receipt must be one complete record")?;
    let mut fields = std::collections::BTreeMap::new();
    for token in record.trim().split('|') {
        if let Some((key, value)) = token.split_once('=') {
            ensure(fields.insert(key, value).is_none(), "duplicate HBP field")?;
        }
    }
    for (key, expected) in [
        ("schema", "1"),
        ("mode", mode),
        ("case", case),
        ("source_sha256", sha.as_str()),
        ("budget_nodes", "128"),
        ("budget_wall_s", "300"),
        ("known_nonce_injected", "NO"),
        ("actual_solver_state", "YES"),
        ("json", "0"),
    ] {
        ensure(fields.get(key) == Some(&expected), "source/receipt field changed")?;
    }
    for (key, expected) in [
        ("variables", cnf.variables as u64),
        ("clauses", cnf.clauses.len() as u64),
    ] {
        ensure(fields.get(key).and_then(|v| v.parse::<u64>().ok()) == Some(expected),
               "original source size differs")?;
    }
    let parse = |key: &str| -> Result<u64, String> {
        fields.get(key).ok_or_else(|| format!("missing field {key}"))?
            .parse::<u64>().map_err(|_| format!("invalid counter {key}"))
    };
    let nodes = parse("nodes")?;
    ensure(nodes <= MAX_NODES, "solver ran outside declared search cap")?;
    let decision = parse("branch_frames")?;
    let unit = parse("unit_frames")?;
    let frames = parse("frames")?;
    let watchers = parse("watcher_reads")?;
    ensure(frames == decision + unit && watchers == frames,
           "matrix/branch/propagation frame accounting differs")?;
    if mode == "live" {
        ensure(fields.get("frame_root").is_some_and(|v| v.len() == 64),
               "source-bound matrix root missing")?;
        ensure(parse("paid_metadata_bytes")? == 192 * frames,
               "unpaid live digest metadata")?;
    } else {
        ensure(frames == 0 && fields.get("frame_root") == Some(&"NONE"),
               "direct baseline includes hidden matrix processing")?;
    }
    let status = fields.get("status").ok_or("missing search result")?;
    if *status == "SAT" {
        let model = fs::read_to_string(result_root.join("model.trits"))
            .map_err(|e| format!("read SAT model: {e}"))?;
        ensure(hash_bytes(model.as_bytes()) == *fields.get("model_sha256").unwrap_or(&""),
               "SAT model digest mismatch")?;
        let states = model.trim_end_matches('\n').chars()
            .map(|c| Trit::from_symbol(c).ok_or("invalid SAT trit".to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        ensure(cnf.valid_complete_model(&states), "model fails original Bitcoin CNF")?;
    } else {
        ensure(*status == "HOLD_NODE", "unapproved original Bitcoin solver outcome")?;
        ensure(!result_root.join("model.trits").exists(),
               "HOLD must not claim SAT witness")?;
    }
    println!(
        "PNP_BTC_LIVE_VERIFY|mode={mode}|case={case}|original_sha256={sha}|\
         state_frames={frames}|searched_nodes={nodes}|source_CNF=PASS|\
         receipt=PASS|status=PASS|json=0"
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let result = (|| -> Result<(), String> {
        if args.len() != 6 {
            return Err("usage: live_sat_benchmark <run|verify> <direct|live> <nonce32|merkle_nonce288> <source.cnf> <result_folder>".to_owned());
        }
        let mode = &args[2];
        let case = &args[3];
        let source = Path::new(&args[4]);
        let root = Path::new(&args[5]);
        match args[1].as_str() {
            "run" => run(mode, case, source, root),
            "verify" => verify(mode, case, source, root),
            _ => Err("expected run or verify".to_owned()),
        }
    })();
    if let Err(error) = result {
        eprintln!("PNP_BTC_LIVE_ERROR|reason={error}|json=0");
        std::process::exit(1);
    }
}
