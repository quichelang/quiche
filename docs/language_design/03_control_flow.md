# Quiche Language Design: Control Flow

## Enums
Quiche uses Python classes inheriting from `Enum` to define Algebraic Data Types (ADTs).

```python
class Shape(Enum):
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

### Guards
Quiche supports match guards using standard Python `if` syntax.

```python
match kind:
    case ButtonKind.Toggle(label=l, state=s) if s:
        print(f"Toggle is on: {l}")
```

## Conditionals & Loops
Standard Python syntax maps to Rust.

-   `if / elif / else` -> `if / else if / else`.
-   `while cond:` -> `while cond {}`.
-   `for x in iter:` -> `for x in iter {}`.

## Range Iteration

The `range()` function provides Python-style iteration:

```python
for i in range(10):        # 0..10
    print(i)

for i in range(5, 10):     # 5..10
    print(i)

for i in range(0, 10, 2):  # (0..10).step_by(2)
    print(i)
```

## Slice Operators

Rust-style slice syntax for subsequences:

```python
data: Vec[i32] = get_data()

# Slice expressions
subset = data[1..3]    # Elements at index 1, 2
tail = data[2..]       # From index 2 to end
head = data[..5]       # From start to index 4
copy = data[..]        # Full slice

# String slicing
text: String = "hello world"
hello = text[0..5]
world = text[6..]
```

### Generated Rust

| Quiche | Rust Output |
|--------|-------------|
| `data[1..3]` | `data[1..3]` |
| `data[2..]` | `data[2..]` |
| `data[..5]` | `data[..5]` |
| `range(10)` | `(0..10)` |
| `range(5, 10)` | `(5..10)` |
| `range(0, 10, 2)` | `((0..10).step_by(2 as usize))` |
