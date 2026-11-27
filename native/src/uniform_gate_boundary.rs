//! Exact bounded truth-relation discovery and a general SAT reduction stress control.
use crate::blind_cnf_factorization::{dpll, shuffle};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

type Cnf = Vec<Vec<i32>>;
type Table = (Vec<usize>, Vec<bool>);

fn arity(g: usize) -> usize {
    if g < 2 { 2 } else { 3 }
}
fn value(g: usize, a: usize) -> bool {
    match g {
        0 => a == 3,
        1 => a.count_ones() % 2 == 1,
        2 => a.count_ones() >= 2,
        3 => if a & 1 != 0 { a & 2 != 0 } else { a & 4 != 0 },
        4 => a != 0,
        _ => unreachable!(),
    }
}
fn encode(g: usize, ins: &[i32], out: i32) -> Cnf {
    assert_eq!(ins.len(), arity(g));
    (0..1usize << ins.len()).map(|a| {
        let mut row: Vec<_> = ins.iter().enumerate().map(|(i, &v)| {
            if a & (1 << i) != 0 { -v } else { v }
        }).collect();
        row.push(if value(g, a) { out } else { -out });
        row
    }).collect()
}
fn hide(n: usize, cs: Cnf, seed: u64) -> String {
    let mut state = seed;
    let mut names: Vec<_> = (1..=n as i32).collect();
    shuffle(&mut names, &mut state);
    let mut cs: Cnf = cs.iter().map(|c| c.iter().map(|&v| {
        names[v.unsigned_abs() as usize - 1] * v.signum()
    }).collect()).collect();
    shuffle(&mut cs, &mut state);
    let mut text = format!("p cnf {n} {}\n", cs.len());
    for c in &mut cs {
        shuffle(c, &mut state);
        for v in c {
            write!(&mut text, "{v} ").expect("write");
        }
        text.push_str("0\n");
    }
    text
}
fn discover(text: &str) -> (usize, Vec<Table>) {
    let mut lines = text.lines();
    let header: Vec<_> = lines.next().expect("header").split_whitespace().collect();
    assert_eq!(&header[..2], &["p", "cnf"]);
    let n: usize = header[2].parse().expect("n");
    let count: usize = header[3].parse().expect("m");
    let mut groups: BTreeMap<Vec<usize>, Vec<bool>> = BTreeMap::new();
    let mut actual = 0;
    for line in lines {
        let mut c: Vec<i32> = line.split_whitespace().map(|s| s.parse().expect("literal")).collect();
        assert_eq!(c.pop(), Some(0));
        let mut key: Vec<_> = c.iter().map(|v| v.unsigned_abs() as usize).collect();
        key.sort_unstable();
        assert!(key.len() <= 4 && key.windows(2).all(|w| w[0] != w[1]));
        assert!(key.iter().all(|&v| v > 0 && v <= n));
        let p = key.iter().enumerate().fold(0, |p, (i, &v)| {
            p | (usize::from(c.contains(&-(v as i32))) << i)
        });
        groups.entry(key).or_insert_with(|| vec![true; 1 << c.len()])[p] = false;
        actual += 1;
    }
    assert_eq!(actual, count);
    (n, groups.into_iter().collect())
}
fn signatures() -> BTreeSet<(usize, usize)> {
    let mut out = BTreeSet::new();
    let permutations = [[0,1,2],[0,2,1],[1,0,2],[1,2,0],[2,0,1],[2,1,0]];
    for g in 0..5 {
        let k = arity(g);
        for p in permutations {
            if p[..k].iter().any(|&i| i >= k) { continue; }
            for flip in 0..1usize << (k+1) {
                let mut mask = 0;
                for a in 0..1usize << k {
                    let b = p[..k].iter().enumerate().fold(0, |b, (j, &i)| {
                        b | ((((a >> i) ^ (flip >> j)) & 1) << j)
                    });
                    mask |= usize::from(value(g, b) ^ (flip >> k != 0)) << a;
                }
                out.insert((k, mask));
            }
        }
    }
    out
}
fn recognized((key,t): &Table, lib: &BTreeSet<(usize,usize)>) -> bool {
    let k = key.len();
    if !(3..=4).contains(&k) || t.iter().filter(|&&v| v).count() != 1 << (k-1) {
        return false;
    }
    (0..k).any(|o| {
        let mut mask = 0;
        for a in 0..1usize << (k-1) {
            let lo = (a & ((1 << o)-1)) | ((a >> o) << (o+1));
            let hi = lo | (1 << o);
            if t[lo] == t[hi] { return false; }
            mask |= usize::from(t[hi]) << a;
        }
        lib.contains(&(k-1,mask))
    })
}
fn propagate(mut tables: Vec<Table>) -> (Vec<Table>, BTreeMap<usize,bool>) {
    let mut fixed = BTreeMap::new();
    loop {
        let mut todo = BTreeMap::new();
        for (key,t) in &tables {
            assert!(t.iter().any(|&v| v));
            for (j,&v) in key.iter().enumerate() {
                let zero = t.iter().enumerate().any(|(a,&b)| b && a & (1 << j) == 0);
                let one = t.iter().enumerate().any(|(a,&b)| b && a & (1 << j) != 0);
                if zero != one {
                    if let Some(old) = todo.insert(v,one) {
                        assert_eq!(old,one);
                    }
                }
            }
        }
        if todo.is_empty() { return (tables,fixed); }
        for (&v,&b) in &todo { assert!(fixed.insert(v,b).is_none()); }
        let mut next = Vec::new();
        for (key,t) in tables {
            let keep: Vec<_> = key.iter().filter(|v| !todo.contains_key(v)).copied().collect();
            let new: Vec<_> = (0..1usize << keep.len()).map(|a| {
                let p = key.iter().enumerate().fold(0, |p,(j,v)| {
                    let bit = todo.get(v).copied().unwrap_or_else(|| {
                        let i = keep.iter().position(|w| w == v).expect("free variable");
                        a & (1 << i) != 0
                    });
                    p | (usize::from(bit) << j)
                });
                t[p]
            }).collect();
            if !new.iter().all(|&v| v) { next.push((keep,new)); }
        }
        tables = next;
    }
}
fn source_formula(n: usize, seed: u64) -> Cnf {
    let mut state = seed;
    let mut vars: Vec<_> = (1..=n as i32).collect();
    let mut cs = BTreeSet::new();
    while cs.len() < 4*n {
        shuffle(&mut vars,&mut state);
        let mut c: Vec<_> = vars[..3].iter().enumerate().map(|(j,&v)| {
            if state & (1 << j) == 0 { v } else { -v }
        }).collect();
        c.sort_unstable_by_key(|v| v.abs());
        cs.insert(c);
    }
    cs.into_iter().collect()
}
fn circuit(n: usize, cs: &Cnf, seed: u64) -> (String, usize) {
    let mut next = n;
    let mut clauses = Vec::new();
    let mut outputs = Vec::new();
    for c in cs {
        next += 1;
        clauses.extend(encode(4,c,next as i32));
        outputs.push(next as i32);
    }
    while outputs.len() > 1 {
        let mut row = Vec::new();
        for pair in outputs.chunks(2) {
            if pair.len() == 1 { row.push(pair[0]); }
            else {
                next += 1;
                clauses.extend(encode(0,pair,next as i32));
                row.push(next as i32);
            }
        }
        outputs = row;
    }
    clauses.push(vec![outputs[0]]);
    (hide(next,clauses,seed),next-n)
}

