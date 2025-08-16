// DIMACS CNF parser.

use crate::formula::*;
use crate::solver_state::*;

use itertools::Itertools;
use std::io;
use std::io::Write;

pub fn parse_dimacs(reader: impl io::BufRead) -> Result<CnfFormula, io::Error> {
    // Split into non-comment lines and tokenize
    let mut tokens = reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.starts_with('c'))
        .flat_map(|line| {
            line.split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // Some of these files end with "%" "0" for some reason, so remove them
    if tokens.ends_with(&["%".into(), "0".into()]) {
        tokens.truncate(tokens.len() - 2);
    }

    // Parse header, num_vars, and expected_clauses in one go
    let (num_vars, expected_clauses) =
        match (tokens.first(), tokens.get(1), tokens.get(2), tokens.get(3)) {
            (Some(p), Some(cnf), Some(vars), Some(clauses)) if p == "p" && cnf == "cnf" => {
                let num_vars = vars.parse::<usize>().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid number of variables")
                })?;
                let expected_clauses = clauses.parse::<usize>().map_err(|_| {
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
    let nums = tokens[4..]
        .iter()
        .map(|token| match token.parse::<isize>() {
            Ok(num) if num.unsigned_abs() <= num_vars => Ok(num),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid literal: {token}"),
            )),
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
                            index: num.unsigned_abs(),
                        },
                        value: if num > 0 { Val::True } else { Val::False },
                    })
                })
                .collect::<Result<Vec<Lit>, io::Error>>()?;
            Ok(Clause { literals })
        })
        .collect::<Result<Vec<Clause>, io::Error>>()?;

    if clauses.len() != expected_clauses {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Incorrect number of clauses",
        ));
    }

    Ok(CnfFormula { num_vars, clauses })
}

#[cfg(test)]
pub fn parse_dimacs_str(text: &[u8]) -> Result<CnfFormula, io::Error> {
    parse_dimacs(&mut io::BufReader::new(text))
}

pub fn output_dimacs<W: io::Write>(
    writer: &mut io::BufWriter<W>,
    assignment: &Option<Assignment>,
) -> io::Result<()> {
    if let Some(assignment) = assignment {
        writer.write_all(b"s SATISFIABLE\nv")?;

        for i in 1..=assignment.num_vars() {
            let Some(satisfied) = assignment.get(&Lit {
                var: Var { index: i },
                value: Val::True,
            }) else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid assignment",
                ));
            };
            writer.write_all(b" ")?;
            if !satisfied {
                writer.write_all(b"-")?;
            }
            writer.write_all(format!("{i}").as_bytes())?;
        }
    } else {
        writer.write_all(b"s UNSATISFIABLE")?;
    }
    writer.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_var_out_of_bounds() {
        let cnf = parse_dimacs_str(b"p cnf 3 3\n1 2 0\n1 -2 0\n3 -4 0");
        assert!(cnf.is_err());
    }

    #[test]
    fn test_parse_too_few_clauses() {
        let cnf = parse_dimacs_str(b"p cnf 3 3\n1 2 0\n1 -2 0");
        assert!(cnf.is_err());
    }

    #[test]
    fn test_parse_too_many_clauses() {
        let cnf = parse_dimacs_str(b"p cnf 3 2\n1 2 0\n1 -2 0\n3 -2 0");
        assert!(cnf.is_err());
    }

    #[test]
    fn test_parse_normal() {
        let cnf = parse_dimacs_str(
            b"c comment\np cnf 5 5\n1 2 0\n1 -2 0\nc another comment\n3 4 0\n3 -4 0\n-1 -3 0",
        );
        assert!(cnf.is_ok());
        assert_eq!(
            cnf.unwrap(),
            CnfFormula {
                num_vars: 5,
                clauses: vec![
                    Clause {
                        literals: vec![
                            Lit {
                                var: Var { index: 1 },
                                value: Val::True
                            },
                            Lit {
                                var: Var { index: 2 },
                                value: Val::True
                            }
                        ]
                    },
                    Clause {
                        literals: vec![
                            Lit {
                                var: Var { index: 1 },
                                value: Val::True
                            },
                            Lit {
                                var: Var { index: 2 },
                                value: Val::False
                            }
                        ]
                    },
                    Clause {
                        literals: vec![
                            Lit {
                                var: Var { index: 3 },
                                value: Val::True
                            },
                            Lit {
                                var: Var { index: 4 },
                                value: Val::True
                            }
                        ]
                    },
                    Clause {
                        literals: vec![
                            Lit {
                                var: Var { index: 3 },
                                value: Val::True
                            },
                            Lit {
                                var: Var { index: 4 },
                                value: Val::False
                            }
                        ]
                    },
                    Clause {
                        literals: vec![
                            Lit {
                                var: Var { index: 1 },
                                value: Val::False
                            },
                            Lit {
                                var: Var { index: 3 },
                                value: Val::False
                            }
                        ]
                    },
                ]
            }
        );
    }
}
