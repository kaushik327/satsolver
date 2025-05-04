use std::collections::HashSet;

use crate::formula::*;
use crate::solver_state::*;

pub fn solve_cdcl_from_state(mut state: SolverState) -> Option<Assignment> {
    loop {
        // println!("\n{:?}", &state);

        state.unit_propagate();

        match state.get_status() {
            Status::Falsified => {
                // We use the last UIP cut here (i.e. cutting right after the last decision literal)
                let Some(cut_idx) = state.get_last_decision_index() else {
                    // If decision level zero, return unsat.
                    return None;
                };
                let cut_element = &state.trail[cut_idx];
                let up_to_cut = &state.trail[0..=cut_idx];
                let after_cut = &state.trail[cut_idx + 1..];

                // Get all variables whose values have been decided or inferred before the cut.
                let decided_before_cut =
                    up_to_cut.iter().map(|i| i.lit.var).collect::<HashSet<_>>();

                // Get all literals that were decided or inferred before the cut,
                // and were used to infer literals after the cut (and the contradiction).
                // The negations of these literals are put into the learned clause.
                let lits_in_learned_clause = after_cut
                    .iter()
                    .flat_map(|i| match &i.reason {
                        TrailReason::UnitProp(clause) => clause
                            .literals
                            .iter()
                            .filter(|lit| decided_before_cut.contains(&lit.var)),
                        _ => panic!(),
                    })
                    .map(|lit| lit.not())
                    .collect::<HashSet<_>>();

                // Backjumping to snapshotted state
                state.assignment = match &cut_element.reason {
                    TrailReason::UnitProp(_) => panic!(),
                    TrailReason::Decision(assignment) => assignment.clone(),
                };
                state.trail.truncate(cut_idx);

                state.learn_clause(Vec::from_iter(lits_in_learned_clause));

                // TODO: If the elements in the learned clause are all literals that were
                // decided multiple decisions beforehand, we can backjump even further.
                // This is not implemented here.
            }
            Status::Satisfied => {
                state.assignment.fill_unassigned();
                return Some(state.assignment);
            }
            Status::Unassigned(lit) => {
                // Decide some random literal and add it to the trail.

                state.decide(lit.var, lit.value);
            }
        }
    }
}

pub fn solve_cdcl(cnf: &CnfFormula) -> Option<Assignment> {
    let state = SolverState::from_cnf(cnf);
    solve_cdcl_from_state(state)
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
