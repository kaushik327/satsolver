use itertools::Itertools;

use crate::formula::*;

impl Clause {
    pub fn is_satisfied(&self, a: &Assignment) -> bool {
        self.literals.iter().any(|lit| a.get(lit) == Some(true))
    }

    pub fn is_falsified(&self, a: &Assignment) -> bool {
        self.literals.iter().all(|lit| a.get(lit) == Some(false))
    }

    #[cfg(test)]
    pub fn get_equivalent_clause(&self, a: &Assignment) -> Option<Vec<Lit>> {
        if self.is_satisfied(a) {
            return None;
        }

        Some(
            self.literals
                .iter()
                .filter(|lit| a.get(lit).is_none())
                .map(|lit| lit.clone())
                .collect_vec(),
        )
    }

    pub fn get_unit_literal(&self, a: &Assignment) -> Option<Lit> {
        // TODO: this code is wasteful
        let assignments = self.literals.iter().map(|lit| a.get(lit)).collect_vec();
        if assignments.contains(&Some(true)) {
            return None;
        }
        let num_unassigned_vars = assignments.iter().filter(|b| b.is_none()).count();
        if num_unassigned_vars != 1 {
            return None;
        }
        self.literals
            .iter()
            .zip(assignments)
            .find(|(_, b)| b.is_none())
            .map(|(lit, _)| lit.clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SolverState {
    pub formula: CnfFormula,
    pub assignment: Assignment,
    pub trail: Vec<TrailElement>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TrailReason {
    Decision(Assignment),
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
            formula: cnf.clone(),
            assignment: Assignment::from_vector(vec![None; cnf.num_vars]),
            trail: vec![],
        }
    }

    #[cfg(test)]
    pub fn get_equivalent_clauses(&self) -> Vec<Clause> {
        self.formula
            .clauses
            .iter()
            .filter_map(|solver_clause| solver_clause.get_equivalent_clause(&self.assignment))
            .map(|lits| Clause { literals: lits })
            .collect_vec()
    }

    pub fn is_satisfied(&self) -> bool {
        self.formula
            .clauses
            .iter()
            .all(|clause| clause.is_satisfied(&self.assignment))
    }
    pub fn is_falsified(&self) -> bool {
        self.formula
            .clauses
            .iter()
            .any(|clause| clause.is_falsified(&self.assignment))
    }

    pub fn decide(&self, var: &Var, value: Val) -> Self {
        let mut modified_trail = self.trail.clone();
        modified_trail.push(TrailElement {
            lit: Lit {
                var: var.clone(),
                value,
            },
            reason: TrailReason::Decision(self.assignment.clone()),
        });
        Self {
            formula: self.formula.clone(),
            assignment: self.assignment.set(var, value),
            trail: modified_trail,
        }
    }

    pub fn assign_unitprop(&self, var: &Var, value: Val, clause: &Clause) -> Self {
        let mut modified_trail = self.trail.clone();
        modified_trail.push(TrailElement {
            lit: Lit {
                var: var.clone(),
                value,
            },
            reason: TrailReason::UnitProp(clause.clone()),
        });
        Self {
            formula: self.formula.clone(),
            assignment: self.assignment.set(var, value),
            trail: modified_trail,
        }
    }

    pub fn get_unassigned_lit(&self) -> Option<Lit> {
        self.formula
            .clauses
            .iter()
            .flat_map(|clause| clause.literals.clone())
            .find(|lit| self.assignment.get(lit).is_none())
    }

    pub fn learn_clause(&mut self, lits: Vec<Lit>) {
        self.formula.clauses.push(Clause { literals: lits });
    }

    pub fn get_last_decision_index(&self) -> Option<usize> {
        self.trail
            .iter()
            .enumerate()
            .filter(|(_, x)| matches!(x.reason, TrailReason::Decision(_)))
            .last()
            .map(|(x, _)| x)
    }
}

pub fn pure_literal_eliminate(state: &SolverState) -> SolverState {
    let mut seen_positive = vec![false; state.formula.num_vars];
    let mut seen_negative = vec![false; state.formula.num_vars];
    for clause in &state.formula.clauses {
        for lit in &clause.literals {
            if lit.value == Val::True {
                seen_positive[lit.var.index - 1] = true;
            } else {
                seen_negative[lit.var.index - 1] = true;
            }
        }
    }

    let mut new_state = state.clone();
    for (i, (pos, neg)) in seen_positive.into_iter().zip(seen_negative).enumerate() {
        if (pos, neg) == (true, false) {
            new_state = new_state.decide(&Var { index: i + 1 }, Val::True);
        } else if (pos, neg) == (false, true) {
            new_state = new_state.decide(&Var { index: i + 1 }, Val::False);
        }
    }
    new_state
}

pub fn unit_propagate(state: &SolverState) -> Option<SolverState> {
    // One round of unit propagation
    if state.is_satisfied() || state.is_falsified() {
        None
    } else {
        state
            .formula
            .clauses
            .iter()
            .map(|clause| (clause, clause.get_unit_literal(&state.assignment)))
            .find(|(_, maybe_lit)| maybe_lit.is_some())
            .map(|(clause, maybe_lit)| {
                let Some(lit) = maybe_lit else { panic!() };
                state.assign_unitprop(&lit.var, lit.value, clause)
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
            .any(|lit| assignment.get(lit) == Some(true))
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

        let Some(post_ucp) = unit_propagate(&pre_ucp) else {
            panic!();
        };

        assert_eq!(
            post_ucp.trail[0],
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

        assert_eq!(post_ucp.get_equivalent_clauses(), expected.clauses);
    }

    #[test]
    fn test_ple() {
        let pre_ple = SolverState::from_cnf(
            &parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-3 0").unwrap(),
        );
        let expected = parse_dimacs_str(b"\np cnf 5 3\n3 4 0\n3 -4 0\n-3 0").unwrap();

        let post_ple = pure_literal_eliminate(&pre_ple);

        assert_eq!(post_ple.get_equivalent_clauses(), expected.clauses);
    }
}
