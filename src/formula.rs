use itertools::Itertools;

// Defining the types of clauses, literals, and variables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Var {
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::True => write!(f, "true"),
            Val::False => write!(f, "false"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl std::fmt::Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.value == Val::True {
            write!(f, "x{}", self.var.index)
        } else {
            write!(f, "-x{}", self.var.index)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    pub literals: Vec<Lit>,
}

impl std::fmt::Display for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.literals.iter().join(" V "))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CnfFormula {
    pub num_vars: usize,
    pub clauses: Vec<Clause>,
}

impl std::fmt::Display for CnfFormula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.clauses.iter().join(" ^ "))
    }
}
