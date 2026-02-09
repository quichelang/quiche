def main():
    items: Vec[i64] = [10, 20, 30, 40, 50]
    first, *rest = items
    print("first:", first)
    print("rest length:", rest.len())

    nums: Vec[i64] = [1, 2, 3, 4, 5]
    a, *middle, last = nums
    print("a:", a)
    print("last:", last)
