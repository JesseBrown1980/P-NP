//! Reversible original Asolaria binary-floor / balanced-ternary address bridge.
//! 243 valid five-trit states occupy byte codes 0..242; 243..255 remain
//! explicitly reserved and are rejected as data without a domain-specific tag.
#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;

use crate::{fabric_gc, matrix, prism, sha256::hash_bytes, ternary::Trit};

const HBP_HEADER: &str =
    "HBPHEADER|schema=pnp.binary.ternary.bridge.v1|mode=EXACT_ADDRESS|seal=HBI-HBP-SHA-SH-HASH|reserved_codes=REJECT_DATA|json=0\n";
const HBI_HEADER: &str =
    "HBIHEADER|schema=pnp.binary.ternary.bridge.index.v1|for=PNP_BRIDGE.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";

const POW3: [u16; 9] = [1,3,9,27,81,243,729,2187,6561];
const FLOOR_DEFS: [(u16, usize); 4] = [(64,3),(256,5),(1024,6),(4096,7)];
const LADDER: [usize; 8] = [1,2,3,4,5,6,7,8];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FloorWord {
    pub floor: u16,
    pub code: u16,
}
fn digits(floor: u16) -> Option<usize> {
    FLOOR_DEFS.iter().find(|(f,_)| *f == floor).map(|(_,d)| *d)
}
fn trit_digit(value: Trit) -> u16 {
    match value {
        Trit::Minus => 0, Trit::Zero => 1, Trit::Plus => 2,
    }
}
fn from_digit(value: u16) -> Result<Trit, &'static str> {
    match value {
        0 => Ok(Trit::Minus), 1 => Ok(Trit::Zero), 2 => Ok(Trit::Plus),
        _ => Err("trit digit outside radix 3"),
    }
}

/// The binary floor contains the *valid* ternary alphabet as an injection.
/// Its remaining codes are reserved and have no automatic data interpretation.
pub fn encode_floor(floor: u16, trits: &[Trit]) -> Result<FloorWord, &'static str> {
    let width = digits(floor).ok_or("unknown binary floor")?;
    if trits.len() != width {
        return Err("trit count not equal to floor width");
    }
    let mut code = 0_u16;
    for &value in trits {
        code = code * 3 + trit_digit(value);
    }
    if code >= floor {
        return Err("binary floor cannot hold ternary value");
    }
    Ok(FloorWord { floor, code })
}
pub fn decode_floor(word: FloorWord) -> Result<Vec<Trit>, &'static str> {
    let width = digits(word.floor).ok_or("unknown binary floor")?;
    if word.code >= POW3[width] {
        return Err("reserved binary floor code cannot decode as ternary data");
    }
    let mut x = word.code;
    let mut out = vec![Trit::Zero; width];
    for i in (0..width).rev() {
        out[i] = from_digit(x % 3)?;
        x /= 3;
    }
    if x != 0 {
        return Err("invalid ternary ordinal");
    }
    Ok(out)
}

/// A trit sequence is encoded as the smallest certified floor in the fixed
/// ladder. Eight trits use 256(5 trits) + 64(3 trits); no information loss.
pub fn encode_route(trits: &[Trit]) -> Result<Vec<FloorWord>, &'static str> {
    let (floor, take) = match trits.len() {
        1..=3 => (64,3), 4..=5 => (256,5),
        6 => (1024,6), 7 => (4096,7),
        8 => (256,5),
        _ => return Err("route accepts 1 through 8 trits"),
    };
    let mut first = trits[..trits.len().min(take)].to_vec();
    first.resize(take,Trit::Zero);
    let mut out = vec![encode_floor(floor,&first)?];
    if trits.len() == 8 {
        out.push(encode_floor(64,&trits[5..])?);
    }
    Ok(out)
}
pub fn decode_route(words: &[FloorWord], ntrits: usize)
    -> Result<Vec<Trit>, &'static str>
{
    let (floor, take, nwords) = match ntrits {
        1..=3 => (64,3,1), 4..=5 => (256,5,1),
        6 => (1024,6,1), 7 => (4096,7,1),
        8 => (256,5,2),
        _ => return Err("route accepts 1 through 8 trits"),
    };
    if words.len() != nwords || words[0].floor != floor
        || (ntrits == 8 && words[1].floor != 64)
    {
        return Err("binary floor path or length changed");
    }
    let mut out = decode_floor(words[0])?;
    if ntrits == 8 {
        out.extend(decode_floor(words[1])?);
    } else {
        if out[take - (take - ntrits)..].iter().any(|v| *v != Trit::Zero) {
            return Err("noncanonical zero-padding trits");
        }
        out.truncate(ntrits);
    }
    Ok(out)
}
fn ordinal_to_trits(mut ordinal: u16, ntrits: usize) -> Vec<Trit> {
    let mut out = vec![Trit::Zero; ntrits];
    for index in (0..ntrits).rev() {
        out[index] = from_digit(ordinal % 3).expect("base3 digit");
        ordinal /= 3;
    }
    out
}
fn full_exhaustive() -> Result<(usize, usize, usize, usize), String> {
    let mut floor_cases = 0_usize;
    for (floor,width) in FLOOR_DEFS {
        for ordinal in 0..POW3[width] {
            let trits = ordinal_to_trits(ordinal,width);
            let word = encode_floor(floor,&trits).map_err(str::to_owned)?;
            if word.code != ordinal || decode_floor(word).map_err(str::to_owned)? != trits {
                return Err(format!("floor identity mismatch at {floor}:{ordinal}"));
            }
            floor_cases += 1;
        }
        // Independently test every rejected control/reserved code on all floors.
        for reserved in POW3[width]..floor {
            if decode_floor(FloorWord{floor,code:reserved}).is_ok() {
                return Err(format!("reserved floor symbol accepted: {floor}:{reserved}"));
            }
        }
    }
    let mut routed = 0_usize;
    for ntrits in LADDER {
        for ordinal in 0..POW3[ntrits] {
            let data = ordinal_to_trits(ordinal,ntrits);
            let route = encode_route(&data).map_err(str::to_owned)?;
            if decode_route(&route,ntrits).map_err(str::to_owned)? != data {
                return Err(format!("binary/ternary route inverse mismatch: {ntrits}:{ordinal}"));
            }
            if route.iter().any(|word| word.code >= word.floor) {
                return Err("out-of-floor route".to_owned());
            }
            routed += 1;
        }
    }
    let mut fabric = 0_usize;
    for ordinal in 0..27 {
        let t = ordinal_to_trits(ordinal,3);
        let x = usize::from(trit_digit(t[2]));
        let y = usize::from(trit_digit(t[1]));
        let z = usize::from(trit_digit(t[0]));
        let a = matrix::address(x,y,z).ok_or("missing matrix room")?;
        if a != usize::from(ordinal) || matrix::coordinates(a) != Some((x,y,z)) {
            return Err("27-cell matrix witness mapping incorrect".to_owned());
        }
        fabric += 1;
    }
    if matrix::OMEGA != 13
        || matrix::kind(13) != Some(matrix::CellKind::Omega)
        || encode_floor(256,&[Trit::Zero;5]).map_err(str::to_owned)?.code != 121
    {
        return Err("ground-center witness changed".to_owned());
    }
    // Existing source-authenticated PRISM 256<->1024 translations must preserve
    // the exact five-trit byte domain and produce an identical inverse.
    let valid: Vec<u8> = (0..243).map(|i| i as u8).collect();
    let glyphs = prism::encode(&valid);
    if prism::decode(&glyphs,valid.len()).map_err(str::to_owned)? != valid {
        return Err("PRISM/ternary bridge violated byte identity".to_owned());
    }
    Ok((floor_cases,routed,fabric,glyphs.len()))
}

