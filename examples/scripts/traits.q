class Point[T](Struct):
    x: T
    y: T

    def new(x: T, y: T) -> Point[T]:
        return Point(x, y)
    
    def label(self) -> String:
        return "Point(" + str(self.x) + ", " + str(self.y) + ")"

    def to_string(self) -> String:
        return self.label()

class Line[T](Struct):
    p1: Point[T]
    p2: Point[T]

    def new(p1: Point[T], p2: Point[T]) -> Line[T]:
        return Line(p1, p2)
    
    def label(self) -> String:
        return "Line(" + str(self.p1) + ", " + str(self.p2) + ")"

    def to_string(self) -> String:
        return self.label()
        
# Structural polymorphism — no trait bound needed!
# Elevate infers T must have a .label() -> String method
def describe[T](p: T) -> String:
    return p.label()

def main():
    p: Point[i32] = Point.new(y=6, x=5)
    print(describe(p))
    
    p2: Point[i64] = Point.new(5, 6)
    print(describe(p2))

    l: Line[i32] = Line.new(p1=p, p2=p2)
    print(describe(l))