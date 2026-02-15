# List Examples
# Demonstrates Quiche's chainable list type with functional operations.

def test_construction():
    # List literals
    nums = [1, 2, 3, 4, 5]
    assert nums.len() == 5

    # Empty list
    empty: List[i64] = []
    assert empty.is_empty() == True

    # Chainable push
    built = List.new().push(10).push(20).push(30)
    assert built.len() == 3
    assert built[0] == 10
    print(built)

def test_map():
    nums = [1, 2, 3, 4]

    # Double every element
    doubled = nums.map(|x| x * 2)
    assert doubled[0] == 2
    assert doubled[3] == 8
    print(doubled)

def test_filter():
    nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    # Keep only evens
    evens = nums.filter(|x| x % 2 == 0)
    assert evens.len() == 5
    assert evens[0] == 2
    assert evens[4] == 10
    print(evens)

def test_chaining():
    # The real power: chain operations together
    result = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].filter(|x| x % 2 == 0).map(|x| x * x)

    # [4, 16, 36, 64, 100]
    assert result.len() == 5
    assert result[0] == 4
    assert result[4] == 100
    print(result)

def test_concat():
    a = [1, 2, 3]
    b = [4, 5, 6]
    merged = a.concat(b)
    assert merged.len() == 6
    assert merged[5] == 6
    print(merged)

def test_vec_methods():
    # All Vec methods work through Deref
    nums = [10, 20, 30]
    assert nums.len() == 3
    assert nums[1] == 20
    assert nums.contains(20) == True
    assert nums.is_empty() == False

    # Iteration
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

    print("  Concat...")
    test_concat()

    print("  Vec methods...")
    test_vec_methods()

    print("=== List Examples Complete ===")
