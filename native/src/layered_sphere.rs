//! Reversible staged star-glyph languages and occupied/empty microkernel rooms.
//! In the four certified binary alphabets 64/256/1024/4096 and a separately
//! tagged proposed 16384-symbol rung, a tuple is a reversible address.
//! Its code/shell/length/identity are charged; a larger alphabet is not a
//! smaller stored message. The 64-symbol floor is SIX bits per symbol.
#![forbid(unsafe_code)]

use crate::binary_ternary_bridge::{decode_floor, encode_floor, FloorWord};
use crate::fischer_prime_tower::reverse_fischer_address;
use crate::freeze::fischer_address;
use crate::matrix::{coordinates, CELLS, OMEGA};
use crate::omega_operator_catalog as omega;
use crate::prism;
use crate::sha256::hash_bytes;
use crate::ternary::Trit;

const FRAME_VERSION: u8 = 1;
pub const FLOORS: [(u16, u8, usize); 5] = [
    (64, 6, 3), (256, 8, 5), (1024, 10, 6),
    (4096, 12, 7), (16384, 14, 8),
];
const SHA_TEXT: usize = 64;
const HEADER: usize = 7;
const SAMPLE_TRITS: usize = CELLS;

fn digit(trit: Trit) -> u16 {
    match trit {
        Trit::Minus => 0, Trit::Zero => 1, Trit::Plus => 2,
    }
}
fn trit(digit: u16) -> Result<Trit, &'static str> {
    match digit {
        0 => Ok(Trit::Minus),
        1 => Ok(Trit::Zero),
        2 => Ok(Trit::Plus),
        _ => Err("invalid trit digit"),
    }
}
fn floor(floor: u16) -> Result<(u8, usize), &'static str> {
    FLOORS.iter().find(|row| row.0 == floor)
        .map(|row| (row.1, row.2)).ok_or("unknown binary star floor")
}
fn payload_sha(trits: &[Trit]) -> String {
    let mut payload = Vec::with_capacity(trits.len());
    for &t in trits {
        payload.push(digit(t) as u8);
    }
    hash_bytes(&payload)
}
fn ordinal(trits: &[Trit]) -> u16 {
    trits.iter().fold(0_u16, |v, t| v * 3 + digit(*t))
}
fn inverse_ordinal(mut value: u16, width: usize)
    -> Result<Vec<Trit>, &'static str>
{
    let mut out = vec![Trit::Zero; width];
    for item in out.iter_mut().rev() {
        *item = trit(value % 3)?;
        value /= 3;
    }
    if value != 0 {
        return Err("star-glyph overflow");
    }
    Ok(out)
}

