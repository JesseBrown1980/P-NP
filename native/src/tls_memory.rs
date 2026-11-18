//! Fixed-point classical Solomon-population experiment inspired by the
//! Gosling et al. PRL qubit-environment-memory work. This simulator is not
//! measured qubit hardware, a TLS fit, or a quantum coherence calculation.
#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;

use crate::sha256::hash_bytes;

const SOURCE: &str = include_str!("../fixtures/PRL_TLS_SOURCE.hbi");
const SCALE: i64 = 10_000;
const BACKGROUND: i64 = 2_000;
const GAMMA_Q: i64 = 900;
const GAMMA_TLS_BATH: i64 = 150;
const COUPLED: i64 = 1_200;
const CELLS: [usize; 5] = [0, 1, 3, 27, 81];
const START_Q: [i64; 3] = [5_000, 3_000, 7_000];
const LAGS: [usize; 5] = [1, 2, 4, 8, 16];
const TICKS: usize = 32;
const ROWS: usize = 1 + 10 + 30 + 150 + 4 + 1;
const HBP_HEADER: &str =
    "HBPHEADER|schema=pnp.prl.tls.fixedpoint.v1|scope=SYNTHETIC_CLASSICAL|seal=HBI-HBP-SHA-SH-HASH|json=0\n";
const HBI_HEADER: &str =
    "HBIHEADER|schema=pnp.prl.tls.fixedpoint.index.v1|for=PNP_TLS.hbp|seal=HBI-HBP-SHA-SH-HASH|json=0\n";

