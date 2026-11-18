//! Bitcoin genesis SHA256d bit-blast SAT: source-pinned comparison with
//! frozen Rime disk addressing and a batched Pump-Funnel control.
//!
//! Only original 2026-11-17 CNF bytes are accepted. One shared watched-literal
//! Boolean SAT kernel is used for every arm, with binary decisions encoded as
//! signed ternary (-1,0,+1). Time/node budgets produce explicit HOLD.
//! Generated CNFs and existing seven-arm historical source stay unchanged.
#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use crate::freeze::fischer_address;
use crate::rime_disk::{self, DiskRime};
use crate::sha256::hash_bytes;

const EXPECTED_DIGEST: &str =
    "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
const MERKLE: &str =
    "3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a";
const GENESIS_NONCE: u32 = 2_083_236_893;

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    bits: usize,
    variables: usize,
    clauses: usize,
    digest: &'static str,
    nodes: u64,
    seconds: u64,
}
const CASES: [Case; 4] = [
    Case {
        name:"nonce4", bits:4, variables:236_344, clauses:867_665,
        digest:"204e9e08c5fb1a5f2555217c87bceb3385f9dc70405790015fd04393cb95d2aa",
        nodes:64, seconds:8,
    },
    Case {
        name:"nonce8", bits:8, variables:236_931, clauses:869_855,
        digest:"9968b60351e29037d9c7183f8c9b59a48f2da0be7b9c393d5013a503034ffa9a",
        nodes:512, seconds:12,
    },
    Case {
        name:"nonce32", bits:32, variables:239_112, clauses:877_879,
        digest:"6fe3e686c0b4c5dbd18a550c0b28c10ec6cabd41b02367a1fb6747eac49f88d3",
        nodes:96, seconds:5,
    },
    Case {
        name:"merkle_nonce288", bits:288, variables:361_188, clauses:1_325_357,
        digest:"601412f3d918c401b00df22a1b2799e50811fdbf2f1e87602211d7fb1fa6bb4b",
        nodes:96, seconds:5,
    },
];

struct Source {
    count: usize,
    clauses: Vec<Vec<i32>>,
}
impl Source {
    fn from_original(path: &Path, case: Case) -> Result<(Self, u128), String> {
        let started = Instant::now();
        let bytes = fs::read(path).map_err(|e| format!("original CNF read: {e}"))?;
        if hash_bytes(&bytes) != case.digest {
            return Err(format!("Bitcoin original CNF hash mismatch: {}",case.name));
        }
        let raw = std::str::from_utf8(&bytes).map_err(|_| "non-ASCII CNF")?;
        let mut count = 0_usize;
        let mut declared = 0_usize;
        let mut found_header = false;
        let mut clauses = Vec::with_capacity(case.clauses);
        let mut clause = Vec::new();
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("c ") { continue; }
            if trimmed.starts_with("p ") {
                if found_header { return Err("multiple CNF headers".into()); }
                let tokens: Vec<_> = trimmed.split_ascii_whitespace().collect();
                if tokens.len() != 4 || tokens[0] != "p" || tokens[1] != "cnf" {
                    return Err("invalid Bitcoin DIMACS header".into());
                }
                count = tokens[2].parse().map_err(|_| "bad variable count")?;
                declared = tokens[3].parse().map_err(|_| "bad clause count")?;
                if (count, declared) != (case.variables, case.clauses) {
                    return Err("Bitcoin source dimension changed".into());
                }
                found_header = true;
                continue;
            }
            if !found_header { return Err("CNF clause before header".into()); }
            for word in trimmed.split_ascii_whitespace() {
                let lit: i32 = word.parse().map_err(|_| "invalid signed literal")?;
                if lit == 0 {
                    if clause.is_empty() { return Err("Bitcoin empty clause".into()); }
                    clauses.push(std::mem::take(&mut clause));
                    if clauses.len() > declared { return Err("excess original clauses".into()); }
                } else {
                    if usize::try_from(lit.unsigned_abs()).map_err(|_| "variable width")? > count {
                        return Err("original literal out of range".into());
                    }
                    clause.push(lit);
                }
            }
        }
        if !found_header || !clause.is_empty() || clauses.len() != declared {
            return Err("original Bitcoin CNF incomplete".into());
        }
        Ok((Self{count,clauses},started.elapsed().as_nanos()))
    }
    fn check(&self, values: &[i8]) -> bool {
        values.len() == self.count+1
            && self.clauses.iter().all(|row|row.iter().any(|&lit|{
                let v=values[lit.unsigned_abs() as usize];
                (lit>0 && v==1)||(lit<0 && v== -1)
            }))
    }
}

