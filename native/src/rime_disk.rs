//! Source-authenticated, CPU-only disk playback for the published four-axes Rime GGUF.
//! An intentionally narrow GGUF-v3 reader: 24 one-dimensional IEEE-754 binary32
//! tensors of 27 exact integer coordinates. No general GGUF/LLM interpreter.
//! File contents stay on disk between address reads; source identity is pinned
//! at open and rechecked at the independent verification boundary.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use crate::freeze::{fischer_address, power_mod};
use crate::sha256::hash_bytes;

pub const FOUR_AXES_SHA256: &str =
    "9fe92da2b064e39ad52b389fb92d0c59739e1fd7d9db5b82c6d25c0ebf8fa735";
pub const FOUR_AXES_BYTES: usize = 4_908;
pub const UPSTREAM_COMMIT: &str = "e979ebe114f32c5b6ab9c2325e252c4b8ce28ddd";
pub const P: u64 = 1_000_081;
pub const ROOT: u64 = 951_846;
pub const GLYPHS: usize = 27;
const ALIGNMENT: usize = 32;
const TENSORS: usize = 24;
const REQUEST_BYTES: usize = 11;
const REPLY_BYTES: usize = 15;

fn slice<'a>(bytes: &'a [u8], cursor: &mut usize, size: usize) -> Result<&'a [u8], String> {
    let end = cursor.checked_add(size).ok_or("GGUF cursor overflow")?;
    let out = bytes.get(*cursor..end).ok_or("GGUF input truncated")?;
    *cursor = end;
    Ok(out)
}
fn number32(bytes: &[u8], cursor: &mut usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(slice(bytes, cursor, 4)?.try_into().map_err(|_| "u32 width")?))
}
fn number64(bytes: &[u8], cursor: &mut usize) -> Result<u64, String> {
    Ok(u64::from_le_bytes(slice(bytes, cursor, 8)?.try_into().map_err(|_| "u64 width")?))
}
fn string(bytes: &[u8], cursor: &mut usize) -> Result<String, String> {
    let n = usize::try_from(number64(bytes, cursor)?).map_err(|_| "string too large")?;
    if n > 4_096 { return Err("GGUF string capacity exceeded".into()); }
    String::from_utf8(slice(bytes, cursor, n)?.to_vec()).map_err(|_| "invalid GGUF text".into())
}
fn aligned(n: usize) -> Result<usize, String> {
    n.checked_add(ALIGNMENT - 1)
        .map(|value| value & !(ALIGNMENT - 1))
        .ok_or_else(|| "alignment overflow".into())
}

/// Decode a binary32 WORD as an exact nonnegative integer using integer operations.
/// This preserves all source values without native floating point calculations.
fn exact_integer(word: u32) -> Result<u64, String> {
    if word == 0 { return Ok(0); }
    if word >> 31 != 0 { return Err("negative tensor coordinate".into()); }
    let exponent = ((word >> 23) & 255) as i32 - 127;
    if !(0..=20).contains(&exponent) { return Err("non-integer or oversized coordinate".into()); }
    let significand = (word & 0x7f_ffff) | 0x80_0000;
    let shift = u32::try_from(23 - exponent).map_err(|_| "exponent shift")?;
    let mask = (1_u32 << shift) - 1;
    if significand & mask != 0 { return Err("fractional tensor coordinate".into()); }
    let value = u64::from(significand >> shift);
    if value >= P { return Err("tensor coordinate outside Rime field".into()); }
    Ok(value)
}

fn tensor_names() -> Vec<String> {
    let mut names = Vec::with_capacity(TENSORS);
    for body in ["MYTHOS", "WIKIPEDIA", "OFFICE"] {
        for axis in ["time", "colour", "energy", "space"] {
            names.push(format!("{body}_{axis}"));
            names.push(format!("{body}_{axis}_rimed"));
        }
    }
    names
}

#[derive(Clone, Debug)]
pub struct DiskRime {
    path: PathBuf,
    length: usize,
    digest: String,
    tensor_offsets: Vec<u64>,
    tensor_names: Vec<String>,
}

impl DiskRime {
    /// The public constructor accepts exactly the byte-pinned original fixture.
    pub fn open_four_axes(path: &Path) -> Result<Self, String> {
        Self::open_checked(path, FOUR_AXES_SHA256, FOUR_AXES_BYTES)
    }

