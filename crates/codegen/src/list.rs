//! List (Vec) method translations from Python to Rust

/// Maps a Python list method name to its Rust Vec equivalent.
/// Returns Some((rust_method, needs_ref)) where needs_ref indicates if args need & prefix.
pub fn map_list_method(method: &str) -> Option<(&'static str, bool)> {
    match method {
        // Direct mappings
        "append" => Some(("push", false)),
        "pop" => Some(("pop", false)),
        "clear" => Some(("clear", false)),
        "reverse" => Some(("reverse", false)),
        "sort" => Some(("sort", false)),
        "insert" => Some(("insert", false)),
        "extend" => Some(("extend", false)),

        // len is handled as built-in function
        // These need special handling (not simple renames)
        // "remove" => requires find + swap_remove
        // "count" => requires iter().filter().count()
        // "index" => requires iter().position()
        _ => None,
    }
}

/// Check if a method name is a Python list method
pub fn is_list_method(method: &str) -> bool {
    matches!(
        method,
        "append" | "pop" | "clear" | "reverse" | "sort" | "insert" | "extend"
    )
}
