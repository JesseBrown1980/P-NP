//! Source-pinned 3-D GitHub THRUTH and SAFE-ASI temporal provenance adapter.
//! Real final receipts, source Git objects and bounded observed-history rings.
//! Exact Rust 1.81, no external crates, no automatic network capability.
#![forbid(unsafe_code)]

use pnp_native::sha256::hash_bytes;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

const FOLLOW_COMMIT: &str = "8712368c851abe347ddf97a25bfbd96275d4316c";
const SAFE_COMMIT: &str = "6e3e082d574a4ad45bceb06ccf0b86ac44f942cc";
const THREE_D_BLOB: &str = "fc2e3f83c7bc6ef6232a359c5d0b690ec64ce323";
const THREE_D_SHA: &str = "56bc5ee06e9bb4f7780668005d92793abbab8291094fb8ca56e46a45efccf1e4";
const GRAPH_BLOB: &str = "9ea40d80a16b9c4e45608bfffb2a2199e88b5ae2";
const GRAPH_SHA: &str = "258699681441f7c20a34e8de8c60fd8b0c8b0bc282bf31e60d4ad78f92a75cff";
const SAFE_LANGUAGE_BLOB: &str = "d9c5a62297976ac931f7924d36048e69e7e99ad0";
const SAFE_INTEGRITY_BLOB: &str = "c062c10370122b10b6905040f3dc22cbde4b90ed";
const SAFE_RESPONSE_BLOB: &str = "d5d297211a4a33f4cd04dd73a706c5f99a4cd19d";
const CHARIOT_SEAL: &str = "2b1889ee7a3a18ddd54cd9ec4229dbb266bdafd6f03cb9d2cf5f979cbc23967e";
const MEMBERS: &str = "HBI,HBP,SHA,SH,HASH";
const TRAVERSAL: &str = "HBI->HBP->SH->HASH->SHA";
const FAMILIES: [&str; 3] = ["BROWN", "ANTI_BROWN", "ANTI_ANTI_BROWN"];
const DETECTORS: [&str; 4] = ["BYTE_COMMITMENT", "CLAIM_EVIDENCE", "MEDIA_BINDING", "RUNTIME_AUTHORITY"];
const DIRECTIONS: [&str; 3] = ["NEGATIVE", "CENTRE", "POSITIVE"];
const MAX_BYTES: usize = 128 * 1024;

