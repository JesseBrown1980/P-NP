//! PRL-inspired causal prediction carried by the real binary-floor/Omnibit40/
//! Fischer HTTP-0/matrix fabric. Original accepted stochastic source is read
//! without regenerating it. Nine models fit ONLY training and are selected
//! ONLY on validation, then evaluated on 8,000 unseen time-ordered events
//! and the exact same events in count-preserving shuffled order.
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::fs::{self,File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::ensemble::{self,Phase,Signals};
use crate::http0_compute::{Task,CELLS};
use crate::sha256::hash_bytes;
use crate::temporal_fabric::{TemporalClient,EVENT_COUNT,FRAME_COUNT};
use crate::tls_entropy_omnibit::source_temporal_cases;
use crate::tls_stochastic::{self as stochastic,Context,Fit};

const SOURCE:&str="PNP_TLS_STOCHASTIC_TRACE.bin";
const PINS:[&str;6]=[
    "PNP_TLS.hbp",
    "PNP_TLS_STOCHASTIC.hbp",
    "PNP_FULL_MATRIX.hbp",
    "PNP_GNN_GATE.hbp",
    "PNP_CHECKPOINT.hbp",
    "PNP_FULL_MATRIX_RESULTS.bin",
];
const FLOORS:[u16;4]=[64,256,1024,4096];
const MAG:&[u8]=b"PNP-PRL-CAUSAL-FORECAST-v1\0";
const HEAD:&str="HBPHEADER|schema=pnp.prl.temporal.fabric.v1|source=10.1103/j1r2-5ph7|events=SYNTHETIC_HELDOUT|gnn_forward=HOLD|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const IDX:&str="HBIHEADER|schema=pnp.prl.temporal.fabric.index.v1|for=PNP_PRL_MEMORY_FABRIC.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const CASES:usize=9;
const STREAMS_PER_ARITY:usize=CASES*2*FLOORS.len();
const PAIRS_PER_ARITY:usize=CASES*FLOORS.len();
const REQUESTS_PER_ARITY:usize=STREAMS_PER_ARITY*FRAME_COUNT;
const EVENTS_PER_ARITY:usize=STREAMS_PER_ARITY*EVENT_COUNT;
const ROWS:usize=1+CASES+3*PAIRS_PER_ARITY+1+1;

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Row{
    pub arity:usize,
    pub case:u8,
    pub floor:u16,
    pub selected:usize,
    pub held_base:u128,
    pub held_selected:u128,
    pub shuffle_base:u128,
    pub shuffle_selected:u128,
    pub held_sha:String,
    pub shuffle_sha:String,
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Arm{
    pub arity:usize,
    pub calls:usize,
    pub events:usize,
    pub wall_ns:u128,
    pub held_gains:usize,
    pub shuffled_reversals:usize,
    pub iid_holds:usize,
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Stats{
    pub cases:usize,
    pub forecasts:usize,
    pub calls:usize,
    pub events:usize,
    pub verified_pairs:usize,
    pub rows:Vec<Row>,
    pub arms:Vec<Arm>,
    pub checksum:String,
}
type Case=(u8,u8,Vec<u8>,Vec<u8>,Vec<u8>,Vec<u8>);
type Bundle=(String,String,String,Vec<u8>);
fn originals(dir:&Path)->Result<(Vec<Case>,Vec<String>),String>{
    let source=fs::read(dir.join(SOURCE))
        .map_err(|e|format!("source original stochastic TLS trace: {e}"))?;
    let cases=source_temporal_cases(&source)?;
    if cases.len()!=CASES{return Err("stochastic TLS source cases changed".into());}
    let mut roots=vec![hash_bytes(&source)];
    for file in PINS{
        let bytes=fs::read(dir.join(file))
            .map_err(|e|format!("source gate {file}: {e}"))?;
        let magic:&[u8]=if file.ends_with(".bin"){
            b"PNP-MATRIX-ALL40-V1\0"
        }else{b"HBPHEADER|"};
        if bytes.len()<80 || !bytes.starts_with(magic){
            return Err(format!("prior original matrix plane absent: {file}"));
        }
        if matches!(file,"PNP_TLS.hbp"|"PNP_TLS_STOCHASTIC.hbp")
            && !bytes.windows(b"doi=10.1103/j1r2-5ph7".len())
                .any(|slice|slice==b"doi=10.1103/j1r2-5ph7"){
            return Err("source PRL DOI lineage changed".into());
        }
        roots.push(hash_bytes(&bytes));
    }
    if crate::full_matrix_fabric::inspect(&dir.join(
        "PNP_FULL_MATRIX_RESULTS.bin"))?!=54{
        return Err("full matrix original third-seat data failed".into());
    }
    Ok((cases,roots))
}
fn source_models(cases:&[Case])->Result<Vec<(Fit,usize)>,String>{
    let mut models=Vec::with_capacity(CASES);
    for (i,(mode,rep,training,validation,held,shuffled)) in cases.iter().enumerate(){
        if usize::from(*mode)*3+usize::from(*rep)!=i
            || training.len()!=EVENT_COUNT
            || validation.len()!=4_000
            || held.len()!=EVENT_COUNT || shuffled.len()!=EVENT_COUNT
            || held.iter().map(|&x|usize::from(x)).sum::<usize>()
                !=shuffled.iter().map(|&x|usize::from(x)).sum::<usize>(){
            return Err("temporal train/validation or shuffle census violated".into());
        }
        let fitted=stochastic::fit(training)?;
        let choice=stochastic::select(&stochastic::score(validation,&fitted)?);
        if (*mode==0 && choice!=0)||(*mode>0 && choice==0){
            return Err("precommitted validation selector changed".into());
        }
        models.push((fitted,choice));
    }
    Ok(models)
}
fn third_seat_gnn_holds()->Result<usize,String>{
    for _ in 0..CASES{
        let gate=ensemble::evaluate(Phase::Described,Signals{
            edge:None,path:None,gain:None,g1:None,g2:None,g3:None,
        });
        if gate.emitted || gate.next!=Phase::Described || gate.tier0_block
            || !gate.operator_held{
            return Err("authentic G4 admitted absent trained model signals".into());
        }
    }
    Ok(CASES)
}
fn append_record(out:&mut Vec<u8>,case:u8,variant:u8,floor:u16,
                 predictions:&[u8])->Result<(),String>{
    if predictions.len()!=EVENT_COUNT*4{return Err("incomplete temporal forecast vector".into());}
    out.extend_from_slice(&[case,variant]);
    out.extend_from_slice(&floor.to_le_bytes());
    out.extend_from_slice(&(EVENT_COUNT as u16).to_le_bytes());
    out.extend_from_slice(predictions);
    Ok(())
}
fn inspect_bytes(raw:&[u8])->Result<usize,String>{
    if !raw.starts_with(MAG)||raw.len()<MAG.len()+2+64{
        return Err("PRL causal archive header invalid".into());
    }
    let (body,trailer)=raw.split_at(raw.len()-64);
    if hash_bytes(body).as_bytes()!=trailer{
        return Err("temporal prediction archive SHA mismatch".into());
    }
    let amount=usize::from(u16::from_le_bytes([
        body[MAG.len()],body[MAG.len()+1]
    ]));
    let unit=6+4*EVENT_COUNT;
    if amount!=STREAMS_PER_ARITY || body.len()!=MAG.len()+2+amount*unit{
        return Err("temporal forecast archive extent wrong".into());
    }
    for index in 0..amount{
        let start=MAG.len()+2+index*unit;
        let case=body[start];
        let variant=body[start+1];
        let floor=u16::from_le_bytes([body[start+2],body[start+3]]);
        let count=u16::from_le_bytes([body[start+4],body[start+5]]);
        if case as usize!=index/8 || usize::from(variant)!=index%2
            || floor!=FLOORS[(index%8)/2] || usize::from(count)!=EVENT_COUNT{
            return Err("temporal forecast case/floor ordering changed".into());
        }
        for sample in body[start+6..start+unit].chunks_exact(2){
            let p=u16::from_le_bytes([sample[0],sample[1]]);
            if !(1..=9_999).contains(&p){
                return Err("temporal score archive Q10000 forecast out of range".into());
            }
        }
    }
    Ok(amount)
}
fn row(hbp:&mut String,hbi:&mut String,last:&mut String,index:usize,
       tag:&str,id:&str,body:&str)
{
    let bare=format!("{tag}|seq={index}|id={id}|{body}|previous_hash={last}");
    let digest=hash_bytes(bare.as_bytes());
    let next=hash_bytes(format!("{last}|{digest}").as_bytes());
    let line=format!("{bare}|receipt_sha={digest}|hash={next}|json=0\n");
    hbi.push_str(&format!(
        "HBI|seq={index}|id={id}|offset={}|len={}|row_sha256={}|sh={}|hash={next}|json=0\n",
        hbp.len(),line.len(),hash_bytes(line.as_bytes()),&digest[..16],
    ));
    hbp.push_str(&line);
    *last=next;
}
fn execute(source_dir:&Path)->Result<(Bundle,Stats),String>{
    let (cases,roots)=originals(source_dir)?;
    let models=source_models(&cases)?;
    let gnn_holds=third_seat_gnn_holds()?;
    let mut frozen=Vec::from(MAG);
    frozen.extend_from_slice(&(STREAMS_PER_ARITY as u16).to_le_bytes());
    let mut canonical=BTreeMap::<(u8,u8,u16),Vec<u8>>::new();
    let mut arms=Vec::with_capacity(3);
    let mut report=Vec::with_capacity(3*PAIRS_PER_ARITY);

    for arity in [3_usize,27,81]{
        let client=TemporalClient::start()?;
        for (case,(mode,rep,train,valid,_,_)) in cases.iter().enumerate(){
            let loaded=client.load(case as u8,train,valid)?;
            if usize::from(*mode)*3+usize::from(*rep)!=case
                || loaded!=models[case].1{
                return Err("child source-only model selection disagreed".into());
            }
        }
        let negative=Task{id:0,seed:0,trits:[1;CELLS]};
        if client.invalid_frame_rejected(arity,&negative)?!=3{
            return Err("causal HTTP-0 wrong route/operator/order controls failed".into());
        }
        let start=Instant::now();
        let mut gains=0_usize;
        let mut reversals=0_usize;
        let mut iid=0_usize;

        for (case,(mode,rep,_,_,held,shuffled)) in cases.iter().enumerate(){
            let (fit,choice)=&models[case];
            for floor in FLOORS{
                let mut pair=[(0_u128,0_u128);2];
                let mut sha=[String::new(),String::new()];
                for (variant,events) in [held,shuffled].iter().enumerate(){
                    let mut predictions=Vec::with_capacity(EVENT_COUNT*4);
                    let mut context=Context::new();
                    let mut base=0_u128;
                    let mut selected=0_u128;
                    let mut observed=0_usize;
                    for frame in 0..FRAME_COUNT{
                        let offset=frame*CELLS;
                        let count=(EVENT_COUNT-offset).min(CELLS);
                        let mut trits=[1_u8;CELLS];
                        for (dst,&outcome) in trits.iter_mut()
                            .zip(&events[offset..offset+count]){
                            *dst=2*outcome;
                        }
                        let task=Task{id:case as u8,seed:frame as u64,trits};
                        let floor_no=FLOORS.iter().position(|&x|x==floor)
                            .ok_or("missing original floor index")?;
                        let operator=((frame+11*floor_no+case*3)%40)as u8;
                        let response=client.frame(arity,case as u8,variant as u8,
                            floor,frame,count,operator,&task)?;
                        if response.model!=*choice
                            || response.trits!=trits
                            || response.matrix.iter().any(|x|*x>=1_000_081)
                            || response.probs.len()!=count{
                            return Err("temporal independent forward/ternary witness changed".into());
                        }
                        for (&outcome,&(wire_baseline,wire_memory)) in events
                            [offset..offset+count].iter().zip(&response.probs){
                            let expected_b=stochastic::predict(0,fit,&context)?;
                            let expected_m=stochastic::predict(*choice,fit,&context)?;
                            if i64::from(wire_baseline)!=expected_b
                                || i64::from(wire_memory)!=expected_m{
                                return Err(format!(
                                    "original trained-only causal prediction differs at case={case} floor={floor} variant={variant} event={observed}"
                                ));
                            }
                            let label=10_000_i64*i64::from(outcome);
                            let delta_b=label-expected_b;
                            let delta_m=label-expected_m;
                            base+=(delta_b*delta_b)as u128;
                            selected+=(delta_m*delta_m)as u128;
                            context.observe(outcome);
                            predictions.extend_from_slice(&wire_baseline.to_le_bytes());
                            predictions.extend_from_slice(&wire_memory.to_le_bytes());
                            observed+=1;
                        }
                    }
                    if observed!=EVENT_COUNT
                        || client.score(arity,case as u8,variant as u8,floor)?
                            !=(base,selected){
                        return Err("independent native stream Brier tally disagrees".into());
                    }
                    let independent=stochastic::score(events,fit)?;
                    if independent[0]!=base||independent[*choice]!=selected{
                        return Err("source stochastic predictor independent replay differs".into());
                    }
                    let key=(case as u8,variant as u8,floor);
                    if arity==3{
                        append_record(&mut frozen,key.0,key.1,key.2,&predictions)?;
                        canonical.insert(key,predictions.clone());
                    }else if canonical.get(&key)!=Some(&predictions){
                        return Err("binary-floor forecast changed across physical room widths".into());
                    }
                    pair[variant]=(base,selected);
                    sha[variant]=hash_bytes(&predictions);
                }
                if pair[0].0!=pair[1].0{
                    return Err("count-preserving shuffle changed memoryless Brier".into());
                }
                if *mode==0{
                    if *choice!=0||pair[0].0!=pair[0].1||pair[1].0!=pair[1].1{
                        return Err("IID control falsely identified memory".into());
                    }
                    iid+=1;
                }else{
                    if pair[0].1>=pair[0].0{
                        return Err("coupled ordered heldout memory improvement absent".into());
                    }
                    if pair[1].1<=pair[1].0{
                        return Err("shuffled-only conditional memory advantage survived".into());
                    }
                    gains+=1;
                    reversals+=1;
                }
                report.push(Row{
                    arity,case:case as u8,floor,selected:*choice,
                    held_base:pair[0].0,held_selected:pair[0].1,
                    shuffle_base:pair[1].0,shuffle_selected:pair[1].1,
                    held_sha:sha[0].clone(),shuffle_sha:sha[1].clone(),
                });
            }
        }
        let wall_ns=start.elapsed().as_nanos();
        let (loaded,streams,calls,events)=client.stop()?;
        if loaded!=CASES || streams!=STREAMS_PER_ARITY
            || calls!=REQUESTS_PER_ARITY || events!=EVENTS_PER_ARITY
            || gains!=24 || reversals!=24 || iid!=12 {
            return Err("source-trained memory worker/stream accounting wrong".into());
        }
        arms.push(Arm{arity,calls,events,wall_ns,held_gains:gains,
            shuffled_reversals:reversals,iid_holds:iid});
    }
    if canonical.len()!=STREAMS_PER_ARITY
        || report.len()!=3*PAIRS_PER_ARITY{
        return Err("causal full fabric record census wrong".into());
    }
    let forecast_digest=hash_bytes(&frozen);
    frozen.extend_from_slice(forecast_digest.as_bytes());
    if inspect_bytes(&frozen)?!=STREAMS_PER_ARITY{
        return Err("temporal forecast archive cannot be independently scanned".into());
    }
    let mut hbp=HEAD.to_owned();
    let mut hbi=IDX.to_owned();
    let mut previous=hash_bytes(b"PNP-PRL-TEMPORAL-FABRIC-GENESIS");
    let mut seq=1;
    row(&mut hbp,&mut hbi,&mut previous,seq,"PRL_SOURCE","temporal",
        &format!("doi=10.1103/j1r2-5ph7|origin=PRL_EXPERIMENTAL_TLS_MEMORY|source=SYNTHETIC_PRECOMMITTED|trace_sha256={}|floors=64,256,1024,4096|original_operator_count=40|heldout_per_case=8000|shuffled_per_case=8000|source_gnn_forward=HOLD|source_roots={}",
            roots[0],roots[1..].join(",")));
    for (case,((mode,rep,train,valid,_,_),(_,choice))) in cases.iter().zip(&models).enumerate(){
        seq+=1;
        row(&mut hbp,&mut hbi,&mut previous,seq,"PRL_FITTED",
            &format!("case{case}"),
            &format!("regime={mode}|rep={rep}|train_only_sha256={}|validation_only_sha256={}|selected_capacity={}|selector=SOURCE_ORIGINAL_VALIDATION_BRIER_PLUS_PENALTY|heldout_labels_in_training=0|model=INTEGER_OBSERVATION_TRAINED",
                hash_bytes(train),hash_bytes(valid),[0,1,3,27,81][*choice]));
    }
    for item in &report{
        seq+=1;
        let g_held=item.held_base as i128-item.held_selected as i128;
        let g_shuffled=item.shuffle_base as i128-item.shuffle_selected as i128;
        row(&mut hbp,&mut hbi,&mut previous,seq,"PRL_FABRIC_PAIR",
            &format!("rooms{}_case{}_floor{}",item.arity,item.case,item.floor),
            &format!("heldout_base_brier_q2={}|heldout_memory_brier_q2={}|ordered_gain_q2={g_held}|shuffled_base_brier_q2={}|shuffled_memory_brier_q2={}|shuffled_gain_q2={g_shuffled}|trained_capacity={}|heldout_forecast_sha256={}|shuffled_forecast_sha256={}|floor_transform=ORIGINAL_OMNIBIT40|fischer=INDEPENDENT|source_trace=EXACT|causal_no_current_event=PASS",
                item.held_base,item.held_selected,
                item.shuffle_base,item.shuffle_selected,[0,1,3,27,81][item.selected],
                item.held_sha,item.shuffle_sha));
    }
    seq+=1;
    row(&mut hbp,&mut hbi,&mut previous,seq,"PRL_GNN_HOLD","source",
        &format!("source_G4_HOLD={gnn_holds}|trained_GNN_forward=HOLD_NOT_COMPUTED|temporal_predictor=ACTUAL_SOURCE_TRAINED_INTEGER|unseen_heldout=YES|training_and_validation_separated=YES|random_shuffle=SOURCE_PRECOMMITTED"));
    seq+=1;
    row(&mut hbp,&mut hbi,&mut previous,seq,"PRL_FABRIC_SUMMARY","acceptance",
        &format!("cases={CASES}|source_streams=18|room_widths=3,27,81|exact_forecasts={}|http0_requests={}|total_events={}|ordered_coupled_gain_rows=72|shuffle_reversal_rows=72|iid_null_rows=36|original_operators=40|binary_floors=4|root_sha256={}|forecast_sha256={}|status=PASS",
            3*STREAMS_PER_ARITY*EVENT_COUNT,
            3*REQUESTS_PER_ARITY,3*EVENTS_PER_ARITY,
            roots[0],hash_bytes(&frozen)));
    if seq!=ROWS || hbi.lines().filter(|s|s.starts_with("HBI|")).count()!=ROWS{
        return Err("PRL causal witness row count wrong".into());
    }
    let anchor_data=format!(
        "PRL_FABRIC_ANCHOR|rows={seq}|source_sha256={}|prediction_sha256={}|hbp_sha256={}|hbi_sha256={}|end_hash={previous}|seal=HBI-HBP-SHA-SH-HASH",
        roots[0],hash_bytes(&frozen),hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes()),
    );
    let anchor=format!("{anchor_data}|checksum={}|json=0\n",
        hash_bytes(anchor_data.as_bytes()));
    let stats=Stats{
        cases:CASES,forecasts:3*STREAMS_PER_ARITY*EVENT_COUNT,
        calls:3*REQUESTS_PER_ARITY,events:3*EVENTS_PER_ARITY,
        verified_pairs:report.len(),rows:report,arms,
        checksum:hash_bytes(&frozen),
    };
    Ok(((hbp,hbi,anchor,frozen),stats))
}
fn output(file:&Path,bytes:&[u8])->Result<(),String>{
    let mut f=File::create(file).map_err(|e|e.to_string())?;
    f.write_all(bytes).map_err(|e|e.to_string())?;
    f.sync_all().map_err(|e|e.to_string())
}
pub fn attest(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((hbp,hbi,anchor,predictions),stats)=execute(prior)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,body) in [
        ("PNP_PRL_MEMORY_FABRIC.hbp",hbp.as_bytes()),
        ("PNP_PRL_MEMORY_FABRIC.hbi",hbi.as_bytes()),
        ("PNP_PRL_MEMORY_FABRIC.anchor",anchor.as_bytes()),
        ("PNP_PRL_MEMORY_FORECAST.bin",predictions.as_slice()),
    ]{output(&out.join(name),body)?;}
    Ok(stats)
}
pub fn verify(prior:&Path,out:&Path)->Result<Stats,String>{
    let ((hbp,hbi,anchor,predictions),stats)=execute(prior)?;
    for (name,expected) in [
        ("PNP_PRL_MEMORY_FABRIC.hbp",hbp.as_bytes()),
        ("PNP_PRL_MEMORY_FABRIC.hbi",hbi.as_bytes()),
        ("PNP_PRL_MEMORY_FABRIC.anchor",anchor.as_bytes()),
        ("PNP_PRL_MEMORY_FORECAST.bin",predictions.as_slice()),
    ]{
        if fs::read(out.join(name)).map_err(|e|e.to_string())?!=expected{
            return Err(format!("PRL causal fabric source/evidence changed: {name}"));
        }
    }
    Ok(stats)
}
pub fn inspect(file:&Path)->Result<usize,String>{
    inspect_bytes(&fs::read(file).map_err(|e|e.to_string())?)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn prediction_evidence_rejects_tamper_and_preserves_floor_order(){
        let mut data=Vec::from(MAG);
        data.extend_from_slice(&(STREAMS_PER_ARITY as u16).to_le_bytes());
        for case in 0..9_u8{
            for variant in 0..2_u8{
                for floor in FLOORS{
                    append_record(&mut data,case,variant,floor,
                        &vec![7_u8,0,11,0].repeat(EVENT_COUNT)).unwrap();
                }
            }
        }
        let digest=hash_bytes(&data);
        data.extend_from_slice(digest.as_bytes());
        assert_eq!(inspect_bytes(&data).unwrap(),STREAMS_PER_ARITY);
        data[140]^=1;
        assert!(inspect_bytes(&data).is_err());
        assert_eq!(ROWS,120);
    }
}
