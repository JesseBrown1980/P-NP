//! Exact normalized-row quotient for finite min-plus SAT boundary kernels.
//! Research scope: local three-literal CNF windows, not arbitrary CNF.
use std::collections::BTreeMap;

pub type Matrix = Vec<Vec<Option<u64>>>;

#[derive(Debug)]
pub struct Quotient {
    rows: Matrix,
    ids: Vec<usize>,
    offsets: Vec<u64>,
}

fn square(a: &Matrix) -> bool {
    !a.is_empty() && a.iter().all(|r| r.len() == a.len())
}

pub fn quotient(a: &Matrix) -> Result<Quotient, &'static str> {
    if !square(a) {
        return Err("invalid boundary shape");
    }
    let mut index = BTreeMap::new();
    let mut rows = Vec::new();
    let mut ids = Vec::new();
    let mut offsets = Vec::new();
    for row in a {
        let base = row.iter().flatten().copied().min().unwrap_or(0);
        let key: Vec<_> = row.iter().map(|x| x.map(|v| v-base)).collect();
        let id = *index.entry(key.clone()).or_insert_with(|| {
            let id = rows.len();
            rows.push(key);
            id
        });
        ids.push(id);
        offsets.push(base);
    }
    Ok(Quotient { rows, ids, offsets })
}

impl Quotient {
    pub fn classes(&self) -> usize {
        self.rows.len()
    }

    pub fn compose(&self, b: &Matrix) -> Result<Matrix, &'static str> {
        if !square(b) || self.ids.len() != b.len() {
            return Err("invalid boundary shape");
        }
        let compact = product_rows(&self.rows, b)?;
        self.ids.iter().zip(&self.offsets).map(|(&id, &offset)| {
            compact[id].iter().map(|&v| {
                v.map(|x| x.checked_add(offset).ok_or("action overflow")).transpose()
            }).collect()
        }).collect()
    }
}

fn product_rows(a: &Matrix, b: &Matrix) -> Result<Matrix, &'static str> {
    let mut out = vec![vec![None; b.len()]; a.len()];
    for (row, dst) in a.iter().zip(&mut out) {
        for (left, right) in row.iter().zip(b) {
            if let Some(x) = left {
                for (y, cell) in right.iter().zip(dst.iter_mut()) {
                    if let Some(y) = y {
                        let value = x.checked_add(*y).ok_or("action overflow")?;
                        *cell = Some(cell.map_or(value, |old: u64| old.min(value)));
                    }
                }
            }
        }
    }
    Ok(out)
}

pub fn dense(a: &Matrix, b: &Matrix) -> Result<Matrix, &'static str> {
    if !square(a) || !square(b) || a.len() != b.len() {
        return Err("invalid boundary shape");
    }
    product_rows(a,b)
}

#[cfg(test)]
mod tests {
    use super::*;
    type Clause = [(usize, usize); 3];

    fn next(s: &mut u64) -> u64 {
        *s ^= *s << 13;
        *s ^= *s >> 7;
        *s ^= *s << 17;
        *s
    }

    fn source(cell: usize, case: usize, w: usize) -> Vec<Vec<Clause>> {
        let mut s = 20261128 + 100003 * cell as u64 + 8191 * case as u64;
        let mut triples = Vec::new();
        for a in 0..=w {
            for b in a+1..=w {
                for c in b+1..=w {
                    triples.push([a,b,c]);
                }
            }
        }
        (0..2*w).map(|_| {
            (0..[1,3,6,12][case%4]).map(|_| {
                let p = triples[(next(&mut s) % triples.len() as u64) as usize];
                let sign = next(&mut s) as usize % 8;
                [(p[0], sign&1), (p[1], (sign>>1)&1), (p[2], (sign>>2)&1)]
            }).collect()
        }).collect()
    }

    fn one(w: usize, clauses: &[Clause]) -> Matrix {
        let d = 1 << w;
        let mut a = vec![vec![None;d];d];
        for (i, row) in a.iter_mut().enumerate() {
            for bit in 0..2 {
                let window = 2*i+bit;
                let cost = clauses.iter().filter(|clause| {
                    clause.iter().all(|&(p,b)| ((window>>(w-p))&1)!=b)
                }).count() as u64;
                row[window & (d-1)] = Some(cost);
            }
        }
        a
    }

