use crate::formula::*;

use itertools::Itertools;

pub fn pure_literal_eliminate(state: &SolverState) -> SolverState {
    let mut seen_positive = vec![false; state.num_vars as usize];
    let mut seen_negative = vec![false; state.num_vars as usize];
    for clause in &state.clauses {
        for lit in &clause.literals {
            if lit.value == Val::True {
                seen_positive[(lit.var.index - 1) as usize] = true;
            } else {
                seen_negative[(lit.var.index - 1) as usize] = true;
            }
        }
    }

    let mut new_state = state.clone();
    for (i, (pos, neg)) in seen_positive.into_iter().zip(seen_negative).enumerate() {
        if (pos, neg) == (true, false) {
            new_state = new_state.assign(
                &Var {
                    index: i as u32 + 1,
                },
                Val::True,
            );
        } else if (pos, neg) == (false, true) {
            new_state = new_state.assign(
                &Var {
                    index: i as u32 + 1,
                },
                Val::False,
            );
        }
    }
    new_state
}

pub fn unit_propagate(state: &SolverState) -> SolverState {
    fn get_unit_literal(state: &SolverState) -> Option<Lit> {
        state
            .clauses
            .iter()
            .find(|clause| clause.literals.len() == 1)
            .map(|clause| clause.literals[0].clone())
    }

    let mut new_state = state.clone();

    while !new_state.is_satisfied() && !new_state.is_falsified() {
        if let Some(lit) = get_unit_literal(&new_state) {
            new_state = new_state.assign(&lit.var, lit.value);
        } else {
            break;
        }
    }
    new_state
}

pub fn check_assignment(cnf: &CnfFormula, assignment: &Assignment) -> bool {
    // Returns true if the assignment fully satisfies the formula, and
    // false if the formula is either falsified or undecided.

    cnf.clauses.iter().all(|clause| {
        clause
            .literals
            .iter()
            .any(|lit| assignment.get(&lit.var).is_some_and(|b| b == lit.value))
    })
}

pub fn solve_basic(cnf: &CnfFormula) -> Option<Assignment> {
    // Literally iterate through every possible assignment.
    std::iter::repeat([Some(Val::False), Some(Val::True)])
        .take(cnf.num_vars as usize)
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
        let ucp_state = unit_propagate(state);
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
    use crate::parser;
    use std::io;

    #[test]
    fn test_solve_basic_sat() {
        let text = "\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_basic(&cnf)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    fn test_solve_basic_unsat() {
        let text = "\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_basic(&cnf).is_none());
    }

    #[test]
    fn test_solve_backtrack_sat() {
        let text = "\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_backtrack(&cnf)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    fn test_solve_backtrack_unsat() {
        let text = "\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_backtrack(&cnf).is_none());
    }

    #[test]
    fn test_solve_dpll_sat() {
        let text = "\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_dpll(&cnf)
            .is_some_and(|a| a.get_unassigned_var().is_none() && check_assignment(&cnf, &a)));
    }

    #[test]
    fn test_solve_dpll_unsat() {
        let text = "\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_dpll(&cnf).is_none());
    }
}