    fn open_checked(path: &Path, sha: &str, length: usize) -> Result<Self, String> {
        let raw = fs::read(path).map_err(|e| format!("GGUF open: {e}"))?;
        if raw.len() != length || hash_bytes(&raw) != sha {
            return Err("GGUF source length/SHA-256 mismatch".into());
        }
        let mut offset = 0;
        if slice(&raw, &mut offset, 4)? != b"GGUF" || number32(&raw, &mut offset)? != 3 {
            return Err("GGUF magic/version rejected".into());
        }
        if number64(&raw, &mut offset)? != TENSORS as u64 {
            return Err("unexpected four-axes tensor count".into());
        }
        let count = number64(&raw, &mut offset)?;
        if count > 64 { return Err("GGUF metadata count outside pilot".into()); }
        let mut metadata = BTreeMap::new();
        for _ in 0..count {
            let key = string(&raw, &mut offset)?;
            if number32(&raw, &mut offset)? != 8 {
                return Err("four-axes pilot requires string metadata".into());
            }
            let value = string(&raw, &mut offset)?;
            if metadata.insert(key, value).is_some() {
                return Err("duplicate GGUF metadata key".into());
            }
        }
        if metadata.get("general.architecture").map(String::as_str)
            != Some("asolaria-four-axes-rime")
        {
            return Err("source architecture mismatched".into());
        }
        let prism = metadata.get("asolaria.prism").ok_or("original prism metadata missing")?;
        if !["p=1000081", "g=7", "w=951846", "k=27"].iter()
            .all(|part| prism.contains(part))
        {
            return Err("prism constants mismatch".into());
        }

        let names = tensor_names();
        let mut tensor_offsets = Vec::with_capacity(TENSORS);
        for name in &names {
            let actual = string(&raw, &mut offset)?;
            if &actual != name { return Err("unexpected GGUF tensor identity/order".into()); }
            if number32(&raw, &mut offset)? != 1 || number64(&raw, &mut offset)? != GLYPHS as u64 {
                return Err("invalid 27-coordinate tensor extent".into());
            }
            if number32(&raw, &mut offset)? != 0 {
                return Err("tensor element encoding outside pilot".into());
            }
            let relative = number64(&raw, &mut offset)?;
            if relative % ALIGNMENT as u64 != 0 {
                return Err("GGUF tensor relative offset not aligned".into());
            }
            tensor_offsets.push(relative);
        }
        let start = aligned(offset)?;
        let mut spans = Vec::with_capacity(TENSORS);
        for relative in &tensor_offsets {
            let absolute = u64::try_from(start).map_err(|_| "data start width")?
                .checked_add(*relative).ok_or("tensor offset overflow")?;
            let end = absolute.checked_add((GLYPHS * 4) as u64)
                .ok_or("tensor end overflow")?;
            if end > raw.len() as u64 { return Err("tensor runs beyond file".into()); }
            spans.push((absolute, end));
        }
        for pair in spans.windows(2) {
            if pair[0].1 > pair[1].0 { return Err("overlapping GGUF tensors".into()); }
        }

        Ok(Self {
            path: path.to_path_buf(), length, digest: sha.to_owned(),
            tensor_offsets: spans.into_iter().map(|(start, _)| start).collect(),
            tensor_names: names,
        })
    }

    /// Hash the source again at a checkpoint; per-read disk I/O stays bounded.
    pub fn verify_source(&self) -> Result<(), String> {
        let bytes = fs::read(&self.path).map_err(|e| format!("GGUF verification read: {e}"))?;
        if bytes.len() != self.length || hash_bytes(&bytes) != self.digest {
            return Err("frozen GGUF changed after admission".into());
        }
        Ok(())
    }

    pub fn tensor_count(&self) -> usize { self.tensor_names.len() }

