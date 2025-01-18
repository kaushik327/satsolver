use crate::formula::*;
use crate::solver_state::*;

use itertools::Itertools;

pub fn solve_basic(cnf: &CnfFormula) -> Option<Assignment> {
    // Literally iterate through every possible assignment.
    std::iter::repeat([Some(Val::False), Some(Val::True)])
        .take(cnf.num_vars)
        .multi_cartesian_product()
        .map(|v| Assignment::from_vector(v.to_vec()))
        .find(|assignment| check_assignment(cnf, assignment))
}

pub fn solve_backtrack(cnf: &CnfFormula) -> Option<Assignment> {
    // Recursively assign each variable to true or false
    pub fn solve_backtrack_rec(state: &SolverState) -> Option<Assignment> {
        if state.is_satisfied() {
            Some(state.assignment.fill_unassigned())
        } else if state.is_falsified() {
            None
        } else {
            state.assignment.get_unassigned_var().and_then(|v| {
                solve_backtrack_rec(&state.assign(&v, Val::False))
                    .or(solve_backtrack_rec(&state.assign(&v, Val::True)))
            })
        }
    }
    let blank_state = SolverState::from_cnf(cnf);
    solve_backtrack_rec(&blank_state)
}

pub fn solve_dpll(cnf: &CnfFormula) -> Option<Assignment> {
    // Recursively assign each variable to true or false
    pub fn solve_dpll_rec(state: &SolverState) -> Option<Assignment> {
        let mut ucp_state = state.clone();
        while let Some(ucp_result) = unit_propagate(&ucp_state) {
            (_, ucp_state) = ucp_result;
        }
        if ucp_state.is_satisfied() {
            Some(ucp_state.assignment.fill_unassigned())
        } else if ucp_state.is_falsified() {
            None
        } else {
            ucp_state.assignment.get_unassigned_var().and_then(|v| {
                solve_dpll_rec(&state.assign(&v, Val::False))
                    .or(solve_dpll_rec(&state.assign(&v, Val::True)))
            })
        }
    }
    let blank_state = SolverState::from_cnf(cnf);
    let ple_state = pure_literal_eliminate(&blank_state);
    solve_dpll_rec(&ple_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dimacs_str;

    #[test]
    fn test_solve_basic_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        assert!(solve_basic(&cnf)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    fn test_solve_basic_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_basic(&cnf).is_none());
    }

    #[test]
    fn test_solve_backtrack_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        assert!(solve_backtrack(&cnf)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    fn test_solve_backtrack_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_backtrack(&cnf).is_none());
    }

    #[test]
    fn test_solve_dpll_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        assert!(solve_dpll(&cnf)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    fn test_solve_dpll_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_dpll(&cnf).is_none());
    }
}
