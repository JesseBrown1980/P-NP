//! Exact conditioned residual replay with dynamically discovered independent components.
use crate::blind_cnf_factorization::shuffle;
use std::collections::BTreeMap;

fn bits(mut m:u128)->impl Iterator<Item=usize> {
    std::iter::from_fn(move || {
        if m==0 {return None;}
        let v=m.trailing_zeros() as usize;
        m&=m-1;
        Some(v)
    })
}
fn components(mut m:u128,adj:&[u128])->Vec<u128> {
    let mut out=Vec::new();
    while m!=0 {
        let mut todo=1u128<<m.trailing_zeros();
        let mut c=0;
        while todo!=0 {
            let v=todo.trailing_zeros() as usize;
            let b=1u128<<v;
            todo^=b;
            if m&b==0 {continue;}
            m^=b;
            c|=b;
            todo|=adj[v]&m;
        }
        out.push(c);
    }
    out
}
#[derive(Clone)]
enum Trace {
    Empty,
    Include(usize,u128),
    Split(Vec<u128>),
}
type Cache=BTreeMap<u128,(u128,Trace)>;
fn solve(m:u128,adj:&[u128],order:Option<&[usize]>,cache:&mut Cache)->Option<u128> {
    if let Some((c,_))=cache.get(&m) {return Some(*c);}
    if cache.len()>=60_000 {return None;}
    if order.is_none() {
        let groups=components(m,adj);
        if groups.len()>1 {
            let mut c=1u128;
            for &g in &groups {c=c.checked_mul(solve(g,adj,order,cache)?)?;}
            cache.insert(m,(c,Trace::Split(groups)));
            return Some(c);
        }
    }
    let v=if let Some(o)=order {
        o.iter().copied().find(|&v|m&(1<<v)!=0)?
    } else {
        bits(m).min_by_key(|&v|(std::cmp::Reverse((adj[v]&m).count_ones()),v))?
    };
    let rem=m^(1<<v);
    let included=rem&!adj[v];
    let c=solve(rem,adj,order,cache)?.checked_add(solve(included,adj,order,cache)?)?;
    cache.insert(m,(c,Trace::Include(v,included)));
    Some(c)
}
fn replay(m:u128,cache:&Cache)->u128 {
    let (count,t)=&cache[&m];
    assert!(*count>0);
    match t {
        Trace::Empty=>0,
        Trace::Include(v,r)=>(1<<v)|replay(*r,cache),
        Trace::Split(gs)=>gs.iter().map(|&g|replay(g,cache)).fold(0,|a,b|a|b),
    }
}
fn neighborhood(adj:&[u128])->Vec<usize> {
    let mut m=(1u128<<adj.len())-1;
    let mut f=0;
    let mut order=Vec::new();
    while m!=0 {
        let cand=if m&f!=0 {m&f}else{m};
        let v=bits(cand).min_by_key(|&i|(std::cmp::Reverse(adj[i].count_ones()),i)).expect("candidate");
        order.push(v);
        m^=1<<v;
        f|=adj[v];
    }
    order
}
fn graph(n:usize,edges:&[(usize,usize)],seed:u64)->Vec<u128> {
    let mut ids:Vec<_>=(0..n).collect();
    let mut state=seed;
    shuffle(&mut ids,&mut state);
    let mut a=vec![0;n];
    for &(x,y) in edges {
        a[ids[x]]|=1<<ids[y];
        a[ids[y]]|=1<<ids[x];
    }
    a
}
fn path_count(n:usize)->u128 {
    let (mut a,mut b)=(1,2);
    for _ in 1..n {(a,b)=(b,a+b);}
    b
}
fn clique_count(k:usize,g:usize)->u128 {
    let k=k as u128;
    let (mut a,mut b)=(k,1);
    for _ in 1..g {(a,b)=((a+b)*k,a);}
    a+b
}
fn grid_count(r:usize,c:usize)->u128 {
    let rows:Vec<_>=(0..1usize<<c).filter(|&a|a&(a<<1)==0).collect();
    let mut dp=vec![0u128;rows.len()];
    dp[0]=1;
    for _ in 0..r {
        dp=rows.iter().map(|&a|rows.iter().zip(&dp).filter(|(&b,_)|a&b==0).map(|(_,v)|v).sum()).collect();
    }
    dp.iter().sum()
}
fn independent_small(adj:&[u128])->u128 {
    // Independent split enumeration and right-side subset zeta transform.
    let l=adj.len()/2;
    let r=adj.len()-l;
    let mut t:Vec<u128>=(0..1usize<<r).map(|a|u128::from(bits(a as u128).all(|i|((adj[l+i]>>l)&a as u128)==0))).collect();
    for i in 0..r {
        for a in 0..1usize<<r {
            if a&(1<<i)!=0 {let v=t[a^(1<<i)];t[a]+=v;}
        }
    }
    let mut total=0;
    for a in 0..1u128<<l {
        if bits(a).any(|i|adj[i]&a!=0) {continue;}
        let f=bits(a).fold(0,|m,i|m|(adj[i]>>l));
        total+=t[(((1u128<<r)-1)&!f) as usize];
    }
    total
}
fn trial(adj:&[u128],want:u128)->(usize,usize) {
    let m=(1u128<<adj.len())-1;
    assert_eq!(components(m,adj).len(),1);
    if adj.len()<=24 {assert_eq!(independent_small(adj),want);}
    let o=neighborhood(adj);
    let mut sizes=Vec::new();
    for order in [Some(o.as_slice()),None] {
        let mut cache=Cache::from([(0,(1,Trace::Empty))]);
        assert_eq!(solve(m,adj,order,&mut cache),Some(want));
        let w=replay(m,&cache);
        assert!(bits(w).all(|v|adj[v]&w==0));
        sizes.push(cache.len());
    }
    (sizes[0],sizes[1])
}
#[test]
fn automatic_connected_shadow_replay() {
    let (mut cases,mut fixed,mut adaptive)=(0,0,0);
    let mut run=|n,edges:&[(usize,usize)],want:Option<u128>| {
        for seed in [719,823] {
            let adj=graph(n,edges,seed);
            let (a,b)=trial(&adj,want.unwrap_or_else(||independent_small(&adj)));
            fixed+=a;
            adaptive+=b;
            cases+=1;
        }
    };
    for n in [12,24,48,72] {
        let e:Vec<_>=(0..n-1).map(|i|(i,i+1)).collect();
        run(n,&e,Some(path_count(n)));
    }
    for (k,g) in [(4,4),(8,4),(12,4),(12,6)] {
        let mut e=Vec::new();
        for i in 0..g {
            for a in 0..k {
                for b in a+1..k {e.push((i*k+a,i*k+b));}
            }
        }
        for i in 0..g-1 {e.push((i*k,(i+1)*k));}
        run(k*g,&e,Some(clique_count(k,g)));
    }
    for (r,c) in [(3,4),(4,6),(5,6),(6,6)] {
        let mut e=Vec::new();
        for i in 0..r {
            for j in 0..c {
                if j+1<c {e.push((i*c+j,i*c+j+1));}
                if i+1<r {e.push((i*c+j,(i+1)*c+j));}
            }
        }
        run(r*c,&e,Some(grid_count(r,c)));
    }
    for n in [12,18,24] {
        let mut e:Vec<_>=(0..n).map(|i|(i,(i+1)%n)).collect();
        let mut ids:Vec<_>=(0..n).collect();
        let mut state=812+n as u64;
        for _ in 0..n {
            shuffle(&mut ids,&mut state);
            e.push((ids[0],ids[1]));
        }
        run(n,&e,None);
    }
    assert_eq!(cases,30);
    println!("AUTO_SHADOW|cases={cases}|neighborhood_states={fixed}|adaptive_states={adaptive}");
}
