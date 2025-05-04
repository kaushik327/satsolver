use crate::formula::*;
use crate::solve_cdcl::*;
use crate::solver_state::*;
use std::sync::mpsc;
use std::thread;

pub fn solve_cnc(cnf: &CnfFormula, depth: usize) -> Option<Assignment> {
    pub fn solve_cnc_rec(
        mut state: SolverState,
        depth: usize,
        tx: mpsc::Sender<Option<Assignment>>,
    ) {
        state.unit_propagate();

        match state.get_status() {
            Status::Satisfied => {
                let mut assignment = state.assignment;
                assignment.fill_unassigned();
                let _ = tx.send(Some(assignment));
            }
            Status::Falsified => {
                let _ = tx.send(None);
            }
            Status::Unassigned(lit) if depth > 0 => {
                let (tstate, fstate) = branch_on_variable(state, lit.var);
                // Spawn new thread for one branch
                let tx1 = tx.clone();
                thread::spawn(move || solve_cnc_rec(tstate, depth - 1, tx1));
                // Continue with current thread for the other branch
                solve_cnc_rec(fstate, depth - 1, tx);
            }
            _ => {
                let _ = tx.send(solve_cdcl_from_state(state));
            }
        }
    }

    let mut blank_state = SolverState::from_cnf(cnf);
    blank_state.pure_literal_eliminate();

    let (tx, rx) = mpsc::channel();
    solve_cnc_rec(blank_state, depth, tx);

    rx.recv().into_iter().find(|x| x.is_some()).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dimacs_str;

    #[test]
    fn test_solve_cnc_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        assert!(solve_cnc(&cnf, 3)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    fn test_solve_cnc_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_cnc(&cnf, 3).is_none());
    }
}
