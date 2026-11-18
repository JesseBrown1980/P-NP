//! Original Omnibit source-rooted public fixture tournament across binary,
//! balanced ternary and mixed floor routing. This is a six-vantage proxy,
//! never a claim of three independent private-seat computations.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{
    binary_ternary_bridge::{self as bridge, FloorWord},
    sha256::hash_bytes,
    ternary::Trit,
};

const COMMIT: &str = "dad4647ad19a1b08ee90990c34aab1255f9a6046";
const SOURCE_PINS: &str = include_str!("../../policy/OMNIBIT_TOURNAMENT_PINS.hbi");
const FIXTURES: &str = include_str!("../fixtures/OMNIBIT_PUBLIC_TRITS.hbp");
const ARCHIVE_HEADER: &str = "HBPHEADER|schema=pnp.omnibit.public.tournament.v1|scope=PUBLIC_SYNTHETIC|seat_local_runs=0|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const INDEX_HEADER: &str = "HBIHEADER|schema=pnp.omnibit.public.index.v1|for=PNP_OMNIBIT.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const SOURCE_ENTRIES: [(&str, &str, &str); 5] = [
    ("architecture_contract","contracts/ARCHITECTURE.hbp","fcd2f1f40034525c6b9b90c6861cd2c27d7f7f4d"),
    ("tournament_record","provenance/receipts/tournament/OMNIBIT-TOURNAMENT-RECORD-2026-10-09.hbp","7f1385e2e8751479cffac776aee738dd14bbfb59"),
    ("operator_contract","provenance/receipts/operator-catalog/OPERATOR-CATALOG.hbp","e5c9d7e807a557d3b2fdd6689ec43a0aad5d06d6"),
    ("operator_source","reference/omega-operator-catalog-v0/omega_operator_catalog.rs","1535e158b21b1c2b9ece4dce84e0e9779498ec19"),
    ("gc_codec_contract","contracts/ACER-GLYPH-CODEC-V1.hbp","afa29fe3fd1d8eabfdcd5901c02cead7e8a47037"),
];
const SEATS: [&str; 3] = ["acer","liris","relic"];
const VIEWS: [&str; 2] = ["NORMAL","ANTI"];
const ARMS: [&str; 3] = ["BINARY","TERNARY","MIXED"];
const CASE_COUNT: usize = 20;
const TOURNEY_ROWS: usize = 3*2*3*CASE_COUNT;
const TOTAL_ROWS: usize = 5+TOURNEY_ROWS+3+1;

