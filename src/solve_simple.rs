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
    pub fn solve_backtrack_rec(mut state: SolverState) -> Option<Assignment> {
        if state.is_satisfied() {
            state.assignment.fill_unassigned();
            Some(state.assignment)
        } else if state.is_falsified() {
            None
        } else {
            state.assignment.get_unassigned_var().and_then(|v| {
                let (tstate, fstate) = branch_on_variable(state, v);
                solve_backtrack_rec(fstate).or(solve_backtrack_rec(tstate))
            })
        }
    }
    let blank_state = SolverState::from_cnf(cnf);
    solve_backtrack_rec(blank_state)
}

pub fn solve_dpll(cnf: &CnfFormula) -> Option<Assignment> {
    // Recursively assign each variable to true or false
    pub fn solve_dpll_rec(mut state: SolverState) -> Option<Assignment> {
        state.unit_propagate();
        if state.is_satisfied() {
            state.assignment.fill_unassigned();
            Some(state.assignment)
        } else if state.is_falsified() {
            None
        } else {
            state.assignment.get_unassigned_var().and_then(|v| {
                let (tstate, fstate) = branch_on_variable(state, v);
                solve_dpll_rec(fstate).or(solve_dpll_rec(tstate))
            })
        }
    }
    let mut blank_state = SolverState::from_cnf(cnf);
    blank_state.pure_literal_eliminate();
    solve_dpll_rec(blank_state)
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
