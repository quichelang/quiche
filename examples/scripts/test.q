
# Auto-borrowing test

def sum(x: Vec[i32]) -> i32:
    # Vec doesn't have reduce, use iterator
    # reduce returns Option, unwrap
    return x.iter().fold(0, lambda a, b: a + b)

def seedling_func(msg: String, num: i32) -> String:
    # [num] creates Vec
    return msg + " " + sum([num]).to_string()

def main():
    print(seedling_func("Hello", 42))