// Defining the types of clauses, literals, and variables
#[derive(Debug, Clone)]
pub struct Var {
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Val {
    TRUE,
    FALSE,
}

#[derive(Debug, Clone)]
pub struct Lit {
    pub var: Var,
    pub value: Val,
}

#[derive(Debug, Clone)]
pub struct Clause {
    pub literals: Vec<Lit>,
}

#[derive(Debug, Clone)]
pub struct CNF {
    pub num_vars: u32,
    pub clauses: Vec<Clause>,
}

impl CNF {
    pub fn is_satisfied(&self) -> bool {
        self.clauses.is_empty()
    }
    pub fn is_falsified(&self) -> bool {
        self.clauses.iter().any(|clause| clause.literals.is_empty())
    }
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
            .map(|v| v.or(Some(Val::FALSE)))
            .collect::<Vec<_>>();
        Self::from_vector(new_assignment)
    }
}
