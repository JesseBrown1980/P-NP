//! Exact original Bitcoin SAT: direct DPLL vs actual GGUF freeze/play and
//! matrix-guided DPLL resumed from its source-bound GGUF ternary state.
//! All GGUF write/read/Fischer inversions are inside each 300s worker window.
#![forbid(unsafe_code)]

use pnp_native::bitcoin_gguf::{freeze, play_variable, thaw};
use pnp_native::guidance::{Guidance, Strategy};
use pnp_native::sha256::hash_bytes;
use pnp_native::solver::{
    solve_from_assignment, solve_guided_from_assignment, solve_live_from_assignment, Status,
};
use pnp_native::ternary::{Cnf, Trit};
use std::collections::BTreeSet;
use std::{env, fs, path::Path, time::Instant};

const NONCE32: &str =
    "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";
const MERKLE288: &str =
    "601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b";
const BUDGET_NODES: u64 = 128;
type Result<T> = std::result::Result<T, String>;

fn require(condition: bool, error: &str) -> Result<()> {
    if condition { Ok(()) } else { Err(error.to_owned()) }
}

fn authentic(case: &str, raw: &[u8]) -> Result<(Cnf, String)> {
    let (digest, variables, clauses) = match case {
        "nonce32" => (NONCE32, 239_112, 877_879),
        "merkle_nonce288" => (MERKLE288, 361_188, 1_325_357),
        _ => return Err("unknown original Bitcoin double-SHA256 case".to_owned()),
    };
    let sha = hash_bytes(raw);
    require(sha == digest, "original Bitcoin full-CNF SHA256 changed")?;
    let source = std::str::from_utf8(raw).map_err(|_| "original DIMACS UTF8 invalid")?;
    let cnf = Cnf::parse(source)?;
    require(cnf.variables == variables && cnf.clauses.len() == clauses,
            "original Bitcoin DIMACS variable and clause count changed")?;
    require(cnf.clauses.iter().all(|clause| !clause.is_empty()),
            "original known-SAT Bitcoin CNF has empty clause")?;
    Ok((cnf, sha))
}

fn positions(n: usize) -> Vec<usize> {
    let mut result = BTreeSet::new();
    for x in [1, 2, 27, 28, 81, 82, n / 2, n - 1, n] {
        if x > 0 && x <= n {
            result.insert(x);
        }
    }
    for x in 0..27 {
        result.insert(1 + ((x * 65_537) % n));
    }
    result.into_iter().collect()
}

