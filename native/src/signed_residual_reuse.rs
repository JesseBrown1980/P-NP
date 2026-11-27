//! Certified signed-address reuse: exact mapped clause equality, bounded discovery.
use crate::blind_cnf_factorization::{dpll, shuffle};
use std::collections::BTreeSet;

type Cnf = Vec<Vec<i32>>;
type Profile = ([usize; 4], [usize; 4]);

fn canonical(mut cs: Cnf) -> Cnf {
    for c in &mut cs { c.sort_unstable(); c.dedup(); }
    cs.sort_unstable();
    cs.dedup();
    cs
}
fn mapped(cs: &Cnf, map: &[i32]) -> Cnf {
    canonical(cs.iter().map(|c| c.iter().map(|&v| {
        map[v.unsigned_abs() as usize - 1] * v.signum()
    }).collect()).collect())
}
fn profile(n: usize, cs: &Cnf) -> Vec<Profile> {
    let mut p = vec![([0; 4], [0; 4]); n];
    for c in cs {
        for &v in c {
            let a = &mut p[v.unsigned_abs() as usize - 1];
            if v > 0 { a.0[c.len() - 1] += 1; }
            else { a.1[c.len() - 1] += 1; }
        }
    }
    p
}
struct Matcher<'a> {
    source: &'a Cnf,
    target: &'a Cnf,
    candidates: Vec<Vec<i32>>,
    order: Vec<usize>,
    map: Vec<i32>,
    used: u32,
    probes: usize,
    cap: usize,
}
impl Matcher<'_> {
    fn search(&mut self, depth: usize) -> bool {
        if depth == self.order.len() {
            return mapped(self.source, &self.map) == *self.target;
        }
        let v = self.order[depth];
        for j in 0..self.candidates[v].len() {
            let t = self.candidates[v][j];
            let bit = 1u32 << (t.unsigned_abs() - 1);
            if self.used & bit != 0 { continue; }
            if self.probes >= self.cap { return false; }
            self.probes += 1;
            self.map[v] = t;
            self.used |= bit;
            let compatible = self.source.iter().all(|c| {
                if c.iter().any(|x| self.map[x.unsigned_abs() as usize - 1] == 0) {
                    return true;
                }
                let mut image: Vec<_> = c.iter().map(|x| {
                    self.map[x.unsigned_abs() as usize - 1] * x.signum()
                }).collect();
                image.sort_unstable();
                self.target.binary_search(&image).is_ok()
            });
            if compatible && self.search(depth + 1) { return true; }
            self.map[v] = 0;
            self.used &= !bit;
        }
        false
    }
}
fn discover(n: usize, source: &Cnf, target: &Cnf, cap: usize) -> Option<Vec<i32>> {
    if source.len() != target.len() { return None; }
    let p = profile(n, source);
    let q = profile(n, target);
    let candidates: Vec<Vec<i32>> = p.iter().map(|&a| {
        let mut out = Vec::new();
        for (j, &b) in q.iter().enumerate() {
            if a == b { out.push((j + 1) as i32); }
            if a == (b.1, b.0) { out.push(-((j + 1) as i32)); }
        }
        out
    }).collect();
    let mut order: Vec<_> = (0..n).collect();
    order.sort_by_key(|&i| (candidates[i].len(), i));
    let mut s = Matcher { source, target, candidates, order, map: vec![0; n],
        used: 0, probes: 0, cap };
    if s.search(0) { Some(s.map) } else { None }
}
fn satisfies(cs: &Cnf, bits: u32) -> bool {
    cs.iter().all(|c| c.iter().any(|&v| {
        (bits & (1 << (v.unsigned_abs() - 1)) != 0) == (v > 0)
    }))
}
fn reverse(bits: u32, map: &[i32]) -> u32 {
    map.iter().enumerate().fold(0, |a, (i, &v)| {
        let val = (bits & (1 << (v.unsigned_abs() - 1)) != 0) == (v > 0);
        a | (u32::from(val) << i)
    })
}
fn renamed(cs: &Cnf, n: usize, seed: &mut u64) -> Cnf {
    let mut p: Vec<i32> = (1..=n as i32).collect();
    shuffle(&mut p, seed);
    let mut signs: Vec<_> = (0..n).map(|i| i % 2 == 0).collect();
    shuffle(&mut signs, seed);
    for (v, neg) in p.iter_mut().zip(signs) {
        if neg { *v = -*v; }
    }
    mapped(cs, &p)
}
fn edges(es: &[(i32, i32)]) -> Cnf {
    canonical(es.iter().map(|&(a, b)| vec![-a, -b]).collect())
}
#[test]
fn signed_residual_cache_certificate_and_collision() {
    let prism = edges(&[(1,2),(2,3),(3,1),(4,5),(5,6),(6,4),(1,4),(2,5),(3,6)]);
    let complete = edges(&(1..=3).flat_map(|a| (4..=6).map(move |b| (a,b))).collect::<Vec<_>>());
    assert_eq!(profile(6, &prism), profile(6, &complete));
    let other = canonical(vec![vec![1,-2,3],vec![2,4],vec![-3,-4,5],vec![1,-5,6],vec![-1,2,-6]]);
    let unsat = canonical((0..8).map(|a| (1..=3).map(|v| if a & (1 << (v-1)) == 0 { v } else { -v }).collect()).collect());
    let templates = [(6, prism.clone()), (6, complete.clone()), (6, other), (3, unsat)];
    let mut state = 20261127;
    let mut accepted = 0;
    let mut witnesses = 0;
    for (n, target) in templates {
        let count = (0..1u32 << n).filter(|&a| satisfies(&target,a)).count() as u64;
        let witness = (0..1u32 << n).find(|&a| satisfies(&target,a));
        for _ in 0..24 {
            let source = renamed(&target,n,&mut state);
            let map = discover(n,&source,&target,4000).expect("exact signed map");
            assert_eq!(mapped(&source,&map),target);
            assert_eq!(map.iter().map(|v| v.unsigned_abs()).collect::<BTreeSet<_>>().len(),n);
            assert_eq!(dpll(source.clone(),(1<<n)-1,&mut 0),count);
            if let Some(w) = witness {
                assert!(satisfies(&source,reverse(w,&map)));
                witnesses += 1;
            }
            assert!(discover(n,&source,&target,0).is_none());
            accepted += 1;
        }
    }
    for _ in 0..24 {
        let a = renamed(&prism,6,&mut state);
        let b = renamed(&complete,6,&mut state);
        assert!(discover(6,&a,&b,4000).is_none());
        let mut m = discover(6,&a,&prism,4000).expect("map");
        m[0] = -m[0];
        assert_ne!(mapped(&a,&m),prism);
    }
    assert_eq!((accepted,witnesses),(96,72));
}
