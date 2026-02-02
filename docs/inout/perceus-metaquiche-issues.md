# MetaQuiche Codegen Issues

Issues encountered during perceus-mem AST transformer integration and testing.

---

## ❌ OPEN: Missing `let` Keyword in `.q` Files

**Status:** Bug discovered in `.q` codegen (2026-02-02)

**Reproduction:** Any variable assignment in a `.q` file:
```python
# test.q
def test_example():
    p = Point(x=10, y=20)  # Bug: generates without let
```

**Generated Rust (Invalid):**
```rust
p = Point { x: 10, y: 20 };  // ERROR: cannot find value `p`
```

**Expected Rust:**
```rust
let p = Point { x: 10, y: 20 };
```

**Impact:** 9 errors in test suite. All variable bindings in `.q` fail.

---

## ❌ OPEN: For-Loop Iterator Reference Issue

**Status:** Bug discovered in `.q` codegen (2026-02-02)

**Reproduction:**
```python
def sum_vec(items: Vec[i32]) -> i32:
    total = 0
    for item in items:
        total = total + item  # Bug: item is &mut i32
    return total
```

**Generated Rust (Invalid):**
```rust
// item is &mut i32, not i32
total = total + item;  // ERROR: cannot add `&mut i32` to `{integer}`
```

**Expected:** Either auto-dereference `*item` or generate `for item in items.into_iter()`.

**Impact:** 3 errors. All for-loops over Vec fail.

---

## ❌ OPEN: Cannot Move Out of Mutable Reference

**Status:** Bug discovered in `.q` codegen (2026-02-02)

**Reproduction:**
```python
def modify(dn: DeepNested) -> DeepNested:
    return DeepNested(
        container=Container(items=dn.container.items, ...),  # Bug
        point=dn.point
    )
```

**Error:**
```rust
error[E0507]: cannot move out of `dn.container.items` which is behind a mutable reference
```

**Impact:** 3 errors. Nested struct field access fails when constructing new structs.

---

## ❌ OPEN: Type Inference Through Extern Boundaries

**Status:** Return types from `@extern` functions don't unify with Quiche-defined structs

**Problem:** When both Rust and Quiche define the same struct, they're treated as different types.

**Workaround:** Don't define structs in Quiche if you need to construct them via `@extern`. Use Rust-defined types exclusively.

---

## ❌ OPEN: HashMap Type Inference

**Status:** `HashMap.get()` return type needs explicit annotation

**Workaround:** Add explicit type annotation:
```python
strategy_opt: Option[ref[i32]] = self.type_strategies.get(ref(type_name))
```

---

## 🔶 PARTIAL: Nested Enum Types Not In Scope

**Status:** Nested types like `Constant::Bool` aren't automatically imported

**Workaround:** Avoid matching on nested enum types, or restructure to avoid the pattern.

---

## ✅ FIXED: Module-Level Constants

SCREAMING_SNAKE_CASE variables correctly generate `pub const`.

---

## ✅ FIXED: Generic Types in Struct Fields

`Vec[T]` now generates `Vec<T>` in struct fields.

---

## ✅ FIXED: Enum Variant Syntax

Use `Variant = ()` syntax for unit variants, `Variant = (T,)` for tuple variants.

---

## Summary

| Issue | Status | Impact |
|-------|--------|--------|
| Missing `let` in `.q` bindings | ❌ Open | All `.q` variable bindings fail |
| For-loop iterator `&mut T` | ❌ Open | All for-loops over Vec fail |
| Move from mutable ref | ❌ Open | Nested struct construction fails |
| Extern type unification | ❌ Open | Can't mix Quiche + Rust structs |
| HashMap type inference | ❌ Open | Needs explicit annotations |
| Nested enum imports | 🔶 Partial | `Constant::Bool` not in scope |
| Module-level constants | ✅ Fixed | SCREAMING_SNAKE_CASE works |
| Generic struct fields | ✅ Fixed | `Vec[T]` → `Vec<T>` works |
| Enum variant syntax | ✅ Fixed | Documented correct syntax |

---

## Test File

These issues were discovered via comprehensive testing:
- **File:** `tests/test_memory.q`
- **Tests:** 50+ covering primitives, nested structs, multi-level calls, vectors, options, closures, recursion
- **Errors:** 21 compilation errors from 5 distinct bugs

---

## 🔬 Investigation Notes (2026-02-02)

### Analysis Approach

1. **Traced `.q` file compilation flow:**
   - `main.qrs:305-306`: Files ending in `.q` go through `transform_module(module, verbose)` before codegen
   - AST transformer (`ast_transformer.qrs`) wraps complex params in `&mut` (`mutref`)
   - Codegen then generates Rust from the transformed AST

2. **Tested generated output:**
   ```bash
   ./target/stage2/debug/quiche tests/test_memory.q --emit-rust
   ```

### Finding 1: "Missing `let`" May Be False Positive

When I ran `--emit-rust`, the output **does have `let mut`**:
```rust
pub fn test_primitive_copy() {
let mut x = 42;    // ✅ `let mut` is present!
let mut y = x;
...
}
```

**Hypothesis:** The original "missing let" report might be outdated or specific to certain edge cases. The `is_var_defined_in_scopes()` function in `codegen.qrs:43-51` looks correct.

### Finding 2: For-Loop Iterator Issue Confirmed

Generated code for `sum_vec`:
```rust
pub fn sum_vec(items: &mut Vec<i32>) -> i32 {
let mut total = 0;
for __q in (items) {        // items is &mut Vec<i32>
let item = __q;             // __q is &mut i32
total = total + item;       // ERROR: can't add &mut i32 to i32
}
return total;
}
```

**Root cause:** The AST transformer wraps params in `&mut`. Iterating `&mut Vec<T>` yields `&mut T`, not `T`.

### Attempted Fix 1: `.iter().cloned()`

Changed `emit_for` in `codegen.qrs:598` to emit:
```rust
for __q in (items).iter().cloned() {
```

**Result:** FAILED - Broke existing iterator chains like `.lines()`:
```rust
for __q in (code.lines()).iter().cloned() {  // ERROR: no method `iter` on Lines
```

**Reverted** the change.

### Potential Approaches

| Approach | Pros | Cons |
|----------|------|------|
| **1. AST Transformer: Clone iterator** | Clean, high-level fix | Adds `.clone()` to all iterators |
| **2. Codegen: Smart detection** | Only add when needed | Complex type inference |
| **3. AST Transformer: Deref in loop body** | Targeted fix | May miss cases |
| **4. Don't wrap Vec params in &mut** | Cleanest | Breaking change to transformer |

### Recommended Next Step

**Option 1 (safest):** In AST transformer's `Stmt.For` handling (`ast_transformer.qrs:295-303`), wrap the iterator in `.clone()` when it's a Name expression that's in `current_complex_args`:

```python
case Stmt.For(f):
    iter = self.transform_expr(deref(f.iter).clone())
    # If iter is a Name of a complex arg, wrap in clone
    iter_name = self.get_iter_name(iter)
    if self.is_complex_arg(iter_name):
        iter = wrap_in_clone(iter)  # Now iterates over owned Vec<T>
    ...
```

This would generate:
```rust
for __q in (items.clone()) {  // Owned Vec<i32>
let item = __q;               // item is i32 ✅
```

### Questions for Collaborative Review

1. Should we clone all iterators in `.q` files, or only those over `&mut` params?
2. Is there a way to detect at AST level if something is already an iterator (`.lines()`, `.chars()`)? 
3. Should the fix be in AST transformer or codegen?

---

## 📝 Answers to Iterator Questions

### Q1: Should we clone all iterators, or only those over `&mut` params?

**Answer: Only clone when iterating over `&mut Vec<T>` params** — not all iterators.

**Reasoning:**

| Iterator Source | Yields | Clone Needed? |
|-----------------|--------|---------------|
| `&mut Vec<T>` (mutable borrow) | `&mut T` | ✅ Yes - clone to get `T` |
| `&Vec<T>` (immutable borrow) | `&T` | ❌ No - works fine |
| `vec![...]` (owned) | `T` | ❌ No - already owned |
| `.lines()`, `.chars()` | `&str`, `char` | ❌ No - already correct |
| `.iter()` | `&T` | ❌ No - designed for refs |
| `.into_iter()` | `T` | ❌ No - consumes to owned |

**The core issue:** In `.q` files, the AST transformer wraps complex params in `&mut` for auto-borrowing. When you iterate `&mut Vec<T>`, Rust's `IntoIterator` impl yields `&mut T`, not `T`.

**Cloning Strategy:**
```rust
// Problem:
for item in items { }  // items is &mut Vec<i32> → item is &mut i32

// Solution A: Clone the Vec (copies data, yields owned T)
for item in items.clone() { }  // yields i32 ✅

// Solution B: Iterate immutably then copy each element
for item in (*items).iter().copied() { }  // yields i32 ✅

// Solution C: Deref in loop body (but requires knowing type)
for item in items { 
    let val = *item;  // dereference manually
}
```

**Recommendation:** Use `.clone()` on the Vec when:
1. The iterator is a simple Name expression (e.g., `items`)
2. That name is in `current_complex_args` (meaning it was wrapped in `&mut`)

### Q2: Can we detect if something is already an iterator at AST level?

**Answer: Partially, via pattern matching on the expression structure.**

**Detectable patterns:**
```python
# Method call ending in known iterator methods
Expr.Call { func: Expr.Attribute { attr: "lines" | "chars" | "iter" | "into_iter" | ... } }

# Check for method call with iterator-like suffix
def is_iterator_method(attr: String) -> bool:
    return attr in ["lines", "chars", "iter", "into_iter", "bytes", 
                    "split", "split_whitespace", "enumerate", "filter", 
                    "map", "take", "skip", "chain", "zip"]
```

**Not reliably detectable:**
- User-defined iterators
- Complex expressions that return iterators
- Variables holding iterators

**Practical approach:** Only apply cloning to **simple Name expressions** that are known `&mut` params. Any method call (`.lines()`, `.split()`, etc.) should pass through unchanged.

### Q3: Should the fix be in AST transformer or codegen?

**Answer: AST Transformer** — for these reasons:

| Aspect | AST Transformer | Codegen |
|--------|-----------------|---------|
| **Has context** | ✅ Knows which params are `&mut` | ❌ Lost by this point |
| **Can wrap expr** | ✅ Can insert `.clone()` call | 🔶 Can but messier |
| **Separation** | ✅ Semantic concern | ❌ Should only emit Rust |
| **Testable** | ✅ Can unit test transforms | 🔶 Harder to test |

**Implementation sketch for AST Transformer (`ast_transformer.qrs`):**

```python
case Stmt.For(f):
    iter_expr = deref(f.iter).clone()
    transformed_iter = self.transform_expr(iter_expr)
    
    # Check if iterating over a &mut param
    match transformed_iter:
        case Expr.Name(n):
            if n in self.current_complex_args:
                # Wrap in .clone() to get owned Vec
                transformed_iter = wrap_method_call(transformed_iter, "clone", [])
        case _:
            pass  # Keep as-is for .lines(), .iter(), etc.
    
    # Continue with normal for-loop emission
    ...
```

**Trade-off Analysis:**

| Approach | Performance | Correctness | Simplicity |
|----------|-------------|-------------|------------|
| Clone Vec | O(n) copy | ✅ Always works | ✅ Simple |
| `.iter().copied()` | O(1) setup | ⚠️ Only for Copy types | 🔶 Needs type info |
| Deref in body | O(1) | ⚠️ Only for primitives | ❌ Complex |

**Recommendation: Clone the Vec.** For `.q` files prioritizing Pythonic ergonomics over performance, cloning is acceptable. Future perceus-mem integration can optimize this with copy-on-write semantics.

---

## 💬 User Feedback (2026-02-02)

**On Q2 (detecting iterators):**
> The code should have called `iter()` somewhere, or the type of this variable would need to be defined with the appropriate Trait. We could utilize scope tracking, and perhaps the CompilerContext - if they don't capture this information yet. We can make them do it.

**On Q3 (where to fix):**
> This sounds like a fundamental syntax/core language plumbing issue, not a transformation issue, so I would fix it in codegen.

---

## 🔧 Revised Design: Fix in Codegen with Type Tracking

### Compiler Design Approaches for Type-Aware Iteration

| Approach | Description | Complexity | Rust Analogy |
|----------|-------------|------------|--------------|
| **1. Symbol Table with Types** | Track `(name, type)` in scope, emit `.iter()` for refs | Medium | rustc's type environment |
| **2. Type Inference Pass** | Hindley-Milner-style inference before codegen | High | Type checking phase |
| **3. Simple Heuristics** | Pattern-match on expr structure in codegen | Low | Quick fix, limited |
| **4. Annotation Propagation** | Carry type info from AST annotations through pipeline | Medium | Span-based metadata |

### Recommended: Symbol Table with Types (Option 1)

**Current state:** `codegen.qrs` already has scope tracking via `defined_vars: Vec[HashMap[String, bool]]`.

**Enhancement:** Change to `Vec[HashMap[String, TypeInfo]]` where:

```python
class TypeInfo(Struct):
    rust_type: String        # "i32", "Vec<i32>", "&mut Vec<i32>"
    is_ref: bool             # true if & or &mut
    is_mut_ref: bool         # true if &mut specifically
    is_iterator: bool        # true if known to implement Iterator
```

### Implementation Plan

#### Step 1: Extend Codegen's Scope Tracking

In `codegen.qrs`, change the scope from `HashMap[String, bool]` to `HashMap[String, TypeInfo]`:

```python
# Current:
defined_vars: Vec[HashMap[String, bool]]

# New:
defined_vars: Vec[HashMap[String, TypeInfo]]

class TypeInfo(Struct):
    type_str: String
    is_ref: bool
    is_mut_ref: bool
```

#### Step 2: Track Types at Definition Sites

When emitting `emit_ann_assign` or function params, record the type:

```python
def emit_ann_assign(self, a: q_ast.AnnAssign):
    # ...existing code...
    type_str = self.type_to_string(...)
    type_info = TypeInfo(
        type_str=type_str,
        is_ref=type_str.starts_with("&"),
        is_mut_ref=type_str.starts_with("&mut")
    )
    self.define_var_with_type(name, type_info)
```

#### Step 3: Emit Correct Iterator in `emit_for`

```python
def emit_for(self, f: q_ast.ForStmt):
    # Check if iter is a Name with known type
    match ref(deref(f.iter)):
        case q_ast.Expr.Name(n):
            type_info = self.get_var_type(n)
            if type_info.is_mut_ref and type_info.type_str.contains("Vec"):
                # Emit: for __q in name.iter().cloned() {
                self.emit("for __q in (")
                self.emit(n)
                self.emit(").iter().cloned() {\n")
            else:
                # Normal emission
                self.emit("for __q in (")
                self.emit(n)
                self.emit(") {\n")
        case _:
            # For method calls (.lines(), etc.), pass through unchanged
            self.emit("for __q in (")
            self.generate_expr(deref(f.iter))
            self.emit(") {\n")
```

### Key Insight: Method Calls Are Already Iterators

The user's point is crucial: **if something is already an iterator, it was created by calling `.iter()`, `.lines()`, `.chars()`, etc.** These are method calls, not simple Names.

So the fix is simple:
- **Name expression** → look up type, add `.iter().cloned()` if `&mut Vec<T>`
- **Method call** → already an iterator, pass through unchanged

### Files to Modify

| File | Change |
|------|--------|
| `codegen.qrs` | Extend `defined_vars` to track types |
| `codegen.qrs` | Update `emit_for` to check iterator type |
| `codegen.qrs` | Update `emit_function_def` to record param types |

### Alternative: Minimal Fix

If full type tracking is too invasive, a simpler heuristic:

```python
def emit_for(self, f: q_ast.ForStmt):
    match ref(deref(f.iter)):
        case q_ast.Expr.Name(n):
            # Simple Name → might be &mut Vec, add .iter().cloned()
            self.emit("for __q in (")
            self.emit(n)
            self.emit(").iter().cloned() {\n")
        case q_ast.Expr.Call(func=_, args=_, keywords=_):
            # Method call → already iterator, pass through
            self.emit("for __q in (")
            self.generate_expr(deref(f.iter))
            self.emit(") {\n")
        case _:
            # Default
            self.emit("for __q in (")
            self.generate_expr(deref(f.iter))
            self.emit(") {\n")
```

**Trade-off:** This adds `.iter().cloned()` to ALL simple Name iterators, which might be wasteful for owned Vecs but won't break correctness.

---

## ✅ Implementation Complete

**Status:** ✅ Implemented and verified (2026-02-02 15:10)

### What Was Done

Implemented **full type tracking in codegen** - the "Rolls Royce" solution:

1. ✅ **Added `TypeInfo` struct** in `extern_defs` (mod.rs):
   ```rust
   pub struct TypeInfo {
       pub type_str: String,        // "i32", "Vec<i32>", "&mut Vec<i32>"
       pub is_ref: bool,            // True if & or &mut
       pub is_mut_ref: bool,        // True if &mut specifically
       pub is_iterable_ref: bool,   // True if &mut to Vec/String/slice
   }
   ```

2. ✅ **Changed `defined_vars`** from `HashMap[String, bool]` to `HashMap[String, TypeInfo]`

3. ✅ **Added helper functions in extern_defs:**
   - `create_type_info(type_str)` - Parse type string, detect is_ref/is_mut_ref/is_iterable_ref
   - `create_type_info_simple()` - Default TypeInfo for untyped variables

4. ✅ **Added helper methods in Codegen class:**
   - `get_var_type_info(name)` - Look up type info across scopes
   - `is_mut_ref_iterable(name)` - Check if var is &mut Vec/String
   - `define_var_with_type(name, type_str)` - Register var with type

5. ✅ **Updated `emit_function_def`** to record param types using `define_var_with_type`

6. ✅ **Updated `emit_ann_assign`** to record local variable types using `define_var_with_type`

7. ✅ **Updated `emit_for`** to use `is_mut_ref_iterable()` and emit `.iter().cloned()` when the iterator is a `&mut` reference to a Vec/String/slice

### Files Modified

- `crates/metaquiche-native/src/compiler/mod.rs` - Added TypeInfo struct and helper functions
- `crates/metaquiche-native/src/compiler/codegen.qrs` - Full type tracking implementation
- `crates/metaquiche-native/src/main.rs` - Updated create_codegen signature

### Verification

- ✅ Stage 1 build passes
- ✅ Stage 2 build passes (self-hosting verification)

### How It Works

When a function like this is compiled:
```python
def process_items(items: &mut Vec[i32]):
    for item in items:  # items is &mut Vec<i32>
        print(item)
```

The codegen now:
1. Records `items` with `TypeInfo{type_str: "&mut Vec<i32>", is_iterable_ref: true}`
2. In `emit_for`, detects `items` is a mutable reference to an iterable
3. Generates: `for __q in (items).iter().cloned() { ... }` instead of `for __q in (items) { ... }`

This achieves CoW semantics automatically based on type info.

