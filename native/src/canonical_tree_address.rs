//! Exact signed canonical tree addresses; route keys using the original Fischer address.
use crate::blind_cnf_factorization::shuffle;
use crate::freeze::{fischer_address, RIME_GLYPHS, RIME_PRIME};
use crate::sha256::hash_bytes;
use std::collections::{BTreeMap, BTreeSet};

type Cnf = Vec<Vec<i32>>;
type Graph = Vec<Vec<(usize, bool)>>;
type Key = Vec<usize>;
type Map = Vec<(usize, bool)>;
type Canonical = (Key, Cnf, Map);
type Bank = BTreeMap<u64, Vec<(Key, Cnf, u128, Option<u128>)>>;

fn norm(mut cs: Cnf) -> Cnf {
    for c in &mut cs { c.sort_unstable(); c.dedup(); }
    cs.sort_unstable();
    cs.dedup();
    cs
}
fn graph(n: usize, cs: &Cnf) -> Graph {
    let mut adj = vec![Vec::new(); n + cs.len()];
    for (i, c) in cs.iter().enumerate() {
        for &v in c {
            let j = v.unsigned_abs() as usize - 1;
            assert!(j < n);
            adj[j].push((n+i, v<0));
            adj[n+i].push((j, v<0));
        }
    }
    adj
}
fn centers(adj: &Graph) -> Option<Vec<usize>> {
    if adj.is_empty() || adj.iter().map(Vec::len).sum::<usize>() != 2*(adj.len()-1) {
        return None;
    }
    let mut seen = BTreeSet::from([0]);
    let mut pending = vec![0];
    while let Some(v) = pending.pop() {
        for &(u, _) in &adj[v] {
            if seen.insert(u) { pending.push(u); }
        }
    }
    if seen.len() != adj.len() { return None; }
    let mut degree: Vec<_> = adj.iter().map(Vec::len).collect();
    let mut leaves: Vec<_> = (0..adj.len()).filter(|&i| degree[i]<=1).collect();
    let mut left = adj.len();
    while left > 2 {
        left -= leaves.len();
        let mut next = Vec::new();
        for v in leaves {
            degree[v] = 0;
            for &(u, _) in &adj[v] {
                if degree[u] > 0 {
                    degree[u] -= 1;
                    if degree[u] == 1 { next.push(u); }
                }
            }
        }
        leaves = next;
    }
    Some(leaves)
}
fn rooted(adj: &Graph, n: usize, v: usize, parent: usize, flip: bool) -> (Key, Map) {
    let variable = v < n;
    let mut children: Vec<_> = adj[v].iter().filter(|&&(u, _)| u != parent).map(|&(u, s)| {
        let (mut key, map) = rooted(adj, n, u, v, !variable && s);
        if variable { key.insert(0, usize::from(s ^ flip)); }
        (key, map)
    }).collect();
    children.sort_by(|a, b| a.0.cmp(&b.0));
    let mut key = vec![usize::from(!variable), children.len()];
    let mut map = Vec::new();
    if variable { map.push((v, flip)); }
    for (k, m) in children { key.extend(k); map.extend(m); }
    (key, map)
}
fn canonical(n: usize, cs: &Cnf) -> Option<Canonical> {
    let adj = graph(n, cs);
    let mut options = Vec::new();
    for v in centers(&adj)? {
        options.push(rooted(&adj, n, v, usize::MAX, false));
        if v < n { options.push(rooted(&adj, n, v, usize::MAX, true)); }
    }
    let (key, map) = options.into_iter().min_by(|a, b| a.0.cmp(&b.0))?;
    assert_eq!(map.len(), n);
    let mut forward = vec![0i32; n];
    for (j, &(v, flip)) in map.iter().enumerate() {
        forward[v] = if flip { -((j+1) as i32) } else { (j+1) as i32 };
    }
    let image = norm(cs.iter().map(|c| c.iter().map(|&v| {
        forward[v.unsigned_abs() as usize-1]*v.signum()
    }).collect()).collect());
    Some((key, image, map))
}
// Independent count: exact clause-to-variable messages on the source incidence tree.
fn message(adj: &Graph, n: usize, v: usize, parent: usize, fixed: &[Option<bool>]) -> [u128;2] {
    let mut weight = [1u128;2];
    if let Some(b) = fixed[v] { weight[usize::from(!b)] = 0; }
    for &(c,s) in &adj[v] {
        if c == parent { continue; }
        let (mut total, mut bad) = (1u128,1u128);
        for &(u,t) in &adj[c] {
            if u == v { continue; }
            assert!(u < n);
            let q = message(adj,n,u,c,fixed);
            total *= q[0]+q[1];
            bad *= q[usize::from(t)];
        }
        weight[0] *= total - if s { 0 } else { bad };
        weight[1] *= total - if s { bad } else { 0 };
    }
    weight
}
fn count(n: usize, cs: &Cnf, fixed: &[Option<bool>]) -> u128 {
    message(&graph(n,cs),n,0,usize::MAX,fixed).iter().sum()
}
fn witness(n: usize, cs: &Cnf, c: u128) -> Option<u128> {
    if c == 0 { return None; }
    let mut f = vec![None;n];
    let mut x = 0u128;
    for i in 0..n {
        f[i] = Some(false);
        if count(n,cs,&f) == 0 {
            f[i] = Some(true);
            x |= 1 << i;
        }
    }
    Some(x)
}
fn sat(cs: &Cnf,x:u128)->bool {
    cs.iter().all(|c| c.iter().any(|&v| (x&(1<<(v.unsigned_abs()-1))!=0)==(v>0)))
}
fn replay(x:u128,map:&Map)->u128 {
    map.iter().enumerate().fold(0,|a,(i,&(v,f))| a|(u128::from((x&(1<<i)!=0)^f)<<v))
}
fn route(key: &Key) -> u64 {
    let bytes: Vec<_> = key.iter().flat_map(|&v| (v as u64).to_le_bytes()).collect();
    let sha = hash_bytes(&bytes);
    let h = u64::from_str_radix(&sha[..16],16).expect("digest");
    let e = h % (RIME_PRIME-1);
    fischer_address(e%RIME_GLYPHS,e/RIME_GLYPHS).expect("bounded address")
}
fn template(n: usize, seed: usize) -> Cnf {
    let mut cs = Vec::new();
    let mut state = (29101+n*100+seed) as u64;
    let mut used = 1;
    while used < n {
        let mut prior: Vec<_> = (1..=used as i32).collect();
        shuffle(&mut prior,&mut state);
        let k = (n-used).min(1+(state as usize)%3);
        let mut c = vec![prior[0]];
        c.extend((used+1..=used+k).map(|v|v as i32));
        for (j,v) in c.iter_mut().enumerate() { if state&(1<<j)!=0 { *v = -*v; } }
        cs.push(c);
        used += k;
    }
    let mut ids:Vec<_> = (1..=n as i32).collect();
    shuffle(&mut ids,&mut state);
    for (j,&v) in ids.iter().take(n/7).enumerate() { cs.push(vec![if j%2==0 {v}else{-v}]); }
    if seed==0 { cs.extend([vec![1],vec![-1]]); }
    norm(cs)
}
fn rename(n:usize,cs:&Cnf,state:&mut u64)->Cnf {
    let mut ids:Vec<_>=(1..=n as i32).collect();
    shuffle(&mut ids,state);
    let mut signs:Vec<_>=(0..n).map(|i|i%2==0).collect();
    shuffle(&mut signs,state);
    for (v,neg) in ids.iter_mut().zip(signs) {if neg {*v = -*v;}}
    norm(cs.iter().map(|c|c.iter().map(|&v|ids[v.unsigned_abs() as usize-1]*v.signum()).collect()).collect())
}
#[test]
fn canonical_tree_generator_and_native_fischer_routing() {
    let mut bank: Bank = BTreeMap::new();
    let (mut cases,mut hits,mut witnesses)=(0,0,0);
    let mut state=20261127;
    for n in [8,16,32,64,96] {
        for seed in 0..6 {
            let cs=template(n,seed);
            let expected=count(n,&cs,&vec![None;n]);
            if n<=16 {
                assert_eq!(expected,(0..1u128<<n).filter(|&x|sat(&cs,x)).count() as u128);
            }
            let (base,_,_)=canonical(n,&cs).expect("tree");
            for _ in 0..12 {
                let src=rename(n,&cs,&mut state);
                let (key,image,map)=canonical(n,&src).expect("tree");
                assert_eq!(base,key);
                let bucket=bank.entry(route(&key)).or_default();
                let (c,w)=if let Some((_,stored,c,w))=bucket.iter().find(|(k,_,_,_)|*k==key) {
                    assert_eq!(*stored,image);
                    hits+=1;
                    (*c,*w)
                } else {
                    let c=count(n,&image,&vec![None;n]);
                    let w=witness(n,&image,c);
                    bucket.push((key,image,c,w));
                    (c,w)
                };
                assert_eq!(c,expected);
                if let Some(w)=w {
                    assert!(sat(&src,replay(w,&map)));
                    witnesses+=1;
                }
                cases+=1;
            }
        }
    }
    assert_eq!(cases,360);
    assert!(hits>=330);
    assert!(witnesses>0);
    // Same tree shape, different signed relation and count. Force the same
    // routing bucket: exact key and full clause image must still distinguish.
    let a=norm(vec![vec![1],vec![1,2]]);
    let b=norm(vec![vec![1],vec![-1,2]]);
    let (ka,ia,_)=canonical(2,&a).expect("tree");
    let (kb,ib,_)=canonical(2,&b).expect("tree");
    assert_ne!(ka,kb);
    let collision=[(ka,ia,2u128),(kb.clone(),ib.clone(),1u128)];
    assert_eq!(collision.iter().find(|(k,c,_)|*k==kb && *c==ib).expect("exact key").2,1);
    assert!(canonical(3,&norm(vec![vec![1,2],vec![2,3],vec![1,3]])).is_none());
    println!("CANONICAL_TREE|cases={cases}|hits={hits}|witnesses={witnesses}");
}