/// Self-contained level translation: version, floor alphabet, original trit
/// count, 16-bit glyph count, little-endian glyph words, 64 ASCII digest.
/// No original trits are carried as an uncharged shadow bank.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LevelPacket {
    bytes: Vec<u8>,
}
impl LevelPacket {
    pub fn freeze(floor_name: u16, trits: &[Trit]) -> Result<Self, &'static str> {
        let (_, width) = floor(floor_name)?;
        let count = trits.len().div_ceil(width);
        let input_len = u16::try_from(trits.len()).map_err(|_| "trit length overflow")?;
        let count_word = u16::try_from(count).map_err(|_| "symbol count overflow")?;
        let mut bytes = vec![FRAME_VERSION];
        bytes.extend_from_slice(&floor_name.to_le_bytes());
        bytes.extend_from_slice(&input_len.to_le_bytes());
        bytes.extend_from_slice(&count_word.to_le_bytes());
        for chunk in trits.chunks(width) {
            let mut block = chunk.to_vec();
            block.resize(width, Trit::Zero);
            let code = ordinal(&block);
            if code >= floor_name {
                return Err("glyph exceeds binary alphabet");
            }
            if floor_name != 16384
                && encode_floor(floor_name, &block)?.code != code
            {
                return Err("source-certified floor parity mismatch");
            }
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        bytes.extend_from_slice(payload_sha(trits).as_bytes());
        let packet = Self { bytes };
        if packet.play()? != trits {
            return Err("frozen star floor not byte-exact");
        }
        Ok(packet)
    }
    pub fn load(bytes: Vec<u8>) -> Result<Self, &'static str> {
        let candidate = Self { bytes };
        candidate.play()?;
        Ok(candidate)
    }
    pub fn serialized(&self) -> &[u8] {
        &self.bytes
    }
    pub fn paid_bytes(&self) -> usize {
        self.bytes.len()
    }
    pub fn name(&self) -> Result<u16, &'static str> {
        if self.bytes.len() < HEADER {
            return Err("truncated star floor header");
        }
        Ok(u16::from_le_bytes([self.bytes[1], self.bytes[2]]))
    }
    fn words(&self) -> Result<(u16, usize, Vec<u16>), &'static str> {
        if self.bytes.len() < HEADER + SHA_TEXT
            || self.bytes[0] != FRAME_VERSION
        {
            return Err("star floor version or length invalid");
        }
        let floor_name = self.name()?;
        let (_, width) = floor(floor_name)?;
        let count = usize::from(u16::from_le_bytes([self.bytes[5], self.bytes[6]]));
        let original_len = usize::from(u16::from_le_bytes([self.bytes[3], self.bytes[4]]));
        if count != original_len.div_ceil(width)
            || self.bytes.len() != HEADER + count * 2 + SHA_TEXT
        {
            return Err("star floor count or record length mismatch");
        }
        let mut words = Vec::with_capacity(count);
        for pair in self.bytes[HEADER..HEADER + count * 2].chunks_exact(2) {
            words.push(u16::from_le_bytes([pair[0], pair[1]]));
        }
        Ok((floor_name, original_len, words))
    }
    pub fn play(&self) -> Result<Vec<Trit>, &'static str> {
        let (floor_name, original_len, words) = self.words()?;
        let (_, width) = floor(floor_name)?;
        let mut output = Vec::with_capacity(words.len() * width);
        for code in words {
            if u32::from(code) >= 3_u32.pow(width as u32) {
                return Err("reserved binary glyph used as data");
            }
            if floor_name == 16384 {
                output.extend(inverse_ordinal(code, width)?);
            } else {
                output.extend(decode_floor(FloorWord { floor: floor_name, code })?);
            }
        }
        if output[original_len..].iter().any(|t| *t != Trit::Zero) {
            return Err("noncanonical star padding");
        }
        output.truncate(original_len);
        let end = self.bytes.len() - SHA_TEXT;
        if payload_sha(&output).as_bytes() != &self.bytes[end..] {
            return Err("star floor payload SHA mismatch");
        }
        Ok(output)
    }
    /// Independent decoder for the third-seat witness: no calls to the
    /// bridge decoder and no stored original payload.
    pub fn independently_recover(&self) -> Result<Vec<Trit>, &'static str> {
        let (floor_name, original_len, words) = self.words()?;
        let (_, width) = floor(floor_name)?;
        let mut decoded = Vec::with_capacity(words.len() * width);
        for code in words {
            if u32::from(code) >= 3_u32.pow(width as u32) {
                return Err("third-seat reserved glyph");
            }
            let mut tmp = vec![0_u16; width];
            let mut n = code;
            for i in (0..width).rev() {
                tmp[i] = n % 3;
                n /= 3;
            }
            if n != 0 {
                return Err("third-seat glyph overflow");
            }
            for symbol in tmp {
                decoded.push(trit(symbol)?);
            }
        }
        if decoded[original_len..].iter().any(|t| *t != Trit::Zero) {
            return Err("third-seat padding mismatch");
        }
        decoded.truncate(original_len);
        if payload_sha(&decoded).as_bytes() != &self.bytes[self.bytes.len() - SHA_TEXT..] {
            return Err("third-seat checksum mismatch");
        }
        Ok(decoded)
    }
    pub fn rebase(&self, next_floor: u16) -> Result<Self, &'static str> {
        Self::freeze(next_floor, &self.play()?)
    }
    pub fn star_glyphs(&self) -> Result<Vec<u8>, &'static str> {
        let source = self.play()?;
        let mut out = Vec::with_capacity(source.len().div_ceil(3));
        for chunk in source.chunks(3) {
            let mut triad = chunk.to_vec();
            triad.resize(3, Trit::Zero);
            out.push(ordinal(&triad) as u8);
        }
        Ok(out)
    }
    pub fn omnibit_roundtrip(&self, operator: u8) -> Result<(), &'static str> {
        let (floor_name, _, words) = self.words()?;
        if floor_name == 16384 {
            return Err("16384 Omnibit operator awaits source-certified 14-bit map");
        }
        if operator >= 40 {
            return Err("unknown Omnibit function");
        }
        let (bits, _) = floor(floor_name)?;
        let wound = omega::apply_symbols(operator, bits, &words)
            .map_err(|_| "Omnibit forward failed")?;
        let undone = omega::invert_symbols(operator, bits, &wound)
            .map_err(|_| "Omnibit inverse failed")?;
        if undone != words {
            return Err("Omnibit lost original star glyphs");
        }
        Ok(())
    }
    pub fn qprism_roundtrip(&self) -> Result<(), &'static str> {
        let raw = self.serialized();
        let projected = prism::encode(raw);
        if prism::decode(&projected, raw.len())? != raw {
            return Err("Q-PRISM 256↔1024 did not recover star floor");
        }
        Ok(())
    }
}

