//! Quiche `Path` module — Elixir-style path manipulation.
//!
//! Pure string operations on paths. Does not touch the filesystem
//! (except `expand` which resolves to an absolute path).

use crate::{List, Str};
use std::sync::Arc;

/// Static module for path operations, used as `Path.join(a, b)` in Quiche.
#[allow(non_camel_case_types)]
pub struct Path;

impl Path {
    /// Join two path segments.
    pub fn join(base: Str, child: Str) -> Str {
        let joined = std::path::Path::new(&*base).join(&*child);
        Str(Arc::from(joined.to_string_lossy().as_ref()))
    }

    /// Return the last component of the path (filename).
    pub fn basename(path: Str) -> Str {
        let p = std::path::Path::new(&*path);
        match p.file_name() {
            Some(name) => Str(Arc::from(name.to_string_lossy().as_ref())),
            None => path,
        }
    }

    /// Return the directory component of the path.
    pub fn dirname(path: Str) -> Str {
        let p = std::path::Path::new(&*path);
        match p.parent() {
            Some(parent) => Str(Arc::from(parent.to_string_lossy().as_ref())),
            None => Str(Arc::from(".")),
        }
    }

    /// Return the extension of the path (including the dot).
    pub fn extname(path: Str) -> Str {
        let p = std::path::Path::new(&*path);
        match p.extension() {
            Some(ext) => {
                let s = format!(".{}", ext.to_string_lossy());
                Str(Arc::from(s.as_str()))
            }
            None => Str(Arc::from("")),
        }
    }

    /// Return the path without its extension.
    pub fn rootname(path: Str) -> Str {
        let p = std::path::Path::new(&*path);
        match p.file_stem() {
            Some(stem) => {
                let parent = p.parent().unwrap_or(std::path::Path::new(""));
                let result = parent.join(stem);
                Str(Arc::from(result.to_string_lossy().as_ref()))
            }
            None => path,
        }
    }

    /// Expand a path to its absolute form, resolving `.` and `..`.
    pub fn expand(path: Str) -> Str {
        let p = std::path::Path::new(&*path);
        match std::fs::canonicalize(p) {
            Ok(abs) => Str(Arc::from(abs.to_string_lossy().as_ref())),
            Err(_) => {
                // If file doesn't exist, do best-effort with current dir
                match std::env::current_dir() {
                    Ok(cwd) => {
                        let joined = cwd.join(p);
                        Str(Arc::from(joined.to_string_lossy().as_ref()))
                    }
                    Err(_) => path,
                }
            }
        }
    }

    /// Split a path into its components.
    pub fn split(path: Str) -> List<Str> {
        let components: Vec<Str> = std::path::Path::new(&*path)
            .components()
            .map(|c| Str(Arc::from(c.as_os_str().to_string_lossy().as_ref())))
            .collect();
        List(components)
    }

    /// Match files against a glob pattern.
    ///
    /// Uses simple prefix+suffix matching (not full glob).
    pub fn wildcard(pattern: Str) -> List<Str> {
        // Simple implementation: list directory and filter by pattern
        let p = std::path::Path::new(&*pattern);
        let dir = p.parent().unwrap_or(std::path::Path::new("."));
        let pat = p
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut results: Vec<Str> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if Self::glob_match(&pat, &name) {
                    let full = entry.path().to_string_lossy().to_string();
                    results.push(Str(Arc::from(full.as_str())));
                }
            }
        }
        results.sort();
        List(results)
    }

    /// Simple glob matching: supports `*` as wildcard.
    fn glob_match(pattern: &str, name: &str) -> bool {
        if let Some(pos) = pattern.find('*') {
            let prefix = &pattern[..pos];
            let suffix = &pattern[pos + 1..];
            name.starts_with(prefix) && name.ends_with(suffix)
        } else {
            pattern == name
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::str;

    #[test]
    fn path_join() {
        let result = Path::join(str("src"), str("main.q"));
        assert_eq!(&*result, "src/main.q");
    }

    #[test]
    fn path_basename() {
        assert_eq!(&*Path::basename(str("/a/b/foo.q")), "foo.q");
        assert_eq!(&*Path::basename(str("foo.q")), "foo.q");
    }

    #[test]
    fn path_dirname() {
        assert_eq!(&*Path::dirname(str("/a/b/foo.q")), "/a/b");
    }

    #[test]
    fn path_extname() {
        assert_eq!(&*Path::extname(str("foo.q")), ".q");
        assert_eq!(&*Path::extname(str("foo")), "");
    }

    #[test]
    fn path_rootname() {
        assert_eq!(&*Path::rootname(str("foo.q")), "foo");
    }

    #[test]
    fn path_split() {
        let parts = Path::split(str("a/b/c"));
        assert_eq!(parts.len(), 3);
    }
}
