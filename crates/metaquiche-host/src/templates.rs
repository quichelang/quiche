// Project Scaffolding Templates
//
// This module provides template functions for both the host compiler
// and uses the shared templates from metaquiche-shared.

use metaquiche_shared::templates::{get_and_render, render, templates};

pub fn get_quiche_toml(name: &str) -> String {
    get_and_render("quiche_toml", &[("name", name)])
}

pub fn get_cargo_toml(name: &str, is_lib: bool, compiler_path: &str) -> String {
    let mut s = get_and_render(
        "cargo_toml",
        &[("name", name), ("compiler_path", compiler_path)],
    );

    if is_lib {
        s.push_str(templates().get_content("cargo_toml_lib_section"));
    } else {
        s.push_str(&get_and_render("cargo_toml_bin_section", &[("name", name)]));
    }
    s
}

pub fn get_build_rs() -> String {
    templates().get_content("build_rs").to_string()
}

pub fn get_lib_qrs() -> String {
    templates().get_content("lib_qrs").to_string()
}

pub fn get_lib_rs() -> String {
    let quiche_module = templates().get_content("quiche_module");
    get_and_render("lib_rs_wrapper", &[("quiche_module", quiche_module)])
}

pub fn get_main_qrs() -> String {
    templates().get_content("main_qrs").to_string()
}

pub fn get_main_rs() -> String {
    let quiche_module = templates().get_content("quiche_module");
    get_and_render("main_rs_wrapper", &[("quiche_module", quiche_module)])
}
