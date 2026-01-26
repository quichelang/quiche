//! Dict (HashMap) method translations from Python to Rust

/// Maps a Python dict method name to its Rust HashMap equivalent.
/// Returns Some((rust_method, key_needs_ref)) where key_needs_ref indicates if key arg needs &.
pub fn map_dict_method(method: &str) -> Option<(&'static str, bool)> {
    match method {
        // Direct mappings (key needs &)
        "get" => Some(("get", true)),
        "remove" => Some(("remove", true)),
        "contains_key" => Some(("contains_key", true)),

        // Direct mappings (no & needed)
        "insert" => Some(("insert", false)),
        "clear" => Some(("clear", false)),
        "keys" => Some(("keys", false)),
        "values" => Some(("values", false)),
        "items" => Some(("iter", false)), // Python items() -> Rust iter()

        // Python update() -> Rust extend()
        "update" => Some(("extend", false)),

        // Python pop(k) is like remove but returns value
        // HashMap::remove already returns Option<V>
        "pop" => Some(("remove", true)),

        _ => None,
    }
}

/// Check if a method name is a Python dict method
pub fn is_dict_method(method: &str) -> bool {
    matches!(
        method,
        "get"
            | "remove"
            | "insert"
            | "clear"
            | "keys"
            | "values"
            | "items"
            | "update"
            | "pop"
            | "contains_key"
    )
}
