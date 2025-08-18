use crate::formula::*;
use crate::solve_cdcl::*;
use crate::solver_state::*;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

#[allow(dead_code)]
#[deprecated]
pub fn solve_cnc(cnf: &CnfFormula, depth: usize) -> Option<Assignment> {
    // Create a recursive function that returns a vector of thread handles
    fn solve_cnc_rec(
        mut state: SolverState,
        depth: usize,
        tx: Arc<mpsc::Sender<Option<Assignment>>>,
    ) -> Vec<thread::JoinHandle<()>> {
        if depth == 0 {
            // Base case: use CDCL solver
            let result = solve_cdcl_from_state(state);
            let _ = tx.send(result);
            return vec![];
        }
        state.unit_propagate();
        match state.get_status() {
            Status::Satisfied => {
                // Found a satisfying assignment
                let _ = tx.send(Some(state.assignment.fill_unassigned()));
                vec![]
            }
            Status::Falsified(_) => {
                // This branch is unsatisfiable
                let _ = tx.send(None);
                vec![]
            }
            Status::UnassignedDecision(lit) => {
                // Branch on the unassigned literal
                let (tstate, fstate) = branch_on_variable(state, lit.var);
                let mut handles = Vec::new();
                for new_state in [tstate, fstate] {
                    let tx_new = Arc::clone(&tx);
                    let handle_new = thread::spawn(move || {
                        // Recursively create threads and collect their handles
                        let nested_handles = solve_cnc_rec(new_state, depth - 1, tx_new);
                        // Join all nested threads
                        for handle in nested_handles {
                            handle.join().expect("Thread panicked");
                        }
                    });
                    handles.push(handle_new);
                }
                handles
            }
            Status::UnassignedUnit(_, _) => unreachable!(),
        }
    }

    // Initialize solver state
    let mut blank_state = SolverState::from_cnf(cnf);
    blank_state.pure_literal_eliminate();

    // Set up communication channel
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(tx);

    // Start recursive solving process
    let handles = solve_cnc_rec(blank_state, depth, tx.clone());

    // Join all top-level threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Drop the sender to close the channel
    drop(tx);

    // Collect and process results
    rx.iter().flatten().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dimacs_str;

    #[test]
    #[allow(deprecated)]
    fn test_solve_cnc_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        assert!(solve_cnc(&cnf, 3)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    #[allow(deprecated)]
    fn test_solve_cnc_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(solve_cnc(&cnf, 3).is_none());
    }
}
