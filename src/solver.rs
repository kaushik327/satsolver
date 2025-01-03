use super::formula::{Assignment, CNF};

use itertools::Itertools;

pub fn solve_basic(cnf: &CNF) -> Option<Assignment> {
    // Literally iterate through every possible assignment.
    std::iter::repeat([false, true])
        .take(cnf.num_vars as usize)
        .multi_cartesian_product()
        .map(|v| Assignment::from_vector(v.to_vec()))
        .find(|assignment| {
            cnf.clauses.iter().all(|clause| {
                clause
                    .literals
                    .iter()
                    .any(|lit| assignment.get(&lit.var) == lit.positive)
            })
        })
}
