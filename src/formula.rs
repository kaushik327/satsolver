// Defining the types of clauses, literals, and variables
#[derive(Debug)]
pub struct Var {
    pub index: u32,
}

#[derive(Debug)]
pub struct Lit {
    pub var: Var,
    pub negated: bool,
}

#[derive(Debug)]
pub struct Clause {
    pub literals: Vec<Lit>,
}

#[derive(Debug)]
pub struct CNF {
    pub num_vars: u32,
    pub clauses: Vec<Clause>,
}
