use crate::formula::*;
use crate::solve_cdcl::*;
use crate::solver_state::*;

pub fn solve_cnc(cnf: &CnfFormula, depth: usize) -> Option<Assignment> {

    pub fn solve_cnc_rec(state: &CdclState, depth: usize) -> Option<Assignment> {
        let mut ucp_state = state.clone();

        while let Some((trail_elem, ucp_result)) = unit_propagate(&ucp_state.state) {
            ucp_state.state = ucp_result;
            ucp_state.trail.push(trail_elem);
        }
        if ucp_state.state.is_satisfied() {
            Some(ucp_state.state.assignment.fill_unassigned())
        } else if ucp_state.state.is_falsified() {
            None
        } else if depth > 0 {
            let unassigned_var = ucp_state.state.assignment.get_unassigned_var().unwrap();

            let snapshot = ucp_state.state;

            let true_state = CdclState {
                trail: {
                    let mut v = ucp_state.trail.clone();
                    v.push(TrailElement {
                        lit: Lit {
                            var: unassigned_var.clone(),
                            value: Val::True,
                        },
                        reason: TrailReason::Decision(snapshot.clone()),
                    });
                    v
                },
                state: snapshot.assign(&unassigned_var.clone(), Val::True),
            };

            let false_state = CdclState {
                trail: {
                    let mut v = ucp_state.trail.clone();
                    v.push(TrailElement {
                        lit: Lit {
                            var: unassigned_var.clone(),
                            value: Val::False,
                        },
                        reason: TrailReason::Decision(snapshot.clone()),
                    });
                    v
                },
                state: snapshot.assign(&unassigned_var.clone(), Val::False),
            };

            solve_cnc_rec(&true_state, depth - 1).or(solve_cnc_rec(&false_state, depth - 1))
        } else {
            solve_cdcl_from_cdcl_state(&mut ucp_state)
        }
    }

    let blank_state = SolverState::from_cnf(cnf);
    let ple_state = pure_literal_eliminate(&blank_state);

    let state_with_trail = CdclState {
        state: ple_state,
        trail: vec![],
    };

    solve_cnc_rec(&state_with_trail, depth)
}
