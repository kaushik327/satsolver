use std::io::Write;

use clap::Parser;
use satsolver::config::*;
use satsolver::random;
use satsolver::solve_cdcl;
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum PolarityOption {
    AlwaysFalse,
    AlwaysTrue,
    PhaseSaving,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum RestartOption {
    None,
    Luby,
    Geometric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum DeletionOption {
    None,
    Lbd,
    Activity,
}

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

    #[arg(long, default_value = "phase-saving")]
    polarity: PolarityOption,

    #[arg(long, default_value = "luby")]
    restart: RestartOption,

    #[arg(long, default_value_t = 100)]
    restart_unit: u32,

    #[arg(long, default_value_t = 100)]
    restart_initial: u32,

    #[arg(long, default_value_t = 1.5)]
    restart_factor: f64,

    #[arg(long, default_value = "lbd")]
    deletion: DeletionOption,

    #[arg(long, default_value_t = 6)]
    deletion_max_lbd: u32,

    #[arg(long, default_value_t = 0.5)]
    deletion_fraction: f64,
}

fn main() {
    let args = Args::parse();

    let config = SolverConfig {
        polarity: match args.polarity {
            PolarityOption::AlwaysFalse => PolarityHeuristic::AlwaysFalse,
            PolarityOption::AlwaysTrue => PolarityHeuristic::AlwaysTrue,
            PolarityOption::PhaseSaving => PolarityHeuristic::PhaseSaving,
        },
        restart: match args.restart {
            RestartOption::None => RestartStrategy::None,
            RestartOption::Luby => RestartStrategy::Luby {
                unit: args.restart_unit,
            },
            RestartOption::Geometric => RestartStrategy::Geometric {
                initial: args.restart_initial,
                factor: args.restart_factor,
            },
        },
        deletion: match args.deletion {
            DeletionOption::None => DeletionStrategy::None,
            DeletionOption::Lbd => DeletionStrategy::Lbd {
                max_lbd: args.deletion_max_lbd,
            },
            DeletionOption::Activity => DeletionStrategy::Activity {
                fraction: args.deletion_fraction,
            },
        },
    };

    let config_label = format!(
        "{}/{}/{}",
        match args.polarity {
            PolarityOption::AlwaysFalse => "always-false",
            PolarityOption::AlwaysTrue => "always-true",
            PolarityOption::PhaseSaving => "phase-saving",
        },
        match args.restart {
            RestartOption::None => "none".to_string(),
            RestartOption::Luby => format!("luby({})", args.restart_unit),
            RestartOption::Geometric =>
                format!("geo({},{})", args.restart_initial, args.restart_factor),
        },
        match args.deletion {
            DeletionOption::None => "none".to_string(),
            DeletionOption::Lbd => format!("lbd({})", args.deletion_max_lbd),
            DeletionOption::Activity => format!("act({})", args.deletion_fraction),
        },
    );

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
                let answer = solve_cdcl::solve_cdcl(&cnf, &config);
                let duration = start.elapsed();

                let result = json!({
                    "config": config_label,
                    "n": num_variables,
                    "k": args.num_variables_per_clause,
                    "l": num_clauses,
                    "sat": answer.is_satisfiable(),
                    "duration_ms": duration.as_millis(),
                });
                println!("{result}");
                std::io::stdout().flush().unwrap();

                if answer.is_satisfiable() {
                    successful += 1;
                }
            }
            eprintln!(
                "[{}] n={} l={}: SAT {}/{}",
                config_label, num_variables, num_clauses, successful, args.repetitions
            );
        }
    }
}
