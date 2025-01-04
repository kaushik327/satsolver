// DIMACS CNF parser.

use super::formula::{Clause, Lit, Val, Var, CNF};

use itertools::Itertools;
use std::io;
use std::io::BufRead;

pub fn parse_dimacs<R: io::Read>(reader: &mut io::BufReader<R>) -> Result<CNF, io::Error> {
    // Split into non-comment lines and tokenize
    let mut tokens = reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !line.starts_with('c'))
        .flat_map(|line| {
            line.split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        });

    // Parse header, num_vars, and expected_clauses in one go
    let (num_vars, expected_clauses) =
        match (tokens.next(), tokens.next(), tokens.next(), tokens.next()) {
            (Some(p), Some(cnf), Some(vars), Some(clauses)) if p == "p" && cnf == "cnf" => {
                let num_vars = vars.parse::<u32>().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid number of variables")
                })?;
                let expected_clauses = clauses.parse::<u32>().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid number of clauses")
                })?;
                (num_vars, expected_clauses)
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid DIMACS header",
                ))
            }
        };

    // Split numeric tokens by zeros and turn into literals and clauses
    let nums = tokens.map(|token| {
        if let Ok(num) = token.parse::<i32>() {
            if num.abs() as u32 <= num_vars {
                return Ok(num);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid literal",
        ))
    });
    let clauses = nums
        .chunk_by(|res| matches!(res, Ok(0)))
        .into_iter()
        .filter(|(key, _)| !key)
        .map(|(_, chunk)| {
            let literals = chunk
                .map(|res| {
                    res.map(|num| Lit {
                        var: Var {
                            index: num.abs() as u32,
                        },
                        value: if num > 0 { Val::TRUE } else { Val::FALSE },
                    })
                })
                .collect::<Result<Vec<Lit>, io::Error>>()?;
            Ok(Clause { literals })
        })
        .collect::<Result<Vec<Clause>, io::Error>>()?;

    if clauses.len() != expected_clauses as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Incorrect number of clauses",
        ));
    }

    Ok(CNF { num_vars, clauses })
}
