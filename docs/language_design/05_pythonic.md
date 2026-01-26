# Quiche Language Design: Pythonic Features

Quiche bridges Python's ergonomics with Rust's performance by transforming idiomatic Python patterns into efficient Rust code.

## Built-in Functions

### `len(x)`
Returns the length of a collection or string.

```python
v: Vec[i32] = [1, 2, 3]
print(len(v))  # 3

s: String = "hello"
print(len(s))  # 5
```

**Compiles to:** `x.len()`

## Negative Indexing

Access elements from the end of a sequence using negative indices.

```python
v: Vec[i32] = [1, 2, 3, 4, 5]
print(v[-1])   # 5 (last element)
print(v[-2])   # 4 (second to last)
```

**Compiles to:** `v[v.len() - n]`

## Method Aliasing (Planned)

| Python Method | Rust Method |
|---------------|-------------|
| `list.append(x)` | `vec.push(x)` |
| `list.pop()` | `vec.pop()` |
| `str.upper()` | `str.to_uppercase()` |

## Slicing (Planned)

```python
v[1:3]   # -> &v[1..3]
v[::2]   # -> v.iter().step_by(2)
```
