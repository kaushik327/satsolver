use itertools::Itertools;

use crate::formula::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Record {
    value: Val,
    decision_level: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    assignment: Vec<Option<Record>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SolverResult {
    Satisfiable(Assignment),
    Unsatisfiable,
}

impl SolverResult {
    pub fn is_satisfiable(&self) -> bool {
        matches!(self, Self::Satisfiable(_))
    }

    pub fn is_unsatisfiable(&self) -> bool {
        matches!(self, Self::Unsatisfiable)
    }

    pub fn assignment(&self) -> Option<&Assignment> {
        match self {
            Self::Satisfiable(assignment) => Some(assignment),
            Self::Unsatisfiable => None,
        }
    }

    pub fn into_assignment(self) -> Option<Assignment> {
        match self {
            Self::Satisfiable(assignment) => Some(assignment),
            Self::Unsatisfiable => None,
        }
    }
}

impl Assignment {
    pub fn empty(num_vars: usize) -> Self {
        Self {
            assignment: vec![None; num_vars],
        }
    }
    pub fn get(&self, lit: &Lit) -> Option<bool> {
        self.assignment[lit.var.index - 1].map(|v| v.value == lit.value)
    }
    pub fn get_decision_level(&self, lit: &Lit) -> Option<u32> {
        self.assignment[lit.var.index - 1].map(|v| v.decision_level)
    }
    pub fn set(&mut self, var: Var, value: Val, decision_level: u32) {
        self.assignment[var.index - 1] = Some(Record {
            value,
            decision_level,
        });
    }
    pub fn get_unassigned_var(&self) -> Option<Var> {
        self.assignment
            .iter()
            .position(|v| v.is_none())
            .map(|n| Var { index: n + 1 })
    }
    pub fn fill_unassigned(self) -> Self {
        Self {
            assignment: self
                .assignment
                .iter()
                .map(|v| {
                    v.or(Some(Record {
                        value: Val::False,
                        decision_level: u32::MAX,
                    }))
                })
                .collect::<Vec<_>>(),
        }
    }
    pub fn num_vars(&self) -> usize {
        self.assignment.len()
    }
    pub fn every_possible(num_vars: usize) -> impl Iterator<Item = Self> {
        std::iter::repeat_n(
            [
                Some(Record {
                    value: Val::False,
                    decision_level: 0,
                }),
                Some(Record {
                    value: Val::True,
                    decision_level: 0,
                }),
            ],
            num_vars,
        )
        .multi_cartesian_product()
        .map(|v| Self {
            assignment: v.to_vec(),
        })
    }
}

impl std::fmt::Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.assignment
                .iter()
                .enumerate()
                .filter_map(|(i, v)| {
                    v.map(|v| format!("x{}={}(d{})", i + 1, v.value, v.decision_level))
                })
                .join(", ")
        )
    }
}

impl Clause {
    #[cfg(test)]
    pub fn get_equivalent_clause(&self, a: &Assignment) -> Option<Vec<Lit>> {
        if self.literals.iter().any(|lit| a.get(lit) == Some(true)) {
            return None;
        }

        Some(
            self.literals
                .iter()
                .filter(|lit| a.get(lit).is_none())
                .cloned()
                .collect_vec(),
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SolverState {
    pub formula: CnfFormula,
    pub assignment: Assignment,
    pub trail: Vec<TrailElement>,
    pub decision_level: u32,
}

impl std::fmt::Display for SolverState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Assignment: {}\nFormula: {}",
            self.assignment, self.formula
        )
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum TrailReason {
    // At a decision, we snapshot the previous assignment so we can backjump to it if needed.
    Decision(Assignment),
    // At unit propagation we save the clause that was used to infer the unit literal.
    UnitProp(Clause),
}

#[derive(PartialEq, Clone, Debug)]
pub struct TrailElement {
    pub lit: Lit,
    pub reason: TrailReason,
}

impl std::fmt::Display for TrailElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
            TrailReason::Decision(_) => write!(f, "{}(D)", self.lit),
            TrailReason::UnitProp(_) => write!(f, "{}(U)", self.lit),
        }
    }
}

pub enum Status {
    Satisfied,
    Falsified(Clause),
    UnassignedDecision(Lit),
    UnassignedUnit(Lit, Clause),
}

