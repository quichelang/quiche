# Quiche Language Design: Control Flow

## Enums
Quiche introduces the `enum` keyword to define Algebraic Data Types (ADTs), similar to Rust.

```python
enum Shape:
    Circle(radius: f64)
    Rectangle(width: f64, height: f64)
    Point  # Unit variant
```

## Pattern Matching
Quiche uses Python's `match` / `case` syntax but enforces **Strict Rust Semantics**.

### Rules
1.  **Exhaustiveness**: All cases must be handled. Missing variants cause a compile-time error.
2.  **Structural**: Matches structure (Enums, Statics), not runtime types.
3.  **Binding**: Variables in patterns bind to values.

### Examples

**Matching Enums**:
```python
match s:
    case Shape.Circle(r):
        print(r)
    case Shape.Rectangle(w, h):
        print(w * h)
    case Shape.Point:
        print("Point")
```

**Matching Literals**:
```python
match x:
    case 0:
        print("Zero")
    case _:
        print("Other")
```

**Matching Strings**:
```python
# String matching works against String or &str
match name:
    case "Alice":
        print("Hello Alice")
    case String("Bob"): # Explicit constructor match
        print("Hello Bob") 
```

## Conditionals & Loops
Standard Python syntax maps to Rust.

-   `if / elif / else` -> `if / else if / else`.
-   `while cond:` -> `while cond {}`.
-   `for x in iter:` -> `for x in iter {}`.
