#![forbid(unsafe_code)]
//! Rust 1.81 exact-integer P-NP policy and native matrix-fabric pilot.
//! Historical Python algorithms are retained solely as legacy evidence.

pub mod freeze;
pub mod guidance;
pub mod matrix;
pub mod policy;
pub mod relic;
pub mod solver;
pub mod ternary;
pub mod triad;

pub fn certify_matrix_fabric() -> Result<(usize, usize, usize), &'static str> {
    use crate::freeze::FrozenMatrix;
    use crate::matrix::{Matrix, CELLS};
    use crate::triad::{wave_cascade, Fault, Seat};

    let mut rooms_tested = 0;
    let mut played_cells = 0;
    let mut corruptions_rejected = 0;
    for (room_count, height) in [(3, 1), (9, 2), (27, 3), (81, 4)] {
        let original = Matrix::generated(room_count)?;
        let frozen = FrozenMatrix::freeze(&original);
        if !frozen.verify() {
            return Err("freeze shadow integrity failure");
        }
        for room in 0..room_count {
            for index in 0..CELLS {
                let actual = frozen.play(room, index).ok_or("frozen playback failure")?;
                if actual.value != original.room(room).ok_or("original room missing")?[index] {
                    return Err("playback differed from original");
                }
                played_cells += 1;
            }
        }
        let clean = wave_cascade(&frozen, None).ok_or("clean wave failed")?;
        if !clean.accepted || clean.nodes != clean.fabric_reads {
            return Err("clean 3-seat wave not accepted");
        }
        for level in 0..=height {
            for seat in [Seat::Worker, Seat::Reflection, Seat::MatrixFabric] {
                let bad = wave_cascade(&frozen, Some(Fault { level, seat }))
                    .ok_or("fault trace unavailable")?;
                if bad.accepted || bad.mismatches == 0 {
                    return Err("planted false report incorrectly accepted");
                }
                corruptions_rejected += 1;
            }
        }
        rooms_tested += room_count;
    }
    if rooms_tested != 120 || played_cells != 3240 || corruptions_rejected != 42 {
        return Err("acceptance totals changed");
    }
    Ok((rooms_tested, played_cells, corruptions_rejected))
}

/// Stronger third-seat acceptance: same-column cancelling errors and
/// coordinated worker/reflection claims must fail against a reversible
/// matrix witness at every supervisor level.
pub fn certify_exact_witness() -> Result<(usize, usize, usize), &'static str> {
    use crate::freeze::FrozenMatrix;
    use crate::matrix::Matrix;
    use crate::triad::{
        wave_cascade_balanced, wave_cascade_coordinated, Fault, Seat,
    };

    let mut balanced_rejected = 0;
    let mut coordinated_rejected = 0;
    let mut exact_triples = 0;
    for (rooms, height) in [(3, 1), (9, 2), (27, 3), (81, 4)] {
        let matrix = Matrix::generated(rooms)?;
        let frozen = FrozenMatrix::freeze(&matrix);
        if !frozen.verify() {
            return Err("reversible frozen witness failure");
        }
        for room in 0..rooms {
            for column in 0..9 {
                let triple = frozen.reconstruct_triple(room, column)
                    .ok_or("missing exact triple")?;
                for (depth, value) in triple.iter().enumerate() {
                    if *value != matrix.room(room).ok_or("missing source room")?
                        [column + 9 * depth]
                    {
                        return Err("exact triple mismatch");
                    }
                }
                exact_triples += 1;
            }
        }
        for level in 0..=height {
            for seat in [Seat::Worker, Seat::Reflection, Seat::MatrixFabric] {
                let verdict = wave_cascade_balanced(
                    &frozen, Fault { level, seat },
                ).ok_or("balanced test could not execute")?;
                if verdict.accepted || verdict.mismatches < 2
                    || verdict.worker_total != verdict.reflection_total
                    || verdict.worker_total != verdict.fabric_total
                {
                    return Err("balanced tamper bypassed exact witness");
                }
                balanced_rejected += 1;
            }
            let verdict = wave_cascade_coordinated(&frozen, level)
                .ok_or("coordinated test could not execute")?;
            if verdict.accepted || verdict.mismatches == 0 {
                return Err("coordinated reports bypassed matrix witness");
            }
            coordinated_rejected += 1;
        }
    }
    if (balanced_rejected, coordinated_rejected, exact_triples)
        != (42, 14, 1080)
    {
        return Err("exact witness acceptance totals changed");
    }
    Ok((balanced_rejected, coordinated_rejected, exact_triples))
}
