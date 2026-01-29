import os
import subprocess
import glob
import sys
import shutil
import re

# Configuration
WORKSPACE_ROOT = os.getcwd()
QUICHE_SELF_DIR = os.path.join(WORKSPACE_ROOT, "crates", "quiche-self")
SRC_DIR = os.path.join(QUICHE_SELF_DIR, "src")
TARGET_DIR = os.environ.get("CARGO_TARGET_DIR", os.path.join(WORKSPACE_ROOT, "target"))
RUNTIME_PATH = os.path.join(WORKSPACE_ROOT, "crates", "runtime")
PARSLEY_PATH = os.path.join(WORKSPACE_ROOT, "crates", "parsley-qrs")

def find_stage1_out():
    pattern = os.path.join(TARGET_DIR, "debug", "build", "quiche_self-*", "out")
    candidates = glob.glob(pattern)
    if not candidates:
        print("Error: Could not find Stage 1 output directory. Run 'cargo build -p quiche_self' first.")
        sys.exit(1)
    return max(candidates, key=os.path.getmtime)

def setup_compilation_dir(stage_name, rust_sources_dir):
    """Sets up a Cargo project to compile the transpiled Rust code."""
    project_dir = os.path.join(TARGET_DIR, f"bootstrap_{stage_name}")
    if os.path.exists(project_dir):
        shutil.rmtree(project_dir)
    
    os.makedirs(os.path.join(project_dir, "src"))

    # Copy generated .rs files (recursive)
    for root, _dirs, files in os.walk(rust_sources_dir):
        for name in files:
            if not name.endswith(".rs"):
                continue
            src_path = os.path.join(root, name)
            rel = os.path.relpath(src_path, rust_sources_dir)
            dest_path = os.path.join(project_dir, "src", rel)
            os.makedirs(os.path.dirname(dest_path), exist_ok=True)
            shutil.copy(src_path, dest_path)
    
    # Copy the wrapper main.rs from quiche-self/src
    shutil.copy(os.path.join(SRC_DIR, "main.rs"), os.path.join(project_dir, "src", "main.rs"))

    # Create Cargo.toml
    cargo_toml = f"""
[package]
name = "quiche_bootstrap_{stage_name}"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
quiche_runtime = {{ path = "{RUNTIME_PATH.replace('\\', '/')}" }}
parsley-qrs = {{ path = "{PARSLEY_PATH.replace('\\', '/')}" }}
ruff_python_parser = {{ git = "https://github.com/astral-sh/ruff" }}
ruff_python_ast = {{ git = "https://github.com/astral-sh/ruff" }}

[features]
bootstrap = []
"""
    with open(os.path.join(project_dir, "Cargo.toml"), "w") as f:
        f.write(cargo_toml)
    
    return project_dir

def compile_stage_binary(project_dir):
    print(f"Building binary in {project_dir}...")
    result = subprocess.run(["cargo", "build", "--quiet", "--features", "bootstrap"], cwd=project_dir)
    if result.returncode != 0:
        print(f"Failed to build binary in {project_dir}")
        sys.exit(1)

    dir_name = os.path.basename(project_dir)
    stage_name = dir_name
    if dir_name.startswith("bootstrap_"):
        stage_name = dir_name[len("bootstrap_"):]
    bin_name = f"quiche_bootstrap_{stage_name}"
    bin_path = os.path.join(project_dir, "target", "debug", bin_name)
    if os.name == 'nt': bin_path += ".exe"
    return bin_path

def _collect_qrs_files():
    qrs_files = []
    for root, _dirs, files in os.walk(SRC_DIR):
        for name in files:
            if name.endswith(".qrs"):
                qrs_files.append(os.path.join(root, name))
    return qrs_files

def _module_path_from_rel(rel_path):
    rel_dir, rel_file = os.path.split(rel_path)
    parts = []
    if rel_dir:
        parts.extend(rel_dir.split(os.sep))
    if rel_file == "mod.qrs":
        return ".".join(parts), True
    stem, _ = os.path.splitext(rel_file)
    parts.append(stem)
    return ".".join(parts), False

def _output_rel_from_rel(rel_path, is_mod):
    if is_mod:
        rel_dir = os.path.dirname(rel_path)
        if rel_dir:
            return os.path.join(rel_dir, "mod.rs")
        return "mod.rs"
    return os.path.splitext(rel_path)[0] + ".rs"

