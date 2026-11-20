//! Genuine streaming original-source Bitcoin RimeSphere.
//!
//! 729 source bytes per gulp -> six exact balanced trits per byte,
//! 27-cell rooms, two groups of 81 rooms at a complete slice.
//! A source/offset/slice key disambiguates the finite Fischer address cycle.
//! The stream is append-only: a readback reconstructs the exact source bytes.
//! Neither a byte projection nor reversible play is a SAT solver or a
//! cryptographic preimage search.
#![forbid(unsafe_code)]

use std::io::{Read, Write};

use crate::brown_ledger::BrownSlice;
use crate::fischer_prime_tower::reverse_fischer_address;
use crate::freeze::fischer_address;
use crate::matrix::CELLS;
use crate::sha256::hash_bytes;

pub const BYTES_PER_SLICE: usize = 729;
pub const DIGITS_PER_BYTE: usize = 6;
pub const ROOMS_PER_SPHERE: usize = 81;
pub const MAX_ROOMS_PER_SLICE: usize = 162;
pub const MAGIC: &[u8; 8] = b"PNPRIME1";
pub const HEADER_BYTES: usize = 160;
const ZERO_ROOT_LABEL: &[u8] = b"PNP_BITCOIN_RIMESPHERE_STREAM_V1";
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SphereStats {
    pub slices: u64,
    pub bytes: u64,
    pub trits: u64,
    pub rooms: u64,
    pub fischer_reads: u64,
    pub stored_bytes: u64,
    pub root: String,
}

pub fn initial_root() -> String {
    hash_bytes(ZERO_ROOT_LABEL)
}

fn failure(detail: &str) -> String {
    detail.to_owned()
}

fn put_u16(out: &mut Vec<u8>, number: u16) {
    out.extend_from_slice(&number.to_le_bytes());
}
fn put_u32(out: &mut Vec<u8>, number: u32) {
    out.extend_from_slice(&number.to_le_bytes());
}
fn put_u64(out: &mut Vec<u8>, number: u64) {
    out.extend_from_slice(&number.to_le_bytes());
}

pub fn byte_trits(byte: u8) -> [u8; DIGITS_PER_BYTE] {
    let mut n = u16::from(byte);
    let mut digits = [0_u8; DIGITS_PER_BYTE];
    for digit in digits.iter_mut().rev() {
        *digit = match n % 3 {
            0 => 255, // -1
            1 => 0,   //  0
            _ => 1,   // +1
        };
        n /= 3;
    }
    digits
}

pub fn trits_byte(digits: &[u8]) -> Result<u8> {
    if digits.len() != DIGITS_PER_BYTE {
        return Err(failure("a source byte requires exactly six ternary digits"));
    }
    let mut value = 0_u16;
    for &digit in digits {
        let trit = match digit {
            255 => 0_u16,
            0 => 1,
            1 => 2,
            _ => return Err(failure("source sphere contains invalid ternary digit")),
        };
        value = value.checked_mul(3).and_then(|n| n.checked_add(trit))
            .ok_or("ternary digit width overflow")?;
    }
    u8::try_from(value).map_err(|_| failure("noncanonical six-trit source byte"))
}

/// Exact local RIME coordinate. A finite Fischer modulus necessarily
/// repeats across arbitrary-size source files: the slice ID belongs to
/// the global address, and local room/cell are inverted within each slice.
pub fn addressed_trit(
    slice: u64,
    room: usize,
    cell: usize,
    trits: &[u8],
) -> Result<(u64, u64, u8)> {
    if room >= MAX_ROOMS_PER_SLICE || cell >= CELLS
        || match room.checked_mul(CELLS).and_then(|n| n.checked_add(cell)) {
            Some(index) => index >= trits.len(),
            None => true,
        }
    {
        return Err(failure("Fischer address exceeds actual RimeSphere slice"));
    }
    let local = fischer_address(cell as u64, room as u64)
        .ok_or("local Fischer address unavailable")?;
    if reverse_fischer_address(local) != Some((cell as u64, room as u64)) {
        return Err(failure("source-bound Fischer forward/reverse address differs"));
    }
    Ok((slice, local, trits[room * CELLS + cell]))
}

