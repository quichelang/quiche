def main():
    # 1. List comprehension (no filter)
    nums: Vec[i64] = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]
    print("doubled length:", doubled.len())

    # 2. List comprehension with filter
    source: Vec[i64] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    big = [x for x in source if x > 5]
    print("items > 5, count:", big.len())

    # 3. List comp with expression + filter
    vals: Vec[i64] = [1, 2, 3, 4, 5, 6]
    evens_doubled = [x * 2 for x in vals if x % 2 == 0]
    print("even*2 count:", evens_doubled.len())

    # 4. Dict comprehension (i64 keys — no ownership issues)
    items: Vec[i64] = [1, 2, 3, 4, 5]
    d = {x: x * 10 for x in items}
    print("dict size:", d.len())
