# List Examples
# Demonstrates Quiche's list type with functional operations via Enum module.

def double(x: i64) -> i64:
    return x * 2

def is_even(x: i64) -> bool:
    return x % 2 == 0

def square(x: i64) -> i64:
    return x * x

def test_construction():
    nums = [1, 2, 3, 4, 5]
    assert nums.len() == 5

def test_map():
    nums = [1, 2, 3, 4]
    doubled = nums |> Enum.map(double)
    assert doubled[0] == 2
    assert doubled[3] == 8
    print(doubled)

def test_filter():
    nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    evens = nums |> Enum.filter(is_even)
    assert evens[0] == 2
    assert evens[4] == 10
    print(evens)

def test_chaining():
    result = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] |> Enum.filter(is_even) |> Enum.map(square)
    assert result[0] == 4
    assert result[4] == 100
    print(result)

def test_vec_methods():
    nums = [10, 20, 30]
    assert nums.len() == 3
    assert nums[1] == 20
    assert nums.contains(20) == True
    assert nums.is_empty() == False

    total: i64 = 0
    for n in nums:
        total = total + n
    assert total == 60
    print(total)

def main():
    print("=== List Examples ===")

    print("  Construction...")
    test_construction()

    print("  Map...")
    test_map()

    print("  Filter...")
    test_filter()

    print("  Chaining...")
    test_chaining()

    print("  Vec methods...")
    test_vec_methods()

    print("=== List Examples Complete ===")
