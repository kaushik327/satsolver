use std::collections::HashSet;

use crate::formula::*;
use crate::solver_state::*;

#[derive(Clone, Debug)]
pub struct CdclState {
    pub state: SolverState,
    pub trail: Vec<TrailElement>,
}

impl CdclState {
    pub fn get_last_decision_index(&self) -> Option<usize> {
        self.trail
            .iter()
            .enumerate()
            .filter(|(_, x)| matches!(x.reason, TrailReason::Decision(_)))
            .last()
            .map(|(x, _)| x)
    }

    pub fn decide_literal_inplace(&mut self, lit: &Lit) {
        let snapshot = self.state.clone();
        self.trail.push(TrailElement {
            lit: lit.clone(),
            reason: TrailReason::Decision(snapshot),
        });
        self.state = self.state.assign(&lit.var, lit.value);
    }

    pub fn decide_literal_outofplace(&self, lit: &Lit) -> Self {
        let mut state = self.clone();
        state.decide_literal_inplace(lit);
        state
    }
}

pub fn solve_cdcl_from_cdcl_state(state: &mut CdclState) -> Option<Assignment> {
    loop {
        while let Some((trail_elem, ucp_result)) = unit_propagate(&state.state) {
            state.state = ucp_result;
            state.trail.push(trail_elem);
        }

        if state.state.is_falsified() {
            // We use the last UIP cut here (i.e. cutting right after the last decision literal)
            let Some(cut_idx) = state.get_last_decision_index() else {
                // If decision level zero, return unsat.
                return None;
            };
            let cut_element = &state.trail[cut_idx];
            let up_to_cut = &state.trail[0..=cut_idx];
            let after_cut = &state.trail[cut_idx + 1..];

            // Get all variables whose values have been decided or inferred before the cut.
            let decided_before_cut = up_to_cut
                .iter()
                .map(|i| i.lit.var.clone())
                .collect::<HashSet<_>>();

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
            state.state = match &cut_element.reason {
                TrailReason::UnitProp(_) => panic!(),
                TrailReason::Decision(snapshot) => snapshot.clone(),
            };
            state.trail.truncate(cut_idx);

            state
                .state
                .learn_clause(Vec::from_iter(lits_in_learned_clause));

            // TODO: If the elements in the learned clause are all literals that were
            // decided multiple decisions beforehand, we can backjump even further.
            // This is not implemented here.
        } else if state.state.is_satisfied() {
            return Some(state.state.assignment.fill_unassigned());
        } else {
            // Decide some random literal and add it to the trail.
            // Note: If the formula is neither falsified nor satisfied, there
            // must be at least one unassigned variable, hence the unwrap().

            let lit = state.state.get_unassigned_lit().unwrap();
            state.decide_literal_inplace(&lit);
        }
    }
}

pub fn solve_cdcl(cnf: &CnfFormula) -> Option<Assignment> {
    let mut state = CdclState {
        state: SolverState::from_cnf(cnf),
        trail: vec![],
    };
    solve_cdcl_from_cdcl_state(&mut state)
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
