mod formula;
mod parser;
mod solver;
mod solverstate;

use std::io;

fn main() {
    let mut reader = io::BufReader::new(io::stdin());
    let cnf = parser::parse_dimacs(&mut reader).unwrap();
    println!("{:#?}", cnf);
    println!("{:#?}", solver::solve_basic(&cnf));
    println!("{:#?}", solver::solve_backtrack(&cnf));
    println!("{:#?}", solver::solve_dpll(&cnf));
    println!("{:#?}", solver::solve_cdcl(&cnf));
}
