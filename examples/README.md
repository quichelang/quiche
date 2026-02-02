# Examples

This directory contains example Quiche code.

## Script Examples

Standalone `.qrs` scripts that can be run directly:

```bash
# From the repository root
./bin/quiche examples/scripts/sudoku.qrs

# Or from this directory
../../bin/quiche scripts/sudoku.qrs
```

| Script | Description |
|--------|-------------|
| `scripts/sudoku.qrs` | Sudoku solver using backtracking algorithm |
| `scripts/traits.qrs` | Trait implementation example |

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
