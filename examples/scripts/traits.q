from std.fmt import Display

class Point[T](Struct):
    x: T
    y: T

    def new(x: T, y: T) -> Point[T]:
        return Point(x, y)

class Printable(Trait):
    def display(self) -> String: pass

@impl(Printable)
class Point[T: Display]:
    def display(self) -> String:
        return f"Point({self.x}, {self.y})"

def main():
    p: Point[i32] = Point.new(y=6, x=5)
    print(p.display())
    
    p2: Point[i32] = Point.new(5, 6)
    print(p2.display())
    
    p3: Point[i32] = Point(y=6, x=5)
    print(p3.display())
    
    p4: Point[i32] = Point(5, 6)
    print(p4.display())