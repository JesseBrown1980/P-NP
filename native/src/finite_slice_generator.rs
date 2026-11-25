//! Research-only finite generator control: explicit parity recovery, bounded slices, exact cache.
use crate::blind_cnf_factorization::fixture;
use std::collections::{BTreeMap, BTreeSet};

type Form = (u128, bool);
type Clause = Vec<Form>;
type Key = (usize, Vec<Clause>);
type Bucket = BTreeMap<Vec<u32>, BTreeSet<u32>>;
type Slice = (Vec<usize>, usize);

#[derive(Debug)]
struct Generator {
    forms: Vec<Form>,
    slices: Vec<Slice>,
    tables: Vec<(usize, u64)>,
    uncached: usize,
    cached: usize,
}
fn parity(x: u128) -> bool {
    x.count_ones() % 2 != 0
}
fn compact(mask: u128, coords: &[usize]) -> u128 {
    coords.iter().enumerate().fold(0, |m, (j, &i)| {
        m | (((mask >> i) & 1) << j)
    })
}
fn parse(text: &str) -> (usize, Vec<Vec<i32>>) {
    let mut lines = text.lines();
    let h: Vec<_> = lines.next().expect("header").split_whitespace().collect();
    assert_eq!(&h[..2], &["p", "cnf"]);
    let n: usize = h[2].parse().expect("n");
    assert!((1..=120).contains(&n));
    let clauses: Vec<Vec<i32>> = lines.map(|line| {
        let mut c: Vec<i32> = line.split_whitespace().map(|s| s.parse().expect("literal")).collect();
        assert_eq!(c.pop(), Some(0));
        assert!(c.iter().all(|v| v.unsigned_abs() > 0 && v.unsigned_abs() as usize <= n));
        assert_eq!(c.iter().map(|v| v.unsigned_abs()).collect::<BTreeSet<_>>().len(), c.len());
        c
    }).collect();
    assert_eq!(clauses.len(), h[3].parse::<usize>().expect("m"));
    (n, clauses)
}
fn recognition(clauses: &[Vec<i32>]) -> (Vec<Form>, Vec<Vec<i32>>) {
    let mut buckets = Bucket::new();
    for c in clauses {
        let mut key: Vec<_> = c.iter().map(|v| v.unsigned_abs()).collect();
        key.sort_unstable();
        let p = key.iter().enumerate().fold(0, |m, (j, &v)| {
            m | (u32::from(c.contains(&-(v as i32))) << j)
        });
        buckets.entry(key).or_default().insert(p);
    }
    let mut eq = Vec::new();
    let mut residual = Vec::new();
    for (key, mut patterns) in buckets {
        if (2..=3).contains(&key.len()) {
            for rhs in [false, true] {
                let wrong: Vec<u32> = (0..1 << key.len()).filter(|&p| parity(u128::from(p)) != rhs).collect();
                if wrong.iter().all(|p| patterns.contains(p)) {
                    eq.push((key.iter().fold(0, |m, &v| m | (1 << (v-1))), rhs));
                    for p in wrong { patterns.remove(&p); }
                }
            }
        }
        for p in patterns {
            residual.push(key.iter().enumerate().map(|(j, &v)| {
                if p & (1 << j) != 0 { -(v as i32) } else { v as i32 }
            }).collect());
        }
    }
    (eq, residual)
}
fn coordinates(n: usize, mut eq: Vec<Form>, residual: &[Vec<i32>]) -> Option<(Vec<Form>, usize)> {
    let active = residual.iter().flatten().fold(0u128, |m, v| m | (1 << (v.unsigned_abs()-1)));
    let mut order: Vec<_> = (0..n).collect();
    order.sort_by_key(|&i| (active & (1 << i) != 0, i));
    let mut piv = Vec::new();
    for i in order {
        let r = piv.len();
        let Some(j) = (r..eq.len()).find(|&j| eq[j].0 & (1 << i) != 0) else { continue; };
        eq.swap(r, j);
        let p = eq[r];
        for (j, e) in eq.iter_mut().enumerate() {
            if j != r && e.0 & (1 << i) != 0 {
                e.0 ^= p.0;
                e.1 ^= p.1;
            }
        }
        piv.push(i);
    }
    if eq.iter().any(|&(m, b)| m == 0 && b) { return None; }
    let free: Vec<_> = (0..n).filter(|i| !piv.contains(i)).collect();
    let mut forms: Vec<_> = (0..n).map(|i| (compact(1 << i, &free), false)).collect();
    for (j, &i) in piv.iter().enumerate() {
        forms[i] = (compact(eq[j].0, &free), eq[j].1);
    }
    Some((forms, free.len()))
}
fn components(forms: &[Form], n: usize, residual: &[Vec<i32>]) -> (Vec<u128>, Vec<(u128, Clause)>) {
    let mut groups: Vec<_> = (0..n).map(|i| 1 << i).collect();
    let mut tagged = Vec::new();
    for c in residual {
        let local: Clause = c.iter().map(|&v| {
            let (m, b) = forms[v.unsigned_abs() as usize-1];
            (m, b ^ (v < 0))
        }).collect();
        let support = local.iter().fold(0, |m, &(a, _)| m | a);
        // Structured fixture domain: residual supports are nonempty.
        assert_ne!(support, 0);
        let mut u = 0;
        groups.retain(|&g| {
            if g & support != 0 { u |= g; false } else { true }
        });
        assert_eq!(u & support, support);
        groups.push(u);
        tagged.push((support, local));
    }
    groups.sort_unstable();
    (groups, tagged)
}
fn compile(text: &str) -> Option<Generator> {
    let (n, clauses) = parse(text);
    let (eq, residual) = recognition(&clauses);
    let (forms, free) = coordinates(n, eq, &residual)?;
    let (groups, tagged) = components(&forms, free, &residual);
    if groups.iter().any(|g| g.count_ones() > 6) { return None; }
    let mut out = Generator { forms, slices: Vec::new(), tables: Vec::new(), uncached: 0, cached: 0 };
    let mut cache: BTreeMap<Key, usize> = BTreeMap::new();
    for g in groups {
        let coords: Vec<_> = (0..free).filter(|&i| g & (1 << i) != 0).collect();
        let width = coords.len();
        let mut local: Vec<Clause> = tagged.iter().filter(|(s, _)| s & g != 0).map(|(_, c)| {
            let mut v: Clause = c.iter().map(|&(m, b)| (compact(m, &coords), b)).collect();
            v.sort_unstable();
            v
        }).collect();
        local.sort_unstable();
        let key = (width, local);
        out.uncached += 1 << width;
        let idx = if let Some(&i) = cache.get(&key) { i } else {
            let mut table = 0u64;
            for a in 0..1u128 << width {
                let valid = key.1.iter().all(|c| c.iter().any(|&(m, b)| parity(a & m) ^ b));
                table |= u64::from(valid) << a;
                out.cached += 1;
            }
            let idx = out.tables.len();
            out.tables.push((width, table));
            cache.insert(key, idx);
            idx
        };
        out.slices.push((coords, idx));
    }
    Some(out)
}
fn count(g: &Generator) -> u128 {
    g.slices.iter().map(|(_, i)| u128::from(g.tables[*i].1.count_ones())).product()
}
fn query(g: &Generator, a: u128) -> (u128, bool) {
    let x = g.forms.iter().enumerate().fold(0, |x, (j, &(m, b))| {
        x | (u128::from(parity(m & a) ^ b) << j)
    });
    let ok = g.slices.iter().all(|(coords, i)| g.tables[*i].1 & (1 << compact(a, coords)) != 0);
    (x, ok)
}
fn original(clauses: &[Vec<i32>], x: u128) -> bool {
    clauses.iter().all(|c| c.iter().any(|&v| (x & (1 << (v.unsigned_abs()-1)) != 0) == (v > 0)))
}
#[test]
fn finite_generating_slice_scaling() {
    let (mut cases, mut checks, mut corruptions, mut holds) = (0, 0, 0, 0);
    let (mut uncached, mut cached) = (0, 0);
    for n in [6, 12, 24, 48, 60] {
        for family in 0..2 {
            for seed in 0..2 {
                for mode in 0..2 {
                    let (s, _, _) = fixture(n, family, seed, mode);
                    let (_, clauses) = parse(&s);
                    let mut g = compile(&s).expect("bounded generator");
                    let want = if mode == 0 { 6u128.pow((n/3) as u32) } else { 27 * 6u128.pow((n/3-2) as u32) };
                    assert_eq!(count(&g), want);
                    let limit = if n <= 12 { 1 << n } else { 128 };
                    for i in 0..limit {
                        let a = if n <= 12 { i as u128 } else { (i as u128 * 0x9e3779b97f4a7c15) & ((1 << n)-1) };
                        let (x, ok) = query(&g, a);
                        assert_eq!(ok, original(&clauses, x));
                        checks += 1;
                    }
                    let idx = g.slices[0].1;
                    let old = g.tables[idx].1;
                    let mut witness = 0u128;
                    for (coords, i) in &g.slices {
                        let a = g.tables[*i].1.trailing_zeros();
                        for (j, &q) in coords.iter().enumerate() {
                            witness |= u128::from((a >> j) & 1) << q;
                        }
                    }
                    assert!(query(&g, witness).1);
                    g.tables[idx].1 ^= 1 << old.trailing_zeros();
                    assert!(!query(&g, witness).1);
                    assert!(original(&clauses, query(&g, witness).0));
                    corruptions += 1;
                    uncached += g.uncached;
                    cached += g.cached;
                    cases += 1;
                    println!("FINITE_GENERATOR|n={n}|family={family}|seed={seed}|mode={mode}|count={want}|uncached={}|cached={}", g.uncached, g.cached);
                }
            }
            let (s, _, _) = fixture(n, family, 0, 2);
            if n > 6 {
                assert!(compile(&s).is_none());
                holds += 1;
            } else {
                assert!(compile(&s).is_some());
            }
        }
    }
    assert_eq!((cases, checks, corruptions, holds), (40, 36352, 40, 8));
    assert_eq!(uncached, 4160);
    assert!(cached < uncached);
    println!("FINITE_GENERATOR_TOTAL|cases={cases}|checks={checks}|corruptions={corruptions}|holds={holds}|uncached={uncached}|cached={cached}");
}
