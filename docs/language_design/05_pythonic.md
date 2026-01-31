# Quiche Language Design: Pythonic Features

Quiche bridges Python's ergonomics with Rust's performance by transforming idiomatic Python patterns into efficient Rust code.

## Built-in Functions

### `len(x)`
Returns the length of a collection or string.

```python
v = [1, 2, 3]
print(len(v))  # 3

s = "hello"
print(len(s))  # 5
```

**Compiles to:** `x.len()`

## Negative Indexing

Access elements from the end of a sequence using negative indices.

```python
v = [1, 2, 3, 4, 5]
print(v[-1])   # 5 (last element)
print(v[-2])   # 4 (second to last)
```

**Compiles to:** `v[v.len() - n]`

## F-Strings (String Interpolation)

Embed expressions inside string literals using `f"..."`.

```python
name = "World"
msg = f"Hello {name}, 2+2={2+2}"
```

**Compiles to:** `format!("Hello {}, 2+2={}", name, 2+2)`

## Method Aliasing

| Quiche Method | Rust Method |
|---------------|-------------|
| `vec.append(x)` | `vec.push(x)` |
| `vec.pop()` | `vec.pop()` |
| `vec.insert(i, x)` | `vec.insert(i, x)` |
| `vec.clear()` | `vec.clear()` |
| `map.get(k)` | `map.get(k)` |
| `map.keys()` | `map.keys()` |

## Slicing (Planned)

```python
v[1:3]   # -> &v[1..3]
v[::2]   # -> v.iter().step_by(2)
```
