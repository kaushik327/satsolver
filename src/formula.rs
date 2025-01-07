// Defining the types of clauses, literals, and variables
#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Val {
    True,
    False,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lit {
    pub var: Var,
    pub value: Val,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    pub literals: Vec<Lit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CnfFormula {
    pub num_vars: u32,
    pub clauses: Vec<Clause>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    assignment: Vec<Option<Val>>,
}

impl Assignment {
    pub fn from_vector(assignment: Vec<Option<Val>>) -> Self {
        Self { assignment }
    }
    pub fn get(&self, var: &Var) -> Option<Val> {
        self.assignment[var.index as usize - 1]
    }
    pub fn set(&self, var: &Var, value: Val) -> Self {
        let mut new_assignment = self.assignment.clone();
        new_assignment[var.index as usize - 1] = Some(value);
        Self::from_vector(new_assignment)
    }
    pub fn get_unassigned_var(&self) -> Option<Var> {
        self.assignment
            .iter()
            .position(|v| v.is_none())
            .map(|n| Var {
                index: (n + 1) as u32,
            })
    }
    pub fn fill_unassigned(&self) -> Self {
        let new_assignment = self
            .assignment
            .iter()
            .map(|v| v.or(Some(Val::False)))
            .collect::<Vec<_>>();
        Self::from_vector(new_assignment)
    }
}

#[derive(Clone, Debug)]
pub struct SolverState {
    pub num_vars: u32,
    pub clauses: Vec<Clause>,
    pub assignment: Assignment,
}

impl SolverState {
    pub fn from_cnf(cnf: &CnfFormula) -> Self {
        Self {
            num_vars: cnf.num_vars,
            clauses: cnf.clauses.clone(),
            assignment: Assignment::from_vector(vec![None; cnf.num_vars as usize]),
        }
    }

    pub fn is_satisfied(&self) -> bool {
        self.clauses.is_empty()
    }
    pub fn is_falsified(&self) -> bool {
        self.clauses.iter().any(|clause| clause.literals.is_empty())
    }

    pub fn assign(&self, var: &Var, value: Val) -> Self {
        let mut new_cnf_clauses: Vec<Clause> = vec![];
        for clause in &self.clauses {
            if !clause.literals.contains(&Lit {
                var: var.clone(),
                value,
            }) {
                new_cnf_clauses.push(Clause {
                    literals: clause
                        .literals
                        .iter()
                        .filter(|lit| &lit.var != var)
                        .cloned()
                        .collect::<Vec<_>>(),
                })
            }
        }
        Self {
            num_vars: self.num_vars,
            clauses: new_cnf_clauses,
            assignment: self.assignment.set(var, value),
        }
    }
}
