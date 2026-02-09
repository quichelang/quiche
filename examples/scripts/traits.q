class Point(Struct):
    x: i32
    y: i32

    def new(x: i32, y: i32) -> Point:
        return Point(x, y)

class Printable(Trait):
    def display(self) -> String: pass

@impl(Printable)
class Point:
    def display(self) -> String:
        return f"Point({self.x}, {self.y})"

def main():
    p: Point = Point.new(y=6, x=5)
    print(p.display())
    
    p2: Point = Point.new(5, 6)
    print(p2.display())
    
    p3: Point = Point(y=6, x=5)
    print(p3.display())
    
    p4: Point = Point(5, 6)
    print(p4.display())