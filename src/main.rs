mod formula;
mod parser;
mod solve_cdcl;
mod solve_simple;
mod solver_state;

use std::io;

fn main() {
    let mut reader = io::BufReader::new(io::stdin());
    let cnf = parser::parse_dimacs(&mut reader).unwrap();
    println!("{:#?}", cnf);
    println!("{:#?}", solve_simple::solve_basic(&cnf));
    println!("{:#?}", solve_simple::solve_backtrack(&cnf));
    println!("{:#?}", solve_simple::solve_dpll(&cnf));
    println!("{:#?}", solve_cdcl::solve_cdcl(&cnf));
}
