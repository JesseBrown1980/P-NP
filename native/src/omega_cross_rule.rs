//! A bounded, exact cross-rule prediction gate using the original Omega RNQ
//! catalog, Relic free-centre readout, and a declared synthetic transition.
//! Five reversible views are training presentations, not five independent sources.
#![forbid(unsafe_code)]

use crate::{omega_operator_catalog as omega, relic};

type State = [u8; 3];
type Coefficients = [[u8; 10]; 3];

const TRAIN_VIEWS: [u8; 5] = [0, 1, 2, 3, 4];
const WITHHELD_VIEW: u8 = 5;
// Independently enumerated outputs of the seven omitted source states.
// These values are fixture witnesses and are never passed to infer_rule.
const WITHHELD_REFERENCE: [State; 7] = [
    [1, 2, 0], [1, 2, 1], [2, 0, 0], [2, 1, 2],
    [0, 2, 0], [0, 2, 2], [1, 0, 2],
];
const ORACLE_COEFFICIENTS: Coefficients = [
    [1, 2, 0, 1, 1, 2, 0, 2, 1, 0],
    [2, 1, 1, 0, 0, 2, 1, 1, 0, 2],
    [0, 2, 1, 1, 2, 0, 1, 0, 2, 1],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observation {
    pub view: u8,
    pub input: [u16; 3],
    pub output: [u16; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GateReport {
    pub training_states: usize,
    pub training_views: usize,
    pub observed_pairs: usize,
    pub withheld_states: usize,
    pub withheld_correct: usize,
    pub operator_portability_checks: usize,
    pub raw_view_symbol_bytes: usize,
    pub distinct_canonical_symbol_bytes: usize,
    pub coefficient_trits: usize,
    pub independent_view_information_gain: usize,
}

fn decode_ordinal(mut ordinal: u8) -> Result<State, String> {
    if ordinal >= 27 {
        return Err("ternary triple address outside 27".to_owned());
    }
    let mut result = [0_u8; 3];
    for index in (0..3).rev() {
        result[index] = ordinal % 3;
        ordinal /= 3;
    }
    Ok(result)
}

fn basis(x: State) -> [u8; 10] {
    let [a, b, c] = x;
    [
        1, a, b, c, a * a % 3, b * b % 3, c * c % 3,
        a * b % 3, a * c % 3, b * c % 3,
    ]
}

fn transition(x: State, coefficients: &Coefficients) -> State {
    let phi = basis(x);
    std::array::from_fn(|component| {
        coefficients[component].iter().zip(phi)
            .map(|(&weight, value)| u16::from(weight) * u16::from(value))
            .sum::<u16>() as u8 % 3
    })
}

fn view_encode(id: u8, value: State) -> Result<[u16; 3], String> {
    let symbols = value.map(u16::from);
    let transformed = omega::apply_symbols(id, 8, &symbols)
        .map_err(|error| error.to_string())?;
    transformed.try_into()
        .map_err(|_| "Omega view changed triple length".to_owned())
}

fn view_decode(id: u8, value: [u16; 3]) -> Result<State, String> {
    let original = omega::invert_symbols(id, 8, &value)
        .map_err(|error| error.to_string())?;
    let decoded: [u16; 3] = original.try_into()
        .map_err(|_| "Omega inverse changed triple length".to_owned())?;
    let mut state = [0_u8; 3];
    for (index, word) in decoded.into_iter().enumerate() {
        if word > 2 {
            return Err("inverse produced a non-trit word".to_owned());
        }
        state[index] = u8::try_from(word).map_err(|_| "invalid trit")?;
    }
    Ok(state)
}

fn observe(id: u8, input: State, output: State) -> Result<Observation, String> {
    Ok(Observation {
        view: id,
        input: view_encode(id, input)?,
        output: view_encode(id, output)?,
    })
}

/// Learn one quadratic rule over F_3. Rank and all source observations are
/// checked; inconsistent observations fail rather than inventing a rule.
fn infer_rule(observations: &[Observation]) -> Result<Coefficients, String> {
    let mut rows = Vec::<[u8; 13]>::with_capacity(observations.len());
    for observation in observations {
        let x = view_decode(observation.view, observation.input)?;
        let y = view_decode(observation.view, observation.output)?;
        let mut row = [0_u8; 13];
        row[..10].copy_from_slice(&basis(x));
        row[10..].copy_from_slice(&y);
        rows.push(row);
    }
    if rows.len() < 10 {
        return Err("too few independent input states".to_owned());
    }
    for column in 0..10 {
        let pivot = (column..rows.len())
            .find(|&r| rows[r][column] != 0)
            .ok_or("rank-deficient rule observation matrix")?;
        rows.swap(column, pivot);
        let inverse = if rows[column][column] == 1 { 1 } else { 2 };
        for j in column..13 {
            rows[column][j] = rows[column][j] * inverse % 3;
        }
        for r in 0..rows.len() {
            if r == column {
                continue;
            }
            let factor = rows[r][column];
            if factor == 0 {
                continue;
            }
            for j in column..13 {
                let subtract = factor * rows[column][j] % 3;
                rows[r][j] = (rows[r][j] + 3 - subtract) % 3;
            }
        }
    }
    let mut coefficients = [[0_u8; 10]; 3];
    for (output, coefficient_row) in coefficients.iter_mut().enumerate() {
        for (column, coefficient) in coefficient_row.iter_mut().enumerate() {
            *coefficient = rows[column][10 + output];
        }
    }
    for observation in observations {
        let x = view_decode(observation.view, observation.input)?;
        let y = view_decode(observation.view, observation.output)?;
        if transition(x, &coefficients) != y {
            return Err("inconsistent cross-view transition observation".to_owned());
        }
    }
    Ok(coefficients)
}

fn training_observations() -> Result<Vec<Observation>, String> {
    let mut samples = Vec::with_capacity(100);
    for ordinal in 0..27_u8 {
        if ordinal % 4 == 0 {
            continue; // Seven whole source states are withheld, across every view.
        }
        let x = decode_ordinal(ordinal)?;
        let y = transition(x, &ORACLE_COEFFICIENTS);
        for &view in &TRAIN_VIEWS {
            samples.push(observe(view, x, y)?);
        }
    }
    Ok(samples)
}

/// Source-scoped fixture. The predicted sixth output is computed before the
/// withheld oracle result is consulted. Other 34 operators are portability
/// checks, not extra observations used to fit the law.
pub fn run_gate() -> Result<GateReport, String> {
    if relic::anchor(0) != Some(27)
        || relic::anchor(1) != Some(243)
        || relic::center_at_z(crate::ternary::Trit::Zero) != (0, 1)
    {
        return Err("Relic source invariant changed".to_owned());
    }
    let training = training_observations()?;
    let fitted = infer_rule(&training)?;
    let one_view: Vec<_> = training.iter().copied()
        .filter(|observation| observation.view == 0).collect();
    let fitted_from_one = infer_rule(&one_view)?;
    if fitted != fitted_from_one || fitted != ORACLE_COEFFICIENTS {
        return Err("five views changed the rule inferred from one view".to_owned());
    }

    let mut withheld_correct = 0;
    let mut portability = 0;
    for ordinal in 0..27_u8 {
        if ordinal % 4 != 0 {
            continue;
        }
        let original = decode_ordinal(ordinal)?;
        let view_input = view_encode(WITHHELD_VIEW, original)?;
        let source_recovered = view_decode(WITHHELD_VIEW, view_input)?;
        let predicted = view_encode(
            WITHHELD_VIEW, transition(source_recovered, &fitted),
        )?;
        // The withheld target is generated only after the prediction exists.
        let expected = view_encode(WITHHELD_VIEW, WITHHELD_REFERENCE[withheld_correct])?;
        if predicted != expected {
            return Err("held-out sixth-view prediction differed".to_owned());
        }

        for id in 0..omega::OPERATOR_COUNT {
            let id = u8::try_from(id).map_err(|_| "catalog ID overflow")?;
            let encoded_input = view_encode(id, original)?;
            let recovered = view_decode(id, encoded_input)?;
            if recovered != original {
                return Err("catalog inverse failed on hidden input".to_owned());
            }
            let output = view_encode(id, transition(recovered, &fitted))?;
            let actual = view_encode(id, WITHHELD_REFERENCE[withheld_correct])?;
            let restored = view_decode(id, output)?;
            if output != actual || restored != WITHHELD_REFERENCE[withheld_correct] {
                return Err(format!(
                    "40-operator held-out transport mismatch: ordinal={ordinal} id={id} predicted={output:?} target={actual:?} restored={restored:?} source_target={:?}",
                    WITHHELD_REFERENCE[withheld_correct],
                ));
            }
            portability += 1;
        }
        withheld_correct += 1;

        let raw = original.map(i64::from);
        let frozen = relic::freeze_triple(raw);
        if relic::rebuild_triple(frozen) != Some(raw) {
            return Err("Relic common-mode/difference reconstruction failed".to_owned());
        }
    }
    if withheld_correct != 7 || portability != 280 {
        return Err("withheld-state census incomplete".to_owned());
    }
    // This is only a *raw symbol payload lower bound*, before framing,
    // view labels, code, model, witnesses, stored roots, and CPU costs.
    // Training has 20 source states; each has 5 views and 2 triples.
    Ok(GateReport {
        training_states: 20,
        training_views: TRAIN_VIEWS.len(),
        observed_pairs: training.len(),
        withheld_states: 7,
        withheld_correct,
        operator_portability_checks: portability,
        raw_view_symbol_bytes: training.len() * 2 * 3,
        distinct_canonical_symbol_bytes: 20 * 2 * 3,
        coefficient_trits: 3 * 10,
        independent_view_information_gain: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_views_infer_the_same_rule_as_one_and_predict_sixth() {
        let report = run_gate().expect("exact held-out native test");
        assert_eq!(report.training_states, 20);
        assert_eq!(report.observed_pairs, 100);
        assert_eq!(report.withheld_correct, 7);
        assert_eq!(report.operator_portability_checks, 280);
        assert_eq!(report.raw_view_symbol_bytes, 600);
        assert_eq!(report.distinct_canonical_symbol_bytes, 120);
        assert_eq!(report.coefficient_trits, 30);
        assert_eq!(report.independent_view_information_gain, 0);
    }

    #[test]
    fn tampered_view_output_fails_even_when_four_other_views_agree() {
        let mut sample = training_observations().expect("canonical observations");
        let original = view_decode(sample[0].view, sample[0].output)
            .expect("output");
        let mut changed = original;
        changed[0] = (changed[0] + 1) % 3;
        sample[0].output = view_encode(sample[0].view, changed)
            .expect("valid but forged view");
        assert!(infer_rule(&sample).is_err());
    }

    #[test]
    fn common_mode_cancels_distinctions_but_relic_delta_restores_them() {
        let a = [0_i64, 0, 1];
        let b = [0_i64, 1, 0];
        assert_eq!(a.iter().sum::<i64>(), b.iter().sum::<i64>());
        let fa = relic::freeze_triple(a);
        let fb = relic::freeze_triple(b);
        assert_ne!(fa, fb);
        assert_eq!(relic::rebuild_triple(fa), Some(a));
        assert_eq!(relic::rebuild_triple(fb), Some(b));
        assert_ne!(transition([0, 0, 1], &ORACLE_COEFFICIENTS),
                   transition([0, 1, 0], &ORACLE_COEFFICIENTS));
    }

    #[test]
    fn incomplete_rank_bad_symbols_and_unknown_operators_fail_closed() {
        let sample = training_observations().expect("observations");
        assert!(infer_rule(&sample[..5]).is_err());
        assert!(view_decode(0, [3, 0, 1]).is_err());
        assert!(view_encode(40, [0, 0, 0]).is_err());
        assert!(view_decode(40, [0, 0, 0]).is_err());
    }
}
