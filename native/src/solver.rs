//! Complete exact ternary-state DPLL for source-frozen Boolean CNF.
//! Unary trit updates: -1=false, 0=unassigned, +1=true.
//! Correct SAT search must explore BOTH nonzero values at each decision;
//! 3/27/81 are routing arities, not extra Boolean truth values.

use crate::guidance::Guidance;
use crate::ternary::{Cnf, Trit};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Sat,
    Unsat,
    Hold,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sat => "SAT",
            Self::Unsat => "UNSAT",
            Self::Hold => "HOLD_RESOURCE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Solution {
    pub status: Status,
    pub model: Option<Vec<Trit>>,
    pub nodes: u64,
    pub propagations: u64,
    pub literal_checks: u64,
    pub budget: Option<u64>,
}

enum Search {
    Sat(Vec<Trit>),
    Unsat,
    Hold,
}

struct Searcher<'a> {
    cnf: &'a Cnf,
    order: &'a [usize],
    nodes: u64,
    propagations: u64,
    literal_checks: u64,
    budget: Option<u64>,
}

impl Searcher<'_> {
    fn increment_checks(&mut self) {
        self.literal_checks = self.literal_checks.checked_add(1)
            .expect("exact literal-check counter overflow");
    }

    fn assign(&mut self, values: &mut [Trit], variable: usize, target: Trit) -> bool {
        match values[variable - 1] {
            Trit::Zero => {
                values[variable - 1] = target;
                self.propagations = self.propagations.checked_add(1)
                    .expect("exact propagation counter overflow");
                true
            }
            other => other == target,
        }
    }

    fn propagate(&mut self, values: &mut [Trit]) -> bool {
        loop {
            let mut changed = false;
            for clause in &self.cnf.clauses {
                let mut satisfied = false;
                let mut remaining = 0_usize;
                let mut last = 0_i32;
                for &literal in clause {
                    self.increment_checks();
                    let assigned = values[literal.unsigned_abs() as usize - 1];
                    if assigned == Trit::from_literal(literal) {
                        satisfied = true;
                        break;
                    }
                    if assigned == Trit::Zero {
                        remaining += 1;
                        last = literal;
                    }
                }
                if satisfied {
                    continue;
                }
                if remaining == 0 {
                    return false;
                }
                if remaining == 1 {
                    let variable = last.unsigned_abs() as usize;
                    if !self.assign(values, variable, Trit::from_literal(last)) {
                        return false;
                    }
                    changed = true;
                }
            }
            if !changed {
                return true;
            }
        }
    }

    fn recurse(&mut self, original: &[Trit]) -> Search {
        if self.budget.is_some_and(|budget| self.nodes >= budget) {
            return Search::Hold;
        }
        self.nodes = self.nodes.checked_add(1).expect("node counter overflow");
        let mut propagated = original.to_vec();
        if !self.propagate(&mut propagated) {
            return Search::Unsat;
        }
        let choice = self.order.iter()
            .copied()
            .find(|&variable| propagated[variable - 1] == Trit::Zero);
        let Some(variable) = choice else {
            return if self.cnf.valid_complete_model(&propagated) {
                Search::Sat(propagated)
            } else {
                Search::Unsat
            };
        };
        // Unassigned variables not present in clauses may be set to minus.
        // For full correctness, a fully assigned valid model is independently
        // verified before and after search.
        let mut saw_hold = false;
        for branch in [Trit::Plus, Trit::Minus] {
            let mut next = propagated.clone();
            if !self.assign(&mut next, variable, branch) {
                continue;
            }
            match self.recurse(&next) {
                Search::Sat(model) => return Search::Sat(model),
                Search::Hold => saw_hold = true,
                Search::Unsat => {}
            }
        }
        if saw_hold {
            Search::Hold
        } else {
            Search::Unsat
        }
    }
}

