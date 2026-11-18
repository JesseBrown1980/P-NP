//! Whole-path first integration: frozen source -> four binary floors -> all
//! original 40 Omnibit transforms -> Fischer-addressed backend-only HTTP-0
//! native rooms -> independent 27-trit reconstruction and 243-tick output ->
//! exact triad matrix witness -> source-bound GNN gate and durable HBI/HBP.
//! Trained GNN forward weights remain gated by original-model parity.
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines)]

use std::fs::{self,File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::ensemble::{self,Phase,Signals};
use crate::fabric_gc::{self,FabricTuple,Scope,Tier};
use crate::freeze::FrozenMatrix;
use crate::http0_compute::{Client,Task,CELLS};
use crate::matrix::Matrix;
use crate::matrix_wire;
use crate::microkernel_compute::{compute_primary,source_tasks};
use crate::sha256::hash_bytes;
use crate::triad::wave_cascade;

const MAGIC:&[u8]=b"PNP-MATRIX-ALL40-V1\0";
const SOURCE:&str="PNP_TLS_STOCHASTIC_TRACE.bin";
const PRIOR_FILES:[&str;6]=[
    "PNP_BRIDGE.hbp",
    "PNP_OMNI40.hbp",
    "PNP_GNN_GATE.hbp",
    "PNP_CHECKPOINT.hbp",
    "PNP_FABRIC_GC.hbp",
    "PNP_MICROKERNEL_RESULTS.bin",
];
const FLOORS:[u16;4]=[64,256,1024,4096];
const OPERATORS:usize=40;
const TASKS:usize=54;
const CALLS_PER_ARM:usize=TASKS*4*OPERATORS;
const HBP_HEAD:&str="HBPHEADER|schema=pnp.full.matrix.fabric.v1|transport=HTTP0_BACKEND_ONLY|trained_gnn=HOLD_FORWARD|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HBI_HEAD:&str="HBIHEADER|schema=pnp.full.matrix.fabric.index.v1|for=PNP_FULL_MATRIX.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Arm{
    pub mode:u8,
    pub arity:usize,
    pub elapsed_ns:u128,
    pub calls:usize,
    pub hits:usize,
    pub rooms:usize,
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Stats{
    pub jobs:usize,
    pub crossings:usize,
    pub independent_calls:usize,
    pub matrix_triads:usize,
    pub tuple_checks:usize,
    pub gnn_hold_checks:usize,
    pub direct_all_ns:u128,
    pub direct_memo_ns:u128,
    pub arms:Vec<Arm>,
}

fn witnesses(tasks:&[Task],expected:&[[u64;CELLS]],source_sha:&str)
    ->Result<(usize,usize,usize),String>
{
    let mut triads=0;
    for section in expected.chunks_exact(3){
        let rooms=section.iter().map(|result|{
            let mut values=[0_i64;CELLS];
            for (cell,value) in result.iter().enumerate(){
                values[cell]=i64::try_from(*value).expect("all values are below prime");
            }
            values
        }).collect::<Vec<_>>();
        let matrix=Matrix::from_rooms(rooms).map_err(str::to_owned)?;
        let frozen=FrozenMatrix::freeze(&matrix);
        if !frozen.verify(){return Err("source matrix freeze failed".into());}
        let report=wave_cascade(&frozen,None)
            .ok_or("full matrix triad refused original source")?;
        if !report.accepted || report.worker_total!=report.reflection_total
            || report.worker_total!=report.fabric_total{
            return Err("full matrix third-seat reconstruction disagreed".into());
        }
        triads+=1;
    }
    if triads!=18{return Err("not all source matrix triads were checked".into());}
    let mut tuple_count=0;
    let mut gnn_count=0;
    for (i,task) in tasks.iter().enumerate(){
        let mut coordinates=[0_i64;60];
        coordinates[0]=i as i64;
        coordinates[1]=27;
        coordinates[2]=243;
        coordinates[3]=i64::from(task.id);
        coordinates[4]=i64::from(task.trits[13])-1;
        let tuple=FabricTuple{
            coordinates,tier:Tier::Public,scope:Scope::Pid,
            source_digest:source_sha.to_owned(),
        };
        let encoded=tuple.encode().map_err(str::to_owned)?;
        let parsed=encoded.split("|coordinates=").nth(1)
            .and_then(|s|s.split('|').next())
            .and_then(FabricTuple::parse_coordinates);
        if parsed!=Some(coordinates)
            || tuple.access_cell()!=5
            || fabric_gc::invert_access_cell(5)!=Some((Tier::Public,Scope::Pid)){
            return Err("frozen GAC 60D metadata tuple mismatched".into());
        }
        tuple_count+=1;
        // G4 is the authentic, source-derived state machine. Its required
        // G1/G2/G3 rows are missing until actual checkpoint forward parity.
        // Therefore no synthetic values may promote a Fischer Tier-0 action.
        let held=ensemble::evaluate(Phase::Described,Signals{
            edge:None,path:None,gain:None,g1:None,g2:None,g3:None,
        });
        if held.next!=Phase::Described || held.emitted
            || held.tier0_block || !held.operator_held{
            return Err("trained GNN absent-signal hold was bypassed".into());
        }
        gnn_count+=1;
    }
    Ok((triads,tuple_count,gnn_count))
}
fn source_and_roots(prior:&Path)->Result<(Vec<Task>,Vec<String>),String>{
    let source=fs::read(prior.join(SOURCE))
        .map_err(|e|format!("frozen temporal source: {e}"))?;
    let tasks=source_tasks(&source)?;
    if tasks.len()!=TASKS{return Err("original P-NP source task census changed".into());}
    let mut roots=vec![hash_bytes(&source)];
    for filename in PRIOR_FILES {
        let bytes=fs::read(prior.join(filename))
            .map_err(|e|format!("pinned gate {filename}: {e}"))?;
        let expected_header:&[u8]=if filename.ends_with(".bin"){
            b"PNP-TERNARY-KERNEL-RESULTS-v1\0"
        }else{
            b"HBPHEADER|"
        };
        if bytes.len()<80 || !bytes.starts_with(expected_header){
            return Err(format!("prior original plane missing: {filename}"));
        }
        roots.push(hash_bytes(&bytes));
    }
    if crate::microkernel_compute::verify_witness_file(
        &prior.join("PNP_MICROKERNEL_RESULTS.bin"))?!=TASKS{
        return Err("prior 54-task native witness failed".into());
    }
    Ok((tasks,roots))
}
fn output_bytes(tasks:&[Task],values:&[[u64;CELLS]])->Result<Vec<u8>,String>{
    let mut body=MAGIC.to_vec();
    body.extend_from_slice(&u16::try_from(tasks.len())
        .map_err(|_|"matrix task count overflow")?.to_le_bytes());
    for (task,result) in tasks.iter().zip(values){
        body.extend_from_slice(&task.encode());
        for value in result {body.extend_from_slice(&value.to_le_bytes());}
    }
    body.extend_from_slice(hash_bytes(&body).as_bytes());
    Ok(body)
}
fn inspect_bytes(raw:&[u8])->Result<usize,String>{
    if raw.len()<MAGIC.len()+2+64 || !raw.starts_with(MAGIC){
        return Err("full matrix witness header missing".into());
    }
    let (body,trailer)=raw.split_at(raw.len()-64);
    if hash_bytes(body).as_bytes()!=trailer{
        return Err("full matrix witness SHA changed".into());
    }
    let count=usize::from(u16::from_le_bytes([
        body[MAGIC.len()],body[MAGIC.len()+1]
    ]));
    let record=37+CELLS*8;
    if count!=TASKS || body.len()!=MAGIC.len()+2+count*record{
        return Err("full matrix witness frame extent invalid".into());
    }
    for index in 0..count{
        let at=MAGIC.len()+2+index*record;
        let data=&body[at..at+37];
        if data[0]!=1 || usize::from(data[1])!=index
            || data[10..].iter().any(|t|*t>2){
            return Err("full matrix witness source tag invalid".into());
        }
        let seed=u64::from_le_bytes(data[2..10].try_into()
            .map_err(|_|"full matrix witness seed malformed")?);
        let trits:[u8;CELLS]=data[10..37].try_into()
            .map_err(|_|"full matrix witness ternary extent")?;
        let source=Task{id:index as u8,seed,trits};
        let expected=compute_primary(&source);
        for (cell,true_value) in expected.iter().enumerate(){
            let offset=at+37+cell*8;
            let found=u64::from_le_bytes(body[offset..offset+8]
                .try_into().map_err(|_|"full matrix result extent")?);
            if found!=*true_value{
                return Err("full matrix independently reconstructed value differs".into());
            }
        }
    }
    Ok(count)
}
fn add(hbp:&mut String,hbi:&mut String,last:&mut String,
       index:usize,tag:&str,id:&str,detail:&str)
{
    let bare=format!("{tag}|seq={index}|id={id}|{detail}|previous_hash={last}");
    let digest=hash_bytes(bare.as_bytes());
    let next=hash_bytes(format!("{last}|{digest}").as_bytes());
    let row=format!("{bare}|receipt_sha={digest}|hash={next}|json=0\n");
    hbi.push_str(&format!(
        "HBI|seq={index}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={next}|json=0\n",
        hbp.len(),row.len(),hash_bytes(row.as_bytes()),&digest[..16]
    ));
    hbp.push_str(&row);
    *last=next;
}
type Bundle=(Vec<u8>,String,String,String);
fn execute(prior:&Path)->Result<(Bundle,Stats),String>{
    let (tasks,roots)=source_and_roots(prior)?;
    let expected=tasks.iter().map(compute_primary).collect::<Vec<_>>();
    let (matrix_triads,tuple_checks,gnn_hold_checks)=witnesses(&tasks,&expected,&roots[0])?;
    let output=output_bytes(&tasks,&expected)?;
    if inspect_bytes(&output)?!=TASKS{
        return Err("full matrix result file invalid".into());
    }
    let begin=Instant::now();
    let mut direct_all=0_usize;
    for task in &tasks{
        for floor in FLOORS{
            for operator in 0..OPERATORS{
                let framed=matrix_wire::encode(task,floor,operator as u8)?;
                let (decoded,f,o)=matrix_wire::decode_independent(&framed)?;
                if decoded!=*task || f!=floor || o!=operator as u8
                    || compute_primary(&decoded)!=expected[usize::from(task.id)]{
                    return Err("source-side direct whole-path computation differed".into());
                }
                direct_all+=1;
            }
        }
    }
    let direct_all_ns=begin.elapsed().as_nanos();
    if direct_all!=CALLS_PER_ARM{
        return Err("direct all40 source cohort incomplete".into());
    }
    let begin=Instant::now();
    let mut direct_map=std::collections::BTreeMap::new();
    let mut direct_hits=0_usize;
    for task in &tasks{
        for floor in FLOORS{
            for operator in 0..OPERATORS{
                let framed=matrix_wire::encode(task,floor,operator as u8)?;
                let (decoded,_,_)=matrix_wire::decode_independent(&framed)?;
                let v=if let Some(old)=direct_map.get(&decoded.id){
                    direct_hits+=1;
                    *old
                }else{
                    let v=compute_primary(&decoded);
                    direct_map.insert(decoded.id,v);
                    v
                };
                if v!=expected[usize::from(task.id)]{
                    return Err("in-process cross-floor cache changed task".into());
                }
            }
        }
    }
    let direct_memo_ns=begin.elapsed().as_nanos();
    if direct_hits!=CALLS_PER_ARM-TASKS || direct_map.len()!=TASKS{
        return Err("direct cross-floor cache reuse failed".into());
    }
    let mut arms=Vec::with_capacity(6);
    for arity in [3_usize,27,81]{
        for mode in [4_u8,5]{
            let child=Client::start()?;
            let negative=matrix_wire::encode(&tasks[0],64,0)?;
            if child.matrix_negative(arity,&negative)?!=3{
                return Err("independent backend rejected fewer than 3 negative controls".into());
            }
            let started=Instant::now();
            let mut hit_count=0_usize;
            let mut requests=0_usize;
            for task in &tasks{
                let lane=usize::from(task.id)%arity;
                for floor in FLOORS{
                    for operator in 0..OPERATORS{
                        let framed=matrix_wire::encode(task,floor,operator as u8)?;
                        let response=child.execute_matrix(mode,arity,lane,&framed)?;
                        if response.reconstructed!=task.trits
                            || response.computed.values!=expected[usize::from(task.id)]{
                            return Err(format!(
                                "matrix crossing failed: id={} floor={floor} omega={operator} arity={arity} mode={mode}",task.id
                            ));
                        }
                        hit_count+=usize::from(response.computed.hit);
                        requests+=1;
                    }
                }
            }
            let elapsed_ns=started.elapsed().as_nanos();
            let stop=child.stop()?;
            let needed_hits=if mode==4 {0}else{CALLS_PER_ARM-TASKS};
            let needed_entries=if mode==4 {0}else{TASKS};
            if stop.calls!=CALLS_PER_ARM || stop.hits!=hit_count
                || stop.hits!=needed_hits || stop.entries!=needed_entries
                || requests!=CALLS_PER_ARM
                || stop.rooms!=arity.min(TASKS){
                return Err(format!("room compute/recall/physical-width counters wrong at {arity}/{mode}"));
            }
            arms.push(Arm{mode,arity,elapsed_ns,calls:stop.calls,
                hits:stop.hits,rooms:stop.rooms});
        }
    }
    let mut hbp=HBP_HEAD.to_owned();
    let mut hbi=HBI_HEAD.to_owned();
    let mut last=hash_bytes(b"PNP-FULL-MATRIX-FABRIC-GENESIS");
    add(&mut hbp,&mut hbi,&mut last,1,"MATRIX_SOURCE","pr43",
        &format!("sha256={}|jobs=54|trits_per_job=27|ticks_per_kernel=243|prior_task_sha256={}|catalog=ORIGINAL40",roots[0],roots[6]));
    for (i,name) in PRIOR_FILES.iter().enumerate(){
        add(&mut hbp,&mut hbi,&mut last,i+2,"MATRIX_PIN",
            name.trim_end_matches(".hbp").trim_end_matches(".bin"),
            &format!("sha256={}|source=PRIOR_ACCEPTED|checked=YES",roots[i+1]));
    }
    let mut seq=8;
    for arm in &arms{
        add(&mut hbp,&mut hbi,&mut last,seq,"MATRIX_ARM",
            &format!("mode{}_rooms{}",arm.mode,arm.arity),
            &format!("binary_floors=64,256,1024,4096|operators=40|requests={}|reconstruction=ALL27|independent_stencil=243|physical_rooms={}|cache_hits={}|actual_computations={}|original_task_data=NO_SOURCE_BYTES|http0=BACKEND_ONLY|fischer=INDEPENDENT_ADDRESS",
                arm.calls,arm.rooms,arm.hits,arm.calls-arm.hits));
        seq+=1;
    }
    add(&mut hbp,&mut hbi,&mut last,seq,"MATRIX_TRIAD","exact",
        &format!("groups={matrix_triads}|cells_per_room=27|worker=EXACT|reflection=EXACT|frozen_matrix_fabric=EXACT|gac60_metadata_tuple_checks={tuple_checks}|gac_physical_axis_interchange=HOLD"));
    seq+=1;
    add(&mut hbp,&mut hbi,&mut last,seq,"MATRIX_GNN","source_gate",
        &format!("gnn_missing_signal_holds={gnn_hold_checks}|original_nine_checkpoints=PINNED_AND_Q20_IMPORTED|g4_glsm=ORIGINAL_SOURCE_GATE|g1_g2_g3_forward=HOLD|trained_hrm_forward=HOLD|no_synthetic_gnn_signal_promoted=YES"));
    seq+=1;
    add(&mut hbp,&mut hbi,&mut last,seq,"MATRIX_SUMMARY","all",
        &format!("jobs=54|all40_operator_floor_crossings={}|independent_http0_calls={}|floor_counts=4|operator_families=RNQ8_TRI12_PI20|matrix_groups={matrix_triads}|backend=ONLY|recall_hits={}|verified_results_sha256={}|input=FROZEN|status=PASS",
            direct_all,arms.iter().map(|a|a.calls).sum::<usize>(),
            arms.iter().map(|a|a.hits).sum::<usize>(),hash_bytes(&output)));
    if seq!=16{return Err("full matrix receipt count wrong".into());}
    let basis=format!(
        "FULL_MATRIX_ANCHOR|rows={seq}|source_sha256={}|results_sha256={}|hbp_sha256={}|hbi_sha256={}|end_hash={last}|seal=HBI-HBP-SHA-SH-HASH",
        roots[0],hash_bytes(&output),hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes())
    );
    let anchor=format!("{basis}|checksum={}|json=0\n",hash_bytes(basis.as_bytes()));
    let stats=Stats{jobs:TASKS,crossings:direct_all,
        independent_calls:arms.iter().map(|a|a.calls).sum(),
        matrix_triads,tuple_checks,gnn_hold_checks,direct_all_ns,direct_memo_ns,arms};
    Ok(((output,hbp,hbi,anchor),stats))
}
fn persist(path:&Path,content:&[u8])->Result<(),String>{
    let mut file=File::create(path).map_err(|e|e.to_string())?;
    file.write_all(content).map_err(|e|e.to_string())?;
    file.sync_all().map_err(|e|e.to_string())
}
pub fn attest(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((data,hbp,hbi,anchor),stats)=execute(prior)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,content) in [
        ("PNP_FULL_MATRIX_RESULTS.bin",data.as_slice()),
        ("PNP_FULL_MATRIX.hbp",hbp.as_bytes()),
        ("PNP_FULL_MATRIX.hbi",hbi.as_bytes()),
        ("PNP_FULL_MATRIX.anchor",anchor.as_bytes()),
    ]{persist(&out.join(name),content)?;}
    Ok(stats)
}
pub fn verify(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((data,hbp,hbi,anchor),stats)=execute(prior)?;
    for (name,expected) in [
        ("PNP_FULL_MATRIX_RESULTS.bin",data.as_slice()),
        ("PNP_FULL_MATRIX.hbp",hbp.as_bytes()),
        ("PNP_FULL_MATRIX.hbi",hbi.as_bytes()),
        ("PNP_FULL_MATRIX.anchor",anchor.as_bytes()),
    ]{
        if fs::read(out.join(name)).map_err(|e|e.to_string())?!=expected{
            return Err(format!("full matrix published evidence changed: {name}"));
        }
    }
    Ok(stats)
}
pub fn inspect(path:&Path)->Result<usize,String>{
    let bytes=fs::read(path).map_err(|e|e.to_string())?;
    inspect_bytes(&bytes)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn independent_matrix_and_gnn_gate_remain_fail_closed(){
        let tasks=(0..TASKS).map(|i|Task{
            id:i as u8,seed:i as u64+1,
            trits:[(i%3)as u8;CELLS],
        }).collect::<Vec<_>>();
        let expected=tasks.iter().map(compute_primary).collect::<Vec<_>>();
        assert_eq!(witnesses(&tasks,&expected,&hash_bytes(b"fixture")).unwrap(),(18,54,54));
        let body=output_bytes(&tasks,&expected).unwrap();
        assert_eq!(inspect_bytes(&body).unwrap(),54);
        let mut corrupted=body;
        corrupted[72]^=1;
        assert!(inspect_bytes(&corrupted).is_err());
    }
}
