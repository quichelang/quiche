//! Dict (HashMap) method translations from Python to Rust

/// Maps a Python dict method name to its Rust HashMap equivalent.
/// Returns Some((rust_method, key_needs_ref, is_mutating)).
pub fn map_dict_method(method: &str) -> Option<(&'static str, bool, bool)> {
    match method {
        // Direct mappings (key needs &)
        "get" => Some(("get", true, false)),
        "remove" => Some(("remove", true, true)),
        "contains_key" => Some(("contains_key", true, false)),

        // Direct mappings (no & needed)
        "insert" => Some(("insert", false, true)),
        "clear" => Some(("clear", false, true)),
        "keys" => Some(("keys", false, false)),
        "values" => Some(("values", false, false)),
        "items" => Some(("iter", false, false)), // Python items() -> Rust iter()

        // Python update() -> Rust extend()
        "update" => Some(("extend", false, true)),

        // Python pop(k) is like remove but returns value
        // HashMap::remove already returns Option<V>
        "pop" => Some(("remove", true, true)),

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
