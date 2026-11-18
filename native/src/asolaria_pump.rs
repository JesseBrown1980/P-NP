//! Source-equivalent Asolaria spherical pump kernel: 27-point Rime prism,
//! exact inverse, ternary phase winding and logarithmic energy shell.
//! Original: JesseBrown1980/Asolaria@1c03dca255cd374ff7e6db19790c2317c1eb2c85,
//! docs/handoffs/acer-host8-rooms-guide-2026-10-19/host8rooms/src/tribit/mod.rs.
//! This is the exact-integer Rime pump layer. Separate historic 27-node
//! gradient-energy and ACER kernel-pump descriptor laws are provenance HOLD.
#![forbid(unsafe_code)]

pub const P:u64=1_000_081;
pub const G:u64=7;
pub const K:usize=27;
pub const W:u64=951_846;
pub const PHASES:usize=3;
pub const SPECTRAL_VALUES:usize=PHASES*K;

fn mul(a:u64,b:u64)->u64{(a*b)%P}
fn add(a:u64,b:u64)->u64{(a+b)%P}
pub const fn pow(mut base:u64,mut exp:u64)->u64{
    let mut out=1_u64;
    while exp>0{
        if exp&1==1{out=(out*base)%P;}
        base=(base*base)%P;
        exp>>=1;
    }
    out
}
pub fn prism(x:&[u64;K])->[u64;K]{
    let mut out=[0_u64;K];
    for (j,answer) in out.iter_mut().enumerate(){
        let wj=pow(W,j as u64);
        let mut acc=0_u64;
        let mut t=1_u64;
        for value in x{
            acc=add(acc,mul(*value%P,t));
            t=mul(t,wj);
        }
        *answer=acc;
    }
    out
}
pub fn unprism(x:&[u64;K])->[u64;K]{
    let winv=pow(W,P-2);
    let kinv=pow(K as u64,P-2);
    let mut out=[0_u64;K];
    for (j,answer) in out.iter_mut().enumerate(){
        let wj=pow(winv,j as u64);
        let mut acc=0_u64;
        let mut t=1_u64;
        for value in x{
            acc=add(acc,mul(*value%P,t));
            t=mul(t,wj);
        }
        *answer=mul(acc,kinv);
    }
    out
}
/// The original Asolaria pump-shell rule, integer-only.
pub const fn pump_shell(energy:u64)->u32{
    let mut shell=0_u32;
    let mut rung=1_u64;
    while rung<=energy && shell<40{
        rung*=3;
        shell+=1;
    }
    shell
}
/// The three source labels are ordinals 0/1/2. A rotated phase is a
/// *new committed state*, not a replacement for the frozen input.
pub fn phase_trits(x:&[u8;K],phase:usize)->Result<[u8;K],String>{
    if phase>=PHASES||x.iter().any(|v|*v>2){
        return Err("pump invalid cyclic phase or ternary source".into());
    }
    let mut result=[0_u8;K];
    for (i,digit) in result.iter_mut().enumerate(){
        *digit=(x[i]+phase as u8)%3;
    }
    Ok(result)
}
/// Explicit digital proxy: one unit for each non-neutral signed trit;
/// physical energy and original gradient objective are separate claims.
pub fn signal_energy(x:&[u8;K])->Result<u64,String>{
    if x.iter().any(|v|*v>2){
        return Err("pump energy cannot include invalid trits".into());
    }
    Ok(x.iter().filter(|v|**v!=1).count() as u64)
}
pub struct PumpFrame{
    pub shells:[u32;PHASES],
    pub spectra:[[u64;K];PHASES],
}
pub fn source_sweep(input:&[u8;K])->Result<PumpFrame,String>{
    let mut shells=[0_u32;PHASES];
    let mut spectra=[[0_u64;K];PHASES];
    for phase in 0..PHASES{
        let turned=phase_trits(input,phase)?;
        shells[phase]=pump_shell(signal_energy(&turned)?);
        let mut samples=[0_u64;K];
        for (dst,digit) in samples.iter_mut().zip(turned){
            *dst=u64::from(digit);
        }
        spectra[phase]=prism(&samples);
        if unprism(&spectra[phase])!=samples{
            return Err("original Asolaria Rime prism did not reverse".into());
        }
    }
    if phase_trits(&phase_trits(&phase_trits(input,1)?,1)?,1)?!=*input{
        return Err("Asolaria R3 failed closure".into());
    }
    Ok(PumpFrame{shells,spectra})
}
/// Alternate independent reference for the HTTP-0 pump worker:
/// evaluate the root power at each (sample, harmonic), not by the
/// original sequential inner multiplier.
pub fn independent_spectrum(input:&[u8;K],phase:usize)
    ->Result<([u64;K],u32),String>
{
    let turned=phase_trits(input,phase)?;
    let mut spectrum=[0_u64;K];
    for (j,output) in spectrum.iter_mut().enumerate(){
        let mut value=0_u64;
        for (i,&digit) in turned.iter().enumerate(){
            value=(value+u64::from(digit)*pow(W,(i*j)as u64))%P;
        }
        *output=value;
    }
    Ok((spectrum,pump_shell(signal_energy(&turned)?)))
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn source_original_constants_and_27_point_prism(){
        assert_eq!(pow(G,(P-1)/K as u64),W);
        assert_eq!(pow(W,K as u64),1);
        for divisor in [1_usize,3,9]{
            assert_ne!(pow(W,divisor as u64),1);
        }
        assert_eq!([0,1,3,9,27,81,1_000_000].map(pump_shell),
            [0,1,2,3,4,5,13]);
        let mut sample=[0_u8;K];
        for (i,value) in sample.iter_mut().enumerate(){
            *value=(i%3)as u8;
        }
        let pump=source_sweep(&sample).unwrap();
        for phase in 0..PHASES{
            let (other,shell)=independent_spectrum(&sample,phase).unwrap();
            assert_eq!(pump.spectra[phase],other);
            assert_eq!(pump.shells[phase],shell);
            assert_eq!(pump.spectra[phase][0],
                phase_trits(&sample,phase).unwrap()
                    .iter().map(|&v|u64::from(v)).sum::<u64>()%P);
        }
        assert!(phase_trits(&[3;K],0).is_err());
    }
}
