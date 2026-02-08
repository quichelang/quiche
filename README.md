# Quiche

A Python-like language that compiles to Rust.

> [!IMPORTANT]
> Quiche is a self-hosted, experimental language exploring how far safe Rust can be stretched to achieve Python-like expressiveness without sacrificing Rust-level performance.

**[See Examples →](examples/)** | **[Documentation](docs/)** | **[Tests](tests/)** | **[Status](docs/status.md)**

## What It Looks Like

### Quiche (`.q` files) — Pythonic, clean

```python
from rust.std.collections import HashMap

class Student(Struct):
    name: String
    age: u8

    def new(name: String, age: u8) -> Student:
        return Student(name=name, age=age)

    def bio(self) -> String:
        return f"{self.name} is {self.age} years old"

def main():
    nums = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]

    add = |x: i32, y: i32| x + y
    print(f"Sum: {add(2, 3)}")
    print(f"Length: {len(doubled)}")

    students = [
        Student.new("Alice", 20),
        Student.new("Bob", 22)
    ]
    for s in students:
        print(s.bio())
```

### MetaQuiche (`.qrs` files) — Explicit memory control

```python
def solve(board: mutref[Vec[Vec[i32]]]) -> bool:
    row: i32 = find_empty_row(ref(deref(board)))
    if row == -1:
        return True

    for num in range(1, 10):
        if is_valid(ref(deref(board)), row as usize, col as usize, num as i32):
            deref(board)[row as usize][col as usize] = num as i32
            if solve(board):
                return True
            deref(board)[row as usize][col as usize] = 0

    return False
```

## Quick Start

```bash
# Build bootstrap compiler + Quiche compiler
make

# Run Quiche (.q) code
./bin/quiche examples/scripts/demo.q
./bin/quiche examples/scripts/sudoku.q
```

See [examples/](examples/) and [tests/](tests/) for more code samples.

## Status

| Phase | Description | Status |
|-------|-------------|--------|
| 1. Bootstrap | Host compiler in Rust | ✅ Done |
| 2. Self-hosting | Compiler compiles itself | ✅ Done |
| 3. Minimal deps | 15 crates total (was 100+) | ✅ Done |
| 4. Custom parser | Hand-written recursive-descent | ✅ Done |
| 5. Template system | Shared codegen strings for parity | ✅ Done |
| 6. i18n | Zero-dep template-based `t!()` | ✅ Done |
| 7. Perceus memory | FBIP, regions, weak refs, managed types | ✅ Done |
| 8. Quiche dialect | Comprehensions, f-strings, auto-borrowing | 🔄 Active |
| 9. Diagnostics | Colorized errors, telemetry | 🔄 Active |
| 10. Testing | qtest framework + smoke tests | 🔄 Active |

See **[docs/status.md](docs/status.md)** for a detailed breakdown including metrics, gaps, and roadmap.

## Documentation

- [Design Philosophy](docs/design.md) — MetaQuiche vs Quiche dialects
- [Language Features](docs/features.md) — What works and what's coming
- [Build Guide](docs/building.md) — Detailed build instructions
- [Compiler Architecture](docs/compiler_architecture.md) — Multi-stage bootstrap design
- [Project Status](docs/status.md) — Detailed progress and metrics
- [Why Quiche?](docs/why-quiche.md) — Vision and motivation

## License

BSD-3
