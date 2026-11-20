//! Complete exact ternary-state DPLL for source-frozen Boolean CNF.
//! Unary trit updates: -1=false, 0=unassigned, +1=true.
//! Correct SAT search must explore BOTH nonzero values at each decision;
//! 3/27/81 are routing arities, not extra Boolean truth values.

use crate::guidance::Guidance;
use crate::live_sat_matrix::{freeze_live, play_live};
use crate::matrix::CELLS;
use crate::matrix_selector::{MatrixSelectionMetrics, MatrixSelector};
use crate::sha256::hash_bytes;
use crate::shadow_selector::{ShadowMetrics, ShadowMode, ShadowSelector};
use crate::ternary::{Cnf, Trit};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Sat,
    Unsat,
    Hold,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sat => "SAT",
            Self::Unsat => "UNSAT",
            Self::Hold => "HOLD_RESOURCE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Solution {
    pub status: Status,
    pub model: Option<Vec<Trit>>,
    pub nodes: u64,
    pub propagations: u64,
    pub literal_checks: u64,
    pub budget: Option<u64>,
    /// True only when the source-to-receipt experiment's internal deadline fired.
    pub time_exhausted: bool,
    /// True if the bounded recursive engine reached its explicit safe stack cap.
    pub stack_exhausted: bool,
}


/// Accounting for genuine solver search-state snapshots and decision frames.
/// A receipt root commits the ordered source-bound reversible frame events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveSearchMetrics {
    pub source_sha256: String,
    pub decision_frames: u64,
    pub propagation_frames: u64,
    pub freeze_play_pairs: u64,
    pub watcher_reads: u64,
    pub frozen_bytes: u64,
    pub digest_metadata_bytes: u64,
    pub matrix_wall_ns: u128,
    pub event_root: String,
}

struct LiveObserver<'a> {
    source_sha256: &'a str,
    stats: LiveSearchMetrics,
}

impl<'a> LiveObserver<'a> {
    fn new(source_sha256: &'a str) -> Result<Self, String> {
        if source_sha256.len() != 64
            || !source_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("live SAT search requires a full original source SHA256".to_owned());
        }
        let seed = hash_bytes(format!("PNP_SAT_LIVE_SEARCH_V1|{source_sha256}").as_bytes());
        Ok(Self {
            source_sha256,
            stats: LiveSearchMetrics {
                source_sha256: source_sha256.to_owned(),
                decision_frames: 0,
                propagation_frames: 0,
                freeze_play_pairs: 0,
                watcher_reads: 0,
                frozen_bytes: 0,
                digest_metadata_bytes: 0,
                matrix_wall_ns: 0,
                event_root: seed,
            },
        })
    }

    fn record(&mut self, assignment: &[Trit], variable: usize, phase: &str) {
        assert!((1..=assignment.len()).contains(&variable));
        assert!(matches!(phase, "BRANCH" | "UNIT"));
        let start = ((variable - 1) / CELLS) * CELLS;
        let end = (start + CELLS).min(assignment.len());
        let began = Instant::now();
        let frame = freeze_live(self.source_sha256, start + 1, &assignment[start..end])
            .expect("source-bound live SAT frame must freeze");
        // Explicit second replay is the watcher read. The freeze function
        // also checks its own first playback, and every byte is charged.
        let watched = play_live(&frame, self.source_sha256)
            .expect("live SAT watcher must reconstruct source-bound frame");
        assert_eq!(watched, assignment[start..end], "matrix watcher differs from live solver");
        let next_root = hash_bytes(format!(
            "PNP_SAT_EVENT_V1|{}|{}|{}|{}|{}|{}",
            self.stats.event_root, phase, start + 1, frame.used,
            frame.state_sha256, hash_bytes(&frame.frozen)
        ).as_bytes());
        self.stats.event_root = next_root;
        self.stats.frozen_bytes = self.stats.frozen_bytes
            .checked_add(frame.frozen.len() as u64).expect("frozen meter overflow");
        // A 64-digit frozen state digest, 64-digit source SHA, and the
        // rolling 64-digit event digest are explicit receipt metadata.
        self.stats.digest_metadata_bytes = self.stats.digest_metadata_bytes
            .checked_add(192).expect("metadata meter overflow");
        self.stats.freeze_play_pairs += 1;
        self.stats.watcher_reads += 1;
        if phase == "BRANCH" {
            self.stats.decision_frames += 1;
        } else {
            self.stats.propagation_frames += 1;
        }
        self.stats.matrix_wall_ns += began.elapsed().as_nanos();
    }

    fn propagate_changes(&mut self, before: &[Trit], after: &[Trit]) {
        assert_eq!(before.len(), after.len());
        for first in (0..before.len()).step_by(CELLS) {
            let end = (first + CELLS).min(before.len());
            if before[first..end] != after[first..end] {
                self.record(after, first + 1, "UNIT");
            }
        }
    }
}

