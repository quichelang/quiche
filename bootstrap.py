import os
import subprocess
import glob
import sys
import shutil

# Configuration
WORKSPACE_ROOT = os.getcwd()
QUICHE_SELF_DIR = os.path.join(WORKSPACE_ROOT, "crates", "quiche-self")
SRC_DIR = os.path.join(QUICHE_SELF_DIR, "src")
TARGET_DIR = os.path.join(WORKSPACE_ROOT, "target")
RUNTIME_PATH = os.path.join(WORKSPACE_ROOT, "crates", "runtime")

def find_stage0_out():
    pattern = os.path.join(TARGET_DIR, "debug", "build", "quiche_self-*", "out")
    candidates = glob.glob(pattern)
    if not candidates:
        print("Error: Could not find Stage 0 output directory. Run 'cargo build -p quiche_self' first.")
        sys.exit(1)
    return max(candidates, key=os.path.getmtime)

def setup_compilation_dir(stage_name, rust_sources_dir):
    """Sets up a Cargo project to compile the transpiled Rust code."""
    project_dir = os.path.join(TARGET_DIR, f"bootstrap_{stage_name}")
    if os.path.exists(project_dir):
        shutil.rmtree(project_dir)
    
    os.makedirs(os.path.join(project_dir, "src"))
    
    # Copy generated .rs files
    for f in glob.glob(os.path.join(rust_sources_dir, "*.rs")):
        shutil.copy(f, os.path.join(project_dir, "src"))
    
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
ruff_python_parser = {{ git = "https://github.com/astral-sh/ruff" }}
ruff_python_ast = {{ git = "https://github.com/astral-sh/ruff" }}
num-bigint = "0.4"

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
    
    bin_path = os.path.join(project_dir, "target", "debug", os.path.basename(project_dir))
    if os.name == 'nt': bin_path += ".exe"
    return bin_path

def run_transpile(binary_path, output_dir):
    if os.path.exists(output_dir):
        shutil.rmtree(output_dir)
    os.makedirs(output_dir)

    qrs_files = glob.glob(os.path.join(SRC_DIR, "*.qrs"))
    modules = [os.path.splitext(os.path.basename(f))[0] for f in qrs_files if os.path.basename(f) not in ["main.qrs", "lib.qrs"]]

    for qrs_file in qrs_files:
        basename = os.path.basename(qrs_file)
        stem = os.path.splitext(basename)[0]
        out_stem = stem
        if stem == "main": out_stem = "main_gen"
        output_file = os.path.join(output_dir, f"{out_stem}.rs")
        
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
            mod_decls = "".join([f"pub mod {m};\n" for m in sorted(modules)])
            rust_code = mod_decls + "\n" + rust_code
            
        with open(output_file, "w") as f:
            f.write(rust_code)

def compare_dirs(dir1, dir2):
    print(f"\nComparing {dir1} vs {dir2}...")
    files1 = glob.glob(os.path.join(dir1, "*.rs"))
    diffs = []
    
    for f1 in files1:
        basename = os.path.basename(f1)
        f2 = os.path.join(dir2, basename)
        
        if not os.path.exists(f2):
            diffs.append(f"Missing in {dir2}: {basename}")
            continue
            
        with open(f1, "r") as f: c1 = f.read()
        with open(f2, "r") as f: c2 = f.read()
        
        if c1 != c2:
            diffs.append(f"Content mismatch: {basename}")
        else:
            print(f"  MATCH: {basename}")

    return diffs

def main():
    # 0. Initial Build (Host -> Stage 1 Output)
    print("Step 0: Building Stage 0 binary with Host compiler...")
    subprocess.run(["cargo", "build", "-p", "quiche_self"], check=True)
    stage0_bin = os.path.join(TARGET_DIR, "debug", "quiche_self")
    if os.name == 'nt': stage0_bin += ".exe"
    
    # 1. Stage 0 Binary -> Stage 1 Output
    print("\nStep 1: Transpiling with Stage 0 binary -> Stage 1 Output...")
    stage1_out = os.path.join(TARGET_DIR, "stage1_out")
    run_transpile(stage0_bin, stage1_out)
    
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
