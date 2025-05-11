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
