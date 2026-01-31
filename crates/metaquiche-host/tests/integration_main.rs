use metaquiche_host::compile;

#[test]
fn test_compile_struct() {
    let source = r#"
@extern(path="path/to/Struct", no_generic=true)
class Struct: pass

class Point(Struct):
    x: "i32"
    y: "i32"
"#;
    let rust_code = compile(source).expect("Compilation failed");
    println!("{}", rust_code);
    assert!(rust_code.contains("pub struct Point"));
    assert!(rust_code.contains("pub x: i32"));
    assert!(rust_code.contains("pub y: i32"));
}

#[test]
fn test_compile_enum() {
    let source = r#"
@extern(path="path/to/Enum", no_generic=true)
class Enum: pass

class Color(Enum):
    Red = (1,)
    Green = (2, "i32")
"#;
    let rust_code = compile(source).expect("Compilation failed");
    println!("{}", rust_code);
    assert!(rust_code.contains("pub enum Color"));
    assert!(rust_code.contains("Red(1)")); // Wait, fields logic might differ.
                                           // In parser.rs: variants.push(VariantDef { name, fields })
                                           // In stmt.rs: output.push_str(&variant.fields.join(", "))
                                           // Fields are strings.
                                           // Parser `expr_to_string_compat` uses `to_string()`.
                                           // My parser logic: `fields.push(expr_to_string_compat(elt))` where elt is tuple element.
                                           // If elt is `1` (Int), it becomes "1".
                                           // If elt is `"i32"`, it becomes "i32".
                                           // So "Red(1)" or "Green(2, i32)".
    assert!(rust_code.contains("Green(2, i32)"));
}

#[test]
fn test_compile_function() {
    let source = r#"
def add(a: i32, b: i32) -> i32:
    return a + b
"#;
    let rust_code = compile(source);
    assert_ne!(None, rust_code, "Compilation failed");

    if let Some(rust_code) = rust_code {
        debug_assert!(
            rust_code.contains("pub fn add(a: i32, b: i32) -> i32"),
            "{}",
            rust_code
        );
        assert!(rust_code.contains("return a + b;"), "{}", rust_code);
    }
}