pub fn solve(cnf: &Cnf, guidance: &Guidance, budget: Option<u64>) -> Solution {
    let mut searcher = Searcher {
        cnf,
        order: &guidance.order,
        nodes: 0,
        propagations: 0,
        literal_checks: 0,
        budget,
    };
    let initial = vec![Trit::Zero; cnf.variables];
    let status = searcher.recurse(&initial);
    let (status, model) = match status {
        Search::Sat(values) => {
            assert!(cnf.valid_complete_model(&values),
                    "independent original-CNF model check failed");
            (Status::Sat, Some(values))
        }
        Search::Unsat => (Status::Unsat, None),
        Search::Hold => (Status::Hold, None),
    };
    Solution {
        status, model, nodes: searcher.nodes,
        propagations: searcher.propagations,
        literal_checks: searcher.literal_checks,
        budget,
    }
}

pub fn check_against_exhaustive(cnf: &Cnf, solution: &Solution) -> bool {
    if cnf.variables > 16 || cnf.variables >= usize::BITS as usize {
        return false;
    }
    let mut has_model = false;
    for mask in 0..(1_usize << cnf.variables) {
        let states: Vec<Trit> = (0..cnf.variables)
            .map(|index| if (mask >> index) & 1 == 0 {
                Trit::Minus
            } else {
                Trit::Plus
            })
            .collect();
        if cnf.valid_complete_model(&states) {
            has_model = true;
            break;
        }
    }
    match solution.status {
        Status::Sat => has_model && solution.model.as_ref()
            .is_some_and(|values| cnf.valid_complete_model(values)),
        Status::Unsat => !has_model && solution.model.is_none(),
        Status::Hold => solution.model.is_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guidance::Strategy;

    #[test]
    fn nine_native_strategies_are_exact_on_five_independent_truth_classes() {
        let examples = [
            ("p cnf 3 2\n1 2 0\n-2 3 0\n", Status::Sat),
            ("p cnf 1 2\n1 0\n-1 0\n", Status::Unsat),
            ("p cnf 2 4\n1 2 0\n-1 2 0\n1 -2 0\n-1 -2 0\n",
                Status::Unsat),
            ("p cnf 3 1\n0\n", Status::Unsat),
            ("p cnf 4 2\n1 -2 0\n3 -4 0\n", Status::Sat),
        ];
        for (source, expected) in examples {
            let cnf = Cnf::parse(source).expect("valid CNF");
            for strategy in Strategy::all() {
                let guidance = Guidance::for_source(&cnf, strategy)
                    .expect("verified source-derived guidance");
                let result = solve(&cnf, &guidance, None);
                assert_eq!(result.status, expected, "{}", strategy.label());
                assert!(check_against_exhaustive(&cnf, &result),
                        "independent exhaustive SAT verification");
            }
        }
    }

    #[test]
    fn bounded_search_returns_an_honest_hold() {
        let cnf = Cnf::parse("p cnf 2 1\n1 2 0\n").expect("source");
        let guided = Guidance::for_source(&cnf, Strategy::Base).expect("source");
        let stopped = solve(&cnf, &guided, Some(0));
        assert_eq!(stopped.status, Status::Hold);
        assert_eq!(stopped.nodes, 0);
        assert!(check_against_exhaustive(&cnf, &stopped));
        let complete = solve(&cnf, &guided, None);
        assert_eq!(complete.status, Status::Sat);
        assert!(check_against_exhaustive(&cnf, &complete));
    }

    #[test]
    fn exhaustive_crosscheck_all_256_two_clause_boolean_sources() {
        // Each 2-variable clause has four possible signed literals;
        // check all 16 pairs with repetition, and all nine strategies.
        let literals = [1, -1, 2, -2];
        let mut checked = 0;
        for &a in &literals {
            for &b in &literals {
                for &c in &literals {
                    for &d in &literals {
                        let cnf = Cnf {
                            variables: 2,
                            clauses: vec![vec![a, b], vec![c, d]],
                        };
                        for strategy in Strategy::all() {
                            let order = Guidance::for_source(&cnf, strategy)
                                .expect("source guidance");
                            let result = solve(&cnf, &order, None);
                            assert!(check_against_exhaustive(&cnf, &result));
                            checked += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(checked, 2_304);
    }
}
