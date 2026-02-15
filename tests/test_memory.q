# Memory Management Comprehensive Tests
#
# Bug-hunting tests for memory system with multi-level objects,
# cross-function value passing, and complex allocation patterns.

# =============================================================================
# DATA STRUCTURES FOR TESTING
# =============================================================================

type Point:
    x: i64
    y: i64

type Rect:
    top_left: Point
    bottom_right: Point

type Node:
    value: i64
    name: str

type TreeNode:
    value: i64
    left: Option[Box[TreeNode]]
    right: Option[Box[TreeNode]]

type Container:
    items: Vec[i64]
    name: str

type DeepNested:
    container: Container
    point: Point

# =============================================================================
# BASIC VALUE TESTS
# =============================================================================

def test_primitive_copy():
    x = 42
    y = x
    assert x == 42, "original should remain"
    assert y == 42, "copy should match"

def test_bool_operations():
    a = True
    b = False
    assert a == True
    assert b == False
    assert a != b
    assert (a and b) == False
    assert (a or b) == True

def test_arithmetic():
    assert 1 + 2 == 3
    assert 10 - 3 == 7
    assert 4 * 5 == 20
    assert 10 / 2 == 5
    assert 17 % 5 == 2

def test_primitive_mutation():
    """
    Test that mutation of a variable does not affect other variables, primitives are copied by default
    """
    x = 42
    y = x
    x = x + 1
    assert x == 43
    assert y == 42

def test_vector_mutation():
    """
    Test that mutation of a vector DOES affect other vectors (which are copies)
    """
    v = [1, 2, 3]
    u = v
    v.push(4)
    assert v == [1, 2, 3, 4]
    assert u == [1, 2, 3, 4]


# =============================================================================
# STRUCT CREATION AND FIELD ACCESS  
# =============================================================================

def test_struct_creation():
    p = Point(x=10, y=20)
    assert p.x == 10
    assert p.y == 20

def test_nested_struct():
    r = Rect(
        top_left=Point(x=0, y=0),
        bottom_right=Point(x=100, y=100)
    )
    assert r.top_left.x == 0
    assert r.top_left.y == 0
    assert r.bottom_right.x == 100
    assert r.bottom_right.y == 100

def test_struct_with_string():
    n = Node(value=42, name="test")
    assert n.value == 42
    assert n.name == "test"

def test_container_with_vec():
    c = Container(items=[1, 2, 3], name="nums")
    assert c.items.len() == 3
    assert c.items[0] == 1
    assert c.items[1] == 2
    assert c.items[2] == 3
    assert c.name == "nums"

# =============================================================================
# FUNCTION PARAMETER PASSING
# =============================================================================

def add_points(a: Point, b: Point) -> Point:
    return Point(x=a.x + b.x, y=a.y + b.y)

def test_struct_to_function():
    p1 = Point(x=1, y=2)
    p2 = Point(x=3, y=4)
    result = add_points(p1, p2)
    assert result.x == 4
    assert result.y == 6

def get_rect_area(r: Rect) -> i64:
    w = r.bottom_right.x - r.top_left.x
    h = r.bottom_right.y - r.top_left.y
    return w * h

def test_nested_struct_to_function():
    r = Rect(
        top_left=Point(x=0, y=0),
        bottom_right=Point(x=10, y=5)
    )
    area = get_rect_area(r)
    assert area == 50

def extract_x(p: Point) -> i64:
    return p.x

def test_struct_field_extraction():
    p = Point(x=99, y=88)
    x_val = extract_x(p)
    assert x_val == 99

# =============================================================================
# MULTI-LEVEL FUNCTION CALLS
# =============================================================================

def level3(val: i64) -> i64:
    return val * 2

def level2(val: i64) -> i64:
    return level3(val) + 1

def level1(val: i64) -> i64:
    return level2(val) * 3

def test_multi_level_calls():
    # level1(5) -> level2(5)*3 -> (level3(5)+1)*3 -> (10+1)*3 = 33
    result = level1(5)
    assert result == 33

def pass_point_3_levels(p: Point) -> Point:
    return transform_point_2(p)

