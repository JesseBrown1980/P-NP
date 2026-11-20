//! Independent randomized exact-SAT oracle and restart schedule audit.
//! This test module deliberately does not use the frontier engine to decide
//! satisfiability: it enumerates every complete Boolean assignment.
//! It also compares uninterrupted versus frozen/replayed frontier traces.
#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use crate::continuous_frontier::{Frontier, FrontierMode, FrontierState};
    use crate::guidance::{Guidance, Strategy};
    use crate::sha256::hash_bytes;
    use crate::ternary::{Cnf, Trit};

    fn next(random: &mut u64) -> u64 {
        *random ^= *random << 13;
        *random ^= *random >> 7;
        *random ^= *random << 17;
        *random
    }

    /// Standalone truth-table oracle; no calls to DPLL internals.
    fn exhaustive(cnf: &Cnf) -> bool {
        (0..(1_u64 << cnf.variables)).any(|mask| {
            cnf.clauses.iter().all(|clause| {
                clause.iter().any(|&literal| {
                    let bit = (mask >> (literal.unsigned_abs() as usize - 1)) & 1;
                    (literal > 0 && bit == 1) || (literal < 0 && bit == 0)
                })
            })
        })
    }

    fn random_source(random: &mut u64) -> String {
        let variables = 1 + (next(random) % 6) as usize;
        let clause_count = (next(random) % 25) as usize;
        let mut text = format!("p cnf {variables} {clause_count}\n");
        for _ in 0..clause_count {
            let len = (next(random) % 5) as usize;
            for _ in 0..len {
                let variable = 1 + (next(random) % variables as u64) as i32;
                let literal = if next(random) % 2 == 0 { variable } else { -variable };
                text.push_str(&format!("{literal} "));
            }
            text.push_str("0\n");
        }
        text
    }

    #[test]
    fn independent_truth_table_and_frozen_restart_parity_480_cases() {
        let mut random = 0x6b8b4567327b23c6_u64;
        for sample in 0..160_usize {
            let source = random_source(&mut random);
            let cnf = Cnf::parse(&source).expect("random DIMACS parser");
            let witness = exhaustive(&cnf);
            let guide = Guidance::for_source(&cnf, Strategy::Fischer)
                .expect("exact source Fischer guidance");
            let seed = vec![Trit::Zero; cnf.variables];
            let digest = hash_bytes(source.as_bytes());
            for mode in [FrontierMode::Direct, FrontierMode::One, FrontierMode::Three] {
                let mut whole = Frontier::new(&cnf,&guide,&seed,&digest,mode)
                    .expect("source-bound whole frontier");
                whole.advance(&cnf,&guide,1000).expect("whole exact computation");
                assert_ne!(whole.state, FrontierState::Running,
                    "sample {sample}: bounded truth-table-sized CNF must terminate");
                assert_eq!(whole.state == FrontierState::Sat,witness,
                    "sample {sample}: mismatch against independent brute force");
                if witness {
                    assert!(cnf.valid_complete_model(whole.model().expect("SAT model")));
                } else {
                    assert!(whole.model().is_none());
                }

                let mut resumed = Frontier::new(&cnf,&guide,&seed,&digest,mode)
                    .expect("source-bound restart frontier");
                for iteration in 0..1000_usize {
                    let quantum = [1_u64, 2, 5, 3, 7, 4, 11][iteration % 7];
                    resumed.advance(&cnf,&guide,quantum)
                        .expect("exact new node work");
                    let checkpoint = resumed.freeze().expect("full frozen frontier");
                    let restored = Frontier::thaw(
                        &checkpoint,&cnf,&guide,&seed,&digest,mode
                    ).expect("verified source-bound checkpoint");
                    assert_eq!(checkpoint,restored.freeze().expect("canonical bytes"),
                        "sample {sample}: noncanonical checkpoint");
                    resumed = restored;
                    if resumed.state != FrontierState::Running { break; }
                }
                assert_eq!(whole.state,resumed.state,"sample {sample}: status diverged");
                assert_eq!(whole.nodes,resumed.nodes,"sample {sample}: node replayed");
                assert_eq!(whole.propagations,resumed.propagations,
                    "sample {sample}: propagation diverged");
                assert_eq!(whole.literal_checks,resumed.literal_checks,
                    "sample {sample}: literal counter diverged");
                assert_eq!(whole.decisions,resumed.decisions,
                    "sample {sample}: branch counter diverged");
                assert_eq!(whole.trace_sha256,resumed.trace_sha256,
                    "sample {sample}: exact trace differs");
                assert_eq!(whole.model(),resumed.model(),
                    "sample {sample}: complete model differs");
                assert_eq!(whole.projected_rooms,resumed.projected_rooms);
                assert_eq!(whole.projected_bytes,resumed.projected_bytes);
                assert_eq!(whole.pixel_reads,resumed.pixel_reads);
                assert_eq!(whole.reconstructed_rooms,resumed.reconstructed_rooms);
                assert_eq!(whole.source_trit_reads,resumed.source_trit_reads);
            }
        }
    }

    #[test]
    fn nonzero_original_seed_survives_all_branch_backtracks() {
        let source = "p cnf 3 4\n1 2 0\n1 -2 0\n-1 2 3 0\n-1 -2 3 0\n";
        let cnf = Cnf::parse(source).expect("source");
        let guide = Guidance::for_source(&cnf, Strategy::Fischer).expect("Fischer");
        let digest = hash_bytes(source.as_bytes());
        let seed = [Trit::Zero,Trit::Zero,Trit::Plus];
        for mode in [FrontierMode::Direct,FrontierMode::One,FrontierMode::Three] {
            let mut whole = Frontier::new(&cnf,&guide,&seed,&digest,mode).expect("new");
            whole.advance(&cnf,&guide,100).expect("complete");
            assert_eq!(whole.state,FrontierState::Sat);
            assert_eq!(whole.model().expect("model")[2],Trit::Plus);
            let mut restarted = Frontier::new(&cnf,&guide,&seed,&digest,mode).expect("new");
            restarted.advance(&cnf,&guide,1).expect("branch");
            let copy = restarted.freeze().expect("freeze");
            restarted = Frontier::thaw(&copy,&cnf,&guide,&seed,&digest,mode)
                .expect("complete seed-binding thaw");
            restarted.advance(&cnf,&guide,100).expect("continue");
            assert_eq!(whole.trace_sha256,restarted.trace_sha256);
            assert_eq!(whole.nodes,restarted.nodes);
        }
    }
}
