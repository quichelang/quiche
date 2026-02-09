from std.collections import HashMap

# Quiche Demo: structs, methods, constructors, lambdas
# This demos features that fully transpile through Elevate to Rust.

class Student(Struct):
    name: String
    age: i64

    def new(name: String, age: i64) -> Student:
        return Student(name, age)

    def bio(self) -> String:
        return f"{self.name} is {self.age} years old"

class Classroom(Struct):
    code: String
    teacher: String

    def new(code: String, teacher: String) -> Classroom:
        return Classroom(code, teacher)

    def info(self) -> String:
        return f"Class {self.code} taught by {self.teacher}"

# Lambdas and list comprehensions
def main():
    # List comprehension
    nums = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]

    # Lambda
    add = |x: i64, y: i64| x + y
    print(f"Sum: {add(2, 3)}")

    # Struct construction and methods
    s: Student = Student.new("Alice", 20)
    print(s.bio())

    room: Classroom = Classroom.new("CS101", "Dr. Smith")
    print(room.info())