def run_transpile(binary_path, output_dir):
    if os.path.exists(output_dir):
        shutil.rmtree(output_dir)
    os.makedirs(output_dir)

    qrs_files = _collect_qrs_files()
    module_children = {}
    top_modules = set()

    for qrs_file in qrs_files:
        rel = os.path.relpath(qrs_file, SRC_DIR)
        module_path, _is_mod = _module_path_from_rel(rel)
        stem = os.path.splitext(os.path.basename(qrs_file))[0]
        if stem in ["main", "lib"]:
            continue
        if module_path:
            if "." not in module_path:
                top_modules.add(module_path)
            if "." in module_path:
                parent, child = module_path.rsplit(".", 1)
                module_children.setdefault(parent, set()).add(child)
            else:
                module_children.setdefault(module_path, set())

    for qrs_file in qrs_files:
        rel = os.path.relpath(qrs_file, SRC_DIR)
        basename = os.path.basename(qrs_file)
        stem = os.path.splitext(basename)[0]
        module_path, is_mod = _module_path_from_rel(rel)

        out_rel = _output_rel_from_rel(rel, is_mod)
        if stem == "main":
            out_rel = os.path.join(os.path.dirname(out_rel), "main_gen.rs")
        output_file = os.path.join(output_dir, out_rel)
        out_dirname = os.path.dirname(output_file)
        if out_dirname:
            os.makedirs(out_dirname, exist_ok=True)
        
        # Run binary
        result = subprocess.run([binary_path, qrs_file], capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Error transpiling {basename} with {binary_path}:")
            print(result.stdout)
            print(result.stderr)
            sys.exit(1)
            
        rust_code = result.stdout

        # Prepend mod decls to root
        if stem == "main":
            mod_decls = "".join([f"pub mod {m};\n" for m in sorted(top_modules)])
            rust_code = mod_decls + "\n" + rust_code

        # Prepend child module decls to mod.rs
        if is_mod and module_path in module_children:
            children = sorted(module_children[module_path])
            if children:
                mod_decls = "".join([f"pub mod {m};\n" for m in children])
                rust_code = mod_decls + "\n" + rust_code
        
        with open(output_file, "w") as f:
            f.write(rust_code)

def compare_dirs(dir1, dir2):
    print(f"\nComparing {dir1} vs {dir2}...")
    files1 = glob.glob(os.path.join(dir1, "**", "*.rs"), recursive=True)
    diffs = []
    
    for f1 in files1:
        rel = os.path.relpath(f1, dir1)
        f2 = os.path.join(dir2, rel)
        
        if not os.path.exists(f2):
            diffs.append(f"Missing in {dir2}: {rel}")
            continue
            
        with open(f1, "r") as f: c1 = f.read()
        with open(f2, "r") as f: c2 = f.read()
        
        if c1 != c2:
            diffs.append(f"Content mismatch: {rel}")
        else:
            print(f"  MATCH: {rel}")

    return diffs

def check_shadowing(root_dir):
    print("\nChecking for shadowed `let mut` declarations in Stage 1 output...")
    shadowed = []
    let_mut_re = re.compile(r"\blet\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\b")
    for path in glob.glob(os.path.join(root_dir, "**", "*.rs"), recursive=True):
        with open(path, "r") as f:
            text = f.read()
        scope_stack = [set()]
        line_no = 1
        for line in text.splitlines():
            for ch in line:
                if ch == "{":
                    scope_stack.append(set())
                elif ch == "}":
                    if len(scope_stack) > 1:
                        scope_stack.pop()
            m = let_mut_re.search(line)
            if m:
                name = m.group(1)
                if any(name in s for s in scope_stack[:-1]):
                    shadowed.append((path, line_no, name))
                scope_stack[-1].add(name)
            line_no += 1
    if not shadowed:
        print("  No shadowed `let mut` found.")
        return
    for path, line_no, name in shadowed:
        rel = os.path.relpath(path, WORKSPACE_ROOT)
        print(f"  Shadowed: {rel}:{line_no} -> {name}")

def main():
    # 0. Initial Build (Host -> Stage 1 Output)
    print("Step 0: Building host binary (implicit stage) with the Rust compiler...")
    subprocess.run(["cargo", "build", "-p", "quiche_self"], check=True)
    host_bin = os.path.join(TARGET_DIR, "debug", "quiche_self")
    if os.name == 'nt': host_bin += ".exe"

    # 1. Host Binary -> Stage 1 Output
    print("\nStep 1: Transpiling with host binary -> Stage 1 Output...")
    stage1_out = os.path.join(TARGET_DIR, "stage1_out")
    run_transpile(host_bin, stage1_out)
    check_shadowing(stage1_out)
    
    # 2. Compile Stage 1 Output -> Stage 1 Binary
    print("\nStep 2: Compiling Stage 1 Output into Stage 1 Binary...")
    stage1_proj = setup_compilation_dir("stage1", stage1_out)
    stage1_bin = compile_stage_binary(stage1_proj)

    # 3. Stage 1 Binary -> Stage 2 Output
    print("\nStep 3: Transpiling with Stage 1 binary -> Stage 2 Output...")
    stage2_out = os.path.join(TARGET_DIR, "stage2_out")
    run_transpile(stage1_bin, stage2_out)

    # 4. Verification: Stage 1 Output == Stage 2 Output
    diffs = compare_dirs(stage1_out, stage2_out)
    
    if diffs:
        print("\n[FAIL] Self-hosting verification failed!")
        for d in diffs:
            print(f"  - {d}")
        sys.exit(1)
    else:
        print("\n[SUCCESS] Self-hosting verified! Stage 1 == Stage 2.")

if __name__ == "__main__":
    main()
