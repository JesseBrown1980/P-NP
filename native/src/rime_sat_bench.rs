//! Reproducible CPU-only SAT comparison with published Constellation Rime body.
//! All measured arms solve the same small, exact-verifiable SAT/UNSAT cases.
//! Times are measurements, not deterministic certificate bytes or P=NP claims.
#![forbid(unsafe_code)]

use std::path::Path;
use std::time::Instant;

use crate::guidance::{Guidance, Strategy};
use crate::rime_disk::{self, DiskRime};
use crate::solver::{check_against_exhaustive, solve, Status};
use crate::ternary::{Cnf, Trit};

const ARM_COUNT: usize = 4;
const CASE_SEEDS: usize = 10;

#[derive(Clone, Copy)]
enum Arm {
    Direct,
    Fischer,
    FrozenDisk,
    PumpFunnel,
}
impl Arm {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Direct,
            1 => Self::Fischer,
            2 => Self::FrozenDisk,
            _ => Self::PumpFunnel,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Direct => "DIRECT_BASE",
            Self::Fischer => "FISCHER",
            Self::FrozenDisk => "FROZEN_DISK",
            Self::PumpFunnel => "PUMP_FUNNEL_HTTP0",
        }
    }
}

fn prng(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn planted_sat(seed: u64) -> Cnf {
    let mut rng = seed.wrapping_mul(13).wrapping_add(1);
    let plant: [Trit; 10] = std::array::from_fn(|i| {
        if prng(&mut rng).wrapping_add(i as u64) & 1 == 0 {
            Trit::Minus
        } else {
            Trit::Plus
        }
    });
    let mut clauses = Vec::with_capacity(36);
    for _ in 0..36 {
        let mut variables = Vec::new();
        while variables.len() < 3 {
            let var = (prng(&mut rng) >> 32) % 10 + 1;
            if !variables.contains(&(var as i32)) {
                variables.push(var as i32);
            }
        }
        let mut clause: Vec<i32> = variables.iter().map(|&var| {
            if prng(&mut rng) >> 63 == 0 { var } else { -var }
        }).collect();
        let satisfied = clause.iter().any(|&lit| {
            plant[lit.unsigned_abs() as usize - 1] == Trit::from_literal(lit)
        });
        if !satisfied { clause[0] = -clause[0]; }
        clauses.push(clause);
    }
    let cnf = Cnf { variables: 10, clauses };
    assert!(cnf.valid_complete_model(&plant));
    cnf
}

fn parity_unsat(seed: u64) -> Cnf {
    let mut variables: Vec<i32> = (1..=10).collect();
    let mut rng = seed * 29 + 3;
    for i in (1..variables.len()).rev() {
        let j = (prng(&mut rng) as usize) % (i + 1);
        variables.swap(i, j);
    }
    let mut clauses = Vec::new();
    for pair in variables.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        // Every variable in the chain must agree with its neighbor.
        clauses.push(vec![-a, b]);
        clauses.push(vec![a, -b]);
    }
    let (a, b) = (variables[0], *variables.last().expect("chain nonempty"));
    clauses.push(vec![a, b]);
    clauses.push(vec![-a, -b]);
    Cnf { variables: 10, clauses }
}

fn blocking_unsat() -> Cnf {
    let mut clauses = Vec::new();
    for assignment in 0_usize..(1 << 8) {
        let clause = (0..8)
            .map(|i| if assignment & (1 << i) != 0 {
                -(i + 1)
            } else {
                i + 1
            })
            .collect();
        clauses.push(clause);
    }
    Cnf { variables: 8, clauses }
}

fn original_suite() -> Result<Vec<(String, Cnf, Status)>, String> {
    let fixed = [
        ("sat3", include_str!("../fixtures/sat3.cnf"), Status::Sat),
        ("unsat2", include_str!("../fixtures/unsat2.cnf"), Status::Unsat),
        ("php4_3", include_str!("../fixtures/php4_3.cnf"), Status::Unsat),
    ];
    let mut cases = Vec::new();
    for (label, raw, expected) in fixed {
        cases.push((label.to_owned(), Cnf::parse(raw)?, expected));
    }
    for seed in 1..=CASE_SEEDS {
        cases.push((format!("planted_{seed}"), planted_sat(seed as u64), Status::Sat));
        cases.push((format!("parity_{seed}"), parity_unsat(seed as u64), Status::Unsat));
    }
    cases.push(("blocking_8".to_owned(), blocking_unsat(), Status::Unsat));
    Ok(cases)
}

