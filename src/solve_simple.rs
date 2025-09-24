use crate::formula::*;
use crate::solver_state::*;

pub fn solve_basic(cnf: &CnfFormula) -> SolverResult {
    // Literally iterate through every possible assignment.
    match Assignment::every_possible(cnf.num_vars)
        .find(|assignment| check_assignment(cnf, assignment))
    {
        Some(assignment) => SolverResult::Satisfiable(assignment),
        None => SolverResult::Unsatisfiable,
    }
}

pub fn solve_backtrack(cnf: &CnfFormula) -> SolverResult {
    // Recursively assign each variable to true or false
    pub fn solve_backtrack_rec(state: SolverState) -> SolverResult {
        match state.get_status() {
            Status::Satisfied => SolverResult::Satisfiable(state.assignment.fill_unassigned()),
            Status::Falsified(_) => SolverResult::Unsatisfiable,
            Status::UnassignedDecision(lit) | Status::UnassignedUnit(lit, _) => {
                let (tstate, fstate) = branch_on_variable(state, lit.var);
                match solve_backtrack_rec(fstate) {
                    SolverResult::Satisfiable(assignment) => SolverResult::Satisfiable(assignment),
                    SolverResult::Unsatisfiable => solve_backtrack_rec(tstate),
                }
            }
        }
    }
    let blank_state = SolverState::from_cnf(cnf);
    solve_backtrack_rec(blank_state)
}

pub fn solve_dpll(cnf: &CnfFormula) -> SolverResult {
    // Recursively assign each variable to true or false
    pub fn solve_dpll_rec(mut state: SolverState) -> SolverResult {
        match state.get_status() {
            Status::Satisfied => SolverResult::Satisfiable(state.assignment.fill_unassigned()),
            Status::Falsified(_) => SolverResult::Unsatisfiable,
            Status::UnassignedDecision(lit) => {
                let (tstate, fstate) = branch_on_variable(state, lit.var);
                match solve_dpll_rec(fstate) {
                    SolverResult::Satisfiable(assignment) => SolverResult::Satisfiable(assignment),
                    SolverResult::Unsatisfiable => solve_dpll_rec(tstate),
                }
            }
            Status::UnassignedUnit(lit, clause) => {
                state.assign_unitprop(lit.var, lit.value, clause);
                solve_dpll_rec(state)
            }
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
        let result = solve_basic(&cnf);
        assert!(result.is_satisfiable());
        let assignment = result.into_assignment().unwrap();
        assert!(assignment.get_unassigned_var().is_none() && check_assignment(&cnf, &assignment));
    }

    #[test]
    fn test_solve_basic_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_basic(&cnf).is_unsatisfiable());
    }

    #[test]
    fn test_solve_backtrack_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        let result = solve_backtrack(&cnf);
        assert!(result.is_satisfiable());
        let assignment = result.into_assignment().unwrap();
        assert!(assignment.get_unassigned_var().is_none() && check_assignment(&cnf, &assignment));
    }

    #[test]
    fn test_solve_backtrack_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_backtrack(&cnf).is_unsatisfiable());
    }

    #[test]
    fn test_solve_dpll_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        let result = solve_dpll(&cnf);
        assert!(result.is_satisfiable());
        let assignment = result.into_assignment().unwrap();
        assert!(assignment.get_unassigned_var().is_none() && check_assignment(&cnf, &assignment));
    }

    #[test]
    fn test_solve_dpll_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_dpll(&cnf).is_unsatisfiable());
    }
}
