//! Frozen source snapshot, exact 2D projection, and Bobby Fischer address play.
//! The shadow is a projection, not an independently invertible reconstruction.
//! The complete frozen cells remain stored in this pilot: no compression claim.

use crate::matrix::{address, Matrix, CELLS};

pub const RIME_PRIME: u64 = 1_000_081;
pub const RIME_GENERATOR: u64 = 7;
pub const RIME_GLYPHS: u64 = 27;

pub fn power_mod(mut base: u64, mut exponent: u64, modulus: u64) -> u64 {
    assert!(modulus > 1);
    let mut result = 1_u64;
    base %= modulus;
    while exponent > 0 {
        if exponent & 1 == 1 {
            result = (result * base) % modulus;
        }
        base = (base * base) % modulus;
        exponent >>= 1;
    }
    result
}

pub fn fischer_address(glyph: u64, position: u64) -> Option<u64> {
    if glyph >= RIME_GLYPHS {
        return None;
    }
    let exponent = position.checked_mul(RIME_GLYPHS)?.checked_add(glyph)?;
    Some(power_mod(RIME_GENERATOR, exponent, RIME_PRIME))
}

pub fn shadow(cells: &[i64; CELLS]) -> [i128; 9] {
    let mut out = [0_i128; 9];
    for z in 0..3 {
        for y in 0..3 {
            for x in 0..3 {
                let cell = address(x, y, z).expect("fixed 3D index");
                let projected = x + 3 * y;
                out[projected] = out[projected]
                    .checked_add(i128::from(cells[cell]))
                    .expect("sum of three i64 values fits i128");
            }
        }
    }
    out
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenMatrix {
    data: Vec<[i64; CELLS]>,
    shadow_2d: Vec<[i128; 9]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayedCell {
    pub room: usize,
    pub cell: usize,
    pub value: i64,
    pub shadow_value: i128,
    pub address: u64,
}

impl FrozenMatrix {
    pub fn freeze(matrix: &Matrix) -> Self {
        Self {
            data: matrix.rooms().to_vec(),
            shadow_2d: matrix.rooms().iter().map(shadow).collect(),
        }
    }

    pub fn verify(&self) -> bool {
        !self.data.is_empty()
            && self.data.len() == self.shadow_2d.len()
            && self.data.iter().zip(self.shadow_2d.iter())
                .all(|(cells, projection)| shadow(cells) == *projection)
    }

    pub fn room_count(&self) -> usize {
        self.data.len()
    }

    pub fn room(&self, room: usize) -> Option<&[i64; CELLS]> {
        self.data.get(room)
    }

    pub fn projection(&self, room: usize) -> Option<&[i128; 9]> {
        self.shadow_2d.get(room)
    }

    pub fn play(&self, room: usize, cell: usize) -> Option<PlayedCell> {
        if !self.verify() || cell >= CELLS {
            return None;
        }
        let source = self.data.get(room)?;
        let projection = self.shadow_2d.get(room)?;
        let x = cell % 3;
        let y = (cell / 3) % 3;
        let glyph = u64::try_from(cell).ok()?;
        let position = u64::try_from(room).ok()?;
        Some(PlayedCell {
            room,
            cell,
            value: source[cell],
            shadow_value: projection[x + 3 * y],
            address: fischer_address(glyph, position)?,
        })
    }

    pub fn retained_words(&self) -> usize {
        self.data.len() * CELLS + self.shadow_2d.len() * 9 * 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::Matrix;

    #[test]
    fn correct_frozen_shadow_and_exact_addressed_play() {
        for n in [3, 9, 27, 81] {
            let source = Matrix::generated(n).expect("source");
            let snapshot = FrozenMatrix::freeze(&source);
            assert!(snapshot.verify());
            for room in 0..n {
                for cell in 0..CELLS {
                    let played = snapshot.play(room, cell).expect("addressed");
                    assert_eq!(played.value, source.room(room).expect("room")[cell]);
                    assert_eq!(Some(played.address),
                        fischer_address(u64::try_from(cell).expect("cell"),
                                        u64::try_from(room).expect("room")));
                }
            }
            assert_eq!(snapshot.room_count(), n);
            assert!(snapshot.play(n, 0).is_none());
            assert!(snapshot.play(0, CELLS).is_none());
            assert_eq!(snapshot.retained_words(), n * (CELLS + 18));
        }
    }

    #[test]
    fn modified_shadow_is_rejected_and_must_not_play() {
        let matrix = Matrix::generated(3).expect("matrix");
        let mut frozen = FrozenMatrix::freeze(&matrix);
        frozen.shadow_2d[0][0] += 1;
        assert!(!frozen.verify());
        assert!(frozen.play(0, 0).is_none());
    }

    #[test]
    fn altered_frozen_cell_is_rejected() {
        let matrix = Matrix::generated(3).expect("matrix");
        let mut frozen = FrozenMatrix::freeze(&matrix);
        frozen.data[1][13] += 1;
        assert!(!frozen.verify());
    }

    #[test]
    fn address_is_exact_integer_without_hidden_state() {
        assert_eq!(power_mod(7, 0, RIME_PRIME), 1);
        assert_eq!(fischer_address(0, 0), Some(1));
        assert_eq!(fischer_address(1, 0), Some(7));
        assert_eq!(fischer_address(27, 0), None);
        assert_eq!(fischer_address(1, u64::MAX), None);
        assert_eq!(power_mod(7, RIME_PRIME - 1, RIME_PRIME), 1);
    }
}
