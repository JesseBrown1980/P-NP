//! Exact floor/Omnibit wire format crossing into independent backend rooms.
//! Producer calls the accepted binary_ternary_bridge; the separate room
//! independently decodes ordinals, using the original 40-operator inverse.
#![forbid(unsafe_code)]

use crate::binary_ternary_bridge::{encode_floor,FloorWord};
use crate::http0_compute::{Task,CELLS};
use crate::omega_operator_catalog as omega;
use crate::ternary::Trit;

const VERSION:u8=2;
const HEADER:usize=14;

pub fn width(floor:u16)->Option<(usize,u8)>{
    match floor {
        64=>Some((3,6)),256=>Some((5,8)),
        1024=>Some((6,10)),4096=>Some((7,12)),
        _=>None,
    }
}
fn source_trit(digit:u8)->Result<Trit,String>{
    match digit {
        0=>Ok(Trit::Minus),
        1=>Ok(Trit::Zero),
        2=>Ok(Trit::Plus),
        _=>Err("invalid balanced ternary value".into()),
    }
}
pub fn encode(task:&Task,floor:u16,operator:u8)->Result<Vec<u8>,String>{
    let (per,bits)=width(floor).ok_or("unrecognized binary floor")?;
    if operator>=40 || task.id>=54 || task.trits.iter().any(|t|*t>2){
        return Err("invalid original task/operator".into());
    }
    let mut original=Vec::new();
    for block in task.trits.chunks(per){
        let mut balanced=block.iter().copied().map(source_trit)
            .collect::<Result<Vec<_>,_>>()?;
        balanced.resize(per,Trit::Zero);
        original.push(encode_floor(floor,&balanced).map_err(str::to_owned)?.code);
    }
    let transformed=omega::apply_symbols(operator,bits,&original)
        .map_err(|e|e.to_string())?;
    let count=u8::try_from(transformed.len()).map_err(|_|"too many floor symbols")?;
    let mut bytes=Vec::with_capacity(HEADER+2*transformed.len());
    bytes.extend_from_slice(&[VERSION,task.id]);
    bytes.extend_from_slice(&task.seed.to_le_bytes());
    bytes.extend_from_slice(&floor.to_le_bytes());
    bytes.push(operator);
    bytes.push(count);
    for value in transformed { bytes.extend_from_slice(&value.to_le_bytes()); }
    Ok(bytes)
}
/// Independent integer base-three expansion (different source from the
/// producer's generic bridge), with canonical zero-padding and reserved
/// binary codes rejected. No original events or expected answer are sent.
pub fn decode_independent(bytes:&[u8])->Result<(Task,u16,u8),String>{
    if bytes.len()<HEADER || bytes[0]!=VERSION || bytes[1]>=54{
        return Err("matrix wire version or task identity invalid".into());
    }
    let id=bytes[1];
    let seed=u64::from_le_bytes(bytes[2..10].try_into()
        .map_err(|_|"invalid matrix task seed")?);
    let floor=u16::from_le_bytes([bytes[10],bytes[11]]);
    let operator=bytes[12];
    let count=usize::from(bytes[13]);
    let (per,bits)=width(floor).ok_or("matrix wire floor unknown")?;
    let expected=CELLS.div_ceil(per);
    if operator>=40 || count!=expected || bytes.len()!=HEADER+2*count {
        return Err("matrix wire operator/count/length invalid".into());
    }
    let mut transformed=Vec::with_capacity(count);
    for chunk in bytes[HEADER..].chunks_exact(2){
        transformed.push(u16::from_le_bytes([chunk[0],chunk[1]]));
    }
    let original=omega::invert_symbols(operator,bits,&transformed)
        .map_err(|e|e.to_string())?;
    if original.len()!=expected{return Err("Omnibit inverse changed symbol count".into());}
    let mut trits=Vec::with_capacity(count*per);
    let maximum=[27_u16,243,729,2187][match floor {64=>0,256=>1,1024=>2,_=>3}];
    for code in original {
        if code>=maximum {return Err("reserved binary code rejected".into());}
        let mut powers=vec![0_u8;per];
        let mut value=code;
        for digit in powers.iter_mut().rev(){
            *digit=(value%3)as u8;
            value/=3;
        }
        if value!=0{return Err("binary ordinal overflow".into());}
        trits.extend_from_slice(&powers);
    }
    if trits[CELLS..].iter().any(|value|*value!=1) {
        return Err("noncanonical ternary zero padding".into());
    }
    let bounded:[u8;CELLS]=trits[..CELLS].try_into()
        .map_err(|_|"matrix vector width")?;
    Ok((Task{id,seed,trits:bounded},floor,operator))
}
pub fn check_floor_word(word:FloorWord)->Result<Vec<Trit>,String>{
    let (width,_)=width(word.floor).ok_or("unknown floor")?;
    if u32::from(word.code)>=3_u32.pow(width as u32){
        return Err("reserved binary code".into());
    }
    let mut ordinal=word.code;
    let mut digits=vec![Trit::Zero;width];
    for cell in digits.iter_mut().rev(){
        *cell=source_trit((ordinal%3)as u8)?;
        ordinal/=3;
    }
    Ok(digits)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn all_40_actual_operators_cross_all_four_binary_floors(){
        for seed in [0_u64,25,1_000_081] {
            let mut original=[0_u8;CELLS];
            for (i,t) in original.iter_mut().enumerate(){
                *t=((i as u64+seed)%3)as u8;
            }
            let task=Task{id:7,seed,trits:original};
            for floor in [64_u16,256,1024,4096]{
                for operator in 0..40 {
                    let message=encode(&task,floor,operator).unwrap();
                    let (restored,f,o)=decode_independent(&message).unwrap();
                    assert_eq!((restored,f,o),(task.clone(),floor,operator));
                }
            }
        }
        assert!(check_floor_word(FloorWord{floor:256,code:243}).is_err());
        let task=Task{id:0,seed:10,trits:[1;CELLS]};
        let mut payload=encode(&task,256,0).unwrap();
        payload[14]=243;
        payload[15]=0;
        assert!(decode_independent(&payload).is_err());
        let mut payload=encode(&task,64,0).unwrap();
        payload[12]=40;
        assert!(decode_independent(&payload).is_err());
    }
}
