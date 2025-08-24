use std::io::Write;

use clap::Parser;
use satsolver::random;
use satsolver::solve_cdcl;
use serde_json::json;

#[derive(Parser, Debug)]
struct Args {
    /// Variables per clause
    #[arg(short = 'k', long, default_value_t = 3)]
    num_variables_per_clause: usize,

    /// Number of variables
    #[arg(short = 'n', long, num_args(1..))]
    num_variables: Vec<usize>,

    /// Number of clauses
    #[arg(short = 'l', long, num_args(1..))]
    num_clauses: Vec<usize>,

    #[arg(short, long, default_value_t = 1)]
    repetitions: usize,
}

fn main() {
    let args = Args::parse();
    for num_variables in &args.num_variables {    
        for num_clauses in &args.num_clauses {
            let mut successful = 0;
            for _ in 0..args.repetitions {
                let cnf = random::generate_random_cnf(
                    *num_variables,
                    args.num_variables_per_clause,
                    *num_clauses,
                );
                let start = std::time::Instant::now();
                let answer = solve_cdcl::solve_cdcl(&cnf);
                let duration = start.elapsed();

                let result = json!({
                    "n": num_variables,
                    "k": args.num_variables_per_clause,
                    "l": num_clauses,
                    "sat": answer.is_some(),
                    "duration": duration.as_millis(),
                });
                println!("{}", result.to_string());
                std::io::stdout().flush().unwrap();

                if answer.is_some() {
                    successful += 1;
                }
            }
            eprintln!(
                "n={} k={} l={}: SAT {}/{}",
                num_variables,
                args.num_variables_per_clause,
                num_clauses,
                successful,
                args.repetitions
            );
        }
    }
}
