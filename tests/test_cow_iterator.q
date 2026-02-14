# Tests for Copy-on-Write (CoW) iterator semantics
# In .q, auto-borrowing handles ref/mutref automatically.

def sum_items(items: Vec[i64]) -> i64:
    total = 0
    for item in items:
        total = total + item
    return total

def double_values(items: Vec[i64]) -> Vec[i64]:
    result: Vec[i64] = []
    for item in items:
        result.push(item * 2)
    return result

def count_matching(items: Vec[i64], target: i64) -> i64:
    count = 0
    for item in items:
        if item == target:
            count = count + 1
    return count

def test_cow_iterator_sum():
    numbers = [1, 2, 3, 4, 5]
    result = sum_items(numbers)
    assert result == 15
    print("test_cow_iterator_sum: PASSED")

def test_cow_iterator_transform():
    numbers = [10, 20, 30]
    doubled = double_values(numbers)
    assert doubled.len() == 3
    assert doubled[0] == 20
    assert doubled[1] == 40
    assert doubled[2] == 60
    print("test_cow_iterator_transform: PASSED")

def test_cow_iterator_count():
    numbers = [1, 2, 2, 3, 2, 4]
    count = count_matching(numbers, 2)
    assert count == 3
    print("test_cow_iterator_count: PASSED")

def test_nested_cow_iteration():
    outer = [1, 2, 3]
    inner = [10, 20]

    outer_sum = sum_items(outer)
    inner_sum = sum_items(inner)
    total = outer_sum * inner_sum

    # (1+2+3) * (10+20) = 6 * 30 = 180
    assert total == 180
    print("test_nested_cow_iteration: PASSED")

def main():
    print("=== CoW Iterator Semantics Tests ===")
    test_cow_iterator_sum()
    test_cow_iterator_transform()
    test_cow_iterator_count()
    test_nested_cow_iteration()
    print("=== All CoW Iterator Tests Passed ===")