enum Search {
    Sat(Vec<Trit>),
    Unsat,
    Hold,
}

struct Searcher<'a> {
    cnf: &'a Cnf,
    order: &'a [usize],
    nodes: u64,
    propagations: u64,
    literal_checks: u64,
    budget: Option<u64>,
    live: Option<LiveObserver<'a>>,
    selector: Option<MatrixSelector>,
    shadow: Option<ShadowSelector>,
    deadline: Option<Instant>,
    time_exhausted: bool,
    stack_exhausted: bool,
}

impl Searcher<'_> {
    fn increment_checks(&mut self) {
        self.literal_checks = self.literal_checks.checked_add(1)
            .expect("exact literal-check counter overflow");
    }

    fn assign(&mut self, values: &mut [Trit], variable: usize, target: Trit) -> bool {
        match values[variable - 1] {
            Trit::Zero => {
                values[variable - 1] = target;
                self.propagations = self.propagations.checked_add(1)
                    .expect("exact propagation counter overflow");
                true
            }
            other => other == target,
        }
    }

    fn propagate(&mut self, values: &mut [Trit]) -> bool {
        loop {
            let mut changed = false;
            for clause in &self.cnf.clauses {
                let mut satisfied = false;
                let mut remaining = 0_usize;
                let mut last = 0_i32;
                for &literal in clause {
                    self.increment_checks();
                    let assigned = values[literal.unsigned_abs() as usize - 1];
                    if assigned == Trit::from_literal(literal) {
                        satisfied = true;
                        break;
                    }
                    if assigned == Trit::Zero {
                        remaining += 1;
                        last = literal;
                    }
                }
                if satisfied {
                    continue;
                }
                if remaining == 0 {
                    return false;
                }
                if remaining == 1 {
                    let variable = last.unsigned_abs() as usize;
                    if !self.assign(values, variable, Trit::from_literal(last)) {
                        return false;
                    }
                    changed = true;
                }
            }
            if !changed {
                return true;
            }
        }
    }

    fn recurse(&mut self, original: &[Trit], depth: usize) -> Search {
        // An internal deadline lets the worker publish independently
        // verifiable HOLD_TIME meters before the external 300-second kill.
        if self.deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            self.time_exhausted = true;
            return Search::Hold;
        }
        // A high node ceiling must not turn a legitimate large-CNF hold
        // into a process-aborting recursion stack overflow.
        if self.deadline.is_some() && depth >= 1024 {
            self.stack_exhausted = true;
            return Search::Hold;
        }
        if self.budget.is_some_and(|budget| self.nodes >= budget) {
            return Search::Hold;
        }
        self.nodes = self.nodes.checked_add(1).expect("node counter overflow");
        let mut propagated = original.to_vec();
        let consistent = self.propagate(&mut propagated);
        if let Some(observer) = &mut self.live {
            observer.propagate_changes(original, &propagated);
        }
        if !consistent {
            return Search::Unsat;
        }
        let choice = if let Some(selector) = &mut self.selector {
            if self.order.iter().any(|&v| propagated[v - 1] == Trit::Zero) {
                Some(selector.choose(self.cnf, self.order, &propagated)
                    .expect("source-verified matrix selector must return an open variable"))
            } else {
                None
            }
        } else if let Some(shadow) = &mut self.shadow {
            if self.order.iter().any(|&v| propagated[v - 1] == Trit::Zero) {
                Some(shadow.choose(self.cnf, self.order, &propagated)
                    .expect("source-verified one/three shadow guide must choose an open variable"))
            } else {
                None
            }
        } else {
            self.order.iter()
                .copied()
                .find(|&variable| propagated[variable - 1] == Trit::Zero)
        };
        let Some(variable) = choice else {
            return if self.cnf.valid_complete_model(&propagated) {
                Search::Sat(propagated)
            } else {
                Search::Unsat
            };
        };
        // Unassigned variables not present in clauses may be set to minus.
        // For full correctness, a fully assigned valid model is independently
        // verified before and after search.
        let mut saw_hold = false;
        for branch in [Trit::Plus, Trit::Minus] {
            let mut next = propagated.clone();
            if !self.assign(&mut next, variable, branch) {
                continue;
            }
            if let Some(observer) = &mut self.live {
                observer.record(&next, variable, "BRANCH");
            }
            match self.recurse(&next, depth + 1) {
                Search::Sat(model) => return Search::Sat(model),
                Search::Hold => saw_hold = true,
                Search::Unsat => {}
            }
        }
        if saw_hold {
            Search::Hold
        } else {
            Search::Unsat
        }
    }
}

// Both original matrix and independent three-shadow routing are optional
// private modes on a shared exact DPLL core. Keep one narrowly scoped lint
// allowance rather than weakening the repository's strict Clippy gate.
#[allow(clippy::too_many_arguments)]
fn solve_internal(
    cnf: &Cnf,
    guidance: &Guidance,
    budget: Option<u64>,
    live: Option<LiveObserver<'_>>,
    selector: Option<MatrixSelector>,
    starting: Option<&[Trit]>,
    deadline: Option<Instant>,
    shadow: Option<ShadowSelector>,
) -> (Solution, Option<LiveSearchMetrics>, Option<MatrixSelectionMetrics>, Option<ShadowMetrics>) {
    let mut searcher = Searcher {
        cnf,
        order: &guidance.order,
        nodes: 0,
        propagations: 0,
        literal_checks: 0,
        budget,
        live,
        selector,
        shadow,
        deadline,
        time_exhausted: false,
        stack_exhausted: false,
    };
    let zero;
    let initial = if let Some(source) = starting {
        source
    } else {
        zero = vec![Trit::Zero; cnf.variables];
        &zero
    };
    let status = searcher.recurse(initial, 0);
    let (status, model) = match status {
        Search::Sat(values) => {
            assert!(cnf.valid_complete_model(&values),
                    "independent original-CNF model check failed");
            (Status::Sat, Some(values))
        }
        Search::Unsat => (Status::Unsat, None),
        Search::Hold => (Status::Hold, None),
    };
    let solution = Solution {
        status, model, nodes: searcher.nodes,
        propagations: searcher.propagations,
        literal_checks: searcher.literal_checks,
        budget,
        time_exhausted: searcher.time_exhausted,
        stack_exhausted: searcher.stack_exhausted,
    };
    (solution, searcher.live.map(|observer| observer.stats),
     searcher.selector.map(MatrixSelector::into_metrics),
     searcher.shadow.map(ShadowSelector::into_metrics))
}

/// Source-only exact baseline: no matrix work is performed in this path.
pub fn solve(cnf: &Cnf, guidance: &Guidance, budget: Option<u64>) -> Solution {
    solve_internal(cnf, guidance, budget, None, None, None, None, None).0
}

/// The identical exact DPLL propagation and branch logic with live 27-cell
/// freeze/play at actual decisions and every changed unit-propagation frame.
/// Known public Bitcoin witnesses are never injected into the search.
pub fn solve_with_live_matrix(
    cnf: &Cnf,
    guidance: &Guidance,
    budget: Option<u64>,
    source_sha256: &str,
) -> Result<(Solution, LiveSearchMetrics), String> {
    let observer = LiveObserver::new(source_sha256)?;
    let (solution, stats, _, _) = solve_internal(cnf, guidance, budget, Some(observer), None, None, None, None);
    Ok((solution, stats.expect("live observer must produce exact metrics")))
}

/// Genuine matrix-driven SAT branch selection: the current ternary solver
/// assignment is frozen into 27-cell source-bound rooms, replayed, and scored
/// against its still-active original CNF clauses. Every decision is metered.
/// This is an experimental exact selector; speedups require separate evidence.
pub fn solve_with_matrix_guidance(
    cnf: &Cnf,
    guidance: &Guidance,
    budget: Option<u64>,
    source_sha256: &str,
) -> Result<(Solution, LiveSearchMetrics, MatrixSelectionMetrics), String> {
    let observer = LiveObserver::new(source_sha256)?;
    let selector = MatrixSelector::new(cnf, source_sha256)?;
    let (solution, live, guided, _) =
        solve_internal(cnf, guidance, budget, Some(observer), Some(selector), None, None, None);
    Ok((
        solution,
        live.expect("matrix-guided solver must retain live frame receipts"),
        guided.expect("matrix-guided solver must retain selection receipts"),
    ))
}

