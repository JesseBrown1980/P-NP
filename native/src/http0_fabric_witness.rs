//! HTTP-0 BACKEND-ONLY function-room witness for continuous temporal streams.
//! Separate Rust worker process, TCP loopback transport, independent arithmetic
//! decoder, autonomous eight-byte room state, and source-free event recovery.
//! The historical Host8/HTTP-0 machine is a provenance reference, not required
//! as a connected disk or an HTTP browser page.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};

use crate::freeze::fischer_address;

const COUNT: usize = 8_000;
const PRIME: u64 = 1_000_081;
const GENERATOR: u64 = 7;
const TOP: u64 = 0xffff_ffff;
const HALF: u64 = 0x8000_0000;
const QUARTER: u64 = 0x4000_0000;
const THREE_QUARTERS: u64 = 0xc000_0000;

struct Model {
    level: usize,
    prior: i64,
    average: i64,
    slope: i64,
    table: Vec<(u32, u32)>,
}
fn next_u64(bytes: &[u8], cursor: &mut usize) -> Result<i64, String> {
    if bytes.len().saturating_sub(*cursor) < 8 {
        return Err("truncated independent model parameter".into());
    }
    let value = i64::from_le_bytes(bytes[*cursor..*cursor + 8].try_into()
        .map_err(|_| "model value width")?);
    *cursor += 8;
    Ok(value)
}
fn model(bytes: &[u8]) -> Result<Model, String> {
    if bytes.len() < 10 || bytes[0] != 1 || bytes[1] > 4 {
        return Err("invalid independent model header".into());
    }
    let level = usize::from(bytes[1]);
    let expected = [10, 26, 34, 226, 658][level];
    if bytes.len() != expected {
        return Err("model snapshot extent changed".into());
    }
    let mut cursor = 2;
    let prior = next_u64(bytes, &mut cursor)?;
    if !(1..=9_999).contains(&prior) {
        return Err("independent prior outside domain".into());
    }
    let (average, slope) = if level == 1 {
        let mean = next_u64(bytes, &mut cursor)?;
        let coefficient = next_u64(bytes, &mut cursor)?;
        if !(0..=20_000).contains(&coefficient) {
            return Err("independent EMA slope out of range".into());
        }
        (mean, coefficient)
    } else { (0, 0) };
    let mut table = Vec::new();
    if level >= 2 {
        let cells = [3, 27, 81][level - 2];
        for _ in 0..cells {
            let count = u32::from_le_bytes(bytes[cursor..cursor + 4]
                .try_into().map_err(|_| "bucket count")?);
            let yes = u32::from_le_bytes(bytes[cursor + 4..cursor + 8]
                .try_into().map_err(|_| "bucket success")?);
            if yes > count || count > 8_000 {
                return Err("fabric model bucket inconsistent".into());
            }
            table.push((count, yes));
            cursor += 8;
        }
    }
    if cursor != bytes.len() {
        return Err("trailing independent model bytes".into());
    }
    Ok(Model { level, prior, average, slope, table })
}
struct History { recent: [u8; 5], ema: i64 }
impl History {
    fn new() -> Self { Self { recent: [0; 5], ema: 5_000 } }
    fn probability(&self, m: &Model) -> i64 {
        let h = &self.recent;
        let a = usize::from(h[0]) + usize::from(h[1]);
        let b = usize::from(h[1]) + usize::from(h[2]);
        let c = usize::from(h[2]) + usize::from(h[3]);
        let d = usize::from(h[3]) + usize::from(h[4]);
        let p = match m.level {
            0 => m.prior,
            1 => m.prior + m.slope * (self.ema - m.average) / 10_000,
            2..=4 => {
                let idx = [a, a + 3*b + 9*c, a + 3*b + 9*c + 27*d][m.level - 2];
                let (n, successes) = m.table[idx];
                let smooth = [12_i64, 48, 96][m.level - 2];
                (i64::from(successes) * 10_000 + smooth * m.prior)
                    / (i64::from(n) + smooth)
            }
            _ => unreachable!("model constructor validates level"),
        };
        p.clamp(1, 9_999)
    }
    fn push(&mut self, bit: u8) {
        for i in (1..5).rev() { self.recent[i] = self.recent[i - 1]; }
        self.recent[0] = bit;
        self.ema = (15 * self.ema + 10_000 * i64::from(bit)) / 16;
    }
}
struct InBits<'a> { input: &'a [u8], cursor: usize }
impl InBits<'_> {
    fn take(&mut self) -> u64 {
        let i = self.cursor;
        self.cursor += 1;
        if i / 8 >= self.input.len() { 0 }
        else { u64::from((self.input[i / 8] >> (7 - i % 8)) & 1) }
    }
}
#[derive(Default)]
struct OutBits { bytes: Vec<u8>, partial: u8, count: u8 }
impl OutBits {
    fn put(&mut self, bit: u8) {
        self.partial = (self.partial << 1) | (bit & 1);
        self.count += 1;
        if self.count == 8 {
            self.bytes.push(self.partial);
            self.partial = 0;
            self.count = 0;
        }
    }
    fn done(mut self) -> Vec<u8> {
        if self.count != 0 {
            self.bytes.push(self.partial << (8 - self.count));
        }
        while self.bytes.len() < 4 { self.bytes.push(0); }
        self.bytes
    }
}
fn discharge(writer: &mut OutBits, bit: u8, deferred: &mut usize) {
    writer.put(bit);
    for _ in 0..*deferred { writer.put(1 - bit); }
    *deferred = 0;
}
fn reference_reencode(events: &[u8], params: &Model) -> Result<Vec<u8>, String> {
    let mut state = History::new();
    let mut lo = 0_u64;
    let mut hi = TOP;
    let mut delayed = 0_usize;
    let mut encoded = OutBits::default();
    for &event in events {
        if event > 1 { return Err("nonbinary replay".into()); }
        let probability = state.probability(params) as u64;
        let threshold = lo + (hi - lo + 1) * (10_000 - probability) / 10_000;
        if event == 0 { hi = threshold - 1; } else { lo = threshold; }
        loop {
            if hi < HALF {
                discharge(&mut encoded, 0, &mut delayed);
            } else if lo >= HALF {
                discharge(&mut encoded, 1, &mut delayed);
                lo -= HALF;
                hi -= HALF;
            } else if lo >= QUARTER && hi < THREE_QUARTERS {
                delayed += 1;
                lo -= QUARTER;
                hi -= QUARTER;
            } else { break; }
            lo = (lo << 1) & TOP;
            hi = ((hi << 1) | 1) & TOP;
        }
        state.push(event);
    }
    delayed += 1;
    discharge(&mut encoded, u8::from(lo >= QUARTER), &mut delayed);
    Ok(encoded.done())
}
fn decode_independently(payload: &[u8], snapshot: &[u8])
    -> Result<Vec<u8>, String>
{
    if payload.len() < 4 || payload.len() > 65_536 {
        return Err("independent range body size invalid".into());
    }
    let params = model(snapshot)?;
    let mut bits = InBits { input: payload, cursor: 0 };
    let mut code = 0_u64;
    for _ in 0..32 { code = (code << 1) | bits.take(); }
    let mut lower = 0_u64;
    let mut upper = TOP;
    let mut history = History::new();
    let mut out = Vec::with_capacity(COUNT);
    for _ in 0..COUNT {
        if code < lower || code > upper {
            return Err("independent interval rejected code".into());
        }
        let probability = history.probability(&params) as u64;
        let border = lower + (upper - lower + 1) * (10_000 - probability) / 10_000;
        let bit = if code < border { 0_u8 } else { 1_u8 };
        if bit == 0 { upper = border - 1; }
        else { lower = border; }
        loop {
            if upper < HALF {
            } else if lower >= HALF {
                code -= HALF;
                lower -= HALF;
                upper -= HALF;
            } else if lower >= QUARTER && upper < THREE_QUARTERS {
                code -= QUARTER;
                lower -= QUARTER;
                upper -= QUARTER;
            } else { break; }
            lower = (lower << 1) & TOP;
            upper = ((upper << 1) | 1) & TOP;
            code = ((code << 1) & TOP) | bits.take();
        }
        out.push(bit);
        history.push(bit);
    }
    if reference_reencode(&out, &params)? != payload {
        return Err("independent coder canonical replay failed".into());
    }
    Ok(out)
}
fn independent_address(frame: usize, arity: usize) -> Option<u64> {
    if !matches!(arity, 3 | 27 | 81) || frame >= 54 { return None; }
    let lane = frame % arity;
    let position = frame / arity + lane / 27;
    let glyph = lane % 27;
    let mut point = 1_u64;
    for _ in 0..(position * 27 + glyph) {
        point = (point * GENERATOR) % PRIME;
    }
    Some(point)
}
fn headers_and_body(stream: &mut TcpStream)
    -> Result<(String, Vec<u8>), String>
{
    let mut reader = BufReader::new(stream);
    let mut start = String::new();
    reader.read_line(&mut start).map_err(|e|e.to_string())?;
    if start.len() > 256 || !start.ends_with("\r\n") {
        return Err("invalid HTTP-0 backend request line".into());
    }
    let mut length = None;
    let mut terminated = false;
    for _ in 0..32 {
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e|e.to_string())?;
        if line == "\r\n" { terminated = true; break; }
        if line.len() > 256 || !line.ends_with("\r\n") {
            return Err("noncanonical HTTP-0 header".into());
        }
        if let Some(raw) = line.strip_prefix("Content-Length: ") {
            let number: usize = raw.trim().parse().map_err(|_| "bad body length")?;
            if length.replace(number).is_some() { return Err("double content length".into()); }
        }
    }
    if !terminated { return Err("HTTP-0 header terminator absent".into()); }
    let count = length.ok_or("body length missing")?;
    if count > 66_000 { return Err("HTTP-0 body too large".into()); }
    let mut body = vec![0; count];
    reader.read_exact(&mut body).map_err(|e|e.to_string())?;
    Ok((start.trim_end().to_owned(), body))
}
fn respond(stream: &mut TcpStream, status: &str, body: &[u8])
    -> Result<(), String>
{
    let header = format!("HTTP/1.1 {status}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    stream.write_all(header.as_bytes()).map_err(|e|e.to_string())?;
    stream.write_all(body).map_err(|e|e.to_string())?;
    stream.flush().map_err(|e|e.to_string())
}
fn execute_request(path: &str, body: &[u8], states: &mut BTreeMap<u64, u64>)
    -> Result<Vec<u8>, String>
{
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() != 5 || parts[0] != "" || parts[1] != "room" {
        return Err("room dispatch path invalid".into());
    }
    let arity: usize = parts[2].parse().map_err(|_| "room arity")?;
    let frame: usize = parts[3].parse().map_err(|_| "room index")?;
    let given: u64 = parts[4].parse().map_err(|_| "room address")?;
    if Some(given) != independent_address(frame, arity) {
        return Err("Fischer room address not independently reproduced".into());
    }
    if body.len() < 7 || !matches!(body[0], 1 | 2) {
        return Err("room message version missing".into());
    }
    let sn = usize::from(u16::from_le_bytes([body[1],body[2]]));
    let pn = u32::from_le_bytes(body[3..7].try_into().map_err(|_| "payload length")?) as usize;
    if body.len() != 7 + sn + pn || sn > 658 {
        return Err("room wire lengths inconsistent".into());
    }
    let result = if body[0] == 1 {
        decode_independently(&body[7 + sn..], &body[7..7 + sn])?
    } else {
        if sn != 0 || pn != COUNT / 8 {
            return Err("bitpack frame has invalid dimensions".into());
        }
        (0..COUNT).map(|index| {
            (body[7 + index / 8] >> (7 - index % 8)) & 1
        }).collect::<Vec<_>>()
    };
    if result.len() != COUNT { return Err("fabric reconstructed incorrect count".into()); }
    // Exactly one eight-byte state cell per visited function room. This is an
    // execution checksum, not an archive of the original event stream.
    let state = states.entry(given).or_insert(0);
    let mut fold = *state ^ given;
    for chunk in result.chunks(32) {
        for &event in chunk {
            fold = fold.rotate_left(5).wrapping_mul(0x9e37_79b1).wrapping_add(u64::from(event));
        }
    }
    *state = fold;
    Ok(result)
}
pub fn serve() -> Result<(), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e|e.to_string())?;
    let port = listener.local_addr().map_err(|e|e.to_string())?.port();
    println!("PNP_HTTP0_FABRIC_PORT|port={port}|status=READY|json=0");
    std::io::stdout().flush().map_err(|e|e.to_string())?;
    let mut states: BTreeMap<u64, u64> = BTreeMap::new();
    let mut calls = 0_usize;
    for accepted in listener.incoming() {
        let mut stream = accepted.map_err(|e|e.to_string())?;
        let handled = headers_and_body(&mut stream);
        match handled {
            Ok((first, _)) if first == "POST /stop HTTP/1.1" => {
                let body = format!("PNP_HTTP0_STOP|calls={calls}|rooms={}|status=PASS|json=0\n",states.len());
                respond(&mut stream, "200 OK", body.as_bytes())?;
                break;
            }
            Ok((first, body)) => {
                let path = first.strip_prefix("POST ")
                    .and_then(|s|s.strip_suffix(" HTTP/1.1"));
                let outcome = path.ok_or("only native POST is accepted")
                    .and_then(|route|execute_request(route, &body, &mut states));
                match outcome {
                    Ok(recovered) => {
                        calls += 1;
                        respond(&mut stream, "200 OK", &recovered)?;
                    }
                    Err(error) => { respond(&mut stream, "422 Unprocessable Content", error.as_bytes())?; }
                }
            }
            Err(error) => { respond(&mut stream, "400 Bad Request",error.as_bytes())?; }
        }
    }
    Ok(())
}
pub struct FabricClient { child: Child, port: u16 }
impl FabricClient {
    pub fn start() -> Result<Self, String> {
        let mut child = Command::new(std::env::current_exe().map_err(|e|e.to_string())?)
            .arg("stream-fabric-serve")
            .stdout(Stdio::piped()).spawn().map_err(|e|e.to_string())?;
        let stdout = child.stdout.take().ok_or("child stdout unavailable")?;
        let mut reader = BufReader::new(stdout);
        let mut welcome = String::new();
        reader.read_line(&mut welcome).map_err(|e|e.to_string())?;
        if !welcome.starts_with("PNP_HTTP0_FABRIC_PORT|port=") {
            let _ = child.kill();
            let _ = child.wait();
            return Err("HTTP-0 native worker did not report ready".into());
        }
        let port = welcome.strip_prefix("PNP_HTTP0_FABRIC_PORT|port=")
            .and_then(|x|x.split('|').next())
            .ok_or("missing worker port")?.parse::<u16>()
            .map_err(|_| "worker port invalid")?;
        Ok(Self { child, port })
    }
    fn request(&self, path: &str, payload: &[u8]) -> Result<(String,Vec<u8>), String> {
        let mut socket = TcpStream::connect(("127.0.0.1", self.port))
            .map_err(|e|format!("HTTP-0 room transport: {e}"))?;
        let request = format!("POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",payload.len());
        socket.write_all(request.as_bytes()).map_err(|e|e.to_string())?;
        socket.write_all(payload).map_err(|e|e.to_string())?;
        let mut reader = BufReader::new(socket);
        let mut status = String::new();
        reader.read_line(&mut status).map_err(|e|e.to_string())?;
        let mut size = None;
        for _ in 0..16 {
            let mut line = String::new();
            reader.read_line(&mut line).map_err(|e|e.to_string())?;
            if line == "\r\n" { break; }
            if let Some(raw) = line.strip_prefix("Content-Length: ") {
                size = Some(raw.trim().parse::<usize>().map_err(|_|"HTTP-0 response length")?);
            }
        }
        let n = size.ok_or("HTTP-0 response not framed")?;
        if n > 16_384 { return Err("HTTP-0 excessive response".into()); }
        let mut out = vec![0_u8;n];
        reader.read_exact(&mut out).map_err(|e|e.to_string())?;
        let mut trailing = Vec::new();
        reader.read_to_end(&mut trailing).map_err(|e|e.to_string())?;
        if !trailing.is_empty() { return Err("HTTP-0 trailing response".into()); }
        Ok((status,out))
    }
    pub fn decode(&self, arity: usize, frame: usize, codec: u8,
                  snapshot: &[u8], payload: &[u8]) -> Result<Vec<u8>, String> {
        let lane = frame % arity;
        let address = fischer_address((lane % 27) as u64,
            (frame / arity + lane / 27) as u64).ok_or("Fischer routing impossible")?;
        let mut body = Vec::with_capacity(snapshot.len() + payload.len() + 7);
        if !matches!(codec, 1 | 2) { return Err("unknown fabric codec".into()); }
        body.push(codec);
        body.extend_from_slice(&u16::try_from(snapshot.len())
            .map_err(|_| "snapshot too large")?.to_le_bytes());
        body.extend_from_slice(&u32::try_from(payload.len())
            .map_err(|_| "payload too large")?.to_le_bytes());
        body.extend_from_slice(snapshot);
        body.extend_from_slice(payload);
        let (status, result) = self.request(&format!("/room/{arity}/{frame}/{address}"), &body)?;
        if status != "HTTP/1.1 200 OK\r\n" {
            return Err(format!("fabric room rejected reconstructed payload: {}",
                String::from_utf8_lossy(&result)));
        }
        if result.len() != COUNT {
            return Err("fabric response event length invalid".into());
        }
        Ok(result)
    }
    pub fn corrupted_model_rejected(&self, snapshot: &[u8], payload: &[u8])
        -> Result<bool, String>
    {
        let mut body = Vec::with_capacity(snapshot.len()+payload.len()+7);
        body.push(1);
        body.extend_from_slice(&u16::try_from(snapshot.len())
            .map_err(|_| "snapshot size")?.to_le_bytes());
        body.extend_from_slice(&u32::try_from(payload.len())
            .map_err(|_| "payload size")?.to_le_bytes());
        let mut altered = snapshot.to_vec();
        altered[0] = 9;
        body.extend_from_slice(&altered);
        body.extend_from_slice(payload);
        let (status, _) = self.request("/room/3/0/1", &body)?;
        Ok(status == "HTTP/1.1 422 Unprocessable Content\r\n")
    }
    pub fn stop(mut self) -> Result<usize, String> {
        let (status, report) = self.request("/stop", &[])?;
        if status != "HTTP/1.1 200 OK\r\n" { return Err("fabric stop failed".into()); }
        let receipt = String::from_utf8(report).map_err(|_| "fabric stop text invalid")?;
        let number = receipt.strip_prefix("PNP_HTTP0_STOP|calls=")
            .and_then(|x|x.split('|').next()).ok_or("stop receipt absent")?
            .parse::<usize>().map_err(|_|"stop count invalid")?;
        if !self.child.wait().map_err(|e|e.to_string())?.success() {
            return Err("independent native witness process failed".into());
        }
        Ok(number)
    }
}
impl Drop for FabricClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_address_matches_frozen_fischer_all_54_frames() {
        for arity in [3,27,81] {
            for frame in 0..54 {
                let lane=frame%arity;
                assert_eq!(independent_address(frame,arity), fischer_address(
                    (lane%27) as u64,(frame/arity+lane/27) as u64));
            }
        }
        assert_eq!(independent_address(54,3),None);
    }
    #[test]
    fn independent_arithmetic_roundtrip_and_negative_snapshot() {
        let snapshot = {
            let mut b=vec![1,0];
            b.extend_from_slice(&5_000_i64.to_le_bytes());
            b
        };
        let params=model(&snapshot).unwrap();
        let values:Vec<u8>=(0..COUNT).map(|i|u8::from(i%7<3)).collect();
        let compressed=reference_reencode(&values,&params).unwrap();
        assert_eq!(decode_independently(&compressed,&snapshot).unwrap(),values);
        let mut corrupted=snapshot;
        corrupted[0]=3;
        assert!(decode_independently(&compressed,&corrupted).is_err());
    }
}
