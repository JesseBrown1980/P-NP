//! Held-out, source-frozen native microkernel computation contest.
//! Matches a direct worker, direct worker with memo, Fischer HTTP-0,
//! Fischer + per-room recall, and deterministic HRM-shape proxy + recall.
//! Trained HRM inference is a separate HOLD, never imitated as loaded.
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::fs::{self,File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use crate::freeze::RIME_PRIME;
use crate::http0_compute::{Client,Task,CELLS};
use crate::sha256::hash_bytes;
use crate::tls_entropy_omnibit::source_event_frames;

const TASKS:usize=54;
const PASSES:usize=3;
const STEPS:usize=243;
const HBP_HEADER:&str="HBPHEADER|schema=pnp.microkernel.compute.v1|source=PR43_PR46|backend=HTTP0_ONLY|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HBI_HEADER:&str="HBIHEADER|schema=pnp.microkernel.compute.index.v1|for=PNP_MICROKERNEL_COMPUTE.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Arm{
    pub mode:u8,
    pub arity:usize,
    pub elapsed_ns:u128,
    pub calls:usize,
    pub hits:usize,
    pub rooms:usize,
    pub entries:usize,
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Stats{
    pub source_sha:String,
    pub result_sha:String,
    pub jobs:usize,
    pub direct_ns:u128,
    pub direct_cache_ns:u128,
    pub arms:Vec<Arm>,
    pub full_binary_bytes:usize,
}
pub(crate) fn source_tasks(bytes:&[u8])->Result<Vec<Task>,String>{
    let streams=source_event_frames(bytes)?;
    if streams.len()!=18 {return Err("expected 18 source-frozen streams".into());}
    let mut tasks=Vec::with_capacity(TASKS);
    for (_,_,_,events) in streams {
        if events.len()!=8_000{return Err("source event count changed".into());}
        let source_digest=hash_bytes(&events);
        let seed_base=u64::from_str_radix(&source_digest[..16],16)
            .map_err(|_|"source SHA prefix encoding")?;
        for route in 0..3_usize {
            let id=tasks.len();
            let mut trits=[0_u8;CELLS];
            for (index,t) in trits.iter_mut().enumerate(){
                let offset=(index*17+route*7)%events.len();
                *t=events[offset]+events[(offset+1)%events.len()];
            }
            let seed=seed_base
                ^ (u64::try_from(id).map_err(|_|"task id too large")?
                    .wrapping_mul(0x9e37_79b9_7f4a_7c15))
                ^ ((route as u64 + 1)<<57);
            tasks.push(Task{id:u8::try_from(id).map_err(|_|"too many tasks")?,
                seed,trits});
        }
    }
    if tasks.len()!=TASKS{return Err("task count changed".into());}
    Ok(tasks)
}
/// Primary exact finite stencil, 27 signed ternary cells x 243 ticks.
/// Independent HTTP-0 fabric computes by a separately written ring method.
pub(crate) fn compute_primary(task:&Task)->[u64;CELLS]{
    let mut state=[0_u64;CELLS];
    let initial=task.seed%RIME_PRIME;
    for (i,value) in state.iter_mut().enumerate() {
        let x=i as u64+1;
        *value=(initial+13*x*x+17*u64::from(task.trits[i]))%RIME_PRIME;
    }
    for step in 0..STEPS{
        let mut update=[0_u64;CELLS];
        for i in 0..CELLS{
            let right=state[(i+1)%CELLS];
            let left=state[(i+CELLS-1)%CELLS];
            let center=state[i];
            let external=11*(u64::from(task.trits[i])+(step%3)as u64);
            update[i]=(3*center+5*right+7*left+external)%RIME_PRIME;
        }
        state=update;
    }
    state
}
/// Historical HRM family shape classes, deterministic route-only proxy.
/// Training or running a learned HRM is explicitly NOT part of this arm.
fn hrm_proxy_lane(task:&Task,arity:usize)->usize{
    let sum=task.trits.iter().map(|&v|usize::from(v)).sum::<usize>();
    let zeros=task.trits.iter().filter(|&&x|x==0).count();
    let highs=task.trits.iter().filter(|&&x|x==2).count();
    let shape=(sum+zeros+3*highs)%8;
    (usize::from(task.id)*7+shape*13+sum)%arity
}
fn permutation(pass:usize,index:usize)->usize{
    match pass {
        0=>index,
        1=>TASKS-1-index,
        2=>(index+17)%TASKS,
        _=>unreachable!("three fixed repetition orders"),
    }
}
fn output_bytes(tasks:&[Task],expect:&[[u64;CELLS]])->Result<Vec<u8>,String>{
    let mut out=b"PNP-TERNARY-KERNEL-RESULTS-v1\0".to_vec();
    out.extend_from_slice(&u16::try_from(tasks.len())
        .map_err(|_|"kernel task count")?.to_le_bytes());
    for (task,values) in tasks.iter().zip(expect){
        out.extend_from_slice(&task.encode());
        for value in values {out.extend_from_slice(&value.to_le_bytes());}
    }
    out.extend_from_slice(hash_bytes(&out).as_bytes());
    Ok(out)
}
fn check_result_bytes(bytes:&[u8])->Result<usize,String>{
    let magic=b"PNP-TERNARY-KERNEL-RESULTS-v1\0";
    if bytes.len()<magic.len()+2+64 || !bytes.starts_with(magic){
        return Err("kernel witness header absent".into());
    }
    let (body,check)=bytes.split_at(bytes.len()-64);
    if hash_bytes(body).as_bytes()!=check {
        return Err("kernel witness checksum mismatch".into());
    }
    let n=usize::from(u16::from_le_bytes([body[magic.len()],body[magic.len()+1]]));
    let unit=37+8*CELLS;
    if n!=TASKS || body.len()!=magic.len()+2+n*unit {
        return Err("kernel witness dimensions invalid".into());
    }
    for index in 0..n {
        let position=magic.len()+2+index*unit;
        let task_bytes=&body[position..position+37];
        let task=Task{id:task_bytes[1],seed:u64::from_le_bytes(task_bytes[2..10]
            .try_into().map_err(|_|"witness task seed")?),
            trits:task_bytes[10..37].try_into().map_err(|_|"witness trit count")?};
        if task_bytes[0]!=1 || usize::from(task.id)!=index
            || task.trits.iter().any(|&x|x>2){
            return Err("kernel witness task invalid".into());
        }
        let exact=compute_primary(&task);
        for (cell,value) in exact.iter().enumerate(){
            let at=position+37+cell*8;
            let found=u64::from_le_bytes(body[at..at+8]
                .try_into().map_err(|_|"kernel witness cell")?);
            if found!=*value{return Err("kernel witness integer replay mismatch".into());}
        }
    }
    Ok(n)
}
fn line(hbp:&mut String,hbi:&mut String,previous:&mut String,
        seq:usize,tag:&str,id:&str,fields:&str){
    let row=format!("{tag}|seq={seq}|id={id}|{fields}|previous_hash={previous}");
    let digest=hash_bytes(row.as_bytes());
    let new=hash_bytes(format!("{previous}|{digest}").as_bytes());
    let wire=format!("{row}|receipt_sha={digest}|hash={new}|json=0\n");
    hbi.push_str(&format!("HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={new}|json=0\n",
        hbp.len(),wire.len(),hash_bytes(wire.as_bytes()),&digest[..16]));
    hbp.push_str(&wire);
    *previous=new;
}
type Bundle=(String,String,String,Vec<u8>);
fn execute(source:&[u8])->Result<(Bundle,Stats),String>{
    let tasks=source_tasks(source)?;
    let expected:Vec<[u64;CELLS]>=tasks.iter().map(compute_primary).collect();
    let source_sha=hash_bytes(source);
    let witness=output_bytes(&tasks,&expected)?;
    if check_result_bytes(&witness)?!=TASKS{
        return Err("kernel result archive did not independently parse".into());
    }
    let result_sha=hash_bytes(&witness);
    let mut reference_ops=0_usize;
    let started=Instant::now();
    for pass in 0..PASSES{
        for i in 0..TASKS{
            let at=permutation(pass,i);
            let got=compute_primary(&tasks[at]);
            if got!=expected[at]{return Err("unmemoized worker disagreed".into());}
            reference_ops+=1;
        }
    }
    let direct_ns=started.elapsed().as_nanos();
    let started=Instant::now();
    let mut direct_memo:BTreeMap<u8,[u64;CELLS]>=BTreeMap::new();
    let mut direct_hits=0_usize;
    for pass in 0..PASSES{
        for i in 0..TASKS{
            let at=permutation(pass,i);
            let task=&tasks[at];
            let result=if let Some(known)=direct_memo.get(&task.id){
                direct_hits+=1;
                *known
            }else{
                let calculated=compute_primary(task);
                direct_memo.insert(task.id,calculated);
                calculated
            };
            if result!=expected[at]{return Err("direct recall changed output".into());}
        }
    }
    let direct_cache_ns=started.elapsed().as_nanos();
    if reference_ops!=162 || direct_hits!=108 || direct_memo.len()!=TASKS{
        return Err("direct/cache control count changed".into());
    }
    let mut arms=Vec::with_capacity(9);
    for arity in [3_usize,27,81]{
        for mode in 1..=3_u8{
            let client=Client::start()?;
            if !client.mutated_route_rejected(&tasks[0],arity)?
                || !client.mutated_trit_rejected(&tasks[0],arity)?{
                return Err("independent third-seat routing/trit mutation accepted".into());
            }
            let begin=Instant::now();
            let mut counted_hits=0_usize;
            for pass in 0..PASSES{
                for i in 0..TASKS{
                    let index=permutation(pass,i);
                    let task=&tasks[index];
                    let lane=if mode==3{hrm_proxy_lane(task,arity)}
                        else{usize::from(task.id)%arity};
                    let actual=client.execute(mode,arity,task,lane)?;
                    if actual.values!=expected[index]{
                        return Err(format!("fabric independent 243-step compute differed at mode={mode} arity={arity} task={index}"));
                    }
                    counted_hits+=usize::from(actual.hit);
                }
            }
            let elapsed_ns=begin.elapsed().as_nanos();
            let stop=client.stop()?;
            if stop.calls!=162 || stop.hits!=counted_hits
                || stop.hits!=if mode==1{0}else{108}
                || stop.entries!=if mode==1{0}else{54}
                || stop.rooms==0 || stop.rooms>arity{
                return Err(format!("fabric storage/recall accounting failed: mode={mode} arity={arity}"));
            }
            arms.push(Arm{mode,arity,elapsed_ns,
                calls:stop.calls,hits:stop.hits,rooms:stop.rooms,entries:stop.entries});
        }
    }
    if arms.len()!=9{return Err("four-way comparison missing an arm".into());}
    let executable=fs::metadata(std::env::current_exe().map_err(|e|e.to_string())?)
        .map_err(|e|e.to_string())?.len() as usize;
    let mut hbp=HBP_HEADER.to_owned();
    let mut hbi=HBI_HEADER.to_owned();
    let mut previous=hash_bytes(b"PNP-MICROKERNEL-COMPUTE-GENESIS");
    line(&mut hbp,&mut hbi,&mut previous,1,"COMPUTE_SOURCE","frozen",
        &format!("source_sha256={source_sha}|witness_sha256={result_sha}|tasks=54|repetition_orders=FORWARD_REVERSE_ROTATE17|calls_per_arm=162|trits_per_task=27|ticks=243|exact_modulus={RIME_PRIME}|source=PR43_ACCEPTED"),
    );
    line(&mut hbp,&mut hbi,&mut previous,2,"COMPUTE_ARM","native_direct",
        "backend=IN_PROCESS|calls=162|compute_ops=162|cache_hits=0|exact_witness=PASS");
    line(&mut hbp,&mut hbi,&mut previous,3,"COMPUTE_ARM","native_direct_recall",
        "backend=IN_PROCESS|calls=162|compute_ops=54|cache_hits=108|memo_entries=54|exact_witness=PASS");
    for (i,arm) in arms.iter().enumerate(){
        let label=match arm.mode{1=>"fischer_rooms",2=>"fischer_recall",_=>"fischer_hrm_proxy_recall"};
        line(&mut hbp,&mut hbi,&mut previous,i+4,"COMPUTE_ARM",
            &format!("{label}_{}",arm.arity),
            &format!("arity={}|calls={}|compute_ops={}|cache_hits={}|visited_rooms={}|memo_entries={}|http0=BACKEND_ONLY|separate_native_process=YES|independent_stencil=PASS|source_events_not_sent=YES|fischer_route=MODULAR_INDEPENDENT|hrm_checkpoint=NOT_LOADED",
                arm.arity,arm.calls,arm.calls-arm.hits,arm.hits,arm.rooms,arm.entries));
    }
    line(&mut hbp,&mut hbi,&mut previous,13,"COMPUTE_SUMMARY","verdict",
        &format!("source_sha256={source_sha}|result_sha256={result_sha}|jobs=54|verified_reconstruction_calls={}|reused_result_cache=SOURCE_BOUND|logical_cache_bytes_per_54_entries={}|third_seat=SEPARATE_PROCESS|hrm_shape_proxy=DETERMINISTIC|trained_hrm=HOLD_NOT_LOADED|source_retained=YES|native_program_bytes={executable}|status=PASS",
            arms.iter().map(|x|x.calls).sum::<usize>(),54*(37+CELLS*8)));
    let basis=format!("COMPUTE_ANCHOR|rows=13|source_sha256={source_sha}|witness_sha256={result_sha}|hbp_sha256={}|hbi_sha256={}|last_hash={previous}|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes()));
    let anchor=format!("{basis}|checksum={}|json=0\n",hash_bytes(basis.as_bytes()));
    Ok(((hbp,hbi,anchor,witness),Stats{
        source_sha,result_sha,jobs:54,direct_ns,direct_cache_ns,arms,
        full_binary_bytes:executable,
    }))
}
fn file(path:&Path,contents:&[u8])->Result<(),String>{
    let mut f=File::create(path).map_err(|e|e.to_string())?;
    f.write_all(contents).map_err(|e|e.to_string())?;
    f.sync_all().map_err(|e|e.to_string())
}
pub fn attest(prior:&Path,out:&Path)->Result<Stats,String>{
    let source=fs::read(prior.join("PNP_TLS_STOCHASTIC_TRACE.bin"))
        .map_err(|e|e.to_string())?;
    let ((hbp,hbi,anchor,witness),stats)=execute(&source)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,body) in [
        ("PNP_MICROKERNEL_COMPUTE.hbp",hbp.as_bytes()),
        ("PNP_MICROKERNEL_COMPUTE.hbi",hbi.as_bytes()),
        ("PNP_MICROKERNEL_COMPUTE.anchor",anchor.as_bytes()),
        ("PNP_MICROKERNEL_RESULTS.bin",witness.as_slice()),
    ]{file(&out.join(name),body)?;}
    Ok(stats)
}
pub fn verify(prior:&Path,out:&Path)->Result<Stats,String>{
    let source=fs::read(prior.join("PNP_TLS_STOCHASTIC_TRACE.bin"))
        .map_err(|e|e.to_string())?;
    let ((hbp,hbi,anchor,witness),stats)=execute(&source)?;
    for (name,expected) in [
        ("PNP_MICROKERNEL_COMPUTE.hbp",hbp.as_bytes()),
        ("PNP_MICROKERNEL_COMPUTE.hbi",hbi.as_bytes()),
        ("PNP_MICROKERNEL_COMPUTE.anchor",anchor.as_bytes()),
        ("PNP_MICROKERNEL_RESULTS.bin",witness.as_slice()),
    ]{
        if fs::read(out.join(name)).map_err(|e|e.to_string())?!=expected{
            return Err(format!("microkernel archive or witness changed: {name}"));
        }
    }
    Ok(stats)
}
pub fn verify_witness_file(path:&Path)->Result<usize,String>{
    let data=fs::read(path).map_err(|e|e.to_string())?;
    check_result_bytes(&data)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn exact_ternary_27cell_243step_and_canonical_result(){
        let t=Task{id:0,seed:17,trits:[2_u8;CELLS]};
        let first=compute_primary(&t);
        assert!(first.iter().all(|v|*v<RIME_PRIME));
        assert!(hrm_proxy_lane(&t,3)<3);
        assert!(hrm_proxy_lane(&t,27)<27);
        assert!(hrm_proxy_lane(&t,81)<81);
        let tasks=(0..TASKS).map(|id|Task{id:id as u8,
            seed:id as u64+17,trits:[(id%3)as u8;CELLS]}).collect::<Vec<_>>();
        let out=tasks.iter().map(compute_primary).collect::<Vec<_>>();
        let witness=output_bytes(&tasks,&out).unwrap();
        assert_eq!(check_result_bytes(&witness).unwrap(),TASKS);
        let mut damaged=witness;
        damaged[80]^=1;
        assert!(check_result_bytes(&damaged).is_err());
        for i in 0..TASKS {
            assert_eq!(permutation(1,i),TASKS-1-i);
        }
    }
}