#[derive(Clone, Debug)]
struct Input { id: String, data: Vec<Trit> }
fn trit_to_digit(t: Trit) -> u8 {
    match t { Trit::Minus => 0, Trit::Zero => 1, Trit::Plus => 2 }
}
fn digit_to_trit(n: u8) -> Result<Trit, String> {
    match n { 0 => Ok(Trit::Minus),1 => Ok(Trit::Zero),2 => Ok(Trit::Plus),
        _ => Err("noncanonical ternary digit".to_owned()) }
}
fn fixture_inputs() -> Result<Vec<Input>,String> {
    let rest = FIXTURES.strip_prefix(
        "HBPHEADER|schema=pnp.omnibit.public.fixture.v1|scope=PUBLIC_SYNTHETIC|privacy=NO_SEAT_INPUT|json=0\n"
    ).ok_or("wrong Omnibit fixture schema")?;
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for line in rest.lines() {
        let mut parts = line.split('|');
        if parts.next() != Some("OMNIBIT_FIXTURE") {
            return Err("unknown Omnibit fixture record".to_owned());
        }
        let name = parts.next().and_then(|s|s.strip_prefix("id="))
            .ok_or("fixture name absent")?;
        let payload = parts.next().and_then(|s|s.strip_prefix("trits="))
            .ok_or("fixture trits absent")?;
        let declared = parts.next().and_then(|s|s.strip_prefix("length="))
            .ok_or("fixture length absent")?
            .parse::<usize>().map_err(|_|"fixture length invalid")?;
        if parts.next() != Some("source=DETERMINISTIC_SYNTHETIC")
            || parts.next() != Some("json=0") || parts.next().is_some()
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || !seen.insert(name)
            || !(3..=8).contains(&declared)
            || payload.len() != declared
        {
            return Err("invalid public synthetic fixture".to_owned());
        }
        let data = payload.chars()
            .map(|c| Trit::from_symbol(c).ok_or("unrecognized signed trit".to_owned()))
            .collect::<Result<Vec<_>,_>>()?;
        out.push(Input{id:name.to_owned(),data});
    }
    if out.len()!=CASE_COUNT
        || [3,5,6,7,8].iter().any(|n|out.iter().filter(|case|case.data.len()==*n).count()!=4)
    {
        return Err("frozen public fixture shape changed".to_owned());
    }
    Ok(out)
}
fn pin_original(root:&Path) -> Result<Vec<(String,String)>,String> {
    let mut check = "HBIHEADER|schema=pnp.omnibit.original.v1|seal=HBI-HBP-SHA-SH-HASH|json=0\n".to_owned();
    for (id,path,blob) in SOURCE_ENTRIES {
        check.push_str(&format!(
            "OMNIBIT_PIN|id={id}|commit={COMMIT}|path={path}|git_blob={blob}|json=0\n"
        ));
    }
    if SOURCE_PINS!=check {return Err("source pin manifest differs".to_owned());}
    let git=|args:&[&str]| -> Result<String,String> {
        let result=Command::new("git").arg("-C").arg(root).args(args)
            .output().map_err(|e|format!("source Git unavailable: {e}"))?;
        if !result.status.success(){return Err("original Omnibit source checkout unavailable".to_owned());}
        String::from_utf8(result.stdout).map(|s|s.trim().to_owned())
            .map_err(|_|"Git reply invalid UTF8".to_owned())
    };
    if git(&["rev-parse","HEAD"])?!=COMMIT {
        return Err("Omnibit source commit changed".to_owned());
    }
    let mut results=Vec::new();
    for (id,path,blob) in SOURCE_ENTRIES {
        if git(&["hash-object",path])?!=blob {
            return Err(format!("original Omnibit source changed: {id}"));
        }
        let bytes=fs::read(root.join(path)).map_err(|e|format!("source {id}: {e}"))?;
        let body=String::from_utf8_lossy(&bytes);
        let contract_ok=match id {
            "architecture_contract"=>body.contains("seats=ACER,LIRIS,RELIC|view_bindings=NORMAL,ANTI")
                && body.contains("implemented_recursive=64,256,1024,4096"),
            "tournament_record"=>body.contains("scoreboard=NOT_SCORED")
                && body.contains("winner=NONE"),
            "operator_contract"=>body.contains("operators=40|rnq=8|sector=12|radial_lens=20"),
            "operator_source"=>body.contains("pub const OPERATOR_COUNT: usize = 40;")
                && body.contains("pub fn invert_symbols("),
            "gc_codec_contract"=>body.contains("id=GC_SAFE_TRANSIENT_STATE_ONLY|required=1")
                && body.contains("id=RESIDUAL_COMPLETES_DISCARDED_DISTINCTIONS|required=1"),
            _=>false
        };
        if !contract_ok {return Err(format!("Omnibit original contract drift: {id}"));}
        results.push((id.to_owned(),hash_bytes(&bytes)));
    }
    Ok(results)
}

