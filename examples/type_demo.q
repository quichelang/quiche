# type keyword demo — all forms

# 1. Struct form
type Point:
    x: i32
    y: i32

# 2. Enum with payloads
type Color:
    Red = ()
    Green = (i32,)
    Blue = (i32, i32)

# 3. Enum with bare variants (OCaml-style)
type Direction:
    North
    South
    East
    West

# 4. Inline union
type Number = i64 | f64
type Value = i64 | f64 | String

# 5. Generic discriminated union
type MyResult[T, E]:
    Ok = (T,)
    Err = (E,)

def main():
    p: Point = Point(x=1, y=2)
    d: Direction = Direction::North
    print(p.x, p.y)
