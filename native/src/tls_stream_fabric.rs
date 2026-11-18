//! Fair, fully self-contained 54-frame comparison: one continuous causal
//! arithmetic stream versus ordinary bitpacking with the identical frame
//! metadata; the separately executed HTTP-0 witness checks BOTH encodings.
//! Frozen PR45 model choices are reused, never retuned on held-out bits.
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines)]

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::http0_fabric_witness::FabricClient;
use crate::sha256::hash_bytes;
use crate::tls_entropy_omnibit as old;

const MAGIC: &[u8; 8] = b"PNPSTF46";
const FRAMES: usize = 54;
const EVENTS: usize = 8_000;
const HEADER_HBP: &str = "HBPHEADER|schema=pnp.http0.stream.fabric.v1|transport=BACKEND_ONLY|source=PR45_FROZEN|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HEADER_HBI: &str = "HBIHEADER|schema=pnp.http0.stream.fabric.index.v1|for=PNP_TLS_STREAM_FABRIC.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";

#[derive(Clone, Debug, Eq, PartialEq)]
struct Frame {
    ids: [u8; 4],
    events: Vec<u8>,
    payload: Vec<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stats {
    pub bitpack_bytes: usize,
    pub range_bytes: usize,
    pub former_hybrid_bytes: usize,
    pub model_bytes: usize,
    pub frames: usize,
    pub http0_calls: usize,
    pub fabric_rooms: usize,
    pub decoder_bytes: usize,
    pub wall_ns_by_arity: [u128; 3],
}

fn unpack_bits(body: &[u8]) -> Result<Vec<u8>, String> {
    if body.len() != EVENTS / 8 {
        return Err("bitpack body size is not canonical".into());
    }
    Ok((0..EVENTS).map(|index| {
        (body[index / 8] >> (7 - index % 8)) & 1
    }).collect())
}
fn pack_bits(bits: &[u8]) -> Result<Vec<u8>, String> {
    if bits.len() != EVENTS || bits.iter().any(|bit| *bit > 1) {
        return Err("bitpack source outside binary domain".into());
    }
    let mut body = vec![0_u8; EVENTS / 8];
    for (index, bit) in bits.iter().enumerate() {
        body[index / 8] |= bit << (7 - index % 8);
    }
    Ok(body)
}
fn frame_populations(source: &[u8], snapshots: &[Vec<u8>])
    -> Result<(Vec<Frame>, Vec<Frame>), String>
{
    let bases = old::source_event_frames(source)?;
    if bases.len() != 18 || snapshots.len() != 9 {
        return Err("frozen PR45 source or predictor population changed".into());
    }
    let mut packed = Vec::with_capacity(FRAMES);
    let mut ranged = Vec::with_capacity(FRAMES);
    for (mode, rep, shuffled, events) in bases {
        for route in 0..3_u8 {
            let id = [mode, rep, shuffled, route];
            let fit = &snapshots[(mode as usize) * 3 + rep as usize];
            packed.push(Frame { ids: id, events: events.clone(), payload: pack_bits(&events)? });
            ranged.push(Frame { ids: id, events: events.clone(),
                payload: old::encode_continuous(&events, fit)? });
        }
    }
    if packed.len() != FRAMES || ranged.len() != FRAMES {
        return Err("54 matched source frames required".into());
    }
    Ok((packed, ranged))
}
fn put_u16(bytes: &mut Vec<u8>, value: usize) -> Result<(), String> {
    bytes.extend_from_slice(&u16::try_from(value)
        .map_err(|_| "stream header u16 overflow")?.to_le_bytes());
    Ok(())
}
fn put_u32(bytes: &mut Vec<u8>, value: usize) -> Result<(), String> {
    bytes.extend_from_slice(&u32::try_from(value)
        .map_err(|_| "stream header u32 overflow")?.to_le_bytes());
    Ok(())
}
fn u16_at(data: &[u8], at: &mut usize) -> Result<usize, String> {
    if data.len().saturating_sub(*at) < 2 { return Err("truncated u16".into()); }
    let count = usize::from(u16::from_le_bytes([data[*at],data[*at+1]]));
    *at += 2;
    Ok(count)
}
fn u32_at(data: &[u8], at: &mut usize) -> Result<usize, String> {
    if data.len().saturating_sub(*at) < 4 { return Err("truncated u32".into()); }
    let n = u32::from_le_bytes(data[*at..*at+4]
        .try_into().map_err(|_| "word length")?) as usize;
    *at += 4;
    Ok(n)
}
fn assemble(codec: u8, frames: &[Frame], snapshots: &[Vec<u8>])
    -> Result<Vec<u8>, String>
{
    if !matches!(codec, 0 | 1) || frames.len() != FRAMES
        || (codec == 1 && snapshots.len() != 9)
    {
        return Err("noncanonical archive population".into());
    }
    let mut blob = MAGIC.to_vec();
    blob.extend_from_slice(&[1,codec,if codec == 0 {0} else {9}]);
    if codec == 1 {
        for state in snapshots {
            put_u16(&mut blob,state.len())?;
            blob.extend_from_slice(state);
        }
    }
    put_u16(&mut blob,FRAMES)?;
    for (index, frame) in frames.iter().enumerate() {
        let mode = (index / 18) as u8;
        let rep = ((index / 6) % 3) as u8;
        let shuffle = ((index / 3) % 2) as u8;
        let route = (index % 3) as u8;
        if frame.ids != [mode,rep,shuffle,route] || frame.events.len() != EVENTS {
            return Err("frame identity or source count changed".into());
        }
        blob.extend_from_slice(&frame.ids);
        put_u32(&mut blob, EVENTS)?;
        put_u32(&mut blob, frame.payload.len())?;
        blob.extend_from_slice(&frame.payload);
    }
    blob.extend_from_slice(hash_bytes(&blob).as_bytes());
    Ok(blob)
}
fn extract(archive: &[u8]) -> Result<Vec<Frame>, String> {
    if archive.len() < MAGIC.len()+3+2+64 || !archive.starts_with(MAGIC) {
        return Err("stream archive magic missing".into());
    }
    let (body, end_hash) = archive.split_at(archive.len()-64);
    if hash_bytes(body).as_bytes() != end_hash {
        return Err("stream archive checksum failed".into());
    }
    if body[8] != 1 || body[9] > 1
        || body[10] != if body[9] == 0 {0} else {9}
    {
        return Err("stream archive version or model count invalid".into());
    }
    let codec = body[9];
    let mut at = 11;
    let mut snapshots = Vec::new();
    if codec == 1 {
        for _ in 0..9 {
            let n = u16_at(body, &mut at)?;
            if n > body.len().saturating_sub(at) {
                return Err("stream archive model truncated".into());
            }
            snapshots.push(body[at..at+n].to_vec());
            at += n;
        }
    }
    if u16_at(body, &mut at)? != FRAMES {
        return Err("stream archive wrong frame count".into());
    }
    let mut restored = Vec::with_capacity(FRAMES);
    for index in 0..FRAMES {
        if body.len().saturating_sub(at) < 12 {
            return Err("stream frame truncated".into());
        }
        let ids = [body[at],body[at+1],body[at+2],body[at+3]];
        at += 4;
        if ids != [(index / 18) as u8,((index / 6) % 3) as u8,
                   ((index / 3) % 2) as u8,(index % 3) as u8] {
            return Err("stream archive frame order corrupted".into());
        }
        let n = u32_at(body, &mut at)?;
        let length = u32_at(body, &mut at)?;
        if n != EVENTS || length > body.len().saturating_sub(at) {
            return Err("stream archive payload length invalid".into());
        }
        let payload = body[at..at+length].to_vec();
        at += length;
        let events = if codec == 0 { unpack_bits(&payload)? }
        else { old::decode_continuous(&payload,&snapshots[
            (usize::from(ids[0])) * 3 + usize::from(ids[1])
        ])? };
        restored.push(Frame { ids,events,payload });
    }
    if at != body.len() { return Err("stream trailing archive bytes".into()); }
    Ok(restored)
}
fn originals_equal(expected: &[Frame],actual: &[Frame])->Result<(),String>{
    if expected.len()!=FRAMES || actual.len()!=FRAMES {return Err("frame count changed".into());}
    for (index,(a,b)) in expected.iter().zip(actual).enumerate(){
        if a.ids!=b.ids || a.events!=b.events || a.payload!=b.payload{
            return Err(format!("replay changed canonical frame {index}"));
        }
    }
    Ok(())
}
fn receipt_row(hbp: &mut String, hbi: &mut String,
               sequence: usize, kind: &str, id: &str,
               fields: &str, last: &mut String) {
    let original = format!("{kind}|seq={sequence}|id={id}|{fields}|previous_hash={last}");
    let this_hash = hash_bytes(original.as_bytes());
    let next = hash_bytes(format!("{last}|{this_hash}").as_bytes());
    let line = format!("{original}|receipt_sha={this_hash}|hash={next}|json=0\n");
    hbi.push_str(&format!("HBI|seq={sequence}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={next}|json=0\n",
        hbp.len(),line.len(),hash_bytes(line.as_bytes()),&this_hash[..16]));
    hbp.push_str(&line);
    *last = next;
}
fn restored_bytes(packed: &[Frame]) -> Result<Vec<u8>, String>{
    let mut data = b"PNP-HTTP0-STREAM-RESTORED-v1\0".to_vec();
    put_u16(&mut data,FRAMES)?;
    for frame in packed {
        data.extend_from_slice(&frame.ids);
        put_u32(&mut data,frame.events.len())?;
        data.extend_from_slice(&frame.events);
    }
    Ok(data)
}
type Bundle = (Vec<u8>,Vec<u8>,Vec<u8>,Vec<u8>,String,String,String);
fn make_bundle(prior: &Path) -> Result<(Bundle, Stats), String> {
    let source = fs::read(prior.join("PNP_TLS_STOCHASTIC_TRACE.bin"))
        .map_err(|e|format!("frozen source: {e}"))?;
    let former = fs::read(prior.join("PNP_TLS_ENTROPY_SELECTED.bin"))
        .map_err(|e|format!("accepted entropy: {e}"))?;
    let snapshots = old::frozen_model_snapshots(&former)?;
    let (packed,ranged) = frame_populations(&source, &snapshots)?;
    let archive_bitpack = assemble(0,&packed,&[])?;
    let archive_range = assemble(1,&ranged,&snapshots)?;
    originals_equal(&packed,&extract(&archive_bitpack)?)?;
    originals_equal(&ranged,&extract(&archive_range)?)?;
    let binary_length = fs::metadata(std::env::current_exe().map_err(|e|e.to_string())?)
        .map_err(|e|e.to_string())?.len() as usize;
    let snapshot_charge = snapshots.iter().map(|x|x.len()+2).sum::<usize>();
    let mut routed = FabricClient::start()?;
    let mut times = [0_u128;3];
    let mut verified_calls=0;
    for (lane,arity) in [3,27,81].into_iter().enumerate() {
        let start = Instant::now();
        for frame_index in 0..FRAMES {
            let bit = &packed[frame_index];
            let found = routed.decode(arity,frame_index,2,&[],&bit.payload)?;
            if found != bit.events {
                return Err(format!("independent HTTP-0 bitpack witness failed at {arity}/{frame_index}"));
            }
            verified_calls+=1;
            let range = &ranged[frame_index];
            let snapshot = &snapshots[usize::from(range.ids[0])*3+usize::from(range.ids[1])];
            let reconstructed=routed.decode(arity,frame_index,1,snapshot,&range.payload)?;
            if reconstructed!=range.events {
                return Err(format!("independent HTTP-0 continuous witness failed at {arity}/{frame_index}"));
            }
            verified_calls+=1;
        }
        times[lane]=start.elapsed().as_nanos();
    }
    if !routed.corrupted_model_rejected(&snapshots[0],&ranged[0].payload)? {
        return Err("HTTP-0 third seat accepted corrupted model".into());
    }
    let worker_total=routed.stop()?;
    if worker_total!=verified_calls || worker_total!=324{
        return Err("HTTP-0 third seat lost or fabricated calls".into());
    }
    let restored=restored_bytes(&packed)?;
    let source_sha=hash_bytes(&source);
    let previous_sha=hash_bytes(&former);
    let pack_sha=hash_bytes(&archive_bitpack);
    let range_sha=hash_bytes(&archive_range);
    let restored_sha=hash_bytes(&restored);
    let mut hbp=HEADER_HBP.to_owned();
    let mut hbi=HEADER_HBI.to_owned();
    let mut last=hash_bytes(b"PNP-HTTP0-BACKEND-STREAM-FABRIC-GENESIS");
    receipt_row(&mut hbp,&mut hbi,1,"STREAM_SOURCE","frozen",
        &format!("events_unique=144000|frames=54|routes=3|old_archive_sha256={previous_sha}|original_event_sha256={source_sha}|predictor=PR45_FROZEN_VALIDATION_ONLY|transport=HTTP0_BACKEND_LOOPBACK|browser=NONE|physical_drives=NOT_REQUIRED"),
        &mut last);
    receipt_row(&mut hbp,&mut hbi,2,"STREAM_ARM","bitpack",
        &format!("archive_bytes={}|frame_bytes=12_PER_8000|bitpacking=MSB|models=0|sha256={pack_sha}|decode=EXACT|all_54_framed=YES",
            archive_bitpack.len()),&mut last);
    receipt_row(&mut hbp,&mut hbi,3,"STREAM_ARM","continuous_range",
        &format!("archive_bytes={}|frame_bytes=12_PER_8000|fixed_model_snapshots=9|model_bytes={snapshot_charge}|sha256={range_sha}|prediction=PAST_ONLY|coder_resets=54|decode=EXACT",
            archive_range.len()),&mut last);
    for (i,arity) in [3,27,81].into_iter().enumerate(){
        receipt_row(&mut hbp,&mut hbi,4+i,"STREAM_FABRIC",
            &format!("n{arity}"),
            &format!("arity={arity}|calls=108|original_events_reconstructed=864000|room_state_bytes_each=8|fischer_address=INDEPENDENT_RECOMPUTE|decoder=SEPARATE_NATIVE_PROCESS|source_generator=NOT_SENT|original_bytes=CHECKED|model_tamper=REJECTED|timing=SEPARATE"),
            &mut last);
    }
    receipt_row(&mut hbp,&mut hbi,7,"STREAM_SUMMARY","all",
        &format!("bitpack_fully_framed_bytes={}|range_fully_framed_bytes={}|old_hybrid_bytes={}|range_minus_bitpack_bytes={}|decoder_bytes={binary_length}|range_plus_binary_bytes={}|bitpack_plus_binary_bytes={}|all_fabric_calls=324|original_shas=VERIFIED|restored_sha256={restored_sha}|hrm_checkpoint=NOT_LOADED|fischer_role=ADDRESS_ROUTING_NOT_PREDICTION|frame_count=54",
            archive_bitpack.len(),archive_range.len(),former.len(),
            archive_range.len() as i64 - archive_bitpack.len() as i64,
            archive_range.len()+binary_length,archive_bitpack.len()+binary_length),
        &mut last);
    let prefix=format!("STREAM_FABRIC_ANCHOR|rows=7|old_archive_sha256={previous_sha}|source_sha256={source_sha}|bitpack_sha256={pack_sha}|range_sha256={range_sha}|restored_sha256={restored_sha}|hbp_sha256={}|hbi_sha256={}|last_hash={last}|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes()));
    let anchor=format!("{prefix}|sha256={}|json=0\n",hash_bytes(prefix.as_bytes()));
    let stats=Stats{
        bitpack_bytes:archive_bitpack.len(),range_bytes:archive_range.len(),
        former_hybrid_bytes:former.len(),model_bytes:snapshot_charge,frames:FRAMES,
        http0_calls:verified_calls,fabric_rooms:81,decoder_bytes:binary_length,
        wall_ns_by_arity:times,
    };
    Ok(((archive_bitpack,archive_range,restored,hbp.into_bytes(),
         hbi,anchor,format!("{previous_sha}|{source_sha}")),stats))
}
fn write_file(path:&Path,bytes:&[u8])->Result<(),String>{
    let mut file=File::create(path).map_err(|e|e.to_string())?;
    file.write_all(bytes).map_err(|e|e.to_string())?;
    file.sync_all().map_err(|e|e.to_string())
}
pub fn attest(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((bitpack,range,recovered,hbp,hbi,anchor,_),stats)=make_bundle(prior)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,data) in [
        ("PNP_TLS_STREAM_BITPACK.bin",bitpack.as_slice()),
        ("PNP_TLS_STREAM_RANGE.bin",range.as_slice()),
        ("PNP_TLS_STREAM_RESTORED.bin",recovered.as_slice()),
        ("PNP_TLS_STREAM_FABRIC.hbp",hbp.as_slice()),
        ("PNP_TLS_STREAM_FABRIC.hbi",hbi.as_bytes()),
        ("PNP_TLS_STREAM_FABRIC.anchor",anchor.as_bytes()),
    ] { write_file(&out.join(name),data)?; }
    Ok(stats)
}
pub fn verify(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((bitpack,range,recovered,hbp,hbi,anchor,_),stats)=make_bundle(prior)?;
    for (name,expected) in [
        ("PNP_TLS_STREAM_BITPACK.bin",bitpack.as_slice()),
        ("PNP_TLS_STREAM_RANGE.bin",range.as_slice()),
        ("PNP_TLS_STREAM_RESTORED.bin",recovered.as_slice()),
        ("PNP_TLS_STREAM_FABRIC.hbp",hbp.as_slice()),
        ("PNP_TLS_STREAM_FABRIC.hbi",hbi.as_bytes()),
        ("PNP_TLS_STREAM_FABRIC.anchor",anchor.as_bytes()),
    ] {
        if fs::read(out.join(name)).map_err(|e|e.to_string())?!=expected{
            return Err(format!("HTTP-0 fabric archive altered: {name}"));
        }
    }
    Ok(stats)
}
pub fn verify_archive(path:&Path)->Result<(usize,usize),String>{
    let bytes=fs::read(path).map_err(|e|e.to_string())?;
    let frames=extract(&bytes)?;
    Ok((frames.len(),frames.iter().map(|x|x.events.len()).sum()))
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn strict_bitpacking_and_framing_negative_controls(){
        let events:Vec<u8>=(0..EVENTS).map(|x|u8::from(x%3==0)).collect();
        let bits=pack_bits(&events).unwrap();
        assert_eq!(bits.len(),1000);
        assert_eq!(unpack_bits(&bits).unwrap(),events);
        let mut fixtures=Vec::new();
        for index in 0..FRAMES{
            fixtures.push(Frame{ids:[(index/18)as u8,((index/6)%3)as u8,
                ((index/3)%2)as u8,(index%3)as u8],events:events.clone(),payload:bits.clone()});
        }
        let archived=assemble(0,&fixtures,&[]).unwrap();
        assert_eq!(archived.len(),54_725);
        assert_eq!(extract(&archived).unwrap(),fixtures);
        let mut damaged=archived.clone();
        damaged[100]^=1;
        assert!(extract(&damaged).is_err());
        assert!(extract(&archived[..archived.len()-1]).is_err());
    }
}
