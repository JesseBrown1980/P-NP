//! Independent finite replay of the eight-square signed crossing identity
//! in Anthropic, "A 2^{o(n)} Time Algorithm for 3-SAT" (2026), Eq. (12),
//! with the explicit sign in Eq. (27).
//!
//! Source: https://hexagonmath.org/pdf/2610.00577
//! Fixed 3-bit blocks only; this is not the paper's recursive #SAT solver.
//! All operations are exact, classical, integer-valued, and bounded.
#![forbid(unsafe_code)]

use crate::relic;

const BLOCKS: usize = 8;
const DIMENSION: usize = 64;
const FOUR_BLOCK_STATES: usize = 4096;
const FIELD_PRIME: i64 = 1_000_081;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IdentityReport {
    pub cases: usize,
    pub failed: usize,
    pub crossing_survivors: usize,
    pub signed_cancellations: usize,
    pub row_positive: usize,
    pub row_negative: usize,
    pub column_positive: usize,
    pub column_negative: usize,
    pub asymmetric_sign_pairs: usize,
    pub brown_exact_reconstructions: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RankReport {
    pub row_term_row_split: usize,
    pub column_term_row_split: usize,
    pub row_term_column_split: usize,
    pub column_term_column_split: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CountReport {
    pub formulas: usize,
    pub direct_full_evaluations: usize,
    pub direct_equality_pruned_evaluations: usize,
    pub signed_evaluations: usize,
    pub sat_formulas: usize,
    pub unsat_formulas: usize,
    pub nonzero_signed_components: usize,
    pub witness_canceling_components: usize,
    pub result_sum: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Certificate {
    pub identity: IdentityReport,
    pub ranks: RankReport,
    pub counts: CountReport,
}

/// σ(p,q) = (-1)^φ(p,q), using the *ordered* three-bit definition in
/// the paper, Eq. (27):
/// φ = sum_{1<=i<=j<=3} p_i q_j
///       + q_1 p_2 p_3 + p_1 q_2 p_3 + p_1 p_2 q_3.
pub fn sign(p: u8, q: u8) -> Result<i64, &'static str> {
    if p >= 8 || q >= 8 {
        return Err("eight-square sign requires a three-bit input");
    }
    let p = [(p >> 2) & 1, (p >> 1) & 1, p & 1];
    let q = [(q >> 2) & 1, (q >> 1) & 1, q & 1];
    let mut exponent = 0_u8;
    for (i, &pi) in p.iter().enumerate() {
        for &qj in q.iter().skip(i) {
            exponent += pi * qj;
        }
    }
    exponent += q[0] * p[1] * p[2] + p[0] * q[1] * p[2]
        + p[0] * p[1] * q[2];
    Ok(if exponent % 2 == 0 { 1 } else { -1 })
}

/// R and C are two signed leaves: their sum reconstructs twice the
/// crossing equality, but the individual signs may cancel.
pub fn row_and_column(x: u8, y: u8, u: u8, v: u8)
    -> Result<(i64, i64), &'static str>
{
    if x >= 8 || y >= 8 || u >= 8 || v >= 8 {
        return Err("crossing input exceeds three bits");
    }
    if x ^ y ^ u ^ v != 0 {
        return Ok((0, 0));
    }
    Ok((
        sign(x, y)? * sign(v, u)?,
        sign(x, u)? * sign(v, y)?,
    ))
}

fn expected_crossing(x: u8, y: u8, u: u8, v: u8) -> i64 {
    if x == v && y == u { 2 } else { 0 }
}

pub fn certify_identity() -> Result<IdentityReport, &'static str> {
    let mut report = IdentityReport::default();
    for p in 0..8_u8 {
        for q in 0..8_u8 {
            if sign(p, q)? != sign(q, p)? {
                report.asymmetric_sign_pairs += 1;
            }
        }
    }
    for x in 0..8_u8 {
        for y in 0..8_u8 {
            for u in 0..8_u8 {
                for v in 0..8_u8 {
                    let (row, column) = row_and_column(x, y, u, v)?;
                    let expectation = expected_crossing(x, y, u, v);
                    if row + column != expectation {
                        report.failed += 1;
                    }
                    report.cases += 1;
                    if expectation == 2 {
                        report.crossing_survivors += 1;
                    }
                    if row + column == 0 && row != 0 && column != 0 {
                        report.signed_cancellations += 1;
                    }
                    if row == 1 { report.row_positive += 1; }
                    if row == -1 { report.row_negative += 1; }
                    if column == 1 { report.column_positive += 1; }
                    if column == -1 { report.column_negative += 1; }
                    let original = [row, column, 0];
                    let witness = relic::freeze_triple(original);
                    if relic::rebuild_triple(witness) != Some(original) {
                        return Err("Brown witness failed to reconstruct signed leaves");
                    }
                    report.brown_exact_reconstructions += 1;
                }
            }
        }
    }
    if report.cases != FOUR_BLOCK_STATES || report.failed != 0
        || report.crossing_survivors != 64
        || report.signed_cancellations != 448
        || report.row_positive != 288 || report.row_negative != 224
        || report.column_positive != 288 || report.column_negative != 224
        || report.asymmetric_sign_pairs != 42
        || report.brown_exact_reconstructions != FOUR_BLOCK_STATES
    {
        return Err("paper's finite signed-identity census differs");
    }
    Ok(report)
}

fn positive_mod(x: i64) -> i64 {
    x.rem_euclid(FIELD_PRIME)
}

fn modular_power(mut base: i64, mut exponent: u64) -> i64 {
    let mut result = 1_i64;
    base = positive_mod(base);
    while exponent > 0 {
        if exponent & 1 != 0 {
            result = positive_mod(result * base);
        }
        base = positive_mod(base * base);
        exponent >>= 1;
    }
    result
}

/// Gaussian elimination over a fixed exact finite field. This rank is
/// a *lower bound* on rational rank. Eq. (30) independently provides
/// a factorization through eight states, proving upper bounds of 8
/// for the matching orientation.
fn modular_rank(mut matrix: [[i64; DIMENSION]; DIMENSION]) -> usize {
    let mut rank = 0_usize;
    for col in 0..DIMENSION {
        let pivot = (rank..DIMENSION)
            .find(|&row| positive_mod(matrix[row][col]) != 0);
        let Some(pivot) = pivot else { continue };
        matrix.swap(rank, pivot);
        let inverse = modular_power(matrix[rank][col], (FIELD_PRIME - 2) as u64);
        for cell in matrix[rank].iter_mut().skip(col) {
            *cell = positive_mod(*cell * inverse);
        }
        for other in 0..DIMENSION {
            if other == rank { continue; }
            let factor = positive_mod(matrix[other][col]);
            if factor == 0 { continue; }
            for c in col..DIMENSION {
                matrix[other][c] =
                    positive_mod(matrix[other][c] - factor * matrix[rank][c]);
            }
        }
        rank += 1;
        if rank == DIMENSION { break; }
    }
    rank
}

/// Confirm the paper's two complementary 8-versus-64 ranks in the
/// two orientations: (x,y)|(u,v) and (x,u)|(y,v).
pub fn certify_factor_ranks() -> Result<RankReport, &'static str> {
    let mut row_term_rows = [[0_i64; DIMENSION]; DIMENSION];
    let mut column_term_rows = [[0_i64; DIMENSION]; DIMENSION];
    let mut row_term_columns = [[0_i64; DIMENSION]; DIMENSION];
    let mut column_term_columns = [[0_i64; DIMENSION]; DIMENSION];
    for x in 0..BLOCKS {
        for y in 0..BLOCKS {
            for u in 0..BLOCKS {
                for v in 0..BLOCKS {
                    let (r, c) = row_and_column(x as u8, y as u8, u as u8, v as u8)?;
                    let ri = BLOCKS * x + y;
                    let rj = BLOCKS * u + v;
                    row_term_rows[ri][rj] = r;
                    column_term_rows[ri][rj] = c;
                    let ci = BLOCKS * x + u;
                    let cj = BLOCKS * y + v;
                    row_term_columns[ci][cj] = r;
                    column_term_columns[ci][cj] = c;
                }
            }
        }
    }
    let report = RankReport {
        row_term_row_split: modular_rank(row_term_rows),
        column_term_row_split: modular_rank(column_term_rows),
        row_term_column_split: modular_rank(row_term_columns),
        column_term_column_split: modular_rank(column_term_columns),
    };
    if report != (RankReport {
        row_term_row_split: 8,
        column_term_row_split: 64,
        row_term_column_split: 64,
        column_term_column_split: 8,
    }) {
        return Err("published signed-tensor orientations have changed");
    }
    Ok(report)
}

