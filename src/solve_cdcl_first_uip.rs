use std::collections::HashSet;

use itertools::Itertools;

use crate::formula::*;
use crate::solver_state::*;

pub fn solve_cdcl_first_uip_from_state(mut state: SolverState) -> Option<Assignment> {
    eprintln!("Initial formula: {}", state.formula);
    eprintln!("Initial trail: {}", state.trail.iter().join(" "));
    loop {
        state.unit_propagate();
        eprintln!("Unit propagation: {}", state.trail.iter().join(" "));
        match state.get_status() {
            Status::Satisfied => {
                return Some(state.assignment.fill_unassigned());
            }
            Status::UnassignedDecision(lit) => {
                // Decide some unassigned literal and add it to the trail.
                state.decide(lit.var, lit.value);
                eprintln!("Decision: {}", state.trail.iter().join(" "));
            }
            Status::UnassignedUnit(_, _) => unreachable!(),
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

                eprintln!("Trail: {}", state.trail.iter().join(" "));

                for trail_element in state.trail.iter().rev() {
                    // Check the decision levels of the learned clause's literals.
                    let lits_and_decision_levels = Vec::from_iter(
                        left_of_cut
                            .iter()
                            .map(|lit| (lit, state.assignment.get_decision_level(lit).unwrap())),
                    );

                    eprintln!(
                        "\tLeft of cut: {}",
                        lits_and_decision_levels
                            .iter()
                            .map(|(lit, level)| format!("{lit}({level})"))
                            .join(" ")
                    );

                    let decision_levels = lits_and_decision_levels
                        .into_iter()
                        .map(|(_, level)| level)
                        .collect::<Vec<_>>();

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

                        // Get the second-largest decision level and backjump to it
                        let mut sorted_levels: Vec<u32> = decision_levels.into_iter().collect();
                        sorted_levels.sort_unstable();
                        sorted_levels.reverse();

                        let backjump_level = if sorted_levels.len() > 1 {
                            sorted_levels[1]
                        } else {
                            0
                        };

                        eprintln!(
                            "\tBackjumping from level {} to level {}, learning clause {}",
                            state.decision_level, backjump_level, learned_clause
                        );
                        state.learn_clause(learned_clause);
                        state.backjump_to_decision_level(backjump_level);
                        break;
                    }

                    // Move the cut from the right to the left of this trail element.
                    // That involves removing this element's literal from the learned
                    // clause (if it is present), but adding the literals that were used to infer this
                    // element (if they are not already present).
                    match &trail_element.reason {
                        TrailReason::UnitProp(clause) => {
                            eprintln!(
                                "\tJumping past trail element: {trail_element} from {clause}"
                            );
                            for lit in clause.literals.iter() {
                                if lit.var == trail_element.lit.var {
                                    assert!(lit.value == trail_element.lit.value);
                                } else {
                                    left_of_cut.insert(lit.not());
                                }
                            }
                        }
                        // We should never be moving the UIP cut behind the last decision level.
                        _ => unreachable!(),
                    }
                    left_of_cut.remove(&trail_element.lit);
                }
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
