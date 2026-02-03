from rust.std.collections import HashMap

class Student(Struct):
    name: String
    age: u8

    def new(name: String, age: u8) -> Self:
        return Self(name, age)

    def bio(self):
        return f"{self.name} is {self.age} years old"

class Class(Struct):
    code: String
    description: String
    teacher_name: String
    students: HashMap[String, Student]

    def new(code: String, description: String, teacher_name: String, students: HashMap[String, Student]):
        return Self(code, descripton, teacher_name, students)

    def summary(self):
        return f"Class {self.code} has {self.students.len()} students and is taught by {self.teacher_name}"

    def avg_age(self):
        if self.students.len() > 0:
            avgage = self.students.values().iter().foldl(0, |x, y| x + y) / self.students.len()
            return avgage
        return 0

# List comprehensions and lambdas
def main():
    nums = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]
    
    # Rust-style lambda syntax
    add = |x: i32, y: i32| x + y
    print("Sum: " + add(2, 3))
    
    # Pythonic len()
    print("Length: " + len(doubled))

    student: Student = Student.new("Yolanda", 16)


