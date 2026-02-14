# Test for reference type annotations
# In .q, auto-borrowing handles ref/mutref automatically.

def sum_list(items: Vec[i64]) -> i64:
    total = 0
    for item in items:
        total = total + item
    return total

def test_ref_types():
    numbers = [1, 2, 3]
    total = sum_list(numbers)
    assert total == 6

    print("test_ref_types: PASSED")

def main():
    test_ref_types()
