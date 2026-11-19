//! Exact matrix-informed branch selector over genuine live SAT assignments.
//!
//! This selects among the first 27 currently open Fischer-order candidates
//! using the original, unmodified CNF's active constraint pressure.
//! The candidate-room state is read back from source-bound Brown Ledger
//! snapshots before scoring. No known Bitcoin witness is supplied.
//! Exact SAT correctness remains the responsibility of the original solver
//! and independent complete-model checker.
#![forbid(unsafe_code)]

use crate::live_sat_matrix::{freeze_live, play_live};
use crate::matrix::CELLS;
use crate::sha256::hash_bytes;
use crate::ternary::{Cnf, Trit};
use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatrixSelectionMetrics {
    pub source_sha256: String,
    pub choices: u64,
    pub non_first_choices: u64,
    pub candidate_evaluations: u64,
    pub clause_reads: u64,
    pub literal_reads: u64,
    pub room_replays: u64,
    pub frozen_bytes: u64,
    pub metadata_bytes: u64,
    pub index_bytes: u64,
    pub index_build_ns: u128,
    pub selection_wall_ns: u128,
    pub decision_root: String,
}

pub struct MatrixSelector {
    source: String,
    occurrences: Vec<Vec<usize>>,
    metrics: MatrixSelectionMetrics,
}

fn add(counter: &mut u64, count: u64) {
    *counter = counter.checked_add(count).expect("matrix selection counter overflow");
}

impl MatrixSelector {
    pub fn new(cnf: &Cnf, source_sha256: &str) -> Result<Self, String> {
        if source_sha256.len() != 64
            || !source_sha256.bytes().all(|c| c.is_ascii_hexdigit())
            || cnf.variables == 0
        {
            return Err("invalid original-CNF source for live matrix selection".to_owned());
        }
        let started = Instant::now();
        let mut occurrences = vec![Vec::new(); cnf.variables];
        let mut count = 0_usize;
        for (clause_index, clause) in cnf.clauses.iter().enumerate() {
            for &lit in clause {
                let var = lit.unsigned_abs() as usize;
                if var == 0 || var > cnf.variables {
                    return Err("original CNF contains invalid branch literal".to_owned());
                }
                occurrences[var - 1].push(clause_index);
                count = count.checked_add(1)
                    .ok_or("CNF incidence counter overflow")?;
            }
        }
        let index_bytes = count.checked_mul(std::mem::size_of::<usize>())
            .and_then(|n| n.checked_add(
                cnf.variables.checked_mul(std::mem::size_of::<Vec<usize>>())?
            )).ok_or("CNF index byte count overflow")?;
        let index_build_ns = started.elapsed().as_nanos();
        let root = hash_bytes(format!(
            "PNP_MATRIX_CHOICE_V1|{source_sha256}|{}|{}|{count}",
            cnf.variables, cnf.clauses.len()
        ).as_bytes());
        Ok(Self {
            source: source_sha256.to_owned(),
            occurrences,
            metrics: MatrixSelectionMetrics {
                source_sha256: source_sha256.to_owned(),
                choices: 0,
                non_first_choices: 0,
                candidate_evaluations: 0,
                clause_reads: 0,
                literal_reads: 0,
                room_replays: 0,
                frozen_bytes: 0,
                metadata_bytes: 0,
                index_bytes: u64::try_from(index_bytes)
                    .map_err(|_| "matrix index bytes outside receipt range")?,
                index_build_ns,
                selection_wall_ns: 0,
                decision_root: root,
            },
        })
    }

    /// Complete, source-aware branch choice. Only an unassigned original
    /// variable may be returned. The replayed matrix supplies the candidate
    /// variable states and any nearby states used in the clause inspection.
    pub fn choose(
        &mut self,
        cnf: &Cnf,
        order: &[usize],
        assignment: &[Trit],
    ) -> Result<usize, String> {
        if assignment.len() != cnf.variables || self.occurrences.len() != cnf.variables {
            return Err("live assignment does not match original SAT source".to_owned());
        }
        let candidates: Vec<usize> = order.iter().copied()
            .filter(|&v| v >= 1 && v <= assignment.len()
                && assignment[v - 1] == Trit::Zero)
            .take(CELLS)
            .collect();
        let first = *candidates.first().ok_or("matrix selector has no open variable")?;
        let started = Instant::now();
        let mut rooms: BTreeMap<usize, Vec<Trit>> = BTreeMap::new();
        let mut receipt = Vec::new();
        for &candidate in &candidates {
            let start = ((candidate - 1) / CELLS) * CELLS;
            if let std::collections::btree_map::Entry::Vacant(slot) = rooms.entry(start) {
                let end = (start + CELLS).min(assignment.len());
                let frame = freeze_live(&self.source, start + 1, &assignment[start..end])?;
                let replay = play_live(&frame, &self.source)?;
                if replay != assignment[start..end] {
                    return Err("matrix-replayed SAT room altered its live source".to_owned());
                }
                receipt.extend_from_slice(frame.state_sha256.as_bytes());
                receipt.extend_from_slice(hash_bytes(&frame.frozen).as_bytes());
                add(&mut self.metrics.room_replays, 1);
                add(&mut self.metrics.frozen_bytes,
                    u64::try_from(frame.frozen.len()).map_err(|_| "frozen frame too large")?);
                add(&mut self.metrics.metadata_bytes, 192);
                slot.insert(replay);
            }
        }
        let mut best = first;
        let mut best_score = 0_u64;
        for &candidate in &candidates {
            add(&mut self.metrics.candidate_evaluations, 1);
            let room = ((candidate - 1) / CELLS) * CELLS;
            let actual = rooms.get(&room).ok_or("candidate matrix replay missing")?
                [candidate - 1 - room];
            if actual != Trit::Zero {
                return Err("source-verified candidate is already assigned".to_owned());
            }
            let mut score = 0_u64;
            for &index in &self.occurrences[candidate - 1] {
                let clause = cnf.clauses.get(index)
                    .ok_or("CNF occurrence index refers outside original source")?;
                add(&mut self.metrics.clause_reads, 1);
                let mut open = 0_u32;
                let mut satisfied = false;
                for &literal in clause {
                    add(&mut self.metrics.literal_reads, 1);
                    let v = literal.unsigned_abs() as usize;
                    let start = ((v - 1) / CELLS) * CELLS;
                    let state = match rooms.get(&start) {
                        Some(states) => states[v - 1 - start],
                        None => assignment[v - 1],
                    };
                    if state == Trit::from_literal(literal) {
                        satisfied = true;
                        break;
                    }
                    if state == Trit::Zero {
                        open += 1;
                    }
                }
                if !satisfied {
                    // Higher weight for currently tighter unsatisfied clauses.
                    // Both branch polarities remain subject to exact DPLL.
                    let weight: u64 = match open {
                        0 => 0,
                        1 => 81,
                        2 => 27,
                        3 => 9,
                        4 => 3,
                        _ => 1,
                    };
                    score = score.checked_add(weight)
                        .ok_or("source constraint pressure overflow")?;
                }
            }
            if score > best_score {
                best_score = score;
                best = candidate;
            }
        }
        add(&mut self.metrics.choices, 1);
        if best != first {
            add(&mut self.metrics.non_first_choices, 1);
        }
        receipt.extend_from_slice(format!(
            "|first={first}|selected={best}|score={best_score}|choice={}",
            self.metrics.choices
        ).as_bytes());
        self.metrics.decision_root = hash_bytes(format!(
            "PNP_MATRIX_CHOICE_STEP_V1|{}|{}",
            self.metrics.decision_root, hash_bytes(&receipt)
        ).as_bytes());
        self.metrics.selection_wall_ns = self.metrics.selection_wall_ns
            .checked_add(started.elapsed().as_nanos())
            .ok_or("matrix decision time counter overflow")?;
        Ok(best)
    }

    pub fn into_metrics(self) -> MatrixSelectionMetrics {
        self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::{check_against_exhaustive, solve, solve_with_matrix_guidance, Status};
    use crate::guidance::{Guidance, Strategy};

    #[test]
    fn actual_matrix_changes_choice_toward_tight_source_constraints() {
        let source = "p cnf 4 6\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-3 4 0\n-3 -4 0\n";
        let cnf = Cnf::parse(source).expect("source");
        let sha = hash_bytes(source.as_bytes());
        let mut matrix = MatrixSelector::new(&cnf, &sha).expect("selector");
        let choice = matrix.choose(&cnf, &[1, 2, 3, 4],
            &vec![Trit::Zero; 4]).expect("live choice");
        assert_eq!(choice, 3);
        let report = matrix.into_metrics();
        assert_eq!(report.choices, 1);
        assert_eq!(report.non_first_choices, 1);
        assert_eq!(report.candidate_evaluations, 4);
        assert!(report.room_replays >= 1 && report.frozen_bytes > 0);
    }

    #[test]
    fn exact_decisions_on_sat_unsat_and_bounded_hold() {
        let sources = [
            "p cnf 3 2\n1 2 0\n-2 3 0\n",
            "p cnf 2 4\n1 2 0\n-1 2 0\n1 -2 0\n-1 -2 0\n",
            "p cnf 4 6\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-3 4 0\n-3 -4 0\n",
        ];
        for source in sources {
            let cnf = Cnf::parse(source).expect("parse");
            let guide = Guidance::for_source(&cnf, Strategy::Base).expect("guidance");
            let sha = hash_bytes(source.as_bytes());
            let plain = solve(&cnf, &guide, None);
            let (guided, live, selector) =
                solve_with_matrix_guidance(&cnf, &guide, None, &sha).expect("guided");
            assert_eq!(plain.status, guided.status);
            assert!(check_against_exhaustive(&cnf, &guided));
            assert_eq!(guided.status == Status::Sat, guided.model.is_some());
            assert_eq!(live.freeze_play_pairs, live.watcher_reads);
            assert!(selector.choices > 0 || guided.nodes == 0);
            let (limited, _, _) =
                solve_with_matrix_guidance(&cnf, &guide, Some(0), &sha).expect("limited");
            assert_eq!(limited.status, Status::Hold);
            assert_eq!(limited.nodes, 0);
        }
    }

    #[test]
    fn source_binding_and_assignment_mutation_change_receipts() {
        let source = "p cnf 3 2\n1 2 0\n-2 3 0\n";
        let cnf = Cnf::parse(source).expect("source");
        let sha = hash_bytes(source.as_bytes());
        let mut selector = MatrixSelector::new(&cnf, &sha).expect("selector");
        let zero = vec![Trit::Zero; 3];
        let first = selector.choose(&cnf, &[1, 2, 3], &zero).expect("first");
        assert!((1..=3).contains(&first));
        let root = selector.metrics.decision_root.clone();
        let mut changed = zero;
        changed[first - 1] = Trit::Plus;
        let second = selector.choose(&cnf, &[1, 2, 3], &changed).expect("second");
        assert_ne!(first, second);
        assert_ne!(selector.metrics.decision_root, root);
        assert!(MatrixSelector::new(&cnf, "invalid").is_err());
    }
}
