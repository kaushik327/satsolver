use satsolver::parser;
use satsolver::solve_cdcl;
use satsolver::solve_cnc;
use satsolver::solve_simple;
use satsolver::solver_state;

use clap::Parser;
use std::fs::File;
use std::io::{stdin, BufReader, BufWriter, Read};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "cdcl")]
    solver: SolverOption,

    /// Depth parameter for CNC solver
    #[arg(short, long, default_value_t = 3)]
    depth: usize,

    file: Vec<String>,

    #[arg(long)]
    dimacs_output: Option<String>,

    #[arg(long)]
    drat_output: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum SolverOption {
    Cdcl,
    Cnc,
    Dpll,
    Backtrack,
    Basic,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    if args.file.is_empty() {
        eprintln!("No input files specified");
        std::process::exit(1);
    }

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
        let answer: solver_state::SolverResult = match args.solver {
            SolverOption::Cdcl => solve_cdcl::solve_cdcl(&cnf),
            SolverOption::Cnc => solve_cnc::solve_cnc(&cnf, args.depth),
            SolverOption::Dpll => solve_simple::solve_dpll(&cnf),
            SolverOption::Backtrack => solve_simple::solve_backtrack(&cnf),
            SolverOption::Basic => solve_simple::solve_basic(&cnf),
        };
        let duration = start_time.elapsed();

        if let Some(ref dimacs_output_filename) = args.dimacs_output {
            println!("c runtime: {duration:?}");
            parser::output_dimacs(
                &mut BufWriter::new(File::create(dimacs_output_filename).unwrap()),
                &answer,
            )
            .unwrap();
        }
        if let Some(ref drat_output_filename) = args.drat_output {
            if let Some(proof) = answer.unsat_proof() {
                parser::output_drat(
                    &mut BufWriter::new(File::create(drat_output_filename).unwrap()),
                    &proof,
                )
                .unwrap();
            }
        }

        let line_beginning = if answer.is_satisfiable() {
            "\x1b[32mSAT"
        } else {
            "\x1b[31mUNSAT"
        };
        println!(
            "{line_beginning}: {file} in {:.3}s\x1b[0m",
            duration.as_secs_f64()
        );

        if let Some(assignment) = answer.assignment() {
            assert!(
                assignment.get_unassigned_var().is_none()
                    && solver_state::check_assignment(&cnf, assignment)
            );
        }
    }
}
