# MetaQuiche Codegen Issues

Issues encountered during perceus-mem AST transformer integration. These blocked implementing memory analysis in Quiche (`.qrs`) and required falling back to pure Rust.

---

## 1. Module-Level Constants Generate Invalid Rust

**Problem:** Top-level `let` bindings generate `let mut` which is invalid at module scope.

**Quiche Input:**
```python
STRATEGY_INLINE: i32 = 0
STRATEGY_REGION: i32 = 1
```

**Generated Rust (Invalid):**
```rust
let mut STRATEGY_INLINE: i32 = 0;  // ERROR: expected item, found `let`
let mut STRATEGY_REGION: i32 = 1;
```

**Expected Rust:**
```rust
pub const STRATEGY_INLINE: i32 = 0;
pub const STRATEGY_REGION: i32 = 1;
```

**Workaround:** Define constants as `@extern` functions in Rust:
```python
@extern(path="crate::strategy_inline")
def STRATEGY_INLINE() -> i32: pass
```

---

## 2. Generic Types Use `[T]` Instead of `<T>` in Struct Fields

**Problem:** Struct field type annotations use square brackets, not angle brackets.

**Quiche Input:**
```python
class EscapeInfo(Struct):
    escaping: Vec[String]
    local_only: Vec[String]
```

**Generated Rust (Invalid):**
```rust
pub struct EscapeInfo {
    pub escaping: Vec[String],    // ERROR: expected `,` or `}`, found `[`
    pub local_only: Vec[String],
}
```

**Expected Rust:**
```rust
pub struct EscapeInfo {
    pub escaping: Vec<String>,
    pub local_only: Vec<String>,
}
```

**Workaround:** Define structs in Rust lib.rs, import via `@extern` for construction.

---

## 3. Imports From Parent Crate Fail

**Problem:** Attempting to import types from the parent Rust crate generates invalid `use` statements.

**Quiche Input:**
```python
from rust import MemConfig, EscapeInfo, MemoryAnalyzer
```

**Generated Rust (Invalid):**
```rust
use rust::MemConfig;  // ERROR: unresolved import `rust`
use rust::EscapeInfo;
use rust::MemoryAnalyzer;
```

**Expected Rust:**
```rust
use crate::MemConfig;
use crate::EscapeInfo;
use crate::MemoryAnalyzer;
```

**Workaround:** Don't use Quiche `class` definitions; define structs in Rust and use `@extern` functions for all type construction.

---

## 4. Enum Variant Access Not Fully Supported

**Problem:** Enum variant syntax `AllocationStrategy.INLINE` may not translate correctly.

**Quiche Input:**
```python
class AllocationStrategy(Enum):
    INLINE
    REGION
    MANAGED

def get_strategy() -> AllocationStrategy:
    return AllocationStrategy.INLINE
```

**Potential Issues:**
- Enum definitions in Quiche may generate invalid Rust
- Variant access may need special handling

**Workaround:** Define enums in Rust, use i32 constants with `@extern` accessors.

---

## Summary

| Issue | Impact | Workaround |
|-------|--------|------------|
| Module-level constants | Cannot define global constants | Use `@extern` functions |
| Generic struct fields | Cannot define structs with Vec/HashMap | Define in Rust, use `@extern` |
| Crate imports | Cannot import from parent crate | Define types in Rust |
| Enum variants | Enum access may fail | Use i32 + `@extern` functions |

## Recommendation

For complex type definitions (structs with generics, enums), define them in Rust and expose via `@extern` functions. Quiche works well for:
- Business logic with function calls
- Control flow (if/match/for/while)
- Method calls on existing types
- Simple type annotations
