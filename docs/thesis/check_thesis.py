import os
import re

tex_files = [
    "00_foundations.tex",
    "01_admission_algebra.tex",
    "02_receipt_cryptography.tex",
    "03_planning_geometry.tex",
    "04_projection_and_scale.tex",
    "projection_thesis.tex",
    "synthesis_thesis.tex",
]

hype_words = ["revolutionary", "groundbreaking", "paradigm shift", "unparalleled", "magic", "unprecedented", "ultimate", "silver bullet", "breakthrough", "game-changing", "disruptive", "pioneering"]
overclaims = ["perfectly", "infinitely", "flawless", "trivially", "absolute guarantee", "100%", "solves the halting problem", "impossible to fail", "unhackable"]

for f in tex_files:
    if not os.path.exists(f): continue
    print(f"=== {f} ===")
    
    with open(f, 'r') as file:
        lines = file.readlines()
        
    theorems = []
    proofs = []
    
    for i, line in enumerate(lines):
        line_lower = line.lower()
        
        # Check theorems and proofs
        if "\\begin{theorem}" in line:
            theorems.append(i + 1)
        if "\\begin{lemma}" in line:
            theorems.append(i + 1)
        if "\\begin{corollary}" in line:
            theorems.append(i + 1)
        if "\\begin{proposition}" in line:
            theorems.append(i + 1)
            
        if "\\begin{proof}" in line:
            proofs.append(i + 1)
            
        # Check hype words
        for w in hype_words:
            if w in line_lower:
                print(f"Hype word '{w}' at line {i+1}: {line.strip()}")
                
        # Check overclaims
        for w in overclaims:
            if w in line_lower:
                print(f"Overclaim '{w}' at line {i+1}: {line.strip()}")

    print(f"Theorems/Lemmas/Props/Cors count: {len(theorems)}, Proofs count: {len(proofs)}")
    if len(theorems) != len(proofs):
        print(f"MISMATCH in {f}: {len(theorems)} theorems vs {len(proofs)} proofs")
        print(f"Theorems at: {theorems}")
        print(f"Proofs at: {proofs}")

print("\n=== LOG FILES ===")
log_files = [f.replace('.tex', '.log') for f in tex_files]
for log in log_files:
    if not os.path.exists(log): continue
    with open(log, 'r', encoding='latin-1') as file:
        lines = file.readlines()
        
    for i, line in enumerate(lines):
        if "undefined" in line.lower() and ("warning" in line.lower() or "error" in line.lower()):
            print(f"{log} Warning/Error: {line.strip()}")
