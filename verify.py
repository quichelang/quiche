import os
import glob
import sys
import re
import argparse

def compare_dirs(dir1_pattern, dir2_pattern):
    # Resolve globs to find actual build out dirs (cargo adds random hashes)
    candidates1 = glob.glob(dir1_pattern)
    candidates2 = glob.glob(dir2_pattern)
    
    if not candidates1:
        print(f"Error: No directory found matching {dir1_pattern}")
        sys.exit(1)
    if not candidates2:
        print(f"Error: No directory found matching {dir2_pattern}")
        sys.exit(1)
        
    # Pick the most recent ones if multiple exist
    dir1 = max(candidates1, key=os.path.getmtime)
    dir2 = max(candidates2, key=os.path.getmtime)

    print(f"Comparing:\n  A: {dir1}\n  B: {dir2}")
    
    files1 = glob.glob(os.path.join(dir1, "**", "*.rs"), recursive=True)
    diffs = []
    
    for f1 in files1:
        rel = os.path.relpath(f1, dir1)
        f2 = os.path.join(dir2, rel)
        
        if not os.path.exists(f2):
            diffs.append(f"Missing in B: {rel}")
            continue
            
        with open(f1, "r") as f: c1 = f.read()
        with open(f2, "r") as f: c2 = f.read()
        
        # Normalize whitespace (strip and collapse multiple spaces/newlines to single space)
        c1_norm = " ".join(c1.split())
        c2_norm = " ".join(c2.split())
        
        if c1_norm != c2_norm:
            diffs.append(f"Content mismatch: {rel}")

    if diffs:
        print("\n[FAIL] Mismatches found:")
        for d in diffs:
            print(f"  - {d}")
        sys.exit(1)
    else:
        print("\n[SUCCESS] No differences found.")

def check_shadowing(root_dir):
    print(f"Checking for shadowed `let mut` declarations in {root_dir}...")
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
        print(f"  Shadowed: {path}:{line_no} -> {name}")

def show_diff(dir1_pattern, dir2_pattern):
    """Show actual content differences between stage outputs."""
    import difflib
    import subprocess
    import tempfile
    
    # Resolve globs to find actual build out dirs
    candidates1 = glob.glob(dir1_pattern)
    candidates2 = glob.glob(dir2_pattern)
    
    if not candidates1:
        print(f"Error: No directory found matching {dir1_pattern}")
        sys.exit(1)
    if not candidates2:
        print(f"Error: No directory found matching {dir2_pattern}")
        sys.exit(1)
        
    dir1 = max(candidates1, key=os.path.getmtime)
    dir2 = max(candidates2, key=os.path.getmtime)

    print(f"Comparing:\n  Stage 1: {dir1}\n  Stage 2: {dir2}\n")
    
    files1 = glob.glob(os.path.join(dir1, "**", "*.rs"), recursive=True)
    all_diffs = []
    
    for f1 in files1:
        rel = os.path.relpath(f1, dir1)
        f2 = os.path.join(dir2, rel)
        
        if not os.path.exists(f2):
            all_diffs.append(f"=== Missing in Stage 2: {rel} ===\n")
            continue
            
        with open(f1, "r") as f: c1 = f.readlines()
        with open(f2, "r") as f: c2 = f.readlines()
        
        diff = list(difflib.unified_diff(c1, c2, 
                                          fromfile=f"Stage1/{rel}", 
                                          tofile=f"Stage2/{rel}",
                                          lineterm=""))
        if diff:
            all_diffs.append("\n".join(diff) + "\n")
    
    if not all_diffs:
        print("[SUCCESS] No differences found between Stage 1 and Stage 2.")
        return
    
    # Write to temp file and open with pager
    combined = "\n".join(all_diffs)
    
    # Try to use less with color support, fall back to cat
    try:
        pager = os.environ.get("PAGER", "less -R")
        with tempfile.NamedTemporaryFile(mode="w", suffix=".diff", delete=False) as tmp:
            tmp.write(combined)
            tmp_path = tmp.name
        subprocess.run(f"{pager} {tmp_path}", shell=True)
        os.unlink(tmp_path)
    except Exception:
        print(combined)

def main():
    parser = argparse.ArgumentParser(description="Verification utility")
    subparsers = parser.add_subparsers(dest="command", required=True)
    
    diff_parser = subparsers.add_parser("diff", help="Compare two directories (pass/fail)")
    diff_parser.add_argument("dir1", help="First directory (supports glob)")
    diff_parser.add_argument("dir2", help="Second directory (supports glob)")
    
    show_diff_parser = subparsers.add_parser("show-diff", help="Show actual differences with pager")
    show_diff_parser.add_argument("dir1", help="First directory (supports glob)")
    show_diff_parser.add_argument("dir2", help="Second directory (supports glob)")
    
    shadow_parser = subparsers.add_parser("shadow", help="Check for shadowed variables")
    shadow_parser.add_argument("dir", help="Directory to check")

    args = parser.parse_args()
    
    if args.command == "diff":
        compare_dirs(args.dir1, args.dir2)
    elif args.command == "show-diff":
        show_diff(args.dir1, args.dir2)
    elif args.command == "shadow":
        check_shadowing(args.dir)

if __name__ == "__main__":
    main()

