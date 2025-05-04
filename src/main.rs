mod formula;
mod parser;
mod solve_cdcl;
mod solve_cnc;
mod solve_simple;
mod solver_state;

use clap::Parser;
use std::fs::File;
use std::io::{stdin, stdout, BufReader, BufWriter, Read};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    solver: SolverOption,

    /// Depth parameter for CNC solver
    #[arg(short, long, default_value_t = 3)]
    depth: usize,

    /// Input file (use '-' to read from stdin)
    #[arg(short, long)]
    file: String,
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

    let reader: Box<dyn Read> = if args.file == "-" {
        Box::new(stdin().lock())
    } else {
        Box::new(match File::open(&args.file) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                std::process::exit(1);
            }
        })
    };
    let cnf = parser::parse_dimacs(BufReader::new(reader)).unwrap();

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
    parser::output_dimacs(&mut BufWriter::new(stdout()), &answer).unwrap();

    // We don't have proofs of unsatisfiability yet.

    if let Some(assignment) = answer {
        assert!(
            assignment.get_unassigned_var().is_none()
                && solver_state::check_assignment(&cnf, &assignment)
        );
    }
}