/// Preserve an authenticated GGUF's genuine partial assignment as the exact
/// DPLL root, leaving the original clauses, propagation and branch semantics
/// unchanged. A constrained branch may legitimately be UNSAT even if the
/// unconditioned Bitcoin source is known satisfiable.
pub fn solve_from_assignment(
    cnf: &Cnf,
    guidance: &Guidance,
    budget: Option<u64>,
    starting: &[Trit],
) -> Result<Solution, String> {
    if starting.len() != cnf.variables {
        return Err("GGUF resumed assignment differs from original CNF width".to_owned());
    }
    Ok(solve_internal(cnf, guidance, budget, None, None, Some(starting), None, None).0)
}

/// Same root and exact search tree as solve_from_assignment, with reversible
/// 27-cell snapshots and separate live watcher metering.
pub fn solve_live_from_assignment(
    cnf: &Cnf,
    guidance: &Guidance,
    budget: Option<u64>,
    starting: &[Trit],
    source_sha256: &str,
) -> Result<(Solution, LiveSearchMetrics), String> {
    if starting.len() != cnf.variables {
        return Err("GGUF resumed live assignment differs from original CNF width".to_owned());
    }
    let observer = LiveObserver::new(source_sha256)?;
    let (solution, metrics, _, _) =
        solve_internal(cnf, guidance, budget, Some(observer), None, Some(starting), None, None);
    Ok((solution, metrics.expect("live GGUF root observer missing")))
}

/// Replayed GGUF assignment is the DPLL starting state; original source
/// constraints and real live ternary frames drive the 27-cell selector.
pub fn solve_guided_from_assignment(
    cnf: &Cnf,
    guidance: &Guidance,
    budget: Option<u64>,
    starting: &[Trit],
    source_sha256: &str,
) -> Result<(Solution, LiveSearchMetrics, MatrixSelectionMetrics), String> {
    if starting.len() != cnf.variables {
        return Err("GGUF resumed guided assignment differs from original CNF width".to_owned());
    }
    let observer = LiveObserver::new(source_sha256)?;
    let selector = MatrixSelector::new(cnf, source_sha256)?;
    let (solution, live, matrix, _) = solve_internal(
        cnf, guidance, budget, Some(observer), Some(selector), Some(starting), None, None
    );
    Ok((solution, live.expect("live witness missing"),
        matrix.expect("matrix-guided witness missing")))
}

/// The entire outer worker has its own 300-second limit. This exact
/// recursive SAT call also accepts a conservative internal absolute stop
/// time, so it can return node and replay receipts before host termination.
/// It supports direct/played/guided models from the same authenticated root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeMode {
    Direct,
    Played,
    Guided,
}

pub fn solve_resumed_until(
    cnf: &Cnf,
    guidance: &Guidance,
    budget: Option<u64>,
    starting: &[Trit],
    source_sha256: &str,
    mode: ResumeMode,
    deadline: Instant,
) -> Result<(Solution, Option<LiveSearchMetrics>, Option<MatrixSelectionMetrics>), String> {
    if starting.len() != cnf.variables {
        return Err("GGUF replay root width does not match full original CNF".to_owned());
    }
    let live = if mode != ResumeMode::Direct {
        Some(LiveObserver::new(source_sha256)?)
    } else {
        None
    };
    let selector = if mode == ResumeMode::Guided {
        Some(MatrixSelector::new(cnf, source_sha256)?)
    } else {
        None
    };
    let (solution, live, selector, _) = solve_internal(
        cnf, guidance, budget, live, selector, Some(starting), Some(deadline), None
    );
    Ok((solution, live, selector))
}

