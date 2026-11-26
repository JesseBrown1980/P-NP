//! Boolean cardinality recognition and formal-polynomial boundary research gate.
use crate::blind_cnf_factorization::fixture;
use crate::finite_slice_generator::{coordinates, original, parse, recognition, Form};
use std::collections::BTreeSet;
use std::fmt::Write;

type Clause = Vec<Form>;

fn transform(text: &str) -> (Vec<Form>, usize, Vec<Clause>) {
    let (n, source) = parse(text);
    let (eq, rest) = recognition(&source);
    let (forms, free) = coordinates(n, eq, &rest).expect("consistent fixture");
    let clauses = rest.iter().map(|c| c.iter().map(|&v| {
        let (m, b) = forms[v.unsigned_abs() as usize - 1];
        (m, b ^ (v < 0))
    }).collect()).collect();
    (forms, free, clauses)
}

fn recognize_amo(n: usize, clauses: &[Clause]) -> Option<Vec<Clause>> {
    let mut edges = BTreeSet::new();
    let mut residual = Vec::new();
    for c in clauses {
        if let [(a, true), (b, true)] = c.as_slice() {
            if a.count_ones() == 1 && b.count_ones() == 1 && a != b {
                edges.insert((*a.min(b), *a.max(b)));
                continue;
            }
        }
        residual.push(c.clone());
    }
    for i in 0..n {
        for j in i + 1..n {
            if !edges.contains(&(1u128 << i, 1u128 << j)) {
                return None;
            }
        }
    }
    Some(residual)
}
fn valid(clauses: &[Clause], a: u128) -> bool {
    clauses.iter().all(|c| c.iter().any(|&(m, b)| ((a & m).count_ones() % 2 != 0) ^ b))
}
fn reconstruct(forms: &[Form], a: u128) -> u128 {
    forms.iter().enumerate().fold(0, |x, (i, &(m, b))| {
        x | (u128::from(((a & m).count_ones() % 2 != 0) ^ b) << i)
    })
}
fn clique(n: usize, family: usize, seed: usize, missing: bool, blocks: bool) -> String {
    let (text, _, y) = fixture(n, family, seed, 0);
    let (nv, mut cs) = parse(&text);
    if !blocks {
        cs.retain(|c| !c.iter().all(|v| y.contains(&(v.abs()))));
    }
    for (i, &a) in y.iter().enumerate() {
        for (j, &b) in y.iter().enumerate().skip(i + 1) {
            if !(missing && i == 0 && j == 1) {
                cs.push(vec![-a, -b]);
            }
        }
    }
    let mut out = format!("p cnf {nv} {}\n", cs.len());
    for c in cs {
        for v in c {
            write!(&mut out, "{v} ").expect("write");
        }
        out.push_str("0\n");
    }
    out
}
#[test]
fn complete_exclusion_graph_boolean_language() {
    let (mut cases, mut queries, mut formerly_wide, mut rejected, mut total) = (0, 0, 0, 0, 0);
    for n in [6, 12, 24, 48, 60] {
        for family in 0..2 {
            for seed in 0..4 {
                let text = clique(n, family, seed, false, true);
                let (_, source) = parse(&text);
                let (forms, free, clauses) = transform(&text);
                assert_eq!(free, n);
                let residual = recognize_amo(n, &clauses).expect("complete pair coverage");
                let mut count = 0;
                for a in std::iter::once(0).chain((0..n).map(|i| 1u128 << i)) {
                    let accepted = valid(&residual, a);
                    assert_eq!(accepted, original(&source, reconstruct(&forms, a)));
                    count += usize::from(accepted);
                    queries += 1;
                }
                // Independent generator oracle counts the allowed zero/singleton patterns
                // on the original triples. The Boolean transformation does not use seed.
                let want = (0..=n).filter(|&i| {
                    let a = if i == 0 { 0 } else { 1u128 << (i - 1) };
                    (0..n / 3).all(|j| {
                        let p = ((a >> (3 * j)) & 7) as usize;
                        p != seed && p != (seed ^ 7)
                    })
                }).count();
                assert_eq!(count, want);
                total += count;
                formerly_wide += usize::from(n > 6);
                cases += 1;
            }
            let text = clique(n, family, 0, true, false);
            let (_, source) = parse(&text);
            let (forms, free, clauses) = transform(&text);
            assert_eq!(free, n);
            assert!(recognize_amo(n, &clauses).is_none());
            let mut pairs = 0;
            for i in 0..n {
                for j in i + 1..n {
                    let a = (1u128 << i) | (1u128 << j);
                    if valid(&clauses, a) {
                        assert!(original(&source, reconstruct(&forms, a)));
                        pairs += 1;
                    }
                }
            }
            assert_eq!(pairs, 1);
            rejected += 1;
        }
    }
    assert_eq!((cases, queries, formerly_wide, rejected, total), (40, 1240, 32, 10, 630));
    println!("BOOLEAN_AMO|cases={cases}|queries={queries}|formerly_wide={formerly_wide}|missing_edge_rejected={rejected}|sum_counts={total}");
}

fn pair_product(x: &[i128]) -> i128 {
    x.iter().enumerate().flat_map(|(i, &a)| x[i + 1..].iter().map(move |&b| 1 - a * b)).product()
}
fn singleton_polynomial(x: &[i128]) -> i128 {
    x.iter().map(|&a| 1 - a).product::<i128>()
        + x.iter().enumerate().map(|(i, &a)| {
            a * x.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, &b)| 1 - b).product::<i128>()
        }).sum::<i128>()
}
#[test]
fn boolean_equivalence_is_weaker_than_polynomial_identity() {
    let mut checks = 0;
    for n in 3..=8 {
        for a in 0..1usize << n {
            let x: Vec<_> = (0..n).map(|i| ((a >> i) & 1) as i128).collect();
            assert_eq!(pair_product(&x), singleton_polynomial(&x));
            assert_eq!(pair_product(&x), i128::from(a.count_ones() <= 1));
            checks += 1;
        }
        // Coefficient of x_0^(n-1) x_1 ... x_(n-1) comes uniquely from
        // the n-1 star edges centered at 0. The singleton expression is multilinear.
        let coefficient: i128 = if n % 2 == 1 { 1 } else { -1 };
        assert_ne!(coefficient.rem_euclid(2), 0);
        assert_ne!(coefficient.rem_euclid(3), 0);
        if n <= 5 {
            let edges: Vec<_> = (0..n).flat_map(|i| (i + 1..n).map(move |j| (i, j))).collect();
            let mut actual = 0;
            for m in 0..1usize << edges.len() {
                let mut degrees = vec![0; n];
                for (k, &(i, j)) in edges.iter().enumerate() {
                    if m & (1 << k) != 0 {
                        degrees[i] += 1;
                        degrees[j] += 1;
                    }
                }
                if degrees[0] == n - 1 && degrees[1..].iter().all(|&d| d == 1) {
                    actual += if m.count_ones() % 2 == 0 { 1 } else { -1 };
                }
            }
            assert_eq!(actual, coefficient);
        }
    }
    assert_eq!(checks, 504);
    assert_eq!(pair_product(&[2, 2, 2]), -27);
    assert_eq!(singleton_polynomial(&[2, 2, 2]), 5);
}
