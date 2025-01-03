// Defining the types of clauses, literals, and variables

use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Hash)]

pub struct Var {
    pub index: u32,
}

#[derive(Debug)]
pub struct Lit {
    pub var: Var,
    pub positive: bool,
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

#[derive(Debug)]
pub struct Assignment {
    pub assignment: HashMap<Var, bool>,
}