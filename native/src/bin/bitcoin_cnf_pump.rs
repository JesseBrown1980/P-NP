//! Exact original Bitcoin genesis SHA256d CNF stream through reversible
//! balanced-ternary 27-cell frames; source-only transport, no SAT shortcut.
//! Rust 1.81, no external dependencies, no floating-point, no unsafe code.
#![forbid(unsafe_code)]

use pnp_native::brown_ledger::{BrownDelta, BrownSlice, Direction};
use pnp_native::fischer_prime_tower::reverse_fischer_address;
use pnp_native::freeze::fischer_address;
use pnp_native::sha256::hash_bytes;
use std::{env, fs, process, time::Instant};

const NONCE32: &str = "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";
const MERKLE288: &str = "601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b";
const WIDTH: usize = 27;
const DATA_DIGITS: usize = 21;
const CHECK_DIGITS: usize = 6;
const CHECK_MODULUS: u64 = 729;
const CHECKPOINT_INTERVAL: usize = 243;
const FISCHER_CYCLIC_POSITIONS: u64 = 37_040;

type Result<T> = std::result::Result<T, String>;

fn ensure(ok: bool, why: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(why.to_owned()) }
}
fn expected(case: &str) -> Result<&'static str> {
    match case {
        "nonce32" => Ok(NONCE32),
        "merkle_nonce288" => Ok(MERKLE288),
        _ => Err("unsupported original Bitcoin source case".to_owned()),
    }
}
fn code(word: u32, position: u64) -> [i64; WIDTH] {
    let mut digits = [0_i64; WIDTH];
    let mut value = u64::from(word);
    for d in digits[..DATA_DIGITS].iter_mut().rev() {
        *d = i64::try_from(value % 3).unwrap_or(0) - 1;
        value /= 3;
    }
    let mut check = (u64::from(word)
        ^ position.wrapping_mul(1_664_525).wrapping_add(1_013_904_223))
        % CHECK_MODULUS;
    for d in digits[DATA_DIGITS..].iter_mut().rev() {
        *d = i64::try_from(check % 3).unwrap_or(0) - 1;
        check /= 3;
    }
    digits
}
fn decode(digits: &[i64; WIDTH], position: u64) -> Result<u32> {
    let mut raw = 0_u64;
    for digit in &digits[..DATA_DIGITS] {
        ensure((-1..=1).contains(digit), "invalid source trit")?;
        raw = raw.checked_mul(3)
            .and_then(|v| v.checked_add(u64::try_from(digit + 1).ok()?))
            .ok_or("source trit reconstruction overflow")?;
    }
    let word = u32::try_from(raw).map_err(|_| "noncanonical source word")?;
    let mut check = 0_u64;
    for digit in &digits[DATA_DIGITS..] {
        ensure((-1..=1).contains(digit), "invalid parity trit")?;
        check = check * 3 + u64::try_from(digit + 1).map_err(|_| "invalid trit")?;
    }
    let expected_check = (u64::from(word)
        ^ position.wrapping_mul(1_664_525).wrapping_add(1_013_904_223))
        % CHECK_MODULUS;
    ensure(check == expected_check, "matrix parity or frame address changed")?;
    Ok(word)
}
struct Receipt {
    frames: usize,
    checkpoints: usize,
    witness_bytes: usize,
    addressed_samples: usize,
    proof_sha: String,
    restored: Vec<u8>,
}
fn pump(raw: &[u8]) -> Result<Receipt> {
    let frames = raw.len().div_ceil(4);
    let mut out = Vec::with_capacity(raw.len());
    let mut proof = b"BTC_CNF_FULL_MATRIX_V1\n".to_vec();
    let mut previous = [0_i64; WIDTH];
    let mut checkpoints = 0_usize;
    let mut witness_bytes = 0_usize;
    let mut addressed_samples = 0_usize;
    for (index, slice) in raw.chunks(4).enumerate() {
        let mut four = [0_u8; 4];
        four[..slice.len()].copy_from_slice(slice);
        let frame = code(u32::from_le_bytes(four), index as u64);
        let restored = decode(&frame, index as u64)?.to_le_bytes();
        ensure(restored[..slice.len()] == *slice, "original CNF frame inverse mismatch")?;
        ensure(restored[slice.len()..].iter().all(|byte| *byte == 0),
               "noncanonical final-frame padding")?;
        out.extend_from_slice(&restored[..slice.len()]);
        // Each frame is a 27-trit matrix with 21 exact data trits and six
        // deterministic parity trits. Every frame is inverted, not sampled.
        if index % CHECKPOINT_INTERVAL == 0 || index + 1 == frames {
            let frozen = BrownSlice::freeze(&frame);
            let saved = frozen.serialized().to_vec();
            let checked = BrownSlice::load(saved.clone()).map_err(str::to_owned)?;
            ensure(checked.play().map_err(str::to_owned)? == frame,
                   "Brown Ledger frozen 27-cell witness mismatch")?;
            let delta = BrownDelta::freeze(&previous, &frame);
            let delta_bytes = delta.serialized().to_vec();
            let recovered = BrownDelta::load(delta_bytes.clone(), &previous)
                .map_err(str::to_owned)?;
            ensure(recovered.play(&previous, Direction::Forward).map_err(str::to_owned)? == frame
                   && recovered.play(&frame, Direction::Reverse).map_err(str::to_owned)? == previous,
                   "Brown forward/reverse witness rejected")?;
            // Cyclic room addresses are explicit; they are not unique global
            // source addresses when the source has more than 37,040 frames.
            let room = (index as u64) % FISCHER_CYCLIC_POSITIONS;
            let cell = (index % WIDTH) as u64;
            let address = fischer_address(cell, room).ok_or("invalid Fischer room")?;
            ensure(reverse_fischer_address(address) == Some((cell, room)),
                   "Fischer room inverse mismatch")?;
            proof.extend_from_slice(&(index as u64).to_be_bytes());
            proof.extend_from_slice(&(saved.len() as u64).to_be_bytes());
            proof.extend_from_slice(&saved);
            proof.extend_from_slice(&(delta_bytes.len() as u64).to_be_bytes());
            proof.extend_from_slice(&delta_bytes);
            previous = frame;
            checkpoints += 1;
            addressed_samples += 1;
            witness_bytes += saved.len() + delta_bytes.len() + 24;
        }
    }
    ensure(out == raw, "source CNF byte-perfect reconstruction failed")?;
    Ok(Receipt {
        frames, checkpoints, witness_bytes, addressed_samples,
        proof_sha: hash_bytes(&proof), restored: out,
    })
}
fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let case = args.next().ok_or("missing case")?;
    let input = args.next().ok_or("missing source CNF")?;
    let output = args.next().ok_or("missing reconstructed CNF output")?;
    let receipt = args.next().ok_or("missing HBP output path")?;
    ensure(args.next().is_none(), "unexpected extra arguments")?;
    let begin = Instant::now();
    let raw = fs::read(&input).map_err(|e| format!("read source CNF: {e}"))?;
    ensure(!raw.is_empty(), "source CNF is empty")?;
    let root = hash_bytes(&raw);
    ensure(root == expected(&case)?, "original source CNF SHA256 changed")?;
    let result = pump(&raw)?;
    ensure(hash_bytes(&result.restored) == root, "reconstructed original CNF SHA mismatch")?;
    fs::write(&output, &result.restored).map_err(|e| format!("write inverse CNF: {e}"))?;
    ensure(fs::read(&output).map_err(|e| format!("read inverse CNF: {e}"))? == raw,
           "published inverse CNF is not byte identical")?;
    let elapsed = begin.elapsed().as_nanos();
    let report = format!(
        "BTC_CNF_PUMP|schema=1|case={case}|source_sha256={root}|restored_sha256={root}|\
         source_bytes={}|frames_27_trits={}|full_frames_reversed={}|\
         trits_per_frame=27|source_data_trits=21|parity_trits=6|\
         frozen_brown_checkpoints={}|fischer_inverse_samples={}|\
         fischer_position_scope=CYCLIC_37040|brown_paid_witness_bytes={}|\
         witness_proof_sha256={}|pump_wall_ns={elapsed}|\
         original_cnf_for_solver=BYTE_IDENTICAL|sha256d_search=NOT_RUN|\
         status=PASS|json=0\n",
        raw.len(), result.frames, result.frames, result.checkpoints,
        result.addressed_samples, result.witness_bytes, result.proof_sha,
    );
    fs::write(&receipt, &report).map_err(|e| format!("write receipt: {e}"))?;
    print!("{report}");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("BTC_CNF_PUMP_FAIL|reason={error}|status=FAIL|json=0");
        process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhaustive_byte_symbols_and_representative_word_roundtrip() {
        for byte in u8::MIN..=u8::MAX {
            let word = u32::from_le_bytes([byte, byte ^ 0x55, 0xa5, 0xfe]);
            for position in [0, 1, 242, 243, 37_040, 1_000_081] {
                assert_eq!(decode(&code(word, position), position), Ok(word));
            }
        }
        for raw in [vec![0], vec![255; 79], vec![1, 2, 3, 4, 5, 6, 7],
                    (0..1024).map(|n| n as u8).collect::<Vec<u8>>()] {
            let receipt = pump(&raw).expect("full original byte inverse");
            assert_eq!(receipt.restored, raw);
            assert_eq!(receipt.frames, raw.len().div_ceil(4));
            assert!(receipt.checkpoints > 0);
        }
    }
    #[test]
    fn altered_frame_or_cyclic_address_is_rejected() {
        let base = code(0x1234_abcd, 123);
        let mut damaged = base;
        damaged[23] = if damaged[23] == 1 { 0 } else { 1 };
        assert!(decode(&damaged, 123).is_err());
        assert!(decode(&base, 124).is_err());
        let mut bad = base;
        bad[0] = 2;
        assert!(decode(&bad, 123).is_err());
        let frozen = BrownSlice::freeze(&base);
        let mut bytes = frozen.serialized().to_vec();
        *bytes.last_mut().expect("nonempty witness") ^= 1;
        assert!(BrownSlice::load(bytes).is_err());
    }
}