#[test]
fn actual_native_fischer_and_reverse_order_residual_controls() {
    use crate::guidance::{Guidance, Strategy};
    use crate::solver::{check_against_exhaustive, solve, Status};
    use crate::ternary::Cnf as NativeCnf;
    let mut totals = [0u64;4];
    let mut cases = 0;
    for n in [8,12,16] {
        for seed in 0..4 {
            let mut state = (20261127+n*100+seed) as u64;
            let mut ids: Vec<_> = (1..=n as i32).collect();
            let mut cs = Vec::new();
            for _ in 0..5*n {
                shuffle(&mut ids,&mut state);
                cs.push(ids[..3].iter().enumerate().map(|(j,&v)| {
                    if state&(1<<j)==0 {v} else {-v}
                }).collect());
            }
            let cs = norm(cs);
            assert!(canonical(n,&cs).is_none());
            let cnf = NativeCnf { variables:n, clauses:cs };
            let base = Guidance::for_source(&cnf,Strategy::Base).expect("base");
            let fischer = Guidance::for_source(&cnf,Strategy::Fischer).expect("Fischer");
            let mut reverse = fischer.clone();
            reverse.order.reverse();
            let hybrid = Guidance::for_source(&cnf,Strategy::Hybrid3).expect("frozen hybrid");
            assert_eq!(fischer.fischer_addresses,n);
            assert_eq!(hybrid.wave_reads,n);
            assert!(hybrid.root_accepted);
            for (i,g) in [base,fischer,reverse,hybrid].iter().enumerate() {
                let result = solve(&cnf,g,Some(100_000));
                assert_ne!(result.status,Status::Hold);
                assert!(check_against_exhaustive(&cnf,&result));
                totals[i] += result.nodes;
            }
            cases += 1;
        }
    }
    assert_eq!(cases,12);
    println!("NATIVE_FISCHER_RESIDUAL|cases={cases}|base={}|forward={}|reverse={}|hybrid3={}",totals[0],totals[1],totals[2],totals[3]);
}
