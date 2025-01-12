use crate::formula::*;
use crate::solve_cdcl::*;
use crate::solver_state::*;
use std::sync::mpsc;
use std::thread;

pub fn solve_cnc(cnf: &CnfFormula, depth: usize) -> Option<Assignment> {
    pub fn solve_cnc_rec(state: &CdclState, depth: usize, tx: mpsc::Sender<Option<Assignment>>) {
        let mut ucp_state = state.clone();

        while let Some((trail_elem, ucp_result)) = unit_propagate(&ucp_state.state) {
            ucp_state.state = ucp_result;
            ucp_state.trail.push(trail_elem);
        }

        if ucp_state.state.is_satisfied() {
            let _ = tx.send(Some(ucp_state.state.assignment.fill_unassigned()));
        } else if ucp_state.state.is_falsified() {
            let _ = tx.send(None);
        } else if depth > 0 {
            let unassigned_var = ucp_state.state.assignment.get_unassigned_var().unwrap();

            let true_state = ucp_state.decide_literal_outofplace(&Lit {
                var: unassigned_var.clone(),
                value: Val::True,
            });

            let false_state = ucp_state.decide_literal_outofplace(&Lit {
                var: unassigned_var.clone(),
                value: Val::False,
            });

            let tx1 = tx.clone();
            let tx2 = tx.clone();

            thread::spawn(move || solve_cnc_rec(&true_state, depth - 1, tx1));
            thread::spawn(move || solve_cnc_rec(&false_state, depth - 1, tx2));
        } else {
            thread::spawn(move || {
                let _ = tx.send(solve_cdcl_from_cdcl_state(&mut ucp_state));
            });
        }
    }

    let blank_state = SolverState::from_cnf(cnf);
    let ple_state = pure_literal_eliminate(&blank_state);

    let state_with_trail = CdclState {
        state: ple_state,
        trail: vec![],
    };

    let (tx, rx) = mpsc::channel();
    solve_cnc_rec(&state_with_trail, depth, tx);

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
