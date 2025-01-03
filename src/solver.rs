use super::formula::{Var, CNF, Assignment};
use std::collections::HashMap;

use itertools::Itertools;

pub fn solve_basic(cnf: &CNF) -> Option<Assignment> {
    // Literally iterate through every possible assignment.

    let mut count = 0;

    let res = std::iter::repeat([false, true])
        .take(cnf.num_vars as usize)
        .multi_cartesian_product()
        .find(|assignment| {
            count += 1;
            cnf.clauses
                .iter()
                .all(|clause| clause.literals.iter().any(|lit| assignment[lit.var.index as usize - 1] == lit.positive))
        });
    
    println!("Checked {} assignments", count);
    
    res.map(|assignment| {
        let mut res = HashMap::new();
        for (var, value) in assignment.iter().enumerate() {
            res.insert(Var { index: (var + 1) as u32 }, *value);
        }
        Assignment { assignment: res }
    })
}