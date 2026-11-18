//! Three independent roles: worker, reflection, and the matrix as fabric.
//! A fail-closed 3-ary N-Nest cascade processes each frozen room and its
//! exact 2D projection. No trained GNN/FNN, live hotels, or solver is claimed.

use crate::freeze::FrozenMatrix;

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

fn worker(frozen: &FrozenMatrix, start: usize, stop: usize) -> Option<i128> {
    let mut total = 0_i128;
    for room in start..stop {
        for value in frozen.room(room)? {
            total += i128::from(*value);
        }
    }
    Some(total)
}

fn reflection(frozen: &FrozenMatrix, start: usize, stop: usize) -> Option<i128> {
    let mut total = 0_i128;
    for room in (start..stop).rev() {
        let source = frozen.room(room)?;
        for index in (0..source.len()).rev() {
            total += i128::from(source[index]);
        }
    }
    Some(total)
}

fn fabric(frozen: &FrozenMatrix, start: usize, stop: usize) -> Option<i128> {
    let mut total = 0_i128;
    for room in start..stop {
        let projection = frozen.projection(room)?;
        // The third seat consults the matrix's frozen 2D shadow.
        for x in 0..3 {
            for y in 0..3 {
                total += projection[x + 3 * y];
            }
        }
    }
    Some(total)
}

fn inspect(frozen: &FrozenMatrix, start: usize, stop: usize,
           level: usize, fault: Option<Fault>) -> Option<Verdict> {
    if start >= stop || !frozen.verify() {
        return None;
    }
    let mut children = Vec::new();
    if stop - start > 1 {
        if !(stop - start).is_multiple_of(3) {
            return None;
        }
        let stride = (stop - start) / 3;
        for seat in 0..3 {
            children.push(inspect(frozen, start + stride * seat,
                                  start + stride * (seat + 1), level + 1,
                                  fault)?);
        }
    }
    let mut work = worker(frozen, start, stop)?;
    let mut reflected = reflection(frozen, start, stop)?;
    let mut witnessed = fabric(frozen, start, stop)?;
    if start == 0 {
        if let Some(fault) = fault {
            if fault.level == level {
                match fault.seat {
                    Seat::Worker => work += 1,
                    Seat::Reflection => reflected += 1,
                    Seat::MatrixFabric => witnessed += 1,
                }
            }
        }
    }
    let locally_agreed = work == reflected && work == witnessed;
    let child_agreed = children.iter().all(|child| child.accepted);
    let consistent_children = children.is_empty()
        || (children.iter().map(|child| child.worker_total).sum::<i128>() == work
            && children.iter().map(|child| child.reflection_total).sum::<i128>()
                == reflected
            && children.iter().map(|child| child.fabric_total).sum::<i128>()
                == witnessed);
    let mismatches = usize::from(!locally_agreed || !consistent_children)
        + children.iter().map(|child| child.mismatches).sum::<usize>();
    Some(Verdict {
        accepted: locally_agreed && child_agreed && consistent_children,
        nodes: 1 + children.iter().map(|child| child.nodes).sum::<usize>(),
        fabric_reads: 1 + children.iter().map(|child| child.fabric_reads)
            .sum::<usize>(),
        worker_total: work,
        reflection_total: reflected,
        fabric_total: witnessed,
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
    inspect(frozen, 0, frozen.room_count(), 0, fault)
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
}
