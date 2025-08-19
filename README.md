# SAT solver

This is a Rust-based SAT solver using CDCL (conflict-driven clause learning).

## Usage

```bash
cargo run -- --solver dpll --dimacs-output examples/example2.cnf
```

```bash
RUST_LOG=info cargo run -- --solver cdcl-first-uip --dimacs-output examples/example3.cnf
```

```bash
mkdir examples/aim
wget -qO- https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/DIMACS/AIM/aim.tar.gz | tar xvf - -C examples/aim
cargo run examples/aim/*
```
