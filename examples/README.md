# Examples

This directory contains example Quiche code.

## Script Examples

Standalone scripts that can be run directly:

```bash
# From the repository root
./bin/quiche examples/scripts/demo.q
./bin/quiche examples/scripts/sudoku.q
```

| Script | Dialect | Description |
|--------|---------|-------------|
| `scripts/demo.q` | Quiche | Structs, methods, f-strings, comprehensions, lambdas |
| `scripts/sudoku.q` | Quiche | Sudoku solver with auto-borrowing |
| `scripts/sudoku.qrs` | MetaQuiche | Sudoku solver with explicit refs |
| `scripts/test.q` | Quiche | Basic smoke test |
| `scripts/test.qrs` | MetaQuiche | Basic smoke test |
| `scripts/traits.qrs` | MetaQuiche | Trait implementation example |

## Cargo Examples

*(Coming soon)*

Full Quiche projects with `Cargo.toml` that demonstrate:
- Multi-file projects
- Dependency management
- Library vs binary crates
- Testing patterns

```bash
# Run a cargo example (when available)
cd cargo/example-name
../../bin/quiche build
../../bin/quiche run
```
