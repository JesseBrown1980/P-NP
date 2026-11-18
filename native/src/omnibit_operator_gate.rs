//! Original Omnibit Omega 40-operator catalog on native mixed binary/ternary
//! floors. The catalog source is vendored byte-for-byte from its pinned commit.
#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::collections::BTreeSet;
type SymbolBank = (u8, Vec<u16>);
type BankSet = Vec<SymbolBank>;
use std::fs;
use std::path::Path;

use crate::{
    binary_ternary_bridge::{self as bridge, FloorWord},
    omega_operator_catalog as omega,
    omnibit_tournament,
    sha256::hash_bytes,
    ternary::Trit,
};

const SOURCE_COMMIT: &str = "dad4647ad19a1b08ee90990c34aab1255f9a6046";
const SOURCE_BLOB: &str = "1535e158b21b1c2b9ece4dce84e0e9779498ec19";
const ORIGINAL_PATH: &str = "reference/omega-operator-catalog-v0/omega_operator_catalog.rs";
const LOCAL_CATALOG: &str = include_str!("omega_operator_catalog.rs");
const FIXTURES: &str = include_str!("../fixtures/OMNIBIT_PUBLIC_TRITS.hbp");
const HBP_HEADER: &str = "HBPHEADER|schema=pnp.omnibit.omega40.v1|scope=PUBLIC_SYNTHETIC|seats=LABELS_ONLY|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HBI_HEADER: &str = "HBIHEADER|schema=pnp.omnibit.omega40.index.v1|for=PNP_OMNI40.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const FLOOR_DEFS: [(u16, u8, usize); 4] = [
    (64,6,3), (256,8,5), (1024,10,6), (4096,12,7)
];
const SEATS: [&str;3] = ["acer","liris","relic"];
const VIEWS: [&str;2] = ["NORMAL","ANTI"];
const CASES: usize = 20;
const CATALOG_COUNT: usize = 40;
const BANK_LEN: usize = 20;
const FRAME_RUNS: usize = (CASES * 4 + 4) * 6 * CATALOG_COUNT;
const SELECTED_RUNS: usize = (CASES * 4 + 4) * 6;
const ROWS: usize = 1 + CATALOG_COUNT + 4 + FRAME_RUNS + SELECTED_RUNS + 1;

