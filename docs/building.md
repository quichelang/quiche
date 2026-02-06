# Building Quiche

## Prerequisites

- Rust (nightly recommended)
- Python 3 (for verification script)

## Quick Build

```bash
make          # Bootstrap verify + build Quiche compiler
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

## Quiche Compiler Build

`quiche-compiler` is the user-facing compiler for `.q` source files.

```bash
make quiche         # Build Quiche compiler (depends on stage2 bootstrap compiler)
make quiche-release # Optimized Quiche compiler
```

## Binary Locations

After building:

| Path | Description |
|------|-------------|
| `./bin/quiche` | Quiche compiler (`.q`) |
| `./bin/mq-quiche` | Alias to Quiche compiler |
| `./bin/mq` | Stage 2 bootstrap compiler (`metaquiche-native`) |
| `./bin/stage1` | Stage 1 bootstrap compiler |
| `./bin/mq0` | Host compiler |

## Release Build

```bash
make release   # Build optimized Quiche compiler
make install   # Install Quiche compiler to ~/.cargo/bin
```

## Make Targets

| Target | Description |
|--------|-------------|
| `make` | Verify bootstrap and build Quiche compiler (default) |
| `make bootstrap` | Build stage2 bootstrap compiler |
| `make stage1` | Build with host compiler |
| `make stage2` | Build self-hosted compiler |
| `make verify` | Verify stage1/stage2 parity |
| `make quiche` | Build Quiche compiler |
| `make quiche-release` | Build optimized Quiche compiler |
| `make release` | Alias of `make quiche-release` |
| `make install` | Install Quiche compiler (`quiche` and `mq`) |
| `make test-bootstrap` | Run bootstrap regression test |
| `make test-quiche` | Run Quiche `.q` smoke tests |
| `make test` | Run bootstrap + Quiche tests |
| `make clean` | Remove all build artifacts |

## Running Scripts

```bash
./bin/quiche script.q
./bin/quiche examples/scripts/sudoku.q
./bin/mq script.qrs
```

## Creating Projects

```bash
./bin/quiche new myproject
cd myproject
./bin/quiche build
./bin/quiche run
```
