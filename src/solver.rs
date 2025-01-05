use crate::formula::Clause;

use super::formula::{Assignment, Lit, Val, CNF};

use itertools::Itertools;

// pub fn unit_propagate(cnf: &CNF, assignment: &Assignment) -> (CNF, Assignment) {
//     todo!()
// }

pub fn apply_assignment(cnf: &CNF, assignment: &Assignment) -> CNF {
    // Evaluates incomplete assignment on CNF formula and removes satisfied
    // clauses and false literals.

    let mut new_cnf_clauses: Vec<Clause> = vec![];

    for clause in &cnf.clauses {
        let mut clause_satisfied = false;
        let mut curr_clause: Vec<Lit> = vec![];

        for lit in &clause.literals {
            let lit_satisfied = assignment.get(&lit.var).map(|b| b == lit.value);
            if matches!(lit_satisfied, Some(true)) {
                clause_satisfied = true;
                break;
            } else if lit_satisfied.is_none() {
                curr_clause.push(lit.clone());
            }
        }
        if !clause_satisfied {
            new_cnf_clauses.push(Clause {
                literals: curr_clause,
            });
        }
    }
    CNF {
        num_vars: cnf.num_vars,
        clauses: new_cnf_clauses,
    }
}

pub fn solve_basic(cnf: &CNF) -> Option<Assignment> {
    // Literally iterate through every possible assignment.
    std::iter::repeat([Some(Val::FALSE), Some(Val::TRUE)])
        .take(cnf.num_vars as usize)
        .multi_cartesian_product()
        .map(|v| Assignment::from_vector(v.to_vec()))
        .find(|assignment| apply_assignment(cnf, assignment).is_satisfied())
}

pub fn solve_backtrack(cnf: &CNF) -> Option<Assignment> {
    // Recursively assign each variable to true or false
    pub fn solve_backtrack_rec(cnf: &CNF, cube: &Assignment) -> Option<Assignment> {
        let new_cnf = apply_assignment(cnf, &cube);
        if new_cnf.is_satisfied() {
            Some(cube.fill_unassigned())
        } else if new_cnf.is_falsified() {
            None
        } else {
            cube.get_unassigned_var().and_then(|v| {
                solve_backtrack_rec(&new_cnf, &cube.set(&v, Val::FALSE))
                    .or(solve_backtrack_rec(&new_cnf, &cube.set(&v, Val::TRUE)))
            })
        }
    }
    let blank_assignment = &Assignment::from_vector(vec![None; cnf.num_vars as usize]);
    solve_backtrack_rec(cnf, blank_assignment)
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;
    use std::io;

    #[test]
    fn test_solve_basic_sat() {
        let text = "\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_basic(&cnf).is_some_and(
            |a| a.get_unassigned_var().is_none() && apply_assignment(&cnf, &a).is_satisfied()
        ));
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
        assert!(solve_backtrack(&cnf).is_some_and(
            |a| a.get_unassigned_var().is_none() && apply_assignment(&cnf, &a).is_satisfied()
        ));
    }

    #[test]
    fn test_solve_backtrack_unsat() {
        let text = "\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_backtrack(&cnf).is_none());
    }
}