#[test]
fn blind_nonlinear_relation_and_missing_row_controls() {
    let lib = signatures();
    let mut checked = 0;
    for g in 0..5 {
        let k = arity(g);
        for flip in 0..1usize << (k+1) {
            let ins: Vec<_> = (1..=k as i32).enumerate().map(|(i,v)| {
                if flip & (1 << i) != 0 { -v } else { v }
            }).collect();
            let out = if flip >> k != 0 { -((k+1) as i32) } else { (k+1) as i32 };
            let mut cs = encode(g,&ins,out);
            let (_,tables) = discover(&hide(k+1,cs.clone(),(g*100+flip+1) as u64));
            assert_eq!(tables.len(),1);
            assert!(recognized(&tables[0],&lib));
            cs.remove(0);
            let (_,tables) = discover(&hide(k+1,cs,(g*100+flip+1) as u64));
            assert!(!recognized(&tables[0],&lib));
            checked += 1;
        }
    }
    assert_eq!(checked,64);
}

#[test]
fn complete_gate_discovery_does_not_remove_general_sat() {
    let lib = signatures();
    let (mut cases,mut gates,mut sat) = (0,0,0);
    for n in [8,12,16,20,24] {
        for seed in 0..3 {
            let cs = source_formula(n,(24001+n*100+seed) as u64);
            let (text,expected) = circuit(n,&cs,(80001+n*10+seed) as u64);
            let (nv,tables) = discover(&text);
            assert_eq!(tables.iter().filter(|t| recognized(t,&lib)).count(),expected);
            let (res,fixed) = propagate(tables);
            assert_eq!(fixed.len(),expected);
            let free: Vec<_> = (1..=nv).filter(|v| !fixed.contains_key(v)).collect();
            assert_eq!(free.len(),n);
            let remap: BTreeMap<_,_> = free.iter().enumerate().map(|(i,&v)| (v,(i+1) as i32)).collect();
            let recovered: Cnf = res.iter().map(|(key,t)| {
                let bad: Vec<_> = t.iter().enumerate().filter(|(_,b)| !**b).map(|(a,_)| a).collect();
                assert_eq!(bad.len(),1);
                key.iter().enumerate().map(|(j,v)| {
                    if bad[0] & (1 << j) != 0 { -remap[v] } else { remap[v] }
                }).collect()
            }).collect();
            assert_eq!(recovered.len(),cs.len());
            let (mut a,mut b) = (0,0);
            let want = dpll(cs,(1 << n)-1,&mut a);
            let got = dpll(recovered,(1 << n)-1,&mut b);
            assert_eq!(got,want);
            cases += 1;
            gates += expected;
            sat += usize::from(got>0);
            println!("UNIFORM_GATE|n={n}|seed={seed}|gates={expected}|count={got}|source_nodes={a}|recovered_nodes={b}");
        }
    }
    assert_eq!((cases,gates),(15,1905));
    println!("UNIFORM_GATE_TOTAL|cases={cases}|recognized={gates}|sat={sat}");
}
