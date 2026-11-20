//! Original-CNF-derived live SAT branch guidance from one versus three
//! independently projected 3x3 signed shadows of the *same* 27-cell room.
//! Original Boolean constraints and both branch polarities remain exact.
//! A signed source-constraint majority is the frozen source signal; actual
//! live assignments override that signal at assigned sites. This is a
//! heuristic, not a claimed SHA256d inversion algorithm.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::time::Instant;

use crate::matrix::CELLS;
use crate::sha256::hash_bytes;
use crate::ternary::{Cnf, Trit};
use crate::three_shadow::{ThreeShadows, SHADOW_BYTES};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadowMode {
    One,
    Three,
}
impl ShadowMode {
    pub fn label(self) -> &'static str {
        match self { Self::One => "one", Self::Three => "three" }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShadowMetrics {
    pub mode: ShadowMode,
    pub source_sha256: String,
    pub choices: u64,
    pub non_first_choices: u64,
    pub first_order_candidate: u64,
    pub first_shadow_candidate: u64,
    pub first_shadow_score: u64,
    pub room_projections: u64,
    pub pixel_reads: u64,
    pub source_trit_reads: u64,
    pub stored_projection_bytes: u64,
    pub reconstructed_rooms: u64,
    pub source_index_bytes: u64,
    pub index_wall_ns: u128,
    pub choice_wall_ns: u128,
    pub root: String,
}

#[derive(Clone)]
struct ShadowRoom {
    shadows: [[i8; 9]; 3],
}

pub struct ShadowSelector {
    source: String,
    mode: ShadowMode,
    signs: Vec<i8>,
    metrics: ShadowMetrics,
}
fn increment(counter: &mut u64, number: u64) {
    *counter = counter.checked_add(number)
        .expect("shadow selector count overflow");
}

impl ShadowSelector {
    pub fn new(cnf: &Cnf, source: &str, mode: ShadowMode) -> Result<Self, String> {
        if cnf.variables == 0 || source.len() != 64
            || !source.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("invalid original CNF source-bound shadow selector".to_owned());
        }
        let start = Instant::now();
        let mut signed = vec![0_i64; cnf.variables];
        for clause in &cnf.clauses {
            for &literal in clause {
                let pos = literal.unsigned_abs() as usize;
                if pos == 0 || pos > cnf.variables {
                    return Err("shadow source literal outside original CNF".to_owned());
                }
                signed[pos - 1] = signed[pos - 1]
                    .checked_add(i64::from(literal.signum()))
                    .ok_or("signed original CNF occurrence overflow")?;
            }
        }
        let signs = signed.into_iter().map(|n| n.signum() as i8).collect::<Vec<_>>();
        let index_wall_ns = start.elapsed().as_nanos();
        let root = hash_bytes(format!(
            "PNP_SHADOW_GUIDE_V1|{source}|{}|{}|{}",
            mode.label(), cnf.variables, cnf.clauses.len()
        ).as_bytes());
        Ok(Self {
            source: source.to_owned(), mode, signs,
            metrics: ShadowMetrics {
                mode, source_sha256: source.to_owned(), choices: 0,
                non_first_choices: 0, first_order_candidate: 0,
                first_shadow_candidate: 0, first_shadow_score: 0,
                room_projections: 0,
                pixel_reads: 0, source_trit_reads: 0,
                stored_projection_bytes: 0, reconstructed_rooms: 0,
                source_index_bytes: u64::try_from(cnf.variables)
                    .map_err(|_| "source trit-index bytes overflow")?,
                index_wall_ns, choice_wall_ns: 0, root,
            }
        })
    }

