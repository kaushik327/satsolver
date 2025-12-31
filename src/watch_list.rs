use crate::formula::*;
use crate::solver_state::*;

#[derive(Clone, Debug, PartialEq)]
struct Watch {
    clause_idx: usize,
    blocking_lit: Lit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WatchList {
    watches: Vec<Vec<Watch>>,
    cached_status: Option<Status>,
}

// Implementation of the 2 watched literal algorithm
impl WatchList {
    pub fn new(num_vars: usize) -> Self {
        Self {
            watches: vec![vec![]; num_vars * 2],
            cached_status: None,
        }
    }

    fn to_watch_index(lit: Lit) -> usize {
        let var_idx = lit.var.index - 1;
        match lit.value {
            Val::True => var_idx * 2,
            Val::False => var_idx * 2 + 1,
        }
    }

    pub fn add_clause(&mut self, clause_idx: usize, clause: &Clause) {
        if clause.literals.is_empty() {
            unreachable!();
        } else if clause.literals.len() == 1 {
            let lit0 = clause.literals[0];
            self.watches[Self::to_watch_index(lit0)].push(Watch {
                clause_idx,
                blocking_lit: lit0, // blocked by itself, this is handled in the propagation logic
            });
        } else {
            let lit0 = clause.literals[0];
            let lit1 = clause.literals[1];
            self.watches[Self::to_watch_index(lit0)].push(Watch {
                clause_idx,
                blocking_lit: lit1,
            });
            self.watches[Self::to_watch_index(lit1)].push(Watch {
                clause_idx,
                blocking_lit: lit0,
            });
        }
    }

    pub fn get_cached_status(&self) -> Option<&Status> {
        self.cached_status.as_ref()
    }

    pub fn clear_status(&mut self) {
        self.cached_status = None;
    }

    pub fn update_for_assignment(
        &mut self,
        assigned_lit: Lit,
        assignment: &Assignment,
        clauses: &[Clause],
    ) {
        // clear cached status to be re-assigned later
        self.cached_status = None;

        let neg_lit = assigned_lit.not();

        // look for a clause containing neg_lit
        let mut i = 0;
        while i < self.watches[Self::to_watch_index(neg_lit)].len() {
            let watch = self.watches[Self::to_watch_index(neg_lit)][i].clone();

            if assignment.get(&watch.blocking_lit) == Some(true) {
                // this clause is satisfied, so we skip it
                i += 1;
                continue;
            }

            let clause = &clauses[watch.clause_idx];

            // looking for a new literal to watch; can be either unassigned or assigned true
            let mut found_new_watch = false;
            let mut new_watch_lit = neg_lit;
            for &lit in &clause.literals {
                if lit != neg_lit
                    && lit != watch.blocking_lit
                    && assignment.get(&lit) != Some(false)
                {
                    new_watch_lit = lit;
                    found_new_watch = true;
                    break;
                }
            }

            if found_new_watch {
                let removed_watch = self.watches[Self::to_watch_index(neg_lit)].swap_remove(i);
                self.watches[Self::to_watch_index(new_watch_lit)].push(Watch {
                    clause_idx: removed_watch.clause_idx,
                    blocking_lit: watch.blocking_lit,
                });
                // don't increment i, because of how swap_remove works
            } else {
                // couldn't find a new unassigned variable - clause is unit or falsified
                // cache the status eagerly
                if assignment.get(&watch.blocking_lit) == Some(false) {
                    // falsified
                    self.cached_status = Some(Status::Falsified(clause.clone()));
                    return;
                } else if assignment.get(&watch.blocking_lit).is_none() {
                    // unit clause
                    if self.cached_status.is_none() {
                        self.cached_status =
                            Some(Status::UnassignedUnit(watch.blocking_lit, clause.clone()));
                    }
                }
                i += 1;
            }
        }
    }
}
