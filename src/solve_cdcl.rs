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

    fn contains(&self, lit: Lit) -> bool {
        let level = self.state.assignment.get_decision_level(&lit).unwrap();
        self.literals.contains(&(level, lit))
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

pub fn solve_cdcl_from_state(mut state: SolverState, config: &SolverConfig) -> SolverResult {
    info!("Initial formula: {}", state.formula);

    let mut scheduler = RestartScheduler::new(config.restart);

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
                // the current decision level (the 1-UIP condition).

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
                    info!("\tContradiction: {conflict}");
                    let backjump_level = conflict.get_backjump_level();
                    if backjump_level != state.decision_level {
                        // 1-UIP found: exactly one literal remains at the current level.
                        let learned_clause = conflict.get_learned_clause();
                        debug_assert_eq!(
                            learned_clause
                                .literals
                                .iter()
                                .filter(|lit| {
                                    state.assignment.get_decision_level(lit)
                                        == Some(state.decision_level)
                                })
                                .count(),
                            1,
                            "1-UIP invariant: learned clause must have exactly one literal \
                             at the current decision level"
                        );
                        info!(
                            "\tBackjumping from level {} to level {}, learning clause {}",
                            state.decision_level, backjump_level, learned_clause
                        );
                        state.bump_var_activity(&learned_clause);
                        state.learn_clause_with_meta(learned_clause);
                        state.backjump_to_decision_level(backjump_level);
                        state.conflict_count += 1;

                        if scheduler.should_restart(state.conflict_count) {
                            info!(
                                "Restart at conflict {}, {} learned clauses",
                                state.conflict_count,
                                state.formula.clauses.len()
                            );
                            state.restart();
                            state.delete_weak_learned_clauses(&config.deletion);
                            scheduler.advance(state.conflict_count);
                        }

                        break;
                    }

                    // Only expand trail elements whose literal is in the conflict set.
                    // Expanding unrelated literals would add spurious antecedents to the
                    // learned clause, producing a weaker-than-1-UIP result.
                    if conflict.contains(trail_element.lit) {
                        conflict.update(trail_element);
                    }
                }
            }
        }
    }
}

