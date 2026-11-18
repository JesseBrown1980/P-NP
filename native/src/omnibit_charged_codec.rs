//! Integer-only, source-bound experimental Omnibit coder. The original
//! integrated Omega floor codec is still under construction; this is a
//! complete standalone RAW/RLE baseline with charged selectors and binaries.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use crate::{
    binary_ternary_bridge::{self as bridge, FloorWord},
    omega_operator_catalog as omega,
    omnibit_operator_gate,
    sha256::hash_bytes,
};

const COMMIT: &str = "dad4647ad19a1b08ee90990c34aab1255f9a6046";
const CONTRACT_PATH: &str =
    "reference/omega-floor-codec-contract-v0/INTEGRATED-OMEGA-FLOOR-CODEC-CONTRACT.hbp";
const CONTRACT_BLOB: &str = "a494d4f87cb027e458a5f326dfa10a68fac5c2d1";
const CORPUS: &str = include_str!("../fixtures/OMNI40_CODER_CORPUS.hbi");
const PATTERNS: [&str;7]=[
    "CONSTANT","ALTERNATING","PERIOD7","BURSTS","NOISE",
    "PI_CONTROL","TRI_CONTROL",
];
const VIEWS: [&str;2]=["NORMAL","ANTI"];
const FLOOR_ROUTES: [u8;5]=[6,8,10,12,0];
const N: usize=240;
const CASES:usize=7*2*5;
const CANDIDATES:usize=CASES*40;
const ROWS:usize=1+7+CANDIDATES+CASES+1;
const HBP_HEADER:&str="HBPHEADER|schema=pnp.omni40.charged.rle.v1|scope=PUBLIC_SYNTHETIC|coder=RAW_OR_CANONICAL_RLE|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HBI_HEADER:&str="HBIHEADER|schema=pnp.omni40.charged.index.v1|for=PNP_OMNI40_CODEC.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const SELECTED_MAGIC:&[u8;8]=b"OM40SEL1";
const RESTORED_MAGIC:&[u8;8]=b"OM40SRC1";
const FRAME_MAGIC:&[u8;4]=b"O40C";
type Bank=(u8,Vec<u16>);
type Banks=Vec<Bank>;
type Bundle=(String,String,String,Vec<u8>,Vec<u8>,Vec<u8>);

#[derive(Debug,Clone,Eq,PartialEq)]
pub struct CodecStats {
    pub receipts:usize,
    pub candidates:usize,
    pub contexts:usize,
    pub selected_nonidentity:usize,
    pub baseline_bytes:usize,
    pub selected_bytes:usize,
    pub program_bytes:u64,
    pub candidate_archive_bytes:usize,
}

