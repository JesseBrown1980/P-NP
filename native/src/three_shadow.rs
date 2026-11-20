//! Three-light, three-shadow exact projection of a signed ternary 3x3x3 room.
//!
//! Each orthogonal 2-D light/shadow plane contains nine **signed line sums**
//! (three independent directions, not three duplicate camera views).
//! The 27 shadow entries have rank 19 over the rationals, leaving eight
//! independent cell values. Eight source trits in the interior 2x2x2 cube
//! are retained as an explicit witness; all 27 cells can then be uniquely
//! reconstructed. Three shadows alone are not an invertible 3-D object.
//! The 35-byte tuple is independently checked and is not compression.
#![forbid(unsafe_code)]

use std::io::{BufRead, Read, Write};

use crate::bitcoin_rimesphere_stream::{
    byte_trits, initial_root as original_root, trits_byte, BYTES_PER_SLICE,
    DIGITS_PER_BYTE, HEADER_BYTES as RIME_HEADER_BYTES, MAGIC as RIME_MAGIC,
};
use crate::matrix::CELLS;
use crate::sha256::hash_bytes;

pub const SHADOW_BYTES: usize = 35;
pub const SHADOWS: usize = 3;
pub const PIXELS: usize = 9;
pub const RESIDUALS: usize = 8;
pub const EXPECTED_RANK: usize = 19;
type Result<T> = std::result::Result<T, String>;

