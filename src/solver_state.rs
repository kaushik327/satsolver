use itertools::Itertools;

use crate::formula::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SolverClause {
    pub literals: Vec<Lit>,
    pub original: Vec<Lit>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SolverState {
    pub num_vars: u32,
    pub clauses: Vec<SolverClause>,
    pub assignment: Assignment,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TrailReason {
    Decision(SolverState),
    UnitProp(Clause),
}

#[derive(PartialEq, Clone, Debug)]
pub struct TrailElement {
    pub lit: Lit,
    pub reason: TrailReason,
}

impl SolverState {
    pub fn from_cnf(cnf: &CnfFormula) -> Self {
        Self {
            num_vars: cnf.num_vars,
            clauses: cnf
                .clauses
                .iter()
                .map(|clause| SolverClause {
                    literals: clause.literals.clone(),
                    original: clause.literals.clone(),
                })
                .collect_vec(),
            assignment: Assignment::from_vector(vec![None; cnf.num_vars as usize]),
        }
    }

    #[cfg(test)]
    pub fn get_clauses(&self) -> Vec<Clause> {
        self.clauses
            .iter()
            .map(|solver_clause| Clause {
                literals: solver_clause.literals.clone(),
            })
            .collect_vec()
    }

    pub fn is_satisfied(&self) -> bool {
        self.clauses.is_empty()
    }
    pub fn is_falsified(&self) -> bool {
        self.clauses.iter().any(|clause| clause.literals.is_empty())
    }

    pub fn assign(&self, var: &Var, value: Val) -> Self {
        let mut new_cnf_clauses: Vec<SolverClause> = vec![];
        for clause in &self.clauses {
            if !clause.literals.contains(&Lit {
                var: var.clone(),
                value,
            }) {
                new_cnf_clauses.push(SolverClause {
                    literals: clause
                        .literals
                        .iter()
                        .filter(|lit| &lit.var != var)
                        .cloned()
                        .collect::<Vec<_>>(),
                    original: clause.original.clone(),
                });
            }
        }
        Self {
            num_vars: self.num_vars,
            clauses: new_cnf_clauses,
            assignment: self.assignment.set(var, value),
        }
    }
}

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

pub fn unit_propagate(state: &SolverState) -> Option<(TrailElement, SolverState)> {
    // One round of unit propagation
    if state.is_satisfied() || state.is_falsified() {
        None
    } else {
        state
            .clauses
            .iter()
            .find(|clause| clause.literals.len() == 1)
            .map(|clause| {
                let lit = clause.literals[0].clone();
                (
                    TrailElement {
                        lit: lit.clone(),
                        reason: TrailReason::UnitProp(Clause {
                            literals: clause.original.clone(),
                        }),
                    },
                    state.assign(&lit.var, lit.value),
                )
            })
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_dimacs_str;

    #[test]
    fn test_ucp() {
        let pre_ucp = SolverState::from_cnf(
            &parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n-1 -2 0\n1 0\n3 4 0").unwrap(),
        );
        let expected = parse_dimacs_str(b"\np cnf 5 2\n-2 0\n3 4 0").unwrap();

        let Some((trail, post_ucp)) = unit_propagate(&pre_ucp) else {
            panic!();
        };

        assert_eq!(
            trail,
            TrailElement {
                lit: Lit {
                    var: Var { index: 1 },
                    value: Val::True,
                },
                reason: TrailReason::UnitProp(Clause {
                    literals: vec![Lit {
                        var: Var { index: 1 },
                        value: Val::True,
                    }],
                }),
            }
        );

        assert_eq!(post_ucp.get_clauses(), expected.clauses);
    }

    #[test]
    fn test_ple() {
        let pre_ple = SolverState::from_cnf(
            &parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-3 0").unwrap(),
        );
        let expected = parse_dimacs_str(b"\np cnf 5 3\n3 4 0\n3 -4 0\n-3 0").unwrap();

        let post_ple = pure_literal_eliminate(&pre_ple);

        assert_eq!(post_ple.get_clauses(), expected.clauses);
    }
}
