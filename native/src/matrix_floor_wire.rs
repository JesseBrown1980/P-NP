//! Canonical native binary-floor / ternary / Omnibit40 wire envelope.
//! Exact 20-symbol banks use the original source-pinned Omega operators.
#![forbid(unsafe_code)]
use crate::binary_ternary_bridge::{self as bridge, FloorWord};
use crate::http0_compute::{Task,CELLS};
use crate::omega_operator_catalog as omega;
use crate::ternary::Trit;
use crate::prism;

pub const BYTES:usize=53;
const BANK:usize=20;
const WIDTHS:[usize;4]=[3,5,6,7];
const FLOORS:[u16;4]=[64,256,1024,4096];

pub fn floor_info(index:u8)->Result<(u16,usize,u8),String>{
    let i=usize::from(index);
    if i>=FLOORS.len(){return Err("invalid binary floor selector".into());}
    Ok((FLOORS[i],WIDTHS[i],[6,8,10,12][i]))
}
fn trit(value:u8)->Result<Trit,String>{
    match value{0=>Ok(Trit::Minus),1=>Ok(Trit::Zero),2=>Ok(Trit::Plus),_=>Err("nonternary wire value".into())}
}
fn digit(value:Trit)->u8{
    match value{Trit::Minus=>0,Trit::Zero=>1,Trit::Plus=>2}
}
fn plain(task:&Task,index:u8)->Result<(u8,usize,[u16;BANK]),String>{
    let (floor,width,bits)=floor_info(index)?;
    if task.id>=54{return Err("invalid frozen task identifier".into());}
    let n=CELLS.div_ceil(width);
    let mut words=[0_u16;BANK];
    for (slot,word) in words.iter_mut().take(n).enumerate(){
        let mut block=Vec::with_capacity(width);
        for at in slot*width..(slot+1)*width {
            block.push(if at<CELLS{trit(task.trits[at])?}else{Trit::Zero});
        }
        *word=bridge::encode_floor(floor,&block)
            .map_err(str::to_owned)?.code;
    }
    for slot in n..BANK {words[slot]=words[slot%n];}
    Ok((bits,n,words))
}
pub fn encode(task:&Task,index:u8,operator:u8)->Result<Vec<u8>,String>{
    let (bits,n,words)=plain(task,index)?;
    let transformed=omega::apply_symbols(operator,bits,&words)
        .map_err(|e|e.to_string())?;
    if transformed.len()!=BANK{return Err("Omnibit source output count changed".into());}
    // The actual authenticated Q-PRISM 256<->1024 40-bit rung is
    // exercised over the byte-floor source bank, before the transform.
    if index==1 {
        let input=words.iter().map(|x|u8::try_from(*x)
            .map_err(|_|"byte floor overflow")).collect::<Result<Vec<_>,_>>()?;
        let glyphs=prism::encode(&input);
        if prism::decode(&glyphs,input.len()).map_err(str::to_owned)?!=input
            || glyphs.len()!=16 {
            return Err("Q-PRISM 256/1024 source bank parity failure".into());
        }
    }
    let mut out=Vec::with_capacity(BYTES);
    out.push(2);
    out.push(task.id);
    out.extend_from_slice(&task.seed.to_le_bytes());
    out.push(index);
    out.push(operator);
    out.push(u8::try_from(n).map_err(|_|"too many floor symbols")?);
    for symbol in transformed {out.extend_from_slice(&symbol.to_le_bytes());}
    if out.len()!=BYTES{return Err("noncanonical floor wire width".into());}
    Ok(out)
}
pub fn decode(bytes:&[u8])->Result<(Task,u8,u8),String>{
    if bytes.len()!=BYTES || bytes[0]!=2 || bytes[1]>=54{
        return Err("noncanonical matrix floor frame".into());
    }
    let index=bytes[10];
    let op=bytes[11];
    let (floor,width,bits)=floor_info(index)?;
    let count=CELLS.div_ceil(width);
    if usize::from(bytes[12])!=count || op>=40{
        return Err("floor word count or original operator invalid".into());
    }
    let transformed=(13..BYTES).step_by(2)
        .map(|i|u16::from_le_bytes([bytes[i],bytes[i+1]]))
        .collect::<Vec<_>>();
    let original=omega::invert_symbols(op,bits,&transformed)
        .map_err(|e|e.to_string())?;
    if original.len()!=BANK{return Err("incomplete inverse Omnibit bank".into());}
    for index in count..BANK{
        if original[index]!=original[index%count]{
            return Err("source-derived floor bank suffix altered".into());
        }
    }
    let mut recovered=Vec::new();
    for code in original.iter().take(count){
        let part=bridge::decode_floor(FloorWord{floor,code:*code})
            .map_err(str::to_owned)?;
        recovered.extend(part);
    }
    if recovered.len()<CELLS || recovered[CELLS..].iter().any(|t|*t!=Trit::Zero){
        return Err("floor bridge padding invalid".into());
    }
    let mut trits=[0_u8;CELLS];
    for (slot,t) in trits.iter_mut().zip(recovered.iter()) {*slot=digit(*t);}
    let task=Task{id:bytes[1],seed:u64::from_le_bytes(
        bytes[2..10].try_into().map_err(|_|"seed width")?),trits};
    // Reject malleable alternate wire representations and all reserved codes.
    if encode(&task,index,op)?!=bytes {
        return Err("noncanonical source/Omnibit matrix frame".into());
    }
    Ok((task,index,op))
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn all_four_binary_floors_and_all_forty_omega_operators_round_trip(){
        for task_id in [0_u8,17,53]{
            let mut digits=[0_u8;CELLS];
            for (i,x) in digits.iter_mut().enumerate(){*x=((i+usize::from(task_id))%3) as u8;}
            let task=Task{id:task_id,seed:u64::from(task_id)*9871,trits:digits};
            for floor in 0..4{
                for op in 0..40{
                    let packet=encode(&task,floor,op).unwrap();
                    assert_eq!(decode(&packet),Ok((task.clone(),floor,op)));
                }
            }
        }
    }
    #[test]
    fn reject_reserved_codes_noncanonical_suffix_bad_ops_and_floor(){
        let task=Task{id:4,seed:999,trits:[1;CELLS]};
        let mut payload=encode(&task,0,0).unwrap();
        payload[13..15].copy_from_slice(&63_u16.to_le_bytes());
        assert!(decode(&payload).is_err());
        let mut payload=encode(&task,1,0).unwrap();
        payload[13..15].copy_from_slice(&255_u16.to_le_bytes());
        assert!(decode(&payload).is_err());
        let mut payload=encode(&task,1,0).unwrap();
        payload[49]^=1;
        assert!(decode(&payload).is_err());
        let mut payload=encode(&task,2,7).unwrap();
        payload[11]=40;
        assert!(decode(&payload).is_err());
        let mut payload=encode(&task,3,25).unwrap();
        payload[10]=4;
        assert!(decode(&payload).is_err());
    }
}
