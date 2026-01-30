use crate::ast::{Stmt, Parsed};
use crate::compiler::codegen::{Codegen};
use parsley_qrs::{Parsley};
use std::collections::{HashMap};
use std::any::{Any};
pub type RustString = std::string::String;
pub type ParseError = ruff_python_parser::ParseError;
pub use ruff_python_parser::parse_module as parse_module;
pub use crate::quiche::env_args_helper as env_args;
pub use std::fs::read_to_string as read_to_string;
pub use std::process::exit as exit;
pub use crate::quiche::path_dirname as path_dirname;
pub use crate::quiche::build_module_index as build_module_index;
pub use crate::quiche::module_path_for_file as module_path_for_file;
pub use crate::quiche::module_parent as module_parent;
pub use crate::quiche::dedup_shadowed_let_mut as dedup_shadowed_let_mut;
pub use crate::quiche::module_join as module_join;
pub use crate::quiche::path_exists as path_exists;
pub use crate::quiche::create_dir_all as create_dir_all;
pub use crate::quiche::write_string as write_string;
pub use crate::quiche::set_env_var as set_env_var;
pub use crate::quiche::current_exe_path as current_exe_path;
pub use crate::quiche::run_cargo_command as run_cargo_command;
pub use crate::quiche::run_rust_code as run_rust_code;
pub use crate::quiche::compiler_path_for_new as compiler_path_for_new;
pub use crate::quiche::print_stdout as print_stdout;
pub use crate::version_info::get_stage as version_get_stage;
pub use crate::version_info::get_commit as version_get_commit;
pub use crate::version_info::get_date as version_get_date;
pub use crate::version_info::get_build_kind as version_get_build_kind;
pub fn get_flag_bool(flags: std::collections::HashMap<String, bool>, key: String) -> bool {
match crate::quiche::check!(flags.get(as_ref!(key))) {
Some(v) => {
return v.clone();
}
None => {
}
}
return false;
}

pub type ParseResult = parsley_qrs::ParseResult;
#[derive(Clone, Debug, Default)]
pub struct WarnFlags  {
pub warn: bool,
pub strict: bool,
pub warn_all: bool,
pub warn_quiche: bool,
}

impl WarnFlags {
pub fn new(warn: bool, strict: bool, warn_all: bool, warn_quiche: bool) -> WarnFlags {
return crate::quiche::check!(WarnFlags());
}

}

#[derive(Clone, Debug, Default)]
pub struct ImportMaps  {
pub paths: std::collections::HashMap<String, String>,
pub kinds: std::collections::HashMap<String, String>,
}

impl ImportMaps {
pub fn new(paths: std::collections::HashMap<String, String>, kinds: std::collections::HashMap<String, String>) -> ImportMaps {
return crate::quiche::check!(ImportMaps());
}

}

pub fn module_to_rust_path(module_path: String) -> String {
if module_path == String::from("") {
return String::from("crate");
}
let mut is_external = false;
let mut res = module_path.clone();
if crate::quiche::check!(res.starts_with(as_ref!(String::from("rust.")))) {
res = crate::quiche::check!(res.replace(as_ref!(String::from("rust.")), as_ref!(String::from(""))));
is_external = true;
}
else {
if crate::quiche::check!(res.starts_with(as_ref!(String::from("std.")))) {
is_external = true;
}
}
if !is_external {
res = String::from("crate.").to_string() + as_ref!(res);
}
return crate::quiche::check!(res.replace(as_ref!(String::from(".")), as_ref!(String::from("::"))));
}

pub fn resolve_module_path(current_module_path: String, module_name: String, level: u32) -> String {
let mut base = String::from("");
if level > 0 {
base = crate::quiche::check!(module_parent(current_module_path.clone(), level));
}
if module_name == String::from("") {
return base;
}
if base != String::from("") {
return crate::quiche::check!(module_join(base, module_name));
}
return module_name;
}