/// Same original CNF/DPLL propagation and Boolean branch semantics in all
/// modes. The source-frozen GGUF supplies the starting ternary assignment;
/// the selectable heuristic uses one X shadow or three independent X/Y/Z
/// projected 2-D signed planes. Only three mode reconstructs all 27 cells
/// with the exact eight-trit witness before any score is accepted.
pub fn solve_shadow_until(
    cnf: &Cnf,
    guidance: &Guidance,
    budget: Option<u64>,
    starting: &[Trit],
    source_sha256: &str,
    mode: ShadowMode,
    deadline: Instant,
) -> Result<(Solution, LiveSearchMetrics, ShadowMetrics), String> {
    if starting.len() != cnf.variables {
        return Err("GGUF initial trits differ from original shadow-CNF width".to_owned());
    }
    let observer = LiveObserver::new(source_sha256)?;
    let selector = ShadowSelector::new(cnf, source_sha256, mode)?;
    let (solution, live, _, shadow) = solve_internal(
        cnf, guidance, budget, Some(observer), None, Some(starting),
        Some(deadline), Some(selector),
    );
    Ok((solution, live.expect("shadow watcher disappeared"),
        shadow.expect("shadow guide disappeared")))
}

/// Timer-free finite-node research pilot. It evaluates the original source
/// CNF and true live 27-cell shadow guidance, with the node budget counted
/// as work rather than a wall-clock lifetime. Independent host runner limits
/// are separate from the source SAT engine.
pub fn solve_shadow_from_assignment(
    cnf: &Cnf,
    guidance: &Guidance,
    budget: Option<u64>,
    starting: &[Trit],
    source_sha256: &str,
    mode: ShadowMode,
) -> Result<(Solution, LiveSearchMetrics, ShadowMetrics), String> {
    if starting.len() != cnf.variables {
        return Err("GGUF shadow starting-state width differs from original CNF".to_owned());
    }
    let observer = LiveObserver::new(source_sha256)?;
    let selector = ShadowSelector::new(cnf, source_sha256, mode)?;
    let (solution, live, _, shadow) = solve_internal(
        cnf, guidance, budget, Some(observer), None, Some(starting), None,
        Some(selector)
    );
    Ok((solution, live.expect("exact live shadow frame observer missing"),
        shadow.expect("exact one/three shadow selector missing")))
}

