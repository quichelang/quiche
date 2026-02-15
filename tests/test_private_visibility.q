# Test Private Visibility via Underscore Convention

type _PrivateHelper:
    value: i32
    _tag: str

def new_private_helper(v: i32) -> _PrivateHelper:
    return _PrivateHelper(value=v, _tag="helper")

def get_helper_value(h: _PrivateHelper) -> i32:
    return h.value

# Public struct with private and public fields
type Container:
    name: str
    _internal: i32
    _cache: str

def new_container(name: str, internal: i32) -> Container:
    return Container(name=name, _internal=internal, _cache="")

def compute_container(c: Container) -> i32:
    return c._internal * 2

def get_internal(c: Container) -> i32:
    return c._internal

def test_private_struct():
    print("Running test_private_struct...")
    helper = new_private_helper(42)
    assert get_helper_value(helper) == 42
    print("PASS: test_private_struct")

def test_private_fields():
    print("Running test_private_fields...")
    c = new_container("test", 10)
    assert c.name == "test"
    assert get_internal(c) == 10
    print("PASS: test_private_fields")

def test_computed():
    print("Running test_computed...")
    c = new_container("test", 5)
    assert compute_container(c) == 10
    print("PASS: test_computed")

type PointPV:
    x: i32
    y: i32
    _label: str

def new_point(x: i32, y: i32) -> PointPV:
    return PointPV(x=x, y=y, _label="")

def labeled_point(x: i32, y: i32, label: str) -> PointPV:
    return PointPV(x=x, y=y, _label=label)

def get_label(p: PointPV) -> str:
    return p._label

def test_struct_private_fields():
    print("Running test_struct_private_fields...")
    p = new_point(1, 2)
    assert p.x == 1
    assert p.y == 2

    p2 = labeled_point(3, 4, "origin")
    assert get_label(p2) == "origin"
    print("PASS: test_struct_private_fields")

def main():
    print("=== Private Visibility Tests ===")
    test_private_struct()
    test_private_fields()
    test_computed()
    test_struct_private_fields()
    print("=== All Private Visibility Tests Passed ===")