fn direct(cnf: &Cnf, strategy: Strategy) -> Result<Guidance, String> {
    Guidance::for_source(cnf, strategy)
}

fn rime_guidance(
    cnf: &Cnf,
    disk: &DiskRime,
    via_loopback: bool,
) -> Result<(Guidance, u64), String> {
    let mut prepared = direct(cnf, Strategy::Base)?;
    let total_literals: usize = cnf.clauses.iter().map(Vec::len).sum();
    let mut occurrences = vec![0_u64; cnf.variables];
    for row in &cnf.clauses {
        for &lit in row {
            occurrences[lit.unsigned_abs() as usize - 1] += 1;
        }
    }
    let mut priority = vec![0_i128; cnf.variables];
    for (i, item) in priority.iter_mut().enumerate() {
        let tensor = ((i + total_literals) % disk.tensor_count()) as u8;
        let glyph = ((i + cnf.clauses.len()) % 27) as u8;
        let value = if via_loopback {
            rime_disk::loopback_once(disk, tensor, glyph)?
        } else {
            let req = rime_disk::request(tensor, glyph)?;
            let address = u64::from_le_bytes(req[3..11].try_into()
                .map_err(|_| "request address width")?);
            disk.read(usize::from(tensor), usize::from(glyph), address)?
        };
        // Experimental *source-independent* tie-break proxy. These GGUF
        // coordinates have not been trained to predict SAT branching.
        *item = i128::from(occurrences[i]) * 2_048 + i128::from(value);
    }
    prepared.order.sort_by_key(|&var| (-priority[var - 1], var));
    Ok((prepared, cnf.variables as u64))
}

#[derive(Default)]
struct Counts {
    prep_solve_ns: u128,
    verify_ns: u128,
    nodes: u64,
    literal_checks: u64,
    read_requests: u64,
    path_solves: u64,
    comparisons: u64,
}

fn verify(cnf: &Cnf, status: Status, expected: Status, model: Option<&[Trit]>,
          full: &crate::solver::Solution) -> Result<(), String> {
    if status == Status::Hold || status != expected {
        return Err(format!("SAT/UNSAT mismatch expected {} got {}", expected.label(), status.label()));
    }
    if status == Status::Sat && !model.is_some_and(|x| cnf.valid_complete_model(x)) {
        return Err("satisfiable model failed independent CNF evaluation".into());
    }
    if !check_against_exhaustive(cnf, full) {
        return Err("independent complete enumeration contradicted solver".into());
    }
    Ok(())
}

fn one_case(cnf: &Cnf, expected: Status, disk: &DiskRime, arm: Arm)
    -> Result<Counts, String>
{
    let mut report = Counts::default();
    let start = Instant::now();
    let strategies = match arm {
        Arm::Direct => vec![Strategy::Base],
        Arm::Fischer => vec![Strategy::Fischer],
        Arm::FrozenDisk => vec![Strategy::Base],
        Arm::PumpFunnel => vec![Strategy::Base, Strategy::Fischer, Strategy::Base],
    };
    let mut solutions = Vec::new();
    for (index, strategy) in strategies.into_iter().enumerate() {
        let needs_rime = matches!(arm, Arm::FrozenDisk)
            || matches!(arm, Arm::PumpFunnel) && index == 2;
        let (guidance, reads) = if needs_rime {
            rime_guidance(cnf, disk, matches!(arm, Arm::PumpFunnel))?
        } else {
            (direct(cnf, strategy)?, 0)
        };
        report.read_requests += reads;
        let result = solve(cnf, &guidance, None);
        report.nodes += result.nodes;
        report.literal_checks += result.literal_checks;
        report.path_solves += 1;
        solutions.push(result);
    }
    report.prep_solve_ns = start.elapsed().as_nanos();
    let check_start = Instant::now();
    for solution in &solutions {
        verify(cnf, solution.status, expected, solution.model.as_deref(), solution)?;
        report.comparisons += 1;
    }
    // Funnel: all three independently checked routes must agree before
    // selecting a reusable source-specific result. No uncharged best-of-three.
    if matches!(arm, Arm::PumpFunnel)
        && solutions.iter().any(|out| out.status != solutions[0].status)
    {
        return Err("Pump/Funnel routes do not agree".into());
    }
    report.verify_ns = check_start.elapsed().as_nanos();
    Ok(report)
}

