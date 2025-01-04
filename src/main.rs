mod formula;
mod parser;
mod solver;

use std::io;

fn main() {
    let mut reader = io::BufReader::new(io::stdin());
    let cnf = parser::parse_dimacs(&mut reader).unwrap();
    println!("{:#?}", cnf);
    println!("{:#?}", solver::solve_basic(&cnf));
    println!("{:#?}", solver::solve_backtrack(&cnf));
}
