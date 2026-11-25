//! Exact research control for reversible Boolean coordinates and support closure.
//! Input parity forms are explicit; arbitrary CNF discovery is a separate gate.
#![forbid(unsafe_code)]
use std::collections::BTreeMap;

#[derive(Clone)]
struct Clause {
    forms: Vec<u32>,
    forbidden: u32,
}
fn parity(x: u32) -> u32 {
    x.count_ones() & 1
}
fn evaluate(forms: &[u32], x: u32) -> u32 {
    forms.iter().enumerate().map(|(j, &r)| parity(r & x) << j).sum()
}
fn valid(cs: &[Clause], x: u32) -> bool {
    cs.iter().all(|c| evaluate(&c.forms, x) != c.forbidden)
}
fn fixture(n: usize, family: usize, seed: u32, bridge: bool) -> Vec<Clause> {
    let rows: Vec<u32> = std::iter::once(1)
        .chain((1..n).map(|i| (1 << i) ^ if family == 0 { 1 << (i-1) } else { 1 }))
        .collect();
    let mut cs = Vec::new();
    for (i, forms) in rows.chunks_exact(3).enumerate() {
        let p = (seed + 2 * i as u32) % 8;
        for forbidden in [p, p ^ 7] {
            cs.push(Clause { forms: forms.to_vec(), forbidden });
        }
    }
    if bridge {
        cs.push(Clause { forms: vec![rows[2],rows[3]], forbidden: 3 });
    }
    cs
}
fn recover(n: usize, cs: &[Clause]) -> (Vec<u32>, Vec<Clause>) {
    let mut pivots: BTreeMap<u32,(u32,u32)> = BTreeMap::new();
    let mut basis = Vec::new();
    let mut enc = BTreeMap::new();
    for c in cs {
        for &row in &c.forms {
            if enc.contains_key(&row) { continue; }
            let mut t = row;
            let mut rep = 0;
            while t != 0 {
                let key = t.ilog2();
                if let Some(&(p,r)) = pivots.get(&key) {
                    t ^= p;
                    rep ^= r;
                } else {
                    let code = 1 << basis.len();
                    basis.push(row);
                    pivots.insert(key,(t,rep ^ code));
                    rep = code;
                    break;
                }
            }
            enc.insert(row,rep);
        }
    }
    assert_eq!(basis.len(),n);
    let transformed = cs.iter().map(|c| Clause {
        forms: c.forms.iter().map(|r| enc[r]).collect(),
        forbidden: c.forbidden,
    }).collect();
    (basis,transformed)
}
fn submasks(m: u32) -> impl Iterator<Item = u32> {
    std::iter::successors(Some(m),move |&s| if s == 0 { None } else { Some((s-1)&m) })
}
fn support(c: &Clause) -> u32 {
    c.forms.iter().fold(0,|m,r| m | r)
}
// Ordinary CNF clause encoded by its variable mask and unique falsifying pattern.
fn compile_cnf(cs: &[Clause]) -> Vec<(u32,u32)> {
    let mut out = Vec::new();
    for c in cs {
        let m = support(c);
        for s in submasks(m) {
            if evaluate(&c.forms,s) == c.forbidden { out.push((m,s)); }
        }
    }
    out
}
fn factor(n: usize, cs: &[Clause]) -> (u64,u64) {
    let mut groups: Vec<u32> = (0..n).map(|i| 1 << i).collect();
    for c in cs {
        let m = support(c);
        let mut joined = 0;
        groups.retain(|g| {
            if g & m != 0 { joined |= *g; false } else { true }
        });
        assert_eq!(joined & m,m);
        groups.push(joined);
    }
    let mut count = 1;
    let mut work = 0;
    for g in groups {
        let local: Vec<Clause> = cs.iter().filter(|c| support(c)&g != 0).cloned().collect();
        assert!(local.iter().all(|c| support(c)&!g == 0));
        let mut subtotal = 0;
        for s in submasks(g) {
            subtotal += u64::from(valid(&local,s));
            work += 1;
        }
        count *= subtotal;
    }
    (count,work)
}
#[test]
fn recovered_boolean_address_factorization() {
    let (mut cases,mut checks,mut work,mut clauses,mut controls) = (0u64,0u64,0u64,0u64,0u64);
    for n in [6,9,12,15] {
        for family in 0..2 {
            for seed in 0..3 {
                for bridge in [false,true] {
                    let cs = fixture(n,family,seed,bridge);
                    let (basis,tc) = recover(n,&cs);
                    let cnf = compile_cnf(&cs);
                    let (count,cost) = factor(n,&tc);
                    let mut reference = 0;
                    let mut bad = false;
                    let mut seen = vec![false; 1 << n];
                    for x in 0..(1 << n) {
                        let y = evaluate(&basis,x);
                        assert!(!seen[y as usize]);
                        seen[y as usize] = true;
                        let expected = cnf.iter().all(|&(m,p)| x&m != p);
                        assert_eq!(expected,valid(&tc,y));
                        reference += u64::from(expected);
                        if bridge && valid(&tc[..tc.len()-1],y) != expected { bad = true; }
                    }
                    assert_eq!(reference,count);
                    assert!(seen.iter().all(|&v| v));
                    if bridge {
                        assert!(bad && factor(n,&tc[..tc.len()-1]).0 > count);
                        controls += 1;
                    }
                    cases += 1;
                    checks += 1 << n;
                    work += cost;
                    clauses += cnf.len() as u64;
                    println!("ADDRESS_FACTOR|n={n}|family={family}|seed={seed}|bridge={bridge}|count={count}|local_assignments={cost}");
                }
            }
        }
    }
    assert_eq!((cases,checks,work,clauses,controls),(48,449280,2496,624,24));
    println!("ADDRESS_FACTOR_TOTAL|cases={cases}|assignment_checks={checks}|local_assignments={work}|cnf_clauses={clauses}|omission_rejected={controls}");
}