pub fn check_against_exhaustive(cnf: &Cnf, solution: &Solution) -> bool {
    if cnf.variables > 16 || cnf.variables >= usize::BITS as usize {
        return false;
    }
    let mut has_model = false;
    for mask in 0..(1_usize << cnf.variables) {
        let states: Vec<Trit> = (0..cnf.variables)
            .map(|index| if (mask >> index) & 1 == 0 {
                Trit::Minus
            } else {
                Trit::Plus
            })
            .collect();
        if cnf.valid_complete_model(&states) {
            has_model = true;
            break;
        }
    }
    match solution.status {
        Status::Sat => has_model && solution.model.as_ref()
            .is_some_and(|values| cnf.valid_complete_model(values)),
        Status::Unsat => !has_model && solution.model.is_none(),
        Status::Hold => solution.model.is_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guidance::Strategy;

    #[test]
    fn nine_native_strategies_are_exact_on_five_independent_truth_classes() {
        let examples = [
            ("p cnf 3 2\n1 2 0\n-2 3 0\n", Status::Sat),
            ("p cnf 1 2\n1 0\n-1 0\n", Status::Unsat),
            ("p cnf 2 4\n1 2 0\n-1 2 0\n1 -2 0\n-1 -2 0\n",
                Status::Unsat),
            ("p cnf 3 1\n0\n", Status::Unsat),
            ("p cnf 4 2\n1 -2 0\n3 -4 0\n", Status::Sat),
        ];
        for (source, expected) in examples {
            let cnf = Cnf::parse(source).expect("valid CNF");
            for strategy in Strategy::all() {
                let guidance = Guidance::for_source(&cnf, strategy)
                    .expect("verified source-derived guidance");
                let result = solve(&cnf, &guidance, None);
                assert_eq!(result.status, expected, "{}", strategy.label());
                assert!(check_against_exhaustive(&cnf, &result),
                        "independent exhaustive SAT verification");
            }
        }
    }

    #[test]
    fn direct_and_live_solver_visit_identical_tree_with_exact_event_receipts() {
        use crate::sha256::hash_bytes;
        let examples = [
            "p cnf 3 2\n1 2 0\n-2 3 0\n",
            "p cnf 2 4\n1 2 0\n-1 2 0\n1 -2 0\n-1 -2 0\n",
            "p cnf 4 3\n1 0\n-1 2 0\n-2 3 0\n",
        ];
        for source in examples {
            let cnf = Cnf::parse(source).expect("source");
            let guide = Guidance::for_source(&cnf, Strategy::Base).expect("guidance");
            let sha = hash_bytes(source.as_bytes());
            for budget in [Some(0), Some(1), Some(3), Some(32), None] {
                let direct = solve(&cnf, &guide, budget);
                let (live, metrics) = solve_with_live_matrix(&cnf, &guide, budget, &sha)
                    .expect("live matrix");
                assert_eq!(direct, live, "matrix frame must preserve DPLL work and verdict");
                assert_eq!(metrics.source_sha256, sha);
                assert_eq!(metrics.freeze_play_pairs,
                    metrics.decision_frames + metrics.propagation_frames);
                assert_eq!(metrics.watcher_reads, metrics.freeze_play_pairs);
                assert_eq!(metrics.digest_metadata_bytes,
                    192 * metrics.freeze_play_pairs);
                assert!(check_against_exhaustive(&cnf, &live));
                if live.nodes > 0 && live.propagations > 0 {
                    assert!(metrics.freeze_play_pairs > 0);
                    assert!(metrics.frozen_bytes > 0);
                }
            }
        }
    }

    #[test]
    fn live_witness_receipt_changes_with_branch_and_source() {
        use crate::sha256::hash_bytes;
        let one = "p cnf 2 1\n1 2 0\n";
        let other = "p cnf 2 1\n-1 2 0\n";
        let a = Cnf::parse(one).expect("one");
        let b = Cnf::parse(other).expect("other");
        let ag = Guidance::for_source(&a, Strategy::Base).expect("order a");
        let bg = Guidance::for_source(&b, Strategy::Base).expect("order b");
        let sha_a = hash_bytes(one.as_bytes());
        let sha_b = hash_bytes(other.as_bytes());
        let (_, ma) = solve_with_live_matrix(&a, &ag, Some(8), &sha_a).expect("a");
        let (_, mb) = solve_with_live_matrix(&b, &bg, Some(8), &sha_b).expect("b");
        assert_ne!(ma.event_root, mb.event_root);
        assert!(solve_with_live_matrix(&a, &ag, Some(8), "bad").is_err());
    }

    #[test]
    fn gguf_starting_state_preserves_direct_passive_and_guided_sat_semantics() {
        use crate::bitcoin_gguf::{freeze, thaw};
        use crate::sha256::hash_bytes;
        let source = "p cnf 3 2\n1 2 0\n-2 3 0\n";
        let cnf = Cnf::parse(source).expect("source");
        let guidance = Guidance::for_source(&cnf, Strategy::Base).expect("guidance");
        let sha = hash_bytes(source.as_bytes());
        for initial in [
            [Trit::Zero, Trit::Zero, Trit::Zero],
            [Trit::Plus, Trit::Zero, Trit::Zero],
            [Trit::Minus, Trit::Zero, Trit::Zero],
        ] {
            let archive = freeze(source.as_bytes(), &initial).expect("frozen GGUF");
            let restored = thaw(&archive).expect("played GGUF");
            assert_eq!(restored.states, initial);
            let baseline = solve_from_assignment(&cnf, &guidance, Some(64), &initial)
                .expect("baseline seeded");
            let (passive, watch) = solve_live_from_assignment(
                &cnf, &guidance, Some(64), &restored.states, &sha
            ).expect("watched seeded");
            assert_eq!(baseline, passive, "GGUF passive replay changed DPLL tree");
            let (guided, live, selector) = solve_guided_from_assignment(
                &cnf, &guidance, Some(64), &restored.states, &sha
            ).expect("guided seeded");
            assert_eq!(guided.status, baseline.status);
            assert!(match &guided.model {
                Some(model) => cnf.valid_complete_model(model),
                None => true,
            });
            assert_eq!(watch.watcher_reads, watch.freeze_play_pairs);
            assert_eq!(live.watcher_reads, live.freeze_play_pairs);
            assert!(selector.choices <= guided.nodes);
        }
        assert!(solve_from_assignment(
            &cnf, &guidance, None, &[Trit::Zero]
        ).is_err());
        let impossible = Cnf::parse("p cnf 1 1\n1 0\n").expect("unit");
        let g = Guidance::for_source(&impossible, Strategy::Base).expect("guide");
        let contradictory = solve_from_assignment(
            &impossible, &g, Some(10), &[Trit::Minus]
        ).expect("contradictory seed");
        assert_eq!(contradictory.status, Status::Unsat);
    }

    #[test]
    fn resumed_source_time_and_stack_guards_are_honest() {
        use crate::sha256::hash_bytes;
        let source = "p cnf 3 2\n1 2 0\n-2 3 0\n";
        let cnf = Cnf::parse(source).expect("source");
        let order = Guidance::for_source(&cnf, Strategy::Base).expect("order");
        let fingerprint = hash_bytes(source.as_bytes());
        for mode in [ResumeMode::Direct, ResumeMode::Played, ResumeMode::Guided] {
            let (stopped, _, _) = solve_resumed_until(
                &cnf, &order, Some(1000), &[Trit::Zero; 3],
                &fingerprint, mode, Instant::now()
            ).expect("timeout");
            assert_eq!(stopped.status, Status::Hold);
            assert!(stopped.time_exhausted);
            assert!(!stopped.stack_exhausted);
            assert_eq!(stopped.nodes, 0);
            let (finished, _, _) = solve_resumed_until(
                &cnf, &order, Some(1000), &[Trit::Zero; 3],
                &fingerprint, mode, Instant::now() + std::time::Duration::from_secs(60)
            ).expect("complete");
            assert_eq!(finished.status, Status::Sat);
            assert!(!finished.time_exhausted && !finished.stack_exhausted);
            assert!(check_against_exhaustive(&cnf, &finished));
        }
    }

    #[test]
    fn one_and_three_shadow_guided_exact_sat_and_unsat_complete() {
        use crate::guidance::{Guidance, Strategy};
        let sources = [
            "p cnf 4 2\n1 2 0\n-2 3 0\n",
            "p cnf 3 8\n1 2 3 0\n1 2 -3 0\n1 -2 3 0\n1 -2 -3 0\n-1 2 3 0\n-1 2 -3 0\n-1 -2 3 0\n-1 -2 -3 0\n",
        ];
        for source in sources {
            let cnf = Cnf::parse(source).expect("parse source");
            let guidance = Guidance::for_source(&cnf, Strategy::Fischer).expect("fischer");
            let fingerprint = hash_bytes(source.as_bytes());
            let direct = solve(&cnf, &guidance, Some(256));
            for mode in [ShadowMode::One, ShadowMode::Three] {
                let (candidate, live, projection) = solve_shadow_until(
                    &cnf, &guidance, Some(256), &vec![Trit::Zero; cnf.variables],
                    &fingerprint, mode, Instant::now() + std::time::Duration::from_secs(30)
                ).expect("projected exact SAT");
                assert_eq!(candidate.status, direct.status);
                assert!(!candidate.time_exhausted && !candidate.stack_exhausted);
                assert!(match &candidate.model {
                    Some(m) => cnf.valid_complete_model(m),
                    None => true,
                });
                assert!(projection.room_projections > 0);
                assert_eq!(live.freeze_play_pairs, live.watcher_reads);
                if mode == ShadowMode::Three {
                    assert!(projection.reconstructed_rooms > 0);
                } else {
                    assert_eq!(projection.reconstructed_rooms, 0);
                }
            }
        }
    }

    #[test]
    fn bounded_search_returns_an_honest_hold() {
        let cnf = Cnf::parse("p cnf 2 1\n1 2 0\n").expect("source");
        let guided = Guidance::for_source(&cnf, Strategy::Base).expect("source");
        let stopped = solve(&cnf, &guided, Some(0));
        assert_eq!(stopped.status, Status::Hold);
        assert_eq!(stopped.nodes, 0);
        assert!(check_against_exhaustive(&cnf, &stopped));
        let complete = solve(&cnf, &guided, None);
        assert_eq!(complete.status, Status::Sat);
        assert!(check_against_exhaustive(&cnf, &complete));
    }

    #[test]
    fn exhaustive_crosscheck_all_256_two_clause_boolean_sources() {
        // Each 2-variable clause has four possible signed literals;
        // check all 16 pairs with repetition, and all nine strategies.
        let literals = [1, -1, 2, -2];
        let mut checked = 0;
        for &a in &literals {
            for &b in &literals {
                for &c in &literals {
                    for &d in &literals {
                        let cnf = Cnf {
                            variables: 2,
                            clauses: vec![vec![a, b], vec![c, d]],
                        };
                        for strategy in Strategy::all() {
                            let order = Guidance::for_source(&cnf, strategy)
                                .expect("source guidance");
                            let result = solve(&cnf, &order, None);
                            assert!(check_against_exhaustive(&cnf, &result));
                            checked += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(checked, 2_304);
    }
}