impl SolverState {
    pub fn from_cnf(cnf: &CnfFormula) -> Self {
        Self {
            formula: cnf.clone(),
            assignment: Assignment::empty(cnf.num_vars),
            trail: vec![],
            decision_level: 0,
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

    // TODO: avoid repeated linear scans; get_status takes the most time by far in profiles
    pub fn get_status(&self) -> Status {
        let mut unassigned = None;
        let mut unassigned_unit = None;

        'outer: for clause in self.formula.clauses.iter() {
            let mut unassigned_in_clause = None;
            let mut unassigned_count = 0;

            for lit in clause.literals.iter() {
                match self.assignment.get(lit) {
                    Some(false) => continue,
                    Some(true) => continue 'outer,
                    None => {
                        unassigned_in_clause = Some(*lit);
                        unassigned_count += 1;
                    }
                }
            }

            if let Some(lit) = unassigned_in_clause {
                if unassigned_count == 1 {
                    // This is a unit clause - prioritize it
                    unassigned_unit = Some((lit, clause.clone()));
                }
                unassigned = Some(lit);
            } else {
                return Status::Falsified(clause.clone());
            }
        }
        if let Some((unit_lit, unit_clause)) = unassigned_unit {
            return Status::UnassignedUnit(unit_lit, unit_clause);
        }
        if let Some(lit) = unassigned {
            return Status::UnassignedDecision(lit);
        }
        Status::Satisfied
    }

    pub fn decide(&mut self, var: Var, value: Val) {
        self.decision_level += 1;
        // TODO: repeatedly cloning the assignment for this reason is inefficient
        self.trail.push(TrailElement {
            lit: Lit { var, value },
            reason: TrailReason::Decision(self.assignment.clone()),
        });
        self.assignment.set(var, value, self.decision_level);
    }

    pub fn assign_unitprop(&mut self, var: Var, value: Val, clause: Clause) {
        self.trail.push(TrailElement {
            lit: Lit { var, value },
            reason: TrailReason::UnitProp(clause),
        });
        self.assignment.set(var, value, self.decision_level);
    }

    pub fn learn_clause(&mut self, clause: Clause) {
        self.formula.clauses.push(clause);
    }

    pub fn backjump_to_decision_level(&mut self, decision_level: u32) {
        let (cut_idx, snapshot) = self
            .trail
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(idx, elem)| match &elem.reason {
                TrailReason::Decision(snapshot) => Some((idx, snapshot.clone())),
                _ => None,
            })
            .nth(decision_level as usize)
            .unwrap_or((0, Assignment::empty(self.formula.num_vars)));

        self.trail.truncate(cut_idx);
        self.decision_level = decision_level;
        self.assignment = snapshot;
    }

    pub fn pure_literal_eliminate(&mut self) {
        let mut seen_positive = vec![false; self.formula.num_vars];
        let mut seen_negative = vec![false; self.formula.num_vars];
        for clause in &self.formula.clauses {
            for lit in &clause.literals {
                if lit.value == Val::True {
                    seen_positive[lit.var.index - 1] = true;
                } else {
                    seen_negative[lit.var.index - 1] = true;
                }
            }
        }

        for (i, (pos, neg)) in seen_positive.into_iter().zip(seen_negative).enumerate() {
            if (pos, neg) == (true, false) {
                self.decide(Var { index: i + 1 }, Val::True);
            } else if (pos, neg) == (false, true) {
                self.decide(Var { index: i + 1 }, Val::False);
            }
        }
    }
}

pub fn branch_on_variable(state: SolverState, var: Var) -> (SolverState, SolverState) {
    let mut tstate = state.clone();
    tstate.decide(var, Val::True);
    let mut fstate = state;
    fstate.decide(var, Val::False);
    (tstate, fstate)
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
        let mut ucp = SolverState::from_cnf(
            &parse_dimacs_str(b"\np cnf 5 4\n1 2 0\n-1 -2 0\n1 0\n3 4 0").unwrap(),
        );
        let expected = parse_dimacs_str(b"\np cnf 5 1\n3 4 0").unwrap();

        while let Status::UnassignedUnit(lit, clause) = ucp.get_status() {
            ucp.assign_unitprop(lit.var, lit.value, clause);
        }

        assert_eq!(
            ucp.trail,
            [
                TrailElement {
                    lit: Lit {
                        var: Var { index: 1 },
                        value: Val::True
                    },
                    reason: TrailReason::UnitProp(Clause {
                        literals: vec![Lit {
                            var: Var { index: 1 },
                            value: Val::True
                        }]
                    })
                },
                TrailElement {
                    lit: Lit {
                        var: Var { index: 2 },
                        value: Val::False
                    },
                    reason: TrailReason::UnitProp(Clause {
                        literals: vec![
                            Lit {
                                var: Var { index: 1 },
                                value: Val::False
                            },
                            Lit {
                                var: Var { index: 2 },
                                value: Val::False
                            }
                        ]
                    })
                }
            ]
        );

        assert_eq!(ucp.get_equivalent_clauses(), expected.clauses);
    }

    #[test]
    fn test_ple() {
        let mut ple = SolverState::from_cnf(
            &parse_dimacs_str(b"\np cnf 5 5\n1 2 0\n1 -2 0\n3 4 0\n3 -4 0\n-3 0").unwrap(),
        );
        let expected = parse_dimacs_str(b"\np cnf 5 3\n3 4 0\n3 -4 0\n-3 0").unwrap();
        ple.pure_literal_eliminate();
        assert_eq!(ple.get_equivalent_clauses(), expected.clauses);
    }
}
