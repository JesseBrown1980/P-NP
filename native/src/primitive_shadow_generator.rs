//! Exact primitive-weight shadow composition on consecutive disjoint boundary pairs.
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Edge {
    weight: u128,
    node: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Node {
    level: usize,
    low: Edge,
    high: Edge,
}
struct Dag {
    nodes: Vec<Node>,
    unique: BTreeMap<Node, usize>,
}
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
fn scale(e: Edge, w: u128) -> Edge {
    let weight = e.weight.checked_mul(w).expect("bounded count");
    Edge { weight, node: if weight == 0 { 0 } else { e.node } }
}
impl Dag {
    fn new() -> Self {
        Self { nodes: Vec::new(), unique: BTreeMap::new() }
    }
    fn make(&mut self, level: usize, mut low: Edge, mut high: Edge) -> Edge {
        if low == high { return low; }
        let g = gcd(low.weight, high.weight);
        assert!(g > 0);
        low.weight /= g;
        high.weight /= g;
        let n = Node { level, low, high };
        let id = if let Some(&i) = self.unique.get(&n) {
            i
        } else {
            self.nodes.push(n);
            let i = self.nodes.len();
            self.unique.insert(n, i);
            i
        };
        Edge { weight: g, node: id }
    }
    fn prepend(&mut self, level: usize, table: [u128; 4], suffix: Edge) -> Edge {
        let low = self.make(level + 1, scale(suffix, table[0]), scale(suffix, table[2]));
        let high = self.make(level + 1, scale(suffix, table[1]), scale(suffix, table[3]));
        self.make(level, low, high)
    }
    fn query(&self, mut e: Edge, a: u128) -> u128 {
        let mut w = 1u128;
        loop {
            w = w.checked_mul(e.weight).expect("bounded value");
            if e.node == 0 || w == 0 { return w; }
            let n = self.nodes[e.node - 1];
            e = if a & (1 << n.level) == 0 { n.low } else { n.high };
        }
    }
    fn count(&self, root: Edge, vars: usize) -> (u128, usize) {
        fn rec(d: &Dag, e: Edge, start: usize, vars: usize, cache: &mut BTreeMap<usize, u128>) -> u128 {
            if e.weight == 0 { return 0; }
            if e.node == 0 { return e.weight << (vars - start); }
            let n = d.nodes[e.node - 1];
            let primitive = if let Some(&v) = cache.get(&e.node) { v } else {
                let v = rec(d, n.low, n.level + 1, vars, cache)
                    + rec(d, n.high, n.level + 1, vars, cache);
                cache.insert(e.node, v);
                v
            };
            (e.weight * primitive) << (n.level - start)
        }
        let mut cache = BTreeMap::new();
        let c = rec(self, root, 0, vars, &mut cache);
        (c, cache.len())
    }
}
// Local original CNF semantics: optional selector z allows z=1 only when a=b;
// mandatory equality forces z=a=b. Enumerate only the eight local rows.
fn source(a: bool, b: bool, z: bool, weighted: bool) -> bool {
    if weighted { !z || a == b } else { a == z && b == z }
}
fn local(weighted: bool) -> [u128; 4] {
    std::array::from_fn(|p| {
        [false, true].into_iter().filter(|&z| source(p & 1 != 0, p & 2 != 0, z, weighted)).count() as u128
    })
}
#[test]
fn primitive_shadow_scale_replay() {
    for k in [3, 5, 8, 12, 16, 24, 32] {
        for weighted in [false, true] {
            let table = local(weighted);
            let mut d = Dag::new();
            let mut r = Edge { weight: 1, node: 0 };
            for i in (0..k).rev() { r = d.prepend(2 * i, table, r); }
            assert_eq!(d.nodes.len(), 3 * k);
            let (count, states) = d.count(r, 2 * k);
            assert_eq!(states, 3 * k);
            let expected = if weighted { 6u128 } else { 2u128 };
            assert_eq!(count, expected.pow(k as u32));
            let limit = if k <= 5 { 1usize << (2*k) } else { 256 };
            for i in 0..limit {
                let a = if k <= 5 { i as u128 } else {
                    (i as u128 * 0x9e3779b97f4a7c15) & ((1u128 << (2*k))-1)
                };
                let want = (0..k).map(|j| table[((a >> (2*j)) & 3) as usize]).product::<u128>();
                assert_eq!(d.query(r,a),want);
                if want > 0 {
                    for j in 0..k {
                        let x = a & (1 << (2*j)) != 0;
                        let y = a & (1 << (2*j+1)) != 0;
                        let z = if weighted { false } else { x };
                        assert!(source(x,y,z,weighted));
                    }
                }
            }
            // Alter a shared normalized coefficient: direct local oracle catches it.
            let bad = scale(r, 2);
            assert_ne!(d.query(bad,0),d.query(r,0));
            println!("PRIMITIVE_SHADOW|pairs={k}|weighted={weighted}|nodes={states}|count={count}");
        }
    }
}