pub fn solve_cdcl(cnf: &CnfFormula, config: &SolverConfig) -> SolverResult {
    let mut state = SolverState::from_cnf(cnf);
    state.pure_literal_eliminate();
    state.seal_original_clauses();
    solve_cdcl_from_state(state, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dimacs_str;
    use crate::solve_simple::{solve_backtrack, solve_dpll};

    fn default_config() -> SolverConfig {
        SolverConfig::default()
    }

    #[test]
    fn test_solve_cdcl_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        let result = solve_cdcl(&cnf, &default_config());
        assert!(result.is_satisfiable());
        let assignment = result.into_assignment().unwrap();
        assert!(assignment.get_unassigned_var().is_none() && check_assignment(&cnf, &assignment));
    }

    #[test]
    fn test_solve_cdcl_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(!solve_cdcl(&cnf, &default_config()).is_satisfiable());
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
    fn test_all_configs_agree_unsat() {
        let cnf = parse_dimacs_str(PIGEON_4_3).unwrap();
        let configs = [
            SolverConfig {
                polarity: PolarityHeuristic::AlwaysFalse,
                restart: RestartStrategy::None,
                deletion: DeletionStrategy::None,
            },
            SolverConfig {
                polarity: PolarityHeuristic::PhaseSaving,
                restart: RestartStrategy::Luby { unit: 1 },
                deletion: DeletionStrategy::Lbd { max_lbd: 3 },
            },
            SolverConfig {
                polarity: PolarityHeuristic::AlwaysTrue,
                restart: RestartStrategy::Geometric {
                    initial: 10,
                    factor: 2.0,
                },
                deletion: DeletionStrategy::Activity { fraction: 0.5 },
            },
        ];
        for config in &configs {
            assert!(
                !solve_cdcl(&cnf, config).is_satisfiable(),
                "Expected UNSAT for config {config:?}"
            );
        }
    }

    #[test]
    fn test_all_configs_agree_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 10 5\n1 2 3 0\n4 5 6 0\n7 8 9 0\n1 -4 7 0\n2 5 -8 0")
            .unwrap();
        let configs = [
            SolverConfig {
                polarity: PolarityHeuristic::AlwaysFalse,
                restart: RestartStrategy::None,
                deletion: DeletionStrategy::None,
            },
            SolverConfig {
                polarity: PolarityHeuristic::PhaseSaving,
                restart: RestartStrategy::Luby { unit: 1 },
                deletion: DeletionStrategy::Lbd { max_lbd: 3 },
            },
            SolverConfig {
                polarity: PolarityHeuristic::AlwaysTrue,
                restart: RestartStrategy::Geometric {
                    initial: 3,
                    factor: 1.5,
                },
                deletion: DeletionStrategy::Activity { fraction: 0.5 },
            },
        ];
        for config in &configs {
            let result = solve_cdcl(&cnf, config);
            assert!(
                result.is_satisfiable(),
                "Expected SAT for config {config:?}"
            );
            assert!(check_assignment(&cnf, &result.into_assignment().unwrap()));
        }
    }

    #[test]
    fn test_deletion_lbd_preserves_correctness() {
        let cnf = parse_dimacs_str(PIGEON_4_3).unwrap();
        let config = SolverConfig {
            polarity: PolarityHeuristic::AlwaysFalse,
            restart: RestartStrategy::Luby { unit: 1 },
            deletion: DeletionStrategy::Lbd { max_lbd: 1 },
        };
        assert!(!solve_cdcl(&cnf, &config).is_satisfiable());
    }

    #[test]
    fn test_deletion_activity_preserves_correctness() {
        let cnf = parse_dimacs_str(PIGEON_4_3).unwrap();
        let config = SolverConfig {
            polarity: PolarityHeuristic::AlwaysFalse,
            restart: RestartStrategy::Luby { unit: 1 },
            deletion: DeletionStrategy::Activity { fraction: 0.9 },
        };
        assert!(!solve_cdcl(&cnf, &config).is_satisfiable());
    }

    // Cross-solver agreement: CDCL, DPLL, and backtrack must agree on SAT/UNSAT
    // for a suite of formulas. This catches incorrect learned clauses or
    // backjump bugs that don't affect termination but do affect correctness.

    fn assert_solvers_agree(cnf_bytes: &[u8]) {
        let cnf = parse_dimacs_str(cnf_bytes).unwrap();
        let cdcl = solve_cdcl(&cnf, &default_config());
        let dpll = solve_dpll(&cnf);
        let backtrack = solve_backtrack(&cnf);
        assert_eq!(
            cdcl.is_satisfiable(),
            dpll.is_satisfiable(),
            "CDCL and DPLL disagree on SAT/UNSAT"
        );
        assert_eq!(
            cdcl.is_satisfiable(),
            backtrack.is_satisfiable(),
            "CDCL and backtrack disagree on SAT/UNSAT"
        );
        if let Some(assignment) = cdcl.assignment() {
            assert!(
                check_assignment(&cnf, assignment),
                "CDCL returned an assignment that doesn't satisfy the formula"
            );
        }
    }

    #[test]
    fn test_cross_solver_sat_chain() {
        // x1→x2→x3→x4→x5, satisfiable
        assert_solvers_agree(b"p cnf 5 4\n-1 2 0\n-2 3 0\n-3 4 0\n-4 5 0\n");
    }

    #[test]
    fn test_cross_solver_sat_near_phase_transition() {
        // Structured 3-SAT near the phase transition
        assert_solvers_agree(
            b"p cnf 8 12\n\
              1 2 3 0\n-1 -2 3 0\n1 -2 -3 0\n-1 2 -3 0\n\
              4 5 6 0\n-4 -5 6 0\n4 -5 -6 0\n-4 5 -6 0\n\
              1 4 7 0\n2 5 8 0\n-1 -4 7 0\n-2 -5 8 0\n",
        );
    }

    #[test]
    fn test_cross_solver_unsat_minimal() {
        // Minimal UNSAT: {x1} ∧ {¬x1}
        assert_solvers_agree(b"p cnf 1 2\n1 0\n-1 0\n");
    }

    #[test]
    fn test_cross_solver_unsat_pigeon() {
        assert_solvers_agree(PIGEON_4_3);
    }

    #[test]
    fn test_cross_solver_unsat_structured() {
        // Partial 3-coloring constraints on K4 — UNSAT
        assert_solvers_agree(
            b"p cnf 12 30\n\
              1 2 3 0\n4 5 6 0\n7 8 9 0\n10 11 12 0\n\
              -1 -2 0\n-1 -3 0\n-2 -3 0\n\
              -4 -5 0\n-4 -6 0\n-5 -6 0\n\
              -7 -8 0\n-7 -9 0\n-8 -9 0\n\
              -10 -11 0\n-10 -12 0\n-11 -12 0\n\
              -1 -4 0\n-2 -5 0\n-3 -6 0\n\
              -1 -7 0\n-2 -8 0\n-3 -9 0\n\
              -1 -10 0\n-2 -11 0\n-3 -12 0\n\
              -4 -7 0\n-5 -8 0\n-6 -9 0\n\
              -4 -10 0\n-5 -11 0\n",
        );
    }

    #[test]
    fn test_cross_solver_sat_xor_chain() {
        // XOR chain: (x1 ∨ x2) ∧ (¬x1 ∨ ¬x2), etc. — satisfiable
        assert_solvers_agree(b"p cnf 4 6\n1 2 0\n-1 -2 0\n2 3 0\n-2 -3 0\n3 4 0\n-3 -4 0\n");
    }
}