fn cell(x: usize, y: usize, z: usize) -> usize {
    x + 3 * y + 9 * z
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreeShadows {
    /// Projected signed 2-D line sums for the X, Y, and Z light directions.
    pub shadows: [[i8; PIXELS]; SHADOWS],
    /// Exact eight source trits at x,y,z in 0..2. Not optional metadata.
    pub interior: [i8; RESIDUALS],
}

fn check_trit(value: i8) -> Result<i8> {
    if (-1..=1).contains(&value) {
        Ok(value)
    } else {
        Err("three-light source contains invalid ternary cell".to_owned())
    }
}
fn shadow_index(axis: usize, x: usize, y: usize, z: usize) -> usize {
    match axis {
        0 => 3 * y + z,
        1 => 3 * x + z,
        _ => 3 * x + y,
    }
}

impl ThreeShadows {
    pub fn freeze(source: &[i8; CELLS]) -> Result<Self> {
        let mut shadows = [[0_i8; PIXELS]; SHADOWS];
        let mut interior = [0_i8; RESIDUALS];
        for z in 0..3 {
            for y in 0..3 {
                for x in 0..3 {
                    let value = check_trit(source[cell(x, y, z)])?;
                    for (axis, plane) in shadows.iter_mut().enumerate() {
                        let position = shadow_index(axis, x, y, z);
                        plane[position] = plane[position].checked_add(value)
                            .ok_or("three-shadow line sum overflow")?;
                    }
                    if x < 2 && y < 2 && z < 2 {
                        interior[x + 2 * y + 4 * z] = value;
                    }
                }
            }
        }
        Ok(Self { shadows, interior })
    }

    /// Reconstruct every missing outer face, edge and final corner by
    /// increasing count of coordinates equal to two. Each is determined
    /// by one line sum and its already reconstructed two predecessors.
    /// All 27 line sums are then checked independently for consistency.
    pub fn play(&self) -> Result<[i8; CELLS]> {
        for plane in &self.shadows {
            for &value in plane {
                if !(-3..=3).contains(&value) {
                    return Err("projected shadow sum outside [-3,3]".to_owned());
                }
            }
        }
        let mut values: [Option<i8>; CELLS] = [None; CELLS];
        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    values[cell(x, y, z)] = Some(
                        check_trit(self.interior[x + 2 * y + 4 * z])?
                    );
                }
            }
        }
        for layer in 1..=3 {
            for z in 0..3 {
                for y in 0..3 {
                    for x in 0..3 {
                        let boundary = usize::from(x == 2)
                            + usize::from(y == 2)
                            + usize::from(z == 2);
                        if boundary != layer {
                            continue;
                        }
                        let (axis, pixel, before_a, before_b) = if x == 2 {
                            (0, shadow_index(0, x, y, z),
                             cell(0, y, z), cell(1, y, z))
                        } else if y == 2 {
                            (1, shadow_index(1, x, y, z),
                             cell(x, 0, z), cell(x, 1, z))
                        } else {
                            (2, shadow_index(2, x, y, z),
                             cell(x, y, 0), cell(x, y, 1))
                        };
                        let a = values[before_a]
                            .ok_or("missing first predecessor in shadow reconstruction")?;
                        let b = values[before_b]
                            .ok_or("missing second predecessor in shadow reconstruction")?;
                        let recovered = i16::from(self.shadows[axis][pixel])
                            - i16::from(a) - i16::from(b);
                        let trit = i8::try_from(recovered)
                            .map_err(|_| "reconstructed shadow cell width overflow")?;
                        values[cell(x, y, z)] = Some(check_trit(trit)?);
                    }
                }
            }
        }
        let mut full = [0_i8; CELLS];
        for (destination, value) in full.iter_mut().zip(values) {
            *destination = value.ok_or("three-shadow incomplete 3-D reconstruction")?;
        }
        let replayed = Self::freeze(&full)?;
        if replayed != *self {
            return Err("three 2-D light shadows disagree with reconstructed 3-D source".to_owned());
        }
        Ok(full)
    }

    pub fn bytes(&self) -> [u8; SHADOW_BYTES] {
        let mut result = [0_u8; SHADOW_BYTES];
        for (axis, plane) in self.shadows.iter().enumerate() {
            for (i, &value) in plane.iter().enumerate() {
                result[axis * PIXELS + i] = value as u8;
            }
        }
        for (i, &value) in self.interior.iter().enumerate() {
            result[SHADOWS * PIXELS + i] = value as u8;
        }
        result
    }

    pub fn from_bytes(bytes: [u8; SHADOW_BYTES]) -> Result<Self> {
        let mut shadows = [[0_i8; PIXELS]; SHADOWS];
        let mut interior = [0_i8; RESIDUALS];
        for (axis, plane) in shadows.iter_mut().enumerate() {
            for (i, value) in plane.iter_mut().enumerate() {
                *value = bytes[axis * PIXELS + i] as i8;
            }
        }
        for (i, value) in interior.iter_mut().enumerate() {
            *value = bytes[SHADOWS * PIXELS + i] as i8;
        }
        let record = Self { shadows, interior };
        record.play()?;
        Ok(record)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreeShadowStats {
    pub slices: u64,
    pub rooms: u64,
    pub source_bytes: u64,
    pub original_rime_root: String,
    pub shadow_bytes: u64,
    pub shadow_root: String,
}

fn next_root(
    previous: &str, slice: u64, offset: u64, used: usize, rooms: usize,
    source_sha: &str, shadows_sha: &str,
) -> String {
    hash_bytes(format!(
        "PNP_TRIPLE_LIGHT_STEP_V1|{previous}|{slice}|{offset}|{used}|{rooms}|{source_sha}|{shadows_sha}"
    ).as_bytes())
}
fn initial_shadow_root() -> String {
    hash_bytes(b"PNP_THREE_LIGHT_THREE_SHADOWS_V1")
}

fn parse_num(text: &str, field: &str) -> Result<u64> {
    text.parse::<u64>().map_err(|_| format!("invalid {field} integer"))
}
fn read_field<'a>(line: &'a str, key: &str) -> Result<&'a str> {
    let line = line.trim_end_matches('\n');
    let prefix = format!("{key}=");
    let mut found = line.split('|').filter_map(|field| field.strip_prefix(&prefix));
    let value = found.next().ok_or_else(|| format!("missing field {key}"))?;
    if found.next().is_some() {
        return Err(format!("duplicate field {key}"));
    }
    Ok(value)
}
fn n16(raw: &[u8]) -> u16 {
    u16::from_le_bytes([raw[0], raw[1]])
}
fn n32(raw: &[u8]) -> u32 {
    u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]])
}
fn n64(raw: &[u8]) -> u64 {
    let mut values = [0_u8; 8];
    values.copy_from_slice(raw);
    u64::from_le_bytes(values)
}
fn take<R: Read>(source: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut value = vec![0_u8; len];
    source.read_exact(&mut value).map_err(|e| format!("three-shadow stream truncated: {e}"))?;
    Ok(value)
}
fn get_header<R: Read>(source: &mut R) -> Result<Option<[u8; RIME_HEADER_BYTES]>> {
    let mut first = [0_u8; 1];
    loop {
        match source.read(&mut first) {
            Ok(0) => return Ok(None),
            Ok(_) => break,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(format!("three-shadow RimeSphere read: {e}")),
        }
    }
    let mut header = [0_u8; RIME_HEADER_BYTES];
    header[0] = first[0];
    source.read_exact(&mut header[1..])
        .map_err(|e| format!("three-shadow RimeSphere header truncated: {e}"))?;
    Ok(Some(header))
}
fn old_rime_identity(
    header: &[u8; RIME_HEADER_BYTES],
    expected_slice: u64,
    expected_offset: u64,
) -> Result<(usize, usize, String, String)> {
    if &header[..8] != RIME_MAGIC
        || n64(&header[8..16]) != expected_slice
        || n64(&header[16..24]) != expected_offset
    {
        return Err("original Bitcoin RimeSphere source identity or order changed".to_owned());
    }
    let used = n16(&header[24..26]) as usize;
    let rooms = n16(&header[26..28]) as usize;
    if used == 0 || used > BYTES_PER_SLICE
        || rooms != (used * DIGITS_PER_BYTE).div_ceil(CELLS)
        || n32(&header[28..32]) as usize != rooms * CELLS
    {
        return Err("source RimeSphere original 27-cell geometry changed".to_owned());
    }
    let source_sha = std::str::from_utf8(&header[32..96])
        .map_err(|_| "invalid original RimeSphere SHA")?.to_owned();
    let root = std::str::from_utf8(&header[96..160])
        .map_err(|_| "invalid original RimeSphere root")?.to_owned();
    if source_sha.len() != 64 || root.len() != 64
        || !source_sha.bytes().all(|b| b.is_ascii_hexdigit())
        || !root.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("invalid original RimeSphere root width".to_owned());
    }
    Ok((used, rooms, source_sha, root))
}

