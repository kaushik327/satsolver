use std::collections::BTreeSet;

use itertools::Itertools;
use log::info;

use crate::formula::*;
use crate::solver_state::*;

struct ConflictingLits<'a> {
    literals: BTreeSet<(u32, Lit)>,
    state: &'a SolverState,
}

impl<'a> ConflictingLits<'a> {
    fn new(falsified_clause: Clause, state: &'a SolverState) -> Self {
        let mut ret = Self {
            literals: BTreeSet::new(),
            state,
        };
        for lit in &falsified_clause.literals {
            ret.insert(lit.not());
        }
        ret
    }

    fn get_backjump_level(&self) -> u32 {
        let mut decision_levels = self.literals.iter().map(|(level, _)| *level);

        // Get the last (highest) decision level
        let last_level = decision_levels.next_back().unwrap();
        assert_eq!(last_level, self.state.decision_level);

        // Get the second-to-last decision level, or 0 if none exists
        decision_levels.next_back().unwrap_or(0)
    }

    fn get_learned_clause(&self) -> Clause {
        Clause {
            literals: self.literals.iter().map(|(_, lit)| lit.not()).collect(),
        }
    }

    fn insert(&mut self, lit: Lit) {
        self.literals
            .insert((self.state.assignment.get_decision_level(&lit).unwrap(), lit));
    }

    fn remove(&mut self, lit: Lit) {
        self.literals
            .remove(&(self.state.assignment.get_decision_level(&lit).unwrap(), lit));
    }

    fn update(&mut self, trail_element: &TrailElement) {
        let TrailReason::UnitProp(clause) = &trail_element.reason else {
            // We should never be moving the UIP cut behind the last decision level.
            unreachable!();
        };

        info!("\tTrail element: {trail_element} from {clause}");

        for lit in clause.literals.iter() {
            if lit.var == trail_element.lit.var {
                assert!(lit.value == trail_element.lit.value);
            } else {
                self.insert(lit.not());
            }
        }
        self.remove(trail_element.lit);
    }
}

impl std::fmt::Display for ConflictingLits<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> ø",
            self.literals
                .iter()
                .map(|(level, lit)| format!("{lit}({level})"))
                .join(" ^ ")
        )
    }
}

pub fn solve_cdcl_from_state(mut state: SolverState) -> SolverResult {
    info!("Initial formula: {}", state.formula);
    loop {
        match state.get_status() {
            Status::Satisfied => {
                return SolverResult::Satisfiable(state.assignment.fill_unassigned());
            }
            Status::UnassignedDecision(lit) => {
                // Decide some unassigned literal and add it to the trail.
                info!("Guess: {lit}");
                state.decide(lit.var, lit.value);
            }
            Status::UnassignedUnit(lit, clause) => {
                // Unit-clause propagation available
                info!("Unit: {lit} from {clause}");
                state.assign_unitprop(lit.var, lit.value, clause);
            }
            Status::Falsified(falsified_clause) => {
                // We start with the cut placed after all unit propagations,
                // and incrementally move it backwards until the ensuing
                // learned clause would contain exactly one literal from
                // the current decision level.

                info!(
                    "Falsified {} at trail: {}",
                    falsified_clause,
                    state.trail.iter().join(" ")
                );

                if state.decision_level == 0 {
                    return SolverResult::UnsatisfiableWithProof(state.formula.clauses);
                }

                let mut conflict = ConflictingLits::new(falsified_clause, &state);

                for trail_element in state.trail.iter().rev() {
                    // Check the decision levels of the learned clause's literals.
                    info!("\tContradiction: {conflict}");
                    let backjump_level = conflict.get_backjump_level();
                    if backjump_level != state.decision_level {
                        // We have found a UIP cut.
                        let learned_clause = conflict.get_learned_clause();
                        info!(
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
                    conflict.update(trail_element);
                }
            }
        }
    }
}

pub fn solve_cdcl(cnf: &CnfFormula) -> SolverResult {
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
        let result = solve_cdcl(&cnf);
        assert!(result.is_satisfiable());
        let assignment = result.into_assignment().unwrap();
        assert!(assignment.get_unassigned_var().is_none() && check_assignment(&cnf, &assignment));
    }

    #[test]
    fn test_solve_cdcl_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(!solve_cdcl(&cnf).is_satisfiable());
    }
}
