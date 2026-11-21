//! Independent native acceptance of the experimental paid nine-cat selector.
//! No hidden-Bitcoin-model labels enter its frozen CNF-derived source index.
#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use crate::continuous_frontier::{Frontier,FrontierMode,FrontierState};
    use crate::guidance::{Guidance,Strategy};
    use crate::sha256::hash_bytes;
    use crate::shadow_selector::{ShadowMode,ShadowSelector};
    use crate::ternary::{Cnf,Trit};

    fn expected_features(signs: &[i8],center: usize) -> [i16;9] {
        let mut ret=[0_i16;9];
        let n=signs.len();
        for radius in 1..=3 {
            let mut f=0_i16;
            let mut b=0_i16;
            let mut mid=0_i16;
            for delta in 1..=radius {
                let l=i16::from(signs[(center+n-delta%n)%n]);
                let r=i16::from(signs[(center+delta)%n]);
                b+=l;f+=r;mid+=l*r;
            }
            ret[radius-1]=f;
            ret[3+radius-1]=mid;
            ret[6+radius-1]=b;
        }
        ret
    }

    #[test]
    fn independent_nine_cat_source_guidance_and_exact_paid_observer_cost() {
        let mut source="p cnf 27 9\n".to_owned();
        for literal in [1_i32,-2,3,4,-6,7,-8,-9,13] {
            source.push_str(&format!("{literal} 0\n"));
        }
        let cnf=Cnf::parse(&source).expect("source parser");
        let signs=(
            1..=27
        ).map(|variable| {
            let sum:i32=cnf.clauses.iter()
                .flat_map(|clause| clause.iter())
                .filter(|literal| literal.unsigned_abs()==variable)
                .map(|literal| literal.signum()).sum();
            sum.signum() as i8
        }).collect::<Vec<_>>();
        let candidates=(1_usize..=27).collect::<Vec<_>>();
        let expected=candidates.iter().map(|&var| {
            let view=expected_features(&signs,var-1);
            let score=view.iter().map(|value|u64::from(value.unsigned_abs())).sum::<u64>();
            (var,score)
        }).max_by(|(a,sa),(b,sb)|sa.cmp(sb).then_with(||b.cmp(a)))
          .expect("independent 27 candidate rank").0;
        let sha=hash_bytes(source.as_bytes());
        let state=vec![Trit::Zero;27];
        let mut nine=ShadowSelector::new(&cnf,&sha,ShadowMode::Nine)
            .expect("source only 9 cat selector");
        let choice=nine.choose(&cnf,&candidates,&state)
            .expect("authentic source signed cat choice");
        assert_eq!(choice,expected,"independent 9x27 source observer rank");
        let m=nine.into_metrics();
        assert_eq!(m.room_projections,27);
        assert_eq!(m.stored_projection_bytes,486);
        assert_eq!(m.pixel_reads,243);
        assert_eq!(m.source_trit_reads,162);
        assert_eq!(m.reconstructed_rooms,0);
        assert_eq!(m.source_index_bytes,27);
        assert_eq!(m.choices,1);
        assert_eq!(m.first_order_candidate,1);
        assert_eq!(m.first_shadow_candidate,choice as u64);
    }

    #[test]
    fn nine_cat_exact_sat_unsat_and_interrupted_frontier_equivalence() {
        let unsat="p cnf 3 8\n1 2 3 0\n1 2 -3 0\n1 -2 3 0\n1 -2 -3 0\n-1 2 3 0\n-1 2 -3 0\n-1 -2 3 0\n-1 -2 -3 0\n";
        let sat="p cnf 3 7\n1 2 3 0\n1 2 -3 0\n1 -2 3 0\n1 -2 -3 0\n-1 2 3 0\n-1 2 -3 0\n-1 -2 3 0\n";
        for (source,outcome) in [
            (sat,FrontierState::Sat),(unsat,FrontierState::Unsat)
        ] {
            let cnf=Cnf::parse(source).expect("exact SAT CNF");
            let route=Guidance::for_source(&cnf,Strategy::Fischer).expect("Fischer");
            let seed=vec![Trit::Zero;cnf.variables];
            let digest=hash_bytes(source.as_bytes());
            let mut whole=Frontier::new(&cnf,&route,&seed,&digest,FrontierMode::Nine)
                .expect("new");
            whole.advance(&cnf,&route,1000).expect("complete");
            assert_eq!(whole.state,outcome);
            if outcome==FrontierState::Sat {
                assert!(cnf.valid_complete_model(whole.model().expect("SAT proof")));
            } else {
                assert_eq!(whole.model(),None);
            }
            let mut split=Frontier::new(&cnf,&route,&seed,&digest,FrontierMode::Nine)
                .expect("new");
            for _ in 0..1000 {
                if split.state!=FrontierState::Running {break;}
                split.advance(&cnf,&route,1).expect("one new source node");
                let bytes=split.freeze().expect("canonical full frontier");
                split=Frontier::thaw(&bytes,&cnf,&route,&seed,&digest,FrontierMode::Nine)
                    .expect("exact independent session restart");
                assert_eq!(bytes,split.freeze().expect("exact frozen bytes"));
            }
            assert_eq!(split.state,whole.state);
            assert_eq!(split.nodes,whole.nodes);
            assert_eq!(split.propagations,whole.propagations);
            assert_eq!(split.literal_checks,whole.literal_checks);
            assert_eq!(split.decisions,whole.decisions);
            assert_eq!(split.trace_sha256,whole.trace_sha256);
            assert_eq!(split.projected_rooms,whole.projected_rooms);
            assert_eq!(split.projected_bytes,whole.projected_bytes);
            assert_eq!(split.source_trit_reads,whole.source_trit_reads);
            assert_eq!(split.source_index_bytes,
                whole.source_index_bytes*split.nodes,
                "the shared signed source index must be paid again per process");
        }
    }

    #[test]
    fn nine_cat_checkpoint_tampering_and_mode_swap_rejected() {
        let source="p cnf 3 1\n1 2 3 0\n";
        let cnf=Cnf::parse(source).expect("CNF");
        let route=Guidance::for_source(&cnf,Strategy::Fischer).expect("guide");
        let seed=vec![Trit::Zero;3];
        let digest=hash_bytes(source.as_bytes());
        let mut state=Frontier::new(&cnf,&route,&seed,&digest,FrontierMode::Nine)
            .expect("new");
        state.advance(&cnf,&route,1).expect("one exact source node");
        let snapshot=state.freeze().expect("freeze");
        assert!(Frontier::thaw(
            &snapshot,&cnf,&route,&seed,&digest,FrontierMode::Three
        ).is_err(),"nine cannot thaw as three without paid source");
        let mut altered=snapshot.clone();
        let last=altered.len()-1;
        altered[last]^=1;
        assert!(Frontier::thaw(
            &altered,&cnf,&route,&seed,&digest,FrontierMode::Nine
        ).is_err(),"tampered original source must be rejected");
    }
}
