# SAT solver

This is a Rust-based SAT solver using CDCL (conflict-driven clause learning).

## Usage

```bash
cargo run -- --solver dpll --dimacs-output examples/example2.cnf
```

```bash
RUST_LOG=info cargo run -- --solver cdcl --dimacs-output examples/example3.cnf
```

Run on all of an open SATLIB dataset:
```bash
mkdir examples/aim
wget -qO- https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/DIMACS/AIM/aim.tar.gz | tar xvf - -C examples/aim
cargo run examples/aim/*
```

Run 20 random 3CNF-SAT instances with 40 variables and 160 clauses:
```bash
cargo run --bin random -- -n 40 -k 3 -l 160 -r 20
```