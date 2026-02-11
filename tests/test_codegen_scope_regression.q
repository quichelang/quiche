# Regression test for scope leak bug ("missing let")

type Point:
    x: i32
    y: i32

type Rect:
    top_left: Point
    bottom_right: Point

def test1():
    p = Point(x=10, y=20)
    assert_eq(p.x, 10)

def extract_x(p: Point) -> i32:
    return p.x

def test6():
    r = Rect(top_left=Point(x=0, y=0), bottom_right=Point(x=10, y=5))
    assert_eq(r.bottom_right.x, 10)

def test7():
    p = Point(x=99, y=88)
    assert_eq(p.x, 99)
    x_val = extract_x(p)
    assert_eq(x_val, 99)

def test_shadow_sanity():
    x = 1
    assert_eq(x, 1)

    x = 2
    assert_eq(x, 2)

    if True:
        x = 3
        assert_eq(x, 3)

    assert_eq(x, 3)  # Python behavior: x is modified

def main():
    test1()
    test6()
    test7()
    test_shadow_sanity()
    print("Regression tests passed!")