def transform_point_2(p: Point) -> Point:
    return transform_point_1(p)

def transform_point_1(p: Point) -> Point:
    return Point(x=p.x * 2, y=p.y * 2)

def test_struct_multi_level_passing():
    p = Point(x=5, y=10)
    result = pass_point_3_levels(p)
    assert result.x == 10
    assert result.y == 20

# =============================================================================
# VECTOR OPERATIONS
# =============================================================================

def sum_vec(items: Vec[i64]) -> i64:
    total = 0
    for item in items:
        total = total + item
    return total

def test_vec_to_function():
    v = [1, 2, 3, 4, 5]
    result = sum_vec(v)
    assert result == 15

def double_each(items: Vec[i64]) -> Vec[i64]:
    result: Vec[i64] = []
    for item in items:
        result.push(item * 2)
    return result

def test_vec_return():
    v = [1, 2, 3]
    doubled = double_each(v)
    assert doubled.len() == 3
    assert doubled[0] == 2
    assert doubled[1] == 4
    assert doubled[2] == 6

def test_vec_push_pop():
    v: Vec[i64] = []
    v.push(10)
    v.push(20)
    v.push(30)
    assert v.len() == 3
    last = v.pop().unwrap()
    assert last == 30
    assert v.len() == 2

def test_vec_of_structs():
    points: Vec[Point] = []
    points.push(Point(x=1, y=1))
    points.push(Point(x=2, y=2))
    points.push(Point(x=3, y=3))
    
    assert points.len() == 3
    assert points[0].x == 1
    assert points[1].x == 2
    assert points[2].x == 3

# =============================================================================
# NESTED CONTAINER OPERATIONS
# =============================================================================

def create_deep_nested() -> DeepNested:
    return DeepNested(
        container=Container(items=[10, 20, 30], name="deep".to_string()),
        point=Point(x=5, y=5)
    )

def test_deep_nested_creation():
    dn = create_deep_nested()
    assert dn.container.name == "deep".to_string()
    assert dn.container.items.len() == 3
    assert dn.container.items[0] == 10
    assert dn.point.x == 5

def modify_deep_nested_items(dn: DeepNested) -> DeepNested:
    new_items: Vec[i64] = []
    for item in dn.container.items:
        new_items.push(item + 1)
    return DeepNested(
        container=Container(items=new_items, name=dn.container.name),
        point=dn.point
    )

def test_deep_nested_modification():
    dn = create_deep_nested()
    modified = modify_deep_nested_items(dn)
    assert modified.container.items[0] == 11
    assert modified.container.items[1] == 21
    assert modified.container.items[2] == 31

# =============================================================================
# OPTION HANDLING
# =============================================================================

def find_in_vec(items: Vec[i64], target: i64) -> Option[i64]:
    for item in items:
        if item == target:
            return Some(item)
    return None

def test_option_some():
    v = [1, 2, 3, 4, 5]
    result = find_in_vec(v, 3)
    match result:
        case Some(val):
            assert val == 3
        case None:
            assert False, "should have found 3"

def test_option_none():
    v = [1, 2, 3]
    result = find_in_vec(v, 99)
    match result:
        case Some(_):
            assert False, "should not find 99"
        case None:
            pass  # Expected

def safe_div(a: i64, b: i64) -> Option[i64]:
    if b == 0:
        return None
    return Some(a / b)

def test_option_chain():
    r1 = safe_div(10, 2)
    match r1:
        case Some(v):
            assert v == 5
        case None:
            assert False
    
    r2 = safe_div(10, 0)
    match r2:
        case Some(_):
            assert False, "division by zero should be None"
        case None:
            pass  # Expected

# =============================================================================
# CLOSURE / LAMBDA TESTS (if supported)
# =============================================================================

def test_fold_sum():
    v = [1, 2, 3, 4, 5]
    result = v.iter().fold(0, |acc, x| acc + x)
    assert result == 15

def test_map_operation():
    v = [1, 2, 3]
    doubled = v.iter().map(|x| x * 2).collect()
    assert doubled[0] == 2
    assert doubled[1] == 4
    assert doubled[2] == 6

