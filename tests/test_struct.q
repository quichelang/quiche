# Test struct and class system

class Point(Struct):
    x: i32
    y: i32
    
    def add(self, other: Ref[Point]) -> Point:
        return Point(x=self.x + other.x, y=self.y + other.y)
    
    def to_str(self) -> String:
        return f"({self.x}, {self.y})"

class Person(Struct):
    name: String
    age: u8
    
    def greet(self) -> String:
        return f"Hello, I'm {self.name} and I'm {self.age} years old"
    
    def is_adult(self) -> bool:
        return self.age >= 18

def main():
    # Test Point struct
    p1 = Point(x=10, y=20)
    p2 = Point(x=5, y=5)
    p3 = p1.add(ref(p2))
    print(f"Point p1: {p1.to_str()}")
    print(f"Point p2: {p2.to_str()}")
    print(f"p1 + p2 = {p3.to_str()}")
    
    # Test Person struct
    alice = Person(name="Alice", age=25)
    bob = Person(name="Bob", age=16)
    
    print(alice.greet())
    print(bob.greet())
    
    print(f"Alice is adult: {alice.is_adult()}")
    print(f"Bob is adult: {bob.is_adult()}")
