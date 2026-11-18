//! Original Asolaria Pump archive evidence gate.
//! The exact July 2026 six-arm 27-kernel matrix receipt and full inert
//! experiment data are byte-frozen; NO original ACER pump_*.py execution
//! is asserted here. The source-equivalent 3->9->27 kernel awaits import.
//! Only integer parsing and native HBI/HBP/SHA/SH/HASH verification.
#![forbid(unsafe_code)]
use std::collections::{BTreeMap,BTreeSet};
use std::fs::{self,File};
use std::io::Write;
use std::path::Path;
use crate::sha256::hash_bytes;

const ORIGINAL_HBP:&[u8]=include_bytes!("../fixtures/ASOLARIA_27_PUMP_ARCHIVE.hbp");
const ORIGINAL_DATA:&[u8]=include_bytes!("../fixtures/ASOLARIA_27_PUMP_ARCHIVE.json");
const HBP_SHA:&str="94e665a5e64bcfe39ee18eb0ad95629ad8d767f0c610d1e290efa5a089e1b7b4";
const DATA_SHA:&str="132bd4d57c9f151a52837cd7151454db0b15dd766a0f39ade0836bee2ee947f5";
const INPUT_KEY_SHA:&str="925c818d4c84aa23d7cee5d13e1b142b90824724a4ed4a90f9ee019c59cc2d89";
const WASM_SHA:&str="0c05c67dcf6041810e7cd50219988aa2f7312bb948a78e5e05bf182fcd2cc76c";
const PREDECESSORS:[&str;3]=[
    "PNP_FULL_MATRIX.hbp",
    "PNP_PRL_MEMORY_FABRIC.hbp",
    "PNP_GNN_GATE.hbp",
];
const ARMS:[(&str,usize,usize,usize);6]=[
    ("full_27",27,81,89),
    ("binary_18",18,45,77),
    ("no_center",27,81,96),
    ("no_callings",27,81,96),
    ("oil_shuffled",27,81,89),
    ("matched_control",27,81,86),
];
const HBP_HEADER:&str="HBPHEADER|schema=pnp.asolaria.pump.archive.v1|scope=SOURCE_RECEIPT_ONLY|runtime=SOURCE_IMPORT_HOLD|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HBI_HEADER:&str="HBIHEADER|schema=pnp.asolaria.pump.archive.index.v1|for=PNP_PUMP_PROVENANCE.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct ArchivedArm{
    pub name:String,
    pub nodes:usize,
    pub edges:usize,
    pub accepted_steps:usize,
    pub energy_initial_q9:u128,
    pub energy_final_q9:u128,
}
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Stats{
    pub original_hbp_sha:String,
    pub original_json_sha:String,
    pub upstream_sha:Vec<String>,
    pub arms:Vec<ArchivedArm>,
    pub graph_nodes:usize,
    pub graph_edges:usize,
    pub macro_kernels:usize,
}
fn field<'a>(line:&'a str,name:&str)->Result<&'a str,String>{
    line.split('|')
        .find_map(|part|part.strip_prefix(name).and_then(|s|s.strip_prefix('=')))
        .ok_or_else(||format!("missing original pump field {name}"))
}
fn integer(line:&str,name:&str)->Result<usize,String>{
    field(line,name)?.parse().map_err(|_|format!("noninteger original pump {name}"))
}
fn q9(value:&str)->Result<u128,String>{
    let (integer,fraction)=value.split_once('.')
        .ok_or("original pump energy must be fixed 9-decimal number")?;
    if fraction.len()!=9 || !integer.bytes().all(|b|b.is_ascii_digit())
        || !fraction.bytes().all(|b|b.is_ascii_digit()){
        return Err("original pump fixed-point energy digits invalid".into());
    }
    let whole=integer.parse::<u128>().map_err(|_|"original pump integer overflow")?;
    let minor=fraction.parse::<u128>().map_err(|_|"original pump decimal overflow")?;
    whole.checked_mul(1_000_000_000).and_then(|v|v.checked_add(minor))
        .ok_or("original pump Q9 overflow".into())
}
pub fn original_evidence()->Result<Stats,String>{
    let hbp_sha=hash_bytes(ORIGINAL_HBP);
    let data_sha=hash_bytes(ORIGINAL_DATA);
    if hbp_sha!=HBP_SHA || data_sha!=DATA_SHA
        || ORIGINAL_HBP.len()!=2203 || ORIGINAL_DATA.len()!=29_825 {
        return Err("original matrix pump archive bytes changed".into());
    }
    let text=std::str::from_utf8(ORIGINAL_HBP)
        .map_err(|_|"original pump HBP UTF-8 invalid")?;
    let lines=text.lines().collect::<Vec<_>>();
    if lines.len()!=16{
        return Err("original six-arm pump receipt count changed".into());
    }
    let run=lines.first().ok_or("pump run header absent")?;
    if !run.starts_with("KNETRUN|schema=ASOLARIA-27-KERNEL-MATRIX-NETWORK-V1|")
        || field(lines[1],"key_sha256")?!=INPUT_KEY_SHA
        || field(lines[2],"sha256")?!=WASM_SHA
        || field(lines[2],"prism_exact")?!="1"
        || field(lines[2],"chain_intact")?!="1"
        || field(lines[3],"points")?!="1026"
    {
        return Err("original 27-pump provenance anchors changed".into());
    }
    let arch=lines[4];
    if integer(arch,"stars")?!=3 || integer(arch,"oil_family")?!=3
        || integer(arch,"sign")?!=3 || integer(arch,"nodes")?!=27
        || integer(arch,"macro_kernels")?!=9
        || field(arch,"shared_center")?!="ENERGY_0"
        || integer(arch,"center_independent")?!=0
    {
        return Err("pump nine-kernel star/oil/sign architecture altered".into());
    }
    if field(lines[5],"r0_identity")?!="1" || field(lines[5],"r1_order")?!="3"
        || field(lines[5],"r2_equals_r1_squared")?!="1"
        || field(lines[6],"graph")?!="HAMMING_3X3X3"
        || integer(lines[6],"edges")?!=81 || field(lines[6],"endpoints_retained")?!="1"{
        return Err("pump graph and phase-rotation source altered".into());
    }
    let mut arms=Vec::new();
    let mut unique=BTreeSet::new();
    for (index,(name,nodes,edges,steps)) in ARMS.iter().enumerate(){
        let line=lines[index+7];
        if !line.starts_with("ARM|") || field(line,"name")?!=*name
            || integer(line,"nodes")?!=*nodes || integer(line,"edges")?!=*edges
            || integer(line,"steps")?!=*steps || field(line,"monotone")?!="1"
            || !unique.insert(name){
            return Err("original pump control family or metadata changed".into());
        }
        let initial=q9(field(line,"energy_initial")?)?;
        let final_energy=q9(field(line,"energy_final")?)?;
        if final_energy>=initial{return Err("archived pump objective descent absent".into());}
        arms.push(ArchivedArm{name:(*name).into(),nodes:*nodes,edges:*edges,
            accepted_steps:*steps,energy_initial_q9:initial,
            energy_final_q9:final_energy});
    }
    if arms[0].energy_initial_q9!=76_373_193_019
        || arms[0].energy_final_q9!=14_052_388_015
        || arms[1].energy_initial_q9!=47_697_165_113
        || arms[1].energy_final_q9!=7_034_875_627
        || arms[5].energy_final_q9!=14_264_692_576{
        return Err("historical pump objective numbers changed".into());
    }
    if field(lines[13],"bijection_rate_1")?!="PASS"
        || field(lines[13],"negative_entropy")?!="NOT_SUPPORTED"
        || field(lines[14],"network_exists")?!="MEASURED"
        || field(lines[14],"trinary_superiority_over_binary")?!="NOT_ESTABLISHED"
        || field(lines[14],"key_specificity")?!="SMALL"
        || field(lines[15],"hot_path")?!="1"{
        return Err("historical pump caveat or source verdict altered".into());
    }
    Ok(Stats{original_hbp_sha:hbp_sha,original_json_sha:data_sha,
        upstream_sha:Vec::new(),arms,
        graph_nodes:27,graph_edges:81,macro_kernels:9})
}
fn load_origins(source:&Path)->Result<Vec<String>,String>{
    let mut sha=Vec::with_capacity(PREDECESSORS.len());
    for name in PREDECESSORS{
        let content=fs::read(source.join(name))
            .map_err(|e|format!("missing protected upstream pump source {name}: {e}"))?;
        if !content.starts_with(b"HBPHEADER|") || content.len()<80{
            return Err(format!("invalid protected predecessor {name}"));
        }
        sha.push(hash_bytes(&content));
    }
    Ok(sha)
}
fn row(hbp:&mut String,hbi:&mut String,prior:&mut String,index:usize,
        label:&str,id:&str,data:&str){
    let bare=format!("{label}|seq={index}|id={id}|{data}|prior_sha={prior}");
    let hash=hash_bytes(bare.as_bytes());
    let chained=hash_bytes(format!("{prior}|{hash}").as_bytes());
    let raw=format!("{bare}|source_row_sha={hash}|hash={chained}|json=0\n");
    hbi.push_str(&format!(
        "HBI|seq={index}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={chained}|json=0\n",
        hbp.len(),raw.len(),hash_bytes(raw.as_bytes()),&hash[..16]
    ));
    hbp.push_str(&raw);
    *prior=chained;
}
type Bundle=(String,String,String);
fn calculate(source:&Path)->Result<(Bundle,Stats),String>{
    let mut stats=original_evidence()?;
    stats.upstream_sha=load_origins(source)?;
    let mut hbp=HBP_HEADER.to_owned();
    let mut hbi=HBI_HEADER.to_owned();
    let mut previous=hash_bytes(b"ASOLARIA-PUMP-SOURCE-ARCHIVE-GENESIS");
    row(&mut hbp,&mut hbi,&mut previous,1,"PUMP_ORIGIN","network_27",
        &format!("receipt_sha256={}|original_json_sha256={}|source_status=ARCHIVAL_BENCHMARK|raw_pump_py=SOURCE_IMPORT_HOLD|original_experiment=JULY_2026|json_runtime=0",
            stats.original_hbp_sha,stats.original_json_sha));
    for (index,name) in PREDECESSORS.iter().enumerate(){
        row(&mut hbp,&mut hbi,&mut previous,index+2,"PUMP_PREDECESSOR",name,
            &format!("source_sha256={}|status=PRIOR_ACCEPTANCE",stats.upstream_sha[index]));
    }
    row(&mut hbp,&mut hbi,&mut previous,5,"PUMP_GRAPH","27nodes",
        "stars=3|oil_phases=3|signed_axes=3|macro_kernels=9|hamming_edges=81|shared_center=ENERGY_0|phase_rotation=ORDER_3");
    for (i,arm) in stats.arms.iter().enumerate(){
        row(&mut hbp,&mut hbi,&mut previous,i+6,"PUMP_ARCHIVED_ARM",&arm.name,
            &format!("nodes={}|edges={}|accepted_steps={}|q9_energy_initial={}|q9_energy_final={}|monotone_receipt=1|status=ARCHIVED_REPORTED",
                arm.nodes,arm.edges,arm.accepted_steps,
                arm.energy_initial_q9,arm.energy_final_q9));
    }
    row(&mut hbp,&mut hbi,&mut previous,12,"PUMP_BOUNDARY","source",
        "acer_local_origin=D_COLON_ASOLARIA_ABSORB_KERNEL_PUMP|kernel_ladder=3_9_27|original_python_bytes=NOT_IMPORTED|rust_port=HOLD|matrix_pump=SEPARATE_ARCHIVAL_ARM|trained_gnn_forward=HOLD");
    row(&mut hbp,&mut hbi,&mut previous,13,"PUMP_VERDICT","historical",
        "objective=DECLARED_ENERGY|optimization=BACKTRACK_DESCENT|negative_entropy=NOT_SUPPORTED|superior_to_binary=NOT_ESTABLISHED|edge_residual_advantage=SMALL|corresponding_runtime_reproduction=HOLD|status=PASS_ARCHIVE_ONLY");
    let basis=format!(
        "PNP_PUMP_ANCHOR|rows=13|original_hbp_sha={}|original_json_sha={}|hbp_sha={}|hbi_sha={}|hash={previous}|status=SOURCE_LOCK",
        stats.original_hbp_sha,stats.original_json_sha,hash_bytes(hbp.as_bytes()),hash_bytes(hbi.as_bytes()));
    let anchor=format!("{basis}|checksum={}|json=0\n",hash_bytes(basis.as_bytes()));
    Ok(((hbp,hbi,anchor),stats))
}
fn persist(path:&Path,content:&[u8])->Result<(),String>{
    let mut f=File::create(path).map_err(|e|e.to_string())?;
    f.write_all(content).map_err(|e|e.to_string())?;
    f.sync_all().map_err(|e|e.to_string())
}
pub fn attest(source:&Path,out:&Path)->Result<Stats,String>{
    let ((hbp,hbi,anchor),stats)=calculate(source)?;
    fs::create_dir_all(out).map_err(|e|e.to_string())?;
    for (name,bytes) in [
        ("PNP_PUMP_PROVENANCE.hbp",hbp.as_bytes()),
        ("PNP_PUMP_PROVENANCE.hbi",hbi.as_bytes()),
        ("PNP_PUMP_PROVENANCE.anchor",anchor.as_bytes()),
    ]{persist(&out.join(name),bytes)?;}
    Ok(stats)
}
pub fn verify(source:&Path,out:&Path)->Result<Stats,String>{
    let ((hbp,hbi,anchor),stats)=calculate(source)?;
    for (name,expected) in [
        ("PNP_PUMP_PROVENANCE.hbp",hbp.as_bytes()),
        ("PNP_PUMP_PROVENANCE.hbi",hbi.as_bytes()),
        ("PNP_PUMP_PROVENANCE.anchor",anchor.as_bytes()),
    ]{
        if fs::read(out.join(name)).map_err(|e|e.to_string())?!=expected{
            return Err(format!("pump provenance artifact altered: {name}"));
        }
    }
    Ok(stats)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn six_original_arm_receipts_are_source_frozen_and_exact_q9(){
        let stats=original_evidence().unwrap();
        assert_eq!((stats.graph_nodes,stats.graph_edges,stats.macro_kernels),(27,81,9));
        assert_eq!(stats.arms.len(),6);
        assert_eq!(stats.arms[0].energy_initial_q9,76_373_193_019);
        assert_eq!(stats.arms[0].energy_final_q9,14_052_388_015);
        assert_eq!(stats.arms[5].name,"matched_control");
        assert_eq!(q9("14.052388015").unwrap(),14_052_388_015);
        assert!(q9("14.05238801").is_err());
        assert!(q9("-1.000000000").is_err());
        assert!(q9("99999999999999999999999999999.999999999").is_err());
    }
}