def test_filter_operation():
    v = [1, 2, 3, 4, 5, 6]
    evens = v.iter().filter(|x| x % 2 == 0).collect()
    assert evens.len() == 3
    assert evens[0] == 2
    assert evens[1] == 4
    assert evens[2] == 6

# =============================================================================
# STRING OPERATIONS
# =============================================================================

def test_string_concat():
    a = "Hello".to_string()
    b = " World".to_string()
    result = a + b
    assert result == "Hello World".to_string()

def test_string_in_struct():
    n = Node(value=1, name="first".to_string())
    assert n.name.len() == 5

def concat_names(nodes: Vec[Node]) -> String:
    result = "".to_string()
    for node in nodes:
        result = result + node.name + ",".to_string()
    return result

def test_string_accumulation():
    nodes: Vec[Node] = []
    nodes.push(Node(value=1, name="a".to_string()))
    nodes.push(Node(value=2, name="b".to_string()))
    nodes.push(Node(value=3, name="c".to_string()))
    result = concat_names(nodes)
    assert result == "a,b,c,".to_string()

# # =============================================================================
# # COMPLEX RETURN PATTERNS
# # =============================================================================

# def create_many_points(count: i64) -> Vec[Point]:
#     result: Vec[Point] = []
#     i = 0
#     while i < count:
#         result.push(Point(x=i, y=i * 2))
#         i = i + 1
#     return result

# def test_vec_of_structs_return():
#     points = create_many_points(5)
#     assert points.len() == 5
#     assert points[0].x == 0
#     assert points[0].y == 0
#     assert points[4].x == 4
#     assert points[4].y == 8

# def transform_all_points(points: Vec[Point]) -> Vec[Point]:
#     result: Vec[Point] = []
#     for p in points:
#         result.push(Point(x=p.x + 10, y=p.y + 10))
#     return result

# def test_vec_struct_transformation():
#     original = create_many_points(3)
#     transformed = transform_all_points(original)
#     assert transformed[0].x == 10
#     assert transformed[0].y == 10
#     assert transformed[1].x == 11
#     assert transformed[1].y == 12
#     assert transformed[2].x == 12
#     assert transformed[2].y == 14

# # =============================================================================
# # EDGE CASES AND STRESS TESTS
# # =============================================================================

# def test_empty_vec():
#     v: Vec[i64] = []
#     assert v.len() == 0
#     assert v.is_empty()

# def test_single_element_vec():
#     v = [42]
#     assert v.len() == 1
#     assert v[0] == 42

# def test_large_vec():
#     v: Vec[i64] = []
#     i = 0
#     while i < 1000:
#         v.push(i)
#         i = i + 1
#     assert v.len() == 1000
#     assert v[0] == 0
#     assert v[999] == 999

# def test_many_small_structs():
#     points: Vec[Point] = []
#     i = 0
#     while i < 100:
#         points.push(Point(x=i, y=i))
#         i = i + 1
#     # Verify first and last
#     assert points[0].x == 0
#     assert points[99].x == 99

# def deeply_nested_call_1(x: i64) -> i64:
#     return deeply_nested_call_2(x + 1)

# def deeply_nested_call_2(x: i64) -> i64:
#     return deeply_nested_call_3(x + 1)

# def deeply_nested_call_3(x: i64) -> i64:
#     return deeply_nested_call_4(x + 1)

# def deeply_nested_call_4(x: i64) -> i64:
#     return deeply_nested_call_5(x + 1)

# def deeply_nested_call_5(x: i64) -> i64:
#     return x + 1

# def test_deeply_nested_calls():
#     # 0 + 1 + 1 + 1 + 1 + 1 = 5
#     result = deeply_nested_call_1(0)
#     assert result == 5

# # =============================================================================
# # STRUCT MUTATIONS (create modified copies)
# # =============================================================================

# def move_point(p: Point, dx: i64, dy: i64) -> Point:
#     return Point(x=p.x + dx, y=p.y + dy)

# def test_struct_functional_update():
#     p1 = Point(x=10, y=20)
#     p2 = move_point(p1, 5, -5)
#     assert p2.x == 15
#     assert p2.y == 15

