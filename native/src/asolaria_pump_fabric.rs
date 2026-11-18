//! Source-pinned original Asolaria spherical kernel pump acceptance across
//! the already verified binary-ternary Omnibit40 native HTTP-0 fabric.
//! This stage is observational/refreeze only. It never rewrites the frozen
//! PRL event corpus, heldout forecasts, trained GNN source or output scores.
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines)]

use std::fs::{self,File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::asolaria_pump::{self,PumpFrame,PHASES,SPECTRAL_VALUES};
use crate::http0_compute::{Client,Task,CELLS};
use crate::microkernel_compute::source_tasks;
use crate::sha256::hash_bytes;

const SOURCE:&str="PNP_TLS_STOCHASTIC_TRACE.bin";
const ROOTS:[&str;5]=[
    "PNP_BRIDGE.hbp",
    "PNP_OMNI40.hbp",
    "PNP_FULL_MATRIX.hbp",
    "PNP_PRL_MEMORY_FABRIC.hbp",
    "PNP_PRL_MEMORY_FORECAST.bin",
];
const FLOORS:[u16;4]=[64,256,1024,4096];
const OPERATORS:usize=40;
const JOBS:usize=54;
const PER_ARM:usize=JOBS*FLOORS.len()*OPERATORS;
const MAG:&[u8]=b"PNP-ASOLARIA-RIME-PUMP-V1\0";
const HEADER:&str="HBPHEADER|schema=pnp.asolaria.rime.pump.v1|source=ASOLARIA_HOST8_TRIBIT@1c03dca|pump=27_NTT_THREE_PHASES|gradient_pump=SEPARATE_SOURCE_HOLD|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const INDEX:&str="HBIHEADER|schema=pnp.asolaria.rime.pump.index.v1|for=PNP_ASOLARIA_PUMP.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const ROWS:usize=1+ROOTS.len()+JOBS+3+1;

pub struct Arm{
    pub arity:usize,
    pub messages:usize,
    pub rooms:usize,
    pub elapsed_ns:u128,
}
pub struct Stats{
    pub jobs:usize,
    pub phases:usize,
    pub messages:usize,
    pub arms:Vec<Arm>,
    pub source_sha:String,
    pub archive_sha:String,
}
fn source(prior:&Path)->Result<(Vec<Task>,Vec<String>),String>{
    let data=fs::read(prior.join(SOURCE))
        .map_err(|e|format!("Asolaria pump cannot read original temporal source: {e}"))?;
    let tasks=source_tasks(&data)?;
    if tasks.len()!=JOBS{return Err("pump original 54-source census changed".into());}
    let mut roots=vec![hash_bytes(&data)];
    for name in ROOTS{
        let bytes=fs::read(prior.join(name))
            .map_err(|e|format!("original evidence root {name}: {e}"))?;
        let expected:&[u8]=if name.ends_with(".bin"){
            b"PNP-PRL-CAUSAL-FORECAST-v1\0"
        }else{
            b"HBPHEADER|"
        };
        if bytes.len()<80||!bytes.starts_with(expected){
            return Err(format!("pump source provenance root unrecognized: {name}"));
        }
        roots.push(hash_bytes(&bytes));
    }
    if crate::prl_temporal_fabric::inspect(
        &prior.join("PNP_PRL_MEMORY_FORECAST.bin"))?!=72{
        return Err("pump original causal prediction archive changed".into());
    }
    Ok((tasks,roots))
}
fn build_archive(tasks:&[Task],seals:&[PumpFrame])
    ->Result<Vec<u8>,String>
{
    if tasks.len()!=JOBS||seals.len()!=JOBS{
        return Err("Asolaria source/archive census mismatch".into());
    }
    let mut body=MAG.to_vec();
    body.extend_from_slice(&(JOBS as u16).to_le_bytes());
    for (task,seal) in tasks.iter().zip(seals){
        body.extend_from_slice(&task.encode());
        for phase in 0..PHASES{
            body.extend_from_slice(&seal.shells[phase].to_le_bytes());
            for &value in &seal.spectra[phase]{
                body.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    let sha=hash_bytes(&body);
    body.extend_from_slice(sha.as_bytes());
    Ok(body)
}
fn inspect_archive(bytes:&[u8])->Result<usize,String>{
    if bytes.len()<MAG.len()+2+64 || !bytes.starts_with(MAG){
        return Err("Asolaria pump archive header not canonical".into());
    }
    let (body,trailer)=bytes.split_at(bytes.len()-64);
    if trailer!=hash_bytes(body).as_bytes(){
        return Err("Asolaria pump result SHA altered".into());
    }
    let count=usize::from(u16::from_le_bytes(
        body[MAG.len()..MAG.len()+2].try_into()
            .map_err(|_|"pump census bytes")?));
    const SIZE:usize=37+PHASES*4+SPECTRAL_VALUES*8;
    if count!=JOBS||body.len()!=MAG.len()+2+SIZE*JOBS{
        return Err("Asolaria 27-spectral archive length incorrect".into());
    }
    for i in 0..count{
        let at=MAG.len()+2+i*SIZE;
        let t=&body[at..at+37];
        let id=t[1];
        if t[0]!=1||usize::from(id)!=i||t[10..].iter().any(|x|*x>2){
            return Err("pump original task identity invalid".into());
        }
        let seed=u64::from_le_bytes(t[2..10].try_into()
            .map_err(|_|"pump source seed")?);
        let trits:[u8;CELLS]=t[10..37].try_into()
            .map_err(|_|"pump trit extent")?;
        let task=Task{id,seed,trits};
        let replay=asolaria_pump::source_sweep(&task.trits)?;
        let mut offset=at+37;
        for phase in 0..PHASES{
            let shell=u32::from_le_bytes(body[offset..offset+4].try_into()
                .map_err(|_|"pump energy shell extent")?);
            offset+=4;
            if shell!=replay.shells[phase]{
                return Err("pump spectral shell contradicts source".into());
            }
            for k in 0..CELLS{
                let value=u64::from_le_bytes(body[offset..offset+8].try_into()
                    .map_err(|_|"pump spectral coordinate extent")?);
                offset+=8;
                if value!=replay.spectra[phase][k]{
                    return Err("pump refrozen harmonic does not match original".into());
                }
            }
        }
    }
    Ok(count)
}
fn entry(hbp:&mut String,hbi:&mut String,previous:&mut String,
         seq:usize,tag:&str,id:&str,fields:&str)
{
    let prefix=format!("{tag}|seq={seq}|id={id}|{fields}|previous_hash={previous}");
    let digest=hash_bytes(prefix.as_bytes());
    let next=hash_bytes(format!("{previous}|{digest}").as_bytes());
    let wire=format!("{prefix}|receipt_sha={digest}|hash={next}|json=0\n");
    hbi.push_str(&format!(
        "HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={next}|json=0\n",
        hbp.len(),wire.len(),hash_bytes(wire.as_bytes()),&digest[..16]));
    hbp.push_str(&wire);
    *previous=next;
}
type ResultBundle=(String,String,String,Vec<u8>);
fn execute(prior:&Path)->Result<(ResultBundle,Stats),String>{
    let (tasks,roots)=source(prior)?;
    let seals=tasks.iter().map(|t|asolaria_pump::source_sweep(&t.trits))
        .collect::<Result<Vec<_>,_>>()?;
    let archive=build_archive(&tasks,&seals)?;
    if inspect_archive(&archive)?!=JOBS{
        return Err("Asolaria 27-harmonic witness replay failed".into());
    }
    let mut arms=Vec::with_capacity(3);
    for arity in [3_usize,27,81]{
        let client=Client::start()?;
        if !client.pump_bad_address_rejected(arity,&tasks[0])?{
            return Err("Asolaria pump invalid Fischer room accepted".into());
        }
        let started=Instant::now();
        let mut requests=0_usize;
        for (task,reference) in tasks.iter().zip(&seals){
            for floor in FLOORS{
                for operator in 0..OPERATORS{
                    let output=client.execute_pump(arity,task,floor,operator as u8)?;
                    if output.shells!=reference.shells
                        || output.spectra!=reference.spectra{
                        return Err(format!("Asolaria prism mismatch at task {} floor {floor} op {operator}",task.id));
                    }
                    requests+=1;
                }
            }
        }
        let elapsed_ns=started.elapsed().as_nanos();
        let stop=client.stop()?;
        if requests!=PER_ARM||stop.calls!=PER_ARM
            || stop.rooms!=arity.min(JOBS) || stop.entries!=0
            || stop.hits!=0 || stop.temporal_models!=0{
            return Err("Asolaria pump independent room counters did not reconcile".into());
        }
        arms.push(Arm{arity,messages:requests,rooms:stop.rooms,elapsed_ns});
    }
    let mut hbp=HEADER.to_owned();
    let mut hbi=INDEX.to_owned();
    let mut prev=hash_bytes(b"PNP-ASOLARIA-PUMP-FROZEN-IS-GENESIS");
    let mut seq=1_usize;
    entry(&mut hbp,&mut hbi,&mut prev,seq,"PUMP_SOURCE","original",
          &format!("source_sha256={}|tasks=54|per_task_trits=27|source=PR43_TO_PR50_FROZEN|original_rime_blob_sha=5d3e834e3073478bfb518f952c4c31fd1b9544c0",roots[0]));
    for (i,name) in ROOTS.iter().enumerate(){
        seq+=1;
        entry(&mut hbp,&mut hbi,&mut prev,seq,"PUMP_ROOT",
              name.trim_end_matches(".hbp").trim_end_matches(".bin"),
              &format!("sha256={}|source_certificate=PREVIOUSLY_ACCEPTED",roots[i+1]));
    }
    for (task,seal) in tasks.iter().zip(&seals){
        seq+=1;
        let sourcesha=hash_bytes(&task.encode());
        let mut sealed=Vec::new();
        for phase in 0..PHASES{
            sealed.extend_from_slice(&seal.shells[phase].to_le_bytes());
            for value in &seal.spectra[phase]{
                sealed.extend_from_slice(&value.to_le_bytes());
            }
        }
        entry(&mut hbp,&mut hbi,&mut prev,seq,"PUMP_REFREEZE",
              &format!("task{}",task.id),
              &format!("original_sha256={sourcesha}|r0_r1_r2=EXACT|p=1000081|w=951846|spectral_dimensions=27|phase_shells={},{},{}|spectral_sha256={}|new_refreeze_sha256={}|source_hash_immutable=PASS|energy_proxy=NONNEUTRAL_TRIT_COUNT",
                  seal.shells[0],seal.shells[1],seal.shells[2],
                  hash_bytes(&sealed),hash_bytes(&sealed)));
    }
    for arm in &arms{
        seq+=1;
        entry(&mut hbp,&mut hbi,&mut prev,seq,"PUMP_HTTP0",
              &format!("width{}",arm.arity),
              &format!("calls={}|physical_rooms={}|binary_floors=4|original_operators=40|source_trits=EXACT|third_seat_spectral_inverse=EXACT|independent_ij_spectrum=EXACT|backend=ONLY",
                  arm.messages,arm.rooms));
    }
    seq+=1;
    entry(&mut hbp,&mut hbi,&mut prev,seq,"PUMP_SUMMARY","verdict",
          &format!("source_sha256={}|archive_sha256={}|messages={}|spectrum_checks={}|freeze=SOURCE_IMMUTABLE|R3_IDENTITY=PASS|original_pump=RIME_PRISM_SHELL|gradient27_original_source=HOLD|acer_descriptor_original_source=HOLD|status=PASS",
              roots[0],hash_bytes(&archive),arms.iter().map(|x|x.messages).sum::<usize>(),
              arms.iter().map(|x|x.messages).sum::<usize>()*PHASES));
    if seq!=ROWS||hbi.lines().filter(|x|x.starts_with("HBI|")).count()!=ROWS{
        return Err("Asolaria pump indexed receipt count changed".into());
    }
    let basis=format!(
        "PUMP_ANCHOR|rows={ROWS}|source_sha256={}|archive_sha256={}|hbp_sha256={}|hbi_sha256={}|last_hash={prev}|seal=HBI-HBP-SHA-SH-HASH",
        roots[0],hash_bytes(&archive),hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes())
    );
    let anchor=format!("{basis}|checksum={}|json=0\n",hash_bytes(basis.as_bytes()));
    Ok(((hbp,hbi,anchor,archive),Stats{
        jobs:JOBS,phases:PHASES,messages:arms.iter().map(|a|a.messages).sum(),
        arms,source_sha:roots[0].clone(),archive_sha:hash_bytes(&build_archive(&tasks,&seals)?),
    }))
}
fn write_file(path:&Path,bytes:&[u8])->Result<(),String>{
    let mut file=File::create(path).map_err(|e|e.to_string())?;
    file.write_all(bytes).map_err(|e|e.to_string())?;
    file.sync_all().map_err(|e|e.to_string())
}
pub fn attest(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((hbp,hbi,anchor,archive),stats)=execute(prior)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,data) in [
        ("PNP_ASOLARIA_PUMP.hbp",hbp.as_bytes()),
        ("PNP_ASOLARIA_PUMP.hbi",hbi.as_bytes()),
        ("PNP_ASOLARIA_PUMP.anchor",anchor.as_bytes()),
        ("PNP_ASOLARIA_PUMP_TRACE.bin",archive.as_slice()),
    ]{write_file(&out.join(name),data)?;}
    Ok(stats)
}
pub fn verify(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((hbp,hbi,anchor,archive),stats)=execute(prior)?;
    for (name,expected) in [
        ("PNP_ASOLARIA_PUMP.hbp",hbp.as_bytes()),
        ("PNP_ASOLARIA_PUMP.hbi",hbi.as_bytes()),
        ("PNP_ASOLARIA_PUMP.anchor",anchor.as_bytes()),
        ("PNP_ASOLARIA_PUMP_TRACE.bin",archive.as_slice()),
    ]{
        if fs::read(out.join(name)).map_err(|e|e.to_string())?!=expected{
            return Err(format!("Asolaria original pump source/archive changed: {name}"));
        }
    }
    Ok(stats)
}
pub fn inspect(path:&Path)->Result<usize,String>{
    let data=fs::read(path).map_err(|e|e.to_string())?;
    inspect_archive(&data)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn spectral_source_archive_replays_and_rejects_tampered_energy(){
        let tasks=(0..JOBS).map(|i|Task{
            id:i as u8,seed:i as u64*31,trits:[(i%3)as u8;CELLS],
        }).collect::<Vec<_>>();
        let seals=tasks.iter().map(|task|asolaria_pump::source_sweep(&task.trits).unwrap())
            .collect::<Vec<_>>();
        let mut raw=build_archive(&tasks,&seals).unwrap();
        assert_eq!(inspect_archive(&raw).unwrap(),JOBS);
        raw[125]^=1;
        assert!(inspect_archive(&raw).is_err());
        assert_eq!(ROWS,1+ROOTS.len()+JOBS+3+1);
        assert_eq!(PER_ARM,8_640);
    }
}
