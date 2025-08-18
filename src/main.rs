mod formula;
mod parser;
mod solve_cdcl;
mod solve_cdcl_first_uip;
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
    #[arg(short, long, default_value = "cdcl-first-uip")]
    solver: SolverOption,

    /// Depth parameter for CNC solver
    #[arg(short, long, default_value_t = 3)]
    depth: usize,

    file: Vec<String>,

    #[arg(long)]
    dimacs_output: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum SolverOption {
    Cdcl,
    CdclFirstUip,
    Dpll,
    Backtrack,
    Basic,
}

fn main() {
    let args = Args::parse();

    for file in args.file {
        let reader: Box<dyn Read> = if file == "-" {
            Box::new(stdin().lock())
        } else {
            Box::new(match File::open(&file) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open file: {e}");
                    std::process::exit(1);
                }
            })
        };
        let cnf = parser::parse_dimacs(BufReader::new(reader)).unwrap();

        let start_time = Instant::now();
        let answer: Option<solver_state::Assignment> = match args.solver {
            SolverOption::Cdcl => solve_cdcl::solve_cdcl(&cnf),
            SolverOption::CdclFirstUip => solve_cdcl_first_uip::solve_cdcl_first_uip(&cnf),
            SolverOption::Dpll => solve_simple::solve_dpll(&cnf),
            SolverOption::Backtrack => solve_simple::solve_backtrack(&cnf),
            SolverOption::Basic => solve_simple::solve_basic(&cnf),
        };
        let duration = start_time.elapsed();

        if args.dimacs_output {
            println!("c runtime: {duration:?}");
            parser::output_dimacs(&mut BufWriter::new(stdout()), &answer).unwrap();
        } else {
            let line_beginning = if answer.is_some() {
                "\x1b[32mSAT"
            } else {
                "\x1b[31mUNSAT"
            };
            println!(
                "{line_beginning}: {file} in {:.3}s\x1b[0m",
                duration.as_secs_f64()
            );
        }

        // We don't have proofs of unsatisfiability yet.

        if let Some(assignment) = answer {
            assert!(
                assignment.get_unassigned_var().is_none()
                    && solver_state::check_assignment(&cnf, &assignment)
            );
        }
    }
}
