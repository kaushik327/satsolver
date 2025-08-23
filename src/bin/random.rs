


use clap::Parser;
use satsolver::random;
use satsolver::solve_cdcl;
use std::time::Instant;

#[derive(Parser, Debug)]
struct Args {
    /// Number of variables
    #[arg(short='n', long)]
    num_variables: usize,
    
    /// Variables per clause
    #[arg(short='k', long, default_value_t = 3)]
    num_variables_per_clause: usize,
    
    /// Number of clauses
    #[arg(short='l', long)]
    num_clauses: usize,

    #[arg(short, long, default_value_t = 1)]
    repetitions: usize,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.repetitions {

        let cnf = random::generate_random_cnf(args.num_variables, args.num_variables_per_clause, args.num_clauses);

        let start_time = Instant::now();
        let answer = solve_cdcl::solve_cdcl(&cnf);
        let duration = start_time.elapsed();

        let line_beginning = if answer.is_some() {
            "\x1b[32mSAT"
        } else {
            "\x1b[31mUNSAT"
        };
        println!(
            "{line_beginning}: n={} k={} l={} in {:.3}s\x1b[0m",
            args.num_variables,
            args.num_variables_per_clause,
            args.num_clauses,
            duration.as_secs_f64()
        );
    }
}
