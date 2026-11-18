//! Exact source-only branch guidance: ternary unary signals, 3/27/81
//! actual candidate-dispatch lanes, frozen forward/reverse order-two MTP,
//! Fischer modular-address priorities and independent three-seat matrix.
//!
//! These are CNF-native prototypes of the earlier training/routing ideas.
//! They are not the historical trained GNN/FNN weights or a live hotel.

use std::collections::BTreeMap;

use crate::freeze::{fischer_address, FrozenMatrix, RIME_PRIME};
use crate::matrix::{Matrix, CELLS};
use crate::ternary::{Cnf, Trit};
use crate::triad::wave_cascade;
use crate::wave::WaveRouter;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Strategy {
    Base,
    Mtp,
    Fischer,
    Nest3,
    Nest27,
    Nest81,
    Hybrid3,
    Hybrid27,
    Hybrid81,
}

impl Strategy {
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "base" => Some(Self::Base),
            "mtp" => Some(Self::Mtp),
            "fischer" => Some(Self::Fischer),
            "nest3" => Some(Self::Nest3),
            "nest27" => Some(Self::Nest27),
            "nest81" => Some(Self::Nest81),
            "hybrid3" => Some(Self::Hybrid3),
            "hybrid27" => Some(Self::Hybrid27),
            "hybrid81" => Some(Self::Hybrid81),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Mtp => "mtp",
            Self::Fischer => "fischer",
            Self::Nest3 => "nest3",
            Self::Nest27 => "nest27",
            Self::Nest81 => "nest81",
            Self::Hybrid3 => "hybrid3",
            Self::Hybrid27 => "hybrid27",
            Self::Hybrid81 => "hybrid81",
        }
    }

    pub fn width(self) -> usize {
        match self {
            Self::Nest27 | Self::Hybrid27 => 27,
            Self::Nest81 | Self::Hybrid81 => 81,
            _ => 3,
        }
    }

    pub fn uses_mtp(self) -> bool {
        matches!(self, Self::Mtp | Self::Hybrid3 | Self::Hybrid27 | Self::Hybrid81)
    }

    pub fn uses_fischer(self) -> bool {
        matches!(self, Self::Fischer | Self::Hybrid3 | Self::Hybrid27 | Self::Hybrid81)
    }

    pub fn uses_matrix(self) -> bool {
        matches!(self, Self::Nest3 | Self::Nest27 | Self::Nest81
                        | Self::Hybrid3 | Self::Hybrid27 | Self::Hybrid81)
    }

    pub fn all() -> [Self; 9] {
        [
            Self::Base, Self::Mtp, Self::Fischer,
            Self::Nest3, Self::Nest27, Self::Nest81,
            Self::Hybrid3, Self::Hybrid27, Self::Hybrid81,
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Guidance {
    pub strategy: Strategy,
    pub order: Vec<usize>,
    pub fanout: usize,
    pub lanes_used: usize,
    pub frozen_mtp_patterns: usize,
    pub fischer_addresses: usize,
    pub matrix_rooms: usize,
    pub fabric_witnesses: usize,
    pub root_accepted: bool,
    pub wave_reads: usize,
    pub wave_choices_checked: usize,
}

fn token(lit: i32) -> i8 {
    match Trit::from_literal(lit) {
        Trit::Minus => -1,
        Trit::Zero => 0,
        Trit::Plus => 1,
    }
}

fn frozen_trigrams<'a>(
    rows: impl Iterator<Item = &'a Vec<i32>>,
    backwards: bool,
) -> BTreeMap<(i8, i8, i8), u64> {
    let mut frequencies = BTreeMap::new();
    for row in rows {
        let symbols: Vec<i8> = if backwards {
            row.iter().rev().map(|&lit| token(lit)).collect()
        } else {
            row.iter().map(|&lit| token(lit)).collect()
        };
        for triple in symbols.windows(3) {
            *frequencies.entry((triple[0], triple[1], triple[2]))
                .or_insert(0) += 1;
        }
    }
    frequencies
}

fn source_mtp_scores(cnf: &Cnf, scores: &mut [i128]) -> usize {
    // Even input clauses are training data. Odd clauses only score;
    // this preserves the frozen forward/backward state between phases.
    let forward = frozen_trigrams(cnf.clauses.iter().step_by(2), false);
    let backward = frozen_trigrams(cnf.clauses.iter().step_by(2), true);
    for clause in cnf.clauses.iter().skip(1).step_by(2) {
        let tokens: Vec<i8> = clause.iter().map(|&lit| token(lit)).collect();
        for index in 2..tokens.len() {
            let forward_key = (tokens[index - 2], tokens[index - 1], tokens[index]);
            let reverse_key = (tokens[index], tokens[index - 1], tokens[index - 2]);
            let forward_count = forward.get(&forward_key).copied().unwrap_or(0);
            let back_count = backward.get(&reverse_key).copied().unwrap_or(0);
            let variable = clause[index].unsigned_abs() as usize;
            // Integer-only fixed-point counts. No trained output labels.
            scores[variable - 1] += i128::from(forward_count + back_count + 2) * 64;
        }
    }
    forward.len() + backward.len()
}

fn cnf_matrix(cnf: &Cnf, rooms: usize) -> Result<Matrix, String> {
    if !matches!(rooms, 3 | 27 | 81) {
        return Err("invalid matrix arity".to_owned());
    }
    let mut cells = vec![[0_i64; CELLS]; rooms];
    for (index, clause) in cnf.clauses.iter().enumerate() {
        for &literal in clause {
            let variable = literal.unsigned_abs() as usize - 1;
            let room = (index + variable / CELLS) % rooms;
            let cell = variable % CELLS;
            let sign = match Trit::from_literal(literal) {
                Trit::Plus => 1_i64,
                Trit::Minus => -1_i64,
                Trit::Zero => return Err("zero clause literal".to_owned()),
            };
            cells[room][cell] = cells[room][cell].checked_add(sign)
                .ok_or("integer matrix overflow")?;
        }
    }
    Matrix::from_rooms(cells).map_err(str::to_owned)
}

impl Guidance {
    pub fn for_source(cnf: &Cnf, strategy: Strategy) -> Result<Self, String> {
        let fanout = strategy.width();
        let mut score = vec![0_i128; cnf.variables];
        for clause in &cnf.clauses {
            for literal in clause {
                let variable = literal.unsigned_abs() as usize;
                score[variable - 1] += 1024;
            }
        }

        let mut frozen_mtp_patterns = 0;
        if strategy.uses_mtp() {
            frozen_mtp_patterns = source_mtp_scores(cnf, &mut score);
        }

        let mut fischer_addresses = 0;
        if strategy.uses_fischer() {
            for (index, position) in score.iter_mut().enumerate() {
                let glyph = u64::try_from(index % CELLS).map_err(|_| "glyph range")?;
                let room = u64::try_from(index / CELLS).map_err(|_| "room range")?;
                let point = fischer_address(glyph, room)
                    .ok_or("Fischer address overflow")?;
                *position += i128::from(RIME_PRIME - point);
                fischer_addresses += 1;
            }
        }

        let mut wave_reads = 0_usize;
        let mut wave_choices_checked = 0_usize;
        let (matrix_rooms, fabric_witnesses, root_accepted) =
            if strategy.uses_matrix() {
                let original = cnf_matrix(cnf, fanout)?;
                let frozen = FrozenMatrix::freeze(&original);
                if !frozen.verify() {
                    return Err("matrix shadow corruption".to_owned());
                }
                let witnessed = wave_cascade(&frozen, None)
                    .ok_or("matrix fabric could not inspect source")?;
                if !witnessed.accepted {
                    return Err("third fabric witness rejected source".to_owned());
                }
                let router = WaveRouter::new(&frozen)
                    .ok_or("wave source could not rebuild the signed fabric")?;
                for (index, position) in score.iter_mut().enumerate() {
                    let cell = index % CELLS;
                    let room = (index / CELLS) % fanout;
                    let played = frozen.play(room, cell)
                        .ok_or("Bobby Fischer playback rejected")?;
                    // Exact requested branch traverses 3-ary supervisors.
                    // Each node's signed child sums are frozen/reconstructed
                    // from actual matrix data; these values affect ordering.
                    let wave = router.read((index % fanout) as usize)
                        .ok_or("on-demand ternary wave traversal rejected")?;
                    if !router.verify_read(&wave) {
                        return Err("wave reflection disagreed with fabric".to_owned());
                    }
                    wave_reads = wave_reads.checked_add(1)
                        .ok_or("wave read counter overflow")?;
                    wave_choices_checked = wave_choices_checked
                        .checked_add(wave.choices_checked)
                        .ok_or("wave supervisor counter overflow")?;
                    let wave_score = wave.room_total.unsigned_abs();
                    let wave_score = i128::try_from(wave_score)
                        .map_err(|_| "wave score out of range")?;
                    *position = position
                        .checked_add(i128::from(played.value.unsigned_abs()) * 16)
                        .and_then(|v| v.checked_add(wave_score * 4))
                        .ok_or("integer matrix wave score overflow")?;
                }
                (fanout, witnessed.fabric_reads, witnessed.accepted)
            } else {
                (0, 0, true)
            };

        // Full n-ary dispatch uses actual 3, 27, or 81 independent lanes.
        // The solver's *logical* SAT branches are still the two Boolean
        // values; the 0 trit denotes an unassigned variable.
        let mut lanes = vec![Vec::<usize>::new(); fanout];
        for variable in 1..=cnf.variables {
            lanes[(variable - 1) % fanout].push(variable);
        }
        for lane in &mut lanes {
            if strategy != Strategy::Base {
                lane.sort_by_key(|&variable| (-score[variable - 1], variable));
            }
        }
        let lanes_used = lanes.iter().filter(|lane| !lane.is_empty()).count();
        let mut order = Vec::with_capacity(cnf.variables);
        let max_depth = lanes.iter().map(Vec::len).max().unwrap_or(0);
        for depth in 0..max_depth {
            for lane in &lanes {
                if let Some(&variable) = lane.get(depth) {
                    order.push(variable);
                }
            }
        }
        if order.len() != cnf.variables {
            return Err("routing omitted variables".to_owned());
        }
        let mut visited = vec![false; cnf.variables];
        for variable in &order {
            if visited[*variable - 1] {
                return Err("routing duplicated variable".to_owned());
            }
            visited[*variable - 1] = true;
        }

        Ok(Self {
            strategy, order, fanout, lanes_used,
            frozen_mtp_patterns, fischer_addresses,
            matrix_rooms, fabric_witnesses, root_accepted,
            wave_reads, wave_choices_checked,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authentic_three_twentyseven_eightyone_lane_structures() {
        let formula = Cnf::parse("p cnf 121 2\n1 27 81 121 0\n-1 -27 -81 0")
            .expect("source");
        for strategy in Strategy::all() {
            let guided = Guidance::for_source(&formula, strategy)
                .expect("all nine native engines have source guidance");
            assert!(guided.root_accepted);
            assert_eq!(guided.order.len(), 121);
            assert_eq!(guided.lanes_used, guided.fanout);
            assert!(matches!(guided.fanout, 3 | 27 | 81));
            assert_eq!(guided.matrix_rooms,
                       if strategy.uses_matrix() { guided.fanout } else { 0 });
            assert_eq!(guided.fischer_addresses,
                       if strategy.uses_fischer() { 121 } else { 0 });
            assert_eq!(guided.wave_reads,
                       if strategy.uses_matrix() { 121 } else { 0 });
            assert_eq!(guided.wave_choices_checked,
                       if strategy.uses_matrix() {
                           121 * match guided.fanout {
                               3 => 3, 27 => 9, 81 => 12, _ => unreachable!(),
                           }
                       } else { 0 });
        }
    }

    #[test]
    fn every_strategy_is_deterministic_from_source_alone() {
        let formula = Cnf::parse(
            "p cnf 9 4\n1 2 3 0\n-1 -3 4 0\n5 6 -7 0\n-6 8 9 0")
            .expect("input");
        for strategy in Strategy::all() {
            let a = Guidance::for_source(&formula, strategy).expect("a");
            let b = Guidance::for_source(&formula, strategy).expect("b");
            assert_eq!(a, b);
        }
    }
}