/// Two-bit tags for each trit. The fourth pair 11 is deliberately invalid.
fn binary_pack(trits:&[Trit]) -> Vec<u8> {
    let mut out=vec![1_u8,trits.len() as u8];
    out.resize(2+trits.len().div_ceil(4),0);
    for (i,&trit) in trits.iter().enumerate() {
        out[2+i/4]|=trit_to_digit(trit) << (6-2*(i%4));
    }
    out
}
fn binary_unpack(body:&[u8],n:usize)->Result<Vec<Trit>,String>{
    if body.len()!=2+n.div_ceil(4) || body.first()!=Some(&1) || body.get(1)!=Some(&(n as u8)){
        return Err("binary arm frame/length mismatch".to_owned());
    }
    let mut result=Vec::with_capacity(n);
    for i in 0..n {
        result.push(digit_to_trit((body[2+i/4]>>(6-2*(i%4)))&3)?);
    }
    if n%4!=0 {
        let unused=8-2*(n%4);
        if body[body.len()-1]&((1_u8<<unused)-1)!=0 {
            return Err("binary two-bit noncanonical padding".to_owned());
        }
    }
    Ok(result)
}
fn ternary_pack(trits:&[Trit])->Vec<u8>{
    let mut ordinal=0_u16;
    for &trit in trits {
        ordinal=ordinal*3+u16::from(trit_to_digit(trit));
    }
    let mut out=vec![2_u8,trits.len() as u8];
    out.extend_from_slice(&ordinal.to_le_bytes());
    out
}
fn ternary_unpack(body:&[u8],n:usize)->Result<Vec<Trit>,String>{
    if body.len()!=4 || body[0]!=2 || usize::from(body[1])!=n {
        return Err("ternary frame length mismatch".to_owned());
    }
    let mut ordinal=u16::from_le_bytes([body[2],body[3]]);
    let mut power=1_u16;
    for _ in 0..n {power=power.checked_mul(3).ok_or("ternary power overflow")?;}
    if ordinal>=power {return Err("out-of-domain ternary ordinal".to_owned());}
    let mut out=vec![Trit::Zero;n];
    for index in (0..n).rev() {
        out[index]=digit_to_trit((ordinal%3) as u8)?;
        ordinal/=3;
    }
    Ok(out)
}
fn floor_id(value:u16)->Option<u8>{
    match value {64=>Some(0),256=>Some(1),1024=>Some(2),4096=>Some(3),_=>None}
}
fn floor_value(value:u8)->Option<u16>{
    match value {0=>Some(64),1=>Some(256),2=>Some(1024),3=>Some(4096),_=>None}
}
fn mixed_pack(trits:&[Trit])->Result<Vec<u8>,String>{
    let words=bridge::encode_route(trits).map_err(str::to_owned)?;
    let mut out=vec![3_u8,trits.len() as u8,words.len() as u8];
    for word in words {
        out.push(floor_id(word.floor).ok_or("invalid mixed binary floor")?);
        out.extend_from_slice(&word.code.to_le_bytes());
    }
    Ok(out)
}
fn mixed_unpack(body:&[u8],n:usize)->Result<Vec<Trit>,String>{
    if body.len()<3 || body[0]!=3 || usize::from(body[1])!=n
        || !(1..=2).contains(&body[2])
        || body.len()!=3+usize::from(body[2])*3 {
        return Err("mixed floor frame mismatch".to_owned());
    }
    let mut words=Vec::new();
    for cell in body[3..].chunks_exact(3){
        words.push(FloorWord{
            floor:floor_value(cell[0]).ok_or("invalid mixed floor tag")?,
            code:u16::from_le_bytes([cell[1],cell[2]])
        });
    }
    bridge::decode_route(&words,n).map_err(str::to_owned)
}
#[derive(Clone,Debug)]
struct ArmOutcome {
    body:Vec<u8>,
    trits:Vec<Trit>,
    logical_steps:usize,
    bridge_glyphs:usize,
}
fn arm_run(arm:&str,trits:&[Trit])->Result<ArmOutcome,String>{
    let (body,logical_steps,bridge_glyphs) = match arm {
        "BINARY"=>(binary_pack(trits),trits.len()*2,0),
        "TERNARY"=>(ternary_pack(trits),trits.len()*4,0),
        "MIXED"=>{
            let body=mixed_pack(trits)?;
            let glyphs=crate::prism::encode(&body);
            if crate::prism::decode(&glyphs,body.len()).map_err(str::to_owned)?!=body {
                return Err("source-pinned Q-PRISM mixed route changed".to_owned());
            }
            let words=usize::from(body[2]);
            let logical=trits.len()*4+words*2+glyphs.len()*2;
            (body,logical,glyphs.len())
        }
        _=>return Err("unknown Omnibit arm".to_owned()),
    };
    let restored=match arm {
        "BINARY"=>binary_unpack(&body,trits.len())?,
        "TERNARY"=>ternary_unpack(&body,trits.len())?,
        "MIXED"=>mixed_unpack(&body,trits.len())?,
        _=>return Err("unknown Omnibit arm".to_owned()),
    };
    if restored!=trits{return Err("tournament exact reconstruction failed".to_owned());}
    Ok(ArmOutcome{body,trits:restored,logical_steps,bridge_glyphs})
}
#[derive(Clone,Copy,Debug,Eq,PartialEq)]
enum Retention{SummaryOnly,ExactReplayRequired}
fn can_reclaim(mode:Retention,summary_committed:bool,complete_payload:bool,
               independent_replay:bool)->bool {
    summary_committed && match mode {
        Retention::SummaryOnly=>true,
        Retention::ExactReplayRequired=>complete_payload&&independent_replay,
    }
}

