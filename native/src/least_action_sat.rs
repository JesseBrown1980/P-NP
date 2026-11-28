//! Research-only min-plus kernel for consecutive three-variable CNF windows.
//! Not a general SAT solver; each window contains only three consecutive bits.
//! None is unreachable, never a finite sentinel.
pub type Kernel = [[Option<u64>; 4]; 4];

/// Multiplicity of each of the eight literal-polarity triples (positive = 1).
pub type Window = [u64; 8];

pub fn window_kernel(counts: &Window) -> Kernel {
    let mut out = [[None; 4]; 4];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            if i % 2 == j / 2 {
                let assignment = 2 * i + j % 2;
                *cell = Some(counts[7 - assignment]);
            }
        }
    }
    out
}

pub fn compose(a: &Kernel, b: &Kernel) -> Result<Kernel, &'static str> {
    let mut out = [[None; 4]; 4];
    for (i, row) in out.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            for (k, b_row) in b.iter().enumerate() {
                if let (Some(x), Some(y)) = (a[i][k], b_row[j]) {
                    let cost = x.checked_add(y).ok_or("action overflow")?;
                    *cell = Some(cell.map_or(cost, |old: u64| old.min(cost)));
                }
            }
        }
    }
    Ok(out)
}

pub fn cube(a: &Kernel) -> Result<Kernel, &'static str> {
    compose(&compose(a, a)?, a)
}

pub fn minimum(a: &Kernel) -> Option<u64> {
    a.iter().flatten().filter_map(|&x| x).min()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent exhaustive Boolean evaluation of each original literal.
    fn exhaustive(windows: &[Window]) -> u64 {
        let mut best = u64::MAX;
        for bits in 0..(1_usize << (windows.len() + 2)) {
            let mut cost = 0_u64;
            for (t, counts) in windows.iter().enumerate() {
                for (sign, &count) in counts.iter().enumerate() {
                    let satisfied = (0..3).any(|j| {
                        ((bits >> (t + j)) & 1) == ((sign >> (2 - j)) & 1)
                    });
                    if !satisfied {
                        cost = cost.checked_add(count).expect("oracle overflow");
                    }
                }
            }
            best = best.min(cost);
        }
        best
    }

    #[test]
    fn exhaustive_small_windows() {
        let mut state = 27181_u64;
        for n in 1..=10 {
            for _ in 0..24 {
                let mut windows = Vec::new();
                for _ in 0..n {
                    let mut w = [0; 8];
                    for count in &mut w {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        *count = state % 3;
                    }
                    windows.push(w);
                }
                let mut kernels = windows.iter().map(window_kernel);
                let first = kernels.next().expect("nonempty windows");
                let result = kernels.try_fold(first, |a, b| compose(&a, &b))
                    .expect("bounded test costs");
                assert_eq!(minimum(&result), Some(exhaustive(&windows)));
            }
        }
    }

    #[test]
    fn repeated_contradiction_and_overflow() {
        let mut k = window_kernel(&[1; 8]);
        for _ in 0..8 {
            k = cube(&k).expect("bounded jump");
        }
        assert_eq!(minimum(&k), Some(6561));
        assert_eq!(minimum(&window_kernel(&[0; 8])), Some(0));
        assert!(compose(&[[Some(u64::MAX); 4]; 4], &[[Some(1); 4]; 4]).is_err());
        assert_eq!(compose(&[[None; 4]; 4], &k).unwrap(), [[None; 4]; 4]);
    }

    /// GitRAM cells use disjoint deterministic inputs, not identical repeats.
    #[test]
    #[ignore = "run by the GitRAM cell workflow"]
    fn gitram_cell() {
        let cell: usize = std::env::var("GITRAM_CELL").expect("cell id")
            .parse().expect("numeric cell id");
        assert!(cell < 20);
        let n = cell / 2 + 1;
        let mut state = 27181_u64 + (cell as u64) * 65537;
        let mut minima = Vec::new();
        for case in 0..12 {
            let planted = (cell * 37 + case * 13) % (1 << (n + 2));
            let mut windows = Vec::new();
            for t in 0..n {
                let mut w = [0; 8];
                for (sign, count) in w.iter_mut().enumerate() {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    *count = if case % 3 == 2 { state % 3 } else { u64::from(state % 5 == 0) };
                    if case % 3 == 0 && (0..3).all(|j| {
                        ((planted >> (t + j)) & 1) != ((sign >> (2 - j)) & 1)
                    }) {
                        *count = 0;
                    }
                    if case % 3 == 1 && t == case % n {
                        *count = 1;
                    }
                }
                windows.push(w);
            }
            let kernels: Vec<_> = windows.iter().map(window_kernel).collect();
            let forward = kernels[1..].iter().try_fold(kernels[0], |a, b| compose(&a, b))
                .expect("bounded action");
            let backward = kernels[..n-1].iter().rev().try_fold(kernels[n-1], |b, a| compose(a, &b))
                .expect("bounded action");
            assert_eq!(forward, backward);
            let oracle = exhaustive(&windows);
            assert_eq!(minimum(&forward), Some(oracle));
            minima.push(oracle);
        }
        let sat = minima.iter().filter(|&&v| v == 0).count();
        let values: Vec<_> = minima.iter().map(u64::to_string).collect();
        println!("GITRAM_SHARD_PASS|cell={cell:02}|cases=12|sat={sat}|unsat={}|minima={}", 12-sat, values.join(","));
    }
}
