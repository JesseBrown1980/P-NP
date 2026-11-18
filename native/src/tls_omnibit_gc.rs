//! Lossless, source-anchored 40-operator coding of the PRL-inspired
//! stochastic TLS event streams. The independent decoder consumes only the
//! retained archive and this native executable, never generator hidden state.
#![forbid(unsafe_code)]

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::{
    binary_ternary_bridge as bridge,
    omega_operator_catalog as omega,
    omnibit_operator_gate, sha256::hash_bytes,
    ternary::Trit, tls_memory, tls_stochastic,
};

const SOURCE_MAGIC: &[u8] = b"PNP-TLS-STOCHASTIC-v1\0";
const MAGIC: &[u8;8] = b"PNPTLS40";
const RESTORED_MAGIC: &[u8] = b"PNP-TLS-OMNI40-RESTORED-v1\0";
const STREAMS: usize = 18;
const ROUTES: usize = 3;
const ROWS: usize = 1 + STREAMS*ROUTES + 9 + 3 + 4 + 1;
const EVENTS: usize = 8_000;
const SYMBOLS: usize = 20;
const BLOCKS_PER_PAIR: [usize;3] = [400,80,50];
const CANDIDATES_PER_BLOCK: usize = 40;
const HBP_HEADER: &str = "HBPHEADER|schema=pnp.tls.omnibit40.coded.v1|scope=PUBLIC_SYNTHETIC|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HBI_HEADER: &str = "HBIHEADER|schema=pnp.tls.omnibit40.index.v1|for=PNP_TLS_OMNI40.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const GC_OWNED: &str = "PNP_TLS_OMNI40_GULP_OWNED.tmp";

type Banks = Vec<(u8, Vec<u16>)>;
type Bundle = (String,String,String,Vec<u8>,Vec<u8>);