type ArchiveBundle=(String,String,String,Vec<u8>);
fn make_bundle(sources:&[(String,String)], bridge_root:&str,fixture_digest:&str)
    -> Result<ArchiveBundle,String>
{
    if sources.len()!=5 || bridge_root.len()!=64 || fixture_digest.len()!=64 {
        return Err("Omnibit original source or bridge root absent".to_owned());
    }
    let cases=fixture_inputs()?;
    let mut hbp=ARCHIVE_HEADER.to_owned();
    let mut hbi=INDEX_HEADER.to_owned();
    let mut payload=Vec::<u8>::new();
    let mut prev=hash_bytes(b"PNP-OMNIBIT-PUBLIC-SIX-VANTAGE-GENESIS-v1");
    let mut seq=0_usize;
    let mut totals=[0_usize;3];
    let mut work=[0_usize;3];
    let mut append=|kind:&str,id:&str,fields:&str|{
        seq+=1;
        let data=format!(
            "{kind}|seq={seq}|id={id}|fixture_sha256={fixture_digest}|bridge_sha256={bridge_root}|{fields}|prev_hash={prev}"
        );
        let receipt_sha=hash_bytes(data.as_bytes());
        let chain=hash_bytes(format!("{prev}|{receipt_sha}").as_bytes());
        let row=format!("{data}|receipt_sha={receipt_sha}|hash={chain}|json=0\n");
        hbi.push_str(&format!(
            "HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={chain}|json=0\n",
            hbp.len(),row.len(),hash_bytes(row.as_bytes()),&receipt_sha[..16]
        ));
        hbp.push_str(&row);
        prev=chain;
    };
    for (id,digest) in sources {
        append("OMNIBIT_SOURCE",id,
               &format!("source_sha256={digest}|revision={COMMIT}|family=ORIGINAL_PINNED|status=VERIFIED"));
    }
    for &seat in &SEATS {
        for &view in &VIEWS {
            for case in &cases {
                let mut input=case.data.clone();
                if view=="ANTI"{input.reverse();}
                for (arm_i,&arm) in ARMS.iter().enumerate() {
                    let outcome=arm_run(arm,&input)?;
                    if outcome.trits!=input{return Err("Omnibit comparator input changed".to_owned());}
                    let archive_offset=payload.len();
                    let encoded_len=outcome.body.len();
                    payload.extend_from_slice(&(encoded_len as u32).to_le_bytes());
                    payload.extend_from_slice(&outcome.body);
                    let storage_len=4+encoded_len;
                    totals[arm_i]+=storage_len;
                    work[arm_i]+=outcome.logical_steps;
                    let id=format!("{seat}_{view}_{}_{}",case.id,arm);
                    append("OMNIBIT_RUN",&id,&format!(
                        "seat_label={seat}|view_label={view}|original_view_binding=HOLD|synthetic_view={}|seat_local_execution=0|case={}|ntrits={}|arm={arm}|input_sha256={}|payload_offset={archive_offset}|payload_len={encoded_len}|framed_bytes={storage_len}|payload_sha256={}|logical_steps={}|prism_glyphs={}|exact_replay=PASS|bench_clock=NOT_MEASURED|input_class=PUBLIC_SYNTHETIC|scoreboard=NOT_SCORED|winner=NONE",
                        if view=="NORMAL"{"FORWARD"}else{"REVERSED_PROXY"},
                        case.id,case.data.len(),
                        hash_bytes(case.data.iter().map(|t|t.symbol() as u8).collect::<Vec<_>>().as_slice()),
                        hash_bytes(&outcome.body),outcome.logical_steps,outcome.bridge_glyphs
                    ));
                }
            }
        }
    }
    let retention=[
        ("summary_only",Retention::SummaryOnly,true,false,false,true),
        ("exact_missing",Retention::ExactReplayRequired,true,false,false,false),
        ("exact_restorable",Retention::ExactReplayRequired,true,true,true,true)
    ];
    for (id,mode,committed,payload_retained,replay,expected) in retention {
        let permitted=can_reclaim(mode,committed,payload_retained,replay);
        if permitted!=expected{return Err("GC retention decision contradicted Omnibit contract".to_owned());}
        append("OMNIBIT_GC",id,&format!(
            "retention={}|summary_committed={}|payload_retained={}|independent_replay={}|reclaim={}|manual_approval=NOT_REQUIRED|source_data=PROTECTED",
            if mode==Retention::SummaryOnly{"SUMMARY_ONLY"}else{"EXACT_REPLAY_REQUIRED"},
            u8::from(committed),u8::from(payload_retained),
            u8::from(replay),u8::from(permitted)
        ));
    }
    append("OMNIBIT_SUMMARY","public_tournament",&format!(
        "seats=3|views=2|arms=3|fixtures={CASE_COUNT}|runs={TOURNEY_ROWS}|exact_replay=ALL_PASS|binary_framed_bytes={}|ternary_framed_bytes={}|mixed_framed_bytes={}|binary_logical_steps={}|ternary_logical_steps={}|mixed_logical_steps={}|source_catalog_operators=40|original_source_scoreboard=NOT_SCORED|new_winner=NONE|private_seat_runs=0|performance_rank=HELD|full_decoder_charge=HELD",
        totals[0],totals[1],totals[2],work[0],work[1],work[2]
    ));
    if seq!=TOTAL_ROWS || payload.is_empty(){
        return Err("incomplete source-bound tournament records".to_owned());
    }
    let root=format!(
        "OMNIBIT_ANCHOR|rows={seq}|runs={TOURNEY_ROWS}|fixture_sha256={fixture_digest}|bridge_sha256={bridge_root}|payload_sha256={}|payload_bytes={}|hbp_sha256={}|hbi_sha256={}|chain_hash={prev}|full_decoder_charge=HOLD|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(&payload),payload.len(),
        hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes())
    );
    let anchor=format!("{root}|checksum={}|json=0\n",hash_bytes(root.as_bytes()));
    Ok((hbp,hbi,anchor,payload))
}
fn source_roots(source:&Path,bridge_dir:&Path)->Result<(Vec<(String,String)>,String),String> {
    let roots=pin_original(source)?;
    let b=fs::read(bridge_dir.join("PNP_BRIDGE.hbp"))
        .map_err(|e|format!("binary ternary bridge archive unavailable: {e}"))?;
    let h=fs::read(bridge_dir.join("PNP_BRIDGE.hbi"))
        .map_err(|e|format!("binary ternary bridge HBI unavailable: {e}"))?;
    let anchor=fs::read(bridge_dir.join("PNP_BRIDGE.anchor"))
        .map_err(|e|format!("binary ternary bridge anchor unavailable: {e}"))?;
    if !b.starts_with(b"HBPHEADER|schema=pnp.binary.ternary.bridge.v1")
        || !h.starts_with(b"HBIHEADER|schema=pnp.binary.ternary.bridge.index.v1")
        || !anchor.starts_with(b"BRIDGE_ANCHOR|rows=26|")
    {
        return Err("bridge evidence schema drift".to_owned());
    }
    let text=std::str::from_utf8(&anchor).map_err(|_|"invalid bridge anchor UTF8")?;
    if !text.contains(&format!("hbp_sha256={}",hash_bytes(&b)))
        || !text.contains(&format!("hbi_sha256={}",hash_bytes(&h))) {
        return Err("bridge root HBP/HBI commitment mismatch".to_owned());
    }
    Ok((roots,hash_bytes(&b)))
}
pub fn attest(original:&Path,upstream:&Path,out:&Path)->Result<usize,String> {
    let (sources,bridge_root)=source_roots(original,upstream)?;
    let (hbp,hbi,anchor,payload)=make_bundle(&sources,&bridge_root,
                                             &hash_bytes(FIXTURES.as_bytes()))?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,bytes) in [
        ("PNP_OMNIBIT.hbp",hbp.as_bytes()),
        ("PNP_OMNIBIT.hbi",hbi.as_bytes()),
        ("PNP_OMNIBIT.anchor",anchor.as_bytes()),
        ("PNP_OMNIBIT_PAYLOAD.bin",payload.as_slice())
    ] {
        fs::write(out.join(name),bytes).map_err(|e|format!("Omnibit output {name}: {e}"))?;
    }
    Ok(TOTAL_ROWS)
}
pub fn verify(original:&Path,upstream:&Path,out:&Path)->Result<usize,String> {
    let (sources,bridge_root)=source_roots(original,upstream)?;
    let (hbp,hbi,anchor,payload)=make_bundle(&sources,&bridge_root,
                                             &hash_bytes(FIXTURES.as_bytes()))?;
    for (name,expected) in [
        ("PNP_OMNIBIT.hbp",hbp.as_bytes()),
        ("PNP_OMNIBIT.hbi",hbi.as_bytes()),
        ("PNP_OMNIBIT.anchor",anchor.as_bytes()),
        ("PNP_OMNIBIT_PAYLOAD.bin",payload.as_slice())
    ] {
        let found=fs::read(out.join(name)).map_err(|e|format!("Omnibit {name} missing: {e}"))?;
        if found!=expected {return Err(format!("Omnibit {name} replay differed"));}
    }
    Ok(TOTAL_ROWS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_fixture_and_six_vantages_are_distinct_typed_dimensions() {
        let cases=fixture_inputs().unwrap();
        assert_eq!(cases.len(),20);
        assert_eq!(SEATS.len()*VIEWS.len(),6);
        assert_eq!(ARMS.len(),3);
        assert_eq!(TOURNEY_ROWS,360);
        assert_eq!(TOTAL_ROWS,369);
        assert!(cases.iter().all(|x|(3..=8).contains(&x.data.len())));
    }
    #[test]
    fn all_three_arms_roundtrip_for_every_public_fixture_both_views() {
        for case in fixture_inputs().unwrap() {
            for reverse in [false,true] {
                let mut input=case.data.clone();
                if reverse {input.reverse();}
                for arm in ARMS {
                    let got=arm_run(arm,&input).unwrap();
                    assert_eq!(got.trits,input);
                    assert!(!got.body.is_empty());
                    assert!(got.logical_steps>0);
                    if arm=="MIXED" {assert!(got.bridge_glyphs>0);}
                }
            }
        }
    }
    #[test]
    fn original_controls_and_reconstruction_guards() {
        assert!(!can_reclaim(Retention::SummaryOnly,false,false,false));
        assert!(can_reclaim(Retention::SummaryOnly,true,false,false));
        assert!(!can_reclaim(Retention::ExactReplayRequired,true,false,true));
        assert!(!can_reclaim(Retention::ExactReplayRequired,true,true,false));
        assert!(can_reclaim(Retention::ExactReplayRequired,true,true,true));
        assert!(binary_unpack(&[1,3,255],3).is_err());
        assert!(ternary_unpack(&[2,3,255,255],3).is_err());
        assert!(mixed_unpack(&[3,8,1,0,255,255],8).is_err());
        assert!(binary_unpack(&[1,3,0b01_10_00_01],3).is_err());
    }
}
