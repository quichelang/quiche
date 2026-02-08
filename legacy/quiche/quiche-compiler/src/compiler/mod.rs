pub mod codegen {
    #![allow(unused_imports, dead_code)]
    use crate::compiler;
    use crate::compiler::extern_defs;
    use crate::compiler::extern_defs::RustString;
    use crate::quiche::*;
    use metaquiche_parser::ast;
    // codegen.rs generated from codegen.qrs
    include!(concat!(env!("OUT_DIR"), "/compiler/codegen.rs"));
}

pub mod extern_defs {
    #![allow(unused_imports, dead_code)]
    use crate::quiche::*;
    use std::collections::HashMap;

    // RustString type alias (extern class in QRS)
    pub type RustString = String;

    // TypeInfo struct for tracking variable types in scope (for CoW iterator support)
    #[derive(Clone, Debug, Default)]
    pub struct TypeInfo {
        pub type_str: String,
        pub is_ref: bool,
        pub is_mut_ref: bool,
        pub is_iterable_ref: bool,
    }

    /// Create TypeInfo from a Rust type string
    pub fn create_type_info(type_str: String) -> TypeInfo {
        let is_ref = type_str.starts_with("&");
        let is_mut_ref = type_str.starts_with("&mut");

        // Check if it's a mutable reference to an iterable type
        let is_iterable_ref = if is_mut_ref {
            let inner = if type_str.starts_with("&mut ") {
                &type_str[5..]
            } else if type_str.starts_with("&mut") {
                &type_str[4..]
            } else {
                &type_str[..]
            };
            // Check for Vec, slice, or other common iterables
            inner.starts_with("Vec<") || inner.starts_with("[") || inner.starts_with("String")
        } else {
            false
        };

        TypeInfo {
            type_str,
            is_ref,
            is_mut_ref,
            is_iterable_ref,
        }
    }

    /// Create a simple TypeInfo for untyped variables
    pub fn create_type_info_simple() -> TypeInfo {
        TypeInfo::default()
    }

    // Wrapper functions for extern bindings
    pub fn q_push(mut s: String, val: String) -> String {
        s.push_str(&val);
        s
    }

    pub fn escape_rust_string(s: String) -> String {
        crate::quiche::escape_rust_string(s)
    }

    pub fn vec_to_list<T>(v: Vec<T>) -> Vec<T> {
        v
    }

    pub fn create_codegen(
        output: String,
        tuple_vars: HashMap<String, bool>,
        defined_vars: Vec<HashMap<String, TypeInfo>>,
        import_paths: HashMap<String, String>,
        import_kinds: HashMap<String, String>,
        clone_names: bool,
        current_module_path: String,
        class_fields: HashMap<String, HashMap<String, String>>,
        current_class: String,
    ) -> super::codegen::Codegen {
        super::codegen::Codegen {
            output,
            tuple_vars,
            defined_vars,
            import_paths,
            import_kinds,
            clone_names,
            current_module_path,
            class_fields,
            current_class,
            in_trait_or_impl: false,
        }
    }
}

pub mod type_utils {
    #![allow(unused_imports, dead_code)]

    use crate::quiche::*;
    use metaquiche_parser::ast;
    include!(concat!(env!("OUT_DIR"), "/compiler/type_utils.rs"));
}

#[cfg(test)]
mod tests {
    use super::extern_defs::{TypeInfo, create_type_info, create_type_info_simple};

    #[test]
    fn test_type_info_simple_is_default() {
        let ti = create_type_info_simple();
        assert_eq!(ti.type_str, "");
        assert!(!ti.is_ref);
        assert!(!ti.is_mut_ref);
        assert!(!ti.is_iterable_ref);
    }

    #[test]
    fn test_type_info_owned_type() {
        let ti = create_type_info("i32".to_string());
        assert_eq!(ti.type_str, "i32");
        assert!(!ti.is_ref);
        assert!(!ti.is_mut_ref);
        assert!(!ti.is_iterable_ref);
    }

    #[test]
    fn test_type_info_immutable_ref() {
        let ti = create_type_info("&str".to_string());
        assert_eq!(ti.type_str, "&str");
        assert!(ti.is_ref);
        assert!(!ti.is_mut_ref);
        assert!(!ti.is_iterable_ref);
    }

    #[test]
    fn test_type_info_mutable_ref_non_iterable() {
        let ti = create_type_info("&mut i32".to_string());
        assert_eq!(ti.type_str, "&mut i32");
        assert!(ti.is_ref);
        assert!(ti.is_mut_ref);
        assert!(!ti.is_iterable_ref);
    }

    #[test]
    fn test_type_info_mut_ref_vec_with_space() {
        let ti = create_type_info("&mut Vec<i32>".to_string());
        assert_eq!(ti.type_str, "&mut Vec<i32>");
        assert!(ti.is_ref);
        assert!(ti.is_mut_ref);
        assert!(ti.is_iterable_ref);
    }

    #[test]
    fn test_type_info_mut_ref_vec_no_space() {
        // Edge case: &mutVec<> without space - still detected
        let ti = create_type_info("&mutVec<String>".to_string());
        assert!(ti.is_ref);
        assert!(ti.is_mut_ref);
        assert!(ti.is_iterable_ref);
    }

    #[test]
    fn test_type_info_mut_ref_string() {
        let ti = create_type_info("&mut String".to_string());
        assert!(ti.is_ref);
        assert!(ti.is_mut_ref);
        assert!(ti.is_iterable_ref);
    }

    #[test]
    fn test_type_info_mut_ref_slice() {
        let ti = create_type_info("&mut [u8]".to_string());
        assert!(ti.is_ref);
        assert!(ti.is_mut_ref);
        assert!(ti.is_iterable_ref);
    }

    #[test]
    fn test_type_info_immutable_ref_vec_not_iterable() {
        // Only &mut iterables trigger CoW, not &
        let ti = create_type_info("&Vec<i32>".to_string());
        assert!(ti.is_ref);
        assert!(!ti.is_mut_ref);
        assert!(!ti.is_iterable_ref);
    }

    #[test]
    fn test_type_info_owned_vec_not_iterable_ref() {
        // Owned Vec is not a reference
        let ti = create_type_info("Vec<i32>".to_string());
        assert!(!ti.is_ref);
        assert!(!ti.is_mut_ref);
        assert!(!ti.is_iterable_ref);
    }
}
