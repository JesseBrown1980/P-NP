#![forbid(unsafe_code)]
//! Rust 1.81 exact-integer P-NP policy and native matrix-fabric pilot.
//! Historical Python algorithms are retained solely as legacy evidence.

pub mod freeze;
pub mod matrix;
pub mod policy;
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