    pub fn read(&self, tensor: usize, glyph: usize, address: u64) -> Result<u64, String> {
        let expected = fischer_address(
            u64::try_from(glyph).map_err(|_| "glyph width")?,
            u64::try_from(tensor).map_err(|_| "tensor address width")?,
        ).ok_or("Fischer glyph out of range")?;
        if address != expected { return Err("false Fischer address".into()); }
        let base = *self.tensor_offsets.get(tensor).ok_or("tensor index out of range")?;
        let relative = u64::try_from(glyph.checked_mul(4).ok_or("glyph offset overflow")?)
            .map_err(|_| "glyph offset width")?;
        let absolute = base.checked_add(relative).ok_or("disk seek overflow")?;
        let mut file = File::open(&self.path).map_err(|e| format!("GGUF page open: {e}"))?;
        if usize::try_from(file.metadata().map_err(|e| e.to_string())?.len())
            .map_err(|_| "GGUF file too large")? != self.length
        {
            return Err("GGUF file extent changed".into());
        }
        file.seek(SeekFrom::Start(absolute)).map_err(|e| e.to_string())?;
        let mut word = [0u8; 4];
        file.read_exact(&mut word).map_err(|e| e.to_string())?;
        exact_integer(u32::from_le_bytes(word))
    }

    fn value(&self, tensor: usize, glyph: usize) -> Result<u64, String> {
        let address = fischer_address(glyph as u64, tensor as u64)
            .ok_or("invalid Rime glyph")?;
        self.read(tensor, glyph, address)
    }

    /// Path 1: exact stored value, Path 2: forward Rime, Path 3: inverse
    /// Rime, fourth point: independently compare both directions plus SHA.
    /// Both transform directions use modular integer arithmetic.
    pub fn verify_three_paths(&self) -> Result<usize, String> {
        self.verify_source()?;
        if power_mod(ROOT, GLYPHS as u64, P) != 1
            || power_mod(ROOT, 1, P) == 1
            || power_mod(ROOT, 9, P) == 1
        {
            return Err("Rime root order changed".into());
        }
        let root_inverse = power_mod(ROOT, (GLYPHS - 1) as u64, P);
        let inverse_27 = power_mod(GLYPHS as u64, P - 2, P);
        let mut checked = 0;
        for pair in (0..TENSORS).step_by(2) {
            let mut original = [0_u64; GLYPHS];
            let mut rimed = [0_u64; GLYPHS];
            for i in 0..GLYPHS {
                original[i] = self.value(pair, i)?;
                rimed[i] = self.value(pair + 1, i)?;
            }
            for i in 0..GLYPHS {
                let mut forward = 0_u64;
                let mut backward = 0_u64;
                for j in 0..GLYPHS {
                    let e = ((i * j) % GLYPHS) as u64;
                    forward = (forward + original[j] * power_mod(ROOT, e, P)) % P;
                    backward = (backward + rimed[j] * power_mod(root_inverse, e, P)) % P;
                }
                if forward != rimed[i] || backward * inverse_27 % P != original[i] {
                    return Err(format!("three-path Rime witness failed: pair={pair} glyph={i}"));
                }
                checked += 1;
            }
        }
        self.verify_source()?;
        Ok(checked)
    }

    /// Independently reconstruct every original tensor as a 3x3x3 cube.
    /// Three orthogonal 9-cell shadows plus eight corner residuals recover
    /// all 27 original coordinates without approximation or further data.
    pub fn verify_cube_shadows(&self) -> Result<usize, String> {
        self.verify_source()?;
        for tensor in 0..TENSORS {
            let mut values = [0_u64; GLYPHS];
            for (index, value) in values.iter_mut().enumerate() {
                *value = self.value(tensor, index)?;
            }
            crate::shadow3::ThreeShadows::certify_u64(&values)
                .map_err(|err| format!("three-shadow tensor {tensor}: {err}"))?;
        }
        self.verify_source()?;
        Ok(TENSORS)
    }

    /// Fixed-size HTTP-0-inspired loopback record: version, tensor, glyph,
    /// Fischer address. Source identity is established at store admission.
    pub fn reply(&self, request: &[u8]) -> Result<[u8; REPLY_BYTES], String> {
        if request.len() != REQUEST_BYTES || request[0] != 1 {
            return Err("disk-play protocol frame rejected".into());
        }
        let tensor = usize::from(request[1]);
        let glyph = usize::from(request[2]);
        let address = u64::from_le_bytes(request[3..11].try_into()
            .map_err(|_| "address frame width")?);
        let value = self.read(tensor, glyph, address)?;
        let value = u32::try_from(value).map_err(|_| "disk-play value width")?;
        let mut output = [0_u8; REPLY_BYTES];
        output[0..3].copy_from_slice(&request[0..3]);
        output[3..7].copy_from_slice(&value.to_le_bytes());
        output[7..15].copy_from_slice(&address.to_le_bytes());
        Ok(output)
    }