/// Fischer can choose *any* 27-cell star as the local null and order
/// neighbors by their 3D cube distance to that chosen center.
pub fn fischer_from_any_center(anchor: usize, room: usize)
    -> Result<Vec<(usize, u64)>, &'static str>
{
    let (ax, ay, az) = coordinates(anchor).ok_or("invalid Fischer anchor")?;
    let mut indices: Vec<usize> = (0..CELLS).collect();
    indices.sort_by_key(|&i| {
        let (x, y, z) = coordinates(i).expect("fixed 27-cell cube");
        (x.abs_diff(ax) + y.abs_diff(ay) + z.abs_diff(az), i)
    });
    let mut checked = Vec::with_capacity(CELLS);
    for index in indices {
        let address = fischer_address(index as u64, room as u64)
            .ok_or("Fischer address overflow")?;
        if reverse_fischer_address(address) != Some((index as u64, room as u64)) {
            return Err("Fischer endpoint prime-tower inverse rejected");
        }
        checked.push((index, address));
    }
    if checked.first().map(|entry| entry.0) != Some(anchor) {
        return Err("Fischer anchor not first");
    }
    Ok(checked)
}

/// A reusable one-function MCP-style room. Every call clears its exact
/// owned scratch before returning, including rejected input. Persistent
/// learning weights, trained GNNs and protected source are held separately.
#[derive(Default)]
pub struct Room {
    occupied: Vec<u8>,
    active: bool,
    gulps: usize,
}
impl Room {
    pub fn process(&mut self, packet: &LevelPacket, operator: u8)
        -> Result<Vec<Trit>, &'static str>
    {
        if self.active || !self.occupied.is_empty() {
            return Err("microkernel room not empty");
        }
        self.active = true;
        self.occupied.extend_from_slice(packet.serialized());
        let work = (|| {
            let on_disk = LevelPacket::load(self.occupied.clone())?;
            if on_disk != *packet {
                return Err("microkernel incoming source mismatch");
            }
            if on_disk.name()? != 16384 {
                on_disk.omnibit_roundtrip(operator)?;
            } else if operator != 0 {
                return Err("14-bit Omnibit trained/map source not admitted");
            }
            let worker = on_disk.play()?;
            let reflection = on_disk.independently_recover()?;
            let watched = LevelPacket::freeze(on_disk.name()?, &reflection)?;
            if worker != reflection || watched != on_disk {
                return Err("MTP/reflection/fabric third seat disagrees");
            }
            on_disk.qprism_roundtrip()?;
            Ok(worker)
        })();
        self.occupied.fill(0);
        self.occupied.clear();
        self.active = false;
        self.gulps += 1;
        work
    }
    pub fn clear(&self) -> bool {
        !self.active && self.occupied.is_empty()
    }
    pub fn gulps(&self) -> usize {
        self.gulps
    }
}

