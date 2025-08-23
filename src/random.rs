use crate::formula::*;
use rand::prelude::*;
use std::collections::HashSet;

pub fn generate_random_cnf(n: usize, k: usize, l: usize) -> CnfFormula {
    // Generates a random k-SAT CNF formula with n variables and l clauses

    if k > n {
        panic!("Cannot generate clauses with {k} variables when only {n} variables exist");
    }

    let mut rng = rand::rng();
    let mut clauses = Vec::with_capacity(l);

    // Generate L random clauses
    for _ in 0..l {
        clauses.push(generate_random_clause(n, k, &mut rng));
    }

    CnfFormula {
        num_vars: n,
        clauses,
    }
}

fn generate_random_clause(n: usize, k: usize, rng: &mut ThreadRng) -> Clause {
    // Choose k unique variables from 1..=n
    let mut chosen_vars = HashSet::new();

    while chosen_vars.len() < k {
        chosen_vars.insert(rng.random_range(1..=n));
    }

    // Convert to literals with random negation
    let literals: Vec<Lit> = chosen_vars
        .into_iter()
        .map(|var_index| Lit {
            var: Var { index: var_index },
            value: if rng.random_bool(0.5) {
                Val::True
            } else {
                Val::False
            },
        })
        .collect();
    Clause { literals }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_cnf_basic() {
        let formula = generate_random_cnf(5, 3, 10);

        assert_eq!(formula.num_vars, 5);
        assert_eq!(formula.clauses.len(), 10);

        // Check that each clause has exactly 3 literals
        for clause in &formula.clauses {
            assert_eq!(clause.literals.len(), 3);
        }
    }

    #[test]
    fn test_clause_has_unique_variables() {
        let formula = generate_random_cnf(10, 4, 5);

        for clause in &formula.clauses {
            let mut var_indices = HashSet::new();
            for lit in &clause.literals {
                assert!(
                    var_indices.insert(lit.var.index),
                    "Duplicate variable in clause"
                );
                assert!(
                    lit.var.index >= 1 && lit.var.index <= 10,
                    "Variable index out of range"
                );
            }
        }
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate clauses with 6 variables when only 5 variables exist"
    )]
    fn test_k_greater_than_n_panics() {
        generate_random_cnf(5, 6, 1);
    }

    #[test]
    fn test_edge_cases() {
        // Single variable, single clause
        let formula = generate_random_cnf(1, 1, 1);
        assert_eq!(formula.num_vars, 1);
        assert_eq!(formula.clauses.len(), 1);
        assert_eq!(formula.clauses[0].literals.len(), 1);
        assert_eq!(formula.clauses[0].literals[0].var.index, 1);

        // All variables in each clause
        let formula = generate_random_cnf(3, 3, 2);
        for clause in &formula.clauses {
            assert_eq!(clause.literals.len(), 3);
            let var_indices: HashSet<_> = clause.literals.iter().map(|lit| lit.var.index).collect();
            assert_eq!(var_indices, [1, 2, 3].iter().cloned().collect());
        }
    }
}
