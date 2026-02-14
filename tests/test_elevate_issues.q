# Test Elevate Issues
#
# Each function tests one issue from docs/feedback-for-elevate.md.
# Functions that hit Elevate compile errors are commented out
# so the file can be used to test all issues that DO emit Rust.
#
# Run: quiche tests/test_elevate_issues.q
#
# To test individual issues that fail at compile time,
# uncomment the corresponding function and main() call.

# ── Issue 1: Vec String Indexing (E0507) ──────────────────────────────────
# Vec[String] subscript generates move instead of clone
def test_issue_1_vec_string_index():
    values = ["a", "b"]
    val = values[0]
    print(val)

# ── Issue 2: Integer Literal Inference ─────────────────────────────────────
# i64 literals passed to i32 params — rustc type mismatch
def add_i32(x: i32, y: i32) -> i32:
    return x + y

def test_issue_2_integer_literal_inference():
    print(add_i32(1, 2))

# ── Issue 3: Vec methods not resolved ──────────────────────────────────────
# Uncomment to reproduce: Capability resolution failed for `pop`
#
# def test_issue_3_vec_methods():
#     v = [1, 2, 3]
#     v.pop()
#     v.clear()
#     print(v.len())

# ── Issue 4: match arm closure types ───────────────────────────────────────
# Uncomment to reproduce: "must explicitly return" or closure type mismatch
#
# def classify(n: i64) -> i64:
#     match n:
#         case 1:
#             return 10
#         case 2:
#             return 20
#         case _:
#             return 0

# ── Issue 5: match arms don't satisfy return checker ──────────────────────
# Uncomment to reproduce: "must explicitly return String"
#
# def describe(n: i64) -> String:
#     match n:
#         case 1:
#             return "one"
#         case _:
#             return "other"

# ── Issue 7: Option.unwrap() not resolved ──────────────────────────────────
# Uncomment to reproduce: Capability resolution failed for `unwrap`
#
# def test_issue_7_option_unwrap():
#     x = Some(10)
#     print(x.is_some())
#     print(x.unwrap())

# ── Issue 8: HashMap method resolution ────────────────────────────────────
# Uncomment to reproduce: Capability resolution failed for `insert`
#
# def test_issue_8_hashmap_methods():
#     d = {"a": 1, "b": 2}
#     d.insert("c", 3)
#     d.remove("b")
#     print(d)

# ── Issue 9: String method resolution ─────────────────────────────────────
# Uncomment to reproduce: Capability resolution failed for `to_uppercase`
#
# def test_issue_9_string_methods():
#     s = "hello"
#     print(s.to_uppercase())

# ── Issue 10: Mutable references not inserted ─────────────────────────────
# Compiles, but silently operates on a clone — data loss!

# --- helpers ---

def push_item(nums: Vec[i64], val: i64):
    nums.push(val)

def pop_last(nums: Vec[i64]):
    nums.pop()

def sum_vec(nums: Vec[i64]) -> i64:
    result: i64 = 0
    for n in nums:
        result = result + n
    return result

def extend_with(dst: Vec[i64], src: Vec[i64]):
    for item in src:
        dst.push(item)

def clear_all(nums: Vec[i64]):
    nums.clear()

def double_values(nums: Vec[i64]):
    i: i64 = 0
    while i < nums.len():
        nums[i] = nums[i] * 2
        i = i + 1




# --- 10a: Basic mutate then read ---
def test_issue_10a_mutate_then_read():
    numbers = [1, 2, 3]
    push_item(numbers, 42)
    assert numbers.len() == 4
    assert sum_vec(numbers) == 48

# --- 10b: Multiple mutating functions on same variable ---
def test_issue_10b_multi_mutate():
    data = [10, 20, 30]
    push_item(data, 40)
    push_item(data, 50)
    pop_last(data)
    # Should be [10, 20, 30, 40]
    assert data.len() == 4
    assert sum_vec(data) == 100

# --- 10c: Mutate, read, mutate again (interleaved) ---
def test_issue_10c_interleaved():
    items = [1, 2, 3]
    push_item(items, 4)
    mid_sum = sum_vec(items)  # read: should see 1+2+3+4=10
    assert mid_sum == 10
    push_item(items, 5)
    assert items.len() == 5
    assert sum_vec(items) == 15

# --- 10d: Extend one vec from another ---
def test_issue_10d_extend():
    base = [1, 2]
    extra = [3, 4, 5]
    extend_with(base, extra)
    assert base.len() == 5
    assert sum_vec(base) == 15
    # extra should be unchanged
    assert extra.len() == 3

# --- 10e: Mutate inside a loop, read after ---
def test_issue_10e_loop_accumulate():
    result: Vec[i64] = []
    i: i64 = 0
    while i < 5:
        push_item(result, i * i)
        i = i + 1
    # Should be [0, 1, 4, 9, 16]
    assert result.len() == 5
    assert sum_vec(result) == 30

def keep_even(nums: Vec[i64]):
    # Filter in-place by rebuilding
    # Exercises: mutation inside &mut fn, then passing &mut to ANOTHER &mut fn
    temp: Vec[i64] = []
    for n in nums:
        if n % 2 == 0:
            temp.push(n)
    nums.clear()
    extend_with(nums, temp)

# --- 10f: Index assignment in-place, then read ---
# Exercises: nums[i] = expr inside a function — does the caller see the change?
def test_issue_10f_inplace_transform():
    vals = [1, 2, 3, 4]
    double_values(vals)
    # Should be [2, 4, 6, 8]
    assert sum_vec(vals) == 20
    assert vals.len() == 4

# --- 10g: Pipeline — mutate, filter, mutate again ---
# Exercises: multiple mutation functions chained on the same variable
def test_issue_10g_pipeline():
    data = [1, 2, 3, 4, 5, 6]
    double_values(data)
    # data should be [2, 4, 6, 8, 10, 12]
    keep_even(data)
    # all are even after doubling, so same length
    assert data.len() == 6
    assert sum_vec(data) == 42
    # now push more
    push_item(data, 100)
    assert data.len() == 7
    assert sum_vec(data) == 142

# --- 10h: Two writers, one reader — shared state ---
def add_front(nums: Vec[i64], val: i64):
    # Simulate prepend: push then we'll verify ordering isn't the point, length is
    nums.push(val)

def test_issue_10h_two_writers():
    shared = [0]
    push_item(shared, 1)      # writer 1
    add_front(shared, 2)      # writer 2
    push_item(shared, 3)      # writer 1 again
    assert shared.len() == 4
    total = sum_vec(shared)    # reader
    assert total == 6

# --- 10i: Clear then rebuild ---
def test_issue_10i_clear_rebuild():
    data = [1, 2, 3]
    clear_all(data)
    assert data.len() == 0
    push_item(data, 10)
    push_item(data, 20)
    assert data.len() == 2
    assert sum_vec(data) == 30

# ── Issue 16: Generic functions missing trait bounds ──────────────────────
# Compiles to Rust but rustc fails: == not applicable to type T
def check_eq[T](a: T, b: T) -> bool:
    return a == b

def test_issue_16_generic_bounds():
    print(check_eq(1, 1))

# ── Runner ────────────────────────────────────────────────────────────────

def main():
    print("=== Elevate Issue Regression Tests ===")
    print("")

    # Issues that emit Rust (may fail at rustc stage):
    print("Issue 1: Vec String Indexing")
    test_issue_1_vec_string_index()

    print("Issue 2: Integer Literal Inference")
    test_issue_2_integer_literal_inference()

    print("Issue 10a: Basic mutate then read")
    test_issue_10a_mutate_then_read()

    print("Issue 10b: Multiple mutating functions")
    test_issue_10b_multi_mutate()

    print("Issue 10c: Interleaved mutate/read")
    test_issue_10c_interleaved()

    print("Issue 10d: Extend one vec from another")
    test_issue_10d_extend()

    print("Issue 10e: Loop accumulate")
    test_issue_10e_loop_accumulate()

    print("Issue 10f: In-place index assignment transform")
    test_issue_10f_inplace_transform()

    print("Issue 10g: Pipeline (mutate, filter, mutate)")
    test_issue_10g_pipeline()

    print("Issue 10h: Two writers, one reader")
    test_issue_10h_two_writers()

    print("Issue 10i: Clear then rebuild")
    test_issue_10i_clear_rebuild()

    print("Issue 16: Generic Trait Bounds")
    test_issue_16_generic_bounds()

    print("")
    print("=== Done ===")