/// Bounded three-child shell; each child makes an independently checked
/// presentation and parent verifies all three original source identities.
pub fn three_child_shell(source: &LevelPacket, operator: u8)
    -> Result<usize, &'static str>
{
    let mut successful = 0;
    let mut parent = Room::default();
    let original = source.play()?;
    for _ in 0..3 {
        let mut child = Room::default();
        let message = child.process(source, operator)?;
        if message != original || !child.clear() || child.gulps() != 1 {
            return Err("child presentation/release mismatch");
        }
        let received = LevelPacket::freeze(source.name()?, &message)?;
        if parent.process(&received, operator)? != original
            || !parent.clear()
        {
            return Err("parent MTP/third watcher mismatch");
        }
        successful += 1;
    }
    if parent.gulps() != 3 {
        return Err("parent room not rearmed for every presentation");
    }
    Ok(successful)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SphereStats {
    pub rooms: usize,
    pub floors: usize,
    pub transitions: usize,
    pub microkernel_gulps: usize,
    pub child_presentations: usize,
    pub star_glyphs: usize,
    pub paid_bytes: usize,
    pub source_bytes: usize,
    pub arbitrary_null_checks: usize,
    pub rejected: usize,
}

pub fn certify() -> Result<SphereStats, &'static str> {
    let mut stats = SphereStats {
        rooms: 0, floors: 0, transitions: 0,
        microkernel_gulps: 0, child_presentations: 0,
        star_glyphs: 0, paid_bytes: 0, source_bytes: 0,
        arbitrary_null_checks: 0, rejected: 0,
    };
    for count in [3_usize, 9, 27, 81] {
        for room in 0..count {
            let trits: Vec<Trit> = (0..SAMPLE_TRITS).map(|i| match (room + i * 2) % 3 {
                0 => Trit::Minus,
                1 => Trit::Zero,
                _ => Trit::Plus,
            }).collect();
            let mut last: Option<LevelPacket> = None;
            let mut room_kernel = Room::default();
            for (floor_name, _, _) in FLOORS {
                let packet = LevelPacket::freeze(floor_name, &trits)?;
                let played = room_kernel.process(&packet, 0)?;
                if played != trits || !room_kernel.clear() {
                    return Err("five-floor replay or garbage collector failed");
                }
                stats.microkernel_gulps += 1;
                let stars = packet.star_glyphs()?;
                if stars.len() != 9 || stars.iter().any(|v| *v > 26) {
                    return Err("incorrect balanced ternary star alphabet");
                }
                stats.star_glyphs += stars.len();
                if floor_name != 16384 {
                    packet.omnibit_roundtrip((room % 40) as u8)?;
                }
                if let Some(previous) = &last {
                    let rebased = previous.rebase(floor_name)?;
                    if rebased != packet
                        || packet.rebase(previous.name()?)? != *previous
                    {
                        return Err("binary ternary prism rung not invertible");
                    }
                    stats.transitions += 1;
                }
                if floor_name == 64 {
                    stats.child_presentations += three_child_shell(&packet, 0)?;
                }
                stats.paid_bytes += packet.paid_bytes();
                stats.source_bytes += trits.len();
                stats.floors += 1;
                last = Some(packet);
            }
            if room_kernel.gulps() != 5 {
                return Err("microkernel owned scratch not deleted after every slice");
            }
            for anchor in [0_usize, OMEGA, 26_usize] {
                let visit = fischer_from_any_center(anchor, room)?;
                if visit.len() != CELLS {
                    return Err("Fischer any-center lookup incomplete");
                }
                stats.arbitrary_null_checks += 1;
            }
            stats.rooms += 1;
        }
    }
    if stats.rooms != 120 || stats.floors != 600 || stats.transitions != 480
        || stats.microkernel_gulps != 600 || stats.child_presentations != 360
        || stats.star_glyphs != 5400 || stats.arbitrary_null_checks != 360
    {
        return Err("sphere-shell receipt census changed");
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_level_reversible_star_ladder_and_mcp_rearm() {
        let stats = certify().expect("all accepted original source floor reversals");
        assert_eq!(stats.transitions, 480);
        assert_eq!(stats.microkernel_gulps, 600);
    }
    #[test]
    fn all_star_triples_and_floor_reserved_controls() {
        for n in 0..27 {
            let mut rem = n;
            let mut src = [Trit::Zero; 3];
            for digit in src.iter_mut().rev() {
                *digit = trit(rem % 3).unwrap();
                rem /= 3;
            }
            for (floor, _, _) in FLOORS {
                let p = LevelPacket::freeze(floor, &src).unwrap();
                assert_eq!(p.star_glyphs().unwrap(), vec![n as u8]);
                assert_eq!(p.independently_recover().unwrap(), src);
            }
        }
        for (f, _, width) in FLOORS {
            let src = vec![Trit::Minus; width];
            let p = LevelPacket::freeze(f, &src).unwrap();
            let mut bad = p.serialized().to_vec();
            let first_reserved = 3_u16.pow(width as u32);
            bad[7..9].copy_from_slice(&first_reserved.to_le_bytes());
            assert!(LevelPacket::load(bad).is_err());
            let mut wrong_sha = p.serialized().to_vec();
            *wrong_sha.last_mut().unwrap() ^= 1;
            assert!(LevelPacket::load(wrong_sha).is_err());
        }
    }
    #[test]
    fn any_center_and_room_discard_invalid_operator() {
        for anchor in 0..27 {
            assert_eq!(fischer_from_any_center(anchor, 2).unwrap()[0].0, anchor);
        }
        let packet = LevelPacket::freeze(16384, &[Trit::Zero; 27]).unwrap();
        let mut room = Room::default();
        assert!(room.process(&packet, 1).is_err());
        assert!(room.clear());
        assert_eq!(room.gulps(), 1);
        assert_eq!(room.process(&packet, 0).unwrap(), vec![Trit::Zero;27]);
        assert!(room.clear());
    }
}
