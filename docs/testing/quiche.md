# Testing in Quiche

Quiche provides Python-like testing with `try/except` support (coming soon). For now, use `assert` statements.

## Quick Start

```python
# test_math.q

def test_addition():
    assert 1 + 1 == 2

def test_comparison():
    x = 42
    assert x > 0, "x should be positive"
    assert x == 42, "x should be 42"
```

## Conventions

- Test files: `test_*.q` or `*_test.q`
- Test functions: start with `test_`
- Use `assert` for assertions

## Running Tests

```bash
# Run all tests
quiche test

# Run specific file
quiche test test_math.q

# Run tests matching pattern
quiche test test_add
```

## Future: Exception-Based Testing

When `try/except` is implemented, you'll be able to:

```python
# Future Quiche syntax
def test_division_error():
    try:
        result = 1 / 0
        assert False, "should have raised"
    except ZeroDivisionError:
        pass  # Expected

def test_custom_exception():
    try:
        raise ValueError("test")
    except ValueError as e:
        assert "test" in str(e)
```

## See Also

- [MetaQuiche Testing](metaquiche.md) - Lower-level result-based testing