fn digest_root(
    previous: &str,
    slice: u64,
    offset: u64,
    len: usize,
    rooms: usize,
    source_sha: &str,
    plane_sha: &str,
) -> String {
    hash_bytes(format!(
        "PNP_RIME_STREAM_STEP_V1|{previous}|{slice}|{offset}|{len}|{rooms}|{source_sha}|{plane_sha}"
    ).as_bytes())
}

fn encode(source: &[u8]) -> Vec<u8> {
    let count = source.len() * DIGITS_PER_BYTE;
    let mut output = Vec::with_capacity(count + CELLS);
    for &byte in source {
        output.extend_from_slice(&byte_trits(byte));
    }
    output.resize(count.div_ceil(CELLS) * CELLS, 0);
    output
}

fn verify_selected_rooms(slice: u64, plane: &[u8]) -> Result<u64> {
    let rooms = plane.len() / CELLS;
    let mut visits = 0_u64;
    for room in [0, 1, 26, 27, 80, 81, 161] {
        if room >= rooms {
            continue;
        }
        let first = room * CELLS;
        let mut values = [0_i64; CELLS];
        for (index, value) in values.iter_mut().enumerate() {
            *value = match plane[first + index] {
                255 => -1,
                0 => 0,
                1 => 1,
                _ => return Err(failure("invalid balanced ternary state")),
            };
        }
        let frozen = BrownSlice::freeze(&values);
        let played = frozen.play().map_err(str::to_owned)?;
        if played != values {
            return Err(failure("frozen 27-cell Brown Ledger frame failed playback"));
        }
        for cell in [0, 13, 26] {
            let (_, _, state) = addressed_trit(slice, room, cell, plane)?;
            if state != plane[first + cell] {
                return Err(failure("Fischer-addressed slice cell changed during replay"));
            }
            visits += 1;
        }
    }
    Ok(visits)
}

fn append_slice<W: Write, I: Write>(
    data: &[u8],
    index: u64,
    offset: u64,
    root: &str,
    destination: &mut W,
    hbi: &mut I,
) -> Result<(String, usize, usize, u64)> {
    if data.is_empty() || data.len() > BYTES_PER_SLICE {
        return Err(failure("invalid RimeSphere source gulp"));
    }
    let plane = encode(data);
    let rooms = plane.len() / CELLS;
    if rooms == 0 || rooms > MAX_ROOMS_PER_SLICE {
        return Err(failure("invalid 27-cell RimeSphere size"));
    }
    let source_sha = hash_bytes(data);
    let plane_sha = hash_bytes(&plane);
    let next_root = digest_root(root, index, offset, data.len(), rooms,
                                &source_sha, &plane_sha);
    let visits = verify_selected_rooms(index, &plane)?;
    let mut header = Vec::with_capacity(HEADER_BYTES);
    header.extend_from_slice(MAGIC);
    put_u64(&mut header, index);
    put_u64(&mut header, offset);
    put_u16(&mut header, u16::try_from(data.len()).map_err(|_| "gulp length")?);
    put_u16(&mut header, u16::try_from(rooms).map_err(|_| "room length")?);
    put_u32(&mut header, u32::try_from(plane.len()).map_err(|_| "plane length")?);
    header.extend_from_slice(source_sha.as_bytes());
    header.extend_from_slice(next_root.as_bytes());
    if header.len() != HEADER_BYTES {
        return Err(failure("RimeSphere slice header length differs"));
    }
    destination.write_all(&header).map_err(|e| format!("sphere header write: {e}"))?;
    destination.write_all(&plane).map_err(|e| format!("sphere plane write: {e}"))?;
    writeln!(hbi,
        "RIME_SLICE|index={index}|source_offset={offset}|source_bytes={}|rooms={rooms}|sphere_groups={}|source_sha256={source_sha}|plane_sha256={plane_sha}|root={next_root}|json=0",
        data.len(), rooms.div_ceil(ROOMS_PER_SPHERE)
    ).map_err(|e| format!("RimeSphere index write: {e}"))?;
    Ok((next_root, rooms, HEADER_BYTES + plane.len(), visits))
}

