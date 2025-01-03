mod formula;
mod parser;

use std::io;

fn main() {
    let mut reader = io::BufReader::new(io::stdin());
    let cnf = parser::parse_dimacs(&mut reader).unwrap();
    println!("{:#?}", cnf);
}
