//! Live SAT partial-assignment frames for the 27-cell P-NP matrix.
//! The values are actual unknown/false/true solver states, not a known
//! Bitcoin reference header. Exact round-trip and tamper rejection only.
//! Search-space acceleration is measured in a separate controlled gate.
#![forbid(unsafe_code)]

use crate::brown_ledger::BrownSlice;
use crate::matrix::CELLS;
use crate::sha256::hash_bytes;
use crate::ternary::Trit;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveFrame {
    pub first_variable: usize,
    pub used: usize,
    pub source_sha256: String,
    pub state_sha256: String,
    pub frozen: Vec<u8>,
}

fn value(trit: Trit) -> i64 {
    match trit {
        Trit::Minus => -1,
        Trit::Zero => 0,
        Trit::Plus => 1,
    }
}

fn trit(value: i64) -> Result<Trit, String> {
    match value {
        -1 => Ok(Trit::Minus),
        0 => Ok(Trit::Zero),
        1 => Ok(Trit::Plus),
        _ => Err("live matrix trit outside balanced domain".to_owned()),
    }
}

fn state_digest(source: &str, first: usize, used: usize, states: &[Trit; CELLS]) -> String {
    let mut bytes = Vec::with_capacity(source.len() + 32 + CELLS);
    bytes.extend_from_slice(b"PNP_LIVE_SAT_27_V1\0");
    bytes.extend_from_slice(source.as_bytes());
    bytes.extend_from_slice(&u64::try_from(first).unwrap_or(u64::MAX).to_le_bytes());
    bytes.extend_from_slice(&u64::try_from(used).unwrap_or(u64::MAX).to_le_bytes());
    for &state in states {
        bytes.push((value(state) + 1) as u8);
    }
    hash_bytes(&bytes)
}

pub fn freeze_live(
    source_sha256: &str,
    first_variable: usize,
    states: &[Trit],
) -> Result<LiveFrame, String> {
    if source_sha256.len() != 64 || !source_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        || states.is_empty() || states.len() > CELLS || first_variable == 0
        || first_variable.checked_add(states.len()).is_none()
    {
        return Err("invalid source-bound live SAT frame".to_owned());
    }
    let mut padded = [Trit::Zero; CELLS];
    padded[..states.len()].copy_from_slice(states);
    let mut values = [0_i64; CELLS];
    for (slot, &state) in values.iter_mut().zip(padded.iter()) {
        *slot = value(state);
    }
    let frozen = BrownSlice::freeze(&values).serialized().to_vec();
    let state_sha256 = state_digest(source_sha256, first_variable, states.len(), &padded);
    let frame = LiveFrame {
        first_variable,
        used: states.len(),
        source_sha256: source_sha256.to_owned(),
        state_sha256,
        frozen,
    };
    if play_live(&frame, source_sha256)? != states {
        return Err("live SAT frame failed exact replay".to_owned());
    }
    Ok(frame)
}

pub fn play_live(frame: &LiveFrame, source_sha256: &str) -> Result<Vec<Trit>, String> {
    if source_sha256 != frame.source_sha256
        || frame.used == 0 || frame.used > CELLS || frame.first_variable == 0
        || frame.first_variable.checked_add(frame.used).is_none()
    {
        return Err("live SAT frame source or dimension differs".to_owned());
    }
    let decoded = BrownSlice::load(frame.frozen.clone())
        .map_err(str::to_owned)?
        .play()
        .map_err(str::to_owned)?;
    let mut states = [Trit::Zero; CELLS];
    for (slot, &v) in states.iter_mut().zip(decoded.iter()) {
        *slot = trit(v)?;
    }
    if states[frame.used..].iter().any(|v| *v != Trit::Zero) {
        return Err("live SAT frame padding is not neutral".to_owned());
    }
    if state_digest(source_sha256, frame.first_variable, frame.used, &states)
        != frame.state_sha256
    {
        return Err("live SAT frame digest differs".to_owned());
    }
    Ok(states[..frame.used].to_vec())
}

pub fn freeze_assignment(
    source_sha256: &str,
    assignment: &[Trit],
) -> Result<Vec<LiveFrame>, String> {
    if assignment.is_empty() {
        return Err("empty SAT assignment".to_owned());
    }
    assignment.chunks(CELLS).enumerate().map(|(i, slice)| {
        freeze_live(source_sha256, i * CELLS + 1, slice)
    }).collect()
}

pub fn play_assignment(
    source_sha256: &str,
    frames: &[LiveFrame],
    variables: usize,
) -> Result<Vec<Trit>, String> {
    if variables == 0 || frames.len() != variables.div_ceil(CELLS) {
        return Err("live SAT frame count differs".to_owned());
    }
    let mut result = Vec::with_capacity(variables);
    for frame in frames {
        if frame.first_variable != result.len() + 1 {
            return Err("live SAT frame order or variable address differs".to_owned());
        }
        result.extend(play_live(frame, source_sha256)?);
    }
    if result.len() != variables {
        return Err("live SAT assignment length differs".to_owned());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";

    #[test]
    fn exact_partial_states_and_live_updates() {
        let mut states = vec![Trit::Zero; 55];
        states[0] = Trit::Minus;
        states[26] = Trit::Plus;
        states[27] = Trit::Minus;
        let frames = freeze_assignment(SOURCE, &states).expect("freeze");
        assert_eq!(frames.len(), 3);
        assert_eq!(play_assignment(SOURCE, &frames, 55).expect("play"), states);
        states[26] = Trit::Minus;
        states[27] = Trit::Plus;
        let updated = freeze_assignment(SOURCE, &states).expect("update");
        assert_ne!(updated[0].state_sha256, frames[0].state_sha256);
        assert_ne!(updated[1].state_sha256, frames[1].state_sha256);
        assert_eq!(play_assignment(SOURCE, &updated, 55).expect("updated"), states);
    }

    #[test]
    fn original_cnf_solver_model_roundtrips_through_live_frames() {
        use crate::guidance::{Guidance, Strategy};
        use crate::solver::{solve, Status};
        use crate::ternary::Cnf;
        let source = "p cnf 3 3\n1 2 0\n-1 3 0\n-2 3 0\n";
        let cnf = Cnf::parse(source).expect("original clauses");
        let guidance = Guidance::for_source(&cnf, Strategy::Base).expect("branch order");
        let solved = solve(&cnf, &guidance, Some(128));
        assert_eq!(solved.status, Status::Sat);
        let model = solved.model.expect("complete SAT assignment");
        let root = hash_bytes(source.as_bytes());
        let frozen = freeze_assignment(&root, &model).expect("freeze actual SAT model");
        let replayed = play_assignment(&root, &frozen, cnf.variables)
            .expect("play original SAT model");
        assert!(cnf.valid_complete_model(&replayed));
        assert_eq!(replayed, model);
    }

    #[test]
    fn corruption_replay_and_wrong_source_fail() {
        let states = vec![Trit::Plus, Trit::Zero, Trit::Minus];
        let frame = freeze_live(SOURCE, 1, &states).expect("freeze");
        assert!(play_live(&frame, &"0".repeat(64)).is_err());
        let mut altered = frame.clone();
        altered.state_sha256 = "0".repeat(64);
        assert!(play_live(&altered, SOURCE).is_err());
        altered = frame.clone();
        altered.first_variable += 1;
        assert!(play_live(&altered, SOURCE).is_err());
        altered = frame.clone();
        altered.frozen[0] ^= 1;
        assert!(play_live(&altered, SOURCE).is_err());
        assert!(play_assignment(SOURCE, &[frame], 27).is_err());
    }
}
