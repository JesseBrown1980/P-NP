//! Exact integer temporal entropy × original 40-operator hybrid.
//! Original PR #43 trained estimates and PR #44 inverse are reused verbatim.
//! All model snapshots, per-block selectors, payloads and integrity trailer
//! are serialized. The standalone decoder needs no source or train events.
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines)]

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::{sha256::hash_bytes, tls_omnibit_gc as omni, tls_stochastic as stochastic};

const MAGIC: &[u8; 8] = b"PNPTLE45";
const SOURCE_MAGIC: &[u8] = b"PNP-TLS-STOCHASTIC-v1\0";
const RESTORED_MAGIC: &[u8] = b"PNP-TLS-ENTROPY-RESTORED-v1\0";
const OWNED: &str = "PNP_TLS_ENTROPY_GULP_OWNED.tmp";
const HBP_HEADER: &str = "HBPHEADER|schema=pnp.tls.entropy.omnibit.v1|scope=SYNTHETIC|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HBI_HEADER: &str = "HBIHEADER|schema=pnp.tls.entropy.index.v1|for=PNP_TLS_ENTROPY.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const EVENTS: usize = 8_000;
const ROWS: usize = 1 + 9 + 54 + 9 + 3 + 4 + 1;

#[derive(Clone)]
struct Case {
    mode: u8,
    rep: u8,
    train: Vec<u8>,
    valid: Vec<u8>,
    held: Vec<u8>,
    shuffle: Vec<u8>,
}
#[derive(Clone)]
struct Model {
    choice: usize,
    fit: stochastic::Fit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Decoded {
    mode: u8,
    rep: u8,
    shuffle: u8,
    route: u8,
    events: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counts {
    pub frames: usize,
    pub rows: usize,
    pub archive_bytes: usize,
    pub restored_bytes: usize,
    pub models_bytes: usize,
    pub range_blocks: usize,
    pub omega_blocks: usize,
    pub original_trials: usize,
    pub decoder_bytes: usize,
}
type Bundle = (String, String, String, Vec<u8>, Vec<u8>);

fn u16_read(bytes: &[u8], at: &mut usize) -> Result<usize, String> {
    if bytes.len().saturating_sub(*at) < 2 { return Err("truncated u16".into()); }
    let n = u16::from_le_bytes([bytes[*at], bytes[*at + 1]]) as usize;
    *at += 2;
    Ok(n)
}
fn u32_read(bytes: &[u8], at: &mut usize) -> Result<usize, String> {
    if bytes.len().saturating_sub(*at) < 4 { return Err("truncated u32".into()); }
    let n = u32::from_le_bytes(bytes[*at..*at + 4].try_into().map_err(|_| "bad u32")?) as usize;
    *at += 4;
    Ok(n)
}
fn put_u16(out: &mut Vec<u8>, n: usize) -> Result<(), String> {
    out.extend_from_slice(&u16::try_from(n).map_err(|_| "u16 length overflow")?.to_le_bytes());
    Ok(())
}
fn put_u32(out: &mut Vec<u8>, n: usize) -> Result<(), String> {
    out.extend_from_slice(&u32::try_from(n).map_err(|_| "u32 length overflow")?.to_le_bytes());
    Ok(())
}
fn cases(bytes: &[u8]) -> Result<Vec<Case>, String> {
    if !bytes.starts_with(SOURCE_MAGIC) { return Err("untrusted stochastic source magic".into()); }
    let mut at = SOURCE_MAGIC.len();
    let mut out = Vec::with_capacity(9);
    for mode in 0..3_u8 {
        for rep in 0..3_u8 {
            if bytes.len().saturating_sub(at) < 2 || bytes[at..at + 2] != [mode, rep] {
                return Err("stochastic source ordering changed".into());
            }
            at += 2;
            let mut fields = Vec::with_capacity(4);
            for expected in [8_000, 4_000, 8_000, 8_000] {
                let n = u32_read(bytes, &mut at)?;
                if n != expected || bytes.len().saturating_sub(at) < n {
                    return Err("original train/validation/heldout source length wrong".into());
                }
                let source = bytes[at..at + n].to_vec();
                if source.iter().any(|&b| b > 1) { return Err("nonbinary source event".into()); }
                fields.push(source);
                at += n;
            }
            if fields[2].iter().filter(|&&x| x == 1).count()
                != fields[3].iter().filter(|&&x| x == 1).count() {
                return Err("count-preserving shuffle invalid".into());
            }
            out.push(Case {
                mode, rep,
                train: fields.remove(0), valid: fields.remove(0),
                held: fields.remove(0), shuffle: fields.remove(0),
            });
        }
    }
    if at != bytes.len() { return Err("trailing source event data".into()); }
    Ok(out)
}
fn model_bytes(model: &Model) -> Vec<u8> {
    let mut out = vec![1_u8, model.choice as u8];
    out.extend_from_slice(&model.fit.prior.to_le_bytes());
    match model.choice {
        0 => {}
        1 => {
            out.extend_from_slice(&model.fit.mean_x.to_le_bytes());
            out.extend_from_slice(&model.fit.beta.to_le_bytes());
        }
        2..=4 => {
            for bucket in &model.fit.tables[model.choice - 2] {
                out.extend_from_slice(&bucket.count.to_le_bytes());
                out.extend_from_slice(&bucket.success.to_le_bytes());
            }
        }
        _ => unreachable!("validated model index"),
    }
    out
}
fn parse_model(input: &[u8]) -> Result<Model, String> {
    if input.len() < 10 || input[0] != 1 || input[1] > 4 {
        return Err("invalid model version or capacity".into());
    }
    let choice = input[1] as usize;
    let expected = match choice { 0 => 10, 1 => 26, 2 => 34, 3 => 226, _ => 658 };
    if input.len() != expected { return Err("incomplete model snapshot".into()); }
    let mut at = 2;
    let mut next_i64 = || {
        let v = i64::from_le_bytes(input[at..at + 8].try_into().expect("length checked"));
        at += 8;
        v
    };
    let prior = next_i64();
    let mut mean_x = 0_i64;
    let mut beta = 0_i64;
    if choice == 1 {
        mean_x = next_i64();
        beta = next_i64();
    }
    if !(1..10_000).contains(&prior) || !(0..=20_000).contains(&beta) {
        return Err("model probability parameters outside accepted range".into());
    }
    let mut tables = [
        vec![stochastic::Bucket::default(); 3],
        vec![stochastic::Bucket::default(); 27],
        vec![stochastic::Bucket::default(); 81],
    ];
    if choice >= 2 {
        let index = choice - 2;
        for item in &mut tables[index] {
            item.count = u32::from_le_bytes(input[at..at + 4].try_into().expect("length checked"));
            item.success = u32::from_le_bytes(input[at + 4..at + 8].try_into().expect("length checked"));
            if item.success > item.count || item.count > 8_000 {
                return Err("invalid trained table count".into());
            }
            at += 8;
        }
    }
    let result = Model {
        choice,
        fit: stochastic::Fit { prior, mean_x, beta, tables },
    };
    if model_bytes(&result) != input { return Err("noncanonical trained model state".into()); }
    Ok(result)
}
fn hard_bit(m: &Model, ctx: &stochastic::Context) -> Result<u8, String> {
    Ok(u8::from(stochastic::predict(m.choice, &m.fit, ctx)? >= 5_000))
}

#[derive(Default)]
struct BitWriter { data: Vec<u8>, partial: u8, used: u8 }
impl BitWriter {
    fn bit(&mut self, bit: u8) {
        self.partial = (self.partial << 1) | (bit & 1);
        self.used += 1;
        if self.used == 8 {
            self.data.push(self.partial);
            self.partial = 0;
            self.used = 0;
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.used != 0 { self.data.push(self.partial << (8 - self.used)); }
        while self.data.len() < 4 { self.data.push(0); }
        self.data
    }
}
struct BitReader<'a> { data: &'a [u8], at: usize }
impl BitReader<'_> {
    fn bit(&mut self) -> u64 {
        let n = self.at;
        self.at += 1;
        if n / 8 >= self.data.len() { 0 } else {
            u64::from((self.data[n / 8] >> (7 - n % 8)) & 1)
        }
    }
}
const TOP: u64 = 0xffff_ffff;
const HALF: u64 = 0x8000_0000;
const QUARTER: u64 = 0x4000_0000;
const THREE_Q: u64 = 0xc000_0000;

fn emit(w: &mut BitWriter, bit: u8, follow: &mut usize) {
    w.bit(bit);
    for _ in 0..*follow { w.bit(1 - bit); }
    *follow = 0;
}
fn range_encode(events: &[u8], m: &Model, ctx: &mut stochastic::Context) -> Result<Vec<u8>, String> {
    let mut low = 0_u64;
    let mut high = TOP;
    let mut pending = 0_usize;
    let mut bits = BitWriter::default();
    for &event in events {
        if event > 1 { return Err("nonbinary event in arithmetic coder".into()); }
        let p = stochastic::predict(m.choice, &m.fit, ctx)? as u64;
        let mid = low + (high - low + 1) * (10_000 - p) / 10_000;
        if event == 0 { high = mid - 1; } else { low = mid; }
        loop {
            if high < HALF {
                emit(&mut bits, 0, &mut pending);
            } else if low >= HALF {
                emit(&mut bits, 1, &mut pending);
                low -= HALF;
                high -= HALF;
            } else if low >= QUARTER && high < THREE_Q {
                pending += 1;
                low -= QUARTER;
                high -= QUARTER;
            } else { break; }
            low = (low << 1) & TOP;
            high = ((high << 1) | 1) & TOP;
        }
        ctx.observe(event);
    }
    pending += 1;
    if low < QUARTER { emit(&mut bits, 0, &mut pending); }
    else { emit(&mut bits, 1, &mut pending); }
    Ok(bits.finish())
}
fn range_decode(data: &[u8], count: usize, m: &Model, ctx: &mut stochastic::Context) -> Result<Vec<u8>, String> {
    if data.len() < 4 || count == 0 || count > 8_000 {
        return Err("invalid range coder block extent".into());
    }
    let old = *ctx;
    let mut reader = BitReader { data, at: 0 };
    let mut code = 0_u64;
    for _ in 0..32 { code = (code << 1) | reader.bit(); }
    let mut low = 0_u64;
    let mut high = TOP;
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        if code < low || code > high { return Err("range interval contains no code".into()); }
        let p = stochastic::predict(m.choice, &m.fit, ctx)? as u64;
        let mid = low + (high - low + 1) * (10_000 - p) / 10_000;
        let event = u8::from(code >= mid);
        if event == 0 { high = mid - 1; } else { low = mid; }
        loop {
            if high < HALF {
            } else if low >= HALF {
                code -= HALF;
                low -= HALF;
                high -= HALF;
            } else if low >= QUARTER && high < THREE_Q {
                code -= QUARTER;
                low -= QUARTER;
                high -= QUARTER;
            } else { break; }
            low = (low << 1) & TOP;
            high = ((high << 1) | 1) & TOP;
            code = ((code << 1) & TOP) | reader.bit();
        }
        result.push(event);
        ctx.observe(event);
    }
    let mut check = old;
    if range_encode(&result, m, &mut check)? != data {
        return Err("noncanonical, damaged or truncated range payload".into());
    }
    Ok(result)
}
struct Block { method: u8, payload: Vec<u8>, original_trials: usize }
fn choose_block(events: &[u8], route: u8, m: &Model,
                ctx: &mut stochastic::Context) -> Result<Block, String> {
    let before = *ctx;
    let mut range_ctx = before;
    let range = range_encode(events, m, &mut range_ctx)?;
    let mut residual_ctx = before;
    let mut residual = Vec::with_capacity(events.len());
    for &event in events {
        residual.push(event ^ hard_bit(m, &residual_ctx)?);
        residual_ctx.observe(event);
    }
    if residual_ctx.ema != range_ctx.ema { return Err("coder context diverged".into()); }
    let banks = omni::banks_for(&residual, route)?;
    let mut best = (usize::MAX, u8::MAX, Vec::new());
    for op in 0..40_u8 {
        let candidate = omni::encode_block(op, &banks)?;
        if (candidate.len(), op) < (best.0, best.1) {
            best = (candidate.len(), op, candidate);
        }
    }
    *ctx = range_ctx;
    if range.len() <= best.0 {
        Ok(Block { method: 0, payload: range, original_trials: 40 })
    } else {
        Ok(Block { method: 1, payload: best.2, original_trials: 40 })
    }
}
fn decode_block(method: u8, data: &[u8], route: u8, count: usize,
                m: &Model, ctx: &mut stochastic::Context) -> Result<Vec<u8>, String> {
    if method == 0 { return range_decode(data, count, m, ctx); }
    if method != 1 { return Err("invalid entropy/Omnibit selector".into()); }
    let mut at = 0;
    let residual = omni::decode_block(data, &mut at, route)?;
    if at != data.len() || residual.len() != count { return Err("incomplete Omnibit residual".into()); }
    let op = *data.first().ok_or("missing original operator ID")?;
    if omni::encode_block(op, &omni::banks_for(&residual, route)?)? != data {
        return Err("noncanonical operator residual".into());
    }
    let mut events = Vec::with_capacity(count);
    for bit in residual {
        let event = bit ^ hard_bit(m, ctx)?;
        events.push(event);
        ctx.observe(event);
    }
    Ok(events)
}
fn push_block(out: &mut Vec<u8>, b: &Block) -> Result<(), String> {
    out.push(b.method);
    put_u16(out, b.payload.len())?;
    out.extend_from_slice(&b.payload);
    Ok(())
}
fn validation_cost(data: &[u8], fit: &stochastic::Fit, choice: usize) -> Result<usize, String> {
    let model = Model { choice, fit: fit.clone() };
    let mut total = model_bytes(&model).len() + 4;
    for route in 0..3_u8 {
        let mut ctx = stochastic::Context::new();
        let size = 20 * omni::group_size(route)?;
        total += 12;
        for window in data.chunks_exact(size) {
            let block = choose_block(window, route, &model, &mut ctx)?;
            total += 3 + block.payload.len();
        }
        if data.len() % size != 0 { return Err("validation block geometry changed".into()); }
    }
    Ok(total)
}
fn fitted(case: &Case) -> Result<(Model, usize, [usize; 5]), String> {
    let fit = stochastic::fit(&case.train)?;
    let historical = stochastic::select(&stochastic::score(&case.valid, &fit)?);
    let mut costs = [0_usize; 5];
    for (choice, dest) in costs.iter_mut().enumerate() {
        *dest = validation_cost(&case.valid, &fit, choice)?;
    }
    let chosen = (0..5).min_by_key(|&i| (costs[i], i)).ok_or("no models")?;
    Ok((Model { choice: chosen, fit }, historical, costs))
}
fn serialize_models(models: &[(u8, u8, Model)]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    for (mode, rep, model) in models {
        bytes.extend_from_slice(&[*mode, *rep]);
        let snapshot = model_bytes(model);
        put_u16(&mut bytes, snapshot.len())?;
        bytes.extend_from_slice(&snapshot);
    }
    Ok(bytes)
}
fn decoded_bytes(rows: &[Decoded]) -> Result<Vec<u8>, String> {
    let mut out = RESTORED_MAGIC.to_vec();
    put_u16(&mut out, rows.len())?;
    for row in rows {
        out.extend_from_slice(&[row.mode, row.rep, row.shuffle, row.route]);
        put_u32(&mut out, row.events.len())?;
        out.extend_from_slice(&row.events);
    }
    Ok(out)
}
fn decode_archive(input: &[u8]) -> Result<Vec<Decoded>, String> {
    if input.len() < MAGIC.len() + 64 || !input.starts_with(MAGIC) {
        return Err("entropy archive header missing".into());
    }
    let (data, checksum) = input.split_at(input.len() - 64);
    if hash_bytes(data).as_bytes() != checksum {
        return Err("entropy archive digest corrupted".into());
    }
    let mut at = MAGIC.len();
    let mut models = Vec::with_capacity(9);
    for mode in 0..3_u8 {
        for rep in 0..3_u8 {
            if data.len().saturating_sub(at) < 4 || data[at..at + 2] != [mode, rep] {
                return Err("trained model ordering corrupted".into());
            }
            at += 2;
            let n = u16_read(data, &mut at)?;
            if n > data.len().saturating_sub(at) { return Err("model bytes truncated".into()); }
            models.push(parse_model(&data[at..at + n])?);
            at += n;
        }
    }
    if u16_read(data, &mut at)? != 54 { return Err("entropy frame count wrong".into()); }
    let mut decoded = Vec::with_capacity(54);
    for mode in 0..3_u8 {
        for rep in 0..3_u8 {
            let model = &models[(mode as usize) * 3 + rep as usize];
            for shuffle in 0..2_u8 {
                for route in 0..3_u8 {
                    if data.len().saturating_sub(at) < 12 || data[at..at + 4] != [mode, rep, shuffle, route] {
                        return Err("entropy frame identity corrupted".into());
                    }
                    at += 4;
                    let n = u32_read(data, &mut at)?;
                    let body_len = u32_read(data, &mut at)?;
                    if n != EVENTS || body_len > data.len().saturating_sub(at) {
                        return Err("entropy frame length invalid".into());
                    }
                    let end = at + body_len;
                    let block_n = 20 * omni::group_size(route)?;
                    let mut events = Vec::with_capacity(EVENTS);
                    let mut ctx = stochastic::Context::new();
                    for _ in 0..EVENTS / block_n {
                        if end.saturating_sub(at) < 3 { return Err("truncated selector".into()); }
                        let method = data[at];
                        at += 1;
                        let len = u16_read(data, &mut at)?;
                        if len > end.saturating_sub(at) { return Err("truncated entropy block".into()); }
                        events.extend(decode_block(method, &data[at..at + len], route, block_n, model, &mut ctx)?);
                        at += len;
                    }
                    if at != end || events.len() != EVENTS { return Err("trailing entropy block bytes".into()); }
                    decoded.push(Decoded { mode, rep, shuffle, route, events });
                }
            }
        }
    }
    if at != data.len() { return Err("trailing entropy frames".into()); }
    Ok(decoded)
}
pub fn decode_to_file(source: &Path, destination: &Path) -> Result<usize, String> {
    let encoded = fs::read(source).map_err(|e|e.to_string())?;
    let result = decoded_bytes(&decode_archive(&encoded)?)?;
    let mut f = File::create(destination).map_err(|e|e.to_string())?;
    f.write_all(&result).map_err(|e|e.to_string())?;
    f.sync_all().map_err(|e|e.to_string())?;
    Ok(result.len())
}
fn program() -> Result<(usize, String), String> {
    let path = std::env::current_exe().map_err(|e|e.to_string())?;
    let data = fs::read(path).map_err(|e|e.to_string())?;
    Ok((data.len(), hash_bytes(&data)))
}
fn make_bundle(original: &Path, prior: &Path) -> Result<(Bundle, Counts), String> {
    let old = omni::verify(original, prior, prior)?;
    if old.archive_bytes != 294_615 || old.original_operator_trials != 381_600 {
        return Err("original accepted 40-operator baseline changed".into());
    }
    let source = fs::read(prior.join("PNP_TLS_STOCHASTIC_TRACE.bin")).map_err(|e|e.to_string())?;
    let cases = cases(&source)?;
    let source_hbp = hash_bytes(&fs::read(prior.join("PNP_TLS_STOCHASTIC.hbp")).map_err(|e|e.to_string())?);
    let omni_sha = hash_bytes(&fs::read(prior.join("PNP_TLS_OMNI40_SELECTED.bin")).map_err(|e|e.to_string())?);
    let (decoder_bytes, decoder_sha) = program()?;
    let mut models = Vec::with_capacity(9);
    let mut choices = Vec::with_capacity(9);
    for case in &cases {
        let (model, historical, costs) = fitted(case)?;
        choices.push((historical, costs));
        models.push((case.mode, case.rep, model));
    }
    let serialized = serialize_models(&models)?;
    let mut archive = MAGIC.to_vec();
    archive.extend_from_slice(&serialized);
    put_u16(&mut archive, 54)?;
    let mut hbp = HBP_HEADER.to_owned();
    let mut hbi = HBI_HEADER.to_owned();
    let mut previous = hash_bytes(b"PNP-TLS-ENTROPY-OMNIBIT-2026");
    let mut seq = 0_usize;
    let mut push = |kind: &str, id: &str, fields: &str| {
        seq += 1;
        let bare = format!("{kind}|seq={seq}|id={id}|original_stochastic_sha256={source_hbp}|original_omni40_sha256={omni_sha}|decoder_sha256={decoder_sha}|{fields}|previous_hash={previous}");
        let digest = hash_bytes(bare.as_bytes());
        let next = hash_bytes(format!("{previous}|{digest}").as_bytes());
        let line = format!("{bare}|receipt_sha={digest}|hash={next}|json=0\n");
        hbi.push_str(&format!("HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={next}|json=0\n",
            hbp.len(), line.len(), hash_bytes(line.as_bytes()), &digest[..16]));
        hbp.push_str(&line);
        previous = next;
    };
    push("TLS_ENTROPY_SOURCE", "frozen", &format!("source=PR43_HELDOUT|original_catalog=40|original_binary_sha256={omni_sha}|training=PR43_TRAIN_ONLY|models=0,1,3,27,81|events_unique=144000|frames=54|decoder_program_bytes={decoder_bytes}|bitpack_18_unframed=18000|bitpack_54_unframed=54000|baseline_omni40_54_bytes=294615"));
    for (index, (mode, rep, model)) in models.iter().enumerate() {
        let (historical, costs) = choices[index];
        push("TLS_ENTROPY_MODEL", &format!("m{mode}_r{rep}"),
            &format!("mode={mode}|rep={rep}|selected_capacity={}|historical_validation_brier_choice={historical}|validation_hybrid_charges={},{},{},{},{}|model_snapshot_bytes={}|selection=VALIDATION_ONLY",
                [0,1,3,27,81][model.choice], costs[0],costs[1],costs[2],costs[3],costs[4],
                model_bytes(model).len() + 4));
    }
    let mut range_blocks = 0_usize;
    let mut omega_blocks = 0_usize;
    let mut original_trials = 0_usize;
    let mut by_mode = [0_usize; 3];
    let mut paired = [[[0_usize; 3]; 2]; 9];
    let mut restored = Vec::with_capacity(54);
    for case in &cases {
        let model = &models[(case.mode as usize) * 3 + case.rep as usize].2;
        for shuffle in 0..2_u8 {
            let events = if shuffle == 0 { &case.held } else { &case.shuffle };
            for route in 0..3_u8 {
                let offset = archive.len();
                let mut body = Vec::new();
                let mut ctx = stochastic::Context::new();
                let mut route_range = 0;
                let mut route_omni = 0;
                let group = 20 * omni::group_size(route)?;
                for block in events.chunks_exact(group) {
                    let chosen = choose_block(block, route, model, &mut ctx)?;
                    original_trials += chosen.original_trials;
                    if chosen.method == 0 { range_blocks += 1; route_range += 1; }
                    else { omega_blocks += 1; route_omni += 1; }
                    push_block(&mut body, &chosen)?;
                }
                if events.len() % group != 0 { return Err("heldout block width changed".into()); }
                archive.extend_from_slice(&[case.mode, case.rep, shuffle, route]);
                put_u32(&mut archive, events.len())?;
                put_u32(&mut archive, body.len())?;
                archive.extend_from_slice(&body);
                let size = archive.len() - offset;
                paired[(case.mode as usize) * 3 + case.rep as usize][shuffle as usize][route as usize] = size;
                by_mode[case.mode as usize] += size;
                push("TLS_ENTROPY_FRAME", &format!("m{}_r{}_s{shuffle}_a{route}",case.mode,case.rep),
                    &format!("mode={}|rep={}|shuffle={shuffle}|route={route}|source_events={EVENTS}|frame_bytes={size}|frame_sha256={}|range_blocks={route_range}|original_operator_blocks={route_omni}|block_trials={}|decoded_source_sha256={}|selected_model={}|selector_and_residual=CHARGED",
                        case.mode,case.rep,hash_bytes(&archive[offset..]), (EVENTS/group)*40,hash_bytes(events),model.choice));
                restored.push(Decoded {
                    mode: case.mode, rep: case.rep, shuffle, route, events: events.clone(),
                });
            }
        }
    }
    for case in &cases {
        let pair = &paired[(case.mode as usize) * 3 + case.rep as usize];
        push("TLS_ENTROPY_PAIR", &format!("m{}_r{}",case.mode,case.rep),
            &format!("mode={}|rep={}|heldout_route_bytes={},{},{}|shuffle_route_bytes={},{},{}|event_count_preserving=YES|training=DISJOINT|selector=FROZEN_BEFORE_HELDOUT",
                case.mode,case.rep,pair[0][0],pair[0][1],pair[0][2],pair[1][0],pair[1][1],pair[1][2]));
    }
    for (m, bytes) in by_mode.iter().enumerate() {
        push("TLS_ENTROPY_REGIME", &format!("{m}"), &format!("mode={m}|framed_bytes={bytes}|streams=6|routes_each=3"));
    }
    for (name, committed, retained, replay, allowed) in [
        ("summary_only", true, false, false, true),
        ("exact_missing", true, false, false, false),
        ("exact_replayed", true, true, true, true),
        ("not_committed", false, true, true, false),
    ] {
        let kind = if name == "summary_only" || name == "not_committed" {
            crate::tls_memory::Retention::SummaryOnly
        } else { crate::tls_memory::Retention::ExactReplayRequired };
        if crate::tls_memory::may_discard(kind, committed, retained, replay) != allowed {
            return Err("original temporal retention policy violated".into());
        }
        push("TLS_ENTROPY_GC", name, &format!("owned_scratch_only=YES|protected_source=RETAINED|eligible={allowed}|approval=AUTOMATIC"));
    }
    let before_footer = archive.clone();
    archive.extend_from_slice(hash_bytes(&before_footer).as_bytes());
    if decode_archive(&archive)? != restored { return Err("entropy archive independent inverse changed events".into()); }
    let recovered = decoded_bytes(&restored)?;
    let models_bytes = serialized.len();
    push("TLS_ENTROPY_SUMMARY", "accepted", &format!(
        "frames=54|distinct_streams=18|blocks={}|range_blocks={range_blocks}|omnibit_blocks={omega_blocks}|original_operator_trials={original_trials}|model_snapshot_bytes={models_bytes}|archive_bytes={}|restored_bytes={}|bitpack18_unframed=18000|bitpack54_unframed=54000|accepted_omni40_bytes=294615|full_decoder_bytes={decoder_bytes}|framed_payload_and_binary={}|external_receipts=SEPARATE|decoded_without_generator=YES|scope=SYNTHETIC",
        range_blocks + omega_blocks, archive.len(), recovered.len(), archive.len() + decoder_bytes
    ));
    if seq != ROWS { return Err("entropy evidence row count changed".into()); }
    let root = format!(
        "TLS_ENTROPY_ANCHOR|rows={ROWS}|original_stochastic_sha256={source_hbp}|original_omni40_sha256={omni_sha}|archive_sha256={}|restored_sha256={}|hbp_sha256={}|hbi_sha256={}|decoder_sha256={decoder_sha}|last_hash={previous}|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(&archive), hash_bytes(&recovered), hash_bytes(hbp.as_bytes()), hash_bytes(hbi.as_bytes())
    );
    let anchor = format!("{root}|checksum={}|json=0\n", hash_bytes(root.as_bytes()));
    let stats = Counts {
        frames: 54, rows: ROWS, archive_bytes: archive.len(), restored_bytes: recovered.len(),
        models_bytes, range_blocks, omega_blocks, original_trials, decoder_bytes,
    };
    Ok(((hbp, hbi, anchor, archive, recovered), stats))
}
fn write_file(path: &Path, data: &[u8]) -> Result<(), String> {
    let mut f = File::create(path).map_err(|e|e.to_string())?;
    f.write_all(data).map_err(|e|e.to_string())?;
    f.sync_all().map_err(|e|e.to_string())
}
pub fn attest(original: &Path, prior: &Path, out: &Path) -> Result<Counts, String> {
    let ((hbp,hbi,anchor,archive,recovered), stats) = make_bundle(original,prior)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name, body) in [
        ("PNP_TLS_ENTROPY.hbp", hbp.as_bytes()),
        ("PNP_TLS_ENTROPY.hbi", hbi.as_bytes()),
        ("PNP_TLS_ENTROPY.anchor", anchor.as_bytes()),
        ("PNP_TLS_ENTROPY_SELECTED.bin", archive.as_slice()),
        ("PNP_TLS_ENTROPY_RESTORED.bin", recovered.as_slice()),
    ] {
        write_file(&out.join(name), body)?;
    }
    Ok(stats)
}
pub fn verify(original: &Path, prior: &Path, out: &Path) -> Result<Counts, String> {
    let ((hbp,hbi,anchor,archive,recovered), stats) = make_bundle(original,prior)?;
    for (name, expected) in [
        ("PNP_TLS_ENTROPY.hbp",hbp.as_bytes()),
        ("PNP_TLS_ENTROPY.hbi",hbi.as_bytes()),
        ("PNP_TLS_ENTROPY.anchor",anchor.as_bytes()),
        ("PNP_TLS_ENTROPY_SELECTED.bin",archive.as_slice()),
        ("PNP_TLS_ENTROPY_RESTORED.bin",recovered.as_slice()),
    ] {
        if fs::read(out.join(name)).map_err(|e|e.to_string())? != expected {
            return Err(format!("entropy archival witness differs: {name}"));
        }
    }
    if decoded_bytes(&decode_archive(&archive)?)? != recovered { return Err("independent entropy decode failed".into()); }
    Ok(stats)
}
pub fn timing(original: &Path, prior: &Path, out: &Path) -> Result<(Counts, u128), String> {
    let t = Instant::now();
    let stats = attest(original, prior, out)?;
    if verify(original, prior, out)? != stats { return Err("entropy repeat test changed".into()); }
    Ok((stats,t.elapsed().as_nanos()))
}
pub fn gulp_discard_owned(original: &Path, prior: &Path, out: &Path) -> Result<(usize,usize), String> {
    let stats = verify(original,prior,out)?;
    let old_source = fs::read(prior.join("PNP_TLS_STOCHASTIC_TRACE.bin")).map_err(|e|e.to_string())?;
    let mut owned = Vec::with_capacity(144_000);
    for case in cases(&old_source)? {
        owned.extend_from_slice(&case.held);
        owned.extend_from_slice(&case.shuffle);
    }
    if owned.len() != 144_000 { return Err("owned temporal scratch size wrong".into()); }
    let scratch = out.join(OWNED);
    let mut file = OpenOptions::new().create_new(true).write(true).open(&scratch)
        .map_err(|e|format!("scratch must be exclusively owned: {e}"))?;
    file.write_all(&owned).map_err(|e|e.to_string())?;
    file.sync_all().map_err(|e|e.to_string())?;
    drop(file);
    if fs::read(&scratch).map_err(|e|e.to_string())? != owned {
        return Err("owned scratch changed after durable write".into());
    }
    let archive = fs::read(out.join("PNP_TLS_ENTROPY_SELECTED.bin")).map_err(|e|e.to_string())?;
    let replay = decoded_bytes(&decode_archive(&archive)?)?;
    if fs::read(out.join("PNP_TLS_ENTROPY_RESTORED.bin")).map_err(|e|e.to_string())? != replay {
        return Err("decoder witness absent before GULP".into());
    }
    for name in ["PNP_TLS_ENTROPY_SELECTED.bin","PNP_TLS_ENTROPY_RESTORED.bin",
                 "PNP_TLS_ENTROPY.hbp","PNP_TLS_ENTROPY.hbi","PNP_TLS_ENTROPY.anchor"] {
        File::open(out.join(name)).map_err(|e|e.to_string())?.sync_all().map_err(|e|e.to_string())?;
    }
    fs::remove_file(&scratch).map_err(|e|e.to_string())?;
    if scratch.exists() || fs::read(prior.join("PNP_TLS_STOCHASTIC_TRACE.bin")).map_err(|e|e.to_string())? != old_source {
        return Err("source protection / owned GC not satisfied".into());
    }
    let raw = format!(
        "TLS_ENTROPY_GC_ACTION|deleted_owned_scratch_bytes={}|retained_archive_bytes={}|selected_sha256={}|restored_sha256={}|source_sha256={}|archive_sync_before_delete=PASS|scratch_absent=PASS|original_source=RETAINED|human_approval=NOT_REQUIRED",
        owned.len(),stats.archive_bytes,hash_bytes(&archive),hash_bytes(&replay),hash_bytes(&old_source)
    );
    write_file(&out.join("PNP_TLS_ENTROPY_GC.hbp"),
               format!("{raw}|sha256={}|json=0\n",hash_bytes(raw.as_bytes())).as_bytes())?;
    Ok((owned.len(),stats.archive_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_five_integer_models_roundtrip_without_source() {
        let training = (0..8_000).map(|n| u8::from(n % 7 == 0 || n % 11 == 0)).collect::<Vec<_>>();
        let fitted = stochastic::fit(&training).unwrap();
        let events = (0..160).map(|n| u8::from(n % 9 == 0 || n % 3 == 0)).collect::<Vec<_>>();
        for choice in 0..5 {
            let original = Model { choice, fit: fitted.clone() };
            let parsed = parse_model(&model_bytes(&original)).unwrap();
            let mut left = stochastic::Context::new();
            let encoded = range_encode(&events, &parsed, &mut left).unwrap();
            let mut right = stochastic::Context::new();
            assert_eq!(range_decode(&encoded, events.len(), &parsed, &mut right).unwrap(), events);
            let mut corrupt = encoded.clone();
            corrupt.push(0);
            assert!(range_decode(&corrupt, events.len(), &parsed, &mut stochastic::Context::new()).is_err());
            let mut a = stochastic::Context::new();
            let block = choose_block(&events, 2, &parsed, &mut a).unwrap();
            let mut b = stochastic::Context::new();
            assert_eq!(decode_block(block.method, &block.payload, 2, events.len(), &parsed, &mut b).unwrap(), events);
        }
    }
    #[test]
    fn reject_corrupt_framing_and_model() {
        let training = vec![0_u8;8_000];
        let fitted = stochastic::fit(&training).unwrap();
        let mut model = model_bytes(&Model { choice: 2, fit: fitted });
        model[0] = 9;
        assert!(parse_model(&model).is_err());
        assert!(decode_archive(b"wrong").is_err());
    }
}


/// Expose the four exact frozen segments with their original train/validation
/// boundary. The full-fabric temporal gate must never fit or select on heldout
/// or shuffled labels, and never regenerate the source event streams.
pub(crate) type TemporalSourceCase=(u8,u8,Vec<u8>,Vec<u8>,Vec<u8>,Vec<u8>);

pub(crate) fn source_temporal_cases(source:&[u8])
    ->Result<Vec<TemporalSourceCase>,String>
{
    cases(source).map(|cases|cases.into_iter()
        .map(|case|(case.mode,case.rep,case.train,case.valid,case.held,case.shuffle))
        .collect())
}

/// Historical PR45 source format, ordered canonically; used to give the new
/// frame benchmark the exact same 18 streams without a second generator.
pub(crate) type SourceEventFrame = (u8, u8, u8, Vec<u8>);

pub(crate) fn source_event_frames(source: &[u8])
    -> Result<Vec<SourceEventFrame>, String>
{
    let mut out = Vec::with_capacity(18);
    for case in cases(source)? {
        out.push((case.mode, case.rep, 0, case.held));
        out.push((case.mode, case.rep, 1, case.shuffle));
    }
    Ok(out)
}

/// Only read the nine previously selected, immutable serialized snapshots.
/// No validation or held-out optimization occurs in the streaming gate.
pub(crate) fn frozen_model_snapshots(archive: &[u8])
    -> Result<Vec<Vec<u8>>, String>
{
    if archive.len() < MAGIC.len() + 64 || !archive.starts_with(MAGIC) {
        return Err("original entropy archive header missing".to_owned());
    }
    let (body, digest) = archive.split_at(archive.len() - 64);
    if hash_bytes(body).as_bytes() != digest {
        return Err("original entropy archive checksum mismatch".to_owned());
    }
    // Validate every original frame and its predictor snapshot before reuse.
    if decode_archive(archive)?.len() != 54 {
        return Err("original entropy archive frame count changed".to_owned());
    }
    let mut at = MAGIC.len();
    let mut result = Vec::with_capacity(9);
    for mode in 0..3_u8 {
        for rep in 0..3_u8 {
            if body.len().saturating_sub(at) < 4 || body[at..at + 2] != [mode, rep] {
                return Err("original model identity changed".to_owned());
            }
            at += 2;
            let n = u16_read(body, &mut at)?;
            if body.len().saturating_sub(at) < n {
                return Err("original model was truncated".to_owned());
            }
            parse_model(&body[at..at + n])?;
            result.push(body[at..at + n].to_vec());
            at += n;
        }
    }
    Ok(result)
}

/// Previously accepted PR45 arithmetic coder, now with one causal context
/// and one termination across all 8,000 source events.
pub(crate) fn encode_continuous(events: &[u8], snapshot: &[u8])
    -> Result<Vec<u8>, String>
{
    if events.len() != EVENTS {
        return Err("continuous coder needs exactly 8000 events".to_owned());
    }
    let model = parse_model(snapshot)?;
    range_encode(events, &model, &mut stochastic::Context::new())
}

pub(crate) fn decode_continuous(payload: &[u8], snapshot: &[u8])
    -> Result<Vec<u8>, String>
{
    let model = parse_model(snapshot)?;
    range_decode(payload, EVENTS, &model, &mut stochastic::Context::new())
}

#[cfg(test)]
mod continuous_tests {
    use super::*;

    #[test]
    fn one_continuous_causal_stream_uses_exact_existing_models() {
        let training: Vec<u8> = (0..8_000)
            .map(|i| u8::from(i % 11 == 0 || i % 17 < 2)).collect();
        let fit = stochastic::fit(&training).expect("training corpus");
        let data: Vec<u8> = (0..8_000)
            .map(|i| u8::from(i % 5 < 2)).collect();
        for choice in 0..5 {
            let snapshot = model_bytes(&Model { choice, fit: fit.clone() });
            let encoded = encode_continuous(&data, &snapshot).expect("full stream");
            assert_eq!(decode_continuous(&encoded, &snapshot).unwrap(), data);
            let mut appended = encoded.clone();
            appended.push(0);
            assert!(decode_continuous(&appended, &snapshot).is_err());
        }
    }
}
