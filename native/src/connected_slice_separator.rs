//! Connected finite slice control: exact bounded variable elimination and reverse witness replay.
use crate::blind_cnf_factorization::fixture;
use crate::finite_slice_generator::{coordinates, original, parse, recognition, Form};
use std::collections::BTreeSet;
use std::fmt::Write;

type Clause = Vec<Form>;
type Factor = (u128, Vec<u128>);
type Message = (usize, u128, Vec<bool>);
type Model = (Vec<Form>, usize, Vec<Clause>);
const CAP: usize = 6;

fn bits(mask: u128) -> impl Iterator<Item = usize> {
    (0..128).filter(move |&i| mask & (1 << i) != 0)
}
fn pack(a: u128, mask: u128) -> usize {
    bits(mask).enumerate().fold(0, |m, (j, i)| m | ((((a >> i) & 1) as usize) << j))
}
fn unpack(a: usize, mask: u128) -> u128 {
    bits(mask).enumerate().fold(0, |m, (j, i)| m | (u128::from(((a >> j) & 1) as u8) << i))
}
fn scope(c: &Clause) -> u128 {
    c.iter().fold(0, |m, &(v, _)| m | v)
}
fn discover(text: &str) -> Option<Model> {
    let (n, cnf) = parse(text);
    let (eq, rest) = recognition(&cnf);
    let (forms, free) = coordinates(n, eq, &rest)?;
    let clauses = rest.iter().map(|c| c.iter().map(|&v| {
        let (m, b) = forms[v.unsigned_abs() as usize - 1];
        (m, b ^ (v < 0))
    }).collect()).collect();
    Some((forms, free, clauses))
}
// Incremental minimum-degree heuristic. Its result is a certified order, not a minimal-width claim.
fn plan(n: usize, clauses: &[Clause]) -> Option<(Vec<usize>, usize)> {
    let mut adj = vec![0u128; n];
    for c in clauses {
        let m = scope(c);
        if m.count_ones() as usize > CAP { return None; }
        for v in bits(m) { adj[v] |= m ^ (1 << v); }
    }
    let mut queue: BTreeSet<_> = adj.iter().enumerate().map(|(v, m)| (m.count_ones() as usize, v)).collect();
    let mut order = Vec::new();
    let mut width = 0;
    while let Some((d, v)) = queue.pop_first() {
        if d + 1 > CAP { return None; }
        width = width.max(d + 1);
        let ns = adj[v];
        for a in bits(ns) {
            assert!(queue.remove(&(adj[a].count_ones() as usize, a)));
            adj[a] = (adj[a] | ns) & !(1 << a) & !(1 << v);
            queue.insert((adj[a].count_ones() as usize, a));
        }
        order.push(v);
    }
    assert_eq!(order.len(), n);
    Some((order, width))
}
struct ResultRow {
    count: u128,
    witness: u128,
    entries: usize,
    messages: usize,
    width: usize,
}
fn evaluate(clauses: &[Clause], order: &[usize]) -> ResultRow {
    let mut fs: Vec<Factor> = clauses.iter().map(|c| {
        let m = scope(c);
        let table = (0..1usize << m.count_ones()).map(|i| {
            let a = unpack(i, m);
            u128::from(c.iter().any(|&(r, b)| ((a & r).count_ones() % 2 != 0) ^ b))
        }).collect();
        (m, table)
    }).collect();
    let mut trace: Vec<Message> = Vec::new();
    let (mut entries, mut messages, mut width) = (0, 0, 0);
    for &v in order {
        let bit = 1 << v;
        let (bucket, mut rest): (Vec<_>, Vec<_>) = fs.into_iter().partition(|(m, _)| m & bit != 0);
        let union = bucket.iter().fold(bit, |m, (s, _)| m | s);
        let w = union.count_ones() as usize;
        assert!(w <= CAP);
        width = width.max(w);
        let rem = union ^ bit;
        let mut table = Vec::new();
        let mut choices = Vec::new();
        for p in 0..1usize << rem.count_ones() {
            let a = unpack(p, rem);
            let weight = |x| bucket.iter().map(|(m, t)| t[pack(x, *m)]).product::<u128>();
            let (lo, hi) = (weight(a), weight(a | bit));
            entries += 2;
            table.push(lo + hi);
            choices.push(lo == 0 && hi > 0);
        }
        messages += table.len();
        trace.push((v, rem, choices));
        rest.push((rem, table));
        fs = rest;
    }
    let count = fs.iter().map(|(m, t)| { assert_eq!(*m, 0); t[0] }).product();
    let mut witness = 0;
    if count > 0 {
        for (v, m, ch) in trace.into_iter().rev() {
            if ch[pack(witness, m)] { witness |= 1 << v; }
        }
        assert!(clauses.iter().all(|c| c.iter().any(|&(m, b)| ((witness & m).count_ones() % 2 != 0) ^ b)));
    }
    ResultRow { count, witness, entries, messages, width }
}
fn connected(n: usize, family: usize, seed: usize, kind: usize) -> String {
    let (s, _, y) = fixture(n, family, seed, 2);
    let (nv, mut cs) = parse(&s);
    if kind == 1 { cs.push(vec![-y[n - 1], -y[0]]); }
    if kind == 2 {
        for (i, &a) in y.iter().enumerate() {
            for &b in &y[i + 1..] { cs.push(vec![-a, -b]); }
        }
    }
    let mut out = format!("p cnf {nv} {}\n", cs.len());
    for c in cs {
        for v in c { write!(&mut out, "{v} ").expect("write"); }
        out.push_str("0\n");
    }
    out
}
// Independent oracle works on the original generator's eight possible block patterns.
fn oracle(n: usize, seed: usize, cycle: bool) -> u128 {
    let allowed: Vec<_> = (0..8).filter(|&p| p != seed && p != (seed ^ 7)).collect();
    let mut total = 0;
    for &first in &allowed {
        let mut row = [0u128; 8];
        row[first] = 1;
        for _ in 1..n / 3 {
            let mut next = [0u128; 8];
            for &p in &allowed {
                for &q in &allowed {
                    if (q >> 2) & (p & 1) == 0 { next[p] += row[q]; }
                }
            }
            row = next;
        }
        for &p in &allowed {
            if !cycle || (p >> 2) & (first & 1) == 0 { total += row[p]; }
        }
    }
    total
}
#[test]
fn connected_separator_counts_and_reverse_replay() {
    let (mut cases, mut holds, mut rejects, mut entries, mut messages) = (0, 0, 0, 0, 0);
    for n in [6, 12, 24, 48, 60] {
        for family in 0..2 {
            for seed in 0..2 {
                for kind in 0..2 {
                    let text = connected(n, family, seed, kind);
                    let (_, cnf) = parse(&text);
                    let (forms, free, clauses) = discover(&text).expect("consistent");
                    assert_eq!(free, n);
                    let (order, width) = plan(free, &clauses).expect("bounded separator");
                    let result = evaluate(&clauses, &order);
                    assert_eq!(result.count, oracle(n, seed, kind == 1));
                    assert_eq!(width, result.width);
                    assert!(width <= 3);
                    let x = forms.iter().enumerate().fold(0u128, |a, (j, &(m, b))| {
                        a | (u128::from(((result.witness & m).count_ones() % 2 != 0) ^ b) << j)
                    });
                    assert!(original(&cnf, x));
                    assert!(!original(&cnf, x ^ 1));
                    cases += 1;
                    rejects += 1;
                    entries += result.entries;
                    messages += result.messages;
                    println!("CONNECTED_SLICE|n={n}|family={family}|seed={seed}|kind={kind}|count={}|width={width}|entries={}|messages={}", result.count, result.entries, result.messages);
                }
            }
            let text = connected(n, family, 0, 2);
            let (_, free, clauses) = discover(&text).expect("consistent equations");
            if n > CAP {
                assert!(plan(free, &clauses).is_none());
                holds += 1;
            } else {
                assert!(plan(free, &clauses).is_some());
            }
        }
    }
    assert_eq!((cases, holds, rejects), (40, 8, 40));
    assert!(entries <= 40 * 60 * 8);
    println!("CONNECTED_SLICE_TOTAL|cases={cases}|holds={holds}|rejects={rejects}|entries={entries}|messages={messages}");
}
