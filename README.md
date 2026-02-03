# Quiche

A Python-like language that compiles to Rust.

> [!IMPORTANT]
> Quiche is an experimental language exploring how far safe Rust can be stretched to achieve Python-like expressiveness without sacrificing Rust-level performance.

**[See Examples →](examples/)** | **[Documentation](docs/)** | **[Tests](tests/)**

## What It Looks Like

### Quiche (`.q` files) — Pythonic, clean

```python
# List comprehensions and lambdas
def main():
    nums = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]
    
    # Rust-style lambda syntax
    add = |x: i32, y: i32| x + y
    print("Sum: " + add(2, 3))
    
    # Pythonic len()
    print("Length: " + len(doubled))
```

### MetaQuiche (`.qrs` files) — Explicit memory control

```python
# Sudoku solver with explicit references
def solve(board: mutref[Vec[Vec[i32]]]) -> bool:
    row: i32 = find_empty_row(ref(deref(board)))
    if row == -1:
        return True  # Solved!
    
    for num in range(1, 10):
        if is_valid(ref(deref(board)), row as usize, col as usize, num as i32):
            deref(board)[row as usize][col as usize] = num as i32
            if solve(board):
                return True
            deref(board)[row as usize][col as usize] = 0  # Backtrack
    
    return False
```

## Quick Start

```bash
# Build
make

# Run a script
./bin/quiche examples/scripts/sudoku.qrs
```

See [examples/](examples/) and [tests/](tests/) for more code samples.

## Status

| Phase | Description | Status |
|-------|-------------|--------|
| 1. Bootstrap | Host compiler in Rust | ✅ Done |
| 2. Self-hosting | Compiler compiles itself | ✅ Done |
| 3. Minimal deps | Only regex + thiserror | ✅ Done |
| 4. Core Quiche | Pythonic dialect + comprehensions + lambdas | 🔄 WIP |
| 5. Memory mgmt | Perceus-style automatic memory | 📋 Planned |

## Documentation

- [Design Philosophy](docs/design.md) — MetaQuiche vs Quiche dialects
- [Language Features](docs/features.md) — What works and what's coming
- [Build Guide](docs/building.md) — Detailed build instructions

## License

BSD-3