/// Read only one 729-byte chunk into RAM at a time. The output is a
/// self-addressed append-only RimeSphere with one index/receipt row per gulp.
pub fn stream<R: Read, W: Write, I: Write>(
    source: &mut R,
    sphere: &mut W,
    index: &mut I,
) -> Result<SphereStats> {
    let mut stats = SphereStats {
        slices: 0, bytes: 0, trits: 0, rooms: 0,
        fischer_reads: 0, stored_bytes: 0, root: initial_root(),
    };
    let mut source_buffer = [0_u8; BYTES_PER_SLICE];
    loop {
        let mut used = 0_usize;
        while used < source_buffer.len() {
            match source.read(&mut source_buffer[used..]) {
                Ok(0) => break,
                Ok(count) => used += count,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(format!("original source pump failed: {e}")),
            }
        }
        if used == 0 {
            break;
        }
        let (root, rooms, stored, visits) = append_slice(
            &source_buffer[..used], stats.slices, stats.bytes,
            &stats.root, sphere, index,
        )?;
        stats.root = root;
        stats.slices = stats.slices.checked_add(1).ok_or("slice count overflow")?;
        stats.bytes = stats.bytes.checked_add(used as u64)
            .ok_or("source byte count overflow")?;
        stats.trits = stats.trits.checked_add((used * DIGITS_PER_BYTE) as u64)
            .ok_or("source ternary count overflow")?;
        stats.rooms = stats.rooms.checked_add(rooms as u64)
            .ok_or("RimeSphere room count overflow")?;
        stats.fischer_reads = stats.fischer_reads.checked_add(visits)
            .ok_or("Fischer visit count overflow")?;
        stats.stored_bytes = stats.stored_bytes.checked_add(stored as u64)
            .ok_or("RimeSphere storage overflow")?;
    }
    if stats.slices == 0 {
        return Err(failure("original Bitcoin stream contains zero bytes"));
    }
    Ok(stats)
}

fn read_exact<R: Read>(source: &mut R, size: usize) -> Result<Vec<u8>> {
    let mut result = vec![0_u8; size];
    source.read_exact(&mut result).map_err(|e| format!("RimeSphere readback truncated: {e}"))?;
    Ok(result)
}
fn u16le(buf: &[u8]) -> u16 { u16::from_le_bytes([buf[0], buf[1]]) }
fn u32le(buf: &[u8]) -> u32 { u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) }
fn u64le(buf: &[u8]) -> u64 {
    let mut copy = [0_u8; 8];
    copy.copy_from_slice(buf);
    u64::from_le_bytes(copy)
}

