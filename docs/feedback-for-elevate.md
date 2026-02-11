# Feedback for Elevate — Remaining Issues (v0.8.7)

Issues that remain open after updating to Elevate v0.8.7 (`d61350d9`). Previous issues #1, #2, #7, #9, and #16 are now fixed.

**Reproduction test file**: [`tests/test_elevate_issues.q`](file:///Volumes/Dev/code/jagtesh/quiche/tests/test_elevate_issues.q)

---

## 1. Mutating Variables Not Declared `mut` (E0596)

**Severity: High** — affects Vec, HashMap, and any type with mutating methods.

Vec `.pop()` and `.clear()` now resolve correctly (thanks to rustdex) but the generated Rust doesn't declare the variable as `mut`:

```python
def main():
    v = [1, 2, 3]
    v.pop()
    v.clear()
    print(v.len())
```

Emitted Rust (`quiche --emit-rust`):

```rust
pub fn main() -> () {
    let v: Vec<i64> = vec![1, 2, 3];  // ← missing `mut`
    v.pop();    // ← E0596: cannot borrow `v` as mutable
    v.clear();
    println!("{:?}", v.len());
}
```

`rustc` error:
```
error[E0596]: cannot borrow `v` as mutable, as it is not declared as mutable
```

Same problem with HashMap:

```python
def main():
    d = {"a": 1, "b": 2}
    d.insert("c", 3)
    d.remove("b")
    print(d)
```

Emitted Rust:

```rust
pub fn main() -> () {
    let d: HashMap<String, i64> = HashMap::from_iter(
        vec![("a", 1), ("b", 2)].into_iter()
    );  // ← missing `mut`
    d.insert(String::from("c"), 3);  // ← E0596
    d.remove(&String::from("b"));
    println!("{:?}", d);
}
```

**Expected**: Elevate should detect that `.pop()`, `.clear()`, `.insert()`, `.remove()` take `&mut self` and automatically emit `let mut`. This is the same heuristic it already applies to function parameters (where it correctly generates `mut nums: Vec<i64>`) — it just doesn't apply it to local variable declarations.

---

## 2. Function Arguments Cloned Instead of Passed by Mutable Reference

**Severity: P0** — silent data loss. Mutations on function arguments are silently discarded.

```python
def modify_list(nums: Vec[i64]):
    nums.push(42)

def main():
    numbers = [1, 2, 3]
    modify_list(numbers)
    print(numbers.len())    # Prints 3, not 4
```

Emitted Rust (`quiche --emit-rust`):

```rust
// ownership-note: auto-clone inserted in `main` for `numbers` of type `Vec<i64>`

pub fn modify_list(mut nums: Vec<i64>) -> () {
    nums.push(42);
}

pub fn main() -> () {
    let numbers: Vec<i64> = vec![1, 2, 3];
    modify_list(numbers.clone());  // ← clone! caller's `numbers` is unchanged
    println!("{:?}", numbers.len());  // prints 3
}
```

The compiler even admits it: `auto-clone inserted`. The function receives an owned clone, mutates it, and drops it. The caller's `numbers` is unchanged.

**Expected**: This should generate `modify_list(&mut numbers)` with the function signature `fn modify_list(nums: &mut Vec<i64>)`. The fact that Elevate already marks the parameter as `mut` proves it knows mutation happens — it just doesn't propagate this knowledge to the call site.

**Impact**: Any function that modifies its argument (push, insert, field assignment on a struct) silently operates on a copy. This is the single most dangerous bug because it compiles and runs without error but produces wrong results.

---

## 3. `match` Arms Don't Satisfy Return Type Checker

**Severity: Medium** — match-with-return is a common pattern. Two sub-issues:

### 3a. Return checker rejects exhaustive match

```python
def classify(n: i64) -> i64:
    match n:
        case 1:
            return 10
        case 2:
            return 20
        case _:
            return 0

def main():
    print(classify(1))
```

Elevate error (no Rust emitted):
```
Compile error:
Function `classify` must explicitly return `i64`
```

The compiler doesn't recognize that all match arms return, so it insists the function is missing a return. This happens even with a wildcard `_` arm that covers all remaining cases.

### 3b. Same for String returns

```python
def describe(n: i64) -> String:
    match n:
        case 1:
            return "one"
        case _:
            return "other"

def main():
    print(describe(1))
```

Elevate error:
```
Compile error:
Function `describe` must explicitly return `String`
```

**Expected**: The match construct with a wildcard `_` arm is exhaustive. The return checker should recognize that every arm returns and consider the function's return obligation satisfied.

---

## Summary of Root Causes

These remaining issues boil down to **two root causes**:

### Root Cause A: Missing `mut` inference (Issues 1, 2)

Elevate knows when mutation happens (it correctly marks function params as `mut`) but doesn't apply this knowledge consistently:

| Context | `mut` inserted? | Status |
|---------|:---:|---|
| Function parameter receiving mutating call | ✅ `mut nums` | Works |
| Local variable receiving mutating call | ❌ `let v` | **Broken** (E0596) |
| Call-site argument for mutating function | ❌ `.clone()` | **Broken** (silent data loss) |

### Root Cause B: Match exhaustiveness not checked (Issue 3)

The return checker treats `match` as a single statement rather than analyzing branch coverage. A match with all arms returning should satisfy the return obligation.

---

## Priority Ranking

| Priority | Issue | Impact |
|----------|-------|--------|
| **P0** | #2 Clone instead of `&mut` at call sites | Silent data loss |
| **P1** | #1 Missing `let mut` for local variables | E0596 on any mutating method |
| **P1** | #3 Match return checker | Common pattern rejected |

---

## Previously Fixed (v0.8.7)

For the record, these issues from the original dogfooding session are now resolved:

| Issue | What was fixed |
|-------|---------------|
| Vec String indexing (E0507) | `values[0]` now generates `.clone()` for non-Copy types |
| Integer literal inference | Literals match explicit param types (`i32`, `f64`) |
| `Option.unwrap()` | Method resolution via rustdex |
| `String.to_uppercase()` | Method resolution via rustdex |
| Generic trait bounds | `fn check_eq<T: PartialEq>` now generated |