#[derive(Clone,Debug)]
struct Fixture { id:String, data:Vec<Trit> }
fn frozen_cases() -> Result<Vec<Fixture>, String> {
    let rest=FIXTURES.strip_prefix(
        "HBPHEADER|schema=pnp.omnibit.public.fixture.v1|scope=PUBLIC_SYNTHETIC|privacy=NO_SEAT_INPUT|json=0\n"
    ).ok_or("original public Omnibit fixture schema changed")?;
    let mut cases=Vec::new();
    let mut seen=BTreeSet::new();
    for line in rest.lines() {
        let mut split=line.split('|');
        if split.next()!=Some("OMNIBIT_FIXTURE") {
            return Err("unknown public tournament fixture".to_owned());
        }
        let id=split.next().and_then(|x|x.strip_prefix("id="))
            .ok_or("missing fixture ID")?;
        let raw=split.next().and_then(|x|x.strip_prefix("trits="))
            .ok_or("missing fixture trits")?;
        let n=split.next().and_then(|x|x.strip_prefix("length="))
            .ok_or("missing fixture trit length")?
            .parse::<usize>().map_err(|_|"non-numeric fixture length")?;
        if split.next()!=Some("source=DETERMINISTIC_SYNTHETIC")
            || split.next()!=Some("json=0") || split.next().is_some()
            || raw.len()!=n || !(3..=8).contains(&n)
            || !seen.insert(id) {
            return Err("invalid/duplicate Omnibit public fixture".to_owned());
        }
        let data=raw.chars().map(|c|
            Trit::from_symbol(c).ok_or("invalid balanced trit".to_owned())
        ).collect::<Result<Vec<_>,_>>()?;
        cases.push(Fixture{id:id.to_owned(),data});
    }
    if cases.len()!=CASES
        || [3,5,6,7,8].iter().any(|len|cases.iter().filter(|c|c.data.len()==*len).count()!=4)
    {
        return Err("public Omnibit case census changed".to_owned());
    }
    Ok(cases)
}
fn floor_bank(data:&[Trit],floor:u16,width:usize)->Result<Vec<u16>,String>{
    let mut out=Vec::with_capacity(BANK_LEN);
    for round in 0..BANK_LEN {
        let candidate=(0..width)
            .map(|i|data[(i+round)%data.len()])
            .collect::<Vec<_>>();
        let word=bridge::encode_floor(floor,&candidate).map_err(str::to_owned)?;
        out.push(word.code);
    }
    Ok(out)
}
fn composite_bank(data:&[Trit])->Result<BankSet,String>{
    if data.len()!=8 {return Err("composed Omnibit floor requires eight trits".to_owned());}
    let mut first=Vec::with_capacity(BANK_LEN);
    let mut second=Vec::with_capacity(BANK_LEN);
    for round in 0..BANK_LEN {
        let candidate=(0..8)
            .map(|i|data[(i+round)%8])
            .collect::<Vec<_>>();
        let route=bridge::encode_route(&candidate).map_err(str::to_owned)?;
        if route.len()!=2 || route[0].floor!=256 || route[1].floor!=64 {
            return Err("composite 256+64 route changed".to_owned());
        }
        if bridge::decode_route(&route,8).map_err(str::to_owned)?!=candidate {
            return Err("composite route inverse changed".to_owned());
        }
        first.push(route[0].code);
        second.push(route[1].code);
    }
    Ok(vec![(8,first),(6,second)])
}
fn banks_for_case(data:&[Trit], choice:usize)
    ->Result<(String,BankSet),String>
{
    match choice.cmp(&4) {
        Ordering::Less => {
            let (floor,bits,width)=FLOOR_DEFS[choice];
            Ok((floor.to_string(),vec![(bits,floor_bank(data,floor,width)?)]))
        }
        Ordering::Equal => Ok(("256+64".to_owned(),composite_bank(data)?)),
        Ordering::Greater => Err("invalid mixed floor choice".to_owned()),
    }
}
fn work_proxy(operator:u8, symbols:usize, banks:usize)->usize{
    match operator {
        0..=7 => symbols * (1+operator.count_ones() as usize * 2),
        8..=19 => symbols * 2+banks * 2,
        20..=39 => symbols * 4+banks * 400,
        _=>usize::MAX
    }
}
fn checked_omega(id:u8,banks:&[SymbolBank])
    ->Result<BankSet,String>
{
    if usize::from(id)>=CATALOG_COUNT {return Err("operator ID not in source catalog".to_owned());}
    let mut changed=Vec::new();
    for (bits,original) in banks {
        if original.len()!=BANK_LEN || !matches!(bits,6|8|10|12) {
            return Err("unsupported floor bank geometry".to_owned());
        }
        let scrambled=omega::apply_symbols(id,*bits,original).map_err(|e|e.to_string())?;
        let replay=omega::invert_symbols(id,*bits,&scrambled).map_err(|e|e.to_string())?;
        if replay!=*original {
            return Err(format!("source Omnibit operator did not invert: {id}"));
        }
        let floor=1_u16<<bits;
        for value in replay {
            let trits=bridge::decode_floor(FloorWord{floor,code:value})
                .map_err(str::to_owned)?;
            if bridge::encode_floor(floor,&trits).map_err(str::to_owned)?.code!=value {
                return Err("mixed binary/ternary inverse was not canonical".to_owned());
            }
        }
        changed.push((*bits,scrambled));
    }
    Ok(changed)
}
fn frame_encoded(id:u8,banks:&[SymbolBank])->Result<Vec<u8>,String>{
    if usize::from(id)>=CATALOG_COUNT || !(1..=2).contains(&banks.len()){
        return Err("operator selector or bank count invalid".to_owned());
    }
    let mut out=vec![1_u8,id,banks.len() as u8];
    for (bits,values) in banks {
        if !matches!(bits,6|8|10|12) || values.len()!=BANK_LEN {
            return Err("operator source bank cannot be framed".to_owned());
        }
        out.push(*bits);
        out.push(values.len() as u8);
        for symbol in values {
            if *symbol >= 1_u16<<bits {return Err("operator symbol outside floor".to_owned());}
            out.extend_from_slice(&symbol.to_le_bytes());
        }
    }
    Ok(out)
}
fn frame_replayed(frame:&[u8],id:u8,original:&[SymbolBank])
    ->Result<(),String>
{
    if frame.len()<3 || frame[0]!=1 || frame[1]!=id
        || usize::from(frame[2])!=original.len()
        || frame.len()!=3+original.len()*(2+2*BANK_LEN)
    {
        return Err("missing selector, floor, length or residual symbols".to_owned());
    }
    for (index,(bits,expected)) in original.iter().enumerate() {
        let start=3+index*(2+2*BANK_LEN);
        if frame[start]!=*bits || usize::from(frame[start+1])!=BANK_LEN {
            return Err("floor route selector and payload misaligned".to_owned());
        }
        let symbols=frame[start+2..start+2+BANK_LEN*2].chunks_exact(2)
            .map(|pair|u16::from_le_bytes([pair[0],pair[1]]))
            .collect::<Vec<_>>();
        let inverted=omega::invert_symbols(id,*bits,&symbols)
            .map_err(|e|e.to_string())?;
        if inverted!=*expected {return Err("Omnibit selector output not reversible".to_owned());}
    }
    Ok(())
}
fn verify_original(source:&Path,upstream:&Path)->Result<(String,String),String>{
    if omnibit_tournament::verify(source,upstream,upstream)?!=369 {
        return Err("original source-pinned 369-row public Omnibit gate not verified".to_owned());
    }
    let original=fs::read(source.join(ORIGINAL_PATH))
        .map_err(|e|format!("original Omega catalog unavailable: {e}"))?;
    if original!=LOCAL_CATALOG.as_bytes() {
        return Err("vendored Omega operator catalog differs from pinned original".to_owned());
    }
    let original_hash=hash_bytes(&original);
    let root=fs::read(upstream.join("PNP_OMNIBIT.hbp"))
        .map_err(|e|format!("Omnibit source receipt unavailable: {e}"))?;
    Ok((original_hash,hash_bytes(&root)))
}
type Bundle=(String,String,String,Vec<u8>);
fn make_bundle(source_sha:&str,prior_sha:&str)->Result<Bundle,String>{
    let cases=frozen_cases()?;
    if ![source_sha,prior_sha].iter().all(|s|s.len()==64) {
        return Err("original source or preceding Omnibit HBP root absent".to_owned());
    }
    if omega::OPERATORS.len()!=CATALOG_COUNT
        || omega::RNQ_COUNT!=8 || omega::SECTOR_COUNT!=12 || omega::LENS_COUNT!=20 {
        return Err("Omnibit source catalog shape changed".to_owned());
    }
    let mut hbp=HBP_HEADER.to_owned();
    let mut hbi=HBI_HEADER.to_owned();
    let mut payload=Vec::new();
    let mut previous=hash_bytes(b"PNP-OMNIBIT-40-MIXED-FLOOR-GENESIS-v1");
    let mut seq=0_usize;
    let mut append=|tag:&str,id:&str,fields:&str|{
        seq+=1;
        let data=format!(
            "{tag}|seq={seq}|id={id}|original_commit={SOURCE_COMMIT}|original_blob={SOURCE_BLOB}|original_sha256={source_sha}|upstream_sha256={prior_sha}|{fields}|prev_hash={previous}"
        );
        let digest=hash_bytes(data.as_bytes());
        let chain=hash_bytes(format!("{previous}|{digest}").as_bytes());
        let row=format!("{data}|receipt_sha={digest}|hash={chain}|json=0\n");
        hbi.push_str(&format!(
            "HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={chain}|status=EXACT_REPLAY|json=0\n",
            hbp.len(),row.len(),hash_bytes(row.as_bytes()),&digest[..16]
        ));
        hbp.push_str(&row);
        previous=chain;
    };
    append("OMNI40_SOURCE","omega_operator_catalog",&format!(
        "source_blob={SOURCE_BLOB}|source_content_sha256={source_sha}|operator_count=40|native_source=BYTE_IDENTICAL|original_public_gate=VERIFIED"
    ));
    for op in &omega::OPERATORS {
        let family=match op.id {0..=7=>"RNQ8",8..=19=>"TRI12",_=>"PI20"};
        append("OMNI40_OPERATOR",&format!("{}",op.id),&format!(
            "operator_id={}|name={}|family={family}|inverse_id={}|selector_bytes=1|state=PINNED_SOURCE",
            op.id,op.name,op.inverse_id
        ));
    }
    for (floor,bits,width) in FLOOR_DEFS {
        let mut valid=1_u16;
        for _ in 0..width {valid*=3;}
        append("OMNI40_FLOOR",&floor.to_string(),&format!(
            "floor={floor}|bits={bits}|native_trits={width}|valid_trit_codes={valid}|reserved_codes={}|operator_count=40|symbol_domain=FULL_FLOOR",
            floor-valid
        ));
    }
    let mut trials=0_usize;
    let mut selected=0_usize;
    let mut family=[0_usize;3];
    let mut charged_bytes=0_usize;
    let mut logical_steps=0_usize;
    let mut selected_zero=0_usize;
    for seat in SEATS {
        for view in VIEWS {
            for case in &cases {
                let mut data=case.data.clone();
                if view=="ANTI" {data.reverse();}
                let count=if data.len()==8{5}else{4};
                for choice in 0..count {
                    let (floor_label,banks)=banks_for_case(&data,choice)?;
                    let mut best=(usize::MAX,usize::MAX,u8::MAX);
                    for id in 0_u8..40 {
                        let scrambled=checked_omega(id,&banks)?;
                        let bytes=frame_encoded(id,&scrambled)?;
                        frame_replayed(&bytes,id,&banks)?;
                        let offset=payload.len();
                        payload.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        payload.extend_from_slice(&bytes);
                        let cost=bytes.len()+4;
                        let symbols=banks.len()*BANK_LEN;
                        let work=work_proxy(id,symbols,banks.len());
                        let candidate=(cost,work,id);
                        if candidate<best {best=candidate;}
                        family[if id<8{0}else if id<20{1}else{2}]+=1;
                        charged_bytes+=cost;
                        logical_steps+=work;
                        trials+=1;
                        let row_id=format!("{seat}_{view}_{}_{}_OP{id:02}",case.id,floor_label);
                        append("OMNI40_RUN",&row_id,&format!(
                            "seat_label={seat}|view_label={view}|seat_local_execution=0|original_view_binding=HOLD|fixture={}|floor={floor_label}|original_trit_count={}|banks={}|symbols={symbols}|operator_id={id}|family={}|payload_offset={offset}|payload_len={}|framed_bytes={cost}|payload_sha256={}|selector_bytes=1|logical_work_proxy={work}|inverse=EXACT|scoreboard=NOT_SCORED|winner=NONE",
                            case.id,data.len(),banks.len(),
                            if id<8{"RNQ8"}else if id<20{"TRI12"}else{"PI20"},
                            bytes.len(),hash_bytes(&bytes)
                        ));
                    }
                    selected+=1;
                    if best.2==0 {selected_zero+=1;}
                    append("OMNI40_SELECT",
                        &format!("{seat}_{view}_{}_{}",case.id,floor_label),
                        &format!(
                            "seat_label={seat}|view_label={view}|fixture={}|floor={floor_label}|candidate_count=40|selector_id={}|selector_bytes=1|charged_framed_bytes={}|logical_work_proxy={}|selection=MIN_BYTES_THEN_LOGICAL_WORK_THEN_ID|selection_scope=PUBLIC_FIXTURE|original_winner=NONE",
                            case.id,best.2,best.0,best.1
                        )
                    );
                }
            }
        }
    }
    if trials!=FRAME_RUNS || selected!=SELECTED_RUNS
        || family!=[SELECTED_RUNS*8,SELECTED_RUNS*12,SELECTED_RUNS*20]
        || selected_zero!=SELECTED_RUNS {
        return Err("Omnibit original operator census or fixed-rate selection changed".to_owned());
    }
    append("OMNI40_SUMMARY","public_operator_gate",&format!(
        "vantages=6|fixtures=20|catalog=40|floor_widths=6,8,10,12|rnq8_runs={}|tri12_runs={}|pi20_runs={}|full_floor_runs={}|composed_256_64_runs={}|trials={trials}|selection_decisions={selected}|selector_id_zero={selected_zero}|framed_bytes_total={charged_bytes}|logical_work_proxy_total={logical_steps}|payload_bytes={}|source_catalog=EXACT_COPY|invalid_symbols=REJECTED|independent_reference_tests=CI|seat_local_runs=0|scoreboard=NOT_SCORED|winner=NONE|full_decoder_charge=HOLD",
        family[0],family[1],family[2],CASES*6*4*CATALOG_COUNT,
        4*6*CATALOG_COUNT,payload.len()
    ));
    if seq!=ROWS || payload.is_empty() {return Err("Omnibit 40-source receipt tally incomplete".to_owned());}
    let root=format!(
        "OMNI40_ANCHOR|rows={ROWS}|trials={FRAME_RUNS}|selections={SELECTED_RUNS}|source_sha256={source_sha}|upstream_sha256={prior_sha}|payload_sha256={}|payload_bytes={}|hbp_sha256={}|hbi_sha256={}|terminal_hash={previous}|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(&payload),payload.len(),hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes())
    );
    let anchor=format!("{root}|checksum={}|json=0\n",hash_bytes(root.as_bytes()));
    Ok((hbp,hbi,anchor,payload))
}
pub fn attest(original:&Path,upstream:&Path,out:&Path)->Result<usize,String>{
    let (source_sha,prior_sha)=verify_original(original,upstream)?;
    let (hbp,hbi,anchor,payload)=make_bundle(&source_sha,&prior_sha)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,bytes) in [
        ("PNP_OMNI40.hbp",hbp.as_bytes()),
        ("PNP_OMNI40.hbi",hbi.as_bytes()),
        ("PNP_OMNI40.anchor",anchor.as_bytes()),
        ("PNP_OMNI40_PAYLOAD.bin",payload.as_slice())
    ]{
        fs::write(out.join(name),bytes).map_err(|e|format!("write original Omnibit catalog evidence: {e}"))?;
    }
    Ok(ROWS)
}
pub fn verify(original:&Path,upstream:&Path,out:&Path)->Result<usize,String>{
    let (source_sha,prior_sha)=verify_original(original,upstream)?;
    let (hbp,hbi,anchor,payload)=make_bundle(&source_sha,&prior_sha)?;
    for (name,expected) in [
        ("PNP_OMNI40.hbp",hbp.as_bytes()),
        ("PNP_OMNI40.hbi",hbi.as_bytes()),
        ("PNP_OMNI40.anchor",anchor.as_bytes()),
        ("PNP_OMNI40_PAYLOAD.bin",payload.as_slice())
    ]{
        let found=fs::read(out.join(name))
            .map_err(|e|format!("original Omnibit 40-operator evidence missing {name}: {e}"))?;
        if found!=expected {
            return Err(format!("Omnibit operator candidate archive mismatch: {name}"));
        }
    }
    Ok(ROWS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_exact_source_catalog_matches_all_40_families(){
        assert_eq!(omega::OPERATORS.len(),40);
        assert_eq!((omega::RNQ_COUNT,omega::SECTOR_COUNT,omega::LENS_COUNT),(8,12,20));
        assert_eq!(FRAME_RUNS,20160);
        assert_eq!(SELECTED_RUNS,504);
        assert_eq!(ROWS,20710);
        assert_eq!(frozen_cases().unwrap().len(),20);
    }
    #[test]
    fn every_original_operator_runs_on_every_binary_floor_and_compound(){
        let source=frozen_cases().unwrap();
        for case in source {
            for view in [false,true]{
                let mut data=case.data.clone();
                if view {data.reverse();}
                for choice in 0..if data.len()==8{5}else{4}{
                    let (_,banks)=banks_for_case(&data,choice).unwrap();
                    for id in 0_u8..40 {
                        let modified=checked_omega(id,&banks).unwrap();
                        let encoded=frame_encoded(id,&modified).unwrap();
                        frame_replayed(&encoded,id,&banks).unwrap();
                    }
                }
            }
        }
    }
    #[test]
    fn strict_selector_floor_and_payload_corruption_reject(){
        let data=[Trit::Minus,Trit::Plus,Trit::Zero,Trit::Minus,Trit::Plus,Trit::Zero,Trit::Plus,Trit::Minus];
        let (_,banks)=banks_for_case(&data,4).unwrap();
        let modified=checked_omega(29,&banks).unwrap();
        let framed=frame_encoded(29,&modified).unwrap();
        frame_replayed(&framed,29,&banks).unwrap();
        let mut tampered=framed.clone();
        tampered[1]=40;
        assert!(frame_replayed(&tampered,29,&banks).is_err());
        let mut tampered=framed.clone();
        tampered[3]=10;
        assert!(frame_replayed(&tampered,29,&banks).is_err());
        let mut tampered=framed.clone();
        tampered[5]^=1;
        assert!(frame_replayed(&tampered,29,&banks).is_err());
        let mut tampered=framed;
        tampered.pop();
        assert!(frame_replayed(&tampered,29,&banks).is_err());
        assert!(checked_omega(40,&banks).is_err());
    }
    #[test]
    fn active_nontrivial_tri_and_pi_change_twenty_lane_banks(){
        let input=(0..20).map(|v|v as u16).collect::<Vec<_>>();
        let tri=omega::apply_symbols(9,8,&input).unwrap();
        let lens=omega::apply_symbols(29,8,&input).unwrap();
        assert_ne!(tri,input);
        assert_ne!(lens,input);
        assert_eq!(omega::invert_symbols(9,8,&tri).unwrap(),input);
        assert_eq!(omega::invert_symbols(29,8,&lens).unwrap(),input);
    }
}
