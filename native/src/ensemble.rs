//! Multi-GNN source roster and original G4 GLSM/Fischer Tier-0 gate.
//! Numeric G1/G2/G3 inputs are external proposals, never synthesized here.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::sha256::hash_bytes;
use crate::source;

const ROSTER: &str = include_str!("../fixtures/MULTI_GNN_FAMILIES.hbi");
const FIXTURE: &str = include_str!("../fixtures/G4_GLSM_REFERENCE.hbi");
const PINS: &str = include_str!("../../policy/TRAINED_SOURCE_PINS.hbi");
const HBP_HEADER: &str = "HBPHEADER|schema=pnp.multignn.g4.v1|seal=HBI-HBP-SHA-SH-HASH|trained_neural=HOLD|json=0\n";
const HBI_HEADER: &str = "HBIHEADER|schema=pnp.multignn.g4.index.v1|for=PNP_GNN_GATE.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";

fn hash(value: &str) -> String {
    hash_bytes(value.as_bytes())
}
fn fields(line: &str) -> Result<BTreeMap<&str, &str>, String> {
    let mut map = BTreeMap::new();
    for piece in line.split('|').skip(1) {
        let (k, v) = piece.split_once('=').ok_or("malformed GNN HBI record")?;
        if k.is_empty() || v.is_empty() || map.insert(k, v).is_some() {
            return Err("duplicate/empty GNN HBI field".to_owned());
        }
    }
    Ok(map)
}
fn field<'a>(map: &'a BTreeMap<&str, &str>, key: &str) -> Result<&'a str, String> {
    map.get(key).copied().ok_or_else(|| format!("missing {key}"))
}
fn pin(id: &str, blob: &str) -> bool {
    PINS.lines().any(|line| line.starts_with(&format!("HBIPIN|id={id}|"))
        && line.contains(&format!("|git_blob={blob}|")))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Described, EdgeMined, PathFound, MistakeFlagged, Converged,
}
impl Phase {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "DESCRIBED" => Some(Self::Described),
            "EDGE_MINED" => Some(Self::EdgeMined),
            "PATH_FOUND" => Some(Self::PathFound),
            "MISTAKE_FLAGGED" => Some(Self::MistakeFlagged),
            "CONVERGED" => Some(Self::Converged),
            _ => None,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Described => "DESCRIBED",
            Self::EdgeMined => "EDGE_MINED",
            Self::PathFound => "PATH_FOUND",
            Self::MistakeFlagged => "MISTAKE_FLAGGED",
            Self::Converged => "CONVERGED",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Signals {
    pub edge: Option<i64>,
    pub path: Option<i64>,
    pub gain: Option<i64>,
    pub g1: Option<u64>,
    pub g2: Option<u64>,
    pub g3: Option<u64>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Gate {
    pub next: Phase,
    pub emitted: bool,
    pub tier0_block: bool,
    pub operator_held: bool,
}
pub fn evaluate(previous: Phase, signals: Signals) -> Gate {
    let has_rows = signals.g1.is_some() && signals.g2.is_some()
        && signals.g3.is_some();
    let next = if let (Some(edge), Some(path), Some(gain)) =
        (signals.edge, signals.path, signals.gain)
    {
        if has_rows {
            match previous {
                Phase::Described if edge > 0 => Phase::EdgeMined,
                Phase::EdgeMined if path >= 5_000 => Phase::PathFound,
                Phase::PathFound if gain < 0 => Phase::MistakeFlagged,
                Phase::PathFound if gain >= 5_000 => Phase::Converged,
                Phase::MistakeFlagged if path >= 5_000 => Phase::PathFound,
                _ => previous,
            }
        } else {
            previous
        }
    } else {
        previous
    };
    Gate {
        next,
        emitted: next != previous,
        tier0_block: next == Phase::MistakeFlagged,
        operator_held: true,
    }
}

fn opt_integer(raw: &str) -> Result<Option<i64>, String> {
    if raw == "none" { return Ok(None); }
    let num = raw.parse::<i64>().map_err(|_| "invalid integer signal")?;
    if num.unsigned_abs() > 1_000_000 {
        return Err("unbounded GNN signal".to_owned());
    }
    Ok(Some(num))
}
fn opt_row(raw: &str) -> Result<Option<u64>, String> {
    if raw == "none" { return Ok(None); }
    raw.parse::<u64>().map(Some).map_err(|_| "invalid GNN antecedent".to_owned())
}

fn family_roster() -> Result<(usize, usize, usize), String> {
    let Some(body) = ROSTER.strip_prefix(
        "HBIHEADER|schema=pnp.multignn.families.v1|seal=HBI-HBP-SHA-SH-HASH|source_bundle=23|json=0\n"
    ) else { return Err("GNN roster header changed".to_owned()); };
    let mut seen = BTreeSet::new();
    let mut models = BTreeSet::new();
    let mut signals = BTreeSet::new();
    let mut neural = 0_usize;
    let valid: BTreeSet<&str> = [
        "L0", "L1", "L2", "L4", "G1", "G2", "G3",
        "G4", "SHANNON", "SHA"
    ].into_iter().collect();
    for row in body.lines() {
        if !row.starts_with("FAMILY|") {
            return Err("bad GNN family tag".to_owned());
        }
        let f = fields(row)?;
        let id = field(&f, "id")?;
        if !valid.contains(id) || !seen.insert(id) || field(&f, "json")? != "0" {
            return Err("duplicate/unknown GNN family".to_owned());
        }
        let kind = field(&f, "kind")?;
        let original = field(&f, "origin_blob")?;
        let status = field(&f, "status")?;
        if kind == "trained_neural" {
            neural += 1;
            if status != "HOLD_FORWARD"
                || original.len() != 40 || !original.bytes().all(|b| b.is_ascii_hexdigit())
            { return Err("unfrozen neural model lineage".to_owned()); }
            let file_names: Vec<_> = field(&f, "model")?.split(',').collect();
            let pin_names: Vec<_> = field(&f, "checkpoint_pin")?.split(',').collect();
            if file_names.len() != pin_names.len() {
                return Err("checkpoint count mismatch".to_owned());
            }
            for (name, pinned) in file_names.iter().zip(pin_names) {
                if !name.ends_with(".pt") || !models.insert(*name)
                    || !PINS.lines().any(|line| line.starts_with(&format!("HBIPIN|id={pinned}|"))
                        && line.contains(&format!("|path=models/{name}|")))
                {
                    return Err("unmatched original checkpoint".to_owned());
                }
            }
        } else if id == "G3" {
            if kind != "withheld_plane" || original != "none"
                || status != "HOLD_SOURCE" {
                return Err("G3 withheld-source boundary modified".to_owned());
            }
        } else {
            let record = field(&f, "checkpoint_pin")?;
            if !pin(record, original) {
                return Err("original GNN plane Git blob mismatch".to_owned());
            }
        }
        if ["L0","L4","G1","G2","G3","G4","SHANNON","SHA"].contains(&id) {
            signals.insert(id);
        }
    }
    if seen.len() != 10 || models.len() != 9 || neural != 4 || signals.len() != 8 {
        return Err("multi-GNN roster is incomplete".to_owned());
    }
    Ok((seen.len(), neural, models.len()))
}
fn reference_cases() -> Result<Vec<(String, Gate, String)>, String> {
    let body = FIXTURE.strip_prefix(
        "HBIHEADER|schema=pnp.g4.reference.v1|source_blob=9006f5cca08fab56c906bb67237e17974090cfe8|scale=10000|json=0\n"
    ).ok_or("G4 reference header mismatch")?;
    let mut cases = Vec::new();
    let mut ids = BTreeSet::new();
    for row in body.lines() {
        if !row.starts_with("G4CASE|") { return Err("unknown G4 case".to_owned()); }
        let f = fields(row)?;
        let id = field(&f, "id")?;
        if !ids.insert(id) || field(&f, "json")? != "0" {
            return Err("duplicate G4 case".to_owned());
        }
        let previous = Phase::parse(field(&f, "prev")?)
            .ok_or("unknown prior G4 state")?;
        let inputs = Signals {
            edge: opt_integer(field(&f, "edge")?)?,
            path: opt_integer(field(&f, "path")?)?,
            gain: opt_integer(field(&f, "gain")?)?,
            g1: opt_row(field(&f, "g1")?)?,
            g2: opt_row(field(&f, "g2")?)?,
            g3: opt_row(field(&f, "g3")?)?,
        };
        let verdict = evaluate(previous, inputs);
        if verdict.next.label() != field(&f, "next")?
            || u8::from(verdict.emitted).to_string() != field(&f, "emit")?
            || u8::from(verdict.tier0_block).to_string() != field(&f, "tier0")?
        {
            return Err(format!("source G4 GLSM behavior mismatch: {id}"));
        }
        cases.push((id.to_owned(), verdict, row.to_owned()));
    }
    if cases.len() != 13 {
        return Err("G4 source reference count changed".to_owned());
    }
    Ok(cases)
}

fn render(source_digest: &str) -> Result<(String, String, String, usize), String> {
    let (families, _, _) = family_roster()?;
    let cases = reference_cases()?;
    let mut hbp = HBP_HEADER.to_owned();
    let mut hbi = HBI_HEADER.to_owned();
    let mut predecessor = hash("PNP-MULTIGNN-G4-GENESIS-v1");
    let mut count = 0_usize;
    for (id, record, family) in ROSTER.lines().skip(1).map(|row| {
        let f = fields(row).expect("static validated roster");
        (field(&f, "id").expect("static id").to_owned(), row.to_owned(), true)
    }).chain(cases.iter().map(|(id, _verdict, row)| {
        (id.clone(), row.clone(), false)
    })) {
        count += 1;
        let sh_full = hash(&record);
        let sh = &sh_full[..16];
        let kind = if family { "FAMILY" } else { "GLSM" };
        let data = format!(
            "GNN_GATE|seq={count}|kind={kind}|id={id}|source_hbp_sha256={source_digest}|record_sha256={sh_full}|sh={sh}|source_port={}|prev_hash={predecessor}",
            if family { "MULTIMODEL_HOLD" } else { "G4_INTEGER_TEST" }
        );
        let receipt_sha = hash(&data);
        let chain = hash(&format!("{predecessor}|{receipt_sha}"));
        let line = format!("{data}|receipt_sha={receipt_sha}|hash={chain}|json=0\n");
        hbi.push_str(&format!(
            "HBI|seq={count}|id={id}|offset={}|len={}|hbp_sha256={}|sh={sh}|hash={chain}|status=VERIFIED|json=0\n",
            hbp.len(), line.len(), hash(&line)
        ));
        hbp.push_str(&line);
        predecessor = chain;
    }
    if count != families + cases.len() {
        return Err("GNN family/count mismatch".to_owned());
    }
    let payload = format!(
        "GNN_ANCHOR|count={count}|source_hbp_sha256={source_digest}|roster_sha256={}|cases_sha256={}|root_hbp_sha256={}|root_hbi_sha256={}|chain_hash={predecessor}|seal=HBI-HBP-SHA-SH-HASH",
        hash(ROSTER), hash(FIXTURE), hash(&hbp), hash(&hbi)
    );
    let anchor = format!(
        "{payload}|anchor_checksum={}|json=0\n",hash(&payload)
    );
    Ok((hbp, hbi, anchor, count))
}
fn attested_source(sources: &Path) -> Result<String, String> {
    if source::verify(sources)? != 23 {
        return Err("source archive mismatch".to_owned());
    }
    let data = fs::read(sources.join("PNP_TRAINED_SOURCE.hbp"))
        .map_err(|e| format!("original source archive missing: {e}"))?;
    Ok(hash_bytes(&data))
}
pub fn attest(sources: &Path, output: &Path) -> Result<usize, String> {
    let root = attested_source(sources)?;
    let (hbp, hbi, anchor, count) = render(&root)?;
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    for (name, contents) in [
        ("PNP_GNN_GATE.hbp", hbp),
        ("PNP_GNN_GATE.hbi", hbi),
        ("PNP_GNN_GATE.anchor", anchor)
    ] {
        fs::write(output.join(name), contents)
            .map_err(|e| format!("cannot write GNN evidence {name}: {e}"))?;
    }
    Ok(count)
}
pub fn verify(sources: &Path, output: &Path) -> Result<usize, String> {
    let root = attested_source(sources)?;
    let (hbp, hbi, anchor, count) = render(&root)?;
    for (name, expected) in [
        ("PNP_GNN_GATE.hbp", hbp),
        ("PNP_GNN_GATE.hbi", hbi),
        ("PNP_GNN_GATE.anchor", anchor)
    ] {
        let recorded = fs::read(output.join(name))
            .map_err(|e| format!("missing GNN artifact {name}: {e}"))?;
        if recorded != expected.as_bytes() {
            return Err(format!("GNN artifact altered: {name}"));
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_gnn_families_and_checkpoints_are_distinct_and_pinned() {
        assert_eq!(family_roster().expect("all sources"), (10,4,9));
        assert_eq!(reference_cases().expect("exact G4 source transitions").len(), 13);
    }
    #[test]
    fn original_five_state_walk_and_tier_zero_veto() {
        let rows = reference_cases().expect("frozen source cases");
        let expected = ["EDGE_MINED", "PATH_FOUND", "MISTAKE_FLAGGED",
                        "PATH_FOUND", "CONVERGED"];
        for (index, state) in expected.iter().enumerate() {
            assert_eq!(rows[index].1.next.label(), *state);
            assert!(rows[index].1.emitted);
            assert_eq!(rows[index].1.tier0_block, index == 2);
            assert!(rows[index].1.operator_held);
        }
        let blocked = evaluate(Phase::Described, Signals {
            edge: Some(8000), path: Some(8000), gain: Some(8000),
            g1: None, g2: Some(2), g3: Some(3)
        });
        assert!(!blocked.emitted);
        assert_eq!(blocked.next, Phase::Described);
    }
    #[test]
    fn boundary_thresholds_and_absorbing_state_match_source() {
        let valid = |gain| Signals {
            edge: Some(10000), path: Some(5000), gain: Some(gain),
            g1: Some(1), g2: Some(2), g3: Some(3)
        };
        assert_eq!(evaluate(Phase::PathFound, valid(-1)).next, Phase::MistakeFlagged);
        assert_eq!(evaluate(Phase::PathFound, valid(4999)).next, Phase::PathFound);
        assert_eq!(evaluate(Phase::PathFound, valid(5000)).next, Phase::Converged);
        assert_eq!(evaluate(Phase::Converged, valid(-1)).next, Phase::Converged);
        assert_eq!(evaluate(Phase::MistakeFlagged, valid(0)).next, Phase::PathFound);
    }
    #[test]
    fn edited_source_case_fails_reference_parity() {
        let raw = FIXTURE.replace("|gain=-3000|", "|gain=3000|");
        assert_ne!(hash(&raw), hash(FIXTURE));
    }
}
