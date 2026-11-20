//! Independent *nontrivial* Boolean SAT/UNSAT oracle for the frozen
//! exact three-shadow DPLL frontier. Each n-variable problem is generated
//! as an all-assignment-blocking CNF: zero surviving assignments (UNSAT)
//! or exactly one surviving assignment (SAT).
//!
//! All clauses are length n>=3: no initial empty clause or trivial unit
//! contradiction. This forces genuine branching, propagation, backtracking,
//! and preservation of minus-branch frames after a freeze/restart.
#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use crate::continuous_frontier::{Frontier, FrontierMode, FrontierState};
    use crate::guidance::{Guidance, Strategy};
    use crate::sha256::hash_bytes;
    use crate::ternary::{Cnf, Trit};

    fn formula(variables: usize, allowed: Option<u64>, rotation: usize) -> String {
        let total = 1_u64 << variables;
        let mut excluded = (0..total)
            .filter(|&assignment| Some(assignment) != allowed)
            .collect::<Vec<_>>();
        let count = excluded.len();
        excluded.rotate_left(rotation % count);
        let mut source = format!("p cnf {variables} {count}\n");
        for mask in excluded {
            for ix in 0..variables {
                let variable = ((ix + rotation) % variables) + 1;
                let lit = if mask & (1_u64 << (variable - 1)) != 0 {
                    -(variable as i32)
                } else {
                    variable as i32
                };
                source.push_str(&format!("{lit} "));
            }
            source.push_str("0\n");
        }
        source
    }

    /// This oracle only evaluates complete truth assignments and is
    /// algorithmically separate from DPLL, unit propagation, or shadows.
    fn truth_models(cnf: &Cnf) -> Vec<u64> {
        (0..(1_u64 << cnf.variables)).filter(|&model| {
            cnf.clauses.iter().all(|clause| clause.iter().any(|&literal| {
                let value = (model >> (literal.unsigned_abs() as usize - 1)) & 1;
                (literal > 0 && value == 1) || (literal < 0 && value == 0)
            }))
        }).collect()
    }

    fn as_mask(model: &[Trit]) -> u64 {
        model.iter().enumerate().fold(0_u64, |result,(ix,trit)| {
            if *trit == Trit::Plus {
                result | (1_u64 << ix)
            } else {
                assert_eq!(*trit,Trit::Minus);
                result
            }
        })
    }

    #[test]
    fn nontrivial_unique_sat_and_total_unsat_all_three_shadow_modes() {
        let mut cases = 0_usize;
        for variables in 3_usize..=6 {
            let limit = 1_u64 << variables;
            for rotation in 0..6 {
                let target = ((rotation * 7 + 3) as u64) % limit;
                for allowed in [Some(target), None] {
                    let text = formula(variables, allowed, rotation);
                    let cnf = Cnf::parse(&text).expect("complete nontrivial DIMACS");
                    assert!(cnf.clauses.iter().all(|c| c.len()==variables),
                        "oracle fixture must not include initial unit/empty clauses");
                    let truth = truth_models(&cnf);
                    assert_eq!(truth,allowed.into_iter().collect::<Vec<_>>(),
                        "independent complete truth-table oracle");
                    let route = Guidance::for_source(&cnf,Strategy::Fischer)
                        .expect("Fischer route for nontrivial CNF");
                    let fingerprint = hash_bytes(text.as_bytes());
                    let seed = vec![Trit::Zero; variables];
                    for mode in [FrontierMode::Direct,FrontierMode::One,FrontierMode::Three] {
                        let mut uninterrupted = Frontier::new(
                            &cnf,&route,&seed,&fingerprint,mode
                        ).expect("original source bound to exact 3D frontier");
                        uninterrupted.advance(&cnf,&route,1000)
                            .expect("uninterrupted complete SAT/UNSAT search");
                        let expected_status =
                            if allowed.is_some() { FrontierState::Sat } else { FrontierState::Unsat };
                        assert_eq!(uninterrupted.state,expected_status,
                            "independent exhaustive truth-table status");
                        assert!(uninterrupted.nodes > 1,
                            "nontrivial adversarial oracle must actually branch");
                        if let Some(mask) = allowed {
                            let model = uninterrupted.model().expect("completed SAT model");
                            assert!(cnf.valid_complete_model(model));
                            assert_eq!(as_mask(model),mask,
                                "exact unique satisfying target differs");
                        } else {
                            assert!(uninterrupted.model().is_none());
                        }

                        for schedule in [
                            &[1_u64][..], &[2_u64,3,1,5][..], &[7_u64,1,2,4,3][..]
                        ] {
                            let mut resumed = Frontier::new(
                                &cnf,&route,&seed,&fingerprint,mode
                            ).expect("new source-verified SAT frontier");
                            let mut rounds = 0;
                            loop {
                                let step = schedule[rounds % schedule.len()];
                                resumed.advance(&cnf,&route,step)
                                    .expect("advance unvisited source SAT nodes");
                                let checkpoint = resumed.freeze().expect("freeze full stack");
                                resumed = Frontier::thaw(
                                    &checkpoint,&cnf,&route,&seed,&fingerprint,mode
                                ).expect("source-bound complete frontier replay");
                                assert_eq!(checkpoint,resumed.freeze().expect("canonical replay"),
                                    "DPLL snapshot bytes not identical");
                                if resumed.state != FrontierState::Running {break;}
                                rounds += 1;
                                assert!(rounds < 1000,"nontrivial CNF did not terminate");
                            }
                            assert_eq!(resumed.state,uninterrupted.state);
                            assert_eq!(resumed.nodes,uninterrupted.nodes);
                            assert_eq!(resumed.propagations,uninterrupted.propagations);
                            assert_eq!(resumed.literal_checks,uninterrupted.literal_checks);
                            assert_eq!(resumed.decisions,uninterrupted.decisions);
                            assert_eq!(resumed.trace_sha256,uninterrupted.trace_sha256);
                            assert_eq!(resumed.model(),uninterrupted.model());
                            assert_eq!(resumed.first_order,uninterrupted.first_order);
                            assert_eq!(resumed.first_selected,uninterrupted.first_selected);
                            assert_eq!(resumed.projected_rooms,uninterrupted.projected_rooms);
                            assert_eq!(resumed.projected_bytes,uninterrupted.projected_bytes);
                            assert_eq!(resumed.pixel_reads,uninterrupted.pixel_reads);
                            assert_eq!(resumed.reconstructed_rooms,uninterrupted.reconstructed_rooms);
                        }
                        cases += 1;
                    }
                }
            }
        }
        assert_eq!(cases,144,"each of 48 hard CNFs checked in three modes");
    }
}