#[derive(Clone,Debug,Eq,PartialEq)]
struct Input {
    mode:u8, rep:u8, shuffle:u8, events:Vec<u8>,
}
#[derive(Clone,Debug,Eq,PartialEq)]
struct Decoded {
    mode:u8, rep:u8, shuffle:u8, route:u8, events:Vec<u8>,
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Counts {
    pub rows:usize,
    pub streams:usize,
    pub archive_bytes:usize,
    pub restored_bytes:usize,
    pub input_bytes:usize,
    pub selected_nonidentity:usize,
    pub original_operator_trials:usize,
    pub baseline_identity_bytes:usize,
    pub program_bytes:usize,
}
fn take_u32(data:&[u8],at:&mut usize)->Result<usize,String>{
    if data.len().saturating_sub(*at)<4{return Err("truncated source segment length".to_owned());}
    let length=u32::from_le_bytes(data[*at..*at+4].try_into()
        .map_err(|_|"source segment framing invalid")?) as usize;
    *at+=4;
    Ok(length)
}
fn source_streams(data:&[u8])->Result<Vec<Input>,String>{
    if !data.starts_with(SOURCE_MAGIC){return Err("stochastic source archive magic invalid".to_owned());}
    let mut at=SOURCE_MAGIC.len();
    let mut result=Vec::with_capacity(STREAMS);
    for mode in 0_u8..3 {
        for rep in 0_u8..3 {
            if data.len().saturating_sub(at)<2 || data[at]!=mode || data[at+1]!=rep {
                return Err("stochastic scenario identity or ordering changed".to_owned());
            }
            at+=2;
            let mut test:Option<Vec<u8>>=None;
            let mut shuffle:Option<Vec<u8>>=None;
            for kind in 0..4 {
                let n=take_u32(data,&mut at)?;
                let expected=match kind{0=>8_000,1=>4_000,_=>8_000};
                if n!=expected || data.len().saturating_sub(at)<n {
                    return Err("source train/validation/test segment truncated".to_owned());
                }
                if data[at..at+n].iter().any(|v|*v>1){
                    return Err("nonbinary source event alphabet".to_owned());
                }
                if kind==2{test=Some(data[at..at+n].to_vec());}
                if kind==3{shuffle=Some(data[at..at+n].to_vec());}
                at+=n;
            }
            let test=test.ok_or("missing heldout segment")?;
            let shuffled=shuffle.ok_or("missing time-order-shuffled segment")?;
            let count=|v:&Vec<u8>|v.iter().filter(|x|**x==1).count();
            if count(&test)!=count(&shuffled){
                return Err("heldout/shuffle event count differs".to_owned());
            }
            result.push(Input{mode,rep,shuffle:0,events:test});
            result.push(Input{mode,rep,shuffle:1,events:shuffled});
        }
    }
    if at!=data.len()||result.len()!=STREAMS{
        return Err("unexpected trailing source trace data".to_owned());
    }
    Ok(result)
}
fn trit(event:u8)->Result<Trit,String>{
    match event{0=>Ok(Trit::Minus),1=>Ok(Trit::Plus),
        _=>Err("event not a binary history symbol".to_owned())}
}
fn pack_events(input:&[u8])->Result<Vec<bridge::FloorWord>,String>{
    let data=input.iter().copied().map(trit).collect::<Result<Vec<_>,_>>()?;
    bridge::encode_route(&data).map_err(str::to_owned)
}
fn unroute(words:&[bridge::FloorWord],n:usize)->Result<Vec<u8>,String>{
    let trits=bridge::decode_route(words,n).map_err(str::to_owned)?;
    trits.iter().map(|t|match t{
        Trit::Minus=>Ok(0),Trit::Plus=>Ok(1),
        Trit::Zero=>Err("decoded reserved zero trit instead of event".to_owned())
    }).collect()
}
fn group_size(route:u8)->Result<usize,String>{
    match route {0=>Ok(1),1=>Ok(5),2=>Ok(8),
        _=>Err("unknown binary/ternary/mixed floor route".to_owned())}
}
fn floor_bits(route:u8)->Result<Vec<u8>,String>{
    match route{0=>Ok(vec![6]),1=>Ok(vec![8]),2=>Ok(vec![8,6]),
        _=>Err("unknown floor route tag".to_owned())}
}
fn banks_for(events:&[u8],route:u8)->Result<Banks,String>{
    let group=group_size(route)?;
    if events.len()!=SYMBOLS*group {
        return Err("selected source event window length differs".to_owned());
    }
    let mut banks=floor_bits(route)?.into_iter()
        .map(|bits|(bits,Vec::with_capacity(SYMBOLS)))
        .collect::<Banks>();
    for entry in events.chunks_exact(group){
        let words=pack_events(entry)?;
        if words.len()!=banks.len(){
            return Err("mixed 256+64 floor bank cardinality changed".to_owned());
        }
        for (i,word) in words.iter().enumerate(){
            if word.floor != 1_u16<<banks[i].0 {
                return Err("floor symbol width disagrees with accepted bridge".to_owned());
            }
            banks[i].1.push(word.code);
        }
    }
    Ok(banks)
}
fn events_from_banks(banks:&Banks,route:u8)->Result<Vec<u8>,String>{
    let n=group_size(route)?;
    if banks.len()!=floor_bits(route)?.len()
       || banks.iter().any(|(_,values)|values.len()!=SYMBOLS){
        return Err("standalone decoded floor bank geometry invalid".to_owned());
    }
    let mut result=Vec::with_capacity(SYMBOLS*n);
    for i in 0..SYMBOLS{
        let words=banks.iter().map(|(bits,values)|bridge::FloorWord{
            floor:1_u16<<bits,code:values[i]
        }).collect::<Vec<_>>();
        result.extend(unroute(&words,n)?);
    }
    Ok(result)
}
fn rle(values:&[u16])->Vec<u8>{
    let mut out=Vec::new();
    let mut i=0_usize;
    while i<values.len(){
        let mut n=1_usize;
        while i+n<values.len()&&n<128&&values[i+n]==values[i]{n+=1;}
        if n>=2{
            out.push(0x80|((n-1) as u8));
            out.extend_from_slice(&values[i].to_le_bytes());
            i+=n;
        }else{
            let start=i;
            i+=1;
            while i<values.len()&&i-start<128{
                if i+1<values.len()&&values[i]==values[i+1]{break;}
                i+=1;
            }
            out.push((i-start-1) as u8);
            for word in &values[start..i]{out.extend_from_slice(&word.to_le_bytes());}
        }
    }
    out
}
fn un_rle(bytes:&[u8])->Result<Vec<u16>,String>{
    let mut out=Vec::with_capacity(SYMBOLS);
    let mut i=0_usize;
    while i<bytes.len(){
        let op=bytes[i];i+=1;
        let count=usize::from(op&127)+1;
        if out.len()+count>SYMBOLS{return Err("RLE escaped 20-symbol window".to_owned());}
        if op&128!=0{
            if bytes.len()-i<2{return Err("RLE run body truncated".to_owned());}
            let word=u16::from_le_bytes([bytes[i],bytes[i+1]]);
            out.resize(out.len()+count,word);
            i+=2;
        }else{
            let n=count*2;
            if bytes.len()-i<n{return Err("RLE literal body truncated".to_owned());}
            for pair in bytes[i..i+n].chunks_exact(2){
                out.push(u16::from_le_bytes([pair[0],pair[1]]));
            }
            i+=n;
        }
    }
    if out.len()!=SYMBOLS||rle(&out)!=bytes{
        return Err("noncanonical RLE or incomplete event word stream".to_owned());
    }
    Ok(out)
}
fn encode_block(id:u8,banks:&Banks)->Result<Vec<u8>,String>{
    if id>=40{return Err("operator selector outside frozen original catalog".to_owned());}
    let mut frame=vec![id];
    for (bits,data) in banks{
        if data.len()!=SYMBOLS{return Err("original operator bank not 20 symbols".to_owned());}
        let transformed=omega::apply_symbols(id,*bits,data).map_err(|e|e.to_string())?;
        let inverse=omega::invert_symbols(id,*bits,&transformed).map_err(|e|e.to_string())?;
        if inverse!=*data{return Err("original 40-operator source inverse differed".to_owned());}
        let runs=rle(&transformed);
        if runs.len()<SYMBOLS*2{
            frame.push(1);
            frame.extend_from_slice(&(runs.len() as u16).to_le_bytes());
            frame.extend_from_slice(&runs);
        }else{
            frame.push(0);
            frame.extend_from_slice(&((SYMBOLS*2) as u16).to_le_bytes());
            for value in transformed{frame.extend_from_slice(&value.to_le_bytes());}
        }
    }
    Ok(frame)
}
fn decode_block(bytes:&[u8],at:&mut usize,route:u8)->Result<Vec<u8>,String>{
    let count=floor_bits(route)?;
    if *at>=bytes.len(){return Err("missing 40-operator source selector".to_owned());}
    let op=bytes[*at];
    *at+=1;
    if op>=40{return Err("invalid original source operator ID".to_owned());}
    let mut banks=Vec::with_capacity(count.len());
    for bits in count{
        if bytes.len().saturating_sub(*at)<3{return Err("missing floor mode and length".to_owned());}
        let method=bytes[*at];
        let n=u16::from_le_bytes([bytes[*at+1],bytes[*at+2]]) as usize;
        *at+=3;
        if method>1||n>bytes.len()-*at{return Err("invalid residual coder length/mode".to_owned());}
        let payload=&bytes[*at..*at+n];
        *at+=n;
        let transformed=if method==0{
            if n!=SYMBOLS*2{return Err("literal residual count incorrect".to_owned());}
            payload.chunks_exact(2).map(|pair|u16::from_le_bytes([pair[0],pair[1]])).collect::<Vec<_>>()
        }else{un_rle(payload)?};
        if transformed.len()!=SYMBOLS{
            return Err("floor reconstruction missing symbols".to_owned());
        }
        if method==0&&rle(&transformed).len()<n{
            return Err("uncanonical coder mode: RLE smaller than raw".to_owned());
        }
        let original=omega::invert_symbols(op,bits,&transformed).map_err(|e|e.to_string())?;
        banks.push((bits,original));
    }
    events_from_banks(&banks,route)
}
fn block_count(route:u8)->Result<usize,String>{
    Ok(EVENTS/(SYMBOLS*group_size(route)?))
}
fn record_frame(mode:u8,rep:u8,shuffle:u8,route:u8,body:&[u8])
   ->Result<Vec<u8>,String>{
    let len=u32::try_from(body.len()).map_err(|_|"body too large for archived length")?;
    let mut frame=vec![mode,rep,shuffle,route];
    frame.extend_from_slice(&(EVENTS as u32).to_le_bytes());
    frame.extend_from_slice(&len.to_le_bytes());
    frame.extend_from_slice(body);
    Ok(frame)
}
fn decode_archive(archive:&[u8])->Result<Vec<Decoded>,String>{
    if !archive.starts_with(MAGIC)||archive.len()<10||
        u16::from_le_bytes([archive[8],archive[9]])!=54{
        return Err("standalone TLS Omnibit archive magic or count wrong".to_owned());
    }
    let mut at=10;
    let mut decoded=Vec::with_capacity(54);
    for mode in 0..3_u8{
        for rep in 0..3_u8{
            for shuffle in 0..2_u8{
                for route in 0..3_u8{
                    if archive.len().saturating_sub(at)<12{
                        return Err("missing stream identity or residual frame".to_owned());
                    }
                    if archive[at..at+4]!=[mode,rep,shuffle,route]{
                        return Err("stream route identity not canonical".to_owned());
                    }
                    at+=4;
                    let source_n=take_u32(archive,&mut at)?;
                    let body_n=take_u32(archive,&mut at)?;
                    if source_n!=EVENTS||body_n>archive.len()-at{
                        return Err("selected stream length or payload invalid".to_owned());
                    }
                    let end=at+body_n;
                    let mut words=Vec::with_capacity(EVENTS);
                    for _ in 0..block_count(route)?{
                        words.extend(decode_block(archive,&mut at,route)?);
                    }
                    if at!=end||words.len()!=EVENTS{
                        return Err("selected stream has missing or additional symbols".to_owned());
                    }
                    decoded.push(Decoded{mode,rep,shuffle,route,events:words});
                }
            }
        }
    }
    if at!=archive.len(){return Err("trailing archive data after all stream blocks".to_owned());}
    Ok(decoded)
}
fn restored_bytes(rows:&[Decoded])->Result<Vec<u8>,String>{
    if rows.len()!=54{return Err("unexpected standalone inverse stream cardinality".to_owned());}
    let mut out=RESTORED_MAGIC.to_vec();
    out.extend_from_slice(&(rows.len() as u16).to_le_bytes());
    for row in rows{
        out.extend_from_slice(&[row.mode,row.rep,row.shuffle,row.route]);
        out.extend_from_slice(&(row.events.len() as u32).to_le_bytes());
        out.extend_from_slice(&row.events);
    }
    Ok(out)
}
pub fn decode_to_file(archive:&Path,out:&Path)->Result<usize,String>{
    let bytes=fs::read(archive).map_err(|e|format!("standalone TLS archive read: {e}"))?;
    let output=restored_bytes(&decode_archive(&bytes)?)?;
    let mut file=File::create(out).map_err(|e|e.to_string())?;
    file.write_all(&output).map_err(|e|e.to_string())?;
    file.sync_all().map_err(|e|e.to_string())?;
    Ok(output.len())
}
fn verify_sources(original:&Path,prior:&Path)
   ->Result<(String,String,Vec<Input>),String>{
    if tls_memory::verify(prior)?.rows!=196
       ||tls_stochastic::verify(prior,prior)?.rows!=117
       ||omnibit_operator_gate::verify(original,prior,prior)?!=20710{
        return Err("original DOI and source-pinned Omnibit prerequisites unavailable".to_owned());
    }
    let input_archive=fs::read(prior.join("PNP_TLS_STOCHASTIC_TRACE.bin"))
        .map_err(|e|format!("heldout source archive not present: {e}"))?;
    let streams=source_streams(&input_archive)?;
    let source_hbp=fs::read(prior.join("PNP_TLS_STOCHASTIC.hbp"))
        .map_err(|e|format!("heldout source HBP not present: {e}"))?;
    let original_hbp=fs::read(prior.join("PNP_OMNI40.hbp"))
        .map_err(|e|format!("original source catalog HBP not present: {e}"))?;
    Ok((hash_bytes(&source_hbp),hash_bytes(&original_hbp),streams))
}
fn saved_program()->Result<(usize,String),String>{
    let location=std::env::current_exe().map_err(|e|format!("native binary not found: {e}"))?;
    let body=fs::read(&location).map_err(|e|format!("native decoder binary read: {e}"))?;
    Ok((body.len(),hash_bytes(&body)))
}
fn make_bundle(source_sha:&str,operator_sha:&str,inputs:&[Input],
               program_size:usize,program_sha:&str)->Result<(Bundle,Counts),String>{
    if source_sha.len()!=64||operator_sha.len()!=64||program_sha.len()!=64
       ||inputs.len()!=STREAMS || omega::OPERATORS.len()!=40{
        return Err("missing original source, decoder binary or operator pin".to_owned());
    }
    let mut hbp=HBP_HEADER.to_owned();
    let mut hbi=HBI_HEADER.to_owned();
    let mut prior=hash_bytes(b"PNP-TLS-OMNIBIT40-TEMPORAL-v1");
    let mut seq=0_usize;
    let mut push=|tag:&str,id:&str,fields:&str|{
        seq+=1;
        let bare=format!("{tag}|seq={seq}|id={id}|prior_stochastic_hbp_sha256={source_sha}|prior_original_omni40_sha256={operator_sha}|decoder_sha256={program_sha}|{fields}|prev_hash={prior}");
        let digest=hash_bytes(bare.as_bytes());
        let chain=hash_bytes(format!("{prior}|{digest}").as_bytes());
        let row=format!("{bare}|receipt_sha={digest}|hash={chain}|json=0\n");
        hbi.push_str(&format!("HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={chain}|json=0\n",
            hbp.len(),row.len(),hash_bytes(row.as_bytes()),&digest[..16]));
        hbp.push_str(&row);
        prior=chain;
    };
    push("TLS_OMNI_SOURCE","authenticated",&format!(
        "doi=10.1103/j1r2-5ph7|original_catalog_operators=40|original_operators=BYTE_IDENTICAL|source=PR43_FROZEN_EVENTS|hidden_labels=NOT_USED|scenario_groups=9|streams=18|routes=BINARY64,TERNARY256,MIXED256_PLUS_64|binary_symbols=1|ternary_symbols=5|mixed_symbols=8|program_bytes={program_size}|program_sha256={program_sha}|external_model_bytes=0|event_alphabet=0_1|records=72"
    ));
    let mut archive=MAGIC.to_vec();
    archive.extend_from_slice(&(54_u16).to_le_bytes());
    let mut total_candidate=0_usize;
    let mut total_blocks=0_usize;
    let mut nonidentity=0_usize;
    let mut baseline=0_usize;
    let mut route_counts=[[0_usize;3];2];
    let mut by_mode=[[0_usize;3];3];
    let mut restored=Vec::with_capacity(54);
    for mode in 0..3_u8{
        for rep in 0..3_u8{
            let mut pair=[[0_usize;3];2];
            for shuffle in 0..2_u8{
                let input=inputs.iter().find(|x|x.mode==mode&&x.rep==rep&&x.shuffle==shuffle)
                    .ok_or("heldout source selector missing")?;
                for route in 0..3_u8{
                    let mut body=Vec::new();
                    let mut per_baseline=0_usize;
                    let mut per_nonidentity=0_usize;
                    let mut selected_rle=0_usize;
                    let group=SYMBOLS*group_size(route)?;
                    for window in input.events.chunks_exact(group){
                        let banks=banks_for(window,route)?;
                        let raw=encode_block(0,&banks)?;
                        per_baseline+=raw.len();
                        let mut best=(usize::MAX,u8::MAX,Vec::new());
                        for op in 0_u8..40{
                            let candidate=encode_block(op,&banks)?;
                            total_candidate+=1;
                            if (candidate.len(),op)<(best.0,best.1){
                                best=(candidate.len(),op,candidate);
                            }
                        }
                        if best.1!=0{per_nonidentity+=1;}
                        // The first selected bank's mode byte is immediately
                        // after its source-catalog selector.
                        if best.2.get(1)==Some(&1){selected_rle+=1;}
                        let mut at=0_usize;
                        let decoded=decode_block(&best.2,&mut at,route)?;
                        if decoded!=window||at!=best.2.len(){
                            return Err("selected source operator/residual failed exact event replay".to_owned());
                        }
                        body.extend_from_slice(&best.2);
                        total_blocks+=1;
                    }
                    let framed=record_frame(mode,rep,shuffle,route,&body)?;
                    let archive_offset=archive.len();
                    archive.extend_from_slice(&framed);
                    let frame_len=framed.len();
                    let baseline_bytes=per_baseline+12;
                    baseline+=baseline_bytes;
                    nonidentity+=per_nonidentity;
                    pair[shuffle as usize][route as usize]=frame_len;
                    route_counts[shuffle as usize][route as usize]+=frame_len;
                    by_mode[mode as usize][route as usize]+=frame_len;
                    let record_id=format!("m{mode}_r{rep}_s{shuffle}_a{route}");
                    push("TLS_OMNI_RUN",&record_id,&format!(
                        "mode={mode}|rep={rep}|shuffle={shuffle}|route={route}|source_events={EVENTS}|blocks={}|candidate_operators=40|source_sha256={}|source_event_ones={}|saved_frame_offset={archive_offset}|saved_frame_bytes={frame_len}|saved_frame_sha256={}|identity_baseline_bytes={baseline_bytes}|chosen_nonidentity={per_nonidentity}|chosen_first_bank_rle={selected_rle}|complete_selector_and_residual=YES|decoder_external_state=NONE|replay=EXACT",
                        BLOCKS_PER_PAIR[route as usize],hash_bytes(&input.events),
                        input.events.iter().map(|v|usize::from(*v)).sum::<usize>(),
                        hash_bytes(&framed)
                    ));
                    restored.push(Decoded{mode,rep,shuffle,route,events:input.events.clone()});
                }
            }
            for route in 0..3_usize{
                let holder=pair[0][route];
                let shuffled=pair[1][route];
                push("TLS_OMNI_PAIR",&format!("m{mode}_r{rep}_a{route}"),&format!(
                    "mode={mode}|rep={rep}|route={route}|heldout_frame_bytes={holder}|shuffled_frame_bytes={shuffled}|heldout_minus_shuffled_bytes={}|relation=TEMPORAL_ORDER_CONTROL|identity_baseline=PER_STREAM|event_counts_identical=YES",
                    holder as i64-shuffled as i64
                ));
            }
        }
    }
    for mode in 0..3_usize{
        push("TLS_OMNI_REGIME",&format!("{mode}"),&format!(
            "mode={mode}|runs_per_route=6|binary_frame_bytes={}|ternary_frame_bytes={}|mixed_frame_bytes={}|source=HELDOUT_AND_SHUFFLED",
            by_mode[mode][0],by_mode[mode][1],by_mode[mode][2]
        ));
    }
    let gc=[
        ("summary_committed",true,false,false,true),
        ("exact_missing",true,false,false,false),
        ("exact_replayed",true,true,true,true),
        ("not_committed",false,true,true,false)
    ];
    for (id,committed,retained,verified,expected) in gc {
        let kind=if id=="summary_committed"||id=="not_committed" {
            tls_memory::Retention::SummaryOnly
        }else{tls_memory::Retention::ExactReplayRequired};
        if tls_memory::may_discard(kind,committed,retained,verified)!=expected{
            return Err("protected original temporal trace GC policy contradicted".to_owned());
        }
        push("TLS_OMNI_GC",id,&format!(
            "committed={}|reconstructible={}|independent_replay={}|reclaim_eligible={}|owned_scratch_only=YES|source_archive=PROTECTED|live_exact_source_deletion=NO|operator_approval=AUTOMATIC",
            u8::from(committed),u8::from(retained),u8::from(verified),u8::from(expected)
        ));
    }
    let decoded=decode_archive(&archive)?;
    if decoded!=restored||total_blocks!=9_540||total_candidate!=381_600{
        return Err("integrated temporal matrix all40 candidate/evidence tally changed".to_owned());
    }
    let recovered=restored_bytes(&decoded)?;
    let bitpacked_lower=STREAMS*EVENTS/8;
    push("TLS_OMNI_SUMMARY","full_archive",&format!(
        "streams=18|stream_routes=54|original_operators=40|candidate_trials={total_candidate}|selected_blocks={total_blocks}|selected_nonidentity={nonidentity}|selected_archive_bytes={}|identity_only_archive_bytes={}|source_uncompressed_bytes={}|source_bitpacked_bytes={bitpacked_lower}|restored_archive_bytes={}|decoder_program_bytes={program_size}|standalone_decoder=EXACT|binary_heldout_bytes={}|binary_shuffle_bytes={}|ternary_heldout_bytes={}|ternary_shuffle_bytes={}|mixed_heldout_bytes={}|mixed_shuffle_bytes={}|trained_model_state=NOT_NEEDED_FOR_DECODE|live_gc=OWNED_SCRATCH_ONLY|metric=ACTUAL_FRAMED_BYTES|wall_time=SEPARATE_MEASUREMENT|experimental_physics=NO",
        archive.len(),baseline+10,STREAMS*EVENTS,recovered.len(),
        route_counts[0][0],route_counts[1][0],route_counts[0][1],
        route_counts[1][1],route_counts[0][2],route_counts[1][2]
    ));
    if seq!=ROWS{return Err("integrated temporal source receipt count wrong".to_owned());}
    let root=format!("TLS_OMNI_ANCHOR|rows={ROWS}|streams=54|candidate_trials=381600|source_hbp_sha256={source_sha}|original_hbp_sha256={operator_sha}|decoder_sha256={program_sha}|decoder_bytes={program_size}|archive_sha256={}|restored_sha256={}|hbp_sha256={}|hbi_sha256={}|terminal_hash={prior}|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(&archive),hash_bytes(&recovered),hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes()));
    let anchor=format!("{root}|checksum={}|json=0\n",hash_bytes(root.as_bytes()));
    let result=Counts{
        rows:ROWS, streams:STREAMS,archive_bytes:archive.len(),restored_bytes:recovered.len(),
        input_bytes:STREAMS*EVENTS,selected_nonidentity:nonidentity,
        original_operator_trials:total_candidate,baseline_identity_bytes:baseline+10,
        program_bytes:program_size,
    };
    Ok(((hbp,hbi,anchor,archive,recovered),result))
}
fn bundle(original:&Path,prior:&Path)->Result<(Bundle,Counts),String>{
    let (source_sha,operator_sha,streams)=verify_sources(original,prior)?;
    let (size,sha)=saved_program()?;
    make_bundle(&source_sha,&operator_sha,&streams,size,&sha)
}
pub fn attest(original:&Path,prior:&Path,out:&Path)->Result<Counts,String>{
    let ((hbp,hbi,anchor,archive,recovered),stats)=bundle(original,prior)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,bytes) in [
        ("PNP_TLS_OMNI40.hbp",hbp.as_bytes()),
        ("PNP_TLS_OMNI40.hbi",hbi.as_bytes()),
        ("PNP_TLS_OMNI40.anchor",anchor.as_bytes()),
        ("PNP_TLS_OMNI40_SELECTED.bin",archive.as_slice()),
        ("PNP_TLS_OMNI40_RESTORED.bin",recovered.as_slice())
    ]{
        let mut file=File::create(out.join(name)).map_err(|e|format!("archive {name}: {e}"))?;
        file.write_all(bytes).map_err(|e|format!("write {name}: {e}"))?;
        file.sync_all().map_err(|e|format!("sync {name}: {e}"))?;
    }
    Ok(stats)
}
pub fn verify(original:&Path,prior:&Path,out:&Path)->Result<Counts,String>{
    let ((hbp,hbi,anchor,archive,recovered),stats)=bundle(original,prior)?;
    for (name,expected) in [
        ("PNP_TLS_OMNI40.hbp",hbp.as_bytes()),
        ("PNP_TLS_OMNI40.hbi",hbi.as_bytes()),
        ("PNP_TLS_OMNI40.anchor",anchor.as_bytes()),
        ("PNP_TLS_OMNI40_SELECTED.bin",archive.as_slice()),
        ("PNP_TLS_OMNI40_RESTORED.bin",recovered.as_slice())
    ]{
        let found=fs::read(out.join(name)).map_err(|e|format!("verify {name}: {e}"))?;
        if found!=expected{return Err(format!("integrated temporal archive changed: {name}"));}
    }
    Ok(stats)
}
pub fn gulp_discard_owned(original:&Path,prior:&Path,out:&Path)
    ->Result<(usize,usize),String>{
    // Source is always protected. The only deletion target is an exclusively
    // created, locally owned scratch projection of its held-out event bytes.
    let count=verify(original,prior,out)?;
    let (source_sha,_,streams)=verify_sources(original,prior)?;
    let mut owned=Vec::with_capacity(STREAMS*EVENTS);
    for entry in streams{owned.extend_from_slice(&entry.events);}
    if owned.len()!=count.input_bytes{return Err("owned projection changed".to_owned());}
    let scratch=out.join(GC_OWNED);
    let mut f=OpenOptions::new().create_new(true).write(true).open(&scratch)
        .map_err(|e|format!("owned scratch could not be created exclusively: {e}"))?;
    f.write_all(&owned).map_err(|e|e.to_string())?;
    f.sync_all().map_err(|e|e.to_string())?;
    drop(f);
    if fs::read(&scratch).map_err(|e|e.to_string())?!=owned {
        return Err("owned scratch bytes not committed".to_owned());
    }
    let archived=fs::read(out.join("PNP_TLS_OMNI40_SELECTED.bin"))
        .map_err(|e|e.to_string())?;
    let recovered=restored_bytes(&decode_archive(&archived)?)?;
    if fs::read(out.join("PNP_TLS_OMNI40_RESTORED.bin"))
        .map_err(|e|e.to_string())?!=recovered {
        return Err("decoder independent payload not present before GULP".to_owned());
    }
    // Sync durable archive and receipt before removing any owned working copy.
    for name in ["PNP_TLS_OMNI40_SELECTED.bin","PNP_TLS_OMNI40_RESTORED.bin",
        "PNP_TLS_OMNI40.hbp","PNP_TLS_OMNI40.hbi","PNP_TLS_OMNI40.anchor"]{
        File::open(out.join(name)).map_err(|e|e.to_string())?.sync_all()
            .map_err(|e|format!("durability sync {name}: {e}"))?;
    }
    fs::remove_file(&scratch).map_err(|e|format!("owned GULP scratch removal: {e}"))?;
    if scratch.exists(){return Err("owned GULP scratch still exists after collection".to_owned());}
    // The producer archive remains available for downstream science and audit.
    if fs::read(prior.join("PNP_TLS_STOCHASTIC.hbp"))
        .map_err(|e|e.to_string()).map(|b|hash_bytes(&b))?!=source_sha{
        return Err("protected original temporal source lost after GULP".to_owned());
    }
    let anchor=fs::read(out.join("PNP_TLS_OMNI40.anchor"))
        .map_err(|e|e.to_string())?;
    let raw=format!(
        "TLS_OMNI_GC_ACTION|scope=OWNED_SCRATCH_ONLY|deleted_bytes={}|scratch_path={GC_OWNED}|scratch_absent=PASS|original_source=RETAINED|retained_selected_sha256={}|retained_restored_sha256={}|original_hbp_sha256={source_sha}|anchor_sha256={}|durability=ARCHIVE_SYNCED_BEFORE_DELETE|manual_approval=NOT_REQUIRED|json=0",
        owned.len(),hash_bytes(&archived),hash_bytes(&recovered),hash_bytes(&anchor)
    );
    let proof=format!("{raw}|sha256={}\n",hash_bytes(raw.as_bytes()));
    let mut report=File::create(out.join("PNP_TLS_OMNI40_GC.hbp"))
        .map_err(|e|e.to_string())?;
    report.write_all(proof.as_bytes()).map_err(|e|e.to_string())?;
    report.sync_all().map_err(|e|e.to_string())?;
    Ok((owned.len(),archived.len()))
}
pub fn timing(original:&Path,prior:&Path,out:&Path)->Result<(Counts,u128),String>{
    let time=Instant::now();
    let result=attest(original,prior,out)?;
    if verify(original,prior,out)?!=result{
        return Err("timed original-operator result changed under independent replay".to_owned());
    }
    Ok((result,time.elapsed().as_nanos()))
}
#[cfg(test)]
mod tests{
    use super::*;
    fn sample(n:usize)->Vec<u8>{
        (0..n).map(|i|u8::from((i/13)%5==0 || (i/19)%7==0)).collect()
    }
    #[test]
    fn all_original_operators_and_three_routes_invert_twenty_symbol_windows(){
        for route in 0..3_u8{
            let original=sample(SYMBOLS*group_size(route).unwrap());
            let banks=banks_for(&original,route).unwrap();
            assert_eq!(events_from_banks(&banks,route).unwrap(),original);
            for id in 0..40_u8{
                let body=encode_block(id,&banks).unwrap();
                let mut at=0;
                assert_eq!(decode_block(&body,&mut at,route).unwrap(),original);
                assert_eq!(at,body.len());
            }
        }
    }
    #[test]
    fn source_history_order_and_selector_corruption_are_rejected(){
        let original=sample(SYMBOLS*8);
        let banks=banks_for(&original,2).unwrap();
        let source=encode_block(29,&banks).unwrap();
        let mut bad=source.clone();bad[0]=40;
        assert!(decode_block(&bad,&mut 0,2).is_err());
        let mut bad=source.clone();bad[1]=2;
        assert!(decode_block(&bad,&mut 0,2).is_err());
        let mut bad=source.clone();bad.pop();
        assert!(decode_block(&bad,&mut 0,2).is_err());
        assert!(decode_archive(b"not an Omnibit archive").is_err());
        assert_eq!(ROWS,72);
        assert_eq!(BLOCKS_PER_PAIR.iter().sum::<usize>()*STREAMS*40,381_600);
    }
}
