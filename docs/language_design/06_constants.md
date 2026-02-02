# Module-Level Constants

Quiche supports compile-time constants at module level, which are translated to Rust's `const` declarations.

## Syntax

Constants can be declared in two ways:

### 1. Naming Convention (SCREAMING_SNAKE_CASE)

Any module-level variable with an ALL_UPPER_CASE name is automatically treated as a constant:

```python
MAX_SIZE: i32 = 100
BUFFER_SIZE: usize = 4096
DEFAULT_NAME: String = "unnamed"
```

Compiles to:

```rust
pub const MAX_SIZE: i32 = 100;
pub const BUFFER_SIZE: usize = 4096;
pub const DEFAULT_NAME: &str = "unnamed";
```

### 2. Explicit Type Annotation (Const[T])

Use `Const[T]` for constants that don't follow the naming convention:

```python
config_version: Const[i32] = 1
default_timeout: Const[f64] = 30.0
```

Compiles to:

```rust
pub const config_version: i32 = 1;
pub const default_timeout: f64 = 30.0;
```

## Requirements

- **Initializer required**: Constants must have a value at declaration.
- **Module level only**: Constants cannot be defined inside functions.
- **Compile-time values**: The value must be computable at compile time.

## Errors

```python
# ERROR: Constants must have an initializer
MAX_SIZE: i32

# ERROR: Constants cannot be inside functions  
def foo():
    MAX_SIZE: i32 = 100
```