type Cnf = Vec<Vec<i8>>;

/// Bit positions are 1..=12 ordered x2,x1,x0,y2,y1,y0,
/// u2,u1,u0,v2,v1,v0. Clauses have width <= 3.
fn clause_satisfied(literals: &[i8], bits: u16) -> Result<bool, &'static str> {
    if literals.len() > 3 {
        return Err("fixture clause is wider than 3-CNF");
    }
    for &lit in literals {
        if lit == 0 || lit.unsigned_abs() > 12 {
            return Err("literal index is outside the twelve-bit domain");
        }
        let index = usize::from(lit.unsigned_abs()) - 1;
        let value = (bits >> (11 - index)) & 1 != 0;
        if value == (lit > 0) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn formula_satisfied(clauses: &[Vec<i8>], bits: u16)
    -> Result<bool, &'static str>
{
    for clause in clauses {
        if !clause_satisfied(clause, bits)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn bitpack(x: u8, y: u8, u: u8, v: u8) -> u16 {
    (u16::from(x) << 9) | (u16::from(y) << 6)
        | (u16::from(u) << 3) | u16::from(v)
}

fn brute_count(clauses: &[Vec<i8>]) -> Result<u64, &'static str> {
    let mut count = 0_u64;
    for x in 0..8_u8 {
        for y in 0..8_u8 {
            for u in 0..8_u8 {
                for v in 0..8_u8 {
                    if x == v && y == u
                        && formula_satisfied(clauses, bitpack(x, y, u, v))?
                    {
                        count += 1;
                    }
                }
            }
        }
    }
    Ok(count)
}

/// Equality-pruned baseline: only 8*8=64 satisfying-crossing settings.
fn pruned_count(clauses: &[Vec<i8>]) -> Result<u64, &'static str> {
    let mut count = 0_u64;
    for x in 0..8_u8 {
        for y in 0..8_u8 {
            if formula_satisfied(clauses, bitpack(x, y, y, x))? {
                count += 1;
            }
        }
    }
    Ok(count)
}

/// The small finite signed tensor contraction, directly using Eq. (12).
/// No asymptotic speed advantage is inferred from this 4096-case scan.
fn signed_count(clauses: &[Vec<i8>])
    -> Result<(u64, usize, usize), &'static str>
{
    let mut numerator = 0_i64;
    let mut nonzero_components = 0_usize;
    let mut canceled = 0_usize;
    for x in 0..8_u8 {
        for y in 0..8_u8 {
            for u in 0..8_u8 {
                for v in 0..8_u8 {
                    if !formula_satisfied(clauses, bitpack(x, y, u, v))? {
                        continue;
                    }
                    let (row, column) = row_and_column(x, y, u, v)?;
                    numerator += row + column;
                    if row != 0 { nonzero_components += 1; }
                    if column != 0 { nonzero_components += 1; }
                    if row != 0 && row + column == 0 {
                        canceled += 1;
                    }
                    let witness = relic::freeze_triple([row, column, 0]);
                    if relic::rebuild_triple(witness) != Some([row, column, 0]) {
                        return Err("counting leaf witness lost signed contributions");
                    }
                }
            }
        }
    }
    if numerator < 0 || numerator % 2 != 0 {
        return Err("signed #3SAT contraction not an even nonnegative count");
    }
    Ok((u64::try_from(numerator / 2).map_err(|_| "count overflow")?,
        nonzero_components, canceled))
}

fn fixed_cases() -> Vec<Cnf> {
    vec![
        vec![],
        vec![vec![1]],
        vec![vec![1], vec![-1]],
        vec![vec![1, -4, 7], vec![-2, 5, -10], vec![3, 6, 12]],
        vec![vec![-1, -2, -3], vec![4, 5, 6], vec![-7, 8, -12]],
        (1_i8..=12).map(|i| vec![i]).collect(),
        (1_i8..=12).map(|i| vec![-i]).collect(),
        vec![vec![1, 4, 7], vec![-3, 6, -9], vec![10, 11, -12]],
    ]
}

fn generated_cases() -> Vec<Cnf> {
    let mut cases = Vec::new();
    for sample in 0..27_u64 {
        let mut state = 0xa7c1_1357_9bed_1735_u64
            .wrapping_add(sample.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let mut formula = Vec::new();
        for _ in 0..(5 + sample as usize % 4) {
            let mut clause = Vec::new();
            for _ in 0..3 {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let variable = (state % 12 + 1) as i8;
                clause.push(if state & 16 == 0 { variable } else { -variable });
            }
            formula.push(clause);
        }
        cases.push(formula);
    }
    cases
}

pub fn certify_signed_counter() -> Result<CountReport, &'static str> {
    let mut report = CountReport::default();
    let formulas = fixed_cases().into_iter().chain(generated_cases());
    for cnf in formulas {
        let direct = brute_count(&cnf)?;
        let pruned = pruned_count(&cnf)?;
        let (signed, nonzero, canceled) = signed_count(&cnf)?;
        if direct != pruned || pruned != signed {
            return Err("signed #3SAT differs from independent direct counter");
        }
        report.formulas += 1;
        report.direct_full_evaluations += FOUR_BLOCK_STATES;
        report.direct_equality_pruned_evaluations += DIMENSION;
        report.signed_evaluations += FOUR_BLOCK_STATES;
        report.nonzero_signed_components += nonzero;
        report.witness_canceling_components += canceled;
        report.result_sum += signed;
        if signed == 0 {
            report.unsat_formulas += 1;
        } else {
            report.sat_formulas += 1;
        }
    }
    if report.formulas != 35 || report.sat_formulas == 0
        || report.unsat_formulas == 0
        || report.witness_canceling_components == 0
        || report.direct_full_evaluations != 143_360
        || report.direct_equality_pruned_evaluations != 2_240
        || report.signed_evaluations != 143_360
    {
        return Err("signed-count fixture census or negative control changed");
    }
    Ok(report)
}

pub fn certify() -> Result<Certificate, &'static str> {
    Ok(Certificate {
        identity: certify_identity()?,
        ranks: certify_factor_ranks()?,
        counts: certify_signed_counter()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_4096_point_eight_square_identity_and_brown_witness() {
        let report = certify_identity().expect("exact original equation");
        assert_eq!(report.cases, 4096);
        assert_eq!(report.signed_cancellations, 448);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn complementary_ranks_are_eight_and_sixty_four() {
        let ranks = certify_factor_ranks().expect("exact mod-p rank calculation");
        assert_eq!(ranks.row_term_row_split, 8);
        assert_eq!(ranks.column_term_row_split, 64);
        assert_eq!(ranks.row_term_column_split, 64);
        assert_eq!(ranks.column_term_column_split, 8);
    }

    #[test]
    fn signed_counter_equals_both_direct_cnf_baselines() {
        let count = certify_signed_counter().expect("signed exact counting");
        assert_eq!(count.formulas, 35);
        assert!(count.sat_formulas > 0);
        assert!(count.unsat_formulas > 0);
    }

    #[test]
    fn asymmetric_signs_and_malformed_states_rejected() {
        assert_eq!(sign(0, 0), Ok(1));
        assert_eq!(sign(8, 0), Err("eight-square sign requires a three-bit input"));
        assert!(row_and_column(0, 8, 0, 0).is_err());
        assert!(clause_satisfied(&[13], 0).is_err());
        assert!(clause_satisfied(&[], 0).is_ok_and(|result| !result));
        assert!(clause_satisfied(&[1, 2, 3, 4], 0).is_err());
        let asymmetry = (0..8_u8)
            .flat_map(|p| (0..8_u8).map(move |q| (p, q)))
            .filter(|&(p, q)| sign(p, q) != sign(q, p))
            .count();
        assert_eq!(asymmetry, 42);
    }
}
