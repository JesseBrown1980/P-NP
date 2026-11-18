//! Native exact Pohlig–Hellman inverse for the original smooth Rime sphere.
//! Original source: historical/asolaria-2026-07-21/rime_fischer.py (unchanged).
//! One modular Fischer digit cascade per prime-power tower, recombined by CRT.
//! This is exact for the pinned smooth 1_000_080-order group, not generic DLP.
#![forbid(unsafe_code)]

use crate::freeze::{power_mod, RIME_GENERATOR, RIME_GLYPHS, RIME_PRIME};

pub const ORDER: u64 = RIME_PRIME - 1;
const TOWERS: [(u64, u32); 4] = [(2, 4), (3, 3), (5, 1), (463, 1)];

fn baby_step_giant_step(base: u64, target: u64, order: u64) -> Option<u64> {
    let mut size = 1_u64;
    while size.checked_mul(size)? < order {
        size += 1;
    }
    let mut baby = std::collections::BTreeMap::new();
    let mut value = 1_u64;
    for j in 0..size {
        baby.entry(value).or_insert(j);
        value = (value * base) % RIME_PRIME;
    }
    let stride_inverse = power_mod(
        power_mod(base, size, RIME_PRIME), RIME_PRIME - 2, RIME_PRIME,
    );
    let mut giant = target;
    for i in 0..=size {
        if let Some(j) = baby.get(&giant) {
            let exponent = i.checked_mul(size)?.checked_add(*j)?;
            if exponent < order
                && power_mod(base, exponent, RIME_PRIME) == target
            {
                return Some(exponent);
            }
        }
        giant = (giant * stride_inverse) % RIME_PRIME;
    }
    None
}
fn digit_cascade(target: u64, prime: u64, power: u32) -> Option<(u64, u64)> {
    let gamma = power_mod(RIME_GENERATOR, ORDER / prime, RIME_PRIME);
    let generator_inverse = power_mod(RIME_GENERATOR, RIME_PRIME - 2, RIME_PRIME);
    let mut exponent = 0_u64;
    let mut size = 1_u64;
    let mut place = 1_u64;
    for _ in 0..power {
        size = size.checked_mul(prime)?;
        let corrected = target * power_mod(generator_inverse, exponent, RIME_PRIME)
            % RIME_PRIME;
        let digit_target = power_mod(corrected, ORDER / size, RIME_PRIME);
        let digit = baby_step_giant_step(gamma, digit_target, prime)?;
        exponent = exponent.checked_add(digit.checked_mul(place)?)?;
        place = size;
    }
    if power_mod(
        power_mod(RIME_GENERATOR, ORDER / size, RIME_PRIME),
        exponent, RIME_PRIME,
    ) != power_mod(target, ORDER / size, RIME_PRIME) {
        return None;
    }
    Some((exponent, size))
}

/// Small exact Euclidean inverse used only for the pairwise-coprime
/// fixed prime-power CRT moduli. No brute force over the full sphere.
fn invert_mod(value: u64, modulus: u64) -> Option<u64> {
    let (mut t, mut next_t) = (0_i128, 1_i128);
    let (mut r, mut next_r) = (i128::from(modulus), i128::from(value % modulus));
    while next_r != 0 {
        let quotient = r / next_r;
        (t, next_t) = (next_t, t - quotient * next_t);
        (r, next_r) = (next_r, r - quotient * next_r);
    }
    (r == 1).then_some(t.rem_euclid(i128::from(modulus)) as u64)
}

/// Recover the unique canonical exponent k in [0,1_000_080) from
/// h=g^k mod 1_000_081, using original 16/27/5/463 prime towers.
pub fn recover_exponent(target: u64) -> Option<u64> {
    if target == 0 || target >= RIME_PRIME {
        return None;
    }
    // Original generator must have full multiplicative order.
    if power_mod(RIME_GENERATOR, ORDER, RIME_PRIME) != 1
        || TOWERS.iter().any(|(q, _)| {
            power_mod(RIME_GENERATOR, ORDER / q, RIME_PRIME) == 1
        })
    {
        return None;
    }
    let mut reconstructed = 0_u64;
    let mut product = 1_u64;
    for (prime, power) in TOWERS {
        let (residue, modulus) = digit_cascade(target, prime, power)?;
        let partial = ORDER / modulus;
        let inverse = invert_mod(partial, modulus)?;
        // Intermediate product fits u64 for these pinned constants.
        reconstructed = (reconstructed + residue * partial * inverse) % ORDER;
        product = product.checked_mul(modulus)?;
    }
    if product != ORDER
        || power_mod(RIME_GENERATOR, reconstructed, RIME_PRIME) != target
    {
        return None;
    }
    Some(reconstructed)
}

pub fn reverse_fischer_address(address: u64) -> Option<(u64, u64)> {
    let exponent = recover_exponent(address)?;
    Some((exponent % RIME_GLYPHS, exponent / RIME_GLYPHS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::freeze::fischer_address;

    #[test]
    fn prime_tower_crt_factorization_and_full_order() {
        let product = TOWERS.iter().fold(1_u64, |n, (q,e)| n * q.pow(*e));
        assert_eq!(product, ORDER);
        assert_eq!(ORDER, 16 * 27 * 5 * 463);
        assert_eq!(power_mod(RIME_GENERATOR, ORDER, RIME_PRIME), 1);
        assert_eq!(recover_exponent(1), Some(0));
        assert_eq!(recover_exponent(0), None);
        assert_eq!(recover_exponent(RIME_PRIME), None);
    }
    #[test]
    fn full_sphere_prime_digit_cascade_endpoints() {
        for exponent in [
            0_u64, 1, 7, 27, 81, 123_456, 238_602, 500_001,
            711_435, 827_781, 969_278, 994_185, ORDER - 1,
        ] {
            let target = power_mod(RIME_GENERATOR, exponent, RIME_PRIME);
            assert_eq!(recover_exponent(target), Some(exponent));
        }
    }
    #[test]
    fn original_null_outward_all_nested_rooms() {
        for rooms in [3_u64, 9, 27, 81] {
            for room in 0..rooms {
                for glyph in 0..RIME_GLYPHS {
                    let address = fischer_address(glyph, room).expect("valid address");
                    assert_eq!(reverse_fischer_address(address), Some((glyph, room)));
                }
            }
        }
    }
}
