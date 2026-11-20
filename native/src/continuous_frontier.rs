//! Exact iterative ternary DPLL with a source-bound, resumable search frontier.
//!
//! A checkpoint preserves the entire live assignment trail, branch polarity,
//! backtracking frames, deterministic progress counters and a SHA-256 chain.
//! Restart resumes at the next unvisited DPLL node. An external compute
//! session may stop after an arbitrary number of nodes; the logical search
//! itself has no time-based expiry, recursion limit or maximum total budget.
//!
//! SAT/UNSAT proof still requires exact original-CNF validation and completed
//! exhaustive exploration. A checkpoint signature authenticates the record;
//! it is not by itself a proof of either Boolean outcome.
#![forbid(unsafe_code)]

use crate::guidance::Guidance;
use crate::sha256::hash_bytes;
use crate::shadow_selector::{ShadowMode, ShadowSelector};
use crate::ternary::{Cnf, Trit};

const MAGIC: &[u8; 8] = b"PNPFRT01";
const VERSION: u32 = 1;
const DIGEST_SIZE: usize = 64;
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontierMode {
    Direct,
    One,
    Three,
}
impl FrontierMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::One => "one",
            Self::Three => "three",
        }
    }
    fn symbol(self) -> u8 {
        match self { Self::Direct => 0, Self::One => 1, Self::Three => 2 }
    }
    fn from_symbol(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Direct),
            1 => Ok(Self::One),
            2 => Ok(Self::Three),
            _ => Err("unsupported checkpoint Fischer-shadow mode".to_owned()),
        }
    }
    fn shadow(self) -> Option<ShadowMode> {
        match self {
            Self::Direct => None,
            Self::One => Some(ShadowMode::One),
            Self::Three => Some(ShadowMode::Three),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontierState { Running, Sat, Unsat }
impl FrontierState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "RUNNING",
            Self::Sat => "SAT",
            Self::Unsat => "UNSAT",
        }
    }
    fn symbol(self) -> u8 {
        match self {
            Self::Running => 0,
            Self::Sat => 1,
            Self::Unsat => 2,
        }
    }
    fn from_symbol(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Running),
            1 => Ok(Self::Sat),
            2 => Ok(Self::Unsat),
            _ => Err("invalid frozen DPLL status".to_owned()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ChoiceFrame {
    /// Trail length just before the frame's Boolean decision.
    mark: usize,
    variable: usize,
    minus_branch: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frontier {
    pub mode: FrontierMode,
    pub state: FrontierState,
    pub source_sha256: String,
    pub guidance_sha256: String,
    pub seed_sha256: String,
    pub trace_sha256: String,
    pub nodes: u64,
    pub propagations: u64,
    pub literal_checks: u64,
    pub decisions: u64,
    pub first_order: u64,
    pub first_selected: u64,
    pub projected_rooms: u64,
    pub projected_bytes: u64,
    pub pixel_reads: u64,
    pub reconstructed_rooms: u64,
    pub source_trit_reads: u64,
    pub source_index_bytes: u64,
    variables: usize,
    clauses: usize,
    assignment: Vec<Trit>,
    trail: Vec<(usize, Trit)>,
    stack: Vec<ChoiceFrame>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Quantum {
    pub visited: u64,
    pub status: FrontierState,
    pub new_decisions: u64,
    pub projected_rooms: u64,
    pub projected_bytes: u64,
    pub pixel_reads: u64,
    pub reconstructed_rooms: u64,
    pub source_trit_reads: u64,
    pub source_index_bytes: u64,
    pub first_order: u64,
    pub first_selected: u64,
}

fn need(ok: bool, msg: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(msg.to_owned()) }
}
fn add(counter: &mut u64, value: u64) {
    *counter = counter.checked_add(value).expect("DPLL frontier meter overflow");
}
fn trit_byte(value: Trit) -> u8 { value as i8 as u8 }
fn byte_trit(value: u8) -> Result<Trit> {
    match value {
        255 => Ok(Trit::Minus),
        0 => Ok(Trit::Zero),
        1 => Ok(Trit::Plus),
        _ => Err("frontier contains invalid signed trit".to_owned()),
    }
}
fn root_sha(values: &[Trit]) -> String {
    hash_bytes(&values.iter().map(|&v| trit_byte(v)).collect::<Vec<_>>())
}
fn order_sha(order: &[usize]) -> Result<String> {
    let mut bytes = Vec::with_capacity(order.len() * 8);
    for &v in order {
        let value = u64::try_from(v).map_err(|_| "Fischer order index too wide")?;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(hash_bytes(&bytes))
}
fn good_sha(value: &str) -> bool {
    value.len() == DIGEST_SIZE && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn write_u32(data: &mut Vec<u8>, number: usize) -> Result<()> {
    let n = u32::try_from(number).map_err(|_| "frontier index exceeds u32")?;
    data.extend_from_slice(&n.to_le_bytes());
    Ok(())
}
fn write_u64(data: &mut Vec<u8>, value: u64) {
    data.extend_from_slice(&value.to_le_bytes());
}
struct Reader<'a> { bytes: &'a [u8], offset: usize }
impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self { Self { bytes, offset: 0 } }
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(count).ok_or("frontier length overflow")?;
        if end > self.bytes.len() {
            return Err("truncated checkpoint frontier data".to_owned());
        }
        let result = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(result)
    }
    fn u8(&mut self) -> Result<u8> { Ok(self.take(1)?[0]) }
    fn u32(&mut self) -> Result<u32> {
        let a = self.take(4)?;
        Ok(u32::from_le_bytes([a[0],a[1],a[2],a[3]]))
    }
    fn u64(&mut self) -> Result<u64> {
        let a: [u8;8] = self.take(8)?.try_into()
            .map_err(|_| "invalid checkpoint 64-bit field")?;
        Ok(u64::from_le_bytes(a))
    }
    fn sha(&mut self) -> Result<String> {
        let raw = self.take(DIGEST_SIZE)?;
        let value = std::str::from_utf8(raw).map_err(|_| "invalid checkpoint SHA256 ASCII")?;
        need(good_sha(value), "invalid checkpoint SHA256 width")?;
        Ok(value.to_owned())
    }
}

impl Frontier {
    pub fn new(
        cnf: &Cnf, guide: &Guidance, seed: &[Trit],
        source_sha256: &str, mode: FrontierMode,
    ) -> Result<Self> {
        need(good_sha(source_sha256), "frontier original source SHA256 malformed")?;
        need(seed.len() == cnf.variables, "GGUF seed and CNF dimensions differ")?;
        need(guide.order.len() == cnf.variables, "incomplete Fischer guidance order")?;
        let guidance_sha256 = order_sha(&guide.order)?;
        let seed_sha256 = root_sha(seed);
        let trace_sha256 = hash_bytes(format!(
            "PNP_DPLL_FRONTIER_BEGIN_V1|{source_sha256}|{guidance_sha256}|{seed_sha256}|{}",
            mode.label()
        ).as_bytes());
        Ok(Self {
            mode, state: FrontierState::Running,
            source_sha256: source_sha256.to_owned(),
            guidance_sha256, seed_sha256, trace_sha256,
            nodes: 0, propagations: 0, literal_checks: 0, decisions: 0,
            first_order: 0, first_selected: 0,
            projected_rooms: 0, projected_bytes: 0, pixel_reads: 0,
            reconstructed_rooms: 0, source_trit_reads: 0, source_index_bytes: 0,
            variables: cnf.variables, clauses: cnf.clauses.len(),
            assignment: seed.to_vec(), trail: Vec::new(), stack: Vec::new(),
        })
    }

    fn assign(&mut self, variable: usize, target: Trit) -> bool {
        match self.assignment[variable - 1] {
            Trit::Zero => {
                self.assignment[variable - 1] = target;
                self.trail.push((variable, target));
                add(&mut self.propagations, 1);
                true
            }
            existing => existing == target,
        }
    }

    fn undo_to(&mut self, mark: usize) {
        while self.trail.len() > mark {
            let (variable, _) = self.trail.pop().expect("rollback trail");
            self.assignment[variable - 1] = Trit::Zero;
        }
    }

    fn reject_node(&mut self) {
        loop {
            let Some(top) = self.stack.last() else {
                self.state = FrontierState::Unsat;
                return;
            };
            let mark = top.mark;
            let variable = top.variable;
            let minus_started = top.minus_branch;
            self.undo_to(mark);
            if minus_started {
                self.stack.pop();
            } else {
                self.stack.last_mut().expect("branch frame").minus_branch = true;
                let assigned = self.assign(variable, Trit::Minus);
                assert!(assigned, "previous branch assignment was not rolled back");
                return;
            }
        }
    }

    fn propagate(&mut self, cnf: &Cnf) -> bool {
        loop {
            let mut changed = false;
            for clause in &cnf.clauses {
                let mut satisfied = false;
                let mut open = 0_usize;
                let mut last = 0_i32;
                for &literal in clause {
                    add(&mut self.literal_checks, 1);
                    let target = Trit::from_literal(literal);
                    let current = self.assignment[literal.unsigned_abs() as usize - 1];
                    if current == target {
                        satisfied = true;
                        break;
                    }
                    if current == Trit::Zero {
                        open += 1;
                        last = literal;
                    }
                }
                if satisfied { continue; }
                if open == 0 { return false; }
                if open == 1 {
                    let variable = last.unsigned_abs() as usize;
                    if !self.assign(variable, Trit::from_literal(last)) {
                        return false;
                    }
                    changed = true;
                }
            }
            if !changed { return true; }
        }
    }

    fn visit(&mut self, cnf: &Cnf, guide: &Guidance,
             shadow: &mut Option<ShadowSelector>) -> Result<()> {
        add(&mut self.nodes, 1);
        self.trace_sha256 = hash_bytes(format!(
            "PNP_DPLL_FRONTIER_VISIT_V1|{}|{}|{}|{}",
            self.trace_sha256, self.nodes, self.trail.len(),
            root_sha(&self.assignment)
        ).as_bytes());
        if !self.propagate(cnf) {
            self.reject_node();
            return Ok(());
        }
        let first = guide.order.iter().copied()
            .find(|&v| self.assignment[v - 1] == Trit::Zero);
        let Some(first) = first else {
            if cnf.valid_complete_model(&self.assignment) {
                self.state = FrontierState::Sat;
            } else {
                self.reject_node();
            }
            return Ok(());
        };
        let choice = if let Some(selector) = shadow {
            selector.choose(cnf, &guide.order, &self.assignment)?
        } else {
            first
        };
        need(self.assignment[choice - 1] == Trit::Zero,
             "selected shadow branch is already assigned")?;
        if self.decisions == 0 {
            self.first_order = first as u64;
            self.first_selected = choice as u64;
        }
        add(&mut self.decisions, 1);
        self.trace_sha256 = hash_bytes(format!(
            "PNP_DPLL_FRONTIER_BRANCH_V1|{}|{}|{}|{}|{}",
            self.trace_sha256, self.nodes, self.decisions, first, choice
        ).as_bytes());
        self.stack.push(ChoiceFrame {
            mark: self.trail.len(), variable: choice, minus_branch: false,
        });
        assert!(self.assign(choice, Trit::Plus));
        Ok(())
    }

    /// Advance a bounded *amount of completed work*, not a bounded lifetime.
    /// A checkpoint is always emitted between node visits, never halfway
    /// through CNF propagation. An exact larger run is the concatenation of
    /// any number of complete node quanta with no replay of prior nodes.
    pub fn advance(
        &mut self, cnf: &Cnf, guide: &Guidance,
        quota: u64,
    ) -> Result<Quantum> {
        need(self.variables == cnf.variables && self.clauses == cnf.clauses.len(),
             "frontier original CNF dimensions changed")?;
        need(self.guidance_sha256 == order_sha(&guide.order)?,
             "frontier original Fischer ordering changed")?;
        need(quota > 0, "a DPLL session must request positive node work")?;
        let before_nodes = self.nodes;
        let before_decisions = self.decisions;
        let mut shadow = match self.mode.shadow() {
            Some(mode) if self.state == FrontierState::Running =>
                Some(ShadowSelector::new(cnf, &self.source_sha256, mode)?),
            _ => None,
        };
        while self.state == FrontierState::Running && self.nodes - before_nodes < quota {
            self.visit(cnf, guide, &mut shadow)?;
        }
        let mut delta = Quantum {
            visited: self.nodes - before_nodes, status: self.state,
            new_decisions: self.decisions - before_decisions,
            projected_rooms: 0, projected_bytes: 0, pixel_reads: 0,
            reconstructed_rooms: 0, source_trit_reads: 0, source_index_bytes: 0,
            first_order: self.first_order, first_selected: self.first_selected,
        };
        if let Some(selector) = shadow {
            let meters = selector.into_metrics();
            delta.projected_rooms = meters.room_projections;
            delta.projected_bytes = meters.stored_projection_bytes;
            delta.pixel_reads = meters.pixel_reads;
            delta.reconstructed_rooms = meters.reconstructed_rooms;
            delta.source_trit_reads = meters.source_trit_reads;
            delta.source_index_bytes = meters.source_index_bytes;
            add(&mut self.projected_rooms, delta.projected_rooms);
            add(&mut self.projected_bytes, delta.projected_bytes);
            add(&mut self.pixel_reads, delta.pixel_reads);
            add(&mut self.reconstructed_rooms, delta.reconstructed_rooms);
            add(&mut self.source_trit_reads, delta.source_trit_reads);
            add(&mut self.source_index_bytes, delta.source_index_bytes);
        }
        Ok(delta)
    }

    pub fn model(&self) -> Option<&[Trit]> {
        (self.state == FrontierState::Sat).then_some(self.assignment.as_slice())
    }
    pub fn trail_len(&self) -> usize { self.trail.len() }
    pub fn stack_len(&self) -> usize { self.stack.len() }

    pub fn freeze(&self) -> Result<Vec<u8>> {
        need(self.trail.len() <= self.variables && self.stack.len() <= self.trail.len(),
             "frontier trail or stack exceeds original variable width")?;
        let mut result = Vec::new();
        result.extend_from_slice(MAGIC);
        result.extend_from_slice(&VERSION.to_le_bytes());
        for field in [
            &self.source_sha256, &self.guidance_sha256,
            &self.seed_sha256, &self.trace_sha256
        ] {
            need(good_sha(field), "cannot freeze invalid source-bound SHA")?;
            result.extend_from_slice(field.as_bytes());
        }
        result.push(self.mode.symbol());
        result.push(self.state.symbol());
        for value in [
            self.variables as u64, self.clauses as u64, self.nodes,
            self.propagations, self.literal_checks, self.decisions,
            self.first_order, self.first_selected,
            self.projected_rooms, self.projected_bytes, self.pixel_reads,
            self.reconstructed_rooms, self.source_trit_reads,
            self.source_index_bytes,
        ] { write_u64(&mut result, value); }
        write_u32(&mut result, self.trail.len())?;
        write_u32(&mut result, self.stack.len())?;
        for &(variable, trit) in &self.trail {
            write_u32(&mut result, variable)?;
            result.push(trit_byte(trit));
        }
        for frame in &self.stack {
            write_u32(&mut result, frame.variable)?;
            write_u32(&mut result, frame.mark)?;
            result.push(u8::from(frame.minus_branch));
        }
        let digest = hash_bytes(&result);
        result.extend_from_slice(digest.as_bytes());
        Ok(result)
    }

    pub fn thaw(
        bytes: &[u8], cnf: &Cnf, guide: &Guidance,
        seed: &[Trit], expected_source_sha: &str,
        expected_mode: FrontierMode,
    ) -> Result<Self> {
        let minimum = 8 + 4 + DIGEST_SIZE * 5 + 2 + 14 * 8 + 8;
        need(bytes.len() >= minimum, "source-bound DPLL checkpoint too short")?;
        let expected_digest = hash_bytes(&bytes[..bytes.len() - DIGEST_SIZE]);
        let claimed_digest = std::str::from_utf8(&bytes[bytes.len() - DIGEST_SIZE..])
            .map_err(|_| "non-ASCII frontier checkpoint checksum")?;
        need(expected_digest == claimed_digest,
             "DPLL frontier checkpoint SHA256 integrity failed")?;
        let mut reader = Reader::new(&bytes[..bytes.len() - DIGEST_SIZE]);
        need(reader.take(8)? == MAGIC && reader.u32()? == VERSION,
             "unknown source-bound frontier format")?;
        let source_sha256 = reader.sha()?;
        let guidance_sha256 = reader.sha()?;
        let seed_sha256 = reader.sha()?;
        let trace_sha256 = reader.sha()?;
        let mode = FrontierMode::from_symbol(reader.u8()?)?;
        let state = FrontierState::from_symbol(reader.u8()?)?;
        need(expected_source_sha == source_sha256 && good_sha(expected_source_sha)
             && mode == expected_mode, "frontier original source or guide mode changed")?;
        let variables = usize::try_from(reader.u64()?)
            .map_err(|_| "checkpoint variable width exceeds machine")?;
        let clauses = usize::try_from(reader.u64()?)
            .map_err(|_| "checkpoint clause width exceeds machine")?;
        need(variables == cnf.variables && clauses == cnf.clauses.len(),
             "frontier CNF dimensions changed")?;
        need(seed.len() == variables && root_sha(seed) == seed_sha256,
             "frontier initial GGUF state SHA256 changed")?;
        need(guidance_sha256 == order_sha(&guide.order)?,
             "frontier source-frozen Fischer ordering changed")?;
        let nodes = reader.u64()?;
        let propagations = reader.u64()?;
        let literal_checks = reader.u64()?;
        let decisions = reader.u64()?;
        let first_order = reader.u64()?;
        let first_selected = reader.u64()?;
        let projected_rooms = reader.u64()?;
        let projected_bytes = reader.u64()?;
        let pixel_reads = reader.u64()?;
        let reconstructed_rooms = reader.u64()?;
        let source_trit_reads = reader.u64()?;
        let source_index_bytes = reader.u64()?;
        let trail_count = reader.u32()? as usize;
        let frame_count = reader.u32()? as usize;
        need(trail_count <= variables && frame_count <= trail_count,
             "checkpoint DPLL trail/stack size impossible")?;
        let mut assignment = seed.to_vec();
        let mut trail = Vec::with_capacity(trail_count);
        for _ in 0..trail_count {
            let variable = reader.u32()? as usize;
            let value = byte_trit(reader.u8()?)?;
            need(variable > 0 && variable <= variables && value != Trit::Zero,
                 "checkpoint contains invalid Boolean SAT variable")?;
            need(assignment[variable - 1] == Trit::Zero,
                 "checkpoint trail repeats a source SAT variable")?;
            assignment[variable - 1] = value;
            trail.push((variable, value));
        }
        let mut stack: Vec<ChoiceFrame> = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            let variable = reader.u32()? as usize;
            let mark = reader.u32()? as usize;
            let branch = reader.u8()?;
            need(variable > 0 && variable <= variables
                 && mark < trail.len() && branch <= 1,
                 "checkpoint decision frame is outside live SAT trail")?;
            need(trail[mark].0 == variable
                 && trail[mark].1 ==
                    (if branch == 1 { Trit::Minus } else { Trit::Plus }),
                 "checkpoint branch polarity disagrees with actual trail")?;
            need(match stack.last() {
                Some(last) => last.mark < mark,
                None => true,
            },
                 "checkpoint nested decision positions are not monotone")?;
            stack.push(ChoiceFrame {
                variable, mark, minus_branch: branch == 1,
            });
        }
        need(reader.offset == bytes.len() - DIGEST_SIZE,
             "checkpoint has undeclared trailing bytes")?;
        need(!(state == FrontierState::Unsat && !stack.is_empty()),
             "UNSAT certificate contains unresolved original SAT branches")?;
        need(!(state == FrontierState::Sat && !cnf.valid_complete_model(&assignment)),
             "SAT certificate fails original complete CNF")?;
        need(nodes >= decisions && propagations >= trail.len() as u64,
             "checkpoint search-work counters impossible")?;
        Ok(Self {
            mode, state, source_sha256, guidance_sha256,
            seed_sha256, trace_sha256, nodes, propagations,
            literal_checks, decisions, first_order, first_selected,
            projected_rooms, projected_bytes, pixel_reads,
            reconstructed_rooms, source_trit_reads, source_index_bytes,
            variables, clauses, assignment, trail, stack,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guidance::{Guidance, Strategy};

    fn cases() -> Vec<&'static str> {
        vec![
            "p cnf 4 2\n1 2 0\n-2 3 0\n",
            "p cnf 3 8\n1 2 3 0\n1 2 -3 0\n1 -2 3 0\n1 -2 -3 0\n-1 2 3 0\n-1 2 -3 0\n-1 -2 3 0\n-1 -2 -3 0\n",
            "p cnf 1 2\n1 0\n-1 0\n",
        ]
    }

    #[test]
    fn exactly_same_full_trace_after_any_checkpoint_schedule() {
        for text in cases() {
            let cnf = Cnf::parse(text).expect("source");
            let guide = Guidance::for_source(&cnf, Strategy::Fischer).expect("guide");
            let source = hash_bytes(text.as_bytes());
            let seed = vec![Trit::Zero; cnf.variables];
            for mode in [FrontierMode::Direct, FrontierMode::One, FrontierMode::Three] {
                let mut whole = Frontier::new(&cnf, &guide, &seed, &source, mode)
                    .expect("uninterrupted");
                whole.advance(&cnf, &guide, 128).expect("uninterrupted work");
                assert_ne!(whole.state, FrontierState::Running, "128 nodes enough for toy SAT");
                let mut segmented = Frontier::new(&cnf, &guide, &seed, &source, mode)
                    .expect("segmented");
                for step in 0..128 {
                    let quota = if step % 3 == 0 { 1 } else { 2 };
                    segmented.advance(&cnf, &guide, quota).expect("one quantum");
                    let frozen = segmented.freeze().expect("checkpoint");
                    segmented = Frontier::thaw(
                        &frozen, &cnf, &guide, &seed, &source, mode
                    ).expect("authenticated complete frontier restore");
                    if segmented.state != FrontierState::Running { break; }
                }
                assert_eq!(whole.state, segmented.state);
                assert_eq!(whole.nodes, segmented.nodes);
                assert_eq!(whole.literal_checks, segmented.literal_checks);
                assert_eq!(whole.propagations, segmented.propagations);
                assert_eq!(whole.decisions, segmented.decisions);
                assert_eq!(whole.first_order, segmented.first_order);
                assert_eq!(whole.first_selected, segmented.first_selected);
                assert_eq!(whole.trace_sha256, segmented.trace_sha256);
                assert_eq!(whole.model(), segmented.model());
                assert_eq!(whole.projected_rooms, segmented.projected_rooms);
                assert_eq!(whole.projected_bytes, segmented.projected_bytes);
                assert_eq!(whole.pixel_reads, segmented.pixel_reads);
            }
        }
    }

    #[test]
    fn tampered_wrong_source_or_guide_rejected_before_resume() {
        let text = "p cnf 3 2\n1 2 0\n-2 3 0\n";
        let cnf = Cnf::parse(text).expect("cnf");
        let guide = Guidance::for_source(&cnf, Strategy::Fischer).expect("order");
        let root = vec![Trit::Zero; cnf.variables];
        let digest = hash_bytes(text.as_bytes());
        let mut state = Frontier::new(&cnf,&guide,&root,&digest,FrontierMode::Three)
            .expect("frontier");
        state.advance(&cnf,&guide,1).expect("first branch");
        let archive = state.freeze().expect("archive");
        let mut corrupt = archive.clone();
        corrupt[30] ^= 1;
        assert!(Frontier::thaw(&corrupt,&cnf,&guide,&root,&digest,FrontierMode::Three).is_err());
        assert!(Frontier::thaw(&archive,&cnf,&guide,&root,
                              &hash_bytes(b"other"),FrontierMode::Three).is_err());
        assert!(Frontier::thaw(&archive,&cnf,&guide,&root,&digest,FrontierMode::One).is_err());
        assert!(Frontier::thaw(&archive[..archive.len()-1],&cnf,&guide,&root,
                              &digest,FrontierMode::Three).is_err());
        assert!(Frontier::thaw(&archive,&cnf,&guide,
                              &[Trit::Plus,Trit::Zero,Trit::Zero],
                              &digest,FrontierMode::Three).is_err());
    }

    #[test]
    fn minus_branch_is_persisted_and_not_revisited_after_replay() {
        // Four two-literal clauses force all four assignments of a pair
        // to conflict; resumed DPLL must preserve which polarity was tried.
        let source = "p cnf 2 4\n1 2 0\n-1 2 0\n1 -2 0\n-1 -2 0\n";
        let cnf = Cnf::parse(source).expect("CNF");
        let guide = Guidance::for_source(&cnf, Strategy::Fischer).expect("guide");
        let sha = hash_bytes(source.as_bytes());
        let seed = [Trit::Zero; 2];
        let mut f = Frontier::new(&cnf,&guide,&seed,&sha,FrontierMode::Direct)
            .expect("initial");
        f.advance(&cnf,&guide,2).expect("plus conflict then minus active");
        let checkpoint = f.freeze().expect("freeze");
        let mut restored = Frontier::thaw(
            &checkpoint,&cnf,&guide,&seed,&sha,FrontierMode::Direct
        ).expect("play");
        assert_eq!(restored.nodes, 2);
        assert_eq!(restored.state, FrontierState::Running);
        restored.advance(&cnf,&guide,16).expect("remaining exact tree");
        assert_eq!(restored.state, FrontierState::Unsat);
        assert_eq!(restored.nodes, 3);
    }
}
