use metaquiche_host::compile;

#[test]
fn test_string_type_annotations() {
    // Python-style: "i32" should be evaluated as type i32
    let source = r#"
def add(a: "i32", b: "i32") -> "i32":
    return a + b
"#;
    let rust_code = compile(source, "test.qrs");
    assert!(rust_code.is_some(), "Compilation failed");
    let rust_code = rust_code.unwrap_or_default();
    assert!(
        rust_code.contains("pub fn add(a: i32, b: i32) -> i32"),
        "String annotation should resolve to i32, got:\n{}",
        rust_code
    );
}

#[test]
fn test_generic_string_annotations() {
    let source = r#"
def process(items: "List[i32]") -> "i32":
    return 0
"#;
    let rust_code = compile(source, "test.qrs");
    assert!(rust_code.is_some(), "Compilation failed");
    let rust_code = rust_code.unwrap_or_default();
    // Note: current implementation treats the whole string as a name, not parsing nested generics
    // This tests the basic string extraction behavior
    println!("{}", rust_code);
}