fn ensure(ok: bool, why: &str) -> Result<(), String> {
    if ok { Ok(()) } else { Err(why.to_owned()) }
}
fn hex40(v: &str) -> bool {
    v.len() == 40 && v.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn hex64(v: &str) -> bool {
    v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn safe_run(v: &str) -> bool {
    !v.is_empty() && v.len() <= 128 && v.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn read(path: &Path) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    ensure(meta.file_type().is_file() && !meta.file_type().is_symlink(), "source is not a regular file")?;
    ensure(meta.len() > 0 && meta.len() <= MAX_BYTES as u64, "source byte bound")?;
    fs::read(path).map_err(|e| e.to_string())
}
fn text_file(path: &Path) -> Result<String, String> {
    let bytes = read(path)?;
    ensure(!bytes.contains(&b'\r') && bytes.last() == Some(&b'\n'), "file requires exact LF framing")?;
    String::from_utf8(bytes).map_err(|_| "file is not UTF8".to_owned())
}
fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let result = Command::new("git").arg("-C").arg(dir).args(args).output()
        .map_err(|e| format!("git unavailable: {e}"))?;
    ensure(result.status.success(), "pinned Git source query failed")?;
    String::from_utf8(result.stdout).map(|s| s.trim().to_owned())
        .map_err(|_| "Git output is not UTF8".to_owned())
}
fn source_pin(dir: &Path, commit: &str, blobs: &[(&str, &str)]) -> Result<(), String> {
    ensure(git(dir, &["rev-parse", "HEAD"])? == commit, "source commit changed")?;
    for (path, expected) in blobs {
        ensure(git(dir, &["hash-object", path])? == *expected, "original Git source blob changed")?;
    }
    Ok(())
}
fn source_check(follow: &Path, safe: &Path) -> Result<(), String> {
    source_pin(follow, FOLLOW_COMMIT, &[
        ("matrix/3-D-GITHUB-OF-THRUTH.md", THREE_D_BLOB),
        ("matrix/GITHUB-THREE-DIMENSIONALLY-RIMED-2026-07-29.hbp", GRAPH_BLOB),
    ])?;
    source_pin(safe, SAFE_COMMIT, &[
        ("safe_asi/language.py", SAFE_LANGUAGE_BLOB),
        ("docs/INTEGRITY.md", SAFE_INTEGRITY_BLOB),
        ("safe_asi/response_rime.py", SAFE_RESPONSE_BLOB),
    ])?;
    ensure(hash_bytes(&read(&follow.join("matrix/3-D-GITHUB-OF-THRUTH.md"))?) == THREE_D_SHA,
        "3-D GitHub entrypoint SHA changed")?;
    ensure(hash_bytes(&read(&follow.join("matrix/GITHUB-THREE-DIMENSIONALLY-RIMED-2026-07-29.hbp"))?) == GRAPH_SHA,
        "original 3-D GitHub graph SHA changed")?;
    Ok(())
}
fn parse<'a>(line: &'a str, label: &str) -> Result<BTreeMap<&'a str, &'a str>, String> {
    let parts: Vec<_> = line.split('|').collect();
    ensure(parts.len() >= 3 && parts[0] == label && parts.last() == Some(&"json=0"),
           "wrong tuple tag or zero-JSON sentinel")?;
    let mut fields = BTreeMap::new();
    for item in &parts[1..parts.len()-1] {
        let (key, value) = item.split_once('=').ok_or("invalid tuple field")?;
        ensure(!key.is_empty() && !value.is_empty() && fields.insert(key, value).is_none(),
               "empty or duplicated tuple field")?;
    }
    Ok(fields)
}
fn fld<'a>(row: &'a BTreeMap<&str, &str>, key: &str) -> Result<&'a str, String> {
    row.get(key).copied().ok_or_else(|| format!("missing tuple field {key}"))
}
fn one_line<'a>(data: &'a str, label: &str) -> Result<BTreeMap<&'a str, &'a str>, String> {
    ensure(data.lines().count() == 1, "receipt must contain exactly one data row")?;
    parse(data.trim_end_matches('\n'), label)
}
fn source_snapshot(run: &str, snapshot: &str) -> Result<BTreeMap<String, String>, String> {
    let fields = one_line(snapshot, "G3D_IS_REFS")?;
    ensure(fields.len() == 6, "snapshot field population differs")?;
    for field in ["schema", "run", "pnp_sha", "follow_main", "safe_main", "observed"] {
        fld(&fields, field)?;
    }
    ensure(fld(&fields, "schema")? == "1" && fld(&fields, "run")? == run,
           "snapshot run does not match current computation")?;
    ensure(fld(&fields, "observed")? == "GITHUB_LS_REMOTE",
           "snapshot must declare active GitHub branch remeasurement")?;
    for field in ["pnp_sha", "follow_main", "safe_main"] {
        ensure(hex40(fld(&fields, field)?), "snapshot Git ref invalid")?;
    }
    Ok(fields.iter().map(|(k,v)|(k.to_string(),v.to_string())).collect())
}

struct Evidence {
    run: String,
    ch_sha: String,
    word_sha: String,
    snap_sha: String,
    ch_forward: String,
    ch_reverse: String,
    word_forward: String,
    word_reverse: String,
    pnp_sha: String,
    follow_live: String,
    safe_live: String,
}

