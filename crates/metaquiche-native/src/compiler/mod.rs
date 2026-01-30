pub mod codegen {
    #![allow(unused_imports, dead_code)]
    use crate::compiler;
    use crate::compiler::extern_defs;
    use crate::quiche::*;
    // codegen.rs generated from codegen.qrs
    include!(concat!(env!("OUT_DIR"), "/compiler/codegen.rs"));
}

pub mod extern_defs {
    #![allow(unused_imports, dead_code)]
    use crate::quiche::*;
    use std::collections::HashMap;
    // extern_defs.rs generated from extern_defs.qrs
    include!(concat!(env!("OUT_DIR"), "/compiler/extern_defs.rs"));

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
    include!(concat!(env!("OUT_DIR"), "/compiler/type_utils.rs"));
}
