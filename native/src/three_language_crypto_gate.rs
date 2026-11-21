//! Three-language controlled cryptanalysis gate. All observations are public
//! laboratory fixtures; no unknown real-world secrets are targeted.
//! This module explicitly measures when reversible representations add zero
//! independent information to the originating observation.
#![forbid(unsafe_code)]

use crate::{omega_operator_catalog as omega, relic};

type Trit3 = [u8; 3];

const VIEWS: [u8; 3] = [0, 2, 7];
const AFFINE_TRAIN: [u8; 4] = [0, 9, 3, 1];
const TOY_PLAINTEXTS: [u8; 5] = [0, 1, 41, 77, 172];
const TOY_SBOX: [u8; 16] = [6, 4, 12, 5, 0, 7, 2, 14, 1, 15, 3, 13, 8, 10, 9, 11];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Observation {
    view: u8,
    input: u8,
    output: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AffineRule {
    offset: Trit3,
    columns: [Trit3; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GateReport {
    pub affine_training_states: usize,
    pub affine_derived_observations: usize,
    pub affine_withheld_forecasts: usize,
    pub opaque_unknown_symbols: usize,
    pub opaque_consistent_completions: usize,
    pub toy_keyspace: usize,
    pub toy_matching_keys: usize,
    pub aes_keyspace: usize,
    pub aes_matching_keys: usize,
    pub aes_direct_encryptions: usize,
    pub aes_three_view_encryptions: usize,
    pub information_gain_from_derived_views: usize,
}

fn digits(n: u8) -> Trit3 {
    [n / 9, n / 3 % 3, n % 3]
}

fn ordinal([a, b, c]: Trit3) -> u8 {
    a * 9 + b * 3 + c
}

/// Hidden structured language: an invertible affine map on F_3^3.
fn hidden_affine(x: Trit3) -> Trit3 {
    let [a, b, c] = x;
    [(a + 2 * b + c + 1) % 3, (b + 2 * c + 2) % 3, c]
}

fn omega_encode(view: u8, value: u8) -> Result<u8, String> {
    let transformed = omega::apply_operator(view, &[value])
        .map_err(|error| error.to_string())?;
    transformed.first().copied().ok_or("missing encoded byte".to_owned())
}

fn omega_decode(view: u8, value: u8) -> Result<u8, String> {
    let original = omega::invert_operator(view, &[value])
        .map_err(|error| error.to_string())?;
    original.first().copied().ok_or("missing decoded byte".to_owned())
}

fn build_affine_observations() -> Result<Vec<Observation>, String> {
    let mut observations = Vec::with_capacity(AFFINE_TRAIN.len() * VIEWS.len());
    for source in AFFINE_TRAIN {
        let target = ordinal(hidden_affine(digits(source)));
        for view in VIEWS {
            observations.push(Observation {
                view,
                input: omega_encode(view, source)?,
                output: omega_encode(view, target)?,
            });
        }
    }
    Ok(observations)
}

fn rule_from_observations(observations: &[Observation]) -> Result<AffineRule, String> {
    let mut observed = [None::<u8>; 27];
    for item in observations {
        let input = omega_decode(item.view, item.input)?;
        let output = omega_decode(item.view, item.output)?;
        if input >= 27 || output >= 27 {
            return Err("observed source or target outside 27-state alphabet".to_owned());
        }
        let slot = &mut observed[usize::from(input)];
        if slot.is_some() && *slot != Some(output) {
            return Err("three-language correspondence contradiction".to_owned());
        }
        *slot = Some(output);
    }
    let base = digits(observed[0].ok_or("zero origin absent")?);
    let mut columns = [[0_u8; 3]; 3];
    for (index, source) in AFFINE_TRAIN[1..].iter().copied().enumerate() {
        let image = digits(observed[usize::from(source)]
            .ok_or("affine basis vector absent")?);
        for (row, coefficient) in columns[index].iter_mut().enumerate() {
            *coefficient = (image[row] + 3 - base[row]) % 3;
        }
    }
    let rule = AffineRule { offset: base, columns };
    for (source, image) in observed.iter().copied().enumerate() {
        if let Some(image) = image {
            let x = digits(u8::try_from(source).map_err(|_| "source ordinal overflow")?);
            if ordinal(rule.evaluate(x)) != image {
                return Err("observed affine transition not reproduced".to_owned());
            }
        }
    }
    Ok(rule)
}

impl AffineRule {
    fn evaluate(self, x: Trit3) -> Trit3 {
        let mut output = self.offset;
        for (column, coordinate) in self.columns.iter().zip(x) {
            for (row, &weight) in column.iter().enumerate() {
                output[row] = (output[row] + weight * coordinate) % 3;
            }
        }
        output
    }
}

/// All 6! completions of the six unobserved images are consistent with 21
/// known pairs of an otherwise unrestricted 27-element substitution table.
fn opaque_completion_count() -> Result<usize, String> {
    let mut table = std::array::from_fn::<_, 27, _>(|index| ((7 * index + 5) % 27) as u8);
    table.swap(2, 22);
    table.swap(9, 12);
    let mut observed = [None::<u8>; 27];
    for source in 0..21_u8 {
        for view in VIEWS {
            let input = omega_encode(view, source)?;
            let output = omega_encode(view, table[usize::from(source)])?;
            let original = omega_decode(view, input)?;
            let reconstructed = omega_decode(view, output)?;
            if observed[usize::from(original)].is_some()
                && observed[usize::from(original)] != Some(reconstructed)
            {
                return Err("opaque alphabet translation contradicted".to_owned());
            }
            observed[usize::from(original)] = Some(reconstructed);
        }
    }
    let unknown = observed.iter().filter(|value| value.is_none()).count();
    if unknown != 6 {
        return Err("opaque source training changed".to_owned());
    }
    let distinct_images = observed.iter().filter_map(|value| *value)
        .collect::<std::collections::BTreeSet<_>>();
    if distinct_images.len() != 21 {
        return Err("opaque substitution is not bijective".to_owned());
    }
    Ok((1..=unknown).product())
}

/// Six-round 8-bit Feistel toy. A bounded key-search control, not secure
/// encryption and not an attack on deployed encryption.
fn toy_encrypt(plaintext: u8, key: u8) -> u8 {
    let mut left = plaintext >> 4;
    let mut right = plaintext & 15;
    for round in 0..6_u8 {
        let half_key = if round % 2 == 0 { key & 15 } else { key >> 4 };
        let round_key = (half_key + 3 * round) & 15;
        let f = TOY_SBOX[usize::from(right ^ round_key)];
        (left, right) = (right, left ^ f);
    }
    left << 4 | right
}

fn toy_search(expected: &[u8; 5], views: &[u8]) -> Result<Vec<u8>, String> {
    let mut candidates = Vec::new();
    for candidate_key in 0..=u8::MAX {
        let mut match_all = true;
        for (&plaintext, &target) in TOY_PLAINTEXTS.iter().zip(expected.iter()) {
            let actual = toy_encrypt(plaintext, candidate_key);
            for &view in views {
                let transformed = omega_encode(view, actual)?;
                let wanted = omega_encode(view, target)?;
                if transformed != wanted {
                    match_all = false;
                }
            }
        }
        if match_all {
            candidates.push(candidate_key);
        }
    }
    Ok(candidates)
}

// Minimal independently testable AES-128 encryption for standard published
// known-answer vectors. Table-free finite-field S-box. Diagnostic only; this
// code has no production side-channel-hardening or secret-input guarantees.
fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut result = 0_u8;
    for _ in 0..8 {
        if b & 1 != 0 {
            result ^= a;
        }
        a = if a & 128 == 0 { a << 1 } else { (a << 1) ^ 0x1b };
        b >>= 1;
    }
    result
}

fn gf_pow(mut base: u8, mut exponent: u8) -> u8 {
    let mut result = 1_u8;
    while exponent != 0 {
        if exponent & 1 != 0 {
            result = gf_mul(result, base);
        }
        base = gf_mul(base, base);
        exponent >>= 1;
    }
    result
}

fn aes_sbox(input: u8) -> u8 {
    let inverse = if input == 0 { 0 } else { gf_pow(input, 254) };
    inverse ^ inverse.rotate_left(1) ^ inverse.rotate_left(2)
        ^ inverse.rotate_left(3) ^ inverse.rotate_left(4) ^ 0x63
}

fn aes_schedule(key: [u8; 16]) -> [u8; 176] {
    let mut expanded = [0_u8; 176];
    expanded[..16].copy_from_slice(&key);
    let mut written = 16;
    let mut round_constant = 1_u8;
    while written < expanded.len() {
        let mut previous = [
            expanded[written - 4],
            expanded[written - 3],
            expanded[written - 2],
            expanded[written - 1],
        ];
        if written % 16 == 0 {
            previous.rotate_left(1);
            for byte in &mut previous {
                *byte = aes_sbox(*byte);
            }
            previous[0] ^= round_constant;
            round_constant = gf_mul(round_constant, 2);
        }
        for value in previous {
            expanded[written] = expanded[written - 16] ^ value;
            written += 1;
        }
    }
    expanded
}

fn aes_sub_shift(state: &mut [u8; 16]) {
    for byte in state.iter_mut() {
        *byte = aes_sbox(*byte);
    }
    let old = *state;
    for row in 1..4 {
        for column in 0..4 {
            state[4 * column + row] = old[4 * ((column + row) % 4) + row];
        }
    }
}

fn aes_mix(state: &mut [u8; 16]) {
    for column in 0..4 {
        let start = 4 * column;
        let [a, b, c, d] = [
            state[start], state[start + 1], state[start + 2], state[start + 3],
        ];
        state[start] = gf_mul(a, 2) ^ gf_mul(b, 3) ^ c ^ d;
        state[start + 1] = a ^ gf_mul(b, 2) ^ gf_mul(c, 3) ^ d;
        state[start + 2] = a ^ b ^ gf_mul(c, 2) ^ gf_mul(d, 3);
        state[start + 3] = gf_mul(a, 3) ^ b ^ c ^ gf_mul(d, 2);
    }
}

fn aes_add_round(state: &mut [u8; 16], schedule: &[u8; 176], round: usize) {
    for (byte, key) in state.iter_mut().zip(&schedule[16 * round..16 * round + 16]) {
        *byte ^= key;
    }
}

fn aes128_encrypt(block: [u8; 16], key: [u8; 16]) -> [u8; 16] {
    let schedule = aes_schedule(key);
    let mut state = block;
    aes_add_round(&mut state, &schedule, 0);
    for round in 1..=10 {
        aes_sub_shift(&mut state);
        if round < 10 {
            aes_mix(&mut state);
        }
        aes_add_round(&mut state, &schedule, round);
    }
    state
}

const AES_PLAIN: [u8; 16] = [
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
    0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
];
const AES_FIPS_CIPHER: [u8; 16] = [
    0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30,
    0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4, 0xc5, 0x5a,
];

/// Full exhaustive evaluation of a deliberately restricted 8-bit suffix.
/// Candidate keys are inferred from the ciphertext; the true suffix is not
/// passed to the search. This deliberately does not test a 128-bit key search.
fn aes_suffix_search(
    prefix: [u8; 15],
    plaintext: [u8; 16],
    ciphertext: [u8; 16],
    views: &[u8],
) -> Result<(Vec<u8>, usize), String> {
    let mut matching = Vec::new();
    let mut encryptions = 0;
    for suffix in 0..=u8::MAX {
        let mut candidate_key = [0_u8; 16];
        candidate_key[..15].copy_from_slice(&prefix);
        candidate_key[15] = suffix;
        let attempt = aes128_encrypt(plaintext, candidate_key);
        encryptions += 1;
        let mut accepted = true;
        for &view in views {
            let candidate = omega::apply_operator(view, &attempt)
                .map_err(|error| error.to_string())?;
            let target = omega::apply_operator(view, &ciphertext)
                .map_err(|error| error.to_string())?;
            let restored = omega::invert_operator(view, &candidate)
                .map_err(|error| error.to_string())?;
            if candidate != target || restored != attempt {
                accepted = false;
            }
        }
        if accepted {
            matching.push(suffix);
        }
    }
    Ok((matching, encryptions))
}

/// Three-language signed differences retain contributions which cancel in
/// their common mode; the witness does not create fresh ciphertext entropy.
fn verify_brown_witness(value: u8) -> bool {
    let x = i64::from(value);
    let frozen = relic::freeze_triple([x, -x, 0]);
    frozen.sum == 0
        && relic::rebuild_triple(frozen) == Some([x, -x, 0])
}

/// End-to-end controlled gate. Results are exact finite counts; no throughput
/// or security-strength conclusion is inferred from them.
pub fn run_gate() -> Result<GateReport, String> {
    let observations = build_affine_observations()?;
    let one_view = observations.iter().copied()
        .filter(|item| item.view == 0).collect::<Vec<_>>();
    let one = rule_from_observations(&one_view)?;
    let three = rule_from_observations(&observations)?;
    if one != three {
        return Err("three translations introduced independent model coefficients".to_owned());
    }
    let mut withheld = 0;
    for n in 0..27_u8 {
        if AFFINE_TRAIN.contains(&n) {
            continue;
        }
        let predicted = ordinal(three.evaluate(digits(n)));
        let expected = ordinal(hidden_affine(digits(n)));
        if predicted != expected {
            return Err("unknown affine alphabet held-out prediction failed".to_owned());
        }
        for view in VIEWS {
            let encoded = omega_encode(view, predicted)?;
            if omega_decode(view, encoded)? != expected {
                return Err("held-out language transport failed".to_owned());
            }
        }
        withheld += 1;
    }
    if withheld != 23 {
        return Err("affine held-out census changed".to_owned());
    }
    let opaque = opaque_completion_count()?;
    if opaque != 720 {
        return Err("opaque alphabet completion ambiguity changed".to_owned());
    }
    let toy_key = 0x5a_u8; // Generator only; never supplied to toy_search.
    let expected = TOY_PLAINTEXTS.map(|plaintext| toy_encrypt(plaintext, toy_key));
    let toy_direct = toy_search(&expected, &[0])?;
    let toy_three = toy_search(&expected, &VIEWS)?;
    if toy_direct != toy_three || toy_direct != [toy_key] {
        return Err("toy key inference disagreed with direct baseline".to_owned());
    }
    let key = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];
    if aes128_encrypt(AES_PLAIN, key) != AES_FIPS_CIPHER {
        return Err("public FIPS-197 AES-128 vector mismatch".to_owned());
    }
    let mut prefix = [0_u8; 15];
    prefix.copy_from_slice(&key[..15]);
    let (aes_direct, direct_calls) =
        aes_suffix_search(prefix, AES_PLAIN, AES_FIPS_CIPHER, &[0])?;
    let (aes_three, color_calls) =
        aes_suffix_search(prefix, AES_PLAIN, AES_FIPS_CIPHER, &VIEWS)?;
    if aes_direct != aes_three || aes_direct != [0x0f]
        || direct_calls != 256 || color_calls != 256
    {
        return Err("AES key inference differed from equal-budget baseline".to_owned());
    }
    for byte in 0..=u8::MAX {
        if !verify_brown_witness(byte) {
            return Err("signed Brown witness lost original byte".to_owned());
        }
    }
    Ok(GateReport {
        affine_training_states: 4,
        affine_derived_observations: observations.len(),
        affine_withheld_forecasts: withheld,
        opaque_unknown_symbols: 6,
        opaque_consistent_completions: opaque,
        toy_keyspace: 256,
        toy_matching_keys: toy_direct.len(),
        aes_keyspace: 256,
        aes_matching_keys: aes_direct.len(),
        aes_direct_encryptions: direct_calls,
        aes_three_view_encryptions: color_calls,
        information_gain_from_derived_views: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_languages_and_all_controls_have_exact_census() {
        assert_eq!(
            run_gate().expect("three-language gate"),
            GateReport {
                affine_training_states: 4,
                affine_derived_observations: 12,
                affine_withheld_forecasts: 23,
                opaque_unknown_symbols: 6,
                opaque_consistent_completions: 720,
                toy_keyspace: 256,
                toy_matching_keys: 1,
                aes_keyspace: 256,
                aes_matching_keys: 1,
                aes_direct_encryptions: 256,
                aes_three_view_encryptions: 256,
                information_gain_from_derived_views: 0,
            },
        );
    }

    #[test]
    fn affine_observation_substitution_is_rejected() {
        let mut observations = build_affine_observations().expect("fixture");
        let item = &mut observations[1];
        let canonical = omega_decode(item.view, item.output).expect("decode");
        item.output = omega_encode(item.view, (canonical + 1) % 27).expect("encode");
        assert!(rule_from_observations(&observations).is_err());
    }

    #[test]
    fn unfinished_alphabet_is_not_uniquely_decodable() {
        assert_eq!(opaque_completion_count().unwrap(), 720);
    }

    #[test]
    fn aes_128_matches_two_independent_public_known_answer_vectors() {
        let key = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ];
        assert_eq!(aes128_encrypt(AES_PLAIN, key), AES_FIPS_CIPHER);
        assert_eq!(
            aes128_encrypt([0; 16], [0; 16]),
            [
                0x66, 0xe9, 0x4b, 0xd4, 0xef, 0x8a, 0x2c, 0x3b,
                0x88, 0x4c, 0xfa, 0x59, 0xca, 0x34, 0x2b, 0x2e,
            ]
        );
        assert_eq!(aes_sbox(0), 0x63);
        assert_eq!(aes_sbox(0x53), 0xed);
    }

    #[test]
    fn modified_ciphertext_and_wrong_affine_family_are_rejected() {
        let mut target = AES_FIPS_CIPHER;
        target[0] ^= 1;
        let prefix: [u8; 15] = std::array::from_fn(|i| i as u8);
        let (matches, calls) = aes_suffix_search(prefix, AES_PLAIN, target, &VIEWS)
            .expect("deterministic altered cipher");
        assert!(matches.is_empty());
        assert_eq!(calls, 256);

        let mut observations = build_affine_observations().unwrap();
        // Change only the view-zero observation; others retain the true
        // canonical pair and detect the independent false reading.
        observations[0].output ^= 1;
        assert!(rule_from_observations(&observations).is_err());
    }
}
