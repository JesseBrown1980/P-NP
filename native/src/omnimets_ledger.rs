//! Native source-mapped OmniMets monitoring for bounded ternary shell agents.
//! Original: asolaria-behcs-256 @ 818cc235a6db815e3b2591339cf9b42aad3e85a0,
//! packages/omnimets/src/{mets.ts,convergence.ts}.
//! Original 7 per-lane counters, K=3 agreement, ratify-before-seal and early
//! convergence are adapted for an exact integer Rust microkernel witness.
//! This module observes established kernels; it does not simulate GNN inference.
#![forbid(unsafe_code)]

use std::time::Instant;

use crate::sha256::hash_bytes;

pub const SOURCE_COMMIT: &str = "818cc235a6db815e3b2591339cf9b42aad3e85a0";
pub const METS_BLOB: &str = "49bd2c7daec59a8301fd6010f5fdefa11ec2da00";
pub const CONVERGENCE_BLOB: &str = "c29da7ac6ce6b50af20c552915b67442df565c37";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lane {
    Muscular,
    Skeletal,
    Nervous,
    Circulatory,
    Endocrine,
    Digestive,
    Cross,
}
impl Lane {
    pub const ALL: [Self; 7] = [
        Self::Muscular, Self::Skeletal, Self::Nervous,
        Self::Circulatory, Self::Endocrine, Self::Digestive,
        Self::Cross,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Muscular => "muscular",
            Self::Skeletal => "skeletal",
            Self::Nervous => "nervous",
            Self::Circulatory => "circulatory",
            Self::Endocrine => "endocrine",
            Self::Digestive => "digestive",
            Self::Cross => "cross",
        }
    }
    fn index(self) -> usize {
        match self {
            Self::Muscular => 0,
            Self::Skeletal => 1,
            Self::Nervous => 2,
            Self::Circulatory => 3,
            Self::Endocrine => 4,
            Self::Digestive => 5,
            Self::Cross => 6,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LaneCounters {
    pub dispatches: u64,
    pub promotions: u64,
    pub deferrals: u64,
    pub halts: u64,
    pub errors: u64,
    pub latency_sum_ns: u128,
    pub samples: u64,
}
impl LaneCounters {
    fn combine(self, other: Self) -> Self {
        Self {
            dispatches: self.dispatches + other.dispatches,
            promotions: self.promotions + other.promotions,
            deferrals: self.deferrals + other.deferrals,
            halts: self.halts + other.halts,
            errors: self.errors + other.errors,
            latency_sum_ns: self.latency_sum_ns + other.latency_sum_ns,
            samples: self.samples + other.samples,
        }
    }
}
#[derive(Clone, Copy)]
enum Event {
    Dispatch,
    Promote,
    Defer,
    Halt,
    Error,
}
impl Event {
    fn name(self) -> &'static str {
        match self {
            Self::Dispatch => "DISPATCH",
            Self::Promote => "PROMOTE",
            Self::Defer => "DEFER",
            Self::Halt => "HALT",
            Self::Error => "ERROR",
        }
    }
}

/// A 7-lane OmniMets receipt and event-chain for checked microkernel calls.
/// Every dispatch is charged; timed samples include failures, too.
#[derive(Clone, Debug)]
pub struct Meter {
    lanes: [LaneCounters; 7],
    event_root: String,
    events: usize,
    ratifications: usize,
    seals: usize,
}
impl Default for Meter {
    fn default() -> Self {
        Self {
            lanes: [LaneCounters::default(); 7],
            event_root: hash_bytes(b"PNP-OMNIMETS-EXACT-SOURCE-PIN-v1"),
            events: 0,
            ratifications: 0,
            seals: 0,
        }
    }
}
impl Meter {
    fn record(&mut self, lane: Lane, event: Event) {
        let counters = &mut self.lanes[lane.index()];
        match event {
            Event::Dispatch => counters.dispatches += 1,
            Event::Promote => counters.promotions += 1,
            Event::Defer => counters.deferrals += 1,
            Event::Halt => counters.halts += 1,
            Event::Error => counters.errors += 1,
        }
        self.events += 1;
        let row = format!(
            "METS|event={}|lane={}|index={}|prev={}",
            event.name(), lane.name(), self.events, self.event_root
        );
        self.event_root = hash_bytes(row.as_bytes());
    }
    /// Executes and observes a real call, without changing its answer.
    pub fn execute<T>(
        &mut self, lane: Lane,
        call: impl FnOnce() -> Result<T, &'static str>,
    ) -> Result<T, &'static str> {
        self.record(lane, Event::Dispatch);
        let start = Instant::now();
        let result = call();
        let elapsed = start.elapsed().as_nanos();
        let counters = &mut self.lanes[lane.index()];
        counters.latency_sum_ns += elapsed;
        counters.samples += 1;
        if result.is_ok() {
            self.record(lane, Event::Promote);
        } else {
            self.record(lane, Event::Error);
            self.record(lane, Event::Halt);
        }
        result
    }
    fn defer(&mut self) {
        self.record(Lane::Cross, Event::Defer);
    }
    pub fn totals(&self) -> LaneCounters {
        self.lanes.iter().copied().fold(LaneCounters::default(), LaneCounters::combine)
    }
    pub fn lane(&self, lane: Lane) -> LaneCounters {
        self.lanes[lane.index()]
    }
    pub fn ratifications(&self) -> usize {
        self.ratifications
    }
    pub fn seals(&self) -> usize {
        self.seals
    }
    pub fn receipt(&self) -> String {
        let mut body = format!(
            "HBPHEADER|schema=pnp.omnimets.v1|original_commit={SOURCE_COMMIT}|mets_blob={METS_BLOB}|convergence_blob={CONVERGENCE_BLOB}|lanes=7|k=3|seal=RATIFY_FIRST|integer_only=1|json=0\n"
        );
        for lane in Lane::ALL {
            let c = self.lane(lane);
            body.push_str(&format!(
                "OMNIMETS_LANE|lane={}|dispatches={}|promotions={}|deferrals={}|halts={}|errors={}|latency_sum_ns={}|samples={}|json=0\n",
                lane.name(), c.dispatches, c.promotions, c.deferrals,
                c.halts, c.errors, c.latency_sum_ns, c.samples,
            ));
        }
        let total = self.totals();
        body.push_str(&format!(
            "OMNIMETS_TOTAL|dispatches={}|promotions={}|deferrals={}|halts={}|errors={}|latency_sum_ns={}|samples={}|ratified={}|sealed={}|event_count={}|event_root={}|json=0\n",
            total.dispatches, total.promotions, total.deferrals,
            total.halts, total.errors, total.latency_sum_ns, total.samples,
            self.ratifications, self.seals, self.events, self.event_root
        ));
        let digest = hash_bytes(body.as_bytes());
        body.push_str(&format!("OMNIMETS_SHA|body_sha256={digest}|json=0\n"));
        body
    }
    pub fn verify_receipt(&self, receipt: &str) -> bool {
        let Some((body, last)) = receipt.rsplit_once("OMNIMETS_SHA|body_sha256=") else {
            return false;
        };
        let Some(digest) = last.strip_suffix("|json=0\n") else {
            return false;
        };
        digest.len() == 64
            && hash_bytes(body.as_bytes()) == digest
            && receipt == self.receipt()
    }
}

/// The original OmniMets requires at least three consecutive aligned
/// evaluations and a ratification phase before SEAL. This bounded gate
/// uses three independently checked child presentations in one shell.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Convergence {
    consecutive: u8,
    ratified: bool,
    sealed: bool,
}
impl Convergence {
    pub fn observe(&mut self, independent_agreement: bool) -> Result<(), &'static str> {
        if self.sealed {
            return Err("sealed shell cannot accept further observations");
        }
        if independent_agreement {
            self.consecutive = self.consecutive.saturating_add(1);
        } else {
            self.consecutive = 0;
            self.ratified = false;
        }
        Ok(())
    }
    pub fn ratify(&mut self, meter: &mut Meter) -> Result<(), &'static str> {
        if self.consecutive < 3 || self.ratified || self.sealed {
            meter.defer();
            return Err("three consecutive independent presentations required");
        }
        self.ratified = true;
        meter.ratifications += 1;
        Ok(())
    }
    pub fn seal(&mut self, meter: &mut Meter) -> Result<(), &'static str> {
        if !self.ratified || self.consecutive < 3 || self.sealed {
            meter.defer();
            return Err("OmniMets cannot seal without ratification");
        }
        self.sealed = true;
        meter.seals += 1;
        Ok(())
    }
    pub fn sealed(&self) -> bool {
        self.sealed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seven_original_lanes_and_tamper_evident_receipt() {
        let mut meter = Meter::default();
        for lane in Lane::ALL {
            assert_eq!(meter.execute(lane, || Ok(42_i64)), Ok(42));
        }
        let text = meter.receipt();
        assert!(meter.verify_receipt(&text));
        let mut mutated = text;
        mutated = mutated.replacen("dispatches=1", "dispatches=2", 1);
        assert!(!meter.verify_receipt(&mutated));
        assert_eq!(meter.totals().dispatches, 7);
        assert_eq!(meter.totals().promotions, 7);
    }
    #[test]
    fn quorum_three_and_explicit_ratify_before_seal() {
        let mut meter = Meter::default();
        let mut gate = Convergence::default();
        assert!(gate.seal(&mut meter).is_err());
        gate.observe(true).unwrap();
        gate.observe(true).unwrap();
        assert!(gate.ratify(&mut meter).is_err());
        gate.observe(false).unwrap();
        assert!(gate.ratify(&mut meter).is_err());
        for _ in 0..3 {
            gate.observe(true).unwrap();
        }
        gate.ratify(&mut meter).unwrap();
        gate.seal(&mut meter).unwrap();
        assert!(gate.sealed());
        assert!(gate.observe(true).is_err());
        assert_eq!(meter.seals(), 1);
        assert_eq!(meter.ratifications(), 1);
        assert_eq!(meter.totals().deferrals, 3);
    }
}
