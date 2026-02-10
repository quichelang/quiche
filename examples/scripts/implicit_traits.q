# Implicit Trait Resolution Test
# to_string() auto-generates impl Display, so print(c) just works.

class Color(Struct):
    r: u8
    g: u8
    b: u8

    def to_string(self) -> String:
        return f"rgb({self.r}, {self.g}, {self.b})"

class Vec2(Struct):
    x: i32
    y: i32

    def to_string(self) -> String:
        return f"({self.x}, {self.y})"

def main():
    c = Color(r=255, g=0, b=0)
    print(c)                    # Display — uses to_string automatically
    print(c.to_string())        # explicit call — same result

    a = Vec2(x=3, y=4)
    print(a)                    # Display
    print(a.to_string())        # explicit
