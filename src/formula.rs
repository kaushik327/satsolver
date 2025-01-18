// Defining the types of clauses, literals, and variables
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Var {
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Val {
    True,
    False,
}

impl Val {
    pub fn not(&self) -> Self {
        if self == &Val::True {
            Val::False
        } else {
            Val::True
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lit {
    pub var: Var,
    pub value: Val,
}

impl Lit {
    pub fn not(&self) -> Self {
        Lit {
            var: self.var.clone(),
            value: self.value.not(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    pub literals: Vec<Lit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CnfFormula {
    pub num_vars: usize,
    pub clauses: Vec<Clause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    assignment: Vec<Option<Val>>,
}

impl Assignment {
    pub fn from_vector(assignment: Vec<Option<Val>>) -> Self {
        Self { assignment }
    }
    pub fn get(&self, lit: &Lit) -> Option<bool> {
        self.assignment[lit.var.index - 1].map(|v| v == lit.value)
    }
    pub fn set(&self, var: &Var, value: Val) -> Self {
        let mut new_assignment = self.assignment.clone();
        new_assignment[var.index - 1] = Some(value);
        Self::from_vector(new_assignment)
    }
    pub fn get_unassigned_var(&self) -> Option<Var> {
        self.assignment
            .iter()
            .position(|v| v.is_none())
            .map(|n| Var { index: n + 1 })
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
