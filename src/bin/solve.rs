use satsolver::parser;
use satsolver::solve_cdcl;
use satsolver::solve_cnc;
use satsolver::solve_simple;
use satsolver::solver_state;

use clap::Parser;
use std::fs::{self, File};
use std::io::{stdin, BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "cdcl")]
    solver: SolverOption,

    /// Depth parameter for CNC solver
    #[arg(short, long, default_value_t = 3)]
    depth: usize,

    /// Input CNF files to solve. Use '-' for stdin.
    /// Multiple files allowed only when no output files are specified.
    file: Vec<String>,

    /// Output directory. When specified, DIMACS and DRAT files
    /// will be generated for each input file using the input filename as base.
    /// Example: input.cnf -> output_dir/input.dimacs, output_dir/input.drat
    #[arg(short, long)]
    output_dir: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum SolverOption {
    Cdcl,
    Cnc,
    Dpll,
    Backtrack,
    Basic,
}

/// Generate output filename for batch processing
fn generate_output_filename(input_file: &str, output_dir: &str, extension: &str) -> PathBuf {
    let input_path = Path::new(input_file);
    let base_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    Path::new(output_dir).join(format!("{}.{}", base_name, extension))
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    if args.file.is_empty() {
        eprintln!("No input files specified");
        std::process::exit(1);
    }

    // Create output directory if specified
    if let Some(ref output_dir) = args.output_dir {
        if let Err(e) = fs::create_dir_all(output_dir) {
            eprintln!("Failed to create output directory '{}': {}", output_dir, e);
            std::process::exit(1);
        }
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

        // Handle output files
        if file == "-" {
            // We don't output anything in this case
        } else if let Some(ref output_dir) = args.output_dir {
            let dimacs_path = generate_output_filename(&file, output_dir, "dimacs");
            parser::output_dimacs(
                &mut BufWriter::new(File::create(&dimacs_path).unwrap()),
                &answer,
            )
            .unwrap();
            if let Some(proof) = answer.unsat_proof() {
                let drat_path = generate_output_filename(&file, output_dir, "drat");
                parser::output_drat(
                    &mut BufWriter::new(File::create(&drat_path).unwrap()),
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
