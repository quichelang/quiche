# MetaQuiche Codegen Issues

Issues encountered during perceus-mem AST transformer integration.

---

## ✅ FIXED: Enum Variant Syntax Clarification

**Status:** User error — bare identifiers are not valid Python enum syntax

**Incorrect Syntax (doesn't work):**
```python
class AllocationStrategy(Enum):
    Inline        # ❌ Bare identifier = expression statement
    Region
```

**Correct Syntax:**
```python
class AllocationStrategy(Enum):
    Inline = ()           # ✅ Unit variant
    Region = ()
    Managed = ()
    Green = (i32,)        # ✅ Tuple variant with payload
```

> **Note:** Python enums require assignment syntax. Bare identifiers are parsed as expression statements, not enum variants.

---

## ✅ FIXED: Module-Level Constants

**Status:** Fixed via SCREAMING_SNAKE_CASE detection

SCREAMING_SNAKE_CASE variables now correctly generate `pub const`:

```python
STRATEGY_INLINE: i32 = 0   # → pub const STRATEGY_INLINE: i32 = 0;
```

Also supported: `Const[T]` explicit annotation for non-SCREAMING names.

---

## ✅ FIXED: Generic Types in Struct Fields

**Status:** Fixed via `expr_to_type_string` correction

**Bug:** The `expr_to_type_string` function in parser.rs was outputting `Vec[T]` instead of `Vec<T>`.

**Fix:** Changed format string from `"{}[{}]"` to `"{}<{}>"` in `expr_to_type_string`.

```python
class EscapeInfo(Struct):
    escaping: Vec[String]     # → Vec<String> (now works!)
```

> **Note:** Function parameters/return types already worked; only struct fields were affected.

---

## 🔶 PARTIAL: Nested Enum Types Not In Scope

**Status:** Nested types like `Constant::Bool` aren't automatically imported

**Quiche Input:**
```python
match c:
    case Constant.Bool(b):
        # ...
```

**Generated Rust (Invalid):**
```rust
Constant::Bool(b) => {  // ERROR: use of undeclared type `Constant`
```

**Expected:** Either auto-import `use quiche_parser::ast::Constant;` or fully qualify as `quiche_parser::ast::Constant::Bool`.

**Workaround:** Avoid matching on nested enum types, or restructure to avoid the pattern.

---

## ❌ OPEN: Type Inference Through Extern Boundaries

**Status:** Return types from `@extern` functions don't unify with Quiche-defined structs

**Problem:** When both Rust and Quiche define the same struct, they're treated as different types:

```python
# Quiche defines
class MemConfig(Struct):
    is_inline: bool

# Extern returns Rust's crate::MemConfig
@extern(path="crate::create_MemConfig")
def create_MemConfig() -> MemConfig: pass

def new_mem_config() -> MemConfig:
    return create_MemConfig()  # ERROR: mismatched types
```

**Root cause:** Quiche generates its own `MemConfig` struct, but `@extern` returns `crate::MemConfig`. These are different types to Rust.

**Workaround:** Don't define structs in Quiche if you need to construct them via `@extern`. Use the Rust-defined types exclusively.

---

## ❌ OPEN: HashMap Type Inference

**Status:** `HashMap.get()` return type needs explicit annotation

**Quiche Input:**
```python
match self.type_strategies.get(ref(type_name)):
    case Some(strategy):
        return deref(strategy)
```

**Error:**
```
error[E0282]: type annotations needed
   --> return crate::quiche::deref!(strategy);
```

**Workaround:** Add explicit type annotation:
```python
strategy_opt: Option[ref[i32]] = self.type_strategies.get(ref(type_name))
```

---

## Summary

| Issue | Status | Impact |
|-------|--------|--------|
| Module-level constants | ✅ Fixed | SCREAMING_SNAKE_CASE works |
| Generic struct fields | 🔶 Partial | `Vec[T]` in fields broken |
| Nested enum imports | 🔶 Partial | `Constant::Bool` not in scope |
| Extern type unification | ❌ Open | Can't mix Quiche + Rust structs |
| HashMap type inference | ❌ Open | Needs explicit annotations |

## Current Workaround Pattern

For complex types, use this pattern:

```python
# 1. Define structs in Rust (lib.rs)
# 2. Create constructors in Rust
# 3. Use @extern to access from Quiche

@extern(path="crate::create_MemConfig")
def create_MemConfig() -> MemConfig: pass

# 4. DON'T redefine the struct in Quiche
# 5. Use explicit type annotations for HashMap returns
```