/// Independently consume the frozen stream, verify every complete slice and
/// Fischer sample, then output source bytes in exact original order.
pub fn play<R: Read, W: Write>(
    frozen: &mut R,
    recovered: &mut W,
) -> Result<SphereStats> {
    let mut stats = SphereStats {
        slices: 0, bytes: 0, trits: 0, rooms: 0,
        fischer_reads: 0, stored_bytes: 0, root: initial_root(),
    };
    loop {
        let mut first = [0_u8; 1];
        let got = frozen.read(&mut first).map_err(|e| format!("sphere header read: {e}"))?;
        if got == 0 { break; }
        let mut header = Vec::with_capacity(HEADER_BYTES);
        header.push(first[0]);
        header.extend_from_slice(&read_exact(frozen, HEADER_BYTES - 1)?);
        if &header[..8] != MAGIC {
            return Err(failure("RimeSphere stream magic changed"));
        }
        let slice = u64le(&header[8..16]);
        let offset = u64le(&header[16..24]);
        let used = usize::from(u16le(&header[24..26]));
        let rooms = usize::from(u16le(&header[26..28]));
        let payload = u32le(&header[28..32]) as usize;
        if slice != stats.slices || offset != stats.bytes
            || used == 0 || used > BYTES_PER_SLICE
            || rooms != (used * DIGITS_PER_BYTE).div_ceil(CELLS)
            || payload != rooms * CELLS
        {
            return Err(failure("RimeSphere source offsets, room count or length differ"));
        }
        let original_sha = std::str::from_utf8(&header[32..96])
            .map_err(|_| "invalid source SHA header")?;
        let root = std::str::from_utf8(&header[96..160])
            .map_err(|_| "invalid rolling SHA header")?;
        let plane = read_exact(frozen, payload)?;
        let mut plain = Vec::with_capacity(used);
        for trits in plane[..used * DIGITS_PER_BYTE].chunks_exact(DIGITS_PER_BYTE) {
            plain.push(trits_byte(trits)?);
        }
        if plane[used * DIGITS_PER_BYTE..].iter().any(|&trit| trit != 0) {
            return Err(failure("nonzero padding in final 27-cell room"));
        }
        if hash_bytes(&plain) != original_sha {
            return Err(failure("RimeSphere source gulp hash differs"));
        }
        let expected = digest_root(&stats.root, slice, offset, used, rooms,
                                   original_sha, &hash_bytes(&plane));
        if expected != root {
            return Err(failure("RimeSphere streaming receipt hash chain differs"));
        }
        // Rebuilding the canonical source projection also rejects modified
        // ternary data that preserve byte width or attempt alternative padding.
        if encode(&plain) != plane {
            return Err(failure("noncanonical source byte projection"));
        }
        let reads = verify_selected_rooms(slice, &plane)?;
        recovered.write_all(&plain)
            .map_err(|e| format!("replayed source write: {e}"))?;
        stats.root = expected;
        stats.slices += 1;
        stats.bytes += used as u64;
        stats.trits += (used * DIGITS_PER_BYTE) as u64;
        stats.rooms += rooms as u64;
        stats.fischer_reads += reads;
        stats.stored_bytes += (HEADER_BYTES + payload) as u64;
    }
    if stats.slices == 0 {
        return Err(failure("frozen RimeSphere is empty"));
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn every_byte_has_exact_six_trit_projection() {
        for value in 0_u16..=255 {
            assert_eq!(trits_byte(&byte_trits(value as u8))
                       .expect("canonical byte"), value as u8);
        }
        assert!(trits_byte(&[1, 1, 1, 1, 1, 1]).is_err());
        assert!(trits_byte(&[0, 0, 0, 0, 0, 2]).is_err());
    }

    #[test]
    fn streamed_multisphere_bit_exact_source_and_fischer_coordinates() {
        let data: Vec<u8> = (0..(BYTES_PER_SLICE * 3 + 5))
            .map(|i| (i % 256) as u8).collect();
        let mut stream_data = Vec::new();
        let mut index = Vec::new();
        let stats = stream(&mut Cursor::new(&data), &mut stream_data, &mut index)
            .expect("stream Bitcoin-like source");
        assert_eq!(stats.slices, 4);
        assert_eq!(stats.bytes as usize, data.len());
        assert_eq!(stats.trits, (data.len() * DIGITS_PER_BYTE) as u64);
        assert!(stats.fischer_reads > 0);
        assert_eq!(index.iter().filter(|&&c| c == b'\n').count(), 4);
        let mut played = Vec::new();
        let verified = play(&mut Cursor::new(&stream_data), &mut played)
            .expect("Fischer sphere playback");
        assert_eq!(stats, verified);
        assert_eq!(data, played);
        let mut tampered = stream_data.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 1;
        assert!(play(&mut Cursor::new(tampered), &mut Vec::new()).is_err());
        let mut cut = stream_data;
        cut.pop();
        assert!(play(&mut Cursor::new(cut), &mut Vec::new()).is_err());
    }

    #[test]
    fn address_scope_and_short_slice_controls() {
        let data = b"p cnf 1 1\n1 0\n";
        let mut out = Vec::new();
        let stats = stream(&mut Cursor::new(data), &mut out, &mut Vec::new())
            .expect("stream");
        assert_eq!(stats.slices, 1);
        let mut recovered = Vec::new();
        assert_eq!(play(&mut Cursor::new(out), &mut recovered)
                   .expect("replay"), stats);
        assert_eq!(&recovered, data);
        let bad = [0_u8; 27];
        assert!(addressed_trit(0, 162, 0, &bad).is_err());
        assert!(addressed_trit(0, 0, 27, &bad).is_err());
    }
}
