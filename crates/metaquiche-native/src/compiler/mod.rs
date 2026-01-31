pub mod codegen {
    #![allow(unused_imports, dead_code)]
    use crate::compiler;
    use crate::compiler::extern_defs;
    use crate::compiler::extern_defs::RustString;
    use crate::quiche::*;
    use quiche_parser::ast;
    // codegen.rs generated from codegen.qrs
    include!(concat!(env!("OUT_DIR"), "/compiler/codegen.rs"));
}

pub mod extern_defs {
    #![allow(unused_imports, dead_code)]
    use crate::quiche::*;
    use std::collections::HashMap;

    // RustString type alias (extern class in QRS)
    pub type RustString = String;

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
        defined_vars: Vec<HashMap<String, bool>>,
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
        }
    }
}

pub mod type_utils {
    #![allow(unused_imports, dead_code)]

    use crate::quiche::*;
    use quiche_parser::ast;
    include!(concat!(env!("OUT_DIR"), "/compiler/type_utils.rs"));
}
