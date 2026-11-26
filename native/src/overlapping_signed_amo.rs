//! Signed exclusion-class cover, unique-extension counter rewrite and exact replay.
use crate::blind_cnf_factorization::fixture;
use crate::connected_slice_separator::{evaluate, plan};
use crate::finite_slice_generator::{coordinates, original, parse, recognition, Form};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

type Clause = Vec<Form>;
type Edge = (Form, Form);
type Blocks = Vec<(Vec<(usize, bool)>, Vec<(usize, bool)>)>;

fn neg((m, b): Form) -> Form { (m, !b) }
fn edge(a: Form, b: Form) -> Edge { (a.min(b), a.max(b)) }
fn pairs(xs: &[Form]) -> BTreeSet<Edge> {
    xs.iter().enumerate().flat_map(|(i, &a)| xs[i+1..].iter().map(move |&b| edge(a,b))).collect()
}
fn cover(clauses: &[Clause]) -> (Vec<Vec<Form>>, Vec<Clause>) {
    let mut edges = BTreeSet::new();
    let mut rest = Vec::new();
    for c in clauses {
        if let [a,b] = c.as_slice() {
            if a.0.count_ones() == 1 && b.0.count_ones() == 1 && a.0 != b.0 {
                edges.insert(edge(neg(*a),neg(*b)));
                continue;
            }
        }
        rest.push(c.clone());
    }
    let mut adj: BTreeMap<Form,BTreeSet<Form>> = BTreeMap::new();
    for &(a,b) in &edges {
        adj.entry(a).or_default().insert(b);
        adj.entry(b).or_default().insert(a);
    }
    let mut left = edges.clone();
    let mut classes = Vec::new();
    let mut certified = BTreeSet::new();
    while let Some(&(a,b)) = left.first() {
        let mut cl = vec![a,b];
        let mut candidates: Vec<_> = adj[&a].intersection(&adj[&b]).copied().collect();
        candidates.sort_by_key(|v| (std::cmp::Reverse(adj[v].len()), *v));
        for v in candidates {
            if cl.iter().all(|u| u.0 != v.0 && adj[&v].contains(u)) { cl.push(v); }
        }
        let covered = pairs(&cl);
        assert!(covered.is_subset(&edges));
        certified.extend(covered.iter().copied());
        for e in covered { left.remove(&e); }
        if cl.len() >= 3 {
            cl.sort_unstable();
            classes.push(cl);
        } else {
            rest.push(vec![neg(a),neg(b)]);
        }
    }
    assert_eq!(certified, edges);
    (classes, rest)
}
fn rewrite(mut n: usize, classes: &[Vec<Form>], mut rest: Vec<Clause>) -> (usize, Vec<Clause>) {
    for cl in classes {
        let mut prev = cl[0];
        for (j, &v) in cl.iter().enumerate().skip(1) {
            rest.push(vec![neg(prev),neg(v)]);
            if j + 1 == cl.len() { break; }
            assert!(n < 128);
            let next = (1u128 << n,false);
            n += 1;
            rest.push(vec![neg(prev),next]);
            rest.push(vec![neg(v),next]);
            rest.push(vec![prev,v,neg(next)]);
            prev = next;
        }
    }
    (n,rest)
}
fn sign(state: &mut u64) -> bool {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state & 1 != 0
}
fn make(k: usize,g: usize,cycle: bool,family: usize,seed: usize,missing: bool) -> (String,Blocks,usize) {
    let offset = g + usize::from(!cycle);
    let n = offset + g * (k-2);
    let (base,_,y) = fixture(n,family,seed,0);
    let (nv,mut cs) = parse(&base);
    cs.retain(|c| !c.iter().all(|v| y.contains(&v.abs())));
    let mut blocks = Vec::new();
    let mut state = (63190 + k*1000 + g*100 + usize::from(cycle)*20 + family*10 + seed) as u64;
    for i in 0..g {
        let ids: Vec<_> = [i,if cycle {(i+1)%g} else {i+1}].into_iter()
            .chain(offset+i*(k-2)..offset+(i+1)*(k-2)).collect();
        let cl: Vec<_> = ids.iter().map(|&j| (j,sign(&mut state))).collect();
        let local: Vec<_> = [ids[0],ids[2],ids[3]].into_iter().map(|j| (j,sign(&mut state))).collect();
        let lit = |(j,b): (usize,bool)| if b {-y[j]} else {y[j]};
        for (a,&u) in cl.iter().enumerate() {
            for (b,&v) in cl.iter().enumerate().skip(a+1) {
                if !(missing && i==0 && a==0 && b==2) { cs.push(vec![-lit(u),-lit(v)]); }
            }
        }
        cs.push(local.iter().copied().map(lit).collect());
        blocks.push((cl,local));
    }
    let mut text = format!("p cnf {nv} {}\n",cs.len());
    for c in cs {
        for v in c { write!(&mut text,"{v} ").expect("write"); }
        text.push_str("0\n");
    }
    (text,blocks,n)
}
fn oracle(blocks: &Blocks,cycle: bool,missing: bool) -> u128 {
    let mut tables = Vec::new();
    for (i,(cl,local)) in blocks.iter().enumerate() {
        let mut t = [[0u128;2];2];
        for a in 0..1usize << cl.len() {
            let active: Vec<_> = cl.iter().enumerate().map(|(j,&(_,b))| (a>>j&1 != 0)^b).collect();
            let count = active.iter().filter(|&&v| v).count();
            if count>1 && !(missing && i==0 && count==2 && active[0] && active[2]) { continue; }
            if !local.iter().any(|&(j,b)| {
                let p = cl.iter().position(|&(v,_)| v==j).expect("local variable");
                (a>>p&1 != 0)^b
            }) { continue; }
            t[a&1][a>>1&1] += 1;
        }
        tables.push(t);
    }
    let mut total = 0;
    for first in 0..2 {
        let mut row = [0u128;2];
        row[first] = 1;
        for t in &tables {
            row = [row[0]*t[0][0]+row[1]*t[1][0],row[0]*t[0][1]+row[1]*t[1][1]];
        }
        total += if cycle {row[first]} else {row.iter().sum()};
    }
    total
}
fn trial(k: usize,g: usize,cycle: bool,family: usize,seed: usize,missing: bool) -> (usize,usize) {
    let (text,blocks,n) = make(k,g,cycle,family,seed,missing);
    let (nv,source) = parse(&text);
    let (eq,res) = recognition(&source);
    let (forms,free) = coordinates(nv,eq,&res).expect("consistent parity");
    assert_eq!(free,n);
    let clauses: Vec<Clause> = res.iter().map(|c| c.iter().map(|&v| {
        let (m,b) = forms[v.unsigned_abs() as usize-1];(m,b^(v<0))
    }).collect()).collect();
    let (classes,rest) = cover(&clauses);
    if missing {
        assert!(classes.iter().filter(|c| c.len()==k).count() < g);
    } else {
        assert_eq!(classes.iter().filter(|c| c.len()==k).count(),g);
    }
    let (expanded,rewritten) = rewrite(n,&classes,rest);
    let (order,width) = plan(expanded,&rewritten).expect("bounded rewritten scope");
    let result = evaluate(&rewritten,&order);
    assert_eq!(result.count,oracle(&blocks,cycle,missing));
    if !missing {
        assert!(width<=4);
        assert_eq!(plan(n,&clauses).is_some(),k<=6);
    }
    if result.count>0 {
        let x = forms.iter().enumerate().fold(0u128,|a,(j,&(m,b))| {
            a | (u128::from(((result.witness&m).count_ones()%2!=0)^b)<<j)
        });
        assert!(original(&source,x));
        let bad = result.witness ^ (1u128<<n);
        assert!(!rewritten.iter().all(|c| c.iter().any(|&(m,b)| ((bad&m).count_ones()%2!=0)^b)));
    }
    (result.entries,width)
}
#[test]
fn overlapping_signed_classes_exact_count() {
    let mut cases = 0;
    let mut entries = 0;
    for (k,g) in [(4,3),(8,3),(12,3),(8,6),(12,5)] {
        for cycle in [false,true] {
            for family in 0..2 {
                for seed in 0..2 {
                    entries += trial(k,g,cycle,family,seed,false).0;
                    cases += 1;
                }
            }
        }
    }
    // Smaller near-cliques ensure the native width cap is sufficient for every shuffle.
    let mut missing = 0;
    for cycle in [false,true] {
        for family in 0..2 {
            for seed in 0..2 {
                trial(4,3,cycle,family,seed,true);
                missing += 1;
            }
        }
    }
    assert_eq!((cases,missing),(40,8));
    println!("OVERLAPPING_AMO|cases={cases}|missing_edge={missing}|entries={entries}");
}
