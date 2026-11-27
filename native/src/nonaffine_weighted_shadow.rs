//! Weighted non-affine shadow projection: exact local counts and reverse witnesses.
use crate::blind_cnf_factorization::shuffle;
use crate::finite_slice_generator::original;
use crate::connected_slice_separator::plan;
use crate::freeze::FrozenMatrix;
use crate::matrix::{Matrix,CELLS};

type Cnf = Vec<Vec<i32>>;
type Factor = (u128, Vec<u128>);
type Choice = (u128, u128, Vec<Option<usize>>);
type Spec = (Vec<usize>, bool);

fn bits(m: u128) -> impl Iterator<Item=usize> {
    (0..128).filter(move |&i| m & (1 << i) != 0)
}
fn pack(x: u128, m: u128) -> usize {
    bits(m).enumerate().fold(0, |a,(j,i)| a | ((((x>>i)&1) as usize)<<j))
}
fn unpack(x: usize, m: u128) -> u128 {
    bits(m).enumerate().fold(0, |a,(j,i)| a | (u128::from(((x>>j)&1) as u8)<<i))
}
fn mask(c: &[i32]) -> u128 {
    c.iter().fold(0, |m,v| m | (1 << (v.unsigned_abs()-1)))
}
fn discover(n: usize, cs: &Cnf) -> Vec<u128> {
    let mut adj = vec![0u128;n];
    for c in cs {
        let m = mask(c);
        for i in bits(m) { adj[i] |= m ^ (1<<i); }
    }
    let mut pairs = Vec::new();
    let mut freq = vec![0;n];
    for (i,&m) in adj.iter().enumerate() {
        if m.count_ones()!=3 { continue; }
        for j in bits(m).filter(|&j|j>i) {
            if adj[j].count_ones()==3 && (m^(1<<j))==(adj[j]^(1<<i)) {
                pairs.push((1u128<<i)|(1u128<<j));
                freq[i]+=1;
                freq[j]+=1;
            }
        }
    }
    pairs.into_iter().filter(|&m|bits(m).all(|i|freq[i]==1)).collect()
}
fn reduce(fs: &mut Vec<Factor>, inside: u128) -> Choice {
    let old = std::mem::take(fs);
    let (bucket,mut rest):(Vec<_>,Vec<_>) = old.into_iter().partition(|(m,_)|m&inside!=0);
    let u = bucket.iter().fold(inside,|m,(s,_)|m|s);
    assert!(u.count_ones()<=6);
    let b = u^inside;
    let mut counts = vec![0;1<<b.count_ones()];
    let mut picks = vec![None;counts.len()];
    let weight = |x| bucket.iter().map(|(m,t)|t[pack(x,*m)]).product::<u128>();
    for (i,count) in counts.iter_mut().enumerate() {
        for j in 0..1usize<<inside.count_ones() {
            let w=weight(unpack(i,b)|unpack(j,inside));
            *count+=w;
            if w>0 && picks[i].is_none() { picks[i]=Some(j); }
        }
    }
    let mut check = vec![0;counts.len()];
    for i in (0..1usize<<u.count_ones()).rev() {
        let x=unpack(i,u);
        check[pack(x,b)]+=weight(x);
    }
    assert_eq!(check,counts);
    rest.push((b,counts));
    *fs=rest;
    (inside,b,picks)
}
fn fixture(k:usize,mode:usize,seed:usize) -> (Cnf,Vec<Spec>) {
    let mut cs=Vec::new();
    let mut specs=Vec::new();
    let mut state=(83101+100*k+seed) as u64;
    for i in 0..k {
        let (a,b,u,v)=((i+1) as i32,((i+1)%k+1) as i32,(k+2*i+1) as i32,(k+2*i+2) as i32);
        let mut ts=vec![0,1,2,3];
        shuffle(&mut ts,&mut state);
        let mandatory=mode==2 || (mode==1 && i%3==0);
        if mode==1 && i%3==0 {
            ts.truncate(3);
            for (j,&t) in ts.iter().enumerate() {
                let p=if j&1==0 {u}else{-u};
                let q=if j&2==0 {v}else{-v};
                cs.push(vec![p,q,if t&1!=0 {a}else{-a}]);
                cs.push(vec![p,q,if t&2!=0 {b}else{-b}]);
            }
            cs.push(vec![-u,-v]);
        } else {
            ts.truncate(2);
            if mode==2 {ts=if i+1==k {vec![1,2]}else{vec![0,3]};}
            if mode==3 {ts[1]=ts[0];}
            for (&z,&t) in [u,v].iter().zip(&ts) {
                cs.push(vec![-z,if t&1!=0 {a}else{-a}]);
                cs.push(vec![-z,if t&2!=0 {b}else{-b}]);
            }
            cs.push(vec![-u,-v]);
            if mandatory {cs.push(vec![u,v]);}
        }
        specs.push((ts,mandatory));
    }
    let mut names:Vec<_>=(1..=3*k as i32).collect();
    shuffle(&mut names,&mut state);
    for c in &mut cs {
        for v in c { *v=names[v.unsigned_abs() as usize-1]*v.signum(); }
    }
    shuffle(&mut cs,&mut state);
    (cs,specs)
}
fn oracle(specs:&[Spec]) -> u128 {
    let mut total=0;
    for first in 0..2 {
        let mut row=[0u128;2];
        row[first]=1;
        for (ts,m) in specs {
            let mut next=[0;2];
            for (b,v) in next.iter_mut().enumerate() {
                for (a,&r) in row.iter().enumerate() {
                    let w=u128::from(!*m)+ts.iter().filter(|&&t|t==a+2*b).count() as u128;
                    *v+=r*w;
                }
            }
            row=next;
        }
        total+=row[first];
    }
    total
}
#[test]
fn blind_weighted_shadows_and_reverse_recovery() {
    let (mut cases,mut groups,mut sat)=(0,0,0);
    for k in [4,8,12,24,36] {
        for mode in 0..4 {
            for seed in 0..2 {
                let (cs,specs)=fixture(k,mode,seed);
                let n=3*k;
                let mut fs:Vec<Factor>=cs.iter().map(|c|{
                    let m=mask(c);
                    let t=(0..1usize<<m.count_ones()).map(|a|{
                        let x=unpack(a,m);
                        u128::from(c.iter().any(|&v|(x&(1<<(v.unsigned_abs()-1))!=0)==(v>0)))
                    }).collect();
                    (m,t)
                }).collect();
                let selected=discover(n,&cs);
                assert_eq!(selected.len(),k);
                let mut free=(1u128<<n)-1;
                let mut trace=Vec::new();
                for m in selected {
                    assert_eq!(free&m,m);
                    trace.push(reduce(&mut fs,m));
                    free^=m;
                    groups+=1;
                }
                let names:Vec<_>=bits(free).collect();
                let scopes:Vec<_>=fs.iter().map(|(m,_)|vec![(pack(*m,free) as u128,false)]).collect();
                let (order,width)=plan(names.len(),&scopes).expect("bounded residual");
                assert!(width<=3);
                for i in order {trace.push(reduce(&mut fs,1<<names[i]));}
                let count=fs.iter().map(|(m,t)|{assert_eq!(*m,0);t[0]}).product::<u128>();
                assert_eq!(count,oracle(&specs));
                if count>0 {
                    let mut x=0;
                    for (inside,b,picks) in trace.into_iter().rev() {
                        x|=unpack(picks[pack(x,b)].expect("positive witness"),inside);
                    }
                    assert!(original(&cs,x));
                    sat+=1;
                }
                cases+=1;
            }
        }
    }
    assert_eq!((cases,groups,sat),(40,672,30));
}
#[test]
fn joint_weighted_native_shadow_control() {
    let a=[2,1,1,2];
    let b=[1,2,2,1];
    let mut rooms=Vec::new();
    for t in [a,b,a] {
        let mut cells=[0i64;CELLS];
        for (i,&v) in t.iter().enumerate() {
            cells[i]=1;
            cells[i+9]=v-1;
        }
        rooms.push(cells);
    }
    let frozen=FrozenMatrix::freeze(&Matrix::from_rooms(rooms).expect("matrix"));
    assert!(frozen.verify());
    for (r,t) in [a,b,a].iter().enumerate() {
        for (i,&v) in t.iter().enumerate() {
            assert_eq!(frozen.reconstruct_triple(r,i),Some([1,v-1,0]));
            assert_eq!(frozen.projection(r).expect("projection")[i],i128::from(v));
        }
    }
    for t in [a,b] {
        assert_eq!([t[0]+t[1],t[2]+t[3],t[0]+t[2],t[1]+t[3]],[3;4]);
    }
    assert_eq!(a.iter().map(|v|v*v).sum::<i64>(),10);
    assert_eq!(a.iter().zip(b).map(|(v,w)|v*w).sum::<i64>(),8);
}