fn slot(lit:i32)->usize {
    let pos=lit.unsigned_abs() as usize-1;
    2*pos+usize::from(lit<0)
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome { Sat, Unsat, HoldNodes, HoldTime }
impl Outcome {
    fn label(self) -> &'static str {
        match self {
            Self::Sat=>"SAT", Self::Unsat=>"UNSAT",
            Self::HoldNodes=>"HOLD_NODES", Self::HoldTime=>"HOLD_TIME",
        }
    }
}
enum Propagation { Ok, Conflict, Time }
struct Watched<'a> {
    source: &'a Source,
    values: Vec<i8>,
    trail: Vec<i32>,
    head: usize,
    w0: Vec<usize>,
    w1: Vec<usize>,
    lists: Vec<Vec<u32>>,
    nodes: u64,
    checks: u64,
    propagations: u64,
    max_nodes: u64,
    until: Instant,
    order: &'a [usize],
}
impl<'a> Watched<'a> {
    fn new(source: &'a Source, order: &'a [usize], cap: u64, secs: u64)
        -> Result<Self,String>
    {
        if order.len() != source.count { return Err("incomplete branch order".into()); }
        let mut seen=vec![false;source.count+1];
        for &v in order {
            if v==0 || v>source.count || seen[v] { return Err("bad branch permutation".into()); }
            seen[v]=true;
        }
        let mut solver=Self {
            source, values:vec![0;source.count+1],
            trail:Vec::with_capacity(source.count),
            head:0,
            w0:vec![0;source.clauses.len()],w1:vec![0;source.clauses.len()],
            lists:vec![Vec::new();source.count*2],
            nodes:0,checks:0,propagations:0,max_nodes:cap,
            until:Instant::now()+Duration::from_secs(secs),order,
        };
        for (index,row) in source.clauses.iter().enumerate() {
            if row.len()==1 {
                if !solver.enqueue(row[0]) { return Err("original Bitcoin source unit conflict".into()); }
            }else{
                solver.w1[index]=1;
                solver.lists[slot(row[0])].push(index as u32);
                solver.lists[slot(row[1])].push(index as u32);
            }
        }
        Ok(solver)
    }
    fn litval(&self,lit:i32)->i8{
        let value=self.values[lit.unsigned_abs() as usize];
        if lit>0{value}else{-value}
    }
    fn enqueue(&mut self,lit:i32)->bool{
        let variable=lit.unsigned_abs() as usize;
        let requested=if lit>0{1}else{-1};
        if self.values[variable]!=0{return self.values[variable]==requested;}
        self.values[variable]=requested;
        self.trail.push(lit);
        self.propagations+=1;
        true
    }
    fn propagate(&mut self)->Propagation {
        while self.head<self.trail.len() {
            if Instant::now()>=self.until{return Propagation::Time;}
            let falsified=-self.trail[self.head];
            self.head+=1;
            let index=slot(falsified);
            let mut i=0;
            while i<self.lists[index].len(){
                self.checks+=1;
                if self.checks&8191==0 && Instant::now()>=self.until {
                    return Propagation::Time;
                }
                let ci=self.lists[index][i] as usize;
                let row=&self.source.clauses[ci];
                let j0=self.w0[ci];let j1=self.w1[ci];
                let (first,other_index)=if row[j0]==falsified {(true,j1)}
                    else if row[j1]==falsified {(false,j0)}
                    else {return Propagation::Conflict;};
                let other=row[other_index];
                if self.litval(other)==1 {i+=1;continue;}
                let mut moved=None;
                for (j,&lit) in row.iter().enumerate(){
                    if j==j0||j==j1{continue;}
                    self.checks+=1;
                    if self.litval(lit)!=-1{moved=Some((j,lit));break;}
                }
                if let Some((j,lit))=moved{
                    if first{self.w0[ci]=j;}else{self.w1[ci]=j;}
                    self.lists[index].swap_remove(i);
                    self.lists[slot(lit)].push(ci as u32);
                    continue;
                }
                if self.litval(other)==-1 || !self.enqueue(other) {
                    return Propagation::Conflict;
                }
                i+=1;
            }
        }
        Propagation::Ok
    }
    fn backtrack(&mut self,count:usize){
        while self.trail.len()>count{
            let lit=self.trail.pop().expect("trail known");
            self.values[lit.unsigned_abs() as usize]=0;
        }
        self.head=self.head.min(count);
    }
    fn recurse(&mut self)->Outcome{
        if self.nodes>=self.max_nodes {return Outcome::HoldNodes;}
        if Instant::now()>=self.until{return Outcome::HoldTime;}
        self.nodes+=1;
        match self.propagate(){
            Propagation::Conflict=>return Outcome::Unsat,
            Propagation::Time=>return Outcome::HoldTime,
            Propagation::Ok=>{}
        }
        let next=self.order.iter().copied().find(|&i|self.values[i]==0);
        let Some(variable)=next else{
            return if self.source.check(&self.values){Outcome::Sat}else{Outcome::Unsat};
        };
        let before=self.trail.len();
        let mut hold_nodes=false;
        for branch in [variable as i32,-(variable as i32)] {
            if self.enqueue(branch){
                match self.recurse(){
                    Outcome::Sat=>return Outcome::Sat,
                    Outcome::HoldTime=>{self.backtrack(before);return Outcome::HoldTime;},
                    Outcome::HoldNodes=>hold_nodes=true,
                    Outcome::Unsat=>{}
                }
            }
            self.backtrack(before);
        }
        if hold_nodes{Outcome::HoldNodes}else{Outcome::Unsat}
    }
}

