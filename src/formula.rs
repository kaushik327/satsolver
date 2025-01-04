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

#[derive(Debug, Clone)]
pub struct Assignment {
    assignment: Vec<Option<bool>>,
}

impl Assignment {
    pub fn from_vector(assignment: Vec<Option<bool>>) -> Self {
        Self { assignment }
    }
    pub fn get(&self, var: &Var) -> Option<bool> {
        self.assignment[var.index as usize - 1]
    }
    pub fn set(&self, var: &Var, value: bool) -> Self {
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
            .map(|v| v.or(Some(false)))
            .collect::<Vec<_>>();
        Self::from_vector(new_assignment)
    }
}
