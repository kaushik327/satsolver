use std::collections::BTreeSet;

use itertools::Itertools;
use log::info;

use crate::config::*;
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

// Luby sequence: luby(i) for i = 0, 1, 2, ...
// Uses the two-counter recurrence: maintain (u, v); when v catches up to a
// power-of-two boundary (u & -u == v), reset v to 1 and advance u.
struct LubySequence {
    u: u32,
    v: u32,
}

impl LubySequence {
    fn new() -> Self {
        Self { u: 1, v: 1 }
    }

    fn next(&mut self) -> u32 {
        let val = self.v;
        if self.u & self.u.wrapping_neg() == self.v {
            self.u += 1;
            self.v = 1;
        } else {
            self.v *= 2;
        }
        val
    }
}

pub fn solve_cdcl_from_state(mut state: SolverState, config: &SolverConfig) -> SolverResult {
    info!("Initial formula: {}", state.formula);

    let mut luby = LubySequence::new();
    // Absolute conflict count at which to trigger the next restart
    let mut next_restart: u32 = match config.restart {
        RestartStrategy::None => u32::MAX,
        RestartStrategy::Luby { unit } => luby.next() * unit,
        RestartStrategy::Geometric { initial, .. } => initial,
    };
    let mut geometric_gap: u32 = match config.restart {
        RestartStrategy::Geometric { initial, .. } => initial,
        _ => 0,
    };

    loop {
        match state.get_status() {
            Status::Satisfied => {
                return SolverResult::Satisfiable(state.assignment.fill_unassigned());
            }
            Status::UnassignedDecision(var) => {
                let value = match config.polarity {
                    PolarityHeuristic::AlwaysFalse => Val::False,
                    PolarityHeuristic::AlwaysTrue => Val::True,
                    PolarityHeuristic::PhaseSaving => state.get_phase(var),
                };
                info!("Guess: {}", Lit { var, value });
                state.decide(var, value);
            }
            Status::UnassignedUnit(lit, clause) => {
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
                        state.bump_var_activity(&learned_clause);
                        state.learn_clause(learned_clause);
                        state.backjump_to_decision_level(backjump_level);
                        state.conflict_count += 1;

                        if state.conflict_count >= next_restart {
                            info!(
                                "Restart at conflict {}, {} learned clauses",
                                state.conflict_count,
                                state.formula.clauses.len()
                            );
                            state.restart();
                            next_restart = state.conflict_count
                                + match config.restart {
                                    RestartStrategy::None => u32::MAX,
                                    RestartStrategy::Luby { unit } => luby.next() * unit,
                                    RestartStrategy::Geometric { factor, .. } => {
                                        geometric_gap =
                                            ((geometric_gap as f64) * factor).ceil() as u32;
                                        geometric_gap
                                    }
                                };
                        }

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

pub fn solve_cdcl(cnf: &CnfFormula, config: &SolverConfig) -> SolverResult {
    let mut state = SolverState::from_cnf(cnf);
    state.pure_literal_eliminate();
    solve_cdcl_from_state(state, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dimacs_str;

    #[test]
    fn test_solve_cdcl_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        let result = solve_cdcl(&cnf, &SolverConfig::default());
        assert!(result.is_satisfiable());
        let assignment = result.into_assignment().unwrap();
        assert!(assignment.get_unassigned_var().is_none() && check_assignment(&cnf, &assignment));
    }

    #[test]
    fn test_solve_cdcl_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(!solve_cdcl(&cnf, &SolverConfig::default()).is_satisfiable());
    }

    #[test]
    fn test_phase_saving_remembered_after_backjump() {
        let cnf = parse_dimacs_str(b"\np cnf 3 2\n1 2 0\n-1 3 0").unwrap();
        let mut state = SolverState::from_cnf(&cnf);
        let var1 = Var { index: 1 };
        assert_eq!(state.get_phase(var1), Val::False);
        state.decide(var1, Val::True);
        assert_eq!(state.get_phase(var1), Val::True);
        state.backjump_to_decision_level(0);
        // Phase is preserved across backjumps
        assert_eq!(state.get_phase(var1), Val::True);
        assert!(state.assignment.get_unassigned_var().is_some());
    }

    #[test]
    fn test_vsids_prefers_recently_conflicting_var() {
        let cnf = parse_dimacs_str(b"\np cnf 4 4\n1 2 0\n3 4 0\n-1 -2 0\n-3 -4 0").unwrap();
        let mut state = SolverState::from_cnf(&cnf);
        let clause = Clause {
            literals: vec![
                Lit {
                    var: Var { index: 3 },
                    value: Val::True,
                },
                Lit {
                    var: Var { index: 4 },
                    value: Val::True,
                },
            ],
        };
        state.bump_var_activity(&clause);
        let decision = state.next_decision_var().unwrap();
        assert!(decision.index == 3 || decision.index == 4);
    }

    #[test]
    fn test_luby_sequence() {
        // 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8, ...
        let expected = [1u32, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8];
        let mut luby = LubySequence::new();
        for &exp in &expected {
            assert_eq!(luby.next(), exp);
        }
    }

    // Pigeonhole formula: 4 pigeons, 3 holes — UNSAT, good stress test.
    const PIGEON_4_3: &[u8] = b"p cnf 12 22
1 2 3 0
4 5 6 0
7 8 9 0
10 11 12 0
-1 -4 0
-1 -7 0
-1 -10 0
-4 -7 0
-4 -10 0
-7 -10 0
-2 -5 0
-2 -8 0
-2 -11 0
-5 -8 0
-5 -11 0
-8 -11 0
-3 -6 0
-3 -9 0
-3 -12 0
-6 -9 0
-6 -12 0
-9 -12 0
";

    #[test]
    fn test_luby_restart_terminates_unsat() {
        let cnf = parse_dimacs_str(PIGEON_4_3).unwrap();
        let config = SolverConfig {
            polarity: PolarityHeuristic::AlwaysFalse,
            restart: RestartStrategy::Luby { unit: 5 },
            deletion: DeletionStrategy::None,
        };
        assert!(!solve_cdcl(&cnf, &config).is_satisfiable());
    }

    #[test]
    fn test_geometric_restart_terminates_unsat() {
        let cnf = parse_dimacs_str(PIGEON_4_3).unwrap();
        let config = SolverConfig {
            polarity: PolarityHeuristic::AlwaysFalse,
            restart: RestartStrategy::Geometric {
                initial: 5,
                factor: 1.5,
            },
            deletion: DeletionStrategy::None,
        };
        assert!(!solve_cdcl(&cnf, &config).is_satisfiable());
    }

    #[test]
    fn test_polarity_options_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        for polarity in [
            PolarityHeuristic::AlwaysFalse,
            PolarityHeuristic::AlwaysTrue,
            PolarityHeuristic::PhaseSaving,
        ] {
            let config = SolverConfig {
                polarity,
                restart: RestartStrategy::None,
                deletion: DeletionStrategy::None,
            };
            let result = solve_cdcl(&cnf, &config);
            assert!(
                result.is_satisfiable(),
                "Expected SAT for polarity {polarity:?}"
            );
            assert!(check_assignment(&cnf, &result.into_assignment().unwrap()));
        }
    }
}
