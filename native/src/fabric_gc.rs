//! Source-anchored neural-fabric, GAC tuple and GC proof plane.
//! Read-only: source evidence, collection planning, no workers or mutation.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::sha256::hash_bytes;

const SOURCES: &str = include_str!("../../policy/GAC_GC_SOURCE_PINS.hbi");
const HBP_HEADER: &str = "HBPHEADER|schema=pnp.fabric.gc.v1|seal=HBI-HBP-SHA-SH-HASH|mode=READ_ONLY|json=0\n";
const HBI_HEADER: &str = "HBIHEADER|schema=pnp.fabric.gc.index.v1|for=PNP_FABRIC_GC.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const MB: u64 = 1024 * 1024;
pub const ROTATE_BYTES: u64 = 50 * MB;
pub const ROTATE_LINES: u64 = 100_000;
pub const SANITY_BYTES: u64 = 500 * MB;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tier { Public, Restricted, Stealth, Hidden, Shadow, Secret }
impl Tier {
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Public => 0, Self::Restricted => 1, Self::Stealth => 2,
            Self::Hidden => 3, Self::Shadow => 4, Self::Secret => 5,
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "PUBLIC" => Some(Self::Public), "RESTRICTED" => Some(Self::Restricted),
            "STEALTH" => Some(Self::Stealth), "HIDDEN" => Some(Self::Hidden),
            "SHADOW" => Some(Self::Shadow), "SECRET" => Some(Self::Secret),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scope { Drive, Usb, Device, Folder, File, Pid }
impl Scope {
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Drive => 0, Self::Usb => 1, Self::Device => 2,
            Self::Folder => 3, Self::File => 4, Self::Pid => 5,
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "drive" => Some(Self::Drive), "usb" => Some(Self::Usb),
            "device" => Some(Self::Device), "folder" => Some(Self::Folder),
            "file" => Some(Self::File), "pid" => Some(Self::Pid),
            _ => None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FabricTuple {
    /// Opaque exact coordinates. Axis naming remains the original fabric's authority.
    pub coordinates: [i64; 60],
    pub tier: Tier,
    pub scope: Scope,
    pub source_digest: String,
}
impl FabricTuple {
    pub fn access_cell(&self) -> u8 {
        self.tier.ordinal() * 6 + self.scope.ordinal()
    }
    pub fn encode(&self) -> Result<String, &'static str> {
        if self.source_digest.len() != 64
            || !self.source_digest.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err("missing 60D tuple source commitment");
        }
        let coordinates = self.coordinates.iter()
            .map(i64::to_string).collect::<Vec<_>>().join(",");
        Ok(format!(
            "FABRIC60|coordinates={coordinates}|cell={}|source_sha256={}|status=FROZEN|json=0",
            self.access_cell(), self.source_digest
        ))
    }
    pub fn parse_coordinates(raw: &str) -> Option<[i64; 60]> {
        let numbers = raw.split(',').map(str::parse::<i64>)
            .collect::<Result<Vec<_>, _>>().ok()?;
        numbers.try_into().ok()
    }
}
pub fn access_coordinate(tier: Tier, scope: Scope) -> u8 {
    tier.ordinal() * 6 + scope.ordinal()
}
pub fn invert_access_cell(cell: u8) -> Option<(Tier, Scope)> {
    if cell >= 36 { return None; }
    let tier = [
        Tier::Public, Tier::Restricted, Tier::Stealth,
        Tier::Hidden, Tier::Shadow, Tier::Secret
    ][usize::from(cell / 6)];
    let scope = [
        Scope::Drive, Scope::Usb, Scope::Device,
        Scope::Folder, Scope::File, Scope::Pid
    ][usize::from(cell % 6)];
    Some((tier, scope))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcDecision {
    Absent, Retain, ArchiveByBytes, ArchiveByLines, HeldSanity,
}
impl GcDecision {
    pub fn label(self) -> &'static str {
        match self {
            Self::Absent => "ABSENT", Self::Retain => "RETAIN",
            Self::ArchiveByBytes => "ARCHIVE_CANDIDATE_BYTES",
            Self::ArchiveByLines => "ARCHIVE_CANDIDATE_LINES",
            Self::HeldSanity => "HOLD_SANITY",
        }
    }
    pub fn may_delete(self) -> bool { false }
    pub fn may_execute(self) -> bool { false }
}
/// Original Omni GC precedence: oversized sanity hold first; when bytes
/// exceed 50 MiB, do not scan lines; otherwise 100k lines triggers rotation.
pub fn decide_gc(bytes: Option<u64>, counted_lines: Option<u64>) -> GcDecision {
    let Some(bytes) = bytes else { return GcDecision::Absent; };
    if bytes > SANITY_BYTES { return GcDecision::HeldSanity; }
    if bytes >= ROTATE_BYTES { return GcDecision::ArchiveByBytes; }
    if counted_lines.is_some_and(|lines| lines >= ROTATE_LINES) {
        GcDecision::ArchiveByLines
    } else {
        GcDecision::Retain
    }
}