    fn block(w: usize, source: &[Vec<Clause>]) -> Matrix {
        let mut iter = source.iter().map(|c| one(w,c));
        let first = iter.next().expect("nonempty block");
        iter.try_fold(first, |a,b| dense(&a,&b)).expect("bounded costs")
    }

    // Independent enumeration of full assignments, evaluating each original literal.
    fn exhaustive(w: usize, source: &[Vec<Clause>]) -> Matrix {
        let d = 1 << w;
        let length = source.len();
        let mut out = vec![vec![None;d];d];
        for bits in 0..1_usize<<(w+length) {
            let mut cost = 0_u64;
            for (t, clauses) in source.iter().enumerate() {
                for clause in clauses {
                    let satisfied = clause.iter().any(|&(p,b)| {
                        (bits >> (w+length-1-t-p)) & 1 == b
                    });
                    if !satisfied {
                        cost = cost.checked_add(1).expect("bounded oracle");
                    }
                }
            }
            let cell = &mut out[bits>>length][bits&(d-1)];
            *cell = Some(cell.map_or(cost, |old: u64| old.min(cost)));
        }
        out
    }

    fn emit(case: usize, label: &str, a: &Matrix) {
        let values: Vec<_> = a.iter().flatten().map(|v| {
            v.map_or_else(|| "X".to_owned(), |x| x.to_string())
        }).collect();
        println!("MATRIX|case={case}|block={label}|{}", values.join(","));
    }

    #[test]
    fn quotient_negative_controls() {
        let a = vec![vec![Some(0),Some(1)],vec![Some(1),Some(0)]];
        assert_eq!(quotient(&a).unwrap().classes(),2); // same scalar min, different futures
        let b = vec![vec![Some(0),None],vec![None,Some(0)]];
        assert_eq!(quotient(&b).unwrap().classes(),2); // different supports
        let c = vec![vec![Some(0),Some(1)],vec![Some(2),Some(3)]];
        let q=quotient(&c).unwrap();
        assert_eq!(q.classes(),1);
        assert_eq!(q.compose(&b).unwrap(),c);
        let absent=vec![vec![None;2];2];
        assert_eq!(quotient(&absent).unwrap().compose(&b).unwrap(),absent);
        assert!(quotient(&vec![vec![Some(0),Some(1)]]).is_err());
        let huge=vec![vec![Some(u64::MAX);2];2];
        assert!(quotient(&huge).unwrap().compose(&c).is_err());
    }

    #[test]
    #[ignore = "GitRAM disjoint cell"]
    fn gitram_quotient() {
        let cell: usize = std::env::var("GITRAM_CELL").expect("cell id").parse().expect("numeric id");
        assert!(cell<20);
        let w=2+cell%4;
        let d=1_usize<<w;
        let mut ranks=Vec::new();
        let mut minima=Vec::new();
        for case in 0..12 {
            let input=source(cell,case,w);
            let a=block(w,&input[..w]);
            let b=block(w,&input[w..]);
            assert_eq!(a,exhaustive(w,&input[..w]));
            assert_eq!(b,exhaustive(w,&input[w..]));
            let q=quotient(&a).unwrap();
            let result=q.compose(&b).unwrap();
            assert_eq!(result,dense(&a,&b).unwrap());
            assert_eq!(result,exhaustive(w,&input));
            ranks.push(q.classes().to_string());
            minima.push(result.iter().flatten().flatten().min().expect("finite").to_string());
            emit(case,"A",&a); emit(case,"B",&b); emit(case,"C",&result);
        }
        let sat=minima.iter().filter(|v| *v=="0").count();
        println!("QUOTIENT_SHARD_PASS|cell={cell:02}|width={w}|cases=12|entries={}|sat={sat}|unsat={}|classes={}|minima={}",36*d*d,12-sat,ranks.join(","),minima.join(","));
    }

    fn scale_source(cell: usize, w: usize) -> Vec<Vec<Clause>> {
        let mut state = 2026112801 + 100003 * cell as u64;
        let mut triples = Vec::new();
        for a in 0..=w {
            for b in a+1..=w {
                for c in b+1..=w {
                    triples.push([a,b,c]);
                }
            }
        }
        (0..2*w).map(|_| {
            (0..[1,3,6,12][cell%4]).map(|_| {
                let p=triples[(next(&mut state)%triples.len() as u64) as usize];
                let s=next(&mut state) as usize % 8;
                [(p[0],s&1),(p[1],(s>>1)&1),(p[2],(s>>2)&1)]
            }).collect()
        }).collect()
    }

