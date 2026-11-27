#![forbid(unsafe_code)]
//! Rust 1.81 exact-integer P-NP policy and native matrix-fabric pilot.
//! Historical Python algorithms are retained solely as legacy evidence.

pub mod binary_ternary_bridge;
pub mod checkpoint;
pub mod ensemble;
pub mod freeze;
pub mod fabric_gc;
pub mod gc_archive;
pub mod gulp_gc;
pub mod guidance;
pub mod matrix;
pub mod omnibit_tournament;
// The original frozen source contains two historic literal-grouping styles.
// Scope this lint allowance exclusively to the byte-identical vendor module.
#[allow(clippy::unusual_byte_groupings)]
pub mod omega_operator_catalog;
pub mod omnibit_operator_gate;
pub mod policy;
pub mod relic;
pub mod prism;
pub mod scoring;
pub mod sha256;
pub mod source;
pub mod solver;
pub mod ternary;
pub mod tls_memory;
pub mod tls_stochastic;
pub mod tls_omnibit_gc;
pub mod tls_entropy_omnibit;
pub mod tls_stream_fabric;
pub mod http0_fabric_witness;
pub mod http0_compute;
pub mod microkernel_compute;
pub mod matrix_wire;
pub mod full_matrix_fabric;
pub mod temporal_fabric;
pub mod prl_temporal_fabric;
pub mod triad;
pub mod wave;

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

/// Independently exercise every physically materialized 3/9/27/81 room
/// through the lazy signed ternary wave; count checks and reject altered paths.
pub fn certify_lazy_wave() -> Result<(usize, usize, usize), &'static str> {
    use crate::freeze::FrozenMatrix;
    use crate::matrix::Matrix;
    use crate::wave::WaveRouter;

    let mut rooms = 0_usize;
    let mut path_decisions = 0_usize;
    let mut children_verified = 0_usize;
    for count in [3_usize, 9, 27, 81] {
        let source = Matrix::generated(count)?;
        let frozen = FrozenMatrix::freeze(&source);
        let router = WaveRouter::new(&frozen).ok_or("wave source witness unavailable")?;
        for room in 0..count {
            let original = source.room(room).ok_or("room absent")?;
            let expected: i128 = original.iter().map(|&x| i128::from(x)).sum();
            let report = router.read(room).ok_or("wave traversal absent")?;
            if !router.verify_read(&report)
                || report.room_total != expected
                || report.center_value != 1
                || report.address != room as u128
            {
                return Err("wave third-seat source replay disagreed");
            }
            let mut corrupted = report.clone();
            corrupted.room_total = corrupted.room_total.checked_add(1)
                .ok_or("corruption overflow")?;
            if router.verify_read(&corrupted) {
                return Err("altered ternary wave report accepted");
            }
            let mut tampered_path = report.clone();
            tampered_path.digits[0] = tampered_path.digits[0].negate();
            tampered_path.address = tampered_path.address.checked_add(1)
                .ok_or("path corruption overflow")?;
            if router.verify_read(&tampered_path) {
                return Err("altered wave address accepted");
            }
            rooms += 1;
            path_decisions += report.levels;
            children_verified += report.choices_checked;
        }
    }
    if (rooms, path_decisions, children_verified) != (120, 426, 1278) {
        return Err("ternary wave acceptance counts changed");
    }
    Ok((rooms, path_decisions, children_verified))
}

#[cfg(test)]
mod address_factorization;

#[cfg(test)]
mod blind_cnf_factorization;

#[cfg(test)]
mod finite_slice_generator;

#[cfg(test)]
mod connected_slice_separator;

#[cfg(test)]
mod boolean_polynomial_boundary;

#[cfg(test)]
mod overlapping_signed_amo;

#[cfg(test)]
mod uniform_gate_boundary;

#[cfg(test)]
mod shadow_residual_projection;

#[cfg(test)]
mod nonaffine_weighted_shadow;

#[cfg(test)]
mod primitive_shadow_generator;

#[cfg(test)]
mod automatic_shadow_replay;

#[cfg(test)]
mod signed_residual_reuse;