struct Pin<'a> {
    id: &'a str,
    repo: &'a str,
    commit: &'a str,
    path: &'a str,
    blob: &'a str,
}
fn fields(line: &str) -> Result<BTreeMap<&str, &str>, String> {
    let mut out = BTreeMap::new();
    for part in line.split('|').skip(1) {
        let (key, value) = part.split_once('=').ok_or("malformed GAC HBI record")?;
        if key.is_empty() || value.is_empty() || out.insert(key, value).is_some() {
            return Err("duplicate GAC HBI field".to_owned());
        }
    }
    Ok(out)
}
fn field<'a>(map: &BTreeMap<&'a str, &'a str>, key: &str) -> Result<&'a str, String> {
    map.get(key).copied().ok_or_else(|| format!("missing source field {key}"))
}
fn pins() -> Result<Vec<Pin<'static>>, String> {
    let body = SOURCES.strip_prefix(
        "HBIHEADER|schema=pnp.gc.gac.source.v1|seal=HBI-HBP-SHA-SH-HASH|json=0\n"
    ).ok_or("fabric source HBI header mismatch")?;
    let mut seen = BTreeSet::new();
    let mut pins = Vec::new();
    for line in body.lines() {
        if !line.starts_with("GC_PIN|") { return Err("unrecognized fabric source record".to_owned()); }
        let f = fields(line)?;
        let pin = Pin {
            id: field(&f, "id")?,
            repo: field(&f, "repo")?,
            commit: field(&f, "commit")?,
            path: field(&f, "path")?,
            blob: field(&f, "git_blob")?,
        };
        if !matches!(pin.repo, "gac" | "omni" | "neural")
            || !seen.insert(pin.id)
            || ![pin.commit, pin.blob].iter().all(|value| {
                value.len() == 40
                    && value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            })
            || pin.path.starts_with('/')
            || pin.path.split('/').any(|part| part.is_empty() || part == "..")
            || !pin.path.bytes().all(|b| b.is_ascii_alphanumeric()
                || matches!(b, b'/' | b'.' | b'_' | b'-'))
            || field(&f, "json")? != "0"
        {
            return Err("invalid GAC/NN/GC source identity".to_owned());
        }
        pins.push(pin);
    }
    if pins.len() != 10
        || pins.iter().filter(|p| p.repo == "gac").count() != 3
        || pins.iter().filter(|p| p.repo == "omni").count() != 2
        || pins.iter().filter(|p| p.repo == "neural").count() != 5
    {
        return Err("GAC/NN/GC source family incomplete".to_owned());
    }
    Ok(pins)
}
fn git(root: &Path, parts: &[&str]) -> Result<String, String> {
    let out = Command::new("git").arg("-C").arg(root).args(parts)
        .output().map_err(|e| format!("source Git unavailable: {e}"))?;
    if !out.status.success() {
        return Err(format!("original fabric source check failed: {}", parts.join(" ")));
    }
    String::from_utf8(out.stdout)
        .map(|s| s.trim().to_owned()).map_err(|_| "Git output was not UTF8".to_owned())
}
type FabricBundle = (String, String, String);
fn make_bundle(gac: &Path, neural: &Path, omni: &Path) -> Result<FabricBundle, String> {
    let sources = pins()?;
    let mut hbp = HBP_HEADER.to_owned();
    let mut hbi = HBI_HEADER.to_owned();
    let mut prev = hash_bytes(b"PNP-GAC-NEURAL-FABRIC-GC-GENESIS-v1");
    for (index, pin) in sources.iter().enumerate() {
        let root = match pin.repo {
            "gac" => gac, "neural" => neural, "omni" => omni,
            _ => return Err("unrecognized fabric source".to_owned()),
        };
        if git(root, &["rev-parse", "HEAD"])? != pin.commit
            || git(root, &["hash-object", pin.path])? != pin.blob
        {
            return Err(format!("fabric source Git pin changed: {}", pin.id));
        }
        let bytes = fs::read(root.join(pin.path))
            .map_err(|e| format!("missing source {}: {e}", pin.id))?;
        let digest = hash_bytes(&bytes);
        let sh = &digest[..16];
        let data = format!(
            "FABRIC_SOURCE|seq={}|id={}|repo={}|commit={}|path={}|git_blob={}|sha256={}|sh={}|tuple_dim=60|access_cells=36|gc=ARCHIVE_ONLY|authority=HELD|prev_hash={prev}",
            index + 1, pin.id, pin.repo, pin.commit, pin.path, pin.blob, digest, sh
        );
        let receipt_sha = hash_bytes(data.as_bytes());
        let chain = hash_bytes(format!("{prev}|{receipt_sha}").as_bytes());
        let row = format!("{data}|receipt_sha={receipt_sha}|hash={chain}|json=0\n");
        hbi.push_str(&format!(
            "HBI|seq={}|id={}|offset={}|len={}|hbp_sha256={}|sh={sh}|hash={chain}|status=VERIFIED|json=0\n",
            index + 1, pin.id, hbp.len(), row.len(), hash_bytes(row.as_bytes())
        ));
        hbp.push_str(&row);
        prev = chain;
    }
    // Include evaluated policy boundary cases in the same immutable evidence.
    for (n, bytes, lines) in [
        (1_u8, None, None),
        (2, Some(ROTATE_BYTES - 1), Some(ROTATE_LINES - 1)),
        (3, Some(ROTATE_BYTES), None),
        (4, Some(ROTATE_BYTES - 1), Some(ROTATE_LINES)),
        (5, Some(SANITY_BYTES + 1), Some(ROTATE_LINES)),
        (6, Some(SANITY_BYTES), None),
    ] {
        let decision = decide_gc(bytes, lines);
        let data = format!(
            "GC_POLICY|seq={}|case={n}|bytes={}|lines={}|decision={}|delete=0|execute=0|operator_approval=HELD|prev_hash={prev}",
            sources.len() + usize::from(n),
            bytes.map_or_else(|| "none".to_owned(), |v| v.to_string()),
            lines.map_or_else(|| "none".to_owned(), |v| v.to_string()),
            decision.label()
        );
        let receipt_sha = hash_bytes(data.as_bytes());
        let chain = hash_bytes(format!("{prev}|{receipt_sha}").as_bytes());
        let row = format!("{data}|receipt_sha={receipt_sha}|hash={chain}|json=0\n");
        let sh = &receipt_sha[..16];
        hbi.push_str(&format!(
            "HBI|seq={}|id=policy{n}|offset={}|len={}|hbp_sha256={}|sh={sh}|hash={chain}|status=VERIFIED|json=0\n",
            sources.len() + usize::from(n), hbp.len(), row.len(), hash_bytes(row.as_bytes())
        ));
        hbp.push_str(&row);
        prev = chain;
    }
    let intro = format!(
        "FABRIC_GC_ANCHOR|source_count={}|policy_count=6|tuple_dim=60|access_cells=36|source_pins_sha256={}|root_hbp_sha256={}|root_hbi_sha256={}|chain_hash={prev}|seal=HBI-HBP-SHA-SH-HASH|authority=HELD",
        sources.len(), hash_bytes(SOURCES.as_bytes()),
        hash_bytes(hbp.as_bytes()), hash_bytes(hbi.as_bytes())
    );
    let anchor = format!(
        "{intro}|anchor_checksum={}|json=0\n", hash_bytes(intro.as_bytes())
    );
    Ok((hbp, hbi, anchor))
}
pub fn attest(gac: &Path, neural: &Path, omni: &Path, out: &Path) -> Result<usize, String> {
    let (hbp, hbi, anchor) = make_bundle(gac, neural, omni)?;
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    for (path, bytes) in [
        ("PNP_FABRIC_GC.hbp", hbp),
        ("PNP_FABRIC_GC.hbi", hbi),
        ("PNP_FABRIC_GC.anchor", anchor),
    ] {
        fs::write(out.join(path), bytes)
            .map_err(|e| format!("could not write fabric GC receipt {path}: {e}"))?;
    }
    Ok(16)
}
pub fn verify(gac: &Path, neural: &Path, omni: &Path, out: &Path) -> Result<usize, String> {
    let (hbp, hbi, anchor) = make_bundle(gac, neural, omni)?;
    for (path, expected) in [
        ("PNP_FABRIC_GC.hbp", hbp),
        ("PNP_FABRIC_GC.hbi", hbi),
        ("PNP_FABRIC_GC.anchor", anchor),
    ] {
        let observed = fs::read(out.join(path))
            .map_err(|e| format!("missing original fabric archive {path}: {e}"))?;
        if observed != expected.as_bytes() {
            return Err(format!("GAC/NN/GC evidence changed: {path}"));
        }
    }
    Ok(16)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_60d_tuple_coordinates_and_36_access_cells_round_trip() {
        let mut seen = BTreeSet::new();
        for tier in [
            Tier::Public, Tier::Restricted, Tier::Stealth,
            Tier::Hidden, Tier::Shadow, Tier::Secret,
        ] {
            for scope in [
                Scope::Drive, Scope::Usb, Scope::Device,
                Scope::Folder, Scope::File, Scope::Pid
            ] {
                let cell = access_coordinate(tier, scope);
                assert!(seen.insert(cell));
                assert_eq!(invert_access_cell(cell), Some((tier, scope)));
            }
        }
        assert_eq!(seen.len(), 36);
        assert_eq!(invert_access_cell(36), None);
        let mut coordinates = [0_i64; 60];
        coordinates[0] = -1;
        coordinates[59] = i64::MAX;
        let record = FabricTuple {
            coordinates, tier: Tier::Secret, scope: Scope::Pid,
            source_digest: hash_bytes(b"source-tuple"),
        };
        assert_eq!(record.access_cell(), 35);
        assert!(record.encode().expect("tuple").contains("|cell=35|"));
        let joined = record.coordinates.iter()
            .map(i64::to_string).collect::<Vec<_>>().join(",");
        assert_eq!(FabricTuple::parse_coordinates(&joined), Some(coordinates));
        assert!(FabricTuple::parse_coordinates("0,1").is_none());
        assert!(FabricTuple::parse_coordinates(&format!("{joined},0")).is_none());
    }
    #[test]
    fn gc_thresholds_match_original_and_never_delete_or_launch() {
        assert_eq!(decide_gc(None, None), GcDecision::Absent);
        assert_eq!(decide_gc(Some(ROTATE_BYTES - 1), Some(ROTATE_LINES - 1)), GcDecision::Retain);
        assert_eq!(decide_gc(Some(ROTATE_BYTES), None), GcDecision::ArchiveByBytes);
        assert_eq!(decide_gc(Some(ROTATE_BYTES - 1), Some(ROTATE_LINES)), GcDecision::ArchiveByLines);
        assert_eq!(decide_gc(Some(SANITY_BYTES), None), GcDecision::ArchiveByBytes);
        assert_eq!(decide_gc(Some(SANITY_BYTES + 1), Some(ROTATE_LINES)), GcDecision::HeldSanity);
        for bytes in [None, Some(0), Some(ROTATE_BYTES), Some(SANITY_BYTES + 1)] {
            let decision = decide_gc(bytes, Some(ROTATE_LINES));
            assert!(!decision.may_delete());
            assert!(!decision.may_execute());
        }
    }
    #[test]
    fn all_three_source_families_pinned_and_role_distinct() {
        let got = pins().expect("pinned GAC/NN/GC families");
        assert_eq!(got.len(), 10);
        assert_eq!(got.iter().filter(|p| p.repo == "gac").count(), 3);
        assert_eq!(got.iter().filter(|p| p.repo == "neural").count(), 5);
        assert_eq!(got.iter().filter(|p| p.repo == "omni").count(), 2);
    }
}
