#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]

mod quiche {
    #![allow(unused_macros, unused_imports)]

    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    // High Priority: Consumes Self (Result/Option)
    pub trait QuicheResult {
        type Output;
        fn quiche_handle(self) -> Self::Output;
    }

    impl<T, E: std::fmt::Debug> QuicheResult for Result<T, E> {
        type Output = T;
        fn quiche_handle(self) -> T {
            self.expect("Quiche Exception")
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
        ($val:expr) => {{
            use crate::quiche::{QuicheGeneric, QuicheResult};
            ($val).quiche_handle()
        }};
    }
    pub(crate) use check;
    pub(crate) use check as call;

    pub fn env_args_helper() -> Vec<String> {
        std::env::args().collect()
    }

    pub fn push_str_wrapper(mut s: String, val: String) -> String {
        s.push_str(&val);
        s
    }

    pub fn escape_rust_string(s: String) -> String {
        s.replace('\\', "\\\\").replace('\"', "\\\"")
    }

    pub fn path_dirname(path: String) -> String {
        let p = Path::new(&path);
        match p.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => {
                parent.to_string_lossy().into_owned()
            }
            _ => ".".to_string(),
        }
    }

    pub fn module_parent(path: String, levels: u32) -> String {
        if path.is_empty() {
            return "".to_string();
        }
        let mut parts: Vec<&str> = path.split('.').collect();
        let mut remaining = levels;
        while remaining > 0 && !parts.is_empty() {
            parts.pop();
            remaining -= 1;
        }
        parts.join(".")
    }

    pub fn module_join(base: String, sub: String) -> String {
        if base.is_empty() {
            return sub;
        }
        if sub.is_empty() {
            return base;
        }
        format!("{base}.{sub}")
    }

    fn module_path_from_relative(rel: &Path) -> String {
        let file_name = match rel.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return "".to_string(),
        };
        let mut parts: Vec<String> = rel
            .parent()
            .map(|p| p.iter().filter_map(|c| c.to_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        if file_name == "mod.qrs" {
            if parts.is_empty() {
                return "".to_string();
            }
            return parts.join(".");
        }

        if let Some(stem) = Path::new(file_name).file_stem().and_then(|s| s.to_str()) {
            parts.push(stem.to_string());
        }
        parts.join(".")
    }

    fn collect_qrs_files(root: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_qrs_files(&path, out);
                } else if path.extension().and_then(|e| e.to_str()) == Some("qrs") {
                    out.push(path);
                }
            }
        }
    }

    pub fn build_module_index(root: String) -> HashMap<String, String> {
        let root_path = PathBuf::from(root);
        let mut files = Vec::new();
        collect_qrs_files(&root_path, &mut files);
        let mut index = HashMap::new();
        for file in files {
            let rel = match file.strip_prefix(&root_path) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let module_path = module_path_from_relative(rel);
            index.insert(module_path, file.to_string_lossy().into_owned());
        }
        index
    }

    pub fn module_path_for_file(root: String, filename: String) -> String {
        let root_path = PathBuf::from(root);
        let file_path = PathBuf::from(filename);
        let rel = match file_path.strip_prefix(&root_path) {
            Ok(r) => r,
            Err(_) => return "".to_string(),
        };
        module_path_from_relative(rel)
    }
}

#[cfg(feature = "bootstrap")]
include!("main_gen.rs");

#[cfg(not(feature = "bootstrap"))]
include!(concat!(env!("OUT_DIR"), "/main.rs"));
