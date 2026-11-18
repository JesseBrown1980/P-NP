//! Three independent roles: worker, reflection, and the matrix as fabric.
//! A fail-closed 3-ary N-Nest cascade processes each frozen room and its
//! exact 2D projection. No trained GNN/FNN, live hotels, or solver is claimed.

use crate::freeze::FrozenMatrix;
use crate::matrix::CELLS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Seat {
    Worker,
    Reflection,
    MatrixFabric,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fault {
    pub level: usize,
    pub seat: Seat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Verdict {
    pub accepted: bool,
    pub nodes: usize,
    pub fabric_reads: usize,
    pub worker_total: i128,
    pub reflection_total: i128,
    pub fabric_total: i128,
    pub mismatches: usize,
    pub held_for_operator: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaultMode {
    Unit,
    Balanced,
    Coordinated,
}

// Each seat produces an ordered report over every original (room, cell)
// address. Aggregates remain evidence metadata, not the acceptance criterion.
fn worker(frozen: &FrozenMatrix, start: usize, stop: usize) -> Option<Vec<i128>> {
    let mut values = Vec::with_capacity((stop - start).checked_mul(CELLS)?);
    for room in start..stop {
        for value in frozen.room(room)? {
            values.push(i128::from(*value));
        }
    }
    Some(values)
}

fn reflection(frozen: &FrozenMatrix, start: usize, stop: usize) -> Option<Vec<i128>> {
    let mut values = vec![0_i128; (stop - start).checked_mul(CELLS)?];
    for room in (start..stop).rev() {
        let original = frozen.room(room)?;
        for cell in (0..CELLS).rev() {
            values[(room - start) * CELLS + cell] = i128::from(original[cell]);
        }
    }
    Some(values)
}

fn fabric(frozen: &FrozenMatrix, start: usize, stop: usize) -> Option<Vec<i128>> {
    let mut values = vec![0_i128; (stop - start).checked_mul(CELLS)?];
    for room in start..stop {
        for column in 0..9 {
            // The matrix seat reads its separately frozen reversible
            // center-and-signed-difference witness, not source raw cells.
            let triple = frozen.reconstruct_triple(room, column)?;
            for (depth, value) in triple.iter().enumerate() {
                values[(room - start) * CELLS + column + 9 * depth] =
                    i128::from(*value);
            }
        }
    }
    Some(values)
}

fn distort(values: &mut [i128], mode: FaultMode) -> Option<()> {
    let first = values.get_mut(0)?;
    *first = first.checked_add(1)?;
    if mode == FaultMode::Balanced {
        // The same (x,y) shadow column, opposite z layer: both the
        // overall total and 2D projection sum remain identical.
        let second = values.get_mut(9)?;
        *second = second.checked_sub(1)?;
    }
    Some(())
}

fn inspect(
    frozen: &FrozenMatrix,
    start: usize,
    stop: usize,
    level: usize,
    fault: Option<Fault>,
    mode: FaultMode,
) -> Option<Verdict> {
    if start >= stop || (level == 0 && !frozen.verify()) {
        return None;
    }
    let mut children = Vec::new();
    if stop - start > 1 {
        if (stop - start) % 3 != 0 {
            return None;
        }
        let stride = (stop - start) / 3;
        for child in 0..3 {
            children.push(inspect(
                frozen, start + stride * child, start + stride * (child + 1),
                level + 1, fault, mode,
            )?);
        }
    }
    let mut work = worker(frozen, start, stop)?;
    let mut reflected = reflection(frozen, start, stop)?;
    let mut witnessed = fabric(frozen, start, stop)?;
    if start == 0 {
        if let Some(fault) = fault {
            if fault.level == level {
                match mode {
                    FaultMode::Coordinated => {
                        // Worker and reflection collude, but independent
                        // matrix reconstruction still rejects their report.
                        distort(&mut work, FaultMode::Unit)?;
                        distort(&mut reflected, FaultMode::Unit)?;
                    }
                    _ => {
                        let chosen = match fault.seat {
                            Seat::Worker => &mut work,
                            Seat::Reflection => &mut reflected,
                            Seat::MatrixFabric => &mut witnessed,
                        };
                        distort(chosen, mode)?;
                    }
                }
            }
        }
    }
    if work.len() != reflected.len() || work.len() != witnessed.len() {
        return None;
    }
    let local_mismatches = (0..work.len())
        .filter(|&index| {
            work[index] != reflected[index] || work[index] != witnessed[index]
        })
        .count();
    let work_total: i128 = work.iter().sum();
    let reflection_total: i128 = reflected.iter().sum();
    let fabric_total: i128 = witnessed.iter().sum();
    let consistent_children = children.is_empty()
        || (children.iter().map(|child| child.worker_total).sum::<i128>() == work_total
            && children.iter().map(|child| child.reflection_total).sum::<i128>()
                == reflection_total
            && children.iter().map(|child| child.fabric_total).sum::<i128>()
                == fabric_total);
    let child_agreed = children.iter().all(|child| child.accepted);
    let mismatches = local_mismatches + usize::from(!consistent_children)
        + children.iter().map(|child| child.mismatches).sum::<usize>();
    Some(Verdict {
        accepted: local_mismatches == 0 && child_agreed && consistent_children,
        nodes: 1 + children.iter().map(|child| child.nodes).sum::<usize>(),
        fabric_reads: 1 + children.iter().map(|child| child.fabric_reads)
            .sum::<usize>(),
        worker_total: work_total,
        reflection_total,
        fabric_total,
        mismatches,
        held_for_operator: true,
    })
}

pub fn wave_cascade(frozen: &FrozenMatrix, fault: Option<Fault>)
    -> Option<Verdict>
{
    if !matches!(frozen.room_count(), 3 | 9 | 27 | 81) {
        return None;
    }
    inspect(frozen, 0, frozen.room_count(), 0, fault, FaultMode::Unit)
}

/// Report a cancelling +1/-1 edit within the same shadow column.
/// Aggregate-only comparators cannot detect this fault.
pub fn wave_cascade_balanced(
    frozen: &FrozenMatrix, fault: Fault,
) -> Option<Verdict> {
    if !matches!(frozen.room_count(), 3 | 9 | 27 | 81) {
        return None;
    }
    inspect(frozen, 0, frozen.room_count(), 0, Some(fault), FaultMode::Balanced)
}

/// Both source-reading seats agree on a false cell; the independently
/// reconstructed third seat must reject the coordinated report.
pub fn wave_cascade_coordinated(
    frozen: &FrozenMatrix, level: usize,
) -> Option<Verdict> {
    if !matches!(frozen.room_count(), 3 | 9 | 27 | 81) {
        return None;
    }
    inspect(
        frozen, 0, frozen.room_count(), 0,
        Some(Fault { level, seat: Seat::Worker }), FaultMode::Coordinated,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::Matrix;

    #[test]
    fn whole_frozen_matrix_is_third_agent_on_all_four_nested_levels() {
        for n in [3, 9, 27, 81] {
            let source = Matrix::generated(n).expect("matrix");
            let frozen = FrozenMatrix::freeze(&source);
            let result = wave_cascade(&frozen, None).expect("cascade");
            assert!(result.accepted);
            assert!(result.held_for_operator);
            assert_eq!(result.mismatches, 0);
            assert_eq!(result.fabric_reads, result.nodes);
            assert_eq!(result.worker_total, result.reflection_total);
            assert_eq!(result.worker_total, result.fabric_total);
            let mut power = n;
            let mut count = 0;
            while power >= 1 {
                count += power;
                power /= 3;
            }
            assert_eq!(result.nodes, count);
        }
    }

    #[test]
    fn all_seats_fail_closed_at_every_supervisor_depth() {
        let mut planted = 0;
        for (n, max_level) in [(3, 1), (9, 2), (27, 3), (81, 4)] {
            let source = Matrix::generated(n).expect("matrix");
            let frozen = FrozenMatrix::freeze(&source);
            for level in 0..=max_level {
                for seat in [Seat::Worker, Seat::Reflection, Seat::MatrixFabric] {
                    let bad = wave_cascade(&frozen, Some(Fault { level, seat }))
                        .expect("bad trace still produces verdict");
                    assert!(!bad.accepted);
                    assert!(bad.mismatches >= 1);
                    assert!(bad.held_for_operator);
                    planted += 1;
                }
            }
        }
        assert_eq!(planted, 42);
    }

    #[test]
    fn balanced_faults_rejected_with_unchanged_total_and_shadow() {
        let mut planted = 0;
        for (rooms, height) in [(3, 1), (9, 2), (27, 3), (81, 4)] {
            let source = Matrix::generated(rooms).expect("matrix");
            let frozen = FrozenMatrix::freeze(&source);
            for level in 0..=height {
                for seat in [Seat::Worker, Seat::Reflection, Seat::MatrixFabric] {
                    let bad = wave_cascade_balanced(&frozen, Fault { level, seat })
                        .expect("balanced tamper verdict");
                    assert!(!bad.accepted);
                    assert!(bad.mismatches >= 2);
                    // All three reported aggregate sums still agree.
                    assert_eq!(bad.worker_total, bad.reflection_total);
                    assert_eq!(bad.worker_total, bad.fabric_total);
                    planted += 1;
                }
            }
        }
        assert_eq!(planted, 42);
    }

    #[test]
    fn coordinated_worker_reflection_reports_rejected_by_fabric() {
        let mut planted = 0;
        for (rooms, height) in [(3, 1), (9, 2), (27, 3), (81, 4)] {
            let source = Matrix::generated(rooms).expect("matrix");
            let frozen = FrozenMatrix::freeze(&source);
            for level in 0..=height {
                let bad = wave_cascade_coordinated(&frozen, level)
                    .expect("coordinated tamper verdict");
                assert!(!bad.accepted);
                assert!(bad.mismatches >= 1);
                planted += 1;
            }
        }
        assert_eq!(planted, 14);
    }
}