fn upstream(gac: &Path, neural: &Path, omni: &Path, prism_source: &Path,
            receipts: &Path) -> Result<(String,String), String>
{
    if prism::verify(prism_source,receipts)? != 15
        || fabric_gc::verify(gac,neural,omni,receipts)? != 16
    {
        return Err("source-pinned PRISM/GAC evidence incomplete".to_owned());
    }
    let p = fs::read(receipts.join("PNP_PRISM.hbp"))
        .map_err(|e| format!("original PRISM upstream missing: {e}"))?;
    let f = fs::read(receipts.join("PNP_FABRIC_GC.hbp"))
        .map_err(|e| format!("original GAC upstream missing: {e}"))?;
    Ok((hash_bytes(&p),hash_bytes(&f)))
}

type Bundle = (String,String,String);
fn make_bundle(prism_root: &str, fabric_root: &str) -> Result<Bundle,String> {
    let (floor_checks,routed,matrix_checks,prism_glyphs) = full_exhaustive()?;
    let mut hbp = HBP_HEADER.to_owned();
    let mut hbi = HBI_HEADER.to_owned();
    let mut prior = hash_bytes(b"PNP-ASOLARIA-BINARY-TERNARY-BRIDGE-v1");
    let mut index = 0_usize;
    let mut append = |tag: &str, id: &str, data: &str| {
        index += 1;
        let contents = format!(
            "{tag}|seq={index}|id={id}|source_prism_sha256={prism_root}|source_fabric_sha256={fabric_root}|{data}|prev_hash={prior}"
        );
        let receipt = hash_bytes(contents.as_bytes());
        let chain = hash_bytes(format!("{prior}|{receipt}").as_bytes());
        let row = format!("{contents}|receipt_sha={receipt}|hash={chain}|json=0\n");
        hbi.push_str(&format!(
            "HBI|seq={index}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={chain}|status=INVERSE_VERIFIED|json=0\n",
            hbp.len(),row.len(),hash_bytes(row.as_bytes()),&receipt[..16]
        ));
        hbp.push_str(&row);
        prior=chain;
    };
    for (floor,width) in FLOOR_DEFS {
        append("BRIDGE_FLOOR",&floor.to_string(),&format!(
            "floor={floor}|width={width}|ternary_states={}|reserved={}|roundtrip=EXHAUSTIVE|reserved_policy=REJECT_AS_DATA",
            POW3[width], floor - POW3[width]
        ));
    }
    for ntrits in LADDER {
        let (floor,second) = match ntrits {
            1..=3 => (64, "none"),
            4..=5 => (256,"none"),
            6 => (1024,"none"), 7 => (4096,"none"),
            8 => (256,"64"),
            _ => return Err("unknown ternary ladder length".to_owned()),
        };
        append("BRIDGE_LADDER",&ntrits.to_string(),&format!(
            "trits={ntrits}|states={}|floor_first={floor}|floor_second={second}|roundtrip=EXHAUSTIVE",
            POW3[ntrits]
        ));
    }
    for reserved in 243_u16..256 {
        append("BRIDGE_RESERVED",&reserved.to_string(),
            &format!("floor=256|code={reserved}|as_data=REJECT|meaning=UNASSIGNED"));
    }
    append("BRIDGE_MATRIX","center27",&format!(
        "cells={matrix_checks}|center_address=13|center5trit_code=121|floor_checks={floor_checks}|routed_checks={routed}|prism_input_bytes=243|prism_glyphs={prism_glyphs}|source_index=PINNED|roundtrip=PASS"
    ));
    let anchor_data = format!(
        "BRIDGE_ANCHOR|rows={index}|floor_checks={floor_checks}|ladder_checks={routed}|reserved_byte_codes=13|matrix_checks={matrix_checks}|prism_hbp_sha256={prism_root}|fabric_hbp_sha256={fabric_root}|hbp_sha256={}|hbi_sha256={}|hash={prior}|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes())
    );
    let anchor = format!("{anchor_data}|checksum={}|json=0\n",hash_bytes(anchor_data.as_bytes()));
    Ok((hbp,hbi,anchor))
}
pub fn attest(gac: &Path, neural: &Path, omni: &Path,
              prism_source: &Path, input: &Path, output: &Path)
    -> Result<usize,String>
{
    let (prism_root,fabric_root) = upstream(gac,neural,omni,prism_source,input)?;
    let (hbp,hbi,anchor) = make_bundle(&prism_root,&fabric_root)?;
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    for (name,bytes) in [
        ("PNP_BRIDGE.hbp",hbp),("PNP_BRIDGE.hbi",hbi),
        ("PNP_BRIDGE.anchor",anchor)
    ] {
        fs::write(output.join(name),bytes).map_err(|e|format!("bridge artifact: {e}"))?;
    }
    Ok(26)
}
pub fn verify(gac: &Path, neural: &Path, omni: &Path,
              prism_source: &Path, input: &Path, output: &Path)
    -> Result<usize,String>
{
    let (prism_root,fabric_root) = upstream(gac,neural,omni,prism_source,input)?;
    let (hbp,hbi,anchor) = make_bundle(&prism_root,&fabric_root)?;
    for (name,expected) in [
        ("PNP_BRIDGE.hbp",hbp),("PNP_BRIDGE.hbi",hbi),
        ("PNP_BRIDGE.anchor",anchor)
    ] {
        if fs::read(output.join(name))
            .map_err(|e|format!("bridge missing {name}: {e}"))? != expected.as_bytes()
        {
            return Err(format!("bridge receipt was changed: {name}"));
        }
    }
    Ok(26)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn five_trit_byte_encoding_is_exact_and_reserved_codes_reject() {
        assert_eq!(encode_floor(256,&[Trit::Minus;5]).unwrap().code,0);
        assert_eq!(encode_floor(256,&[Trit::Zero;5]).unwrap().code,121);
        assert_eq!(encode_floor(256,&[Trit::Plus;5]).unwrap().code,242);
        assert_eq!(encode_floor(256,&[
            Trit::Plus,Trit::Minus,Trit::Zero,Trit::Plus,Trit::Zero
        ]).unwrap().code,178);
        for i in 0..243_u16 {
            let word=FloorWord{floor:256,code:i};
            let trits=decode_floor(word).unwrap();
            assert_eq!(encode_floor(256,&trits),Ok(word));
        }
        for i in 243..=255_u16 {
            assert!(decode_floor(FloorWord{floor:256,code:i}).is_err());
        }
    }
    #[test]
    fn all_four_original_binary_floors_and_eight_ternary_landings_exhaustive() {
        assert_eq!(full_exhaustive(),Ok((3186,9840,27,195)));
    }
    #[test]
    fn eighth_trit_routes_across_two_binary_floors() {
        let data = [Trit::Minus,Trit::Zero,Trit::Plus,Trit::Minus,
                    Trit::Plus,Trit::Zero,Trit::Minus,Trit::Plus];
        let path=encode_route(&data).unwrap();
        assert_eq!(path.len(),2);
        assert_eq!(path[0].floor,256);
        assert_eq!(path[1].floor,64);
        assert_eq!(decode_route(&path,8),Ok(data.to_vec()));
        let mut wrong=path;
        wrong[1].floor=256;
        assert!(decode_route(&wrong,8).is_err());
    }
    #[test]
    fn reject_noncanonical_padding_and_floor_interchange() {
        let raw=[Trit::Zero;4];
        let mut one=encode_route(&raw).unwrap();
        assert_eq!(one.len(),1);
        one[0].code+=1;
        assert!(decode_route(&one,4).is_err());
        let all=encode_route(&[Trit::Plus;5]).unwrap();
        assert!(decode_route(&all,3).is_err());
        assert!(encode_floor(64,&[Trit::Plus;5]).is_err());
        assert!(decode_floor(FloorWord{floor:16,code:0}).is_err());
        assert!(encode_route(&[]).is_err());
    }
    #[test]
    fn original_matrix_centre_is_independent_of_sat_unknown() {
        assert_eq!(matrix::address(1,1,1),Some(matrix::OMEGA));
        assert_eq!(matrix::OMEGA,13);
        assert_eq!(matrix::kind(13),Some(matrix::CellKind::Omega));
        assert_eq!(decode_floor(FloorWord{floor:256,code:121}),
                   Ok(vec![Trit::Zero;5]));
    }
}
