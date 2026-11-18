//! Native integer port of the exact mathematical interfaces in
//! THE-RELIC-REDISCOVERY: invariant center, balanced ternary prefixes,
//! wave intermediate depths, non-uniform structural landing depths, and
//! injective (center, signed-differences) readouts for 27 ordered triples.
//! The mathematical model is separate from SAT correctness claims.

use crate::ternary::Trit;

pub const CENTER_COORDINATE: i8 = 0;
pub const CENTER_VALUE: u8 = 1;

pub fn exponent(level: u32) -> Option<u32> {
    let product = level.checked_mul(level.checked_add(3)?)?;
    3_u32.checked_add(product / 2)
}

pub fn power_three(power: u32) -> Option<u128> {
    3_u128.checked_pow(power)
}

pub fn anchor(level: u32) -> Option<u128> {
    power_three(exponent(level)?)
}

pub fn multiplier(level: u32) -> Option<u128> {
    power_three(level.checked_add(2)?)
}

pub fn waves(level: u32) -> Option<Vec<u128>> {
    let initial = anchor(level)?;
    let width = usize::try_from(level.checked_add(2)?).ok()?;
    // A finite executable test chooses a bounded prefix of the exact
    // unbounded mathematical recurrence.
    if width > 80 {
        return None;
    }
    let mut current = initial;
    let mut results = Vec::with_capacity(width);
    for _ in 0..width {
        current = current.checked_mul(3)?;
        results.push(current);
    }
    Some(results)
}

pub fn center_at_z(_z: Trit) -> (i8, u8) {
    (CENTER_COORDINATE, CENTER_VALUE)
}

pub fn balanced_address(digits: &[Trit]) -> Option<u128> {
    let mut address = 0_u128;
    for &digit in digits {
        let encoded = match digit {
            Trit::Minus => 0_u128,
            Trit::Zero => 1_u128,
            Trit::Plus => 2_u128,
        };
        address = address.checked_mul(3)?.checked_add(encoded)?;
    }
    Some(address)
}

pub fn prefix_subtree_size(depth: u32, prefix: &[Trit]) -> Option<u128> {
    let used = u32::try_from(prefix.len()).ok()?;
    power_three(depth.checked_sub(used)?)
}

pub fn selected_nodes(depth: u32, prefix: &[Trit]) -> Option<u128> {
    let remaining = depth.checked_sub(u32::try_from(prefix.len()).ok()?)?;
    let leaves = power_three(remaining.checked_add(1)?)?;
    Some((leaves - 1) / 2)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FreeCenterReadout {
    pub sum: i128,
    pub delta0: i128,
    pub delta1: i128,
}

pub fn freeze_triple(values: [i64; 3]) -> FreeCenterReadout {
    let sum = values.iter().map(|&x| i128::from(x)).sum::<i128>();
    FreeCenterReadout {
        sum,
        delta0: 3 * i128::from(values[0]) - sum,
        delta1: 3 * i128::from(values[1]) - sum,
    }
}

pub fn rebuild_triple(readout: FreeCenterReadout) -> Option<[i64; 3]> {
    let delta2 = readout.delta0.checked_add(readout.delta1)?.checked_neg()?;
    let mut result = [0_i64; 3];
    for (index, delta) in [readout.delta0, readout.delta1, delta2]
        .into_iter().enumerate()
    {
        let value = readout.sum.checked_add(delta)?;
        if value % 3 != 0 {
            return None;
        }
        result[index] = i64::try_from(value / 3).ok()?;
    }
    if freeze_triple(result) != readout {
        return None;
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn all_intermediate_waves_and_landings_have_correct_counts() {
        let expected = [27_u128, 243, 6_561, 531_441, 129_140_163];
        let depths = [3_u32, 5, 8, 12, 17];
        for level in 0..5 {
            let i = u32::try_from(level).expect("level");
            assert_eq!(anchor(i), Some(expected[level]));
            assert_eq!(exponent(i), Some(depths[level]));
            assert_eq!(anchor(i).and_then(|a| {
                a.checked_mul(multiplier(i)?)
            }), anchor(i + 1));
            assert_eq!(waves(i).expect("waves").last().copied(), anchor(i + 1));
        }
        assert_eq!(waves(0), Some(vec![81, 243]));
        assert_eq!(waves(1), Some(vec![729, 2_187, 6_561]));
    }

    #[test]
    fn one_unchanged_center_and_all_balanced_ternary_addresses() {
        for z in [Trit::Minus, Trit::Zero, Trit::Plus] {
            assert_eq!(center_at_z(z), (0, 1));
        }
        let mut observed = BTreeSet::new();
        for a in [Trit::Minus, Trit::Zero, Trit::Plus] {
            for b in [Trit::Minus, Trit::Zero, Trit::Plus] {
                for c in [Trit::Minus, Trit::Zero, Trit::Plus] {
                    observed.insert(balanced_address(&[a, b, c]).expect("address"));
                }
            }
        }
        assert_eq!(observed, (0_u128..27).collect());
        assert_eq!(prefix_subtree_size(4, &[Trit::Plus]), Some(27));
        assert_eq!(selected_nodes(4, &[Trit::Plus]), Some(40));
        assert_eq!(prefix_subtree_size(2, &[Trit::Minus; 3]), None);
    }

    #[test]
    fn full_center_and_differences_are_injective_on_every_trit_triple() {
        let mut readouts = BTreeSet::new();
        let mut intensity = BTreeSet::new();
        for a in -1_i64..=1 {
            for b in -1_i64..=1 {
                for c in -1_i64..=1 {
                    let original = [a, b, c];
                    let readout = freeze_triple(original);
                    assert_eq!(rebuild_triple(readout), Some(original));
                    assert!(readouts.insert((readout.sum, readout.delta0,
                                             readout.delta1)));
                    let q = a * a + b * b + c * c - a * b - a * c - b * c;
                    intensity.insert((readout.sum * readout.sum, q));
                }
            }
        }
        assert_eq!(readouts.len(), 27);
        assert!(intensity.len() < readouts.len());
        assert_eq!(rebuild_triple(FreeCenterReadout {
            sum: 0, delta0: 1, delta1: 0
        }), None);
    }
}
