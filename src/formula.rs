// Defining the types of clauses, literals, and variables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        match self {
            Val::True => Val::False,
            Val::False => Val::True,
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
            var: self.var,
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
    pub fn set(&mut self, var: Var, value: Val) {
        self.assignment[var.index - 1] = Some(value);
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
                .map(|v| v.or(Some(Val::False)))
                .collect::<Vec<_>>(),
        }
    }
    pub fn num_vars(&self) -> usize {
        self.assignment.len()
    }
}