fn decode_hex(raw:&str)->Result<Vec<u8>,String>{
    if raw.len()%2!=0{return Err("odd hex source".into());}
    raw.as_bytes().chunks_exact(2).map(|pair|{
        let s=std::str::from_utf8(pair).map_err(|_|"hex not ASCII")?;
        u8::from_str_radix(s,16).map_err(|_|"bad original hex digit".into())
    }).collect()
}
fn verify_bitcoin_header(values:&[i8],free_bits:usize)->Result<u32,String>{
    let mut header=vec![0_u8;80];
    header[0]=1;
    header[36..68].copy_from_slice(&decode_hex(MERKLE)?);
    header[68..72].copy_from_slice(&1_231_006_505_u32.to_le_bytes());
    header[72..76].copy_from_slice(&0x1d00ffff_u32.to_le_bytes());
    header[76..80].copy_from_slice(&GENESIS_NONCE.to_le_bytes());
    let mut variable=2_usize;
    for byte in 0..80 {
        for bit in (0..8).rev(){
            let nonce_bit=if byte>=76 {Some(8*(byte-76)+bit)}else{None};
            let free_nonce=nonce_bit.is_some_and(|n|n<free_bits.min(32));
            let free_merkle=free_bits==288 && (36..68).contains(&byte);
            if !free_nonce&&!free_merkle{continue;}
            let state=*values.get(variable).ok_or("original Bitcoin source input not assigned")?;
            if state==0{return Err("unassigned Bitcoin free input in SAT model".into());}
            let mask=1_u8<<bit;
            if state==1{header[byte]|=mask;}else{header[byte]&=!mask;}
            variable+=1;
        }
    }
    if variable!=free_bits+2{return Err("input variable map differs from original generator".into());}
    let once=decode_hex(&hash_bytes(&header))?;
    let mut twice=decode_hex(&hash_bytes(&once))?;
    twice.reverse();
    let digest=twice.iter().map(|byte|format!("{byte:02x}")).collect::<String>();
    if digest!=EXPECTED_DIGEST {
        return Err(format!("Bitcoin SHA256d witness mismatch: {digest}"));
    }
    Ok(u32::from_le_bytes(header[76..80].try_into()
        .map_err(|_|"nonce field size")?))
}

