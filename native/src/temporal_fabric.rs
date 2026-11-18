//! Native backend-only temporal-memory service. A received frame crosses the
//! real 64/256/1024/4096 floor and original Omnibit40 operator before a
//! separate child independently recovers the 27 causal events and computes
//! the original source-trained temporal predictor, a memoryless control,
//! and the accepted 243-tick exact matrix witness.
#![forbid(unsafe_code)]
use std::collections::BTreeMap;

use crate::freeze::fischer_address;
use crate::http0_compute::{Client,Task,CELLS};
use crate::matrix_wire;
use crate::microkernel_compute::compute_primary;
use crate::tls_stochastic::{self as stochastic,Context,Fit};

pub const EVENT_COUNT:usize=8_000;
pub const FRAME_COUNT:usize=EVENT_COUNT.div_ceil(CELLS);
const MODEL_WIRE:usize=2+8_000+4_000;
const VERSION:u8=3;
const MODULUS:u64=1_000_081;

#[derive(Clone)]
struct Model{fit:Fit,choice:usize}
struct Stream{
    next_frame:usize,
    context:Context,
    events:usize,
    baseline:u128,
    selected:u128,
}
impl Default for Stream{
    fn default()->Self{
        Self{next_frame:0,context:Context::new(),events:0,baseline:0,selected:0}
    }
}
#[derive(Default)]
pub struct TemporalService{
    models:BTreeMap<u8,Model>,
    streams:BTreeMap<(usize,u8,u8,u16),Stream>,
    requests:usize,
    events:usize,
}
impl TemporalService{
    pub fn models(&self)->usize{self.models.len()}
    pub fn streams(&self)->usize{self.streams.len()}
    pub fn calls(&self)->usize{self.requests}
    pub fn events(&self)->usize{self.events}
    pub fn handle(&mut self,route:&str,data:&[u8])->Result<Vec<u8>,String>{
        if let Some(id_raw)=route.strip_prefix("/temporal/load/"){
            let id:u8=id_raw.parse().map_err(|_|"invalid temporal case id")?;
            if id>=9 || data.len()!=MODEL_WIRE || data[0]!=VERSION || data[1]!=id{
                return Err("temporal source-train envelope invalid".into());
            }
            if self.models.contains_key(&id){
                return Err("source-trained model already loaded".into());
            }
            let train=&data[2..8_002];
            let valid=&data[8_002..12_002];
            if valid.iter().any(|&x|x>1){
                return Err("invalid validation event alphabet".into());
            }
            let fit=stochastic::fit(train)?;
            let choice=stochastic::select(&stochastic::score(valid,&fit)?);
            self.models.insert(id,Model{fit,choice});
            return Ok(vec![VERSION,choice as u8]);
        }
        if route.starts_with("/temporal/score/"){
            let segments:Vec<&str>=route.split('/').collect();
            if segments.len()!=7 || !segments[0].is_empty()
                || segments[1]!="temporal" || segments[2]!="score"
                || !data.is_empty(){
                return Err("temporal score request framing wrong".into());
            }
            let arity:usize=segments[3].parse().map_err(|_|"score arity")?;
            let case:u8=segments[4].parse().map_err(|_|"score case")?;
            let variant:u8=segments[5].parse().map_err(|_|"score variant")?;
            let floor:u16=segments[6].parse().map_err(|_|"score floor")?;
            let (b,s)=self.score(arity,case,variant,floor)?;
            let mut out=Vec::with_capacity(32);
            out.extend_from_slice(&b.to_le_bytes());
            out.extend_from_slice(&s.to_le_bytes());
            return Ok(out);
        }
        let parts:Vec<&str>=route.split('/').collect();
        if parts.len()!=11 || !parts[0].is_empty() || parts[1]!="temporal"
            || parts[2]!="frame"{
            return Err("temporal route malformed".into());
        }
        let arity:usize=parts[3].parse().map_err(|_|"temporal room width")?;
        let lane:usize=parts[4].parse().map_err(|_|"temporal room lane")?;
        let address:u64=parts[5].parse().map_err(|_|"temporal room address")?;
        let case:u8=parts[6].parse().map_err(|_|"temporal case")?;
        let variant:u8=parts[7].parse().map_err(|_|"temporal variant")?;
        let floor:u16=parts[8].parse().map_err(|_|"temporal binary floor")?;
        let frame:usize=parts[9].parse().map_err(|_|"temporal frame index")?;
        let count:usize=parts[10].parse().map_err(|_|"temporal event count")?;
        if !matches!(arity,3|27|81) || case>=9 || variant>1
            || lane!=usize::from(case)%arity || lane>=arity
            || matrix_wire::width(floor).is_none()
            || frame>=FRAME_COUNT || count==0
            || count!=if frame+1==FRAME_COUNT{EVENT_COUNT-(FRAME_COUNT-1)*CELLS}else{CELLS}{
            return Err("invalid temporal route/order bounds".into());
        }
        // Independent repeated modular multiplication: no reliance on
        // producer's Fischer exponentiation to verify the room address.
        let mut independently=1_u64;
        for _ in 0..lane {independently=(independently*7)%MODULUS;}
        if address!=independently{return Err("temporal Fischer route corrupted".into());}
        let (task,decoded_floor,operator)=matrix_wire::decode_independent(data)?;
        if decoded_floor!=floor || operator>=40 || task.id!=case
            || task.seed!=frame as u64 {
            return Err("temporal floor or source identity changed".into());
        }
        if task.trits[..count].iter().any(|&x|!matches!(x,0|2))
            || task.trits[count..].iter().any(|&x|x!=1){
            return Err("future-event or noncanonical ternary padding".into());
        }
        let model=self.models.get(&case)
            .ok_or("temporal source model absent; no heldout self-training")?;
        let stream=self.streams.entry((arity,case,variant,floor)).or_default();
        if stream.next_frame!=frame || stream.events+count>EVENT_COUNT{
            return Err("temporal frame skipped/replayed/reordered".into());
        }
        let mut predictions=Vec::with_capacity(count*4);
        for &encoded in &task.trits[..count]{
            // Predict before observing the current event: no look-ahead.
            let baseline=stochastic::predict(0,&model.fit,&stream.context)?;
            let selected=stochastic::predict(model.choice,&model.fit,&stream.context)?;
            let observed=i64::from(encoded/2)*10_000;
            let bdiff=observed-baseline;
            let sdiff=observed-selected;
            stream.baseline+=(bdiff*bdiff)as u128;
            stream.selected+=(sdiff*sdiff)as u128;
            predictions.extend_from_slice(&(baseline as u16).to_le_bytes());
            predictions.extend_from_slice(&(selected as u16).to_le_bytes());
            stream.context.observe(encoded/2);
        }
        stream.events+=count;
        stream.next_frame+=1;
        self.requests+=1;
        self.events+=count;
        let physical=compute_separately(&task);
        let mut out=Vec::with_capacity(3+CELLS+CELLS*8+predictions.len());
        out.push(VERSION);
        out.push(u8::try_from(model.choice).map_err(|_|"model index overflow")?);
        out.push(count as u8);
        out.extend_from_slice(&task.trits);
        for word in physical {out.extend_from_slice(&word.to_le_bytes());}
        out.extend_from_slice(&predictions);
        Ok(out)
    }
    pub fn validate_complete(&self)->Result<(),String>{
        if self.models.len()!=9 || self.streams.len()!=72
            || self.requests!=72*FRAME_COUNT
            || self.events!=72*EVENT_COUNT {
            return Err("incomplete exact PRL temporal fabric corpus".into());
        }
        if self.streams.values().any(|s|s.events!=EVENT_COUNT||s.next_frame!=FRAME_COUNT){
            return Err("truncated causal context stream".into());
        }
        Ok(())
    }
    pub fn score(&self,arity:usize,case:u8,variant:u8,floor:u16)
        ->Result<(u128,u128),String>{
        let state=self.streams.get(&(arity,case,variant,floor))
            .ok_or("missing temporal score stream")?;
        if state.events!=EVENT_COUNT{return Err("not all events scored".into());}
        Ok((state.baseline,state.selected))
    }
}
fn compute_separately(task:&Task)->[u64;CELLS]{
    let mut state=[0_u64;CELLS];
    for (i,v) in state.iter_mut().enumerate(){
        let x=i as u64+1;
        *v=(task.seed%MODULUS+13*x*x+17*u64::from(task.trits[i]))%MODULUS;
    }
    for tick in 0..243_usize{
        let mut next=[0_u64;CELLS];
        for i in 0..CELLS{
            let mut weighted=11*(u64::from(task.trits[i])+(tick%3)as u64);
            for (coef,source) in [(7,state[(i+CELLS-1)%CELLS]),
                                  (5,state[(i+1)%CELLS]),(3,state[i]) ]{
                weighted+=coef*source;
            }
            next[i]=weighted%MODULUS;
        }
        state=next;
    }
    state
}
pub struct TemporalClient{client:Client}
#[derive(Clone)]
pub struct PredictedFrame{
    pub model:usize,
    pub trits:[u8;CELLS],
    pub matrix:[u64;CELLS],
    pub probs:Vec<(u16,u16)>,
}
impl TemporalClient{
    pub fn start()->Result<Self,String>{
        Ok(Self{client:Client::start()?})
    }
    pub fn load(&self,id:u8,train:&[u8],valid:&[u8])->Result<usize,String>{
        if train.len()!=8_000 || valid.len()!=4_000{
            return Err("temporal fit source segment sizes changed".into());
        }
        let mut body=Vec::with_capacity(MODEL_WIRE);
        body.extend_from_slice(&[VERSION,id]);
        body.extend_from_slice(train);
        body.extend_from_slice(valid);
        let (status,response)=self.client.call(&format!("/temporal/load/{id}"),&body)?;
        if status!="HTTP/1.1 200 OK\r\n" || response.len()!=2
            || response[0]!=VERSION || response[1]>=5{
            return Err(format!("native temporal source-train load failed: {status}"));
        }
        Ok(usize::from(response[1]))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn frame(&self,arity:usize,case:u8,variant:u8,
        floor:u16,index:usize,count:usize,operator:u8,
        task:&Task)->Result<PredictedFrame,String>
    {
        let lane=usize::from(case)%arity;
        let address=fischer_address((lane%27)as u64,(lane/27)as u64)
            .ok_or("missing original Fischer room")?;
        let route=format!(
            "/temporal/frame/{arity}/{lane}/{address}/{case}/{variant}/{floor}/{index}/{count}"
        );
        let request=matrix_wire::encode(task,floor,operator)?;
        let (status,response)=self.client.call(&route,&request)?;
        if status!="HTTP/1.1 200 OK\r\n"{
            return Err(format!("temporal matrix worker rejected frame: {}",String::from_utf8_lossy(&response)));
        }
        let fixed=3+CELLS+8*CELLS;
        if response.len()!=fixed+4*count || response[0]!=VERSION
            || response[1]>=5 || usize::from(response[2])!=count{
            return Err("temporal matrix forecast response noncanonical".into());
        }
        let mut trits=[0_u8;CELLS];
        trits.copy_from_slice(&response[3..3+CELLS]);
        let mut matrix=[0_u64;CELLS];
        for (i,value) in matrix.iter_mut().enumerate(){
            let at=3+CELLS+8*i;
            *value=u64::from_le_bytes(response[at..at+8].try_into()
                .map_err(|_|"temporal matrix output word")?);
            if *value>=MODULUS{return Err("temporal kernel output outside modulus".into());}
        }
        let mut probs=Vec::with_capacity(count);
        for chunk in response[fixed..].chunks_exact(4){
            let base=u16::from_le_bytes([chunk[0],chunk[1]]);
            let model=u16::from_le_bytes([chunk[2],chunk[3]]);
            if !(1..=9_999).contains(&base)||!(1..=9_999).contains(&model){
                return Err("temporal predictor outside Q10000 range".into());
            }
            probs.push((base,model));
        }
        if trits!=task.trits||matrix!=compute_primary(task){
            return Err("independent floor/ternary or 243-tick matrix witness disagrees".into());
        }
        Ok(PredictedFrame{model:usize::from(response[1]),trits,matrix,probs})
    }
    pub fn score(&self,arity:usize,case:u8,variant:u8,floor:u16)
        ->Result<(u128,u128),String>
    {
        let path=format!("/temporal/score/{arity}/{case}/{variant}/{floor}");
        let (status,body)=self.client.call(&path,&[])?;
        if status!="HTTP/1.1 200 OK\r\n" || body.len()!=32{
            return Err("temporal independent score receipt malformed".into());
        }
        let b=u128::from_le_bytes(body[..16].try_into()
            .map_err(|_|"temporal baseline score width")?);
        let s=u128::from_le_bytes(body[16..].try_into()
            .map_err(|_|"temporal learned score width")?);
        Ok((b,s))
    }
    pub fn invalid_frame_rejected(&self,arity:usize,task:&Task)->Result<usize,String>{
        let good=matrix_wire::encode(task,256,0)?;
        let lane=usize::from(task.id)%arity;
        let correct=fischer_address((lane%27)as u64,(lane/27)as u64)
            .ok_or("negative-control Fischer address")?;
        let wrong=format!("/temporal/frame/{arity}/{lane}/{}/{}{}",correct+1,task.id,
            "/0/256/0/27");
        let mut count=0;
        let (status,_)=self.client.call(&wrong,&good)?;
        if status=="HTTP/1.1 422 Unprocessable Content\r\n"{count+=1;}
        let mut bad=good.clone();bad[12]=40;
        let normal=format!("/temporal/frame/{arity}/{lane}/{correct}/{}/0/256/0/27",task.id);
        let (status,_)=self.client.call(&normal,&bad)?;
        if status=="HTTP/1.1 422 Unprocessable Content\r\n"{count+=1;}
        let skipped=format!("/temporal/frame/{arity}/{lane}/{correct}/{}/0/256/1/27",task.id);
        let (status,_)=self.client.call(&skipped,&good)?;
        if status=="HTTP/1.1 422 Unprocessable Content\r\n"{count+=1;}
        Ok(count)
    }
    pub fn stop(self)->Result<(usize,usize,usize,usize),String>{
        let counts=self.client.stop()?;
        Ok((counts.temporal_models,counts.temporal_streams,
            counts.temporal_calls,counts.temporal_events))
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn reject_future_outcomes_and_frame_reordering_without_runtime(){
        let mut s=TemporalService::default();
        let task=Task{id:0,seed:0,trits:[1;CELLS]};
        let raw=matrix_wire::encode(&task,64,0).unwrap();
        assert!(s.handle("/temporal/frame/3/0/1/0/0/64/0/27",&raw).is_err());
        assert!(s.handle("/temporal/load/0",&[3,0]).is_err());
        assert_eq!(FRAME_COUNT,297);
        let a=Task{id:1,seed:3,trits:[0;CELLS]};
        assert_eq!(compute_separately(&a),compute_primary(&a));
        assert!(s.validate_complete().is_err());
    }
}
