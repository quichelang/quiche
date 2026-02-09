from std.string import ToString

class Point[T: ToString](Struct):
    x: T
    y: T

    def new(x: T, y: T) -> Point[T]:
        return Point(x, y)
    
    def label(self) -> String:
        return "Point(" + str(self.x) + ", " + str(self.y) + ")"

# Structural polymorphism — no trait bound needed!
# Elevate infers T must have a .label() -> String method
def describe[T](p: T) -> String:
    return p.label()

def main():
    p: Point[i32] = Point.new(y=6, x=5)
    print(describe(p))
    
    p2: Point[i64] = Point.new(5, 6)
    print(describe(p2))