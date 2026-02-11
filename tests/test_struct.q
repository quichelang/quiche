# Test struct and class system

type Point:
    x: i32
    y: i32

def add_points(a: Point, b: Point) -> Point:
    return Point(x=a.x + b.x, y=a.y + b.y)

def point_to_str(p: Point) -> String:
    return f"({p.x}, {p.y})"

type Person:
    name: String
    age: u8

def greet(p: Person) -> String:
    return f"Hello, I'm {p.name} and I'm {p.age} years old"

def is_adult(p: Person) -> bool:
    return p.age >= 18

def main():
    # Test Point struct
    p1 = Point(x=10, y=20)
    p2 = Point(x=5, y=5)
    p3 = add_points(p1, p2)
    print(f"Point p1: {point_to_str(p1)}")
    print(f"Point p2: {point_to_str(p2)}")
    print(f"p1 + p2 = {point_to_str(p3)}")

    # Test Person struct
    alice = Person(name="Alice", age=25)
    bob = Person(name="Bob", age=16)

    print(greet(alice))
    print(greet(bob))

    print(f"Alice is adult: {is_adult(alice)}")
    print(f"Bob is adult: {is_adult(bob)}")
