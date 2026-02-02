# Regression test for scope leak bug ("missing let")
# See docs/inout/perceus-metaquiche-issues.md

class Point(Struct):
    x: i32
    y: i32

class Rect(Struct):
    top_left: Point
    bottom_right: Point

def test1():
    # Defines 'p' in function scope
    p = Point(x=10, y=20)
    assert_eq(p.x, 10)

def extract_x(p: Point) -> i32:
    # Defines 'p' as parameter (shadowing if leaked)
    return p.x

def test6():
    # Single-line struct initialization (triggered the brace counting bug)
    r = Rect(top_left=Point(x=0, y=0), bottom_right=Point(x=10, y=5))
    assert_eq(r.bottom_right.x, 10)

def test7():
    # Should be a fresh variable 'p' (let mut p = ...)
    # If scope leaked, this would treat 'p' as existing and omit 'let mut'
    p = Point(x=99, y=88)
    # Use 'p' to ensure verify it was defined
    assert_eq(p.x, 99)
    x_val = extract_x(p)
    assert_eq(x_val, 99)

def test_shadow_sanity():
    # Explicit shadowing check
    x = 1
    assert_eq(x, 1)
    
    # Re-assignment
    x = 2
    assert_eq(x, 2)
    
    # Inner scope assignment (Pythonic scoping - modifies outer)
    if True:
        x = 3
        assert_eq(x, 3)
    
    assert_eq(x, 3) # Python behavior: x is modified

def main():
    test1()
    test6()
    test7()
    test_shadow_sanity()
    print("Regression tests passed!")


