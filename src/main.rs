mod formula;
mod parser;
mod solve_cdcl;
mod solve_cnc;
mod solve_simple;
mod solver_state;

use clap::Parser;
use std::io;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    solver: SolverOption,

    /// Depth parameter for CNC solver
    #[arg(short, long, default_value_t = 3)]
    depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum SolverOption {
    Cnc,
    Cdcl,
    Dpll,
    Backtrack,
    Basic,
}

fn main() {
    let args = Args::parse();

    let mut reader = io::BufReader::new(io::stdin());
    let cnf = parser::parse_dimacs(&mut reader).unwrap();

    let start_time = Instant::now();
    let answer: Option<formula::Assignment> = match args.solver {
        SolverOption::Cnc => solve_cnc::solve_cnc(&cnf, args.depth),
        SolverOption::Cdcl => solve_cdcl::solve_cdcl(&cnf),
        SolverOption::Dpll => solve_simple::solve_dpll(&cnf),
        SolverOption::Backtrack => solve_simple::solve_backtrack(&cnf),
        SolverOption::Basic => solve_simple::solve_basic(&cnf),
    };
    let duration = start_time.elapsed();

    println!("c runtime: {:?}", duration);
    parser::output_dimacs(&mut io::BufWriter::new(io::stdout()), &answer, cnf.num_vars).unwrap();

    // We don't have proofs of unsatisfiability yet.

    if let Some(assignment) = answer {
        assert!(
            assignment.get_unassigned_var().is_none()
                && solver_state::check_assignment(&cnf, &assignment)
        );
    }
}
