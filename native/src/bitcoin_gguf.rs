//! Canonical GGUF v3 freezer and Bobby Fischer address playback for genuine
//! Bitcoin CNF bytes and mutable balanced-ternary SAT assignment snapshots.
//! Mirrors the GGUF v3/I8 layout used by the original Mythos Blue Rust reader:
//! full standard tensor header, 32-byte aligned tensor data, no dependencies.
//! The original clauses and their source SHA are retained byte for byte.
#![forbid(unsafe_code)]

use crate::fischer_prime_tower::reverse_fischer_address;
use crate::freeze::fischer_address;
use crate::matrix::CELLS;
use crate::sha256::hash_bytes;
use crate::ternary::Trit;

const ARCH: &str = "pnp-bitcoin-sha256d-ternary-sat";
const NAME: &str = "BITCOIN-FISCHER-FROZEN-CNF";
const SOURCE_TENSOR: &str = "bitcoin.original_dimacs";
const STATE_TENSOR: &str = "bitcoin.sat_trits";
const ALIGN: usize = 32;
const I8: u32 = 24;
type Result<T> = std::result::Result<T, String>;

fn fail(message: &str) -> String { message.to_owned() }

fn u32le(output: &mut Vec<u8>, n: u32) {
    output.extend_from_slice(&n.to_le_bytes());
}
fn u64le(output: &mut Vec<u8>, n: u64) {
    output.extend_from_slice(&n.to_le_bytes());
}
fn gguf_string(output: &mut Vec<u8>, text: &str) -> Result<()> {
    u64le(output, u64::try_from(text.len()).map_err(|_| fail("string too long"))?);
    output.extend_from_slice(text.as_bytes());
    Ok(())
}
fn kv(output: &mut Vec<u8>, key: &str, value: &str) -> Result<()> {
    gguf_string(output, key)?;
    u32le(output, 8); // GGUF_TYPE_STRING
    gguf_string(output, value)
}
fn tensor(output: &mut Vec<u8>, name: &str, len: usize, offset: usize) -> Result<()> {
    gguf_string(output, name)?;
    u32le(output, 1); // one-dimensional tensor
    u64le(output, u64::try_from(len).map_err(|_| fail("tensor length overflow"))?);
    u32le(output, I8); // standard GGML_TYPE_I8
    u64le(output, u64::try_from(offset).map_err(|_| fail("offset overflow"))?);
    Ok(())
}
fn align(output: &mut Vec<u8>) {
    let pad = (ALIGN - output.len() % ALIGN) % ALIGN;
    output.resize(output.len() + pad, 0);
}
fn state_byte(t: Trit) -> u8 {
    match t { Trit::Minus => 255, Trit::Zero => 0, Trit::Plus => 1 }
}
fn parse_trit(n: u8) -> Result<Trit> {
    match n {
        255 => Ok(Trit::Minus),
        0 => Ok(Trit::Zero),
        1 => Ok(Trit::Plus),
        _ => Err(fail("GGUF ternary snapshot contains non-trit byte")),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitcoinFrozen {
    pub source: Vec<u8>,
    pub states: Vec<Trit>,
    pub source_sha256: String,
    pub state_sha256: String,
}

pub fn freeze(source: &[u8], states: &[Trit]) -> Result<Vec<u8>> {
    if source.is_empty() || states.is_empty() {
        return Err(fail("cannot freeze an empty source or assignment"));
    }
    let source_sha = hash_bytes(source);
    let state: Vec<u8> = states.iter().copied().map(state_byte).collect();
    let state_sha = hash_bytes(&state);
    let state_offset = source.len().checked_add((ALIGN - source.len() % ALIGN) % ALIGN)
        .ok_or("frozen source byte count overflow")?;
    let mut out = Vec::new();
    out.extend_from_slice(b"GGUF");
    u32le(&mut out, 3);
    u64le(&mut out, 2); // two real I8 tensors
    u64le(&mut out, 5); // five standard typed string metadata keys
    kv(&mut out, "general.architecture", ARCH)?;
    kv(&mut out, "general.name", NAME)?;
    kv(&mut out, "pnp.original_cnf_sha256", &source_sha)?;
    kv(&mut out, "pnp.sat_trits_sha256", &state_sha)?;
    kv(&mut out, "pnp.playback", "BOBBY_FISCHER_RIME_27_V1")?;
    tensor(&mut out, SOURCE_TENSOR, source.len(), 0)?;
    tensor(&mut out, STATE_TENSOR, state.len(), state_offset)?;
    align(&mut out);
    out.extend_from_slice(source);
    align(&mut out);
    out.extend_from_slice(&state);
    Ok(out)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(count).ok_or("GGUF read overflow")?;
        if end > self.bytes.len() {
            return Err(fail("GGUF truncated"));
        }
        let chunk = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(chunk)
    }
    fn n32(&mut self) -> Result<u32> {
        let b: [u8; 4] = self.take(4)?.try_into().map_err(|_| fail("u32"))?;
        Ok(u32::from_le_bytes(b))
    }
    fn n64(&mut self) -> Result<u64> {
        let b: [u8; 8] = self.take(8)?.try_into().map_err(|_| fail("u64"))?;
        Ok(u64::from_le_bytes(b))
    }
    fn string(&mut self) -> Result<String> {
        let n = usize::try_from(self.n64()?).map_err(|_| fail("GGUF string width"))?;
        String::from_utf8(self.take(n)?.to_vec())
            .map_err(|_| fail("GGUF non-UTF8 metadata"))
    }
}

pub fn thaw(raw: &[u8]) -> Result<BitcoinFrozen> {
    let mut c = Cursor { bytes: raw, offset: 0 };
    if c.take(4)? != b"GGUF" || c.n32()? != 3 || c.n64()? != 2 || c.n64()? != 5 {
        return Err(fail("GGUF v3 header and tensor/metadata count mismatch"));
    }
    let mut found = std::collections::BTreeMap::new();
    for _ in 0..5 {
        let key = c.string()?;
        if c.n32()? != 8 {
            return Err(fail("GGUF metadata type changed"));
        }
        if found.insert(key, c.string()?).is_some() {
            return Err(fail("duplicate GGUF metadata"));
        }
    }
    for (key, expected) in [
        ("general.architecture", ARCH),
        ("general.name", NAME),
        ("pnp.playback", "BOBBY_FISCHER_RIME_27_V1"),
    ] {
        if found.get(key).map(String::as_str) != Some(expected) {
            return Err(fail("GGUF architecture or playback identity changed"));
        }
    }
    let source_sha = found.get("pnp.original_cnf_sha256")
        .ok_or("GGUF original source hash missing")?.clone();
    let state_sha = found.get("pnp.sat_trits_sha256")
        .ok_or("GGUF ternary state hash missing")?.clone();
    if [source_sha.as_str(), state_sha.as_str()].iter().any(|h| {
        h.len() != 64 || !h.bytes().all(|n| n.is_ascii_hexdigit())
    }) {
        return Err(fail("GGUF digest metadata invalid"));
    }
    let mut info = Vec::with_capacity(2);
    for _ in 0..2 {
        let name = c.string()?;
        if c.n32()? != 1 {
            return Err(fail("GGUF tensor rank differs"));
        }
        let len = usize::try_from(c.n64()?).map_err(|_| fail("GGUF tensor length"))?;
        let ggml_type = c.n32()?;
        let offset = usize::try_from(c.n64()?).map_err(|_| fail("GGUF tensor offset"))?;
        if ggml_type != I8 {
            return Err(fail("GGUF tensor must be standard I8"));
        }
        info.push((name, len, offset));
    }
    if info[0].0 != SOURCE_TENSOR || info[1].0 != STATE_TENSOR
        || info[0].2 != 0 || info[0].1 == 0 || info[1].1 == 0
    {
        return Err(fail("GGUF original source/state tensor layout changed"));
    }
    let expected_offset = info[0].1.checked_add(
        (ALIGN - info[0].1 % ALIGN) % ALIGN).ok_or("GGUF source offset overflow")?;
    if info[1].2 != expected_offset {
        return Err(fail("GGUF ternary tensor alignment differs"));
    }
    let header_pad = (ALIGN - c.offset % ALIGN) % ALIGN;
    if c.take(header_pad)?.iter().any(|&b| b != 0) {
        return Err(fail("nonzero GGUF header padding"));
    }
    let source = c.take(info[0].1)?.to_vec();
    if c.take(expected_offset - info[0].1)?.iter().any(|&b| b != 0) {
        return Err(fail("nonzero GGUF source tensor padding"));
    }
    let state = c.take(info[1].1)?.to_vec();
    if c.offset != raw.len() || hash_bytes(&source) != source_sha
        || hash_bytes(&state) != state_sha
    {
        return Err(fail("GGUF source, snapshot or end-of-file authenticity differs"));
    }
    let states = state.into_iter().map(parse_trit).collect::<Result<Vec<_>>>()?;
    let canonical = freeze(&source, &states)?;
    if canonical != raw {
        return Err(fail("GGUF canonical byte-exact readback differs"));
    }
    Ok(BitcoinFrozen { source, states, source_sha256: source_sha,
        state_sha256: state_sha })
}

/// Play a single frozen original SAT variable by its Fischer rime address.
/// The input GGUF remains immutable; a changed assignment makes a new freeze.
pub fn play_address(frozen: &BitcoinFrozen, address: u64) -> Result<(usize, Trit)> {
    let (cell, room) = reverse_fischer_address(address)
        .ok_or("Fischer inverse address missing")?;
    let cell = usize::try_from(cell).map_err(|_| fail("cell range"))?;
    let room = usize::try_from(room).map_err(|_| fail("room range"))?;
    let index = room.checked_mul(CELLS).and_then(|n| n.checked_add(cell))
        .ok_or("Fischer room index overflow")?;
    let state = *frozen.states.get(index)
        .ok_or("Fischer address outside frozen Bitcoin assignment")?;
    if fischer_address(cell as u64, room as u64) != Some(address) {
        return Err(fail("Fischer forward/reverse address disagrees"));
    }
    Ok((index + 1, state))
}

pub fn play_variable(frozen: &BitcoinFrozen, variable: usize) -> Result<(u64, Trit)> {
    if variable == 0 || variable > frozen.states.len() {
        return Err(fail("Fischer variable outside original CNF"));
    }
    let zero = variable - 1;
    let address = fischer_address((zero % CELLS) as u64, (zero / CELLS) as u64)
        .ok_or("Fischer forward address failed")?;
    let (recovered, state) = play_address(frozen, address)?;
    if recovered != variable {
        return Err(fail("Fischer variable replay differed"));
    }
    Ok((address, state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_gguf_v3_headers_and_reversible_fischer_reads() {
        let cnf = b"p cnf 29 2\n1 -2 0\n28 29 0\n";
        let mut states = vec![Trit::Zero; 29];
        states[0] = Trit::Plus;
        states[27] = Trit::Minus;
        let bytes = freeze(cnf, &states).expect("GGUF freeze");
        assert_eq!(&bytes[..4], b"GGUF");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().expect("version")), 3);
        let restored = thaw(&bytes).expect("GGUF thaw");
        assert_eq!(restored.source, cnf);
        assert_eq!(restored.states, states);
        for variable in [1, 2, 27, 28, 29] {
            assert_eq!(play_variable(&restored, variable).expect("Fischer play").1,
                       states[variable - 1]);
        }
        let mut updated = restored.states.clone();
        updated[28] = Trit::Plus;
        let next = freeze(cnf, &updated).expect("second frozen state");
        assert_ne!(next, bytes);
        assert_eq!(thaw(&next).expect("next thaw").states, updated);
    }

    #[test]
    fn reject_header_source_state_and_address_corruption() {
        let source = b"p cnf 3 1\n1 2 3 0\n";
        let data = freeze(source, &[Trit::Minus, Trit::Zero, Trit::Plus])
            .expect("source GGUF");
        let original = thaw(&data).expect("original");
        assert!(play_variable(&original, 0).is_err());
        assert!(play_variable(&original, 4).is_err());
        for i in [0, 4, data.len() / 2, data.len() - 1] {
            let mut corrupted = data.clone();
            corrupted[i] ^= 1;
            assert!(thaw(&corrupted).is_err(), "corrupt index {i}");
        }
        let mut extra = data;
        extra.push(0);
        assert!(thaw(&extra).is_err());
    }
}