# def expand_rect(r: Rect, amount: i64) -> Rect:
#     return Rect(
#         top_left=Point(x=r.top_left.x - amount, y=r.top_left.y - amount),
#         bottom_right=Point(x=r.bottom_right.x + amount, y=r.bottom_right.y + amount)
#     )

# def test_nested_struct_update():
#     r = Rect(
#         top_left=Point(x=10, y=10),
#         bottom_right=Point(x=20, y=20)
#     )
#     expanded = expand_rect(r, 5)
#     assert expanded.top_left.x == 5
#     assert expanded.top_left.y == 5
#     assert expanded.bottom_right.x == 25
#     assert expanded.bottom_right.y == 25

# # =============================================================================
# # RECURSIVE PATTERNS (if supported)
# # =============================================================================

# def factorial(n: i64) -> i64:
#     if n <= 1:
#         return 1
#     return n * factorial(n - 1)

# def test_recursion():
#     assert factorial(0) == 1
#     assert factorial(1) == 1
#     assert factorial(5) == 120

# def fibonacci(n: i64) -> i64:
#     if n <= 1:
#         return n
#     return fibonacci(n - 1) + fibonacci(n - 2)

# def test_fibonacci():
#     assert fibonacci(0) == 0
#     assert fibonacci(1) == 1
#     assert fibonacci(10) == 55

# # =============================================================================
# # ALIASING AND REFERENCE PATTERNS  
# # =============================================================================

# def test_multiple_refs_to_struct():
#     p = Point(x=10, y=20)
#     r1 = p
#     r2 = p
#     # All should be separate copies (or properly aliased)
#     assert r1.x == 10
#     assert r2.x == 10

# def test_struct_in_vec_access():
#     points = [Point(x=1, y=2), Point(x=3, y=4)]
#     # Access via index
#     first = points[0]
#     assert first.x == 1
#     second = points[1]
#     assert second.x == 3

# # =============================================================================
# # COMPLEX OWNERSHIP PATTERNS
# # =============================================================================

# def consume_and_return(p: Point) -> Point:
#     # Takes ownership, returns new
#     return Point(x=p.x * 2, y=p.y * 2)

# def test_ownership_chain():
#     p1 = Point(x=1, y=1)
#     p2 = consume_and_return(p1)
#     p3 = consume_and_return(p2)
#     p4 = consume_and_return(p3)
#     assert p4.x == 8  # 1 * 2 * 2 * 2
#     assert p4.y == 8

# def build_container() -> Container:
#     items: Vec[i64] = []
#     items.push(1)
#     items.push(2)
#     items.push(3)
#     return Container(items=items, name="built")

# def test_container_builder():
#     c = build_container()
#     assert c.items.len() == 3
#     assert c.name == "built"

# def test_chained_container_ops():
#     c1 = build_container()
#     # Create a new container with modified items
#     new_items: Vec[i64] = []
#     for item in c1.items:
#         new_items.push(item * 10)
#     c2 = Container(items=new_items, name="modified")
#     assert c2.items[0] == 10
#     assert c2.items[1] == 20
#     assert c2.items[2] == 30

# # =============================================================================  
# # PRINT TESTS (for debugging visibility)
# # =============================================================================

# def test_print_works():
#     print("Test output visible")
#     assert True

# def test_debug_struct():
#     p = Point(x=42, y=99)
#     print("Point created with x=42, y=99")
#     assert p.x == 42

def main():
    test_primitive_copy()
    test_bool_operations()
    test_arithmetic()
    test_primitive_mutation()

    test_struct_to_function()
    test_nested_struct_to_function()
    test_struct_field_extraction()

    test_multi_level_calls()
    test_struct_multi_level_passing()

    test_vec_to_function()
    test_vec_return()
    test_vec_push_pop()
    test_vec_of_structs()

    test_deep_nested_creation()
    test_deep_nested_modification()

    test_option_some()
    test_option_none()
    test_option_chain()

    test_string_concat()
    test_string_in_struct()
    test_string_accumulation()

    test_fold_sum()
    test_map_operation()
    test_filter_operation()