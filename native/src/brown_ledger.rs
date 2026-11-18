//! Brown Ledger: paid Shannon closure, exact center slices and reversible deltas.
//! This is an integer information-and-identity gate, not trained-GNN inference.
//! Source: 2026-07-22 Brown Ledger / RIME container receipts and referee handoff.
//! All bytes charged include center, both separations, remainder and source SHA.
#![forbid(unsafe_code)]

use crate::binary_ternary_bridge::{decode_route, encode_route};
use crate::freeze::{fischer_address, FrozenMatrix};
use crate::fischer_prime_tower::reverse_fischer_address;
use crate::matrix::{kind, CellKind, Matrix, CELLS, OMEGA};
use crate::prism;
use crate::sha256::hash_bytes;
use crate::ternary::Trit;
use crate::triad::{wave_cascade, wave_cascade_balanced, Fault, Seat};

fn state_bytes(state: &[i64; CELLS]) -> Vec<u8> {
    let mut out = Vec::with_capacity(CELLS * 8);
    for value in state {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}
fn state_sha(state: &[i64; CELLS]) -> String {
    hash_bytes(&state_bytes(state))
}
fn put_signed(value: i128, dst: &mut Vec<u8>) {
    // All values entering this encoder are differences/sums of at most
    // three i64 values, hence no i128::MIN or overflow possibility.
    let mut word = if value < 0 {
        ((-value) as u128) * 2 - 1
    } else {
        (value as u128) * 2
    };
    while word >= 128 {
        dst.push(((word as u8) & 0x7f) | 0x80);
        word >>= 7;
    }
    dst.push(word as u8);
}
fn get_signed(src: &[u8], cursor: &mut usize) -> Result<i128, &'static str> {
    let mut word = 0_u128;
    let mut shift = 0_u32;
    for byte_number in 0..19 {
        let byte = *src.get(*cursor).ok_or("truncated signed field")?;
        *cursor += 1;
        let part = u128::from(byte & 0x7f);
        if part > (u128::MAX >> shift) {
            return Err("signed field overflow");
        }
        word |= part << shift;
        if byte & 0x80 == 0 {
            if byte_number > 0 && part == 0 {
                return Err("noncanonical signed field");
            }
            let magnitude = i128::try_from(word >> 1)
                .map_err(|_| "signed magnitude overflow")?;
            return Ok(if word & 1 == 0 { magnitude } else { -magnitude - 1 });
        }
        shift += 7;
        if shift >= 128 {
            return Err("signed field too wide");
        }
    }
    Err("signed field exceeds 19 bytes")
}

/// One 2D shadow column: center = floor((a+b+c)/3), remainder in 0..2,
/// and two signed separations. The third is the paid closure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CenterColumn {
    pub center: i128,
    pub remainder: u8,
    pub negative: i128,
    pub positive: i128,
}
impl CenterColumn {
    pub fn freeze([a, b, c]: [i64; 3]) -> Self {
        let sum = i128::from(a) + i128::from(b) + i128::from(c);
        let center = sum.div_euclid(3);
        Self {
            center,
            remainder: sum.rem_euclid(3) as u8,
            negative: i128::from(a) - center,
            positive: i128::from(b) - center,
        }
    }
    pub fn play(self) -> Result<[i64; 3], &'static str> {
        if self.remainder >= 3 {
            return Err("unpaid or invalid remainder");
        }
        let a = self.center.checked_add(self.negative).ok_or("center overflow")?;
        let b = self.center.checked_add(self.positive).ok_or("center overflow")?;
        let c = self.center.checked_add(i128::from(self.remainder))
            .and_then(|n| n.checked_sub(self.negative))
            .and_then(|n| n.checked_sub(self.positive))
            .ok_or("unpaid closure overflow")?;
        let triple = [
            i64::try_from(a).map_err(|_| "first value outside i64")?,
            i64::try_from(b).map_err(|_| "second value outside i64")?,
            i64::try_from(c).map_err(|_| "third value outside i64")?,
        ];
        if Self::freeze(triple) != self {
            return Err("noncanonical center/closure");
        }
        Ok(triple)
    }
}

/// A self-contained frozen slice. The physical binary record includes a
/// version, nine paid center/separation/remainder records and full SHA-256.
/// It does not secretly retain a raw 27-cell copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrownSlice {
    bytes: Vec<u8>,
}
impl BrownSlice {
    pub fn freeze(state: &[i64; CELLS]) -> Self {
        let mut bytes = vec![1_u8];
        for column in 0..9 {
            let triple = [state[column], state[column + 9], state[column + 18]];
            let code = CenterColumn::freeze(triple);
            put_signed(code.center, &mut bytes);
            put_signed(code.negative, &mut bytes);
            put_signed(code.positive, &mut bytes);
            bytes.push(code.remainder);
        }
        bytes.extend_from_slice(state_sha(state).as_bytes());
        Self { bytes }
    }
    pub fn load(bytes: Vec<u8>) -> Result<Self, &'static str> {
        let slice = Self { bytes };
        slice.play()?;
        Ok(slice)
    }
    pub fn serialized(&self) -> &[u8] {
        &self.bytes
    }
    pub fn paid_bytes(&self) -> usize {
        self.bytes.len()
    }
    pub fn play(&self) -> Result<[i64; CELLS], &'static str> {
        if self.bytes.first() != Some(&1) {
            return Err("wrong Brown Ledger slice version");
        }
        let mut cursor = 1_usize;
        let mut state = [0_i64; CELLS];
        for column in 0..9 {
            let center = get_signed(&self.bytes, &mut cursor)?;
            let negative = get_signed(&self.bytes, &mut cursor)?;
            let positive = get_signed(&self.bytes, &mut cursor)?;
            let remainder = *self.bytes.get(cursor).ok_or("missing closure remainder")?;
            cursor += 1;
            let restored = CenterColumn { center, remainder, negative, positive }.play()?;
            for (z, value) in restored.into_iter().enumerate() {
                state[column + 9 * z] = value;
            }
        }
        if self.bytes.len() != cursor + 64
            || self.bytes[cursor..] != state_sha(&state).as_bytes()[..]
        {
            return Err("frozen slice identity or length rejected");
        }
        Ok(state)
    }
    /// Fischer navigates from null cell 13 through faces, edges, corners.
    /// This is authenticated ordering/addressing, not a DLP speed claim.
    pub fn play_from_zero(&self, room: usize)
        -> Result<Vec<(usize, i64, u64)>, &'static str>
    {
        let restored = self.play()?;
        let mut indices: Vec<usize> = (0..CELLS).collect();
        indices.sort_by_key(|index| match kind(*index) {
            Some(CellKind::Omega) => 0,
            Some(CellKind::Face) => 1,
            Some(CellKind::Edge) => 2,
            Some(CellKind::Corner) => 3,
            None => 4,
        });
        let mut out = Vec::with_capacity(CELLS);
        for index in indices {
            let address = fischer_address(index as u64, room as u64)
                .ok_or("invalid Fischer null-outward address")?;
            if reverse_fischer_address(address) != Some((index as u64, room as u64)) {
                return Err("Fischer prime-tower reverse address rejected");
            }
            out.push((index, restored[index], address));
        }
        if out.first().map(|entry| entry.0) != Some(OMEGA) {
            return Err("Fischer null center changed");
        }
        Ok(out)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Forward,
    Reverse,
}
/// An exact signed sparse state change, with source/target identities.
/// Digest and metadata are charged, not treated as free side information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrownDelta {
    bytes: Vec<u8>,
}
impl BrownDelta {
    pub fn freeze(from: &[i64; CELLS], to: &[i64; CELLS]) -> Self {
        let mut bytes = vec![2_u8];
        bytes.extend_from_slice(state_sha(from).as_bytes());
        bytes.extend_from_slice(state_sha(to).as_bytes());
        let count = (0..CELLS).filter(|i| from[*i] != to[*i]).count();
        bytes.push(count as u8);
        for i in 0..CELLS {
            if from[i] != to[i] {
                bytes.push(i as u8);
                put_signed(i128::from(to[i]) - i128::from(from[i]), &mut bytes);
            }
        }
        Self { bytes }
    }
    pub fn serialized(&self) -> &[u8] {
        &self.bytes
    }
    pub fn paid_bytes(&self) -> usize {
        self.bytes.len()
    }
    pub fn load(bytes: Vec<u8>, from: &[i64; CELLS])
        -> Result<Self, &'static str>
    {
        let candidate = Self { bytes };
        candidate.play(from, Direction::Forward)?;
        Ok(candidate)
    }
    pub fn play(&self, state: &[i64; CELLS], direction: Direction)
        -> Result<[i64; CELLS], &'static str>
    {
        if self.bytes.len() < 130 || self.bytes[0] != 2 {
            return Err("invalid delta version or length");
        }
        let source = &self.bytes[1..65];
        let target = &self.bytes[65..129];
        let (expected, destination) = match direction {
            Direction::Forward => (source, target),
            Direction::Reverse => (target, source),
        };
        if state_sha(state).as_bytes() != expected {
            return Err("delta input source hash changed");
        }
        let count = usize::from(self.bytes[129]);
        if count > CELLS {
            return Err("delta exceeds 27 cells");
        }
        let mut cursor = 130_usize;
        let mut out = *state;
        let mut last = None;
        for _ in 0..count {
            let index = usize::from(*self.bytes.get(cursor).ok_or("missing delta cell")?);
            cursor += 1;
            if index >= CELLS || last.is_some_and(|previous| index <= previous) {
                return Err("unsorted or repeated delta address");
            }
            last = Some(index);
            let signed = get_signed(&self.bytes, &mut cursor)?;
            if signed == 0 {
                return Err("zero delta must be omitted");
            }
            let moved = match direction {
                Direction::Forward => i128::from(out[index]).checked_add(signed),
                Direction::Reverse => i128::from(out[index]).checked_sub(signed),
            }.ok_or("delta arithmetic overflow")?;
            out[index] = i64::try_from(moved).map_err(|_| "delta value outside i64")?;
        }
        if cursor != self.bytes.len() || state_sha(&out).as_bytes() != destination {
            return Err("delta target source hash or trailing bytes rejected");
        }
        Ok(out)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LedgerStats {
    pub rooms: usize,
    pub checked_cells: usize,
    pub source_bytes: usize,
    pub slice_bytes: usize,
    pub delta_bytes: usize,
    pub corruptions_rejected: usize,
    pub floor_roundtrips: usize,
}

/// Complete local verification gate against existing frozen 3-seat architecture.
pub fn certify() -> Result<LedgerStats, &'static str> {
    let mut stats = LedgerStats {
        rooms: 0, checked_cells: 0, source_bytes: 0,
        slice_bytes: 0, delta_bytes: 0, corruptions_rejected: 0,
        floor_roundtrips: 0,
    };
    for rooms in [3, 9, 27, 81] {
        let matrix = Matrix::generated(rooms)?;
        let frozen = FrozenMatrix::freeze(&matrix);
        if !frozen.verify() || !wave_cascade(&frozen, None)
            .ok_or("missing N-Nest three-seat verdict")?.accepted
        {
            return Err("N-Nest original independent fabric failed");
        }
        let bad = wave_cascade_balanced(&frozen, Fault {
            level: 0, seat: Seat::Worker,
        }).ok_or("missing balanced triad negative control")?;
        if bad.accepted {
            return Err("balanced triad fault accepted");
        }
        stats.corruptions_rejected += 1;
        for room in 0..rooms {
            let original = matrix.room(room).ok_or("missing matrix room")?;
            let snapshot = BrownSlice::freeze(original);
            let wire = BrownSlice::load(snapshot.serialized().to_vec())?;
            let restored = wire.play()?;
            if restored != *original {
                return Err("original 27-cell slice mismatch");
            }
            let visited = wire.play_from_zero(room)?;
            if visited.len() != CELLS || visited.iter()
                .any(|(cell, value, addr)| *value != original[*cell]
                     || Some(*addr) != fischer_address(*cell as u64, room as u64))
            {
                return Err("Fischer null-outward replay mismatch");
            }
            let mut modified = restored;
            modified[OMEGA] = modified[OMEGA].checked_add(1)
                .ok_or("test source outside bounds")?;
            let delta = BrownDelta::freeze(&restored, &modified);
            let forward = delta.play(&restored, Direction::Forward)?;
            if forward != modified
                || delta.play(&forward, Direction::Reverse)? != restored
                || BrownDelta::load(delta.serialized().to_vec(), &restored)? != delta
            {
                return Err("freeze/play backward mismatch");
            }
            let mut invalid = snapshot.serialized().to_vec();
            invalid[1] ^= 1;
            if BrownSlice::load(invalid).is_ok() {
                return Err("mutated closure not rejected");
            }
            stats.corruptions_rejected += 1;
            let mut bad_delta = delta.serialized().to_vec();
            let final_byte = bad_delta.len() - 1;
            bad_delta[final_byte] ^= 1;
            if BrownDelta::load(bad_delta, &restored).is_ok() {
                return Err("mutated signed delta not rejected");
            }
            stats.corruptions_rejected += 1;
            let trits: Vec<Trit> = restored.iter().take(8).map(|&v| match v.rem_euclid(3) {
                0 => Trit::Minus,
                1 => Trit::Zero,
                _ => Trit::Plus,
            }).collect();
            let words = encode_route(&trits)?;
            if decode_route(&words, trits.len())? != trits {
                return Err("binary/ternary floors altered trits");
            }
            let payload = state_bytes(&restored);
            let prism_words = prism::encode(&payload);
            if prism::decode(&prism_words, payload.len())? != payload {
                return Err("Q-PRISM inverse changed binary snapshot");
            }
            stats.floor_roundtrips += 1;
            stats.rooms += 1;
            stats.checked_cells += CELLS;
            stats.source_bytes += CELLS * 8;
            stats.slice_bytes += snapshot.paid_bytes();
            stats.delta_bytes += delta.paid_bytes();
        }
    }
    if stats.rooms != 120 || stats.checked_cells != 3240
        || stats.corruptions_rejected != 244 || stats.floor_roundtrips != 120
    {
        return Err("Brown Ledger acceptance totals changed");
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_and_remainder_recover_edges() {
        let cases = [
            [0, 0, 0], [-1, 0, 1], [7, 7, 7], [-99, 3, 100],
            [i64::MIN, i64::MAX, 0], [i64::MAX, i64::MAX, i64::MIN],
        ];
        for triple in cases {
            let code = CenterColumn::freeze(triple);
            assert_eq!(code.play(), Ok(triple));
            let mut bad = code;
            bad.remainder = 3;
            assert!(bad.play().is_err());
        }
    }

    #[test]
    fn snapshots_deltas_and_negatives_all_shell_sizes() {
        let report = certify().expect("Brown Ledger exact gate");
        assert_eq!(report.rooms, 120);
        assert_eq!(report.checked_cells, 3240);
        assert_eq!(report.floor_roundtrips, 120);
        assert_eq!(report.corruptions_rejected, 244);
    }

    #[test]
    fn honest_metadata_and_signed_boundary_checks() {
        let mut original = [0_i64; CELLS];
        original[0] = i64::MIN;
        original[OMEGA] = i64::MAX;
        original[26] = -1;
        let slice = BrownSlice::freeze(&original);
        assert_eq!(slice.play(), Ok(original));
        let mut other = original;
        other[0] = i64::MAX;
        other[OMEGA] = i64::MIN;
        let d = BrownDelta::freeze(&original, &other);
        assert_eq!(d.play(&original, Direction::Forward), Ok(other));
        assert_eq!(d.play(&other, Direction::Reverse), Ok(original));
        assert!(d.play(&other, Direction::Forward).is_err());
        let mut extra = slice.serialized().to_vec();
        extra.push(0);
        assert!(BrownSlice::load(extra).is_err());
        let mut truncated = d.serialized().to_vec();
        truncated.pop();
        assert!(BrownDelta::load(truncated, &original).is_err());
        assert_eq!(state_bytes(&original).len(), CELLS * 8);
    }
}