    // At exactly w transitions the initial and final bit strings uniquely
    // determine every intermediate window. Evaluate original clause literals.
    fn direct_block(w: usize, input: &[Vec<Clause>]) -> Matrix {
        assert_eq!(input.len(),w);
        let d=1_usize<<w;
        let weights: Vec<Vec<u64>>=input.iter().map(|clauses| {
            (0..2*d).map(|v| {
                clauses.iter().filter(|c| {
                    c.iter().all(|&(p,b)| ((v>>(w-p))&1)!=b)
                }).count() as u64
            }).collect()
        }).collect();
        (0..d).map(|i| {
            (0..d).map(|j| {
                let bits=(i<<w)|j;
                Some(weights.iter().enumerate().map(|(t,c)| {
                    c[(bits>>(w-1-t)) & (2*d-1)]
                }).sum())
            }).collect()
        }).collect()
    }

    // Independent incoming-edge recurrence, retaining all endpoint states.
    fn incoming(w: usize, input: &[Vec<Clause>]) -> Matrix {
        let d=1_usize<<w;
        let mut a=vec![vec![None;d];d];
        for (i,row) in a.iter_mut().enumerate() {
            row[i]=Some(0_u64);
        }
        for clauses in input {
            let costs: Vec<u64>=(0..2*d).map(|v| {
                let mut cost=0;
                for clause in clauses {
                    let mut satisfied=false;
                    for &(p,b) in clause {
                        satisfied |= ((v>>(w-p))&1)==b;
                    }
                    if !satisfied {cost+=1;}
                }
                cost
            }).collect();
            for row in &mut a {
                let mut dst=Vec::with_capacity(d);
                for j in 0..d {
                    let first=row[j/2].map(|x| x.checked_add(costs[j]).expect("bounded"));
                    let second=row[j/2+d/2].map(|x| x.checked_add(costs[j+d]).expect("bounded"));
                    dst.push(match (first,second) {
                        (Some(x),Some(y))=>Some(x.min(y)),
                        (Some(x),None)|(None,Some(x))=>Some(x),
                        (None,None)=>None,
                    });
                }
                *row=dst;
            }
        }
        a
    }

    // CNF equality pair (!x OR y) AND (x OR !y) for each crossing bit.
    // Distinct rows have distinct unique zero locations, so no row merges.
    fn equality_adversary(w: usize) -> Matrix {
        let d=1_usize<<w;
        (0..d).map(|i| (0..d).map(|j| {
            let mut cost=0_u64;
            for p in 0..w {
                let x=(i>>p)&1!=0;
                let y=(j>>p)&1!=0;
                cost+=u64::from(x&&!y)+u64::from(!x&&y);
            }
            assert_eq!(cost,u64::from((i^j).count_ones()));
            Some(cost)
        }).collect()).collect()
    }

    #[test]
    #[ignore = "GitRAM wide boundary scaling"]
    fn gitram_scale() {
        use std::io::{BufWriter,Write};
        let cell: usize=std::env::var("GITRAM_CELL").expect("cell id").parse().expect("numeric id");
        assert!(cell<20);
        let w=6+cell/4;
        let d=1_usize<<w;
        let density=[1,3,6,12][cell%4];
        let input=scale_source(cell,w);
        let a=direct_block(w,&input[..w]);
        let b=direct_block(w,&input[w..]);
        assert_eq!(a,incoming(w,&input[..w]));
        assert_eq!(b,incoming(w,&input[w..]));
        let q=quotient(&a).unwrap();
        let c=q.compose(&b).unwrap();
        assert_eq!(c,incoming(w,&input));
        let h=equality_adversary(w);
        let hq=quotient(&h).unwrap();
        assert_eq!(hq.classes(),d);
        let minimum=c.iter().flatten().flatten().min().expect("finite");
        let path=std::env::var("GITRAM_SCALE_OUTPUT").expect("output file");
        let mut writer=BufWriter::new(std::fs::File::create(path).expect("output"));
        for matrix in [&a,&b,&c,&h] {
            for v in matrix.iter().flatten() {
                let value=u16::try_from(v.expect("finite")).expect("bounded cost");
                writer.write_all(&value.to_le_bytes()).expect("write");
            }
        }
        writer.flush().expect("flush");
        println!("SCALE|cell={cell:02}|width={w}|density={density}|states={d}|classes={}|minimum={minimum}|adversarial_classes={}",q.classes(),hq.classes());
    }
}