fn verify_rime_plane(
    plane: &[u8], used: usize, source_sha: &str,
) -> Result<()> {
    let mut original = Vec::with_capacity(used);
    for six in plane[..used * DIGITS_PER_BYTE].chunks_exact(DIGITS_PER_BYTE) {
        let byte = trits_byte(six)?;
        if six != byte_trits(byte).as_slice() {
            return Err("original RimeSphere source ternary projection differs".to_owned());
        }
        original.push(byte);
    }
    if plane[used * DIGITS_PER_BYTE..].iter().any(|&v| v != 0)
        || hash_bytes(&original) != source_sha
    {
        return Err("RimeSphere source hash or original source padding differs".to_owned());
    }
    Ok(())
}

/// Project every source room as three orthogonal signed 3x3 shadows plus the
/// eight residual trits. Output stores only 35 bytes per room and one
/// source-chained HBI row per original 729-byte gulp.
pub fn freeze_stream<R: Read, W: Write, I: Write>(
    rime: &mut R, shadow_writer: &mut W, index: &mut I,
) -> Result<ThreeShadowStats> {
    let mut stats = ThreeShadowStats {
        slices: 0, rooms: 0, source_bytes: 0,
        original_rime_root: original_root(),
        shadow_bytes: 0, shadow_root: initial_shadow_root(),
    };
    while let Some(header) = get_header(rime)? {
        let (used, rooms, source_sha, rime_root) =
            old_rime_identity(&header, stats.slices, stats.source_bytes)?;
        let plane = take(rime, rooms * CELLS)?;
        verify_rime_plane(&plane, used, &source_sha)?;
        let expected_rime = hash_bytes(format!(
            "PNP_RIME_STREAM_STEP_V1|{}|{}|{}|{}|{}|{}|{}",
            stats.original_rime_root, stats.slices, stats.source_bytes,
            used, rooms, source_sha, hash_bytes(&plane)
        ).as_bytes());
        if expected_rime != rime_root {
            return Err("original source RimeSphere rolling root differs".to_owned());
        }
        let mut shadow_slice = Vec::with_capacity(rooms * SHADOW_BYTES);
        for room in plane.chunks_exact(CELLS) {
            let mut source = [0_i8; CELLS];
            for (dest, &value) in source.iter_mut().zip(room) {
                *dest = value as i8;
            }
            let frozen = ThreeShadows::freeze(&source)?;
            let restored = frozen.play()?;
            if restored != source {
                return Err("three independent shadows changed source during freeze".to_owned());
            }
            shadow_slice.extend_from_slice(&frozen.bytes());
        }
        shadow_writer.write_all(&shadow_slice)
            .map_err(|e| format!("three-shadow binary stream write: {e}"))?;
        let shadow_sha = hash_bytes(&shadow_slice);
        let root = next_root(&stats.shadow_root, stats.slices,
                             stats.source_bytes, used, rooms,
                             &source_sha, &shadow_sha);
        writeln!(index,
            "THREE_SHADOW_SLICE|slice={}|source_offset={}|source_bytes={used}|rooms={rooms}|trits={}|original_sha256={source_sha}|original_rime_root={rime_root}|shadow_sha256={shadow_sha}|shadow_root={root}|json=0",
            stats.slices, stats.source_bytes, used * DIGITS_PER_BYTE
        ).map_err(|e| format!("three-shadow source index write: {e}"))?;
        stats.slices += 1;
        stats.rooms += rooms as u64;
        stats.source_bytes += used as u64;
        stats.shadow_bytes += shadow_slice.len() as u64;
        stats.original_rime_root = rime_root;
        stats.shadow_root = root;
    }
    if stats.slices == 0 {
        return Err("no original Bitcoin RimeSphere slices to project".to_owned());
    }
    Ok(stats)
}

/// Independently restore every 3-D room from shadow planes + eight witnesses,
/// enforce canonical source byte groups, and verify the rolling shadow root.
pub fn play_stream<R: Read, I: BufRead, W: Write>(
    shadows: &mut R, index: &mut I, recovered: &mut W,
) -> Result<ThreeShadowStats> {
    let mut stats = ThreeShadowStats {
        slices: 0, rooms: 0, source_bytes: 0,
        original_rime_root: original_root(),
        shadow_bytes: 0, shadow_root: initial_shadow_root(),
    };
    let mut line = String::new();
    loop {
        line.clear();
        let count = index.read_line(&mut line)
            .map_err(|e| format!("three-shadow HBI read: {e}"))?;
        if count == 0 { break; }
        if !line.starts_with("THREE_SHADOW_SLICE|")
            || !line.ends_with("\n")
            || read_field(&line, "json")? != "0"
        {
            return Err("three-shadow index line is malformed".to_owned());
        }
        let slice = parse_num(read_field(&line, "slice")?, "slice")?;
        let offset = parse_num(read_field(&line, "source_offset")?, "offset")?;
        let used = parse_num(read_field(&line, "source_bytes")?, "source bytes")? as usize;
        let rooms = parse_num(read_field(&line, "rooms")?, "rooms")? as usize;
        if slice != stats.slices || offset != stats.source_bytes
            || used == 0 || used > BYTES_PER_SLICE
            || rooms != (used * DIGITS_PER_BYTE).div_ceil(CELLS)
            || parse_num(read_field(&line, "trits")?, "trits")?
                != (used * DIGITS_PER_BYTE) as u64
        {
            return Err("three-shadow room dimensions or slice order differ".to_owned());
        }
        let input = take(shadows, rooms * SHADOW_BYTES)?;
        let wanted_shadow_sha = read_field(&line, "shadow_sha256")?;
        if hash_bytes(&input) != wanted_shadow_sha {
            return Err("three-shadow binary data differs from frozen index".to_owned());
        }
        let mut plane = Vec::with_capacity(rooms * CELLS);
        for bytes in input.chunks_exact(SHADOW_BYTES) {
            let array: [u8; SHADOW_BYTES] = bytes.try_into()
                .map_err(|_| "invalid three-shadow record length")?;
            let projection = ThreeShadows::from_bytes(array)?;
            plane.extend(projection.play()?.map(|v| v as u8));
        }
        let mut original = Vec::with_capacity(used);
        for six in plane[..used * DIGITS_PER_BYTE].chunks_exact(DIGITS_PER_BYTE) {
            let byte = trits_byte(six)?;
            if byte_trits(byte).as_slice() != six {
                return Err("noncanonical recovered source byte from three shadows".to_owned());
            }
            original.push(byte);
        }
        let source_sha = read_field(&line, "original_sha256")?;
        if hash_bytes(&original) != source_sha
            || plane[used * DIGITS_PER_BYTE..].iter().any(|&v| v != 0)
        {
            return Err("three-shadow reconstructed original source differs".to_owned());
        }
        let rime_root = read_field(&line, "original_rime_root")?;
        let shadow_root = next_root(
            &stats.shadow_root, slice, offset, used, rooms,
            source_sha, wanted_shadow_sha,
        );
        if read_field(&line, "shadow_root")? != shadow_root {
            return Err("three-shadow chained index differs from reconstructed source".to_owned());
        }
        // The original RIME root is carried from the source-bound streamer.
        // It is checked against its independent frozen source at freeze.
        if rime_root.len() != 64 || !rime_root.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid parent RIME root in three-shadow replay".to_owned());
        }
        recovered.write_all(&original)
            .map_err(|e| format!("three-shadow restored source write: {e}"))?;
        stats.slices += 1;
        stats.source_bytes += used as u64;
        stats.rooms += rooms as u64;
        stats.shadow_bytes += input.len() as u64;
        stats.original_rime_root = rime_root.to_owned();
        stats.shadow_root = shadow_root;
    }
    let mut extra = [0_u8; 1];
    if stats.slices == 0
        || shadows.read(&mut extra).map_err(|e| format!("shadow trailer read: {e}"))? != 0
    {
        return Err("three-shadow binary file has missing index or extra trailing bytes".to_owned());
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn three_lights_plus_eight_witness_trits_are_exact() {
        let mut state = 0x9e3779b97f4a7c15_u64;
        for _ in 0..1024 {
            let mut source = [0_i8; CELLS];
            for value in &mut source {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                *value = match state % 3 {
                    0 => -1,
                    1 => 0,
                    _ => 1,
                };
            }
            let shadow = ThreeShadows::freeze(&source).expect("three shadows");
            assert_eq!(shadow.play().expect("restore three-dimensional"), source);
            assert_eq!(ThreeShadows::from_bytes(shadow.bytes())
                       .expect("stored shadows").play().expect("reconstructed"), source);
        }
    }

    #[test]
    fn three_shadows_alone_have_eight_missing_directions() {
        // The ± checkerboard in one 2x2x2 interior cube has zero
        // projections along all three axes. Yet the 3-D cell data differs.
        let zero = [0_i8; CELLS];
        let mut checker = zero;
        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    checker[cell(x,y,z)] =
                        if (x + y + z) % 2 == 0 { 1 } else { -1 };
                }
            }
        }
        let base = ThreeShadows::freeze(&zero).expect("zero");
        let changed = ThreeShadows::freeze(&checker).expect("checker");
        assert_eq!(base.shadows, changed.shadows);
        assert_ne!(base.interior, changed.interior);
        assert_eq!(changed.play().expect("witness"), checker);
        assert_eq!(EXPECTED_RANK + RESIDUALS, CELLS);
    }

    #[test]
    fn all_6561_interior_states_reconstruct_exactly() {
        for mut number in 0..6561 {
            let mut source = [0_i8; CELLS];
            for z in 0..2 {
                for y in 0..2 {
                    for x in 0..2 {
                        source[cell(x,y,z)] = (number % 3) as i8 - 1;
                        number /= 3;
                    }
                }
            }
            let shadows = ThreeShadows::freeze(&source).expect("freeze");
            assert_eq!(shadows.play().expect("play"), source);
        }
    }

    #[test]
    fn streaming_three_independent_shadows_recover_exact_original_bytes() {
        let source: Vec<u8> = (0..(BYTES_PER_SLICE * 2 + 19))
            .map(|i| (i % 256) as u8).collect();
        let mut rime = Vec::new();
        let stats_original = crate::bitcoin_rimesphere_stream::stream(
            &mut Cursor::new(&source), &mut rime, &mut Vec::new()
        ).expect("original RimeSphere");
        let mut shadow_binary = Vec::new();
        let mut hbi = Vec::new();
        let frozen = freeze_stream(
            &mut Cursor::new(&rime), &mut shadow_binary, &mut hbi
        ).expect("three-shadow freeze");
        let mut restored = Vec::new();
        let played = play_stream(
            &mut Cursor::new(&shadow_binary),
            &mut BufReader::new(Cursor::new(&hbi)),
            &mut restored,
        ).expect("three-shadow play");
        assert_eq!(frozen, played);
        assert_eq!(stats_original.bytes, frozen.source_bytes);
        assert_eq!(frozen.original_rime_root, stats_original.root);
        assert_eq!(restored, source);
        let mut corrupt = shadow_binary;
        corrupt[0] ^= 0x80;
        assert!(play_stream(
            &mut Cursor::new(&corrupt),
            &mut BufReader::new(Cursor::new(&hbi)),
            &mut Vec::new(),
        ).is_err());
    }

    #[test]
    fn inconsistent_shadows_and_residuals_rejected() {
        let bytes = [0_u8; SHADOW_BYTES];
        let mut invalid_sum = bytes;
        invalid_sum[0] = 8;
        assert!(ThreeShadows::from_bytes(invalid_sum).is_err());
        let mut invalid_witness = bytes;
        invalid_witness[34] = 7;
        assert!(ThreeShadows::from_bytes(invalid_witness).is_err());
    }
}
