//! Source-frozen ordered/shuffled TLS prediction through original mixed-radix
//! binary/ternary, Omnibit40, Fischer HTTP-0 rooms and exact result recall.
//! Predictor is the original observation-trained classical TLS model.
//! Authentic trained GNN forward inference remains a separately held gate.
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines)]
use std::fs::{self,File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use crate::http0_compute::{Client,Task,CELLS};
use crate::matrix_wire;
use crate::sha256::hash_bytes;
use crate::tls_entropy_omnibit::cases;
use crate::tls_stochastic::{fit,predict,score,select,Context,Fit};
use crate::ensemble::{self,Phase,Signals};

const WIDTHS:[u16;4]=[64,256,1024,4096];
const TEST:usize=8_000;
const TRACES:usize=18;
const PACKETS:usize=TEST.div_ceil(CELLS);
const REQUESTS_PER_PASS:usize=TRACES*PACKETS*4;
const MAGIC:&[u8]=b"PNP-TEMPORAL-FABRIC-V1\0";
const HBP_HEADER:&str="HBPHEADER|schema=pnp.temporal.fabric.v1|source=PRL_SYNTHETIC_STOCHASTIC|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HBI_HEADER:&str="HBIHEADER|schema=pnp.temporal.fabric.index.v1|for=PNP_TEMPORAL_FABRIC.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Stats{
    pub cases:usize,
    pub events:usize,
    pub crossings:usize,
    pub uncached_calls:usize,
    pub cached_calls:usize,
    pub cache_hits:usize,
    pub coupled_gains:usize,
    pub shuffle_reversals:usize,
    pub iid_controls:usize,
    pub uncached_ns:u128,
    pub cached_ns:u128,
}
fn task_for(stream:usize,packet:usize,source:&[u8])->Task{
    let mut trits=[1_u8;CELLS];
    let start=packet*CELLS;
    let end=(start+CELLS).min(TEST);
    trits[..end-start].copy_from_slice(&source[start..end]);
    Task{id:stream as u8,
        seed:((stream as u64+1)<<32)|(packet as u64+1),trits}
}
fn send_stream(client:&Client,mode:u8,stream:usize,source:&[u8])
    ->Result<(Vec<u8>,usize,usize),String>
{
    if source.len()!=TEST || source.iter().any(|&x|x>1){
        return Err("temporal stream source is not canonical 8000 events".into());
    }
    let mut recovered=Vec::with_capacity(TEST);
    let mut hits=0;
    let mut calls=0;
    for packet in 0..PACKETS{
        let task=task_for(stream,packet,source);
        let start=packet*CELLS;
        let needed=(TEST-start).min(CELLS);
        for (floor_index,floor) in WIDTHS.iter().enumerate(){
            // Exercise all forty original operators across the full corpus.
            // Preserve a fixed, source-independent schedule.
            let operator=((stream*PACKETS+packet+floor_index)%40)as u8;
            let framed=matrix_wire::encode(&task,*floor,operator)?;
            let result=client.execute_matrix(mode,27,stream,&framed)?;
            if result.reconstructed!=task.trits{
                return Err(format!("temporal independent radix or Omnibit reconstruction mismatch stream={stream} packet={packet} floor={floor}"));
            }
            if result.computed.values!=crate::microkernel_compute::compute_primary(&task){
                return Err("temporal exact matrix worker/reflection disagreed".into());
            }
            if floor_index==0{
                recovered.extend_from_slice(&result.reconstructed[..needed]);
            }
            calls+=1;
            hits+=usize::from(result.computed.hit);
        }
    }
    if recovered!=source{return Err("fabric changed temporal event order or data".into());}
    Ok((recovered,calls,hits))
}
fn score_reconstructed(events:&[u8],model:usize,trained:&Fit)
    ->Result<(u128,u128),String>
{
    let original_scores=score(events,trained)?;
    let mut replay=Context::new();
    let mut incremental=0_u128;
    let mut constant=0_u128;
    for &event in events{
        let p=predict(model,trained,&replay)?;
        let q=predict(0,trained,&replay)?;
        let target=10_000_i64*i64::from(event);
        let selected_error=target-p;
        let constant_error=target-q;
        incremental+=(selected_error*selected_error)as u128;
        constant+=(constant_error*constant_error)as u128;
        replay.observe(event);
    }
    if original_scores[model]!=incremental || original_scores[0]!=constant{
        return Err("independent causal prediction replay disagreed with frozen scorer".into());
    }
    Ok((constant,incremental))
}
fn push(hbp:&mut String,hbi:&mut String,previous:&mut String,
        seq:usize,kind:&str,id:&str,data:&str){
    let row=format!("{kind}|seq={seq}|id={id}|{data}|previous_hash={previous}");
    let digest=hash_bytes(row.as_bytes());
    let new=hash_bytes(format!("{previous}|{digest}").as_bytes());
    let wire=format!("{row}|receipt_sha={digest}|hash={new}|json=0\n");
    hbi.push_str(&format!("HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={new}|json=0\n",
        hbp.len(),wire.len(),hash_bytes(wire.as_bytes()),&digest[..16]));
    hbp.push_str(&wire);
    *previous=new;
}
type Bundle=(Vec<u8>,String,String,String);
fn program(prior:&Path)->Result<(Bundle,Stats),String>{
    let source=fs::read(prior.join("PNP_TLS_STOCHASTIC_TRACE.bin"))
        .map_err(|e|format!("original heldout TLS trace: {e}"))?;
    let original_cases=cases(&source)?;
    if original_cases.len()!=9{return Err("temporal source case census changed".into());}
    let parent_roots=["PNP_TLS.hbp","PNP_TLS_STOCHASTIC.hbp",
        "PNP_FULL_MATRIX.hbp","PNP_GNN_GATE.hbp","PNP_CHECKPOINT.hbp"];
    let mut roots=Vec::new();
    for name in parent_roots{
        let bytes=fs::read(prior.join(name))
            .map_err(|e|format!("missing original source-root {name}: {e}"))?;
        if !bytes.starts_with(b"HBPHEADER|"){return Err("prior evidence header invalid".into());}
        roots.push((name,hash_bytes(&bytes)));
    }
    let child_plain=Client::start()?;
    let child_cached=Client::start()?;
    let mut hbp=HBP_HEADER.to_owned();
    let mut hbi=HBI_HEADER.to_owned();
    let mut last=hash_bytes(b"PNP-TEMPORAL-FABRIC-GENESIS-2026");
    push(&mut hbp,&mut hbi,&mut last,1,"TEMPORAL_SOURCE","prl_tlscorpus",
        &format!("doi=10.1103/j1r2-5ph7|trace_sha256={}|source_case_count=9|ordered_shuffle_pairs=9|events_per_stream=8000|binary_floors=64,256,1024,4096|operators=ORIGINAL40|gnn_forward=HOLD_SOURCE_PARITY",
            hash_bytes(&source)));
    for (index,(name,root)) in roots.iter().enumerate(){
        push(&mut hbp,&mut hbi,&mut last,index+2,"TEMPORAL_PIN",name,
            &format!("sha256={root}|input=ORIGINAL_ACCEPTED"));
    }
    let mut archive=MAGIC.to_vec();
    archive.extend_from_slice(&9_u16.to_le_bytes());
    let mut total_calls=0_usize;
    let mut uncached_calls=0_usize;
    let mut cached_hits=0_usize;
    let mut coupled_gains=0_usize;
    let mut shuffle_reversals=0_usize;
    let mut iid_controls=0_usize;
    let mut uncached_ns=0_u128;
    let mut cached_ns=0_u128;
    for case in &original_cases{
        let trained=fit(&case.train)?;
        let selected=select(&score(&case.valid,&trained)?);
        let mut outcomes=[(0_u128,0_u128);2];
        for (arm,outcome) in outcomes.iter_mut().enumerate(){
            let source_events=if arm==0{&case.held}else{&case.shuffle};
            let stream=usize::from(case.mode)*6+usize::from(case.rep)*2+arm;
            // Every request has a distinct (stream,packet) key; model
            // selection is frozen before the first held-out transport.
            let begun=Instant::now();
            let (recovered,calls,hits)=send_stream(&child_plain,4,stream,source_events)?;
            uncached_ns+=begun.elapsed().as_nanos();
            if calls!=PACKETS*4 || hits!=0{
                return Err("direct uncached fabric ran the wrong source stream".into());
            }
            uncached_calls+=calls;
            let begun=Instant::now();
            let (recovered_cache,calls_cache,hits_first)=
                send_stream(&child_cached,5,stream,source_events)?;
            let (recovered_again,calls_again,hits_second)=
                send_stream(&child_cached,5,stream,source_events)?;
            cached_ns+=begun.elapsed().as_nanos();
            if recovered_cache!=recovered || recovered_again!=recovered
                || calls_cache!=calls || calls_again!=calls
                || hits_first!=PACKETS*3 || hits_second!=PACKETS*4{
                return Err("source-keyed recall did not preserve temporal context".into());
            }
            cached_hits+=hits_first+hits_second;
            total_calls+=calls_cache+calls_again;
            *outcome=score_reconstructed(&recovered,selected,&trained)?;
            archive.extend_from_slice(&[case.mode,case.rep,arm as u8,selected as u8]);
            archive.extend_from_slice(&outcome.0.to_le_bytes());
            archive.extend_from_slice(&outcome.1.to_le_bytes());
            archive.extend_from_slice(hash_bytes(&recovered).as_bytes());
        }
        let ordered_gain=outcomes[0].0 as i128-outcomes[0].1 as i128;
        let shuffled_gain=outcomes[1].0 as i128-outcomes[1].1 as i128;
        if case.mode==0{
            if selected!=0 || ordered_gain!=0 || shuffled_gain!=0{
                return Err("IID control incorrectly learned temporal memory".into());
            }
            iid_controls+=1;
        }else{
            if selected==0 || ordered_gain<=0 || shuffled_gain>=0{
                return Err("coupled ordered gain or shuffled negative control missing".into());
            }
            coupled_gains+=1;
            shuffle_reversals+=1;
        }
        let gate=ensemble::evaluate(Phase::Described,Signals{
            edge:None,path:None,gain:None,g1:None,g2:None,g3:None
        });
        if gate.emitted || gate.next!=Phase::Described || !gate.operator_held{
            return Err("trained GNN unavailable: G4 must hold".into());
        }
        let id=format!("mode{}_rep{}",case.mode,case.rep);
        push(&mut hbp,&mut hbi,&mut last,7+usize::from(case.mode)*3+usize::from(case.rep),
            "TEMPORAL_RESULT",&id,
            &format!("predictor=TRAIN_ONLY|selection=VALIDATION_ONLY|selected_index={selected}|ordered_base_brier_q2={}|ordered_selected_brier_q2={}|ordered_gain_q2={ordered_gain}|shuffle_base_brier_q2={}|shuffle_selected_brier_q2={}|shuffle_gain_q2={shuffled_gain}|transport=4_FLOORS_ORIGINAL40|worker=INDEPENDENT|exact_event_order=PASS|source_bound_recall=PASS|g4_missing_gnn=HOLD",
                outcomes[0].0,outcomes[0].1,outcomes[1].0,outcomes[1].1));
    }
    let plain=child_plain.stop()?;
    let cached=child_cached.stop()?;
    let expected_entries=TRACES*PACKETS;
    if uncached_calls!=REQUESTS_PER_PASS || plain.calls!=REQUESTS_PER_PASS
        || plain.hits!=0 || plain.entries!=0
        || total_calls!=2*REQUESTS_PER_PASS
        || cached.calls!=2*REQUESTS_PER_PASS
        || cached.hits!=cached_hits
        || cached.entries!=expected_entries
        || cached_hits!=2*REQUESTS_PER_PASS-expected_entries
        || plain.rooms!=18 || cached.rooms!=18
        || (coupled_gains,shuffle_reversals,iid_controls)!=(6,6,3){
        return Err("full matrix temporal recall/acceptance census failed".into());
    }
    push(&mut hbp,&mut hbi,&mut last,16,"TEMPORAL_SUMMARY","accepted",
        &format!("cases=9|ordered_shuffle_pairs=9|events_per_stream=8000|unique_events=144000|crossings_per_pass={REQUESTS_PER_PASS}|uncached_calls={uncached_calls}|cached_calls={total_calls}|reused_results={cached_hits}|cache_entries={expected_entries}|coupled_ordered_gains={coupled_gains}|coupled_shuffle_reversals={shuffle_reversals}|iid_memoryless={iid_controls}|gnn_trained_forward=HOLD|status=PASS"));
    archive.extend_from_slice(hash_bytes(&archive).as_bytes());
    let root=format!("TEMPORAL_ANCHOR|rows=16|archive_sha256={}|source_sha256={}|hbp_sha256={}|hbi_sha256={}|last_hash={last}|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(&archive),hash_bytes(&source),hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes()));
    let anchor=format!("{root}|checksum={}|json=0\n",hash_bytes(root.as_bytes()));
    let stats=Stats{cases:9,events:144_000,crossings:REQUESTS_PER_PASS,
        uncached_calls,cached_calls:total_calls,cache_hits:cached_hits,
        coupled_gains,shuffle_reversals,iid_controls,uncached_ns,cached_ns};
    Ok(((archive,hbp,hbi,anchor),stats))
}
fn persist(path:&Path,data:&[u8])->Result<(),String>{
    let mut f=File::create(path).map_err(|e|e.to_string())?;
    f.write_all(data).map_err(|e|e.to_string())?;
    f.sync_all().map_err(|e|e.to_string())
}
pub fn attest(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((archive,hbp,hbi,anchor),stats)=program(prior)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,body) in [
        ("PNP_TEMPORAL_FABRIC_RESULTS.bin",archive.as_slice()),
        ("PNP_TEMPORAL_FABRIC.hbp",hbp.as_bytes()),
        ("PNP_TEMPORAL_FABRIC.hbi",hbi.as_bytes()),
        ("PNP_TEMPORAL_FABRIC.anchor",anchor.as_bytes())
    ]{persist(&out.join(name),body)?;}
    Ok(stats)
}
pub fn verify(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((archive,hbp,hbi,anchor),stats)=program(prior)?;
    for (name,expected) in [
        ("PNP_TEMPORAL_FABRIC_RESULTS.bin",archive.as_slice()),
        ("PNP_TEMPORAL_FABRIC.hbp",hbp.as_bytes()),
        ("PNP_TEMPORAL_FABRIC.hbi",hbi.as_bytes()),
        ("PNP_TEMPORAL_FABRIC.anchor",anchor.as_bytes())
    ]{
        if fs::read(out.join(name)).map_err(|e|e.to_string())?!=expected{
            return Err(format!("temporal source or result receipt mutated: {name}"));
        }
    }
    Ok(stats)
}
pub fn inspect(path:&Path)->Result<usize,String>{
    let bytes=fs::read(path).map_err(|e|e.to_string())?;
    let head=MAGIC.len()+2;
    let entry=4+16+16+64;
    if bytes.len()!=head+18*entry+64 || !bytes.starts_with(MAGIC)
        || bytes[MAGIC.len()..head]!=9_u16.to_le_bytes(){
        return Err("temporal matrix result archive extent invalid".into());
    }
    let (body,tail)=bytes.split_at(bytes.len()-64);
    if hash_bytes(body).as_bytes()!=tail{return Err("temporal result SHA mismatch".into());}
    for index in 0..18{
        let at=head+index*entry;
        if bytes[at]!=index as u8/6 || bytes[at+1]!=index as u8%6/2
            || bytes[at+2]!=index as u8%2 || bytes[at+3]>4 {
            return Err("temporal result source case order changed".into());
        }
    }
    Ok(18)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn frozen_temporal_packet_preserves_order_and_noncyclic_padding(){
        let raw=(0..TEST).map(|i|((i*3+7)/11%2)as u8).collect::<Vec<_>>();
        let first=task_for(13,0,&raw);
        let last=task_for(13,PACKETS-1,&raw);
        assert_eq!(first.id,13);
        assert_eq!(first.trits[0],raw[0]);
        assert_eq!(last.trits[8],1);
        assert_ne!(last.seed,first.seed);
        for f in WIDTHS {
            let wire=matrix_wire::encode(&first,f,39).unwrap();
            let (recovered,_,_)=matrix_wire::decode_independent(&wire).unwrap();
            assert_eq!(recovered,first);
        }
    }
}
