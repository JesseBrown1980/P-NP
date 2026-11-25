//! Bounded blind DIMACS control: exact XOR recognition, affine elimination, support closure.
use std::collections::{BTreeMap,BTreeSet};

type Cnf=Vec<Vec<i32>>;
type Form=(u32,bool);
type Equation=(u32,bool);
type Bucket=BTreeMap<Vec<u32>,BTreeSet<u32>>;
fn bit(v:i32)->u32 {1 << (v.unsigned_abs()-1)}
fn parity(v:u32)->bool {v.count_ones()%2!=0}
fn parse(s:&str)->(usize,Cnf) {
    let mut lines=s.lines();
    let h:Vec<_>=lines.next().expect("header").split_whitespace().collect();
    assert_eq!(&h[..2],&["p","cnf"]);
    let n:usize=h[2].parse().expect("variables");
    let m:usize=h[3].parse().expect("clauses");
    assert!((1..=24).contains(&n));
    let cs:Cnf=lines.map(|line|{
        let mut c:Vec<i32>=line.split_whitespace().map(|s|s.parse().expect("literal")).collect();
        assert_eq!(c.pop(),Some(0));
        let vars:BTreeSet<_>=c.iter().map(|v|v.unsigned_abs()).collect();
        assert_eq!(vars.len(),c.len());
        assert!(vars.iter().all(|&v|v>0 && v<=n as u32));
        c
    }).collect();
    assert_eq!(cs.len(),m);(n,cs)
}
fn dimacs(n:usize,cs:&Cnf)->String {
    let mut out=format!("p cnf {n} {}\n",cs.len());
    for c in cs {for v in c {out.push_str(&format!("{v} "));}out.push_str("0\n");}
    out
}
fn recognize(cs:&Cnf)->(Vec<Equation>,Cnf) {
    let mut groups=Bucket::new();
    for c in cs {
        let mut key:Vec<_>=c.iter().map(|v|v.unsigned_abs()).collect();
        key.sort_unstable();
        let p=key.iter().enumerate().fold(0,|a,(j,&v)|a|(u32::from(c.contains(&-(v as i32)))<<j));
        groups.entry(key).or_default().insert(p);
    }
    let mut eq=Vec::new();let mut rest=Vec::new();
    for (key,mut pats) in groups {
        if (2..=3).contains(&key.len()) {
            for rhs in [false,true] {
                let wrong:Vec<_>=(0..1<<key.len()).filter(|&p|parity(p)!=rhs).collect();
                if wrong.iter().all(|p|pats.contains(p)) {
                    eq.push((key.iter().fold(0,|a,&v|a|(1<<(v-1))),rhs));
                    for p in wrong {pats.remove(&p);}
                }
            }
        }
        for p in pats {
            rest.push(key.iter().enumerate().map(|(j,&v)|if p&(1<<j)!=0 {-(v as i32)} else {v as i32}).collect());
        }
    }
    (eq,rest)
}
fn union(groups:&mut Vec<u32>,m:u32) {
    if m==0 {return;}
    let mut u=0;
    groups.retain(|g|if g&m!=0 {u|=*g;false}else{true});
    assert_eq!(u&m,m);groups.push(u);
}
#[derive(Debug)]
struct ResultRow {count:u64,work:u64,rank:usize,width:u32}
fn solve(s:&str)->ResultRow {
    let (n,cs)=parse(s);let(mut eq,rest)=recognize(&cs);
    let active=rest.iter().flatten().fold(0,|a,&v|a|bit(v));
    let mut order:Vec<_>=(0..n).collect();
    order.sort_by_key(|&i|(active&(1<<i)!=0,i));
    let mut piv=Vec::new();
    for i in order {
        let r=piv.len();
        let Some(j)=(r..eq.len()).find(|&j|eq[j].0&(1<<i)!=0) else {continue;};
        eq.swap(r,j);let p=eq[r];
        for (j,e) in eq.iter_mut().enumerate() {
            if j!=r && e.0&(1<<i)!=0 {e.0^=p.0;e.1^=p.1;}
        }
        piv.push(i);
    }
    let rank=piv.len();
    if eq.iter().any(|&(m,b)|m==0 && b) {return ResultRow{count:0,work:0,rank,width:0};}
    let mut forms:Vec<Form>=(0..n).map(|i|(1<<i,false)).collect();
    for (j,&i) in piv.iter().enumerate(){forms[i]=(eq[j].0^(1<<i),eq[j].1);}
    let mut groups:Vec<_>=(0..n).filter(|i|!piv.contains(i)).map(|i|1<<i).collect();
    let mut tagged=Vec::new();
    for c in rest {
        let local:Vec<Form>=c.iter().map(|&v|{
            let (m,b)=forms[v.unsigned_abs() as usize-1];(m,b^(v<0))
        }).collect();
        let m=local.iter().fold(0,|a,&(m,_)|a|m);
        if m==0 {
            if !local.iter().any(|&(_,b)|b) {return ResultRow{count:0,work:0,rank,width:0};}
            continue;
        }
        union(&mut groups,m);tagged.push((m,local));
    }
    let mut count=1;let mut work=0;let mut width=0;
    for g in groups {
        width=width.max(g.count_ones());
        let local:Vec<_>=tagged.iter().filter(|(m,_)|m&g!=0).collect();
        let mut subtotal=0;let mut a=g;
        loop {
            work+=1;
            subtotal+=u64::from(local.iter().all(|(_,c)|c.iter().any(|&(m,b)|parity(m&a)^b)));
            if a==0 {break;}a=(a-1)&g;
        }
        count*=subtotal;
    }
    ResultRow{count,work,rank,width}
}
// Independent DPLL reference. Remaining variables are counted even when absent.
fn restrict(cs:&Cnf,u:i32)->Cnf {
    cs.iter().filter(|c|!c.contains(&u)).map(|c|c.iter().copied().filter(|&v|v!= -u).collect()).collect()
}
fn dpll(mut cs:Cnf,mut remaining:u32,nodes:&mut u64)->u64 {
    *nodes+=1;
    loop {
        if cs.iter().any(Vec::is_empty){return 0;}
        if cs.is_empty(){return 1u64<<remaining.count_ones();}
        if let Some(u)=cs.iter().find(|c|c.len()==1).map(|c|c[0]){
            remaining&=!bit(u);cs=restrict(&cs,u);
        }else{break;}
    }
    let u=cs.iter().min_by_key(|c|c.len()).expect("clause")[0];
    dpll(restrict(&cs,u),remaining&!bit(u),nodes)+dpll(restrict(&cs,-u),remaining&!bit(u),nodes)
}
fn shuffle<T>(xs:&mut[T],state:&mut u64) {
    for i in (1..xs.len()).rev() {
        *state^=*state<<13;*state^=*state>>7;*state^=*state<<17;
        xs.swap(i,(*state as usize)%(i+1));
    }
}
fn encoding(v:&[i32],rhs:bool)->Cnf {
    (0..1<<v.len()).filter(|&p|parity(p)!=rhs).map(|p|
        v.iter().enumerate().map(|(j,&v)|if p&(1<<j)!=0 {-v}else{v}).collect()).collect()
}
fn fixture(n:usize,family:usize,seed:usize,mode:usize)->(String,Vec<i32>,Vec<i32>) {
    let mut state=(17001+n*100+family*10+seed) as u64;
    let mut names:Vec<i32>=(1..=2*n as i32).collect();shuffle(&mut names,&mut state);
    let (x,y)=names.split_at(n);
    let mut cs=encoding(&[x[0],y[0]],false);
    for i in 1..n {
        cs.extend(encoding(&[x[i],x[if family==0 {i-1}else{0}],y[i]],false));
    }
    if mode==3 {cs.remove(0);}
    for group in y.chunks_exact(3) {
        for p in [seed,seed^7] {
            cs.push(group.iter().enumerate().map(|(j,&v)|if p&(1<<j)!=0 {-v}else{v}).collect());
        }
    }
    if mode==1 {cs.push(vec![-y[2],-y[3]]);}
    if mode==2 {for i in (3..n).step_by(3){cs.push(vec![-y[i-1],-y[i]]);}}
    shuffle(&mut cs,&mut state);for c in &mut cs {shuffle(c,&mut state);}
    (dimacs(2*n,&cs),x.to_vec(),y.to_vec())
}
fn projected_reference(cs:&Cnf,x:&[i32],y:&[i32],family:usize)->u64 {
    let mut count=0;
    for p in 0..1u32<<x.len() {
        let mut a=0;
        for i in 0..x.len() {
            let v=p>>i&1;let w=if i==0 {0}else{p>>(if family==0 {i-1}else{0})&1};
            if v!=0{a|=bit(x[i]);}if v^w!=0{a|=bit(y[i]);}
        }
        count+=u64::from(cs.iter().all(|c|c.iter().any(|&v|(a&bit(v)!=0)==(v>0))));
    }
    count
}
#[test]
fn blind_cnf_recovers_only_complete_parity_classes() {
    let mut cases=0;let mut work=[0;4];let mut max_width=[0;4];let mut projected=0;
    for n in [6,9,12] {for family in 0..2 {for seed in 0..2 {for mode in 0..4 {
        let (s,x,y)=fixture(n,family,seed,mode);let (nv,cs)=parse(&s);
        let got=solve(&s);let mut nodes=0;
        assert_eq!(got.count,dpll(cs.clone(),(1<<nv)-1,&mut nodes));
        assert_eq!(got.rank,n-usize::from(mode==3));
        if mode!=3 {assert_eq!(got.count,projected_reference(&cs,&x,&y,family));projected+=1;}
        work[mode]+=got.work;max_width[mode]=max_width[mode].max(got.width);cases+=1;
        println!("BLIND_CNF|n={n}|family={family}|seed={seed}|mode={mode}|rank={}|width={}|count={}|local_assignments={}|dpll_nodes={nodes}",got.rank,got.width,got.count,got.work);
    }}}}
    assert_eq!((cases,projected),(48,36));
    assert_eq!(max_width,[3,6,12,4]);
    assert_eq!(work,[288,864,18688,384]);
    for seed in 0..12 {
        let mut state=9127+seed;let mut pool=Vec::new();
        for a in 1..=9 {for b in a+1..=9 {for c in b+1..=9 {pool.push(vec![a,b,c]);}}}
        shuffle(&mut pool,&mut state);pool.truncate(18);
        for c in &mut pool {for v in c {state^=state<<13;state^=state>>7;state^=state<<17;if state&1!=0 {*v= -*v;}}}
        let g=solve(&dimacs(9,&pool));assert_eq!(g.rank,0);
        assert_eq!(g.count,dpll(pool,511,&mut 0));
    }
    let mut bad=encoding(&[1,2,3],false);bad.extend(encoding(&[1,2,3],true));
    assert_eq!(solve(&dimacs(3,&bad)).count,0);assert_eq!(dpll(bad,7,&mut 0),0);
    println!("BLIND_CNF_TOTAL|structured=48|projected=36|null=12|contradiction=1|local_assignments={}",work.iter().sum::<u64>());
}