    /// One bounded TCP request on a caller-owned loopback listener.
    pub fn serve_one(&self, listener: &TcpListener) -> Result<(), String> {
        if !listener.local_addr().map_err(|e| e.to_string())?.ip().is_loopback() {
            return Err("HTTP-0 disk play restricted to loopback".into());
        }
        let (mut stream, _) = listener.accept().map_err(|e| e.to_string())?;
        let mut request = [0_u8; REQUEST_BYTES];
        stream.read_exact(&mut request).map_err(|e| e.to_string())?;
        let output = self.reply(&request)?;
        stream.write_all(&output).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn request(tensor: u8, glyph: u8) -> Result<[u8; REQUEST_BYTES], String> {
    let address = fischer_address(u64::from(glyph), u64::from(tensor))
        .ok_or("invalid disk-play glyph")?;
    let mut out = [0_u8; REQUEST_BYTES];
    out[0] = 1; out[1] = tensor; out[2] = glyph;
    out[3..11].copy_from_slice(&address.to_le_bytes());
    Ok(out)
}

/// Batch exact HTTP-0 source reads over one loopback connection, avoiding a
/// TCP listener and new thread for each of the 27-glyph address requests.
/// This is a separate performance control; source fidelity is unchanged.
pub fn loopback_batch(
    store: &DiskRime, addresses: &[(u8, u8)],
) -> Result<Vec<u64>, String> {
    if addresses.is_empty() || addresses.len() > 256 {
        return Err("HTTP-0 batch length outside bounded pilot".into());
    }
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let socket = listener.local_addr().map_err(|e| e.to_string())?;
    let frames = addresses.iter().map(|&(tensor, glyph)| request(tensor, glyph))
        .collect::<Result<Vec<_>, _>>()?;
    std::thread::scope(|scope| -> Result<Vec<u64>, String> {
        let worker = scope.spawn(|| -> Result<(), String> {
            let (mut stream, _) = listener.accept().map_err(|e| e.to_string())?;
            for _ in 0..frames.len() {
                let mut incoming = [0_u8; REQUEST_BYTES];
                stream.read_exact(&mut incoming).map_err(|e| e.to_string())?;
                let reply = store.reply(&incoming)?;
                stream.write_all(&reply).map_err(|e| e.to_string())?;
            }
            Ok(())
        });
        let client = (|| -> Result<Vec<u64>, String> {
            let mut stream = TcpStream::connect(socket).map_err(|e| e.to_string())?;
            let mut values = Vec::with_capacity(frames.len());
            for req in &frames {
                stream.write_all(req).map_err(|e| e.to_string())?;
                let mut reply = [0_u8; REPLY_BYTES];
                stream.read_exact(&mut reply).map_err(|e| e.to_string())?;
                if reply[0..3] != req[0..3] || reply[7..15] != req[3..11] {
                    return Err("batched HTTP-0 response identity mismatch".into());
                }
                values.push(u64::from(u32::from_le_bytes(reply[3..7].try_into()
                    .map_err(|_| "response word width")?)));
            }
            Ok(values)
        })();
        worker.join().map_err(|_| "batched HTTP-0 worker panicked".to_owned())??;
        client
    })
}

pub fn loopback_once(store: &DiskRime, tensor: u8, glyph: u8) -> Result<u64, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let socket: SocketAddr = listener.local_addr().map_err(|e| e.to_string())?;
    let payload = request(tensor, glyph)?;
    std::thread::scope(|scope| -> Result<u64, String> {
        let worker = scope.spawn(|| store.serve_one(&listener));
        let client_result = (|| -> Result<u64, String> {
            let mut stream = TcpStream::connect(socket).map_err(|e| e.to_string())?;
            stream.write_all(&payload).map_err(|e| e.to_string())?;
            let mut reply = [0u8; REPLY_BYTES];
            stream.read_exact(&mut reply).map_err(|e| e.to_string())?;
            if reply[0..3] != payload[0..3] || reply[7..15] != payload[3..11] {
                return Err("HTTP-0 reply identity mismatch".into());
            }
            Ok(u64::from(u32::from_le_bytes(reply[3..7].try_into()
                .map_err(|_| "reply value width")?)))
        })();
        worker.join().map_err(|_| "disk-play worker panic".to_owned())??;
        client_result
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emit_string(out: &mut Vec<u8>, value: &str) {
        out.extend_from_slice(&(value.len() as u64).to_le_bytes());
        out.extend_from_slice(value.as_bytes());
    }
    fn synthetic_gguf() -> Vec<u8> {
        let names = tensor_names();
        let mut out = Vec::new();
        out.extend_from_slice(b"GGUF");
        out.extend_from_slice(&3_u32.to_le_bytes());
        out.extend_from_slice(&(TENSORS as u64).to_le_bytes());
        out.extend_from_slice(&2_u64.to_le_bytes());
        for (key, value) in [
            ("general.architecture", "asolaria-four-axes-rime"),
            ("asolaria.prism", "p=1000081 g=7 w=951846 k=27 verified"),
        ] {
            emit_string(&mut out, key);
            out.extend_from_slice(&8_u32.to_le_bytes());
            emit_string(&mut out, value);
        }
        for (n, name) in names.iter().enumerate() {
            emit_string(&mut out, name);
            out.extend_from_slice(&1_u32.to_le_bytes());
            out.extend_from_slice(&(GLYPHS as u64).to_le_bytes());
            out.extend_from_slice(&0_u32.to_le_bytes());
            out.extend_from_slice(&((n * 128) as u64).to_le_bytes());
        }
        out.resize(aligned(out.len()).expect("alignment"), 0);
        for n in 0..TENSORS {
            for i in 0..GLYPHS {
                // Delta at the zero glyph transforms to the all-ones Rime vector.
                let word = if n % 2 != 0 || i == 0 { 0x3f80_0000_u32 } else { 0 };
                out.extend_from_slice(&word.to_le_bytes());
            }
            out.resize(aligned(out.len()).expect("alignment"), 0);
        }
        out
    }
    #[test]
    fn exact_binary_word_decoding() {
        assert_eq!(exact_integer(0), Ok(0));
        assert_eq!(exact_integer(0x3f80_0000), Ok(1));
        assert_eq!(exact_integer(0x4040_0000), Ok(3));
        assert!(exact_integer(0x3f00_0000).is_err());
        assert!(exact_integer(0xbf80_0000).is_err());
        assert!(exact_integer(0x7f80_0000).is_err());
    }
    #[test]
    fn disk_read_three_paths_and_corruption_rejection() {
        let path = std::env::temp_dir().join(format!("pnp-gguf-test-{}.gguf", std::process::id()));
        let original = synthetic_gguf();
        fs::write(&path, &original).expect("synthetic file");
        let digest = hash_bytes(&original);
        let store = DiskRime::open_checked(&path, &digest, original.len()).expect("open");
        assert_eq!(store.tensor_count(), TENSORS);
        assert_eq!(store.verify_three_paths(), Ok(12 * GLYPHS));
        assert_eq!(store.read(0, 0, 1), Ok(1));
        assert!(store.read(0, 1, 1).is_err());
        assert!(store.read(24, 0, 1).is_err());
        assert!(store.read(0, 27, 1).is_err());
        let record = request(0, 0).expect("request");
        assert_eq!(store.reply(&record).expect("reply")[3..7], 1_u32.to_le_bytes());
        assert_eq!(loopback_batch(&store, &[(0, 0), (0, 1)]), Ok(vec![1, 0]));
        let mut false_address = record;
        false_address[3] ^= 1;
        assert!(store.reply(&false_address).is_err());
        let mut corrupted = original.clone();
        let last = corrupted.len() - 2;
        corrupted[last] = 9;
        fs::write(&path, &corrupted).expect("mutation");
        assert!(store.verify_source().is_err());
        fs::write(&path, &original).expect("restore");
        assert_eq!(store.verify_three_paths(), Ok(12 * GLYPHS));
        fs::remove_file(&path).expect("cleanup");
    }
}