pub fn collect_module_exports(module_index: std::collections::HashMap<String, String>, module_path: String) -> std::collections::HashMap<String, String> {
let mut exports = std::collections::HashMap::new();
match crate::quiche::check!(module_index.get(as_ref!(module_path))) {
Some(path) => {
let mut source = crate::quiche::check!(read_to_string(path.clone()));
let mut parsed = crate::quiche::check!(parse_module(source.as_str()));
let mut module = parsed.into_syntax();
for __q in (module.body) {
let stmt = __q;
match stmt {
Stmt::ClassDef(c) => {
crate::quiche::check!(exports.insert(c.name.as_str().to_string(), String::from("class")));
}
Stmt::FunctionDef(f) => {
crate::quiche::check!(exports.insert(f.name.as_str().to_string(), String::from("func")));
}
_ => {
}
}
}
}
None => {
}
}
return exports;
}

pub fn build_import_maps(stmts: Vec<Stmt>, module_index: std::collections::HashMap<String, String>, current_module_path: String) -> ImportMaps {
let mut import_paths = std::collections::HashMap::new();
let mut import_kinds = std::collections::HashMap::new();
for __q in (stmts) {
let stmt = __q;
match stmt {
Stmt::Import(i) => {
for __q in (i.names) {
let alias = __q;
let mut module_name = alias.name.as_str().to_string();
let mut alias_name = module_name.clone();
match alias.asname.clone() {
Some(a) => {
alias_name = a.as_str().to_string();
}
None => {
}
}
let mut module_path = crate::quiche::check!(resolve_module_path(current_module_path.clone(), module_name, 0));
let mut rust_path = crate::quiche::check!(module_to_rust_path(module_path));
crate::quiche::check!(import_paths.insert(alias_name.clone(), rust_path));
crate::quiche::check!(import_kinds.insert(alias_name.clone(), String::from("module")));
}
}
Stmt::ImportFrom(i) => {
let mut base_module = String::from("");
match i.module {
Some(m) => {
if m.as_str().to_string() == String::from("lib.test") {
continue;
}
base_module = crate::quiche::check!(resolve_module_path(current_module_path.clone(), m.as_str().to_string(), i.level));
}
None => {
base_module = crate::quiche::check!(resolve_module_path(current_module_path.clone(), String::from(""), i.level));
}
}
let mut is_external = (crate::quiche::check!(base_module.starts_with(as_ref!(String::from("rust."))))) || (crate::quiche::check!(base_module.starts_with(as_ref!(String::from("std.")))));
let mut exports = std::collections::HashMap::new();
if !is_external {
match crate::quiche::check!(module_index.get(as_ref!(base_module.clone()))) {
Some(_) => {
exports = crate::quiche::check!(collect_module_exports(module_index.clone(), base_module.clone()));
}
None => {
}
}
}
for __q in (i.names) {
let alias = __q;
let mut name = alias.name.as_str().to_string();
let mut alias_name = name.clone();
match alias.asname.clone() {
Some(a) => {
alias_name = a.as_str().to_string();
}
None => {
}
}
let mut kind = String::from("value");
let mut item_module = crate::quiche::check!(module_join(base_module.clone(), name.clone()));
let mut rust_path = String::from("");
match crate::quiche::check!(module_index.get(as_ref!(item_module.clone()))) {
Some(_) => {
kind = String::from("module");
rust_path = crate::quiche::check!(module_to_rust_path(item_module));
}
None => {
rust_path = crate::quiche::check!(module_to_rust_path(base_module.clone()));
rust_path = rust_path + as_ref!(String::from("::"));
rust_path = rust_path + as_ref!(name);
if is_external {
kind = String::from("module");
}
else {
match crate::quiche::check!(exports.get(as_ref!(name.clone()))) {
Some(k) => {
if deref!(k) == String::from("class") {
kind = String::from("type");
}
else {
if deref!(k) == String::from("func") {
kind = String::from("func");
}
else {
kind = String::from("value");
}
}
}
None => {
kind = String::from("value");
}
}
}
}
}
crate::quiche::check!(import_paths.insert(alias_name.clone(), rust_path));
crate::quiche::check!(import_kinds.insert(alias_name.clone(), kind));
}
}
_ => {
}
}
}
return crate::quiche::check!(ImportMaps.new(import_paths, import_kinds));
}

pub fn normalize_module_path(module_path: String) -> String {
if (module_path == String::from("main")) || (module_path == String::from("lib")) {
return String::from("");
}
return module_path;
}

pub fn compile_source_to_string(source: String, filename: String, warn_quiche: bool, strict: bool) -> String {
let mut parsed = crate::quiche::check!(parse_module(source.as_str()));
let mut module = parsed.into_syntax();
let mut stmts = module.body;
let mut root_dir = crate::quiche::check!(path_dirname(filename.clone()));
let mut module_index = crate::quiche::check!(build_module_index(root_dir.clone()));
let mut current_module_path = crate::quiche::check!(normalize_module_path(crate::quiche::check!(module_path_for_file(root_dir.clone(), filename.clone()))));
let mut import_maps = crate::quiche::check!(build_import_maps(stmts.clone(), module_index, current_module_path.clone()));
let mut codegen = crate::quiche::check!(Codegen.new_with_imports_and_module(import_maps.paths, import_maps.kinds, current_module_path));
for __q in (stmts) {
let stmt = __q;
crate::quiche::check!(codegen.generate_stmt(stmt));
}
let mut cleaned = crate::quiche::check!(dedup_shadowed_let_mut(codegen.output));
return cleaned;
return String::from("");
}

pub fn compile_source(source: String, filename: String, warn_quiche: bool, strict: bool) {
let mut code = crate::quiche::check!(compile_source_to_string(source, filename, warn_quiche, strict));
crate::quiche::check!(print_str(code));
}

pub fn print_usage() {
println!("{}", String::from("Usage:"));
println!("{}", String::from("  quiche-self new <name>    Create a new Quiche project"));
println!("{}", String::from("  quiche-self build         Build the current project"));
println!("{}", String::from("  quiche-self run           Run the current project"));
println!("{}", String::from("  quiche-self test          Run project tests"));
println!("{}", String::from("  quiche-self <file.qrs>    Transpile a single file"));
println!("{}", String::from(""));
println!("{}", String::from("Flags:"));
println!("{}", String::from("  --warn               Show compiler warnings"));
println!("{}", String::from("  --strict             Treat warnings as errors"));
println!("{}", String::from("  --warn-all           Show all warnings (Quiche + Rust)"));
println!("{}", String::from("  --warn-quiche        Show only Quiche warnings"));
}

pub fn build_flag_parser(include_lib: bool) -> Parsley {
let mut parser = crate::quiche::check!(Parsley.new());
crate::quiche::check!(parser.add_flag(String::from("warn-all"), vec![String::from("--warn")], false));
crate::quiche::check!(parser.add_flag(String::from("warn-quiche"), vec![], false));
crate::quiche::check!(parser.add_flag(String::from("strict"), vec![], false));
crate::quiche::check!(parser.add_flag(String::from("emit-rust"), vec![], false));
if include_lib {
crate::quiche::check!(parser.add_flag(String::from("lib"), vec![], false));
}
return parser;
}

pub fn parse_warn_flags(flags: std::collections::HashMap<String, bool>) -> WarnFlags {
let mut warn_all = false;
let mut warn_quiche = false;
let mut strict = false;
let mut k1 = String::from("warn-all");
warn_all = crate::quiche::check!(get_flag_bool(flags.clone(), k1));
let mut k2 = String::from("warn-quiche");
warn_quiche = crate::quiche::check!(get_flag_bool(flags.clone(), k2));
let mut k3 = String::from("strict");
strict = crate::quiche::check!(get_flag_bool(flags.clone(), k3));
let mut warn = warn_all;
if warn_all {
warn = true;
}
return crate::quiche::check!(WarnFlags.new(warn, strict, warn_all, warn_quiche));
}

pub fn template_quiche_toml(name: String) -> String {
return String::from("[package]\nname = \"") + name.as_str() + as_ref!(String::from("\"\nversion = \"0.1.0\"\n"));
}

pub fn template_cargo_toml(name: String, is_lib: bool, compiler_path: String) -> String {
let mut s = String::from("[package]\nname = \"") + name.as_str() + as_ref!(String::from("\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n"));
s = s + as_ref!(String::from("# Break out of any parent workspace\n[workspace]\n\n"));
s = s + as_ref!(String::from("[build-dependencies]\nquiche_compiler = { path = \"")) + compiler_path.as_str() + as_ref!(String::from("\" }\n\n"));
s = s + as_ref!(String::from("[dependencies]\n"));
s = s + as_ref!(String::from("quiche-runtime = { path = \"../quiche-runtime\" }\n"));
if is_lib {
s = s + as_ref!(String::from("\n[lib]\npath = \"src/lib.rs\"\n"));
}
else {
s = s + as_ref!(String::from("\n[[bin]]\nname = \"")) + name.as_str() + as_ref!(String::from("\"\npath = \"src/main.rs\"\n"));
}
return s;
}

pub fn template_build_rs() -> String {
return String::from("\nuse std::env;\nuse std::fs;\nuse std::path::Path;\nuse quiche_compiler::compile;\n\nfn main() {\n    println!(\"cargo:rerun-if-changed=src\");\n    let out_dir = env::var(\"OUT_DIR\").unwrap();\n    \n    // Check for lib.qrs or main.qrs\n    let is_lib = Path::new(\"src/lib.qrs\").exists();\n    let source_path = if is_lib { \"src/lib.qrs\" } else { \"src/main.qrs\" };\n    let dest_name = if is_lib { \"lib.rs\" } else { \"main.rs\" };\n    let dest_path = Path::new(&out_dir).join(dest_name);\n\n    if Path::new(source_path).exists() {\n        let source = fs::read_to_string(source_path).expect(\"Read source failed\");\n        let source = source.replace(\"struct \", \"class \");\n        \n        if let Some(rust_code) = compile(&source) {\n            fs::write(&dest_path, rust_code).expect(\"Write output failed\");\n        } else {\n            panic!(\"Compilation failed\");\n        }\n    } else {\n        fs::write(&dest_path, \"\").unwrap();\n    }\n}\n");
}

pub fn template_lib_qrs() -> String {
return String::from("\ndef hello():\n    print(\"Hello from Lib!\")\n");
}

pub fn template_main_qrs() -> String {
return String::from("\ndef main():\n    print(\"Hello, Quiche!\")\n");
}

pub fn template_lib_rs() -> String {
return String::from("\n#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]\n\nuse quiche_runtime::*;\nuse quiche_runtime::check;\n\n// Re-export everything from the transpiled module\ninclude!(concat!(env!(\"OUT_DIR\"), \"/lib.rs\"));\n");
}

pub fn template_main_rs() -> String {
return String::from("\n#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]\n\nuse quiche_runtime::*;\nuse quiche_runtime::check;\n\ninclude!(concat!(env!(\"OUT_DIR\"), \"/main.rs\"));\n");
}

pub fn create_new_project(name: String, is_lib: bool) {
if crate::quiche::check!(path_exists(name.clone())) {
println!("{}", String::from("Error: Directory '") + name + String::from("' already exists"));
return ;
}
crate::quiche::check!(create_dir_all(name.clone() + as_ref!(String::from("/src"))));
crate::quiche::check!(write_string(name.clone() + as_ref!(String::from("/Quiche.toml")), crate::quiche::check!(template_quiche_toml(name.clone()))));
let mut compiler_path = crate::quiche::check!(compiler_path_for_new());
crate::quiche::check!(write_string(name.clone() + as_ref!(String::from("/Cargo.toml")), crate::quiche::check!(template_cargo_toml(name.clone(), is_lib, compiler_path))));
crate::quiche::check!(write_string(name.clone() + as_ref!(String::from("/build.rs")), crate::quiche::check!(template_build_rs())));
if is_lib {
crate::quiche::check!(write_string(name.clone() + as_ref!(String::from("/src/lib.qrs")), crate::quiche::check!(template_lib_qrs())));
crate::quiche::check!(write_string(name.clone() + as_ref!(String::from("/src/lib.rs")), crate::quiche::check!(template_lib_rs())));
}
else {
crate::quiche::check!(write_string(name.clone() + as_ref!(String::from("/src/main.qrs")), crate::quiche::check!(template_main_qrs())));
crate::quiche::check!(write_string(name.clone() + as_ref!(String::from("/src/main.rs")), crate::quiche::check!(template_main_rs())));
}
println!("{}", String::from("Created new project: ") + name);
}

pub fn run_transpiled_file(filename: String, script_args: Vec<String>, quiet: bool, suppress_output: bool, raw_output: bool, warn: bool, strict: bool, warn_quiche: bool) {
let mut source = crate::quiche::check!(read_to_string(filename.clone()));
let mut rust_code = crate::quiche::check!(compile_source_to_string(source, filename.clone(), warn_quiche, strict));
let mut exit_code = crate::quiche::check!(run_rust_code(rust_code, script_args, quiet, suppress_output, raw_output, warn, strict));
if exit_code != 0 {
crate::quiche::check!(exit(exit_code));
}
}

fn main() {
let mut cli_args: Vec<String> = crate::quiche::check!(env_args());
let mut args: Vec<String> = vec![];
let mut i = 0;
for __q in (cli_args) {
let arg = __q;
if i > 0 {
args.push(arg);
}
i = i + 1;
}
if crate::quiche::check!(args.len()) == 0 {
println!("{}", String::from("Error: No command specified."));
crate::quiche::check!(print_usage());
crate::quiche::check!(exit(1));
}
let mut commands = vec![String::from("new"), String::from("build"), String::from("run"), String::from("test"), String::from("version")];
let mut cmd = args[0].clone();
let mut has_command = false;
for __q in (commands) {
let c = __q;
if c == cmd {
has_command = true;
}
}
if has_command {
let mut subargs: Vec<String> = vec![];
let mut j = 1;
while j < crate::quiche::check!(args.len()) {
subargs.push(args[j].clone());
j = j + 1;
}
if cmd == String::from("new") {
let mut parser = crate::quiche::check!(build_flag_parser(true));
let mut result = crate::quiche::check!(parser.parse(subargs));
if crate::quiche::check!(result.errors.len()) > 0 {
for __q in (result.errors) {
let e = __q;
println!("{}", e);
}
crate::quiche::check!(print_usage());
crate::quiche::check!(exit(1));
}
if crate::quiche::check!(result.positionals.len()) < 1 {
println!("{}", String::from("Error: Missing project name."));
println!("{}", String::from("Usage: quiche-self new [--lib] <project_name>"));
crate::quiche::check!(exit(1));
}
let mut name = result.positionals[0].clone();
let mut is_lib = false;
let mut k_lib = String::from("lib");
k_lib = String::from("lib");
is_lib = crate::quiche::check!(get_flag_bool(result.flags.clone(), k_lib));
crate::quiche::check!(create_new_project(name, is_lib));
return ;
}
if cmd == String::from("build") {
let mut parser = crate::quiche::check!(build_flag_parser(false));
let mut result = crate::quiche::check!(parser.parse(subargs));
if crate::quiche::check!(result.errors.len()) > 0 {
for __q in (result.errors) {
let e = __q;
println!("{}", e);
}
crate::quiche::check!(print_usage());
crate::quiche::check!(exit(1));
}
let mut exit_code = crate::quiche::check!(run_cargo_command(String::from("build").to_string(), result.positionals));
if exit_code != 0 {
crate::quiche::check!(exit(exit_code));
}
return ;
}
if cmd == String::from("run") {
let mut parser = crate::quiche::check!(build_flag_parser(false));
let mut result = crate::quiche::check!(parser.parse(subargs));
if crate::quiche::check!(result.errors.len()) > 0 {
for __q in (result.errors) {
let e = __q;
println!("{}", e);
}
crate::quiche::check!(print_usage());
crate::quiche::check!(exit(1));
}
if crate::quiche::check!(path_exists(String::from("Cargo.toml"))) {
let mut exit_code = crate::quiche::check!(run_cargo_command(String::from("run").to_string(), result.positionals));
if exit_code != 0 {
crate::quiche::check!(exit(exit_code));
}
}
else {
println!("{}", String::from("Error: No Cargo.toml found in current directory."));
println!("{}", String::from("To run a single script, use: quiche-self <file.qrs>"));
crate::quiche::check!(exit(1));
}
return ;
}
if cmd == String::from("test") {
let mut parser = crate::quiche::check!(build_flag_parser(false));
let mut result = crate::quiche::check!(parser.parse(subargs));
if crate::quiche::check!(result.errors.len()) > 0 {
for __q in (result.errors) {
let e = __q;
println!("{}", e);
}
crate::quiche::check!(print_usage());
crate::quiche::check!(exit(1));
}
let mut flags = crate::quiche::check!(parse_warn_flags(result.flags.clone()));
if flags.warn_all {
flags.warn_quiche = true;
}
if crate::quiche::check!(path_exists(String::from("tests/runner.qrs"))) {
let mut exe = crate::quiche::check!(current_exe_path());
if exe != String::from("") {
crate::quiche::check!(set_env_var(String::from("QUICHE_TEST_BIN"), exe));
}
if flags.warn_all {
crate::quiche::check!(set_env_var(String::from("QUICHE_WARN_ALL"), String::from("1")));
}
if flags.warn_quiche {
crate::quiche::check!(set_env_var(String::from("QUICHE_WARN_QUICHE"), String::from("1")));
}
crate::quiche::check!(run_transpiled_file(String::from("tests/runner.qrs"), result.positionals, true, false, true, flags.warn, flags.strict, flags.warn_quiche));
}
else {
if crate::quiche::check!(path_exists(String::from("Cargo.toml"))) {
let mut exit_code = crate::quiche::check!(run_cargo_command(String::from("test").to_string(), result.positionals));
if exit_code != 0 {
crate::quiche::check!(exit(exit_code));
}
}
else {
println!("{}", String::from("Error: No tests/runner.qrs or Cargo.toml found."));
crate::quiche::check!(exit(1));
}
}
return ;
}
if cmd == String::from("version") {
let mut stage = crate::quiche::check!(version_get_stage());
let mut commit = crate::quiche::check!(version_get_commit());
let mut date = crate::quiche::check!(version_get_date());
let mut kind = crate::quiche::check!(version_get_build_kind());
println!("{}", String::from("quiche-self ") + stage);
println!("{}", String::from("  stage:   ") + stage);
println!("{}", String::from("  commit:  ") + commit);
println!("{}", String::from("  built:   ") + date);
println!("{}", String::from("  profile: ") + kind);
return ;
}
}
if crate::quiche::check!(cmd.ends_with(as_ref!(String::from(".qrs")))) {
if !crate::quiche::check!(path_exists(cmd.clone())) {
println!("{}", String::from("Error: File '") + cmd + String::from("' not found."));
crate::quiche::check!(exit(1));
}
let mut parser = crate::quiche::check!(build_flag_parser(false));
let mut rest: Vec<String> = vec![];
let mut k = 1;
while k < crate::quiche::check!(args.len()) {
rest.push(args[k].clone());
k = k + 1;
}
let mut result = crate::quiche::check!(parser.parse(rest));
if crate::quiche::check!(result.errors.len()) > 0 {
for __q in (result.errors) {
let e = __q;
println!("{}", e);
}
crate::quiche::check!(print_usage());
crate::quiche::check!(exit(1));
}
let mut flags = crate::quiche::check!(parse_warn_flags(result.flags.clone()));
let mut emit_rust = false;
let mut k_er = String::from("emit-rust");
k_er = String::from("emit-rust");
emit_rust = crate::quiche::check!(get_flag_bool(result.flags.clone(), k_er));
if emit_rust {
let mut source = crate::quiche::check!(read_to_string(cmd.clone()));
let mut code = crate::quiche::check!(compile_source_to_string(source, cmd.clone(), flags.warn_quiche, flags.strict));
crate::quiche::check!(print_stdout(code));
return ;
}
if flags.warn_all {
flags.warn_quiche = true;
}
crate::quiche::check!(run_transpiled_file(cmd.clone(), result.positionals, true, false, true, flags.warn, flags.strict, flags.warn_quiche));
return ;
}
println!("{}", String::from("Error: Unrecognized command or file '") + cmd + String::from("'"));
println!("{}", String::from(""));
crate::quiche::check!(print_usage());
crate::quiche::check!(exit(1));
}

