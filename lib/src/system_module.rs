//! Quiche `System` module — Elixir-style system interaction.
//!
//! Process spawning, environment variables, and program control.

use crate::{List, Str};
use std::sync::Arc;

/// Static module for system operations, used as `System.cmd(...)` in Quiche.
pub struct System;

impl System {
    /// Execute a command with arguments, returning `(stdout, exit_code)`.
    ///
    /// Panics if the command cannot be spawned.
    pub fn cmd(command: Str, args: List<Str>) -> (Str, i64) {
        let rust_args: Vec<&str> = args.iter().map(|s| &**s).collect();
        let output = std::process::Command::new(&*command)
            .args(&rust_args)
            .output()
            .unwrap_or_else(|e| panic!("System.cmd failed for '{}': {}", &*command, e));

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let code = output.status.code().unwrap_or(-1) as i64;
        (Str(Arc::from(stdout.as_str())), code)
    }

    /// Return command-line arguments passed to the program.
    pub fn argv() -> List<Str> {
        let args: Vec<Str> = std::env::args()
            .map(|a| Str(Arc::from(a.as_str())))
            .collect();
        List(args)
    }

    /// Get an environment variable. Returns empty string if not set.
    pub fn get_env(key: Str) -> Str {
        match std::env::var(&*key) {
            Ok(val) => Str(Arc::from(val.as_str())),
            Err(_) => Str(Arc::from("")),
        }
    }

    /// Set an environment variable.
    pub fn put_env(key: Str, value: Str) {
        unsafe {
            std::env::set_var(&*key, &*value);
        }
    }

    /// Return the current working directory.
    pub fn cwd() -> Str {
        let dir = std::env::current_dir().unwrap_or_else(|e| panic!("System.cwd failed: {}", e));
        Str(Arc::from(dir.to_string_lossy().as_ref()))
    }

    /// Halt the program with an exit code.
    pub fn halt(code: i64) {
        std::process::exit(code as i32);
    }

    /// Find an executable on the system PATH.
    pub fn find_executable(name: Str) -> Str {
        // Search PATH for the executable
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in path_var.split(':') {
                let candidate = std::path::Path::new(dir).join(&*name);
                if candidate.exists() {
                    return Str(Arc::from(candidate.to_string_lossy().as_ref()));
                }
            }
        }
        Str(Arc::from(""))
    }

    /// Return the OS process ID as a string.
    pub fn pid() -> Str {
        Str(Arc::from(std::process::id().to_string().as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::str;

    #[test]
    fn system_cmd_echo() {
        let (output, code) = System::cmd(str("echo"), List(vec![str("hello")]));
        assert_eq!(code, 0);
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn system_argv() {
        let args = System::argv();
        assert!(args.len() >= 1); // at least the binary name
    }

    #[test]
    fn system_get_env() {
        let home = System::get_env(str("HOME"));
        assert!(!home.is_empty());
    }

    #[test]
    fn system_cwd() {
        let cwd = System::cwd();
        assert!(!cwd.is_empty());
    }

    #[test]
    fn system_pid() {
        let pid = System::pid();
        assert!(!pid.is_empty());
    }
}
