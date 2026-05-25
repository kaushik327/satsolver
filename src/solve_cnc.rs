use crate::config::SolverConfig;
use crate::formula::*;
use crate::solve_cdcl::*;
use crate::solver_state::*;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

pub fn solve_cnc(cnf: &CnfFormula, depth: usize, config: &SolverConfig) -> SolverResult {
    fn solve_cnc_rec(
        mut state: SolverState,
        depth: usize,
        config: SolverConfig,
        tx: Arc<mpsc::Sender<SolverResult>>,
    ) -> Vec<thread::JoinHandle<()>> {
        if depth == 0 {
            // Base case: use CDCL solver
            let result = solve_cdcl_from_state(state, &config);
            let _ = tx.send(result);
            return vec![];
        }
        match state.get_status() {
            Status::Satisfied => {
                let _ = tx.send(SolverResult::Satisfiable(
                    state.assignment.fill_unassigned(),
                ));
                vec![]
            }
            Status::Falsified(_) => {
                let _ = tx.send(SolverResult::Unsatisfiable);
                vec![]
            }
            Status::UnassignedDecision(var) => {
                // Branch on the unassigned variable
                let (tstate, fstate) = branch_on_variable(state, var);
                let mut handles = Vec::new();
                for new_state in [tstate, fstate] {
                    let tx_new = Arc::clone(&tx);
                    let config_new = config;
                    let handle_new = thread::spawn(move || {
                        // Recursively create threads and collect their handles
                        let nested_handles =
                            solve_cnc_rec(new_state, depth - 1, config_new, tx_new);
                        // Join all nested threads
                        for handle in nested_handles {
                            handle.join().expect("Thread panicked");
                        }
                    });
                    handles.push(handle_new);
                }
                handles
            }
            Status::UnassignedUnit(lit, clause) => {
                state.assign_unitprop(lit.var, lit.value, clause);
                solve_cnc_rec(state, depth, config, tx)
            }
        }
    }

    // Initialize solver state
    let mut blank_state = SolverState::from_cnf(cnf);
    blank_state.pure_literal_eliminate();
    blank_state.seal_original_clauses();

    // Set up communication channel
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(tx);

    // Start recursive solving process
    let handles = solve_cnc_rec(blank_state, depth, *config, tx.clone());

    // Join all top-level threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Drop the sender to close the channel
    drop(tx);

    // Collect and process results
    rx.iter()
        .find(|result| result.is_satisfiable())
        .unwrap_or(SolverResult::Unsatisfiable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use crate::parser::parse_dimacs_str;

    #[test]
    fn test_solve_cnc_sat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0").unwrap();
        let result = solve_cnc(&cnf, 3, &SolverConfig::default());
        assert!(result.is_satisfiable());
        let assignment = result.into_assignment().unwrap();
        assert!(assignment.get_unassigned_var().is_none() && check_assignment(&cnf, &assignment));
    }

    #[test]
    fn test_solve_cnc_unsat() {
        let cnf = parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0").unwrap();
        assert!(!solve_cnc(&cnf, 3, &SolverConfig::default()).is_satisfiable());
    }
}
