use crate::formula::*;

use itertools::Itertools;

pub fn pure_literal_eliminate(
    cnf: &CnfFormula,
    assignment: &Assignment,
) -> (CnfFormula, Assignment) {
    let mut seen_positive = vec![false; cnf.num_vars as usize];
    let mut seen_negative = vec![false; cnf.num_vars as usize];
    let mut new_assignment = assignment.clone();
    for clause in &cnf.clauses {
        for lit in &clause.literals {
            if lit.value == Val::True {
                seen_positive[(lit.var.index - 1) as usize] = true;
            } else {
                seen_negative[(lit.var.index - 1) as usize] = true;
            }
        }
    }
    for (i, (pos, neg)) in seen_positive.into_iter().zip(seen_negative).enumerate() {
        if (pos, neg) == (true, false) {
            new_assignment = new_assignment.set(
                &Var {
                    index: i as u32 + 1,
                },
                Val::True,
            );
        } else if (pos, neg) == (false, true) {
            new_assignment = new_assignment.set(
                &Var {
                    index: i as u32 + 1,
                },
                Val::False,
            );
        }
    }
    (apply_assignment(cnf, &new_assignment), new_assignment)
}

pub fn unit_propagate(cnf: &CnfFormula, assignment: &Assignment) -> (CnfFormula, Assignment) {
    // Evaluates assignment on CNF, finds a unit clause, satisfies it with an assignment, and repeats

    fn get_unit_clause(cnf: &CnfFormula) -> Option<&Clause> {
        cnf.clauses.iter().find(|clause| clause.literals.len() == 1)
    }

    let mut new_cnf = cnf.clone();
    let mut new_assignment = assignment.clone();

    loop {
        new_cnf = apply_assignment(&new_cnf, &new_assignment);
        if new_cnf.is_satisfied() || new_cnf.is_falsified() {
            break;
        }
        if let Some(unit_clause) = get_unit_clause(&new_cnf) {
            let lit = &unit_clause.literals[0];
            new_assignment = new_assignment.set(&lit.var, lit.value);
        } else {
            break;
        }
    }
    (new_cnf, new_assignment)
}

pub fn apply_assignment(cnf: &CnfFormula, assignment: &Assignment) -> CnfFormula {
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
    CnfFormula {
        num_vars: cnf.num_vars,
        clauses: new_cnf_clauses,
    }
}

pub fn solve_basic(cnf: &CnfFormula) -> Option<Assignment> {
    // Literally iterate through every possible assignment.
    std::iter::repeat([Some(Val::False), Some(Val::True)])
        .take(cnf.num_vars as usize)
        .multi_cartesian_product()
        .map(|v| Assignment::from_vector(v.to_vec()))
        .find(|assignment| apply_assignment(cnf, assignment).is_satisfied())
}

pub fn solve_backtrack(cnf: &CnfFormula) -> Option<Assignment> {
    // Recursively assign each variable to true or false
    pub fn solve_backtrack_rec(cnf: &CnfFormula, cube: &Assignment) -> Option<Assignment> {
        let new_cnf = apply_assignment(cnf, cube);
        if new_cnf.is_satisfied() {
            Some(cube.fill_unassigned())
        } else if new_cnf.is_falsified() {
            None
        } else {
            cube.get_unassigned_var().and_then(|v| {
                solve_backtrack_rec(&new_cnf, &cube.set(&v, Val::False))
                    .or(solve_backtrack_rec(&new_cnf, &cube.set(&v, Val::True)))
            })
        }
    }
    let blank_assignment = &Assignment::from_vector(vec![None; cnf.num_vars as usize]);
    solve_backtrack_rec(cnf, blank_assignment)
}

pub fn solve_dpll(cnf: &CnfFormula) -> Option<Assignment> {
    // Recursively assign each variable to true or false
    pub fn solve_dpll_rec(cnf: &CnfFormula, cube: &Assignment) -> Option<Assignment> {
        let (new_cnf, new_assignment) = unit_propagate(cnf, cube);
        if new_cnf.is_satisfied() {
            Some(new_assignment.fill_unassigned())
        } else if new_cnf.is_falsified() {
            None
        } else {
            cube.get_unassigned_var().and_then(|v| {
                solve_dpll_rec(&new_cnf, &new_assignment.set(&v, Val::False))
                    .or(solve_dpll_rec(&new_cnf, &new_assignment.set(&v, Val::True)))
            })
        }
    }
    let blank_assignment = &Assignment::from_vector(vec![None; cnf.num_vars as usize]);
    let (ple_cnf, ple_assignment) = pure_literal_eliminate(cnf, blank_assignment);
    solve_dpll_rec(&ple_cnf, &ple_assignment)
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

    #[test]
    fn test_solve_dpll_sat() {
        let text = "\np cnf 5 4\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_dpll(&cnf).is_some_and(
            |a| a.get_unassigned_var().is_none() && apply_assignment(&cnf, &a).is_satisfied()
        ));
    }

    #[test]
    fn test_solve_dpll_unsat() {
        let text = "\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-1 -3 0";
        let cnf = parser::parse_dimacs(&mut io::BufReader::new(text.as_bytes())).unwrap();
        assert!(solve_dpll(&cnf).is_none());
    }
}