#[derive(Default)]
struct Samples {
    prep: Vec<u128>,
    verify: Vec<u128>,
    nodes: u64,
    checks: u64,
    reads: u64,
    solves: u64,
    comparisons: u64,
}
impl Samples {
    fn add(&mut self, result: Counts) {
        self.nodes += result.nodes;
        self.checks += result.literal_checks;
        self.reads += result.read_requests;
        self.solves += result.path_solves;
        self.comparisons += result.comparisons;
    }
}
fn median(mut timings: Vec<u128>) -> u128 {
    timings.sort_unstable();
    timings[timings.len() / 2]
}

pub fn run(path: &Path, rounds: usize) -> Result<Vec<String>, String> {
    if !(3..=11).contains(&rounds) || rounds % 2 == 0 {
        return Err("benchmark rounds must be an odd integer from 3 to 11".into());
    }
    let setup_start = Instant::now();
    let disk = DiskRime::open_four_axes(path)?;
    let rime_pairs = disk.verify_three_paths()?;
    let shadow_cubes = disk.verify_cube_shadows()?;
    let preflight_ns = setup_start.elapsed().as_nanos();
    let cases = original_suite()?;
    if cases.len() != 24 || rime_pairs != 324 || shadow_cubes != 24 {
        return Err("published GGUF acceptance or SAT suite cardinality changed".into());
    }
    let sat_cases = cases.iter().filter(|(_, _, e)| *e == Status::Sat).count();
    let unsat_cases = cases.len() - sat_cases;
    let mut stats: [Samples; ARM_COUNT] = std::array::from_fn(|_| Samples::default());
    let mut lines = vec![format!(
        "PNP_RIME_SAT_BENCH|schema=v1|source_sha256={}|cases={}|sat={sat_cases}|unsat={unsat_cases}|rounds={rounds}|rime_pairs={rime_pairs}|shadow_cubes={shadow_cubes}|preflight_ns={preflight_ns}|integer_only=1|page_cache=UNCONTROLLED|json=0",
        rime_disk::FOUR_AXES_SHA256, cases.len()
    )];
    for round in 0..rounds {
        let mut by_arm: [Counts; ARM_COUNT] = std::array::from_fn(|_| Counts::default());
        for index in 0..ARM_COUNT {
            let arm_index = (index + round) % ARM_COUNT;
            let arm = Arm::from_index(arm_index);
            let mut round_total = Counts::default();
            for (label, cnf, expected) in &cases {
                let result = one_case(cnf, *expected, &disk, arm)
                    .map_err(|e| format!("case={label} arm={}: {e}", arm.label()))?;
                round_total.prep_solve_ns += result.prep_solve_ns;
                round_total.verify_ns += result.verify_ns;
                round_total.nodes += result.nodes;
                round_total.literal_checks += result.literal_checks;
                round_total.read_requests += result.read_requests;
                round_total.path_solves += result.path_solves;
                round_total.comparisons += result.comparisons;
            }
            by_arm[arm_index] = round_total;
        }
        for i in 0..ARM_COUNT {
            let report = std::mem::take(&mut by_arm[i]);
            stats[i].prep.push(report.prep_solve_ns);
            stats[i].verify.push(report.verify_ns);
            stats[i].add(report);
        }
    }
    for (i, arm_stats) in stats.iter().enumerate() {
        let prep = median(arm_stats.prep.clone());
        let verified = median(arm_stats.verify.clone());
        // Separate medians cannot be summed to obtain the median total
        // on a round. Report both components rather than a pseudo total.
        lines.push(format!(
            "PNP_RIME_SAT_ARM|arm={}|median_prep_solve_ns={prep}|median_verify_ns={verified}|nodes={}|literal_checks={}|disk_requests={}|path_solves={}|independent_checks={}|status=PASS|json=0",
            Arm::from_index(i).label(), arm_stats.nodes,
            arm_stats.checks, arm_stats.reads, arm_stats.solves, arm_stats.comparisons
        ));
    }
    disk.verify_source()?;
    lines.push(
        "PNP_RIME_SAT_ACCEPT|same_cases=PASS|sat_model=PASS|unsat_exhaustive=PASS|three_path_consensus=PASS|source_after=PASS|speed_status=MEASURED|p_np_status=OPEN|json=0".to_owned()
    );
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_cases_have_independently_validated_truth() {
        let cases = original_suite().expect("fixed suite");
        assert_eq!(cases.len(), 24);
        assert_eq!(cases.iter().filter(|c| c.2 == Status::Sat).count(), 11);
        for (_, cnf, expected) in cases {
            let guidance = direct(&cnf, Strategy::Base).expect("source order");
            let answer = solve(&cnf, &guidance, None);
            verify(&cnf, answer.status, expected, answer.model.as_deref(), &answer)
                .expect("independently exhaustive verdict");
        }
    }
}
