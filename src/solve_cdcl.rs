use std::collections::HashSet;

use crate::formula::*;
use crate::solver_state::*;

use itertools::Itertools;

pub struct CdclState {
    pub state: SolverState,
    pub trail: Vec<TrailElement>,
}

pub fn solve_cdcl(cnf: &CnfFormula) -> Option<Assignment> {
    let mut state = CdclState {
        state: SolverState::from_cnf(cnf),
        trail: vec![],
    };

    loop {
        while let Some((trail_elem, ucp_result)) = unit_propagate(&state.state) {
            state.state = ucp_result;
            state.trail.push(trail_elem);
        }

        if state.state.is_falsified() {
            // We use the last UIP cut here (i.e. cutting right after the last decision literal)
            // TODO: use the first UIP cut

            let Some((cut_idx, cut_element)) = state
                .trail
                .iter()
                .enumerate()
                .filter(|(_, x)| matches!(x.reason, TrailReason::Decision(_)))
                .last()
            else {
                // If decision level zero, return unsat.
                return None;
            };

            let decided_before_cut = state
                .trail
                .iter()
                .take(cut_idx + 1)
                .map(|i| i.lit.var.clone())
                .collect::<HashSet<_>>();

            let lits_in_learned_clause = state
                .trail
                .iter()
                .skip(cut_idx + 1)
                .flat_map(|i| match &i.reason {
                    TrailReason::Decision(_) => {
                        panic!()
                    }
                    TrailReason::UnitProp(clause) => clause
                        .literals
                        .iter()
                        .filter(|lit| decided_before_cut.contains(&lit.var)),
                })
                .map(|lit| Lit {
                    var: lit.var.clone(),
                    value: lit.value.not(),
                })
                .collect::<HashSet<_>>()
                .into_iter()
                .collect_vec();

            // Backjumping to snapshotted state
            state.state = match &cut_element.reason {
                TrailReason::UnitProp(_) => panic!(),
                TrailReason::Decision(snapshot) => snapshot.clone(),
            };
            state.state.clauses.push(SolverClause {
                literals: lits_in_learned_clause.clone(),
                original: lits_in_learned_clause,
            });
            state.trail.truncate(cut_idx);
        } else if state.state.is_satisfied() {
            return Some(state.state.assignment.fill_unassigned());
        } else {
            // Make some random literal true and add it to the trail.
            // Note: If the formula is neither falsified nor satisfied, there
            // must be at least one unassigned variable, hence the unwrap().

            // TODO: we're only deciding true values here. we should also
            // be able to decide false values (really, we should pick a LITERAL
            // from the formula rather than a variable)

            let var = state.state.assignment.get_unassigned_var().unwrap();

            let snapshot = state.state.clone();

            state.trail.push(TrailElement {
                lit: Lit {
                    var: var.clone(),
                    value: Val::True,
                },
                reason: TrailReason::Decision(snapshot),
            });
            state.state = state.state.assign(&var, Val::True);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dimacs_str;

    #[test]
    fn test_solve_cdcl_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        assert!(solve_cdcl(&cnf)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    fn test_solve_cdcl_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_cdcl(&cnf).is_none());
    }
}
