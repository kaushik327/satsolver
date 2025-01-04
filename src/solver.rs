use super::formula::{Assignment, CNF};

use itertools::Itertools;

// pub fn unit_propagate(cnf: &CNF, assignment: &Assignment) -> (CNF, Assignment) {
//     todo!()
// }

pub fn check_assignment(cnf: &CNF, assignment: &Assignment) -> Option<bool> {
    // Evaluates incomplete assignment on CNF formula and determines if
    // the formula is satisfied, falsified, or neither yet.

    for clause in &cnf.clauses {
        let bools = clause
            .literals
            .iter()
            .map(|lit| assignment.get(&lit.var).map(|b| b == lit.positive))
            .collect::<Vec<_>>();

        if bools.iter().all(|v| v == &Some(false)) {
            return Some(false);
        }
        if !bools.iter().any(|v| v == &Some(true)) {
            return None;
        }
    }
    return Some(true);
}

pub fn solve_basic(cnf: &CNF) -> Option<Assignment> {
    // Literally iterate through every possible assignment.
    std::iter::repeat([Some(false), Some(true)])
        .take(cnf.num_vars as usize)
        .multi_cartesian_product()
        .map(|v| Assignment::from_vector(v.to_vec()))
        .find(|assignment| check_assignment(cnf, assignment).is_some_and(|x| x))
}

pub fn solve_backtrack(cnf: &CNF) -> Option<Assignment> {
    // Recursively assign each variable to true or false
    pub fn solve_backtrack_rec(cnf: &CNF, cube: &Assignment) -> Option<Assignment> {
        match check_assignment(cnf, &cube) {
            Some(true) => Some(cube.fill_unassigned()),
            Some(false) => None,
            None => cube.get_unassigned_var().and_then(|v| {
                solve_backtrack_rec(cnf, &cube.set(&v, false))
                    .or(solve_backtrack_rec(cnf, &cube.set(&v, true)))
            }),
        }
    }
    let blank_assignment = &Assignment::from_vector(vec![None; cnf.num_vars as usize]);
    solve_backtrack_rec(cnf, blank_assignment)
}
