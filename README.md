# Quiche

A Python-like language that compiles to Rust.

> [!IMPORTANT]
> Quiche is an experimental language exploring how far safe Rust can be stretched to achieve Python-like expressiveness without sacrificing Rust-level performance.

**[See Examples →](examples/)** | **[Documentation](docs/)**

## What It Looks Like

(Example is written in the more Rust-like MetaQuiche dialect)

```python
# Sudoku solver in MetaQuiche (examples/scripts/sudoku.qrs)
def solve(board: mutref[Vec[Vec[i32]]]) -> bool:
    row: i32 = find_empty_row(ref(deref(board)))
    if row == -1:
        return True  # Solved!
    
    row_idx: usize = row as usize
    col: i32 = find_empty_col(ref(deref(board)), row_idx)
    
    for num in range(1, 10):
        if is_valid(ref(deref(board)), row_idx, col as usize, num as i32):
            deref(board)[row_idx][col as usize] = num as i32
            if solve(board):
                return True
            deref(board)[row_idx][col as usize] = 0  # Backtrack
    
    return False
```

## Quick Start

```bash
# Build
make

# Run a script
./bin/quiche examples/scripts/sudoku.qrs
```

See [examples/](examples/) for more code samples.

## Status

| Phase | Description | Status |
|-------|-------------|--------|
| 1. Bootstrap | Host compiler in Rust | ✅ Done |
| 2. Self-hosting | Compiler compiles itself | ✅ Done |
| 3. Zero deps | Remove external crates | 🔄 WIP |
| 4. Quiche | Higher-level dialect | 📋 Planned |

## Documentation

- [Design Philosophy](docs/design.md) — MetaQuiche vs Quiche dialects
- [Language Features](docs/features.md) — What works and what's coming
- [Build Guide](docs/building.md) — Detailed build instructions

## License

BSD-3
