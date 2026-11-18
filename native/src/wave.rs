//! On-demand three-way room traversal over exact signed matrix witnesses.
//! Only the requested path's supervisors are materialized as values.
//! SAT Zero is unassigned; the Relic reference center is always (0, 1).
#![forbid(unsafe_code)]

use crate::freeze::FrozenMatrix;
use crate::relic::{balanced_address, center_at_z};
use crate::ternary::Trit;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WaveRead {
    pub room: usize,
    pub digits: Vec<Trit>,
    pub address: u128,
    pub levels: usize,
    pub root_total: i128,
    pub room_total: i128,
    pub center_value: u8,
    pub choices_checked: usize,
}

pub struct WaveRouter<'a> {
    frozen: &'a FrozenMatrix,
    room_count: usize,
    levels: usize,
}

fn signed_digit(slot: usize) -> Option<Trit> {
    match slot {
        0 => Some(Trit::Minus),
        1 => Some(Trit::Zero),
        2 => Some(Trit::Plus),
        _ => None,
    }
}

// Exact signed difference transform over arbitrary bounded i128 totals.
fn reconstruct(values: [i128; 3]) -> Option<[i128; 3]> {
    let sum = values.iter().try_fold(0_i128, |a, b| a.checked_add(*b))?;
    let d0 = values[0].checked_mul(3)?.checked_sub(sum)?;
    let d1 = values[1].checked_mul(3)?.checked_sub(sum)?;
    let d2 = d0.checked_add(d1)?.checked_neg()?;
    let mut recovered = [0_i128; 3];
    for (index, d) in [d0, d1, d2].iter().enumerate() {
        let numerator = sum.checked_add(*d)?;
        if numerator % 3 != 0 {
            return None;
        }
        recovered[index] = numerator / 3;
    }
    (recovered == values).then_some(recovered)
}

impl<'a> WaveRouter<'a> {
    pub fn new(frozen: &'a FrozenMatrix) -> Option<Self> {
        if !frozen.verify() || !matches!(frozen.room_count(), 3 | 9 | 27 | 81) {
            return None;
        }
        let room_count = frozen.room_count();
        let mut width = room_count;
        let mut levels = 0_usize;
        while width > 1 {
            if width % 3 != 0 {
                return None;
            }
            width /= 3;
            levels += 1;
        }
        Some(Self { frozen, room_count, levels })
    }

    pub fn levels(&self) -> usize {
        self.levels
    }

    fn room_sum(&self, room: usize) -> Option<i128> {
        let mut result = 0_i128;
        for column in 0..9 {
            // The matrix seat's frozen center-and-difference witness is the
            // actual data source for every addressed room observation.
            let triple = self.frozen.reconstruct_triple(room, column)?;
            for value in triple {
                result = result.checked_add(i128::from(value))?;
            }
        }
        Some(result)
    }

    fn subtree_sum(&self, start: usize, width: usize) -> Option<i128> {
        if start.checked_add(width)? > self.room_count {
            return None;
        }
        let mut total = 0_i128;
        for room in start..start + width {
            total = total.checked_add(self.room_sum(room)?)?;
        }
        Some(total)
    }

    pub fn read(&self, room: usize) -> Option<WaveRead> {
        if room >= self.room_count {
            return None;
        }
        let mut digits = Vec::with_capacity(self.levels);
        let mut start = 0_usize;
        let mut width = self.room_count;
        let mut root_total = None;
        let mut choices_checked = 0_usize;
        for _ in 0..self.levels {
            let stride = width / 3;
            let parts = [
                self.subtree_sum(start, stride)?,
                self.subtree_sum(start + stride, stride)?,
                self.subtree_sum(start + stride * 2, stride)?,
            ];
            // Freeze and reconstruct at every supervisor, rather than
            // treating three child totals as mere labels.
            if reconstruct(parts)? != parts {
                return None;
            }
            let total = parts.iter().try_fold(0_i128, |a, b| a.checked_add(*b))?;
            let slot = (room - start) / stride;
            let digit = signed_digit(slot)?;
            if center_at_z(digit) != (0, 1) {
                return None;
            }
            if digits.is_empty() {
                root_total = Some(total);
            }
            digits.push(digit);
            start += slot * stride;
            width = stride;
            choices_checked += 3;
            // Root stores full total; the next iteration checks its selected
            // subtree independently as a separate supervisor.
            if digits.len() < self.levels {
                // Preserve full root while validating next level directly.
                let selected = parts[slot];
                if self.subtree_sum(start, width)? != selected {
                    return None;
                }
            }
        }
        let room_total = self.room_sum(room)?;
        let address = balanced_address(&digits)?;
        if address != u128::try_from(room).ok()? || start != room || width != 1 {
            return None;
        }
        Some(WaveRead {
            room,
            digits,
            address,
            levels: self.levels,
            root_total: root_total?,
            room_total,
            center_value: 1,
            choices_checked,
        })
    }

    pub fn verify_read(&self, candidate: &WaveRead) -> bool {
        self.read(candidate.room).as_ref() == Some(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::Matrix;

    #[test]
    fn all_120_room_paths_are_exact_and_lazily_selected() {
        let mut reads = 0_usize;
        let mut choices = 0_usize;
        for rooms in [3_usize, 9, 27, 81] {
            let original = Matrix::generated(rooms).expect("source");
            let frozen = FrozenMatrix::freeze(&original);
            let router = WaveRouter::new(&frozen).expect("verified");
            for room in 0..rooms {
                let result = router.read(room).expect("path");
                assert_eq!(result.address, room as u128);
                assert_eq!(result.levels, router.levels());
                assert_eq!(result.center_value, 1);
                assert_eq!(result.choices_checked, 3 * router.levels());
                let actual: i128 = original.room(room).expect("room").iter()
                    .map(|&x| i128::from(x)).sum();
                assert_eq!(result.room_total, actual);
                assert!(router.verify_read(&result));
                let mut corrupt = result.clone();
                corrupt.room_total += 1;
                assert!(!router.verify_read(&corrupt));
                let mut wrong_path = result.clone();
                wrong_path.digits[0] = wrong_path.digits[0].negate();
                wrong_path.address += 1;
                assert!(!router.verify_read(&wrong_path));
                reads += 1;
                choices += result.choices_checked;
            }
            assert!(router.read(rooms).is_none());
        }
        assert_eq!(reads, 120);
        assert_eq!(choices, 1278);
    }

    #[test]
    fn signed_difference_recovery_rejects_overflow() {
        assert_eq!(reconstruct([-1, 0, 1]), Some([-1, 0, 1]));
        assert_eq!(reconstruct([i128::MAX, 0, 0]), None);
    }
}
