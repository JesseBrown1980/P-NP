//! Native 27-symbol blind behavioral language discovery, using genuine
//! state-transition observations and a source-held-out third-seat referee.
//! Classical, synthetic and exact: no inference of deployed encryption keys.
#![forbid(unsafe_code)]

use crate::{omega_operator_catalog as omega, relic};
use std::collections::VecDeque;

const LANGUAGES: [u8; 3] = [0, 2, 7];
const SEEDS: [u64; 3] = [
    0x59a8_f345_23c7_1001,
    0x7712_beda_11c4_0309,
    0x0381_bdaa_9934_7f21,
];
const ALPHABET: usize = 27;
const ACTIONS: usize = 3;
const TREE_OBSERVATIONS: usize = 25;
const ALL_TRANSITIONS: usize = ALPHABET * ACTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservedTransition {
    /// Opaque label seen at the source, never a canonical ternary coordinate.
    pub from: u8,
    /// Public action name: increment the indicated ternary coordinate.
    pub action: u8,
    /// Opaque label observed after the independent action.
    pub to: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Learned {
    /// Canonical ternary coordinate -> unknown visible source label.
    labels: [Option<u8>; ALPHABET],
    known_labels: usize,
    compatible_bijections: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Report {
    pub independent_oracle_fixtures: usize,
    pub initial_behavioral_edges_each: usize,
    pub additional_independent_edges_each: usize,
    pub derived_views_per_edge: usize,
    pub completions_initial: usize,
    pub completions_after_first: usize,
    pub completions_after_second: usize,
    pub uniquely_identified_symbols_each: usize,
    pub source_heldout_transitions: usize,
    pub source_heldout_correct: usize,
    pub direct_edge_observations: usize,
    pub brown_edge_observations: usize,
    pub direct_endpoint_symbols: usize,
    pub three_view_endpoint_symbols: usize,
    pub extra_independent_source_information_from_views: usize,
    pub brown_cancellation_reconstructions: usize,
}

fn digits(value: u8) -> [u8; 3] {
    [value / 9, (value / 3) % 3, value % 3]
}

fn ordinal([a, b, c]: [u8; 3]) -> u8 {
    a * 9 + b * 3 + c
}

fn act(value: u8, axis: u8) -> Result<u8, String> {
    let mut triple = digits(value);
    let coordinate = triple.get_mut(usize::from(axis))
        .ok_or("unknown ternary transition generator")?;
    *coordinate = (*coordinate + 1) % 3;
    Ok(ordinal(triple))
}

/// Private to the fixture/oracle generator. The learner receives only the
/// measured opaque labels/edge observations, never this mapping or seed.
fn hidden_labels(mut seed: u64) -> [u8; ALPHABET] {
    let mut labels = std::array::from_fn(|index| index as u8);
    for last in (1..ALPHABET).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let other = (seed % (last as u64 + 1)) as usize;
        labels.swap(last, other);
    }
    labels
}

/// Source oracle emits 26 graph-tree edges and keeps the entire set of 81
/// possible transitions for the independent post-fit referee. The 26th edge
/// and the other 55 oracle transitions remain outside the learner's 25 edges.
fn oracle_edges(hidden: &[u8; ALPHABET]) -> Result<Vec<ObservedTransition>, String> {
    let mut queue = VecDeque::from([0_u8]);
    let mut discovered = [false; ALPHABET];
    discovered[0] = true;
    let mut tree = Vec::with_capacity(ALPHABET - 1);
    while let Some(source) = queue.pop_front() {
        for axis in 0..ACTIONS {
            let axis = u8::try_from(axis).map_err(|_| "action overflow")?;
            let target = act(source, axis)?;
            if !discovered[usize::from(target)] {
                discovered[usize::from(target)] = true;
                queue.push_back(target);
                tree.push(ObservedTransition {
                    from: hidden[usize::from(source)],
                    action: axis,
                    to: hidden[usize::from(target)],
                });
            }
        }
    }
    if tree.len() != ALPHABET - 1 || !discovered.iter().all(|value| *value) {
        return Err("oracle spanning-tree source did not cover 27 states".to_owned());
    }
    Ok(tree)
}

/// Pairs are copied into a derived alphabet, then inverted and checked.
fn transport(observation: ObservedTransition, view: u8)
    -> Result<ObservedTransition, String>
{
    let encoded = omega::apply_operator(view, &[observation.from, observation.to])
        .map_err(|error| error.to_string())?;
    let decoded = omega::invert_operator(view, &encoded)
        .map_err(|error| error.to_string())?;
    if decoded.as_slice() != [observation.from, observation.to] {
        return Err("RNQ transformed edge could not be reconstructed".to_owned());
    }
    Ok(ObservedTransition {
        from: decoded[0],
        action: observation.action,
        to: decoded[1],
    })
}

/// Learn with the exact same underlying observed source edges for all arms.
/// Reversible colored views add checking work but never new source edges.
fn infer(
    observations: &[ObservedTransition],
    origin_symbol: u8,
    colored_views: bool,
) -> Result<Learned, String> {
    if origin_symbol >= 27 {
        return Err("origin label not in the 27-symbol alphabet".to_owned());
    }
    let mut canonical_edges = Vec::with_capacity(observations.len());
    for &edge in observations {
        if edge.from >= 27 || edge.to >= 27 || edge.action >= 3 {
            return Err("malformed independent behavioral observation".to_owned());
        }
        if colored_views {
            for view in LANGUAGES {
                if transport(edge, view)? != edge {
                    return Err("derived language disagreed with originating observation".to_owned());
                }
            }
        }
        canonical_edges.push(edge);
    }

    let mut labels = [None; ALPHABET];
    let mut coordinates = [None; ALPHABET];
    labels[0] = Some(origin_symbol);
    coordinates[usize::from(origin_symbol)] = Some(0_u8);
    let mut frontier = VecDeque::from([0_u8]);

    while let Some(canonical) = frontier.pop_front() {
        let source = labels[usize::from(canonical)]
            .ok_or("frontier lacks discovered label")?;
        for edge in canonical_edges.iter().filter(|edge| edge.from == source) {
            let successor = act(canonical, edge.action)?;
            let target_index = usize::from(successor);
            let symbol_index = usize::from(edge.to);
            if let Some(previous) = labels[target_index] {
                if previous != edge.to {
                    return Err("inconsistent state-transition language".to_owned());
                }
            }
            if let Some(previous) = coordinates[symbol_index] {
                if previous != successor {
                    return Err("two canonical states collapsed onto one label".to_owned());
                }
            }
            if labels[target_index].is_none() {
                labels[target_index] = Some(edge.to);
                coordinates[symbol_index] = Some(successor);
                frontier.push_back(successor);
            }
        }
    }

    // Observations whose source was inaccessible from the anchor are held,
    // rather than silently treated as independent mapping constraints.
    for edge in canonical_edges {
        let original = coordinates[usize::from(edge.from)]
            .ok_or("observed source disconnected from the origin")?;
        let next = act(original, edge.action)?;
        if labels[usize::from(next)] != Some(edge.to) {
            return Err("observed action disagrees with inferred ternary function".to_owned());
        }
    }
    let known = labels.iter().filter(|symbol| symbol.is_some()).count();
    if known == ALPHABET - 1 {
        let missing_symbol = coordinates.iter().position(|state| state.is_none())
            .ok_or("expected one unused observed symbol")?;
        let missing_coordinate = labels.iter().position(|state| state.is_none())
            .ok_or("expected one undiscovered ternary address")?;
        labels[missing_coordinate] =
            Some(u8::try_from(missing_symbol).map_err(|_| "symbol overflow")?);
    }
    let residual = ALPHABET - known;
    let compatible_bijections = (1..=residual).product();
    let known_labels = labels.iter().filter(|symbol| symbol.is_some()).count();
    Ok(Learned {
        labels,
        known_labels,
        compatible_bijections,
    })
}

fn verify_black_brown_white(
    learned: &Learned,
    hidden: &[u8; ALPHABET],
    observed: &[ObservedTransition],
) -> Result<usize, String> {
    if learned.known_labels != ALPHABET || learned.compatible_bijections != 1 {
        return Err("unresolved alphabet presented for verification".to_owned());
    }
    let mut correct = 0_usize;
    for original in 0..ALPHABET {
        let original_u8 = u8::try_from(original).map_err(|_| "coordinate overflow")?;
        for axis in 0..ACTIONS {
            let axis = u8::try_from(axis).map_err(|_| "action overflow")?;
            let successor = act(original_u8, axis)?;
            let claimed = ObservedTransition {
                from: learned.labels[original].ok_or("unmapped source state")?,
                action: axis,
                to: learned.labels[usize::from(successor)]
                    .ok_or("unmapped target state")?,
            };
            if observed.contains(&claimed) {
                continue; // source training observations are excluded.
            }
            // Independent referee accesses withheld source oracle only here.
            let expected = ObservedTransition {
                from: hidden[original],
                action: axis,
                to: hidden[usize::from(successor)],
            };
            if claimed != expected {
                return Err("independent third seat rejected a held-out transition".to_owned());
            }
            correct += 1;
        }
    }
    if correct != ALL_TRANSITIONS - observed.len() {
        return Err("source-held-out transition census changed".to_owned());
    }
    Ok(correct)
}

fn verify_brown_signed_differences() -> Result<usize, String> {
    let mut reconstructed = 0;
    for address in 0..ALPHABET {
        let trit = digits(u8::try_from(address).map_err(|_| "address overflow")?);
        let signed = trit.map(|v| i64::from(v) - 1);
        let frozen = relic::freeze_triple(signed);
        if relic::rebuild_triple(frozen) != Some(signed) {
            return Err("signed Brown ternary witness reconstruction failed".to_owned());
        }
        reconstructed += 1;
    }
    // Opposed signed components cancel in the common mode but reconstruct.
    let x = [-1_i64, 0, 1];
    let frozen = relic::freeze_triple(x);
    if frozen.sum != 0 || relic::rebuild_triple(frozen) != Some(x) {
        return Err("Brown center/difference cancellation witness failed".to_owned());
    }
    Ok(reconstructed)
}

/// The independent information enters only through the two new genuine
/// oracle transitions. Colored RNQ leaves do not contribute new constraints.
pub fn run_gate() -> Result<Report, String> {
    let mut total_correct = 0;
    let mut total_direct_edges = 0;
    let mut total_brown_edges = 0;
    let mut brown_reconstructions = 0;
    for seed in SEEDS {
        let hidden = hidden_labels(seed);
        let tree = oracle_edges(&hidden)?;
        let origin = hidden[0];

        let initial_direct = infer(&tree[..23], origin, false)?;
        let initial_three = infer(&tree[..23], origin, true)?;
        if initial_direct != initial_three
            || initial_direct.known_labels != 24
            || initial_direct.compatible_bijections != 6
        {
            return Err("23 independently observed transitions did not leave six hypotheses".to_owned());
        }

        let one_extra = infer(&tree[..24], origin, true)?;
        let one_extra_direct = infer(&tree[..24], origin, false)?;
        if one_extra != one_extra_direct
            || one_extra.known_labels != 25
            || one_extra.compatible_bijections != 2
        {
            return Err("first independent measurement did not reduce hypotheses to two".to_owned());
        }

        let complete_direct = infer(&tree[..TREE_OBSERVATIONS], origin, false)?;
        let complete_three = infer(&tree[..TREE_OBSERVATIONS], origin, true)?;
        if complete_direct != complete_three
            || complete_three.compatible_bijections != 1
            || complete_three.known_labels != ALPHABET
        {
            return Err("second independent measurement did not identify entire alphabet".to_owned());
        }
        for (canonical, label) in complete_three.labels.iter().enumerate() {
            if *label != Some(hidden[canonical]) {
                return Err("recovered alphabet does not match hidden source mapping".to_owned());
            }
        }
        total_correct += verify_black_brown_white(
            &complete_three, &hidden, &tree[..TREE_OBSERVATIONS],
        )?;
        total_direct_edges += TREE_OBSERVATIONS;
        total_brown_edges += TREE_OBSERVATIONS;
        brown_reconstructions += verify_brown_signed_differences()?;
    }
    Ok(Report {
        independent_oracle_fixtures: SEEDS.len(),
        initial_behavioral_edges_each: 23,
        additional_independent_edges_each: 2,
        derived_views_per_edge: LANGUAGES.len(),
        completions_initial: 6,
        completions_after_first: 2,
        completions_after_second: 1,
        uniquely_identified_symbols_each: 27,
        source_heldout_transitions: SEEDS.len() * (ALL_TRANSITIONS - TREE_OBSERVATIONS),
        source_heldout_correct: total_correct,
        direct_edge_observations: total_direct_edges,
        brown_edge_observations: total_brown_edges,
        direct_endpoint_symbols: total_direct_edges * 2,
        three_view_endpoint_symbols: total_brown_edges * 2 * LANGUAGES.len(),
        extra_independent_source_information_from_views: 0,
        brown_cancellation_reconstructions: brown_reconstructions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_oracles_blind_heldout_and_matched_baselines_pass() {
        let report = run_gate().expect("three real behavioral input sources");
        assert_eq!(
            report,
            Report {
                independent_oracle_fixtures: 3,
                initial_behavioral_edges_each: 23,
                additional_independent_edges_each: 2,
                derived_views_per_edge: 3,
                completions_initial: 6,
                completions_after_first: 2,
                completions_after_second: 1,
                uniquely_identified_symbols_each: 27,
                source_heldout_transitions: 168,
                source_heldout_correct: 168,
                direct_edge_observations: 75,
                brown_edge_observations: 75,
                direct_endpoint_symbols: 150,
                three_view_endpoint_symbols: 450,
                extra_independent_source_information_from_views: 0,
                brown_cancellation_reconstructions: 81,
            },
        );
    }

    #[test]
    fn duplicated_colored_views_cannot_resolve_missing_oracle_edges() {
        for seed in SEEDS {
            let hidden = hidden_labels(seed);
            let tree = oracle_edges(&hidden).unwrap();
            let direct = infer(&tree[..23], hidden[0], false).unwrap();
            let derived = infer(&tree[..23], hidden[0], true).unwrap();
            assert_eq!(direct.compatible_bijections, 6);
            assert_eq!(direct, derived);
        }
    }

    #[test]
    fn independent_corrupted_action_or_duplicated_label_is_rejected() {
        let hidden = hidden_labels(SEEDS[0]);
        let mut tree = oracle_edges(&hidden).unwrap();
        tree[0].to = tree[0].from;
        assert!(infer(&tree[..TREE_OBSERVATIONS], hidden[0], true).is_err());

        let mut tree = oracle_edges(&hidden).unwrap();
        tree[0].action = 3;
        assert!(infer(&tree[..TREE_OBSERVATIONS], hidden[0], false).is_err());

        let tree = oracle_edges(&hidden).unwrap();
        let recovered = infer(&tree[..TREE_OBSERVATIONS], hidden[0], true).unwrap();
        let mut forged_hidden = hidden;
        forged_hidden.swap(0, 1);
        assert!(
            verify_black_brown_white(
                &recovered, &forged_hidden, &tree[..TREE_OBSERVATIONS],
            )
            .is_err()
        );
    }

    #[test]
    fn heldout_edges_are_excluded_and_all_three_actions_have_order_three() {
        let hidden = hidden_labels(SEEDS[1]);
        let tree = oracle_edges(&hidden).unwrap();
        let recovered = infer(&tree[..TREE_OBSERVATIONS], hidden[0], false).unwrap();
        assert_eq!(
            verify_black_brown_white(
                &recovered, &hidden, &tree[..TREE_OBSERVATIONS],
            )
            .unwrap(),
            56,
        );
        for source in 0..27_u8 {
            for axis in 0..3_u8 {
                let first = act(source, axis).unwrap();
                let second = act(first, axis).unwrap();
                let third = act(second, axis).unwrap();
                assert_eq!(source, third);
            }
            for left in 0..3_u8 {
                for right in 0..3_u8 {
                    assert_eq!(
                        act(act(source, left).unwrap(), right).unwrap(),
                        act(act(source, right).unwrap(), left).unwrap(),
                    );
                }
            }
        }
    }
}