fn run(mode: &str, case: &str, cnf_path: &Path, out: &Path) -> Result<()> {
    require(matches!(mode, "direct" | "gguf" | "guided"),
            "run mode must be direct, gguf or guided")?;
    let started = Instant::now();
    let raw = fs::read(cnf_path).map_err(|e| format!("original source read: {e}"))?;
    let (cnf, sha) = authentic(case, &raw)?;
    let source_parse_ns = started.elapsed().as_nanos();
    fs::create_dir_all(out).map_err(|e| format!("result directory: {e}"))?;

    let mut states = vec![Trit::Zero; cnf.variables];
    let mut gguf_bytes = 0_usize;
    let mut gguf_sha = "NONE".to_owned();
    let mut frozen_ns = 0_u128;
    let mut write_ns = 0_u128;
    let mut read_ns = 0_u128;
    let mut thaw_ns = 0_u128;
    let mut fischer_ns = 0_u128;
    let mut fischer_reads = 0_usize;
    let mut fischer_trace_sha = "NONE".to_owned();
    if mode != "direct" {
        let t = Instant::now();
        let archive = freeze(&raw, &states)?;
        frozen_ns = t.elapsed().as_nanos();
        gguf_bytes = archive.len();
        gguf_sha = hash_bytes(&archive);
        let path = out.join("source_state.gguf");
        let t = Instant::now();
        fs::write(&path, &archive).map_err(|e| format!("GGUF save: {e}"))?;
        write_ns = t.elapsed().as_nanos();
        let t = Instant::now();
        let published = fs::read(&path).map_err(|e| format!("GGUF reread: {e}"))?;
        read_ns = t.elapsed().as_nanos();
        require(published == archive, "GGUF disk image differs from frozen bytes")?;
        let t = Instant::now();
        let restored = thaw(&published)?;
        thaw_ns = t.elapsed().as_nanos();
        require(restored.source == raw && restored.source_sha256 == sha,
                "GGUF restored source differs from original complete CNF")?;
        require(restored.states == states, "GGUF source-state snapshot mutated")?;
        let t = Instant::now();
        let mut trace = Vec::new();
        for variable in positions(cnf.variables) {
            let (address, trit) = play_variable(&restored, variable)?;
            require(trit == Trit::Zero,
                    "GGUF Fischer source playback has unexpected known assignment")?;
            trace.extend_from_slice(format!(
                "FISCHER|variable={variable}|address={address}|trit={}\n",
                trit.symbol()
            ).as_bytes());
            fischer_reads += 1;
        }
        fischer_trace_sha = hash_bytes(&trace);
        fischer_ns = t.elapsed().as_nanos();
        states = restored.states;
    }
    let assignment_sha = hash_bytes(&vec![0_u8; cnf.variables]);

    let t = Instant::now();
    let guidance = Guidance::for_source(&cnf, Strategy::Fischer)?;
    let route_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    let (solution, live, selector) = match mode {
        "direct" => (
            solve_from_assignment(&cnf, &guidance, Some(BUDGET_NODES), &states)?,
            None, None
        ),
        "gguf" => {
            let (solution, live) = solve_live_from_assignment(
                &cnf, &guidance, Some(BUDGET_NODES), &states, &sha
            )?;
            (solution, Some(live), None)
        }
        "guided" => {
            let (solution, live, selector) = solve_guided_from_assignment(
                &cnf, &guidance, Some(BUDGET_NODES), &states, &sha
            )?;
            (solution, Some(live), Some(selector))
        }
        _ => return Err("unsupported mode".to_owned()),
    };
    let solver_ns = t.elapsed().as_nanos();
    require(solution.nodes <= BUDGET_NODES, "original solver violated node cap")?;
    let (status, model) = match (solution.status, solution.model) {
        (Status::Sat, Some(states)) => {
            require(cnf.valid_complete_model(&states),
                    "SAT model fails original complete Bitcoin CNF")?;
            let text = states.iter().map(|state| state.symbol()).collect::<String>() + "\n";
            fs::write(out.join("model.trits"), &text)
                .map_err(|e| format!("SAT model write: {e}"))?;
            ("SAT", hash_bytes(text.as_bytes()))
        }
        (Status::Hold, None) => ("HOLD_NODE", "NONE".to_owned()),
        (Status::Unsat, _) => return Err(
            "original full Bitcoin satisfiable genesis source cannot be UNSAT".to_owned()
        ),
        _ => return Err("solver returned invalid SAT/model pair".to_owned()),
    };
    let (frames, watchers, frame_bytes, metadata_bytes, matrix_ns, frame_root) =
        if let Some(live) = live {
            (live.freeze_play_pairs, live.watcher_reads, live.frozen_bytes,
             live.digest_metadata_bytes, live.matrix_wall_ns, live.event_root)
        } else {
            (0, 0, 0, 0, 0, "NONE".to_owned())
        };
    let (choices, changed_choices, candidates, incidence_bytes, selector_rooms,
         selector_bytes, selector_metadata, selector_index_ns, selector_wall_ns,
         selector_root) = if let Some(sel) = selector {
            (sel.choices, sel.non_first_choices, sel.candidate_evaluations,
             sel.index_bytes, sel.room_replays, sel.frozen_bytes,
             sel.metadata_bytes, sel.index_build_ns, sel.selection_wall_ns,
             sel.decision_root)
        } else {
            (0, 0, 0, 0, 0, 0, 0, 0, 0, "NONE".to_owned())
        };
    let total_ns = started.elapsed().as_nanos();
    let receipt = format!(
        "BTC_GGUF_SAT_RESUME|schema=1|case={case}|mode={mode}|status={status}|\
         original_cnf_sha256={sha}|variables={}|clauses={}|\
         seed_state_sha256={assignment_sha}|seed_known_assignments=0|\
         gguf_sha256={gguf_sha}|gguf_bytes={gguf_bytes}|\
         gguf_freeze_ns={frozen_ns}|gguf_write_ns={write_ns}|gguf_read_ns={read_ns}|\
         gguf_thaw_ns={thaw_ns}|fischer_play_ns={fischer_ns}|\
         fischer_reads={fischer_reads}|fischer_trace_sha256={fischer_trace_sha}|\
         source_parse_ns={source_parse_ns}|fischer_route_ns={route_ns}|\
         solver_ns={solver_ns}|total_ns={total_ns}|wall_budget_seconds=300|\
         node_cap={BUDGET_NODES}|nodes={}|propagations={}|literal_checks={}|\
         live_frames={frames}|live_watcher_reads={watchers}|\
         live_frozen_bytes={frame_bytes}|live_metadata_bytes={metadata_bytes}|\
         live_matrix_ns={matrix_ns}|live_frame_root={frame_root}|\
         selector_choices={choices}|selector_nonfirst={changed_choices}|\
         selector_candidates={candidates}|selector_index_bytes={incidence_bytes}|\
         selector_room_replays={selector_rooms}|selector_frozen_bytes={selector_bytes}|\
         selector_metadata_bytes={selector_metadata}|\
         selector_index_ns={selector_index_ns}|selector_ns={selector_wall_ns}|\
         selector_root={selector_root}|model_sha256={model}|\
         known_bitcoin_nonce_injected=NO|original_clauses=EXACT|json=0\n",
        cnf.variables, cnf.clauses.len(),
        solution.nodes, solution.propagations, solution.literal_checks
    );
    fs::write(out.join("receipt.hbp"), &receipt)
        .map_err(|e| format!("receipt write: {e}"))?;
    print!("{receipt}");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 || args[1] != "run" {
        eprintln!("BTC_GGUF_RESUME_FAIL|reason=usage: run <direct|gguf|guided> <nonce32|merkle_nonce288> <source.cnf> <outdir>|json=0");
        std::process::exit(1);
    }
    if let Err(e) = run(&args[2], &args[3], Path::new(&args[4]), Path::new(&args[5])) {
        eprintln!("BTC_GGUF_RESUME_FAIL|reason={e}|status=FAIL|json=0");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bobby_fischer_addresses_cross_real_27_cell_rooms() {
        let selected = positions(239_112);
        assert!(selected.contains(&1) && selected.contains(&27)
            && selected.contains(&28) && selected.contains(&239_112));
        let source = b"p cnf 29 1\n28 29 0\n";
        let states = vec![Trit::Zero; 29];
        let bytes = freeze(source, &states).expect("freeze source");
        let read = thaw(&bytes).expect("read source");
        for i in [1, 27, 28, 29] {
            assert_eq!(play_variable(&read, i).expect("play").1, Trit::Zero);
        }
    }
}
