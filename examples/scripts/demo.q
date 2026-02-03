from rust.std.collections import HashMap

# Note: Methods REQUIRE return type annotations in Quiche
# Use StructName(field=value, ...) syntax for construction

class Student(Struct):
    name: String
    age: u8

    def new(name: String, age: u8) -> Student:
        return Student(name=name, age=age)

    def bio(self) -> String:
        return f"{self.name} is {self.age} years old"

class Class(Struct):
    code: String
    description: String
    teacher_name: String
    students: HashMap[String, Student]

    def new(code: String, description: String, teacher_name: String, students: Vec[Student]) -> Class:
        return Class(code=code, description=description, teacher_name=teacher_name, students={kv.name: kv for kv in students})

    def summary(self) -> String:
        return f"""
    Summary:
        Class {self.code} has {self.students.len()} students and is taught by {self.teacher_name}.
        The average age of the students is {self.avg_age()}.
        """

    def avg_age(self) -> f32:
        if self.students.len() > 0:
            total = self.students.values().fold(0, |acc, s| acc + s.age) as f32
            return total / self.students.len() as f32
        return 0.0

# List comprehensions and lambdas
def main():
    nums = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]
    
    # Rust-style lambda syntax
    add = |x: i32, y: i32| x + y
    print(f"Sum: {add(2, 3)}")
    
    # Pythonic len()
    print(f"Length: {len(doubled)}")

    students = [
        Student.new("Yolanda", 16),
        Student.new("Droopy", 13),
        Student.new("SnoopDog", 18),
        Student.new("SniperMan", 15)
    ]
    klass = Class.new("AP45", "Advanced placement computer science", "Harman Kodak", students)
    print(students[0].bio())
    print(klass.summary())
