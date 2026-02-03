# List comprehensions and lambdas
def main():
    nums = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]
    
    # Rust-style lambda syntax
    add = |x: i32, y: i32| x + y
    print("Sum: " + add(2, 3))
    
    # Pythonic len()
    print("Length: " + len(doubled))
