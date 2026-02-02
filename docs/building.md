# Building Quiche

## Prerequisites

- Rust (nightly recommended)
- Python 3 (for verification script)

## Quick Build

```bash
make          # Build and verify (default)
```

## Build Stages

The compiler bootstraps through three stages:

```
Stage 0 (Rust)  →  Stage 1 (MetaQuiche)  →  Stage 2 (Self-compiled)
     ↓                    ↓                        ↓
 quiche-host         compiles to            byte-identical
                    native binary            to Stage 1
```

```bash
make stage1    # Build with host compiler (Rust → MetaQuiche)
make stage2    # Build with stage1 (MetaQuiche → Self)
make verify    # Verify stage1 == stage2 output
```

## Binary Locations

After building:

| Path | Description |
|------|-------------|
| `./bin/quiche` | Symlink to latest build |
| `./bin/quiche-host` | Host compiler (Rust) |
| `./bin/quiche-stage1` | Stage 1 compiler |

## Release Build

```bash
make release   # Build optimized binary
make install   # Install to /usr/local/bin (requires sudo)
```

## Make Targets

| Target | Description |
|--------|-------------|
| `make` | Build and verify (default) |
| `make stage1` | Build with host compiler |
| `make stage2` | Build self-hosted compiler |
| `make verify` | Verify stage1/stage2 parity |
| `make release` | Build optimized release |
| `make install` | Install to /usr/local/bin |
| `make clean` | Remove all build artifacts |

## Running Scripts

```bash
./bin/quiche script.qrs
./bin/quiche examples/scripts/sudoku.qrs
```

## Creating Projects

```bash
./bin/quiche new myproject
cd myproject
./bin/quiche build
./bin/quiche run
```