    /// Two policies see identical candidate variables and *identical* live
    /// 27-cell matrices. One inspects only X-line projections, while three
    /// inspects the three orthogonal line sums and verifies the exact
    /// 8-trit reconstruction witness with ThreeShadows::play().
    pub fn choose(
        &mut self, cnf: &Cnf, order: &[usize], assignment: &[Trit],
    ) -> Result<usize, String> {
        if assignment.len() != cnf.variables || assignment.len() != self.signs.len() {
            return Err("source-bound shadow live assignment width differs".to_owned());
        }
        let candidates = order.iter().copied()
            .filter(|&v| v >= 1 && v <= assignment.len()
                && assignment[v - 1] == Trit::Zero)
            .take(CELLS)
            .collect::<Vec<_>>();
        let first = *candidates.first()
            .ok_or("no open SAT decision for shadow selector")?;
        let start = Instant::now();
        let mut rooms: BTreeMap<usize, ShadowRoom> = BTreeMap::new();
        let mut receipt = Vec::new();
        for &candidate in &candidates {
            let beginning = ((candidate - 1) / CELLS) * CELLS;
            if rooms.contains_key(&beginning) { continue; }
            let mut live = [0_i8; CELLS];
            for (i, spot) in live.iter_mut().enumerate() {
                if beginning + i >= assignment.len() { break; }
                let assigned = match assignment[beginning + i] {
                    Trit::Plus => Some(1),
                    Trit::Minus => Some(-1),
                    Trit::Zero => None,
                };
                *spot = assigned.unwrap_or(self.signs[beginning + i]);
                increment(&mut self.metrics.source_trit_reads, 1);
            }
            let mut shadows = [[0_i8; 9]; 3];
            if self.mode == ShadowMode::One {
                for (index, &trit) in live.iter().enumerate() {
                    let y = (index / 3) % 3;
                    let z = index / 9;
                    shadows[0][3 * y + z] += trit;
                }
                increment(&mut self.metrics.stored_projection_bytes, 9);
                receipt.extend_from_slice(&shadows[0].map(|n| n as u8));
            } else {
                let triple = ThreeShadows::freeze(&live)?;
                if triple.play()? != live {
                    return Err("three projected source shadows failed inverse".to_owned());
                }
                shadows = triple.shadows;
                increment(&mut self.metrics.stored_projection_bytes, SHADOW_BYTES as u64);
                increment(&mut self.metrics.reconstructed_rooms, 1);
                receipt.extend_from_slice(&triple.bytes());
            }
            increment(&mut self.metrics.room_projections, 1);
            rooms.insert(beginning, ShadowRoom { shadows });
        }
        let mut best = first;
        let mut max_score = 0_u64;
        for &candidate in &candidates {
            let room = ((candidate - 1) / CELLS) * CELLS;
            let shadow = rooms.get(&room).ok_or("candidate shadow room missing")?;
            let pos = candidate - 1 - room;
            let x = pos % 3;
            let y = (pos / 3) % 3;
            let z = pos / 9;
            let mut score = u64::from(shadow.shadows[0][3 * y + z].unsigned_abs());
            increment(&mut self.metrics.pixel_reads, 1);
            if self.mode == ShadowMode::Three {
                score += u64::from(shadow.shadows[1][3 * x + z].unsigned_abs());
                score += u64::from(shadow.shadows[2][3 * x + y].unsigned_abs());
                increment(&mut self.metrics.pixel_reads, 2);
            }
            if score > max_score {
                best = candidate;
                max_score = score;
            }
        }
        increment(&mut self.metrics.choices, 1);
        if self.metrics.choices == 1 {
            self.metrics.first_order_candidate = first as u64;
            self.metrics.first_shadow_candidate = best as u64;
            self.metrics.first_shadow_score = max_score;
        }
        if best != first {
            increment(&mut self.metrics.non_first_choices, 1);
        }
        let next = hash_bytes(format!(
            "PNP_SHADOW_CHOICE_V1|{}|{}|{}|{}|{}|{}",
            self.metrics.root, self.source, self.mode.label(),
            self.metrics.choices, best, hash_bytes(&receipt),
        ).as_bytes());
        self.metrics.root = next;
        self.metrics.choice_wall_ns = self.metrics.choice_wall_ns
            .checked_add(start.elapsed().as_nanos())
            .ok_or("shadow guide wall timer overflow")?;
        Ok(best)
    }

    pub fn into_metrics(self) -> ShadowMetrics { self.metrics }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shadow_and_three_shadows_can_pick_distinct_open_original_variables() {
        let source = "p cnf 27 7\n1 0\n2 0\n3 0\n4 0\n5 0\n13 0\n22 0\n";
        let cnf = Cnf::parse(source).expect("parse");
        let sha = hash_bytes(source.as_bytes());
        let assignment = vec![Trit::Zero; 27];
        let order = (1..=9).collect::<Vec<_>>();
        let mut one = ShadowSelector::new(&cnf, &sha, ShadowMode::One).expect("one");
        let mut three = ShadowSelector::new(&cnf, &sha, ShadowMode::Three).expect("three");
        assert_eq!(one.choose(&cnf, &order, &assignment).expect("choose one"), 1);
        assert_eq!(three.choose(&cnf, &order, &assignment).expect("choose three"), 4);
        let one = one.into_metrics();
        let three = three.into_metrics();
        assert_eq!(one.pixel_reads, 9);
        assert_eq!(three.pixel_reads, 27);
        assert_eq!(one.stored_projection_bytes, 9);
        assert_eq!(three.stored_projection_bytes, 35);
        assert_eq!(three.reconstructed_rooms, 1);
    }

    #[test]
    fn assignment_changes_projection_and_incorrect_source_width_fails() {
        let cnf = Cnf::parse("p cnf 3 2\n1 2 0\n-2 3 0\n").expect("cnf");
        let sha = hash_bytes(b"p cnf 3 2\n1 2 0\n-2 3 0\n");
        let mut one = ShadowSelector::new(&cnf, &sha, ShadowMode::One).expect("one");
        assert!(one.choose(&cnf, &[1,2,3], &[Trit::Zero]).is_err());
        assert_eq!(one.choose(&cnf, &[1,2,3],
            &[Trit::Minus,Trit::Zero,Trit::Zero]).expect("live"),2);
    }
}
