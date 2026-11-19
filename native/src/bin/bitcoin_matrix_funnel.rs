//! Real Bitcoin genesis SHA256d CNF -> exact 27-cell ternary matrix funnel.
//! Pure Rust 1.81 integer transport and authenticated round-trip reference.
//! Inversion here restores encoded input bytes; SHA256d preimage search is a
//! separately metered optimization gate. This witness never labels a known
//! reference Bitcoin header as a newly discovered SAT solution.
#![forbid(unsafe_code)]

use pnp_native::brown_ledger::{BrownDelta, BrownSlice, Direction};
use pnp_native::fischer_prime_tower::reverse_fischer_address;
use pnp_native::freeze::fischer_address;
use pnp_native::matrix::CELLS;
use pnp_native::sha256::hash_bytes;
use pnp_native::ternary::Cnf;
use std::{env, fs, path::Path, process, time::Instant};

const HEADER_PREFIX_HEX: &str = concat!(
    "01000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a",
    "29ab5f49",
    "ffff001d"
);
const GENESIS_NONCE: u32 = 2_083_236_893;
const GENESIS_HASH: &str =
    "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
const NONCE32_CNF: &str =
    "6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3";
const MERKLE288_CNF: &str =
    "601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b";
const DIGITS_PER_BYTE: usize = 6;
type Result<T> = std::result::Result<T, String>;

fn ensure(pass: bool, reason: &str) -> Result<()> {
    if pass { Ok(()) } else { Err(reason.to_owned()) }
}

fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
fn hex_bytes(hex: &str) -> Result<Vec<u8>> {
    ensure(hex.len() % 2 == 0, "odd-length hex source")?;
    let mut output = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.as_bytes().chunks_exact(2) {
        let high = nibble(chunk[0]).ok_or("invalid high hex nibble")?;
        let low = nibble(chunk[1]).ok_or("invalid low hex nibble")?;
        output.push(high * 16 + low);
    }
    Ok(output)
}
fn header() -> Result<Vec<u8>> {
    let mut bytes = hex_bytes(HEADER_PREFIX_HEX)?;
    ensure(bytes.len() == 76, "source prefix must be 76 bytes")?;
    bytes.extend_from_slice(&GENESIS_NONCE.to_le_bytes());
    ensure(bytes.len() == 80, "genesis header length changed")?;
    Ok(bytes)
}
fn sha256d_display(bytes: &[u8]) -> Result<String> {
    let inner = hex_bytes(&hash_bytes(bytes))?;
    ensure(inner.len() == 32, "first digest length")?;
    let outer = hex_bytes(&hash_bytes(&inner))?;
    let mut displayed = String::with_capacity(64);
    use std::fmt::Write;
    for byte in outer.iter().rev() {
        write!(&mut displayed, "{byte:02x}").map_err(|_| "hash print failed")?;
    }
    Ok(displayed)
}
fn six_trits(byte: u8) -> [i64; DIGITS_PER_BYTE] {
    let mut number = u16::from(byte);
    let mut digits = [0_i64; DIGITS_PER_BYTE];
    for slot in digits.iter_mut().rev() {
        *slot = i64::from((number % 3) as u8) - 1;
        number /= 3;
    }
    digits
}
fn byte_from_trits(trits: &[i64]) -> Result<u8> {
    ensure(trits.len() == DIGITS_PER_BYTE, "invalid trit group width")?;
    let mut value = 0_u16;
    for &trit in trits {
        ensure((-1..=1).contains(&trit), "trit outside balanced domain")?;
        value = value.checked_mul(3)
            .and_then(|v| v.checked_add(u16::try_from(trit + 1).ok()?))
            .ok_or("trit value overflow")?;
    }
    u8::try_from(value).map_err(|_| "decoded byte outside 0..255".to_owned())
}
fn authentic_source(case: &str, raw: &[u8]) -> Result<(usize, usize, String)> {
    let (expected_sha, vars, clauses) = match case {
        "nonce32" => (NONCE32_CNF, 239_112, 877_879),
        "merkle_nonce288" => (MERKLE288_CNF, 361_188, 1_325_357),
        _ => return Err("unknown original Bitcoin SHA256d source case".to_owned()),
    };
    let sha = hash_bytes(raw);
    ensure(sha == expected_sha, "original SHA256d source CNF fingerprint rejected")?;
    let source = std::str::from_utf8(raw).map_err(|_| "non-UTF8 DIMACS input")?;
    let original = Cnf::parse(source)?;
    ensure(original.variables == vars && original.clauses.len() == clauses,
           "original source DIMACS structure differs from pinned record")?;
    ensure(original.clauses.iter().all(|clause| !clause.is_empty()),
           "source contains unexpected empty clause")?;
    Ok((vars, clauses, sha))
}
fn transport(bytes: &[u8]) -> Result<(usize, usize, String, String, usize)> {
    let mut trits = Vec::with_capacity(bytes.len() * DIGITS_PER_BYTE);
    for &byte in bytes {
        trits.extend_from_slice(&six_trits(byte));
    }
    let data_trits = trits.len();
    let pad = (CELLS - data_trits % CELLS) % CELLS;
    trits.resize(data_trits + pad, 0);
    let mut previous = [0_i64; CELLS];
    let mut proof = Vec::new();
    let mut recorded_slices = Vec::new();
    let mut recorded_deltas = Vec::new();
    let mut recorded_states = Vec::new();
    let mut visits = 0_usize;
    for (frame, cells) in trits.chunks_exact(CELLS).enumerate() {
        let state: [i64; CELLS] = cells.try_into()
            .map_err(|_| "invalid 27-cell source chunk")?;
        let frozen = BrownSlice::freeze(&state);
        let frozen_bytes = frozen.serialized().to_vec();
        let restored = BrownSlice::load(frozen_bytes.clone())
            .map_err(str::to_owned)?.play().map_err(str::to_owned)?;
        ensure(restored == state, "Brown matrix slice failed inverse")?;
        let visited = frozen.play_from_zero(frame).map_err(str::to_owned)?;
        ensure(visited.len() == CELLS, "Fischer schedule omitted a matrix cell")?;
        let mut seen = [false; CELLS];
        for (cell, value, address) in visited {
            ensure(cell < CELLS && !seen[cell] && value == state[cell],
                   "Fischer room or cell witness differs")?;
            seen[cell] = true;
            ensure(fischer_address(cell as u64, frame as u64) == Some(address)
                   && reverse_fischer_address(address) == Some((cell as u64, frame as u64)),
                   "Fischer forward/reverse address does not close")?;
            visits += 1;
        }
        let delta = BrownDelta::freeze(&previous, &state);
        let delta_bytes = delta.serialized().to_vec();
        let original = BrownDelta::load(delta_bytes.clone(), &previous)
            .map_err(str::to_owned)?;
        ensure(original.play(&previous, Direction::Forward).map_err(str::to_owned)? == state
            && original.play(&state, Direction::Reverse).map_err(str::to_owned)? == previous,
            "matrix-delta forward/reverse is not exact")?;
        proof.extend_from_slice(hash_bytes(&frozen_bytes).as_bytes());
        proof.extend_from_slice(hash_bytes(&delta_bytes).as_bytes());
        recorded_slices.push(frozen_bytes);
        recorded_deltas.push(delta_bytes);
        recorded_states.push(state);
        previous = state;
    }
    let mut all_trits = Vec::with_capacity(data_trits + pad);
    for slice in &recorded_slices {
        let restored = BrownSlice::load(slice.clone()).map_err(str::to_owned)?
            .play().map_err(str::to_owned)?;
        all_trits.extend_from_slice(&restored);
    }
    ensure(all_trits[data_trits..].iter().all(|v| *v == 0),
           "matrix padding closure differs")?;
    let decoded = all_trits[..data_trits].chunks_exact(DIGITS_PER_BYTE)
        .map(byte_from_trits).collect::<Result<Vec<u8>>>()?;
    ensure(decoded == bytes, "funnel failed source-byte reconstruction")?;
    let mut backwards = previous;
    for index in (0..recorded_deltas.len()).rev() {
        let before = if index == 0 { [0_i64; CELLS] }
                     else { recorded_states[index - 1] };
        let delta = BrownDelta::load(recorded_deltas[index].clone(), &before)
            .map_err(str::to_owned)?;
        backwards = delta.play(&backwards, Direction::Reverse)
            .map_err(str::to_owned)?;
        ensure(backwards == before, "full matrix reverse traversal failed")?;
    }
    ensure(backwards == [0_i64; CELLS],
           "reverse replay did not return to null matrix")?;
    ensure(recorded_slices.len() == 18 && visits == 18 * CELLS,
           "80-byte source did not reach exactly 18 27-cell rooms")?;
    Ok((data_trits, pad, hash_bytes(&proof), hash_bytes(&decoded),
        recorded_slices.iter().map(Vec::len).sum::<usize>() +
        recorded_deltas.iter().map(Vec::len).sum::<usize>()))
}
fn acceptance(case: &str, raw_cnf: &[u8]) -> Result<String> {
    let (vars, clauses, source_sha) = authentic_source(case, raw_cnf)?;
    let bytes = header()?;
    ensure(sha256d_display(&bytes)? == GENESIS_HASH,
           "Bitcoin genesis double-SHA256 original witness changed")?;
    let (digits, pad, matrix_root, roundtrip, paid_bytes) = transport(&bytes)?;
    let mut tampered = bytes.clone();
    tampered[76] ^= 1;
    ensure(sha256d_display(&tampered)? != GENESIS_HASH,
           "altered nonce erroneously passes reference target")?;
    let mut tampered = bytes.clone();
    tampered[36] ^= 1;
    ensure(sha256d_display(&tampered)? != GENESIS_HASH,
           "altered merkle erroneously passes reference target")?;
    let line = format!(
        "BTC_MATRIX_FUNNEL|schema=1|case={case}|cnf_sha256={source_sha}|\
         cnf_bytes={}|variables={vars}|clauses={clauses}|\
         header_sha256={}|target_sha256d={GENESIS_HASH}|\
         encoding=BALANCED_TRITS_SIX_PER_BYTE|data_trits={digits}|\
         frames=18|cells_per_frame=27|padding_cells={pad}|\
         fischer_inverse=EXACT|brown_freeze_play=EXACT|\
         brown_delta_inverse=EXACT|source_bytes_roundtrip_sha256={roundtrip}|\
         paid_slice_and_delta_bytes={paid_bytes}|\
         negative_header_controls=2|solver_search=NOT_RUN|\
         cryptographic_preimage_discovery=NOT_CLAIMED|status=PASS|json=0\n",
        raw_cnf.len(),
        hash_bytes(&bytes)
    );
    let root = hash_bytes(line.as_bytes());
    Ok(format!("{line}BTC_MATRIX_FINAL|case={case}|receipt_sha256={root}|status=PASS|json=0\n"))
}
fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let action = args.next().ok_or("required action: attest or verify")?;
    let case = args.next().ok_or("missing original Bitcoin case")?;
    let source = args.next().ok_or("missing original CNF path")?;
    let receipt = args.next().ok_or("missing receipt path")?;
    ensure(args.next().is_none(), "unexpected trailing arguments")?;
    let now = Instant::now();
    let cnf = fs::read(&source).map_err(|error| format!("source read error: {error}"))?;
    let expected = acceptance(&case, &cnf)?;
    match action.as_str() {
        "attest" => fs::write(Path::new(&receipt), &expected)
            .map_err(|error| format!("unable to write acceptance receipt: {error}"))?,
        "verify" => {
            let published = fs::read_to_string(&receipt)
                .map_err(|error| format!("receipt read error: {error}"))?;
            ensure(published == expected, "submitted funnel receipt differs from original source")?;
        }
        _ => return Err("unknown action: attest or verify".to_owned()),
    }
    print!("{expected}");
    println!("BTC_MATRIX_METER|case={case}|action={action}|wall_ns={}|\
             work_cnf_bytes={}|search_nodes=0|json=0",
             now.elapsed().as_nanos(), cnf.len());
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("BTC_MATRIX_FAIL|reason={error}|status=FAIL|json=0");
        process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_header_hash_and_balanced_digit_bijection() {
        let genesis = header().expect("real Bitcoin header");
        assert_eq!(genesis.len(), 80);
        assert_eq!(sha256d_display(&genesis).expect("reference"), GENESIS_HASH);
        for byte in u8::MIN..=u8::MAX {
            assert_eq!(byte_from_trits(&six_trits(byte)).expect("decode"), byte);
        }
        let (digits, padding, _root, roundtrip, _paid) =
            transport(&genesis).expect("exact matrix");
        assert_eq!((digits, padding), (480, 6));
        assert_eq!(roundtrip, hash_bytes(&genesis));
    }
    #[test]
    fn invalid_headers_and_noncanonical_trits_rejected() {
        assert!(hex_bytes("q0").is_err());
        assert!(byte_from_trits(&[-1, -1, -1, -1, -1, 2]).is_err());
        assert!(byte_from_trits(&[1, 1, 1, 1, 1, 1]).is_err());
        let mut corrupt = header().expect("original");
        corrupt[36] ^= 1;
        assert_ne!(sha256d_display(&corrupt).expect("real hash"), GENESIS_HASH);
        let s = BrownSlice::freeze(&[0_i64; CELLS]);
        let mut damaged = s.serialized().to_vec();
        let last = damaged.len()-1;
        damaged[last] ^= 1;
        assert!(BrownSlice::load(damaged).is_err());
    }
}
