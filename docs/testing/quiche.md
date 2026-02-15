# Testing in Quiche

Quiche uses `assert` statements for testing. Test files live in `tests/` with `.q` extension.

## Quick Start

```python
# test_math.q

def test_addition():
    assert 1 + 1 == 2

def test_comparison():
    x = 42
    assert x > 0, "x should be positive"
    assert x == 42, "x should be 42"

def main():
    test_addition()
    test_comparison()
    print("All tests passed!")
```

## Conventions

- Test files: `test_*.q`
- Test functions: start with `test_`
- Use `assert` for assertions — `assert expr` or `assert expr, "message"`
- Each test file must have a `main()` that calls all test functions

## Running Tests

```bash
# Run all tests
quiche test

# Run a specific test file
quiche tests/test_math.q
```

The `quiche test` command discovers and runs all `tests/*.q` files, reporting pass/fail for each.