#[derive(Clone,Copy)]
enum Arm{Direct,Fischer,RimeDirect,PumpBatch}
impl Arm{
    fn label(self)->&'static str{
        match self{Self::Direct=>"DIRECT",Self::Fischer=>"FISCHER",
                   Self::RimeDirect=>"RIME_DISK",Self::PumpBatch=>"PUMP_FUNNEL_BATCH"}
    }
}
fn order_for(src:&Source,case:Case,disk:&DiskRime,arm:Arm)
    ->Result<(Vec<usize>,usize),String>
{
    let mut free=(2..(case.bits+2)).collect::<Vec<_>>();
    let mut reads=0;
    match arm {
        Arm::Direct=>{}
        Arm::Fischer=>{
            free.sort_by_key(|&v|fischer_address((v%27)as u64,(v/27)as u64)
                .unwrap_or(u64::MAX));
        }
        Arm::RimeDirect|Arm::PumpBatch=>{
            let addresses=free.iter().map(|&v|{
                (((v+src.clauses.len())%disk.tensor_count())as u8,
                 ((v+src.count)%27)as u8)
            }).collect::<Vec<_>>();
            let values=if matches!(arm,Arm::PumpBatch){
                let mut all=Vec::new();
                for chunk in addresses.chunks(256) {
                    all.extend(rime_disk::loopback_batch(disk,chunk)?);
                }
                all
            }else{
                let mut all=Vec::new();
                for &(tensor,glyph) in &addresses{
                    let req=rime_disk::request(tensor,glyph)?;
                    let addr=u64::from_le_bytes(req[3..11].try_into()
                        .map_err(|_|"fischer address length")?);
                    all.push(disk.read(usize::from(tensor),usize::from(glyph),addr)?);
                }
                all
            };
            reads=values.len();
            let mut keyed=free.iter().copied().zip(values).collect::<Vec<_>>();
            keyed.sort_by_key(|&(v,value)|(value,v));
            free=keyed.into_iter().map(|(v,_)|v).collect();
        }
    }
    // Full source variable order for *all* arms. The source's only
    // unconstrained header inputs are 2..=freebits+1.
    let mut ordered=free;
    ordered.push(1);
    ordered.extend((case.bits+2)..=src.count);
    Ok((ordered,reads))
}
struct ArmResult{
    status:Outcome,elapsed_ns:u128,prep_ns:u128,search_ns:u128,verify_ns:u128,
    nodes:u64,checks:u64,propagations:u64,reads:usize,nonce:Option<u32>,
}
fn run_arm(source:&Source,case:Case,disk:&DiskRime,arm:Arm)
    ->Result<ArmResult,String>
{
    let started=Instant::now();
    let prep=Instant::now();
    let (order,reads)=order_for(source,case,disk,arm)?;
    let prep_ns=prep.elapsed().as_nanos();
    let solve_start=Instant::now();
    let mut solver=Watched::new(source,&order,case.nodes,case.seconds)?;
    let status=solver.recurse();
    let search_ns=solve_start.elapsed().as_nanos();
    let verify_started=Instant::now();
    let nonce=match status {
        Outcome::Sat=>{
            if !source.check(&solver.values){return Err("Bitcoin full original CNF model rejected".into());}
            let n=verify_bitcoin_header(&solver.values,case.bits)?;
            Some(n)
        }
        Outcome::Unsat=>return Err("UNSAT contradicts publicly verified Bitcoin genesis witness".into()),
        Outcome::HoldTime|Outcome::HoldNodes=>None,
    };
    let verify_ns=verify_started.elapsed().as_nanos();
    Ok(ArmResult{status,elapsed_ns:started.elapsed().as_nanos(),prep_ns,search_ns,
        verify_ns,nodes:solver.nodes,checks:solver.checks,
        propagations:solver.propagations,reads,nonce})
}
pub fn benchmark(root:&Path,gguf:&Path)->Result<Vec<String>,String>{
    let started=Instant::now();
    let disk=DiskRime::open_four_axes(gguf)?;
    if disk.verify_three_paths()?!=324 || disk.verify_cube_shadows()?!=24 {
        return Err("Rime source witness preflight failed".into());
    }
    let preflight_ns=started.elapsed().as_nanos();
    let mut out=vec![format!(
        "BTC_RIME_HEADER|schema=v1|bitcoin_source=ORIGINAL_20261117|gguf_sha256={}|preflight_ns={preflight_ns}|match_kernel=WATCHED_DPLL|worker=RUST_181|json=0",
        rime_disk::FOUR_AXES_SHA256
    )];
    let mut sat_count=0_usize;
    let mut hold_count=0_usize;
    for case in CASES {
        let path=root.join(format!("{}.cnf",case.name));
        let (source,parse_ns)=Source::from_original(&path,case)?;
        out.push(format!(
            "BTC_RIME_SOURCE|case={}|sha256={}|variables={}|clauses={}|input_bits={}|parse_ns={parse_ns}|sha_identity=PASS|json=0",
            case.name,case.digest,case.variables,case.clauses,case.bits
        ));
        for arm in [Arm::Direct,Arm::Fischer,Arm::RimeDirect,Arm::PumpBatch] {
            if matches!(arm,Arm::PumpBatch){
                let mut paths=Vec::new();
                for sub in [Arm::Direct,Arm::Fischer,Arm::PumpBatch]{
                    paths.push(run_arm(&source,case,&disk,sub)?);
                }
                let mut nonce=None;
                for result in &paths{
                    if let Some(value)=result.nonce {
                        if let Some(prev)=nonce {
                            if prev!=value{return Err("pump path Bitcoin nonce disagreement".into());}
                        }
                        nonce=Some(value);
                    }
                }
                let status=if nonce.is_some(){Outcome::Sat}else if paths.iter().any(|p|p.status==Outcome::HoldTime){
                    Outcome::HoldTime
                }else{Outcome::HoldNodes};
                let elapsed_ns=paths.iter().map(|p|p.elapsed_ns).sum::<u128>();
                let prep_ns=paths.iter().map(|p|p.prep_ns).sum::<u128>();
                let search_ns=paths.iter().map(|p|p.search_ns).sum::<u128>();
                let verify_ns=paths.iter().map(|p|p.verify_ns).sum::<u128>();
                let nodes=paths.iter().map(|p|p.nodes).sum::<u64>();
                let checks=paths.iter().map(|p|p.checks).sum::<u64>();
                let propagations=paths.iter().map(|p|p.propagations).sum::<u64>();
                let reads=paths.iter().map(|p|p.reads).sum::<usize>();
                if nonce.is_some(){sat_count+=1;}else{hold_count+=1;}
                out.push(format!(
                    "BTC_RIME_ARM|arm={}|case={}|status={}|nonce={}|source_sha256={}|path_solves=3|source_reads={reads}|nodes={nodes}|literal_checks={checks}|propagations={propagations}|prep_ns={prep_ns}|search_ns={search_ns}|verify_ns={verify_ns}|total_ns={elapsed_ns}|json=0",
                    arm.label(),case.name,status.label(),nonce.map_or("NONE".into(),|n|n.to_string()),case.digest
                ));
            }else{
                let run=run_arm(&source,case,&disk,arm)?;
                if run.nonce.is_some(){sat_count+=1;}else{hold_count+=1;}
                out.push(format!(
                    "BTC_RIME_ARM|arm={}|case={}|status={}|nonce={}|source_sha256={}|path_solves=1|source_reads={}|nodes={}|literal_checks={}|propagations={}|prep_ns={}|search_ns={}|verify_ns={}|total_ns={}|json=0",
                    arm.label(),case.name,run.status.label(),
                    run.nonce.map_or("NONE".into(),|n|n.to_string()),case.digest,
                    run.reads,run.nodes,run.checks,run.propagations,
                    run.prep_ns,run.search_ns,run.verify_ns,run.elapsed_ns
                ));
            }
        }
    }
    disk.verify_source()?;
    out.push(format!(
        "BTC_RIME_FINAL|cases=4|arms=4|observations=16|sat={sat_count}|hold={hold_count}|all_original_sha256=PASS|every_sat_model=CHECKED|every_sat_sha256d=PASS|source_post=PASS|status=PASS|json=0"
    ));
    Ok(out)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn known_bitcoin_reference_sha256d(){
        let mut values=vec![0_i8;6];
        values[2..=5].copy_from_slice(&[1,1,-1,1]);
        // Exact-bit test uses the full public header separately:
        let mut header=vec![0_u8;80];
        header[0]=1;
        header[36..68].copy_from_slice(&decode_hex(MERKLE).unwrap());
        header[68..72].copy_from_slice(&1_231_006_505_u32.to_le_bytes());
        header[72..76].copy_from_slice(&0x1d00ffff_u32.to_le_bytes());
        header[76..80].copy_from_slice(&GENESIS_NONCE.to_le_bytes());
        let first=decode_hex(&hash_bytes(&header)).unwrap();
        let mut second=decode_hex(&hash_bytes(&first)).unwrap();
        second.reverse();
        let rendered=second.iter().map(|b|format!("{b:02x}")).collect::<String>();
        assert_eq!(rendered,EXPECTED_DIGEST);
    }
    #[test]
    fn native_watched_correctness_and_budget_hold(){
        let formula=Source{count:2,clauses:vec![vec![1,2],vec![-1,2]]};
        let order=[1,2];
        let mut search=Watched::new(&formula,&order,100,2).unwrap();
        assert_eq!(search.recurse(),Outcome::Sat);
        assert!(formula.check(&search.values));
        let impossible=Source{count:2,clauses:vec![vec![1],vec![-1]]};
        assert!(Watched::new(&impossible,&order,100,2).is_err());
        let all=Source{count:2,clauses:vec![vec![1,2],vec![-1,2],vec![1,-2],vec![-1,-2]]};
        let mut bounded=Watched::new(&all,&order,1,2).unwrap();
        assert_eq!(bounded.recurse(),Outcome::HoldNodes);
        let mut full=Watched::new(&all,&order,100,2).unwrap();
        assert_eq!(full.recurse(),Outcome::Unsat);
    }
}
