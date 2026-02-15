//! Quiche `File` module — Elixir-style file operations.
//!
//! All functions use `Str` for paths and content.

use crate::{List, Str};
use std::sync::Arc;

/// Static module for file operations, used as `File.read(path)` in Quiche.
pub struct File;

impl File {
    /// Read entire file contents as a `Str`.
    ///
    /// Panics if the file cannot be read.
    pub fn read(path: Str) -> Str {
        let content = std::fs::read_to_string(&*path)
            .unwrap_or_else(|e| panic!("File.read failed for '{}': {}", &*path, e));
        Str(Arc::from(content.as_str()))
    }

    /// Write content to a file, creating it if it doesn't exist.
    ///
    /// Panics on failure.
    pub fn write(path: Str, content: Str) {
        std::fs::write(&*path, &*content)
            .unwrap_or_else(|e| panic!("File.write failed for '{}': {}", &*path, e));
    }

    /// List files in a directory, returning their names (not full paths).
    ///
    /// Panics if the directory cannot be read.
    pub fn ls(path: Str) -> List<Str> {
        let mut entries: Vec<Str> = std::fs::read_dir(&*path)
            .unwrap_or_else(|e| panic!("File.ls failed for '{}': {}", &*path, e))
            .filter_map(|entry| {
                entry
                    .ok()
                    .and_then(|e| e.file_name().to_str().map(|s| Str(Arc::from(s))))
            })
            .collect();
        entries.sort();
        List(entries)
    }

    /// Check if a file or directory exists.
    pub fn exists(path: Str) -> bool {
        std::path::Path::new(&*path).exists()
    }

    /// Remove a file.
    ///
    /// Panics on failure.
    pub fn rm(path: Str) {
        std::fs::remove_file(&*path)
            .unwrap_or_else(|e| panic!("File.rm failed for '{}': {}", &*path, e));
    }

    /// Create a directory and all parent directories.
    ///
    /// Panics on failure.
    pub fn mkdir_p(path: Str) {
        std::fs::create_dir_all(&*path)
            .unwrap_or_else(|e| panic!("File.mkdir_p failed for '{}': {}", &*path, e));
    }

    /// Copy a file from source to destination.
    ///
    /// Panics on failure.
    pub fn cp(src: Str, dst: Str) {
        std::fs::copy(&*src, &*dst)
            .unwrap_or_else(|e| panic!("File.cp failed '{}' -> '{}': {}", &*src, &*dst, e));
    }

    /// Move (rename) a file or directory.
    ///
    /// Panics on failure.
    pub fn mv(src: Str, dst: Str) {
        std::fs::rename(&*src, &*dst)
            .unwrap_or_else(|e| panic!("File.mv failed '{}' -> '{}': {}", &*src, &*dst, e));
    }

    /// Create or update the modification time of a file (like Unix `touch`).
    ///
    /// Panics on failure.
    pub fn touch(path: Str) {
        if !std::path::Path::new(&*path).exists() {
            std::fs::write(&*path, "")
                .unwrap_or_else(|e| panic!("File.touch failed for '{}': {}", &*path, e));
        }
        // Update mtime by opening and closing
        let _ = std::fs::OpenOptions::new().write(true).open(&*path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::str;

    #[test]
    fn file_write_read_rm() {
        let tmp = str("/tmp/quiche_test_file_module.txt");
        let content = str("hello from quiche");
        File::write(tmp.clone(), content.clone());
        assert!(File::exists(tmp.clone()));
        let read_back = File::read(tmp.clone());
        assert_eq!(&*read_back, &*content);
        File::rm(tmp.clone());
        assert!(!File::exists(tmp));
    }

    #[test]
    fn file_ls() {
        let dir = str("/tmp/quiche_test_ls_dir");
        File::mkdir_p(dir.clone());
        File::write(str("/tmp/quiche_test_ls_dir/a.txt"), str("a"));
        File::write(str("/tmp/quiche_test_ls_dir/b.txt"), str("b"));
        let files = File::ls(dir.clone());
        assert!(files.len() >= 2);
        // Cleanup
        File::rm(str("/tmp/quiche_test_ls_dir/a.txt"));
        File::rm(str("/tmp/quiche_test_ls_dir/b.txt"));
        let _ = std::fs::remove_dir(&*dir);
    }

    #[test]
    fn file_exists() {
        assert!(File::exists(str("Cargo.toml")));
        assert!(!File::exists(str("nonexistent_file_xyz.txt")));
    }
}
