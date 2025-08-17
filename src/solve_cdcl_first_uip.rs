use std::collections::HashSet;

use itertools::Itertools;

use crate::formula::*;
use crate::solver_state::*;

pub fn solve_cdcl_first_uip_from_state(mut state: SolverState) -> Option<Assignment> {
    loop {
        eprintln!("Before unit propagation: {state}");
        state.unit_propagate();
        eprintln!("After unit propagation: {state}");
        match state.get_status() {
            Status::Satisfied => {
                return Some(state.assignment.fill_unassigned());
            }
            Status::Unassigned(lit) => {
                // Decide some unassigned literal and add it to the trail.
                state.decide(lit.var, lit.value);
                eprintln!("After decision: {state}");
            }
            Status::Falsified(falsified_clause) => {
                // We start with the cut placed after all unit propagations,
                // and incrementally move it backwards until the ensuing
                // learned clause would contain exactly one literal from
                // the current decision level.

                if state.decision_level == 0 {
                    return None;
                }

                let mut left_of_cut = HashSet::<Lit>::from_iter(
                    falsified_clause.literals.into_iter().map(|lit| lit.not()),
                );

                eprintln!("Initial left-of-cut lits: {}", left_of_cut.iter().join(","));

                eprintln!(
                    "Trail: {}",
                    state.trail.iter().map(|te| te.lit.clone()).join(",")
                );

                for trail_element in state.trail.iter().rev() {
                    // Move the cut from the right to the left of this trail element.
                    // That involves removing this element's literal from the learned
                    // clause (if it is present), but adding the literals that were used to infer this
                    // element (if they are not already present).

                    eprintln!("Trail element: {trail_element:?}");

                    match &trail_element.reason {
                        TrailReason::UnitProp(clause) => {
                            for lit in clause.literals.iter() {
                                if lit.var == trail_element.lit.var {
                                    assert!(lit.value == trail_element.lit.value);
                                } else {
                                    left_of_cut.insert(lit.not());
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                    assert!(left_of_cut.remove(&trail_element.lit));

                    eprintln!(
                        "Left-of-cut lits after move: {}",
                        left_of_cut.iter().join(",")
                    );

                    // Check the decision levels of the learned clause's literals.
                    let decision_levels = HashSet::<u32>::from_iter(
                        left_of_cut
                            .iter()
                            .map(|lit| state.assignment.get_decision_level(lit).unwrap()),
                    );

                    let current_level_count = decision_levels
                        .iter()
                        .filter(|&&level| level == state.decision_level)
                        .count();
                    assert!(current_level_count > 0);
                    if current_level_count == 1 {
                        // We have found a UIP cut.
                        // Add the learned clause to the state
                        let learned_clause = Clause {
                            literals: left_of_cut.into_iter().map(|lit| lit.not()).collect(),
                        };
                        state.learn_clause(learned_clause);

                        // Get the second-largest decision level and backjump to it
                        let mut sorted_levels: Vec<u32> = decision_levels.into_iter().collect();
                        sorted_levels.sort_unstable();
                        sorted_levels.reverse();

                        let backjump_level = if sorted_levels.len() > 1 {
                            sorted_levels[1]
                        } else {
                            0
                        };

                        state.backjump_to_decision_level(backjump_level);
                        break;
                    }
                }
                eprintln!("After backjump: {state}");
            }
        }
    }
}

pub fn solve_cdcl_first_uip(cnf: &CnfFormula) -> Option<Assignment> {
    let state = SolverState::from_cnf(cnf);
    solve_cdcl_first_uip_from_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dimacs_str;

    #[test]
    fn test_solve_cdcl_first_uip_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        assert!(solve_cdcl_first_uip(&cnf)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    fn test_solve_cdcl_first_uip_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_cdcl_first_uip(&cnf).is_none());
    }
}
