mod formula;
mod parser;
mod solve_cdcl;
mod solve_cnc;
mod solve_simple;
mod solver_state;

use clap::Parser;
use std::io;
use std::time::Instant;

type SolverFn = fn(&formula::CnfFormula) -> Option<formula::Assignment>;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    solver: String,
}

fn main() {
    let args = Args::parse();

    let mut reader = io::BufReader::new(io::stdin());
    let cnf = parser::parse_dimacs(&mut reader).unwrap();

    let solver: SolverFn = match args.solver.as_str() {
        "cnc" => |cnf| solve_cnc::solve_cnc(cnf, 3), // TODO: input depth
        "cdcl" => solve_cdcl::solve_cdcl,
        "dpll" => solve_simple::solve_dpll,
        "backtrack" => solve_simple::solve_backtrack,
        "basic" => solve_simple::solve_basic,
        _ => panic!("Unknown solver: {}", args.solver),
    };

    let start_time = Instant::now();
    let answer = solver(&cnf);
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