fn declared_corpus()->Result<(),String>{
    let mut expected="HBIHEADER|schema=pnp.omni40.coder.corpus.v1|basis=TERNARY_TO_BINARY|length=240|seed=FIXED_INTEGER|json=0\n".to_owned();
    for p in PATTERNS {
        let cls=if p.ends_with("_CONTROL"){"TARGETED_CONTROL"}else{"GENERAL"};
        expected.push_str(&format!(
            "CORPUS|id={p}|class={cls}|symbols=240|floor_routes=64,256,1024,4096,256+64|views=NORMAL,ANTI|source=DETERMINISTIC_GENERATOR|json=0\n"
        ));
    }
    if CORPUS!=expected{return Err("frozen Omnibit coder corpus definition changed".to_owned());}
    Ok(())
}
fn alphabet(bits:u8)->Result<u16,String>{
    match bits {
        6=>Ok(27),8=>Ok(243),10=>Ok(729),12=>Ok(2187),
        _=>Err("invalid coder floor width".to_owned())
    }
}
fn integer_seed(i:usize)->u32{
    let mut v=(i as u32).wrapping_mul(0x9e37_79b9)^0x5c4a_1379;
    v^=v<<13;v^=v>>17;v^=v<<5;
    v
}
fn source_value(p:&str,i:usize,alphabet:u16)->Result<u16,String>{
    let mid=(alphabet-1)/2;
    let hi=alphabet-1;
    let value=match p {
        "CONSTANT"=>mid,
        "ALTERNATING"=>if i%2==0{0}else{hi},
        "PERIOD7"=>((i%7*37+(i%7)*(i%7)*11) as u16)%alphabet,
        "BURSTS"=>match (i/13)%4 {0=>mid,1=>hi,2=>0,_=>mid},
        "NOISE"=>(integer_seed(i)%u32::from(alphabet)) as u16,
        "PI_CONTROL"=>if i%20<10{mid}else{hi},
        "TRI_CONTROL"=>if i%12<6{mid}else{hi},
        _=>return Err("unrecognized coder corpus pattern".to_owned())
    };
    Ok(value)
}
fn source_banks(pattern:&str,route:u8,view:&str)->Result<Banks,String>{
    let bits=if route==0{vec![8_u8,6]}else{vec![route]};
    let control_id=match pattern{"PI_CONTROL"=>Some(20),"TRI_CONTROL"=>Some(9),_=>None};
    let mut result: Banks=Vec::new();
    if route==0 {
        let values=(0..N).map(|i|source_value(pattern,i,6561))
            .collect::<Result<Vec<_>,_>>()?;
        let mut five=values.iter().map(|ordinal|ordinal/27).collect::<Vec<_>>();
        let mut three=values.iter().map(|ordinal|ordinal%27).collect::<Vec<_>>();
        if let Some(id)=control_id {
            five=omega::invert_symbols(id,8,&five).map_err(|e|e.to_string())?;
            three=omega::invert_symbols(id,6,&three).map_err(|e|e.to_string())?;
        }
        result.push((8,five));
        result.push((6,three));
    } else {
        let valid=alphabet(route)?;
        let mut values=(0..N).map(|i|source_value(pattern,i,valid))
            .collect::<Result<Vec<_>,_>>()?;
        if let Some(id)=control_id {
            values=omega::invert_symbols(id,route,&values).map_err(|e|e.to_string())?;
        }
        result.push((route,values));
    }
    if !bits.iter().copied().eq(result.iter().map(|(b,_)|*b)){
        return Err("coder mixed floor bank ordering changed".to_owned());
    }
    if view=="ANTI" {
        for (_,data) in &mut result{data.reverse();}
    } else if view!="NORMAL" {
        return Err("unknown synthetic comparison view".to_owned());
    }
    for (b,values) in &result {
        let valid=alphabet(*b)?;
        if values.len()!=N || values.iter().any(|x|*x>=valid){
            return Err("corpus generator produced noncanonical ternary floor code".to_owned());
        }
        for value in values {
            let trits=bridge::decode_floor(FloorWord{floor:1_u16<<b,code:*value})
                .map_err(str::to_owned)?;
            if bridge::encode_floor(1_u16<<b,&trits).map_err(str::to_owned)?.code!=*value{
                return Err("source corpus lost a balanced trit".to_owned());
            }
        }
    }
    if route==0 {
        for i in 0..N {
            let pair=[
                FloorWord{floor:256,code:result[0].1[i]},
                FloorWord{floor:64,code:result[1].1[i]}
            ];
            if bridge::decode_route(&pair,8).map_err(str::to_owned)?.len()!=8 {
                return Err("source corpus composed 8-trit decode failed".to_owned());
            }
        }
    }
    Ok(result)
}
fn raw_values(values:&[u16])->Vec<u8>{
    let mut out=Vec::with_capacity(values.len()*2);
    for v in values {out.extend_from_slice(&v.to_le_bytes());}
    out
}
fn run_length(values:&[u16],start:usize)->usize {
    let mut n=1;
    while n<128 && start+n<values.len() && values[start+n]==values[start] {n+=1;}
    n
}
fn canonical_rle(values:&[u16])->Vec<u8>{
    let mut out=Vec::new();
    let mut i=0;
    while i<values.len(){
        let repeat=run_length(values,i);
        if repeat>=2 {
            out.push(0x80|((repeat-1) as u8));
            out.extend_from_slice(&values[i].to_le_bytes());
            i+=repeat;
        } else {
            let start=i;
            i+=1;
            while i<values.len() && i-start<128 && run_length(values,i)<2 {i+=1;}
            out.push((i-start-1) as u8);
            out.extend_from_slice(&raw_values(&values[start..i]));
        }
    }
    out
}
fn read_symbols(bytes:&[u8],len:usize)->Result<Vec<u16>,String>{
    if bytes.len()!=len.checked_mul(2).ok_or("literal word count overflow")?{
        return Err("literal word count invalid".to_owned());
    }
    Ok(bytes.chunks_exact(2).map(|b|u16::from_le_bytes([b[0],b[1]])).collect())
}
fn unpack_rle(payload:&[u8],expected:usize)->Result<Vec<u16>,String>{
    let mut at=0;
    let mut out=Vec::with_capacity(expected);
    while at<payload.len(){
        let tag=payload[at];
        at+=1;
        let count=usize::from(tag&127)+1;
        if count>expected.saturating_sub(out.len()) {
            return Err("RLE token extends beyond advertised source symbols".to_owned());
        }
        if tag&128!=0 {
            if payload.len()-at<2{return Err("missing RLE repeat symbol".to_owned());}
            let val=u16::from_le_bytes([payload[at],payload[at+1]]);
            out.extend(std::iter::repeat_n(val,count));
            at+=2;
        }else{
            let n=count.checked_mul(2).ok_or("RLE literal arithmetic overflow")?;
            if payload.len()-at<n{return Err("missing RLE literal payload".to_owned());}
            out.extend(read_symbols(&payload[at..at+n],count)?);
            at+=n;
        }
    }
    if out.len()!=expected || canonical_rle(&out)!=payload{
        return Err("RLE invalid length or noncanonical token stream".to_owned());
    }
    Ok(out)
}
fn emit_frame(id:u8,route:u8,source:&[Bank])->Result<Vec<u8>,String>{
    if id>=40 || source.is_empty() || source.len()>2 {
        return Err("operator or bank count unavailable".to_owned());
    }
    let mut out=Vec::new();
    out.extend_from_slice(FRAME_MAGIC);
    out.extend_from_slice(&[1,id,source.len() as u8,route]);
    out.extend_from_slice(&(source[0].1.len() as u16).to_le_bytes());
    for (bits,values) in source {
        if values.len()!=source[0].1.len(){return Err("bank lengths differ".to_owned());}
        let transformed=omega::apply_symbols(id,*bits,values).map_err(|e|e.to_string())?;
        let raw=raw_values(&transformed);
        let rle=canonical_rle(&transformed);
        let (mode,body)=if rle.len()<raw.len(){(1_u8,rle)}else{(0_u8,raw)};
        out.extend_from_slice(&[*bits,mode]);
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&body);
    }
    Ok(out)
}
fn decode_frame(frame:&[u8])->Result<(u8,Banks),String>{
    if frame.len()<10 || &frame[..4]!=FRAME_MAGIC || frame[4]!=1 || frame[5]>=40
        || !(1..=2).contains(&frame[6]) || !FLOOR_ROUTES.contains(&frame[7])
    {return Err("invalid charged frame header or selector".to_owned());}
    let route=frame[7];
    let bank_count=usize::from(frame[6]);
    if (route==0 && bank_count!=2) || (route!=0 && bank_count!=1){
        return Err("mixed-floor route requires exact bank count".to_owned());
    }
    let n=usize::from(u16::from_le_bytes([frame[8],frame[9]]));
    if n==0 || n>4096{return Err("invalid charged frame symbol count".to_owned());}
    let mut offset=10;
    let mut restored=Vec::new();
    for bank in 0..bank_count{
        if frame.len().saturating_sub(offset)<6 {
            return Err("short floor model and coded symbol length".to_owned());
        }
        let bits=frame[offset];
        let mode=frame[offset+1];
        let declared=u32::from_le_bytes(frame[offset+2..offset+6].try_into()
            .map_err(|_|"bad floor payload length")?) as usize;
        offset+=6;
        let expected_bits=if route==0 {if bank==0{8}else{6}} else {route};
        if bits!=expected_bits || mode>1 || declared>frame.len()-offset {
            return Err("floor width, coder mode or payload length invalid".to_owned());
        }
        let block=&frame[offset..offset+declared];
        offset+=declared;
        let values=if mode==0{
            read_symbols(block,n)?
        }else{
            unpack_rle(block,n)?
        };
        if mode==0 && canonical_rle(&values).len()<block.len() {
            return Err("nonminimal raw-vs-RLE framing".to_owned());
        }
        if values.iter().any(|x|*x>=1_u16<<bits){
            return Err("coded floor symbol outside original alphabet".to_owned());
        }
        let inverse=omega::invert_symbols(frame[5],bits,&values)
            .map_err(|e|e.to_string())?;
        let valid=alphabet(bits)?;
        if inverse.iter().any(|x|*x>=valid){
            return Err("original source did not reconstruct canonical ternary values".to_owned());
        }
        restored.push((bits,inverse));
    }
    if offset!=frame.len(){return Err("unexpected trailing model/residual bytes".to_owned());}
    if route==0 {
        for index in 0..n{
            let joined=[
                FloorWord{floor:256,code:restored[0].1[index]},
                FloorWord{floor:64,code:restored[1].1[index]}
            ];
            if bridge::decode_route(&joined,8).map_err(str::to_owned)?.len()!=8{
                return Err("mixed route did not reconstruct exactly eight trits".to_owned());
            }
        }
    }
    Ok((route,restored))
}
fn canonical_source(out:&mut Vec<u8>,route:u8,banks:&[Bank])->Result<(),String>{
    if banks.is_empty(){return Err("source data cannot be empty".to_owned());}
    out.push(route);
    out.extend_from_slice(&(banks[0].1.len() as u16).to_le_bytes());
    out.push(banks.len() as u8);
    for (bits,values) in banks {
        out.push(*bits);
        out.extend_from_slice(&(values.len() as u16).to_le_bytes());
        out.extend_from_slice(&raw_values(values));
    }
    Ok(())
}
pub fn unpack_selected(archive:&[u8])->Result<Vec<u8>,String>{
    if archive.len()<10 || &archive[..8]!=SELECTED_MAGIC{
        return Err("selected standalone archive header absent".to_owned());
    }
    let n=usize::from(u16::from_le_bytes([archive[8],archive[9]]));
    if n!=CASES {return Err("selected archive context count changed".to_owned());}
    let mut out=RESTORED_MAGIC.to_vec();
    out.extend_from_slice(&(n as u16).to_le_bytes());
    let mut at=10;
    for _ in 0..n {
        if archive.len().saturating_sub(at)<4{
            return Err("selected archive truncated before frame length".to_owned());
        }
        let len=u32::from_le_bytes(archive[at..at+4].try_into()
            .map_err(|_|"invalid selected length")?) as usize;
        at+=4;
        if len>archive.len()-at{return Err("selected payload body truncated".to_owned());}
        let (route,banks)=decode_frame(&archive[at..at+len])?;
        canonical_source(&mut out,route,&banks)?;
        at+=len;
    }
    if at!=archive.len(){return Err("selected archive has extra encoded bytes".to_owned());}
    Ok(out)
}
pub fn unpack_to_file(input:&Path,out:&Path)->Result<usize,String>{
    let archive=fs::read(input).map_err(|e|format!("standalone archive read: {e}"))?;
    let original=unpack_selected(&archive)?;
    fs::write(out,&original).map_err(|e|format!("write recovered source: {e}"))?;
    Ok(original.len())
}
fn confirm_source(original:&Path,prior:&Path)->Result<(String,String),String>{
    if omnibit_operator_gate::verify(original,prior,prior)?!=20710{
        return Err("prior 40-operator original source HBI/HBP proof unavailable".to_owned());
    }
    let output=Command::new("git").arg("-C").arg(original)
        .args(["hash-object",CONTRACT_PATH]).output()
        .map_err(|e|format!("original codec Git unavailable: {e}"))?;
    if !output.status.success()
        || String::from_utf8_lossy(&output.stdout).trim()!=CONTRACT_BLOB{
        return Err("original integrated codec contract Git blob changed".to_owned());
    }
    let src=fs::read(original.join(CONTRACT_PATH))
        .map_err(|e|format!("original integrated codec contract unavailable: {e}"))?;
    let body=std::str::from_utf8(&src).map_err(|_|"invalid original contract bytes")?;
    if !body.contains("CHARGE|submission_bytes=FINAL_ARCHIVE_PLUS_STANDALONE_DECODER_ARTIFACT")
        || !body.contains("CODECFTR|contract_status=BUILD_IN_PROGRESS")
        || !body.contains("MODES|raw=0|path1=1")
        || !body.contains("selector_transmitted=1|selector_charged=1")
    {
        return Err("original coder accounting contract drift".to_owned());
    }
    let root=fs::read(prior.join("PNP_OMNI40.hbp"))
        .map_err(|e|format!("original 40-operator evidence HBP missing: {e}"))?;
    Ok((hash_bytes(&src),hash_bytes(&root)))
}
fn installed_program()->Result<(u64,String),String>{
    let path=std::env::current_exe().map_err(|e|format!("current codec binary path: {e}"))?;
    let raw=fs::read(&path).map_err(|e|format!("read charged codec executable: {e}"))?;
    Ok((raw.len() as u64,hash_bytes(&raw)))
}
fn make_bundle(contract_sha:&str,upstream_sha:&str,program:(u64,&str))
    ->Result<(Bundle,CodecStats),String>
{
    declared_corpus()?;
    let (program_bytes,program_sha)=program;
    if [contract_sha,upstream_sha,program_sha].iter().any(|s|s.len()!=64){
        return Err("missing source, upstream or executable root".to_owned());
    }
    let mut hbp=HBP_HEADER.to_owned();
    let mut hbi=HBI_HEADER.to_owned();
    let mut candidate_archive=Vec::new();
    let mut selected_archive=SELECTED_MAGIC.to_vec();
    selected_archive.extend_from_slice(&(CASES as u16).to_le_bytes());
    let mut restored_source=RESTORED_MAGIC.to_vec();
    restored_source.extend_from_slice(&(CASES as u16).to_le_bytes());
    let mut previous=hash_bytes(b"PNP-OMNI40-CHARGED-CODER-GENESIS-v1");
    let mut seq=0_usize;
    let mut append=|tag:&str,id:&str,fields:&str|{
        seq+=1;
        let record=format!(
            "{tag}|seq={seq}|id={id}|source_commit={COMMIT}|contract_sha256={contract_sha}|original_operator_hbp_sha256={upstream_sha}|executable_sha256={program_sha}|{fields}|prev_hash={previous}"
        );
        let receipt=hash_bytes(record.as_bytes());
        let linked=hash_bytes(format!("{previous}|{receipt}").as_bytes());
        let row=format!("{record}|receipt_sha={receipt}|hash={linked}|json=0\n");
        hbi.push_str(&format!(
            "HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={linked}|json=0\n",
            hbp.len(),row.len(),hash_bytes(row.as_bytes()),&receipt[..16]
        ));
        hbp.push_str(&row);
        previous=linked;
    };
    append("OMNI40_CODEC_SOURCE","source",&format!(
        "original_contract_blob={CONTRACT_BLOB}|contract_source_sha256={contract_sha}|program_bytes={program_bytes}|program_sha256={program_sha}|corpus_sha256={}|model_bytes_external=0|decoder_binary=COMPLETE_PNP_NATIVE|coder=BOUNDED_RAW_OR_CANONICAL_RLE|original_integrated_codec=BUILD_IN_PROGRESS",
        hash_bytes(CORPUS.as_bytes())
    ));
    for pattern in PATTERNS {
        append("OMNI40_CODEC_CORPUS",pattern,&format!(
            "class={}|source=FROZEN_SYNTHETIC|symbols_per_bank={N}|floor_routes=5|comparison_views=2",
            if pattern.ends_with("_CONTROL"){"TARGETED_CONTROL"}else{"GENERAL"}
        ));
    }
    let mut contexts=0_usize;
    let mut trials=0_usize;
    let mut nonidentity=0_usize;
    let mut total_raw=10_usize;
    let mut total_program_work=0_usize;
    let mut total_zero=0_usize;
    let mut totals:BTreeMap<&str,(usize,usize,usize)>=BTreeMap::new();
    for pattern in PATTERNS {
        for view in VIEWS {
            for route in FLOOR_ROUTES {
                let source=source_banks(pattern,route,view)?;
                let mut source_words=Vec::new();
                canonical_source(&mut source_words,route,&source)?;
                let input_sha=hash_bytes(&source_words);
                restored_source.extend_from_slice(&source_words);
                let baseline_len=10+source.len()*6+source.len()*N*2+4;
                total_raw+=baseline_len;
                let mut selected:(usize,u8,Vec<u8>)=(usize::MAX,u8::MAX,Vec::new());
                for id in 0_u8..40 {
                    let frame=emit_frame(id,route,&source)?;
                    let (decoded_route,replayed)=decode_frame(&frame)?;
                    if decoded_route!=route || replayed!=source {
                        return Err(format!("original operator candidate failed replay: {pattern}:{id}"));
                    }
                    let offset=candidate_archive.len();
                    candidate_archive.extend_from_slice(&(frame.len() as u32).to_le_bytes());
                    candidate_archive.extend_from_slice(&frame);
                    let bytes=frame.len()+4;
                    if (bytes,id)<(selected.0,selected.1) {
                        selected=(bytes,id,frame.clone());
                    }
                    let class=if pattern.ends_with("_CONTROL"){"TARGETED_CONTROL"}else{"GENERAL"};
                    let idstr=format!("{pattern}_{view}_{}_OP{id:02}",if route==0{"256_PLUS_64".to_owned()}else{(1_u16<<route).to_string()});
                    append("OMNI40_CODEC_TRIAL",&idstr,&format!(
                        "pattern={pattern}|class={class}|view={view}|original_view_binding=HOLD|route={route}|source_sha256={input_sha}|operator={id}|family={}|candidate_offset={offset}|charged_frame_bytes={bytes}|selector_bytes=1|floor_bits_charged=1|mode_bytes={}|framed_payload_sha256={}|exact_replay=PASS|external_state_bytes=0|decoder_artifact=CHARGED_ONCE|winner=NONE",
                        if id<8{"RNQ"}else if id<20{"TRI"}else{"PI"},
                        source.len(),hash_bytes(&frame)
                    ));
                    trials+=1;
                    total_program_work+=source.len()*N;
                }
                let selected_offset=selected_archive.len();
                selected_archive.extend_from_slice(&(selected.2.len() as u32).to_le_bytes());
                selected_archive.extend_from_slice(&selected.2);
                if selected.1!=0{nonidentity+=1;}else{total_zero+=1;}
                let category=if pattern.ends_with("_CONTROL"){"TARGETED_CONTROL"}else{"GENERAL"};
                let aggregation=totals.entry(category).or_insert((0,0,0));
                aggregation.0+=baseline_len;
                aggregation.1+=selected.0;
                aggregation.2+=usize::from(selected.1!=0);
                append("OMNI40_CODEC_SELECT",&format!("{pattern}_{view}_{route}"),&format!(
                    "pattern={pattern}|class={category}|view={view}|route={route}|source_sha256={input_sha}|candidates=40|selected_operator={}|selected_frame_offset={selected_offset}|selected_frame_bytes={}|selected_frame_sha256={}|raw_baseline_frame_bytes={baseline_len}|gain_vs_raw_bytes={}|selection_policy=MIN_COMPLETE_FRAME_BYTES_THEN_ID|charged_selector=YES|charged_residual=YES|model_external=NONE|inverse=EXACT",
                    selected.1,selected.0,hash_bytes(&selected.2),
                    (baseline_len as isize)-(selected.0 as isize)
                ));
                contexts+=1;
            }
        }
    }
    let replay=unpack_selected(&selected_archive)?;
    if replay!=restored_source {
        return Err("standalone decoder did not reconstruct complete source corpus".to_owned());
    }
    if contexts!=CASES || trials!=CANDIDATES || total_zero+nonidentity!=CASES{
        return Err("unexpected charged coder candidate census".to_owned());
    }
    let general=totals.get("GENERAL").ok_or("missing general corpus accounting")?;
    let controls=totals.get("TARGETED_CONTROL").ok_or("missing targeted controls accounting")?;
    let selected_bytes=selected_archive.len();
    let charge_one=(program_bytes as u128)+(selected_bytes as u128);
    let charge_three=(program_bytes as u128)*3+(selected_bytes as u128);
    append("OMNI40_CODEC_SUMMARY","full_charge",&format!(
        "source_patterns=7|general_patterns=5|targeted_controls=2|floors=64,256,1024,4096,256+64|views=NORMAL,ANTI|contexts={contexts}|candidates={CANDIDATES}|source_operator_count=40|selected_nonidentity={nonidentity}|selected_identity={total_zero}|raw_baseline_archive_bytes={total_raw}|selected_archive_bytes={selected_bytes}|candidate_audit_archive_bytes={}|general_raw_bytes={}|general_selected_bytes={}|general_nonidentity={}|controls_raw_bytes={}|controls_selected_bytes={}|controls_nonidentity={}|program_bytes={program_bytes}|one_binary_plus_archive_bytes={charge_one}|conservative_three_binary_plus_archive_bytes={charge_three}|unpack_sha256={}|candidate_work_symbols={total_program_work}|standalone_replay=PASS|elapsed_time=RECORDED_SEPARATELY|peak_rss=RECORDED_SEPARATELY|source_omega_integrated_codec=BUILD_IN_PROGRESS|hutter_score=NOT_CLAIMED|source_tournament_winner=NONE",
        candidate_archive.len(),general.0,general.1,general.2,
        controls.0,controls.1,controls.2,hash_bytes(&restored_source)
    ));
    if seq!=ROWS{return Err("charged coding receipt tally not closed".to_owned());}
    let root=format!(
        "OMNI40_CODEC_ANCHOR|rows={ROWS}|contexts={CASES}|candidates={CANDIDATES}|contract_sha256={contract_sha}|upstream_sha256={upstream_sha}|executable_sha256={program_sha}|executable_bytes={program_bytes}|selected_archive_sha256={}|candidate_archive_sha256={}|restored_sha256={}|selected_bytes={selected_bytes}|hbp_sha256={}|hbi_sha256={}|terminal_hash={previous}|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(&selected_archive),hash_bytes(&candidate_archive),
        hash_bytes(&restored_source),hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes())
    );
    let anchor=format!("{root}|checksum={}|json=0\n",hash_bytes(root.as_bytes()));
    Ok(((hbp,hbi,anchor,candidate_archive,selected_archive,restored_source),
        CodecStats{receipts:ROWS,candidates:trials,contexts,
                   selected_nonidentity:nonidentity,baseline_bytes:total_raw,
                   selected_bytes,program_bytes,candidate_archive_bytes:0}))
}
fn bundle(original:&Path,prior:&Path)->Result<(Bundle,CodecStats),String>{
    let (contract_sha,prior_sha)=confirm_source(original,prior)?;
    let (program_bytes,program_sha)=installed_program()?;
    let (files,mut stats)=make_bundle(&contract_sha,&prior_sha,(program_bytes,&program_sha))?;
    stats.candidate_archive_bytes=files.3.len();
    Ok((files,stats))
}
pub fn attest(original:&Path,prior:&Path,out:&Path)->Result<CodecStats,String>{
    let ((hbp,hbi,anchor,candidates,selected,restored),stats)=bundle(original,prior)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,bytes) in [
        ("PNP_OMNI40_CODEC.hbp",hbp.as_bytes()),
        ("PNP_OMNI40_CODEC.hbi",hbi.as_bytes()),
        ("PNP_OMNI40_CODEC.anchor",anchor.as_bytes()),
        ("PNP_OMNI40_CODEC_CANDIDATES.bin",candidates.as_slice()),
        ("PNP_OMNI40_CODEC_SELECTED.bin",selected.as_slice()),
        ("PNP_OMNI40_CODEC_RESTORED.bin",restored.as_slice())
    ]{
        fs::write(out.join(name),bytes)
            .map_err(|e|format!("charged Omnibit coder archive {name}: {e}"))?;
    }
    Ok(stats)
}
pub fn verify(original:&Path,prior:&Path,out:&Path)->Result<CodecStats,String>{
    let ((hbp,hbi,anchor,candidates,selected,restored),stats)=bundle(original,prior)?;
    for (name,expected) in [
        ("PNP_OMNI40_CODEC.hbp",hbp.as_bytes()),
        ("PNP_OMNI40_CODEC.hbi",hbi.as_bytes()),
        ("PNP_OMNI40_CODEC.anchor",anchor.as_bytes()),
        ("PNP_OMNI40_CODEC_CANDIDATES.bin",candidates.as_slice()),
        ("PNP_OMNI40_CODEC_SELECTED.bin",selected.as_slice()),
        ("PNP_OMNI40_CODEC_RESTORED.bin",restored.as_slice())
    ]{
        if fs::read(out.join(name))
            .map_err(|e|format!("verify {name}: {e}"))?!=expected{
            return Err(format!("independent coder proof differs: {name}"));
        }
    }
    Ok(stats)
}
pub fn bench(original:&Path,prior:&Path,out:&Path)->Result<(CodecStats,u128),String>{
    let now=Instant::now();
    let stats=attest(original,prior,out)?;
    if verify(original,prior,out)?!=stats {
        return Err("timed codec measured a changed decode/receipt state".to_owned());
    }
    Ok((stats,now.elapsed().as_nanos()))
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn all_coder_modes_and_origins_roundtrip(){
        for pattern in PATTERNS {
            for view in VIEWS {
                for route in FLOOR_ROUTES {
                    let banks=source_banks(pattern,route,view).unwrap();
                    for id in 0..40{
                        let encoded=emit_frame(id,route,&banks).unwrap();
                        assert_eq!(decode_frame(&encoded).unwrap(),(route,banks.clone()));
                    }
                }
            }
        }
    }
    #[test]
    fn canonical_run_encoding_and_invalid_frame_rejection(){
        let v=vec![3_u16;240];
        let r=canonical_rle(&v);
        assert!(r.len()<480);
        assert_eq!(unpack_rle(&r,240).unwrap(),v);
        let mut bad=r.clone();bad.push(0);
        assert!(unpack_rle(&bad,240).is_err());
        let source=source_banks("PI_CONTROL",8,"NORMAL").unwrap();
        let frame=emit_frame(20,8,&source).unwrap();
        let mut broken=frame.clone();broken[5]=40;
        assert!(decode_frame(&broken).is_err());
        let mut broken=frame.clone();broken[7]=7;
        assert!(decode_frame(&broken).is_err());
        let mut broken=frame.clone();broken.pop();
        assert!(decode_frame(&broken).is_err());
        assert!(unpack_selected(b"invalid").is_err());
    }
    #[test]
    fn source_frozen_patterns_and_complete_charges(){
        declared_corpus().unwrap();
        assert_eq!(CASES,70);
        assert_eq!(CANDIDATES,2800);
        assert_eq!(ROWS,2879);
        let ((hbp,_,_,candidates,selected,recovered),stats)
            =make_bundle(&hash_bytes(b"contract"),
                         &hash_bytes(b"upstream"),(1_234_567,&hash_bytes(b"program")))
                .unwrap();
        assert_eq!(stats.candidates,2800);
        assert_eq!(stats.contexts,70);
        assert_eq!(stats.receipts,2879);
        assert_eq!(hbp.lines().filter(|s|s.starts_with("OMNI40_CODEC_TRIAL|")).count(),2800);
        assert_eq!(hbp.lines().filter(|s|s.starts_with("OMNI40_CODEC_SELECT|")).count(),70);
        assert_eq!(unpack_selected(&selected).unwrap(),recovered);
        assert!(!candidates.is_empty());
        assert!(stats.baseline_bytes>=stats.selected_bytes);
    }
}