fn check_evidence(run: &str, snapshot: &str, chariot: &str, words: &str) -> Result<Evidence,String> {
    ensure(safe_run(run), "invalid run key")?;
    let seen = source_snapshot(run, snapshot)?;
    let ch = one_line(chariot, "CH20_FINAL")?;
    ensure(fld(&ch, "schema")? == "1" && fld(&ch, "run")? == run, "Chariot source/run mismatch")?;
    ensure(fld(&ch, "historical_seal")? == CHARIOT_SEAL, "Chariot historical seal mismatch")?;
    for (key, value) in [("forward_leaves","20"), ("reverse_leaves","20"),
                         ("checkpoints","40"), ("restored_cells","540"),
                         ("replay","EXACT"), ("status","PASS")] {
        ensure(fld(&ch, key)? == value, "Chariot final does not satisfy the prior owned gate")?;
    }
    let word = one_line(words, "CW_FINAL")?;
    ensure(fld(&word,"run")? == run && fld(&word,"status")? == "PASS",
           "original typed-word cascade final changed")?;
    for (key,value) in [("unique_execution","93312"), ("unique_glyph_addresses","93312"),
                        ("search_buckets","432"), ("collisions_per_bucket","216")] {
        ensure(fld(&word, key)? == value, "cascade population changed")?;
    }
    let ch_forward = fld(&ch,"forward_root")?.to_owned();
    let ch_reverse = fld(&ch,"reverse_root")?.to_owned();
    for v in [&ch_forward,&ch_reverse] {
        ensure(hex64(v), "Chariot root is malformed")?;
    }
    // Cascade root names are read as graph evidence, never invented.
    let word_forward = fld(&word,"forward_root")?.to_owned();
    let word_reverse = fld(&word,"reverse_root")?.to_owned();
    for v in [&word_forward, &word_reverse] {
        ensure(hex64(v), "word-cascade root is malformed")?;
    }
    Ok(Evidence {
        run: run.to_owned(),
        ch_sha: hash_bytes(chariot.as_bytes()),
        word_sha: hash_bytes(words.as_bytes()),
        snap_sha: hash_bytes(snapshot.as_bytes()),
        ch_forward, ch_reverse, word_forward, word_reverse,
        pnp_sha: seen["pnp_sha"].clone(),
        follow_live: seen["follow_main"].clone(),
        safe_live: seen["safe_main"].clone(),
    })
}
fn domain(label: &str, parts: &[&str]) -> String {
    let mut data = label.to_owned();
    for part in parts {
        data.push('|');
        data.push_str(part);
    }
    hash_bytes(data.as_bytes())
}
fn tuple(tag: &str, parts: &[(&str, &str)]) -> String {
    let mut row = tag.to_owned();
    for (key,value) in parts {
        row.push('|');
        row.push_str(key);
        row.push('=');
        row.push_str(value);
    }
    row.push_str("|json=0\n");
    row
}
fn receipt(e: &Evidence) -> String {
    let mut out = String::new();
    let hdr = tuple("G3D_RIME", &[
        ("schema","1"),("run",&e.run),("source","PINNED_FOLLOW_SAFE"),
        ("observed","EXACT_GITHUB_REFS"),("transport","GITHUB_ACTIONS_ARTIFACTS"),
        ("geometry","SIGNED_INTEGER_3D"),("system_affirmed","0"),
    ]);
    out.push_str(&hdr);
    out.push_str(&tuple("WAS",&[
        ("source","GITHUB_HISTORICAL_2026_07_30"),
        ("original_graph_sha",GRAPH_SHA),("historical_follow_main","a1fdf4cf105ab4e86e652a126f249a2407170777"),
        ("historical_safe_main",SAFE_COMMIT),("historical_universe_main","f43cc2a4c2fe92c08bb8443aa8adf6865a6b9bc4"),
    ]));
    out.push_str(&tuple("IS",&[
        ("snapshot_sha",&e.snap_sha),("pnp_sha",&e.pnp_sha),
        ("follow_main",&e.follow_live),("safe_main",&e.safe_live),
        ("scope","GITHUB_REFS_AT_CAPTURE"),
    ]));
    out.push_str(&tuple("WILL",&[
        ("goal","FRESH_REMEASURE_AND_NEXT_ACCEPTANCE"),
        ("future_commit","NOT_MEASURED"),("future_bytes","NOT_MEASURED"),
        ("scope","NO_FABRICATED_FUTURE"),
    ]));
    let mut identities = BTreeSet::new();
    let nodes = [
        ("repo_follow","REPOSITORY",FOLLOW_COMMIT),
        ("repo_safe","REPOSITORY",SAFE_COMMIT),
        ("repo_pnp","REPOSITORY",&e.pnp_sha),
        ("entrypoint","HARNESS",THREE_D_SHA),
        ("language","LANGUAGE",SAFE_LANGUAGE_BLOB),
        ("historical_graph","HISTORICAL_GRAPH",GRAPH_SHA),
        ("is_snapshot","CURRENT_REF_SNAPSHOT",&e.snap_sha),
        ("chariot_final","RUN_ARTIFACT",&e.ch_sha),
        ("word_final","RUN_ARTIFACT",&e.word_sha),
        ("shared_run","WORKFLOW_RUN",&e.run),
    ];
    for (i,(id,kind,address)) in nodes.iter().enumerate() {
        assert!(identities.insert(*id));
        let x = (i as i32 * 2_000 - 9_000).to_string();
        let y = ((i % 3) as i32 * 4_000 - 4_000).to_string();
        let z = ((i % 4) as i32 * 3_000 - 4_500).to_string();
        out.push_str(&tuple("NODE",&[
            ("id",id),("type",kind),("address",address),
            ("x",&x),("y",&y),("z",&z),("identity_exchange","0"),
        ]));
    }
    for (id,from,to,relation) in [
        ("a","repo_follow","entrypoint","PUBLISHES"),
        ("b","repo_follow","historical_graph","PUBLISHES"),
        ("c","repo_safe","language","PUBLISHES"),
        ("d","repo_pnp","shared_run","EXECUTES"),
        ("e","shared_run","chariot_final","PRODUCES"),
        ("f","shared_run","word_final","PRODUCES"),
        ("g","entrypoint","historical_graph","DESCRIBES"),
        ("h","shared_run","is_snapshot","OBSERVES"),
        ("i","language","is_snapshot","SCHEMA_REFERENCE"),
        ("j","chariot_final","word_final","RIMES_WITH"),
    ] {
        out.push_str(&tuple("EDGE",&[
            ("id",id),("from",from),("to",to),("relation",relation),("directed","1"),
        ]));
    }
    for (parent,body) in [("chariot_final",&e.ch_sha),("word_final",&e.word_sha)] {
        for family in FAMILIES {
            let leaf = domain("G3D_FAMILY", &[parent,family,body]);
            out.push_str(&tuple("FAMILY",&[
                ("parent",parent),("family",family),("leaf_sha",&leaf),
                ("source_sha",body),("signed_projection","VIEW_ONLY"),
            ]));
        }
        for detector in DETECTORS {
            for direction in DIRECTIONS {
                let label = match detector {
                    "BYTE_COMMITMENT" => "RECEIPT_SHA_PRESENT",
                    "CLAIM_EVIDENCE" => "UPSTREAM_OWNER_GATE_REQUIRED",
                    "MEDIA_BINDING" => "NOT_APPLICABLE",
                    _ => "NOT_MEASURED",
                };
                let address = domain("G3D_VIEW", &[parent,body,detector,direction]);
                out.push_str(&tuple("VIEW",&[
                    ("parent",parent),("detector",detector),("direction",direction),
                    ("view_sha",&address),("finding",label),("identity_exchange","0"),
                ]));
            }
        }
    }
    for (artifact,phase,root) in [
        ("chariot_final","FORWARD",&e.ch_forward),
        ("chariot_final","REVERSE",&e.ch_reverse),
        ("word_final","FORWARD",&e.word_forward),
        ("word_final","REVERSE",&e.word_reverse),
    ] {
        out.push_str(&tuple("ROOT",&[
            ("artifact",artifact),("phase",phase),("sha",root),
            ("source","UPSTREAM_NATIVE_VERIFIED"),("identity_exchange","0"),
        ]));
    }
    let rime = domain("RIME_SOURCE_IDENTITY_V1", &[&e.run,&e.ch_sha,&e.word_sha]);
    out.push_str(&tuple("RIME",&[
        ("left","chariot_final"),("right","word_final"),
        ("left_sha",&e.ch_sha),("right_sha",&e.word_sha),
        ("direction","CH20_TO_CW"),("seal",&rime),("unrime","EXACT_MEMBERS"),
    ]));
    let histories = [GRAPH_SHA,e.snap_sha.as_str(),e.ch_sha.as_str(),e.word_sha.as_str()];
    let mut forward = String::new();
    let mut backward = String::new();
    for (i,h) in histories.iter().enumerate() {
        out.push_str(&tuple("LEVEL",&[
            ("n",&i.to_string()),("phase",if i==0 {"WAS"} else {"IS"}),
            ("observed_sha",h),("window_max","60"),
        ]));
        forward.push_str(h);
        forward.push('\n');
    }
    for h in histories.iter().rev() {backward.push_str(h);backward.push('\n');}
    let froot = domain("RIME_OBSERVED_FORWARD_V1", &[&forward]);
    let broot = domain("RIME_OBSERVED_BACKWARD_V1", &[&backward]);
    let reduction = domain("RIME_OBSERVED_REDUCTION_V1", &[&froot,&broot]);
    out.push_str(&tuple("REFLECTION",&[
        ("observed_levels","4"),("max_window","60"),
        ("forward_root",&froot),("backward_root",&broot),
        ("self_reduction",&reduction),("future_generated","0"),
    ]));
    let hbi = domain("G3D_HBI", &[&e.run,&e.snap_sha,GRAPH_SHA]);
    let hbp = domain("G3D_HBP", &[&rime,&reduction]);
    let sha = domain("G3D_SHA", &[&e.ch_sha,&e.word_sha]);
    let sh = domain("G3D_SH", &[THREE_D_BLOB,SAFE_LANGUAGE_BLOB]);
    let hash = domain("G3D_HASH", &[&hbi,&hbp,&sha,&sh]);
    let distinct = [&hbi,&hbp,&sha,&sh,&hash].into_iter().collect::<BTreeSet<_>>();
    assert_eq!(distinct.len(),5);
    out.push_str(&tuple("CENTER",&[
        ("members",MEMBERS),("traversal",TRAVERSAL),
        ("hbi",&hbi),("hbp",&hbp),("sha",&sha),("sh",&sh),("hash",&hash),
        ("sha_equals_hash","0"),
    ]));
    out.push_str(&tuple("VERDICT",&[
        ("source","MEASURED_PASS"),("exact_receipts","MEASURED_PASS"),
        ("model_quality","NOT_MEASURED"),("live_fabric","NOT_MEASURED"),
        ("reason","SEPARATE_OWNING_GATES"),
    ]));
    let body_sha = hash_bytes(out.as_bytes());
    let rows = out.lines().count() + 1;
    out.push_str(&tuple("G3D_RIME_FTR", &[("body_sha",&body_sha),("rows",&rows.to_string())]));
    out
}
fn produce(run: &str, follow: &Path, safe: &Path, snapshot: &Path,
           chariot: &Path, words: &Path) -> Result<String, String> {
    source_check(follow,safe)?;
    let input = text_file(snapshot)?;
    let ch = text_file(chariot)?;
    let cw = text_file(words)?;
    let e = check_evidence(run,&input,&ch,&cw)?;
    Ok(receipt(&e))
}
fn getarg(args: &mut impl Iterator<Item=String>, name: &str) -> Result<String,String> {
    args.next().ok_or_else(|| format!("missing {name}"))
}
fn run() -> Result<(),String> {
    let mut args=env::args().skip(1);
    let command=getarg(&mut args,"command")?;
    match command.as_str() {
        "sources" => {
            let follow=getarg(&mut args,"FOLLOW checkout")?;
            let safe=getarg(&mut args,"SAFE checkout")?;
            ensure(args.next().is_none(),"unexpected source arguments")?;
            source_check(Path::new(&follow),Path::new(&safe))?;
            println!("G3D_SOURCES|follow={FOLLOW_COMMIT}|safe={SAFE_COMMIT}|old_graph={GRAPH_SHA}|status=PASS|json=0");
        }
        "emit" | "verify" => {
            let run=getarg(&mut args,"run")?;
            let follow=getarg(&mut args,"FOLLOW checkout")?;
            let safe=getarg(&mut args,"SAFE checkout")?;
            let snapshot=getarg(&mut args,"IS refs")?;
            let chariot=getarg(&mut args,"CH20 final")?;
            let words=getarg(&mut args,"CW final")?;
            let output=getarg(&mut args,"RIME HBP")?;
            ensure(args.next().is_none(),"unexpected extra command argument")?;
            let expected=produce(&run,Path::new(&follow),Path::new(&safe),
                Path::new(&snapshot),Path::new(&chariot),Path::new(&words))?;
            if command=="emit" {
                let mut file=OpenOptions::new().write(true).create_new(true).open(&output)
                    .map_err(|e|format!("cannot exclusively create provenance HBP: {e}"))?;
                file.write_all(expected.as_bytes()).map_err(|e|e.to_string())?;
                file.sync_all().map_err(|e|e.to_string())?;
            } else {
                ensure(text_file(Path::new(&output))? == expected,
                    "the provenance graph or prior evidence has been altered")?;
            }
            println!("G3D_RIME_{command}|run={run}|bytes={}|sha256={}|status=PASS|json=0",
                expected.len(),hash_bytes(expected.as_bytes()));
        }
        _ => return Err("unknown 3-D provenance command".to_owned()),
    }
    Ok(())
}
fn main(){
    if let Err(error)=run(){
        eprintln!("G3D_RIME_FAIL|reason={error}|json=0");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn good(run: &str) -> (String,String,String) {
        let snapshot=format!("G3D_IS_REFS|schema=1|run={run}|pnp_sha={FOLLOW_COMMIT}|follow_main={FOLLOW_COMMIT}|safe_main={SAFE_COMMIT}|observed=GITHUB_LS_REMOTE|json=0\n");
        let ch=format!("CH20_FINAL|schema=1|run={run}|historical_seal={CHARIOT_SEAL}|forward_root={THREE_D_SHA}|reverse_root={GRAPH_SHA}|forward_leaves=20|reverse_leaves=20|checkpoints=40|restored_cells=540|replay=EXACT|status=PASS|json=0\n");
        let cw=format!("CW_FINAL|schema=1|run={run}|forward_root={GRAPH_SHA}|reverse_root={THREE_D_SHA}|unique_execution=93312|unique_glyph_addresses=93312|search_buckets=432|collisions_per_bucket=216|status=PASS|json=0\n");
        (snapshot,ch,cw)
    }
    #[test]
    fn canonical_identity_graph_and_bounded_reflection() {
        let (snapshot,ch,cw)=good("pilot");
        let e=check_evidence("pilot",&snapshot,&ch,&cw).expect("inputs");
        let a=receipt(&e);
        assert_eq!(a.matches("\nNODE|").count(),10);
        assert_eq!(a.matches("\nEDGE|").count(),10);
        assert_eq!(a.matches("\nFAMILY|").count(),6);
        assert_eq!(a.matches("\nVIEW|").count(),24);
        assert_eq!(a.matches("\nROOT|").count(),4);
        assert_eq!(a.matches("\nLEVEL|").count(),4);
        assert!(a.contains("future_commit=NOT_MEASURED|future_bytes=NOT_MEASURED"));
        assert!(a.contains("live_fabric=NOT_MEASURED"));
        let b=receipt(&check_evidence("pilot",&snapshot,&ch,&cw).expect("replay"));
        assert_eq!(a,b);
    }
    #[test]
    fn wrong_source_tamper_and_cross_run_fail_closed() {
        let (snapshot,ch,cw)=good("pilot");
        assert!(check_evidence("wrong",&snapshot,&ch,&cw).is_err());
        assert!(check_evidence("pilot",&snapshot,&ch.replace("restored_cells=540","restored_cells=539"),&cw).is_err());
        assert!(check_evidence("pilot",&snapshot,&ch,&cw.replace("unique_execution=93312","unique_execution=0")).is_err());
        assert!(check_evidence("pilot",&snapshot.replace("observed=GITHUB_LS_REMOTE","observed=ASSUMED"),&ch,&cw).is_err());
        assert!(check_evidence("pilot",&snapshot.replace("follow_main=","follow_main=XX"),&ch,&cw).is_err());
        assert!(check_evidence("pilot",&snapshot,&ch.replace("status=PASS","status=FAIL"),&cw).is_err());
    }
    #[test]
    fn different_observed_refs_preserve_was_and_modify_is() {
        let (snapshot,ch,cw)=good("pilot");
        let e=check_evidence("pilot",&snapshot,&ch,&cw).expect("valid");
        let changed=snapshot.replace(FOLLOW_COMMIT,SAFE_COMMIT);
        let e2=check_evidence("pilot",&changed,&ch,&cw).expect("valid changed");
        let a=receipt(&e);
        let b=receipt(&e2);
        assert_ne!(a,b);
        assert!(a.contains("historical_follow_main=a1fdf4cf105ab4e86e652a126f249a2407170777"));
        assert!(b.contains(&format!("original_graph_sha={GRAPH_SHA}")));
    }
}