#[derive(Clone, Debug, Eq, PartialEq)]
struct Population {
    q: i64,
    tls: Vec<i64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Retention {
    SummaryOnly,
    ExactReplayRequired,
}

pub fn may_discard(
    kind: Retention,
    committed_summary: bool,
    full_state_retained: bool,
    independent_replay: bool,
) -> bool {
    committed_summary
        && match kind {
            Retention::SummaryOnly => true,
            Retention::ExactReplayRequired => full_state_retained && independent_replay,
        }
}
fn rate_for(index: usize, count: usize, coupled: i64) -> i64 {
    if count == 0 {
        return 0;
    }
    coupled / count as i64 + i64::from(index < coupled as usize % count)
}
fn total_coupling(count: usize, enabled: bool) -> i64 {
    if count == 0 || !enabled {
        0
    } else {
        COUPLED
    }
}
fn start(q: i64, cells: usize, high: bool) -> Population {
    Population {
        q,
        tls: vec![if high { 9_500 } else { 500 }; cells],
    }
}
fn predict_markov(q: i64) -> i64 {
    ((SCALE - GAMMA_Q) * q + GAMMA_Q * BACKGROUND) / SCALE
}
/// Independent net-flux form of the visible-state update. This does not
/// call the simulator's `step` function or use its expanded coefficients.
fn predict_with_hidden(pop: &Population, coupled: i64) -> Result<i64, String> {
    check_population(pop)?;
    let mut flux = GAMMA_Q * (BACKGROUND - pop.q);
    for (index, &reservoir) in pop.tls.iter().enumerate() {
        flux += rate_for(index, pop.tls.len(), coupled) * (reservoir - pop.q);
    }
    // Round once after summing all signed fluxes, as the direct simulator does.
    let numerator = SCALE * pop.q + flux;
    if numerator < 0 {
        return Err("independent hidden-population flux became negative".to_owned());
    }
    Ok(numerator / SCALE)
}
fn check_population(pop: &Population) -> Result<(), String> {
    if !(0..=SCALE).contains(&pop.q)
        || pop.tls.len() > 81
        || pop.tls.iter().any(|v| !(0..=SCALE).contains(v))
    {
        return Err("population escaped certified fixed-point range".to_owned());
    }
    Ok(())
}
fn step(pop: &Population, coupled: i64) -> Result<Population, String> {
    check_population(pop)?;
    let cells = pop.tls.len();
    if coupled < 0 || coupled > COUPLED
        || GAMMA_Q + coupled > SCALE
        || GAMMA_TLS_BATH + coupled > SCALE
    {
        return Err("invalid discrete population rates".to_owned());
    }
    let mut exchange = 0_i64;
    let mut tls_next = Vec::with_capacity(cells);
    for (i, &t) in pop.tls.iter().enumerate() {
        let rate = rate_for(i, cells, coupled);
        exchange += rate * t;
        let numerator =
            rate * pop.q + GAMMA_TLS_BATH * BACKGROUND
            + (SCALE - rate - GAMMA_TLS_BATH) * t;
        tls_next.push(numerator / SCALE);
    }
    let numerator =
        (SCALE - GAMMA_Q - coupled) * pop.q + GAMMA_Q * BACKGROUND + exchange;
    let result = Population {
        q: numerator / SCALE,
        tls: tls_next,
    };
    check_population(&result)?;
    // Independent conservation-style witness: track weighted source sums,
    // rather than replaying the same q numerator expression.
    let witness = predict_markov(pop.q)
        + (exchange - coupled * pop.q) / SCALE;
    // Per-term integer rounding can differ by at most one Q unit.
    if (result.q - witness).abs() > 1 {
        return Err("matrix third-seat TLS update witness disagreed".to_owned());
    }
    Ok(result)
}
fn snapshot(out: &mut Vec<u8>, pop: &Population) -> Result<(), String> {
    check_population(pop)?;
    out.extend_from_slice(&(pop.q as u16).to_le_bytes());
    for &v in &pop.tls {
        out.extend_from_slice(&(v as u16).to_le_bytes());
    }
    Ok(())
}
#[derive(Clone, Debug)]
struct Pair {
    low: Vec<Population>,
    high: Vec<Population>,
    baseline_error: u128,
    full_error: u128,
}
fn pair(q: i64, count: usize, coupling: i64) -> Result<Pair, String> {
    let mut low = vec![start(q, count, false)];
    let mut high = vec![start(q, count, true)];
    let mut markov_error = 0_u128;
    let mut matched_error = 0_u128;
    for _ in 0..TICKS {
        let low_before = low.last().ok_or("missing low history state")?;
        let high_before = high.last().ok_or("missing high history state")?;
        let low_after = step(low_before, coupling)?;
        let high_after = step(high_before, coupling)?;
        for (old, new) in [(low_before, &low_after), (high_before, &high_after)] {
            let markov_delta = new.q - predict_markov(old.q);
            let matched_delta = new.q - predict_with_hidden(old, coupling)?;
            markov_error += (markov_delta * markov_delta) as u128;
            matched_error += (matched_delta * matched_delta) as u128;
        }
        low.push(low_after);
        high.push(high_after);
    }
    Ok(Pair {
        low,
        high,
        baseline_error: markov_error,
        full_error: matched_error,
    })
}
fn source_valid() -> Result<String, String> {
    let expected = "HBIHEADER|schema=pnp.prl.tls.source.v1|scope=PUBLIC_SCIENTIFIC_REFERENCE|seal=HBI-HBP-SHA-SH-HASH|json=0\nPRL_SOURCE|doi=10.1103/j1r2-5ph7|title=Probing_the_memory_of_a_superconducting_qubit_environment|journal=Physical_Review_Letters|accepted=2026-10-28|url=https://journals.aps.org/prl/accepted/10.1103/j1r2-5ph7|model=SOLOMON_POPULATION_RATES|experiment=NON_POISSONIAN_JUMP_TRACES|json=0\nPRL_SCOPE|physics=LONG_LIVED_TLS_MEMORY|simulation=CLASSICAL_FIXED_POINT_EULER|hardware=NOT_MEASURED|experimental_fitting=NOT_PERFORMED|quantum_coherence=NOT_INFERRED|source_content_sha256=NOT_PINNED|json=0\n";
    if SOURCE != expected {
        return Err("frozen PRL DOI/source statement changed".to_owned());
    }
    Ok(hash_bytes(SOURCE.as_bytes()))
}
type Bundle = (String, String, String, Vec<u8>);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Statistics {
    pub rows: usize,
    pub paired_histories: usize,
    pub lag_cases: usize,
    pub control_passes: usize,
    pub coupled_separations: usize,
    pub heldout_baseline_error: u128,
    pub matched_error: u128,
    pub trace_bytes: usize,
}
fn make_bundle() -> Result<(Bundle, Statistics), String> {
    let paper_sha = source_valid()?;
    let mut hbp = HBP_HEADER.to_owned();
    let mut hbi = HBI_HEADER.to_owned();
    let mut trace = b"PNP-TLS-TRACE-v1\0".to_vec();
    let mut prev = hash_bytes(b"PNP-TLS-SOLOMON-INTEGER-CLASSICAL-v1");
    let mut seq = 0_usize;
    let mut append = |kind: &str, id: &str, body: &str| {
        seq += 1;
        let data = format!(
            "{kind}|seq={seq}|id={id}|prl_source_sha256={paper_sha}|{body}|prev_hash={prev}"
        );
        let receipt = hash_bytes(data.as_bytes());
        let chain = hash_bytes(format!("{prev}|{receipt}").as_bytes());
        let row = format!("{data}|receipt_sha={receipt}|hash={chain}|json=0\n");
        hbi.push_str(&format!(
            "HBI|seq={seq}|id={id}|offset={}|len={}|hbp_sha256={}|sh={}|hash={chain}|json=0\n",
            hbp.len(), row.len(), hash_bytes(row.as_bytes()), &receipt[..16]
        ));
        hbp.push_str(&row);
        prev = chain;
    };
    append(
        "TLS_SOURCE", "prl",
        "doi=10.1103/j1r2-5ph7|physics=MEASURED_NON_POISSONIAN_QUBIT_JUMPS|simulation=CLASSICAL_RATE_EULER_PROXY|units=Q10000|bath=2000|gamma_q=900|gamma_tls_bath=150|coupling_on=1200|rate_fit=NONE|input=FROZEN_SYNTHETIC|model_oracle=SELF_GENERATOR",
    );
    let mut pairs = 0_usize;
    let mut lags = 0_usize;
    let mut passed_null = 0_usize;
    let mut coupled_separation = 0_usize;
    let mut heldout_error = 0_u128;
    let mut matched_error = 0_u128;
    for cells in CELLS {
        for enabled in [false, true] {
            let coupling = total_coupling(cells, enabled);
            let mode = if enabled { "coupled" } else { "null" };
            let model_id = format!("n{cells}_{mode}");
            append("TLS_MODEL", &model_id, &format!(
                "tls_cells={cells}|nominal_coupling={}|effective_coupling={coupling}|qubit_bath=900|tls_bath=150|scale=10000|time_step=DISCRETE_EULER|explicit_tls_state_bytes={}|memory_enabled={}",
                if enabled { COUPLED } else { 0 },
                cells * 2,
                u8::from(coupling > 0)
            ));
            for (qidx, &q) in START_Q.iter().enumerate() {
                let history = pair(q, cells, coupling)?;
                let split = if qidx == 0 { "DEVELOPMENT" } else { "HELDOUT" };
                let low_next = history.low[1].q;
                let high_next = history.high[1].q;
                let diff = high_next - low_next;
                if coupling == 0 {
                    if diff != 0 || history.baseline_error != 0 || history.full_error != 0 {
                        return Err("false memory found in the Markovian negative control".to_owned());
                    }
                    passed_null += 1;
                } else {
                    if diff <= 0 || history.baseline_error == 0 || history.full_error != 0 {
                        return Err("coupled TLS two-history separation missing".to_owned());
                    }
                    coupled_separation += 1;
                }
                if split == "HELDOUT" {
                    heldout_error += history.baseline_error;
                    matched_error += history.full_error;
                }
                let start_offset = trace.len();
                trace.extend_from_slice(&[cells as u8, u8::from(enabled), qidx as u8, 2]);
                trace.push((TICKS + 1) as u8);
                for states in [&history.low, &history.high] {
                    for pop in states {
                        snapshot(&mut trace, pop)?;
                    }
                }
                let length = trace.len() - start_offset;
                let pair_id = format!("n{cells}_{mode}_q{q}");
                append("TLS_PAIR", &pair_id, &format!(
                    "split={split}|tls_cells={cells}|coupling={coupling}|visible_start={q}|hidden_start_low={}|hidden_start_high={}|first_next_low={low_next}|first_next_high={high_next}|first_lag_difference={diff}|qubit_only_squared_error={}|matched_latent_squared_error={}|ticks={TICKS}|trace_offset={start_offset}|trace_len={length}|trace_sha256={}|exact_population_replay=PASS|source_data=SIMULATED",
                    if cells == 0 { "ABSENT" } else { "500" },
                    if cells == 0 { "ABSENT" } else { "9500" },
                    history.baseline_error, history.full_error,
                    hash_bytes(&trace[start_offset..])
                ));
                pairs += 1;
                for lag in LAGS {
                    let diff_lag = history.high[lag].q - history.low[lag].q;
                    if coupling == 0 && diff_lag != 0 {
                        return Err("Markovian null lag signature was contaminated".to_owned());
                    }
                    append("TLS_LAG", &format!("{pair_id}_lag{lag}"), &format!(
                        "split={split}|tls_cells={cells}|coupling={coupling}|lag={lag}|q_low={}|q_high={}|signed_difference={diff_lag}|comparison=CONDITIONAL_HISTORY_DIFFERENCE|g2_measured=NO",
                        history.low[lag].q, history.high[lag].q
                    ));
                    lags += 1;
                }
            }
        }
    }
    let gc = [
        ("summary_committed", Retention::SummaryOnly, true, false, false, true),
        ("exact_missing", Retention::ExactReplayRequired, true, false, false, false),
        ("exact_verified", Retention::ExactReplayRequired, true, true, true, true),
        ("uncommitted", Retention::SummaryOnly, false, true, true, false),
    ];
    for (id, kind, committed, retained, replay, expected) in gc {
        let eligible = may_discard(kind, committed, retained, replay);
        if eligible != expected {
            return Err("GULP GC latent-state retention gate disagreed".to_owned());
        }
        append("TLS_GC", id, &format!(
            "scope={}|summary_committed={}|needed_state_retained={}|independent_replay={}|eligible_to_reclaim={}|manual_approval=NOT_REQUIRED|source_archive=IMMUTABLE|disposal_execution=NOT_IN_THIS_GATE",
            if kind == Retention::SummaryOnly { "SUMMARY_ONLY" } else { "EXACT_REPLAY_REQUIRED" },
            u8::from(committed), u8::from(retained), u8::from(replay),
            u8::from(eligible)
        ));
    }
    append("TLS_SUMMARY", "fixedpoint_acceptance", &format!(
        "model_configs=10|paired_histories={pairs}|lag_cases={lags}|markov_null_controls={passed_null}|coupled_two_history_separations={coupled_separation}|heldout_baseline_squared_error={heldout_error}|heldout_matched_squared_error={matched_error}|trace_bytes={}|complete_state_trace_sha256={}|model=CLASSICAL_FIXED_POINT|oracle_knows_generator=YES|physics_fit=NO|quantum_hardware=NO|statistical_predictive_gain=NOT_CLAIMED|gc_policy=AUTOMATIC_SOURCE_SCOPED",
        trace.len(), hash_bytes(&trace)
    ));
    if seq != ROWS || pairs != 30 || lags != 150
        || passed_null != 18 || coupled_separation != 12
        || heldout_error == 0 || matched_error != 0
    {
        return Err("frozen TLS matrix acceptance totals changed".to_owned());
    }
    let anchor_data = format!(
        "TLS_ANCHOR|rows={ROWS}|pairs={pairs}|lags={lags}|source_sha256={paper_sha}|trace_sha256={}|trace_bytes={}|hbp_sha256={}|hbi_sha256={}|terminal_hash={prev}|seal=HBI-HBP-SHA-SH-HASH",
        hash_bytes(&trace), trace.len(), hash_bytes(hbp.as_bytes()), hash_bytes(hbi.as_bytes())
    );
    let anchor = format!(
        "{anchor_data}|checksum={}|json=0\n",
        hash_bytes(anchor_data.as_bytes())
    );
    let stats = Statistics {
        rows: ROWS,
        paired_histories: pairs,
        lag_cases: lags,
        control_passes: passed_null,
        coupled_separations: coupled_separation,
        heldout_baseline_error: heldout_error,
        matched_error,
        trace_bytes: trace.len(),
    };
    Ok(((hbp, hbi, anchor, trace), stats))
}
pub fn attest(out: &Path) -> Result<Statistics, String> {
    let ((hbp, hbi, anchor, trace), stats) = make_bundle()?;
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    for (name, data) in [
        ("PNP_TLS.hbp", hbp.as_bytes()),
        ("PNP_TLS.hbi", hbi.as_bytes()),
        ("PNP_TLS.anchor", anchor.as_bytes()),
        ("PNP_TLS_TRACE.bin", trace.as_slice()),
    ] {
        fs::write(out.join(name), data).map_err(|e| format!("TLS archive: {e}"))?;
    }
    Ok(stats)
}
pub fn verify(out: &Path) -> Result<Statistics, String> {
    let ((hbp, hbi, anchor, trace), stats) = make_bundle()?;
    for (name, expected) in [
        ("PNP_TLS.hbp", hbp.as_bytes()),
        ("PNP_TLS.hbi", hbi.as_bytes()),
        ("PNP_TLS.anchor", anchor.as_bytes()),
        ("PNP_TLS_TRACE.bin", trace.as_slice()),
    ] {
        if fs::read(out.join(name))
            .map_err(|e| format!("missing TLS archive {name}: {e}"))? != expected
        {
            return Err(format!("TLS source or trace receipt changed: {name}"));
        }
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_histories_same_visible_state_separate_only_with_latent_coupling() {
        for cells in CELLS {
            let null = pair(5_000, cells, 0).unwrap();
            assert_eq!(null.low[1].q, null.high[1].q);
            assert_eq!(null.baseline_error, 0);
            if cells > 0 {
                let coupled = pair(5_000, cells, COUPLED).unwrap();
                assert!(coupled.low[1].q < coupled.high[1].q);
                assert!(coupled.baseline_error > 0);
                assert_eq!(coupled.full_error, 0);
            }
        }
    }
    #[test]
    fn shuffled_hidden_reservoir_reverses_conditioned_return() {
        for cells in [1_usize, 3, 27, 81] {
            let low = start(5_000, cells, false);
            let high = start(5_000, cells, true);
            assert_eq!(low.q, high.q);
            let expected_low = predict_with_hidden(&low, COUPLED).unwrap();
            let expected_high = predict_with_hidden(&high, COUPLED).unwrap();
            assert_eq!(expected_low, step(&low, COUPLED).unwrap().q);
            assert_eq!(expected_high, step(&high, COUPLED).unwrap().q);
            assert!(expected_high > expected_low);
            // Rebinding the history rather than the visible state swaps
            // the predicted return: a key that forgets hidden state fails.
            let wrong_low = predict_with_hidden(&high, COUPLED).unwrap();
            let wrong_high = predict_with_hidden(&low, COUPLED).unwrap();
            assert_eq!(wrong_low, expected_high);
            assert_eq!(wrong_high, expected_low);
            assert_ne!(wrong_low, expected_low);
            assert_eq!(predict_markov(low.q), predict_markov(high.q));
        }
    }
    #[test]
    fn fixedpoint_rates_preserve_population_bounds_and_canonical_cell_counts() {
        for cells in CELLS {
            for q in START_Q {
                let high = pair(q, cells, total_coupling(cells, true)).unwrap();
                assert_eq!(high.low.len(), TICKS+1);
                assert_eq!(high.high.len(), TICKS+1);
                assert!(high.low.iter().all(|p| check_population(p).is_ok()));
                assert!(high.high.iter().all(|p| check_population(p).is_ok()));
            }
        }
    }
    #[test]
    fn memory_gc_retains_full_state_when_reconstruction_required() {
        assert!(may_discard(Retention::SummaryOnly, true, false, false));
        assert!(!may_discard(Retention::ExactReplayRequired, true, false, true));
        assert!(!may_discard(Retention::ExactReplayRequired, true, true, false));
        assert!(may_discard(Retention::ExactReplayRequired, true, true, true));
        assert!(!may_discard(Retention::SummaryOnly, false, true, true));
    }
    #[test]
    fn source_bound_receipts_are_repeatable_and_reject_false_memory() {
        let ((hbp, hbi, anchor, trace), stats) = make_bundle().unwrap();
        assert_eq!(stats.rows, 196);
        assert_eq!(stats.control_passes, 18);
        assert_eq!(stats.coupled_separations, 12);
        assert_eq!(hbp.lines().filter(|s| s.starts_with("TLS_PAIR|")).count(),30);
        assert_eq!(hbp.lines().filter(|s| s.starts_with("TLS_LAG|")).count(),150);
        assert_eq!(hbi.lines().filter(|s| s.starts_with("HBI|")).count(),196);
        assert!(anchor.contains("seal=HBI-HBP-SHA-SH-HASH"));
        assert!(trace.len()>1000);
        assert!(stats.heldout_baseline_error>0);
        assert_eq!(stats.matched_error,0);
    }
}
