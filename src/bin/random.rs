use clap::Parser;
use satsolver::random;
use satsolver::solve_cdcl;

#[derive(Parser, Debug)]
struct Args {
    /// Number of variables
    #[arg(short = 'n', long)]
    num_variables: usize,

    /// Variables per clause
    #[arg(short = 'k', long, default_value_t = 3)]
    num_variables_per_clause: usize,

    /// Number of clauses
    #[arg(short = 'l', long)]
    num_clauses: usize,

    #[arg(short, long, default_value_t = 1)]
    repetitions: usize,
}

fn main() {
    let args = Args::parse();

    let mut successful = 0;

    for _ in 0..args.repetitions {
        let cnf = random::generate_random_cnf(
            args.num_variables,
            args.num_variables_per_clause,
            args.num_clauses,
        );
        let answer = solve_cdcl::solve_cdcl(&cnf);

        if answer.is_some() {
            successful += 1;
        }
    }
    println!(
        "n={} k={} l={}: SAT {}/{}",
        args.num_variables,
        args.num_variables_per_clause,
        args.num_clauses,
        successful,
        args.repetitions
    );
}
