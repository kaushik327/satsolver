mod formula;
mod parser;
mod solve_cdcl;
mod solve_cnc;
mod solve_simple;
mod solver_state;

use std::io;
use std::time::Instant;

type SolverFn = fn(&formula::CnfFormula) -> Option<formula::Assignment>;

fn main() {
    let mut reader = io::BufReader::new(io::stdin());
    let cnf = parser::parse_dimacs(&mut reader).unwrap();
    println!("{:?}", cnf);

    let solvers: &[(&str, SolverFn)] = &[
        ("CNC", |cnf| solve_cnc::solve_cnc(cnf, 1)),
        ("CDCL", solve_cdcl::solve_cdcl),
        ("DPLL", solve_simple::solve_dpll),
        ("Backtracking", solve_simple::solve_backtrack),
        ("Basic", solve_simple::solve_basic),
    ];

    for (solver_name, solver) in solvers {
        println!("Solver: {}", solver_name);
        let start_time = Instant::now();
        let answer = solver(&cnf);
        let duration = start_time.elapsed();
        println!("Solver runtime: {:?}", duration);
        // println!("Answer: {:?}", answer);

        // We don't have proofs of unsatisfiability yet.
        assert!(
            answer.is_none()
                || answer.is_some_and(|a| a.get_unassigned_var().is_none()
                    && solver_state::check_assignment(&cnf, &a))
        );
    }
}
