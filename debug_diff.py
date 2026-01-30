
import os
import sys

def normalize(text):
    return " ".join(text.split())

def find_mismatch(file1, file2):
    with open(file1, 'r') as f1, open(file2, 'r') as f2:
        c1 = f1.read()
        c2 = f2.read()
    
    n1 = normalize(c1)
    n2 = normalize(c2)
    
    if n1 == n2:
        print(f"Files {file1} and {file2} match (normalized).")
        return

    print(f"Files {file1} and {file2} MISMATCH.")
    len_min = min(len(n1), len(n2))
    
    for i in range(len_min):
        if n1[i] != n2[i]:
            print(f"Mismatch at index {i}:")
            print(f"File 1 (...): {n1[max(0, i-20):i+20]}")
            print(f"File 2 (...): {n2[max(0, i-20):i+20]}")
            print(f"Char 1: '{n1[i]}'")
            print(f"Char 2: '{n2[i]}'")
            return
            
    if len(n1) != len(n2):
        print(f"Length mismatch: {len(n1)} vs {len(n2)}")
        print(f"Extra content starts at {len_min}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python3 debug_diff.py <file1> <file2>")
        # Fallback to hardcoded for testing if no args
        import glob
        try:
            p1 = glob.glob("target/stage1/debug/build/quiche-self-*/out/compiler/type_utils.rs")[0]
            p2 = glob.glob("target/stage2/debug/build/quiche-self-*/out/compiler/type_utils.rs")[0]
            print(f"Comparing default: {p1} vs {p2}")
            find_mismatch(p1, p2)
        except IndexError:
            print("Could not find default files.")
    else:
        find_mismatch(sys.argv[1], sys.argv[2])
