// Defining the types of clauses, literals, and variables
#[derive(Debug)]
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
    assignment: Vec<bool>,
}

impl Assignment {
    pub fn from_vector(assignment: Vec<bool>) -> Self {
        Self { assignment }
    }
    pub fn get(&self, var: &Var) -> bool {
        self.assignment[var.index as usize - 1]
    }
    pub fn set(&mut self, var: &Var, value: bool) {
        self.assignment[var.index as usize - 1] = value;
    }
}
