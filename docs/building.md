# Building Quiche

## Prerequisites

- Rust (stable or nightly)

## Quick Build

```bash
cargo build -p quiche          # Build the Quiche compiler
cargo build -p quiche --release # Optimized build
```

## Running Scripts

```bash
quiche script.q                # Compile and run
quiche script.q --emit-rust    # Show generated Rust
quiche script.q --emit-elevate # Show Elevate IR
quiche script.q --emit-ast     # Dump parsed AST
```

## Running Tests

```bash
quiche test                    # Run all .q test files
cargo test -p quiche           # Run Rust unit tests
cargo test -p quiche-lib       # Run quiche-lib unit tests
```

## Creating Projects

```bash
quiche init myproject          # Scaffold a Quiche project
cd myproject
quiche build main.q            # Compile to Rust
```

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `quiche` | Compiler binary — parser, Elevate bridge, CLI |
| `quiche-lib` (lib/) | Runtime types — `Str`, `List`, `Dict`, `QuicheType` |
| `elevate` (git dep) | Backend — type inference, ownership, Rust codegen |
