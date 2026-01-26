# Rust Interop Design

**Goal**: Allow Quiche code to call arbitrary Rust functions and use Rust types seamlessly.

## The Problem
Quiche compiles to Rust. calling `std::fs::read_dir(s)` fails if `s` is a Quiche `String` but Rust expects `&str` or `AsRef<Path>`. We need a bridge.

## Solution

We propose a two-tier system:
1. **Precise Bindings** (`@extern`): For standard library and core types.
2. **Automatic Bindings** (`rust.*` namespace): For 3rd party crates using a "Bridge Macro".

---

### Method 1: Precise Bindings (`@extern`)

Use this for the standard library (`std`) or highly-used APIs where performance and exact types matter.

```python
# std/fs.qrs
@extern(path="std::fs::read_dir")
def read_dir(path: String) -> Result[ReadDir, Error]:
    pass
```

The compiler replaces the body with a re-export: `pub use std::fs::read_dir;`.
*Note*: This requires the user to ensure the signatures match exactly or rely on Rust's type inference.

---

### Method 2: Automatic Bindings (The Bridge Macro)

For arbitrary Rust crates (e.g., `serde`, `rand`), we avoid writing manual definitions.

**Syntax:**
Use the `rust` generic namespace to import from any crate available in `Cargo.toml`.

```python
from rust.serde_json import from_str
from rust.rand import random

def main():
    # Helper macros will handle type conversion automatically!
    data = from_str("{\"k\": 1}") 
```

**Compiler Behavior:**
1. Compiler sees `from rust...`.
2. Flags `from_str` as a **Foreign Symbol**.
3. When `from_str(...)` is called, the transpiler emits a macro call instead of a function call.

**Transpilation:**

*Quiche Source:*
```python
from rust.serde_json import from_str
x = from_str(my_string)
```

*Generated Rust:*
```rust
// The import is just a hint to the compiler, or we can generate a 'use'
// But the real magic is at the call site:
let x = quiche::call!(serde_json::from_str, my_string);
```

**The `quiche::call!` Macro:**
This Rust macro handles the "Heavy Lifting" at compile time:
1. **Into Conversion**: Calls `.into()` or `.as_ref()` on arguments to match expected types.
2. **Return Wrapping**: Wraps the result back into a Quiche-compatible type if needed.

```rust
macro_rules! call {
    ($func:path, $($arg:expr),*) => {
        // Pseudo-code logic
        $func( $($arg.into()),* )
    }
}
```

This effectively allows "Duck Typing" interactions with Rust libraries where the "Duck" is actually a sophisticated Rust Trait system.

---

## Path Forward

1.  **Compiler Update**:
    *   Detect `from rust.<crate> import ...`.
    *   Track these symbols in the symbol table as `Foreign`.
    *   Codegen: Transpile calls to `Foreign` symbols as `quiche::call!(path, args...)`.
2.  **Runtime Library (`quiche-runtime`)**:
    *   Implement `call!` macro.
    *   Impl necessary traits (`From<String>` for `PathBuf`, etc.)

## Risks
*   **Ambiguity**: If a Rust function has multiple overloads or complex trait bounds, `into()` might not be enough.
*   **Macros are Opaque**: Error messages might be cryptic ("Error in macro expansion").
