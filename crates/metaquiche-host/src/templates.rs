// Project Scaffolding Templates

pub fn get_quiche_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
"#,
        name
    )
}

pub fn get_cargo_toml(name: &str, is_lib: bool, compiler_path: &str) -> String {
    let mut s = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

# Break out of any parent workspace
[workspace]

[build-dependencies]
quiche-compiler = {{ path = "{}" }}

[dependencies]
"#,
        name, compiler_path
    );

    if is_lib {
        s.push_str("\n[lib]\npath = \"src/lib.rs\"\n");
    } else {
        s.push_str("\n[[bin]]\nname = \"");
        s.push_str(name);
        s.push_str("\"\npath = \"src/main.rs\"\n");
    }
    s
}

pub fn get_build_rs() -> &'static str {
    r#"
use std::env;
use std::fs;
use std::path::Path;
use quiche_compiler::compile;

fn main() {
    println!("cargo:rerun-if-changed=src");
    let out_dir = env::var("OUT_DIR").unwrap();
    
    // Check for lib.qrs or main.qrs
    let is_lib = Path::new("src/lib.qrs").exists();
    let source_path = if is_lib { "src/lib.qrs" } else { "src/main.qrs" };
    let dest_name = if is_lib { "lib.rs" } else { "main.rs" };
    let dest_path = Path::new(&out_dir).join(dest_name);

    if Path::new(source_path).exists() {
        let source = fs::read_to_string(source_path).expect("Read source failed");
        let source = source.replace("struct ", "class ");
        
        if let Some(rust_code) = compile(&source) {
            fs::write(&dest_path, rust_code).expect("Write output failed");
        } else {
            panic!("Compilation failed");
        }
    } else {
        fs::write(&dest_path, "").unwrap();
    }
}
"#
}

pub fn get_lib_qrs() -> &'static str {
    r#"
def hello():
    print("Hello from Lib!")
"#
}

pub fn get_lib_rs() -> &'static str {
    r#"
#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]

mod quiche {
    #![allow(unused_macros, unused_imports)]
    
    // High Priority: Consumes Self (Result/Option)
    pub trait QuicheResult {
        type Output;
        fn quiche_handle(self) -> Self::Output;
    }
    
    impl<T, E: std::fmt::Debug> QuicheResult for Result<T, E> {
        type Output = T;
        fn quiche_handle(self) -> T {
            self.expect("Quiche Error")
        }
    }
    

    
    // Low Priority: Takes &Self (Clone fallback)
    pub trait QuicheGeneric {
        fn quiche_handle(&self) -> Self;
    }
    
    impl<T: Clone> QuicheGeneric for T {
        fn quiche_handle(&self) -> Self {
            self.clone()
        }
    }

    macro_rules! check {
        ($val:expr) => {
            {
                use crate::quiche::{QuicheResult, QuicheGeneric};
                ($val).quiche_handle()
            }
        };
    }
    pub(crate) use check;
    pub(crate) use check as call;
}

// Re-export everything from the transpiled module
include!(concat!(env!("OUT_DIR"), "/lib.rs"));
"#
}

pub fn get_main_qrs() -> &'static str {
    r#"
def main():
    print("Hello, Quiche!")
"#
}

pub fn get_main_rs() -> &'static str {
    r#"
#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]

mod quiche {
    #![allow(unused_macros, unused_imports)]
    
    // High Priority: Consumes Self (Result/Option)
    pub trait QuicheResult {
        type Output;
        fn quiche_handle(self) -> Self::Output;
    }
    
    impl<T, E: std::fmt::Debug> QuicheResult for Result<T, E> {
        type Output = T;
        fn quiche_handle(self) -> T {
            self.expect("Quiche Error")
        }
    }
    


    // Low Priority: Takes &Self (Clone fallback)
    pub trait QuicheGeneric {
        fn quiche_handle(&self) -> Self;
    }
    
    impl<T: Clone> QuicheGeneric for T {
        fn quiche_handle(&self) -> Self {
            self.clone()
        }
    }

    macro_rules! check {
        ($val:expr) => {
            {
                use crate::quiche::{QuicheResult, QuicheGeneric};
                ($val).quiche_handle()
            }
        };
    }
    pub(crate) use check;
    pub(crate) use check as call;
}

include!(concat!(env!("OUT_DIR"), "/main.rs"));
"#
}
