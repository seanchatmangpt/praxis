#!/usr/bin/env python3
import os
import subprocess
import sys
import tempfile
import shutil

RUNNER_PATH = os.path.abspath("docs/thesis/run_tests.py")

def run_target(target_path, flags=None):
    if flags is None:
        flags = ["--check"]
    cmd = [sys.executable, RUNNER_PATH, "--target", target_path] + flags
    result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    return result.returncode, result.stdout, result.stderr

def test_comment_stripping():
    print("--- Running Test: Comment Stripping ---")
    
    # Case 1: \% (escaped percent) - should NOT be stripped.
    # If it is stripped, it might break formulas or text.
    # Let's see if the runner keeps \%.
    content1 = r"""\documentclass{article}
\begin{document}
Our accuracy is 99\% which is not a comment.
\end{document}
"""
    
    # Case 2: \\% (newline backslash followed by comment percent)
    # The % is NOT escaped here because the preceding backslash is escaped by itself.
    # Therefore, this % starts a comment.
    # If the runner does not strip this comment, and the comment contains a notation violation:
    # E.g. \\% \mathcal{O}
    # It will trigger a notation violation because it wasn't stripped!
    content2 = r"""\documentclass{article}
\begin{document}
Some text \\% \mathcal{O}
\end{document}
"""

    # Case 3: \\\% (literal backslash, then escaped percent)
    # This % IS escaped. It should NOT be stripped.
    content3 = r"""\documentclass{article}
\begin{document}
Slash and percent: \\\%
\end{document}
"""

    # Case 4: \\\\% (four backslashes: two literal backslashes, then comment percent)
    # The % is NOT escaped. It starts a comment.
    # E.g., \\\\% \mathcal{A}
    content4 = r"""\documentclass{article}
\begin{document}
Text \\\\% \mathcal{A}
\end{document}
"""

    # Test Case 1
    with tempfile.NamedTemporaryFile(suffix=".tex", mode="w", delete=False) as f:
        f.write(content1)
        path1 = f.name
    code1, out1, err1 = run_target(path1)
    os.remove(path1)
    print(f"Case 1 (escaped percent %): Exit code {code1}")
    
    # Test Case 2
    with tempfile.NamedTemporaryFile(suffix=".tex", mode="w", delete=False) as f:
        f.write(content2)
        path2 = f.name
    code2, out2, err2 = run_target(path2)
    os.remove(path2)
    print(f"Case 2 (\\\\% followed by violation): Exit code {code2} (Expected: 4 if comment not stripped, 0 if stripped)")
    
    # Test Case 4
    with tempfile.NamedTemporaryFile(suffix=".tex", mode="w", delete=False) as f:
        f.write(content4)
        path4 = f.name
    code4, out4, err4 = run_target(path4)
    os.remove(path4)
    print(f"Case 4 (\\\\\\\\% followed by violation): Exit code {code4} (Expected: 4 if comment not stripped, 0 if stripped)")

def test_theorem_to_proof():
    print("\n--- Running Test: Theorem-to-Proof Parsing ---")
    
    # Case 1: Nested theorem environments
    # E.g., a lemma inside a theorem.
    content1 = r"""\documentclass{article}
\begin{document}
\begin{theorem}
\begin{lemma}
Inside lemma.
\end{lemma}
\end{theorem}
\begin{proof}
Proof for theorem/lemma.
\end{proof}
\end{document}
"""
    
    # Case 2: Multiple proofs for one theorem
    content2 = r"""\documentclass{article}
\begin{document}
\begin{theorem}
Thm statement.
\end{theorem}
\begin{proof}
First proof.
\end{proof}
\begin{proof}
Second proof.
\end{proof}
\end{document}
"""
    
    # Case 3: Case-sensitive env names in math blocks vs LaTeX
    content3 = r"""\documentclass{article}
\begin{document}
\begin{Theorem}
Thm statement.
\end{Theorem}
\begin{proof}
Proof.
\end{proof}
\end{document}
"""

    with tempfile.NamedTemporaryFile(suffix=".tex", mode="w", delete=False) as f:
        f.write(content1)
        path1 = f.name
    code1, out1, err1 = run_target(path1)
    os.remove(path1)
    print(f"Case 1 (Nested lemma): Exit code {code1} (Expected: 3 if nested not supported, 0 if supported)")
    if code1 != 0:
        print(f"  Stdout: {out1.strip()}")
        
    with tempfile.NamedTemporaryFile(suffix=".tex", mode="w", delete=False) as f:
        f.write(content2)
        path2 = f.name
    code2, out2, err2 = run_target(path2)
    os.remove(path2)
    print(f"Case 2 (Multiple proofs): Exit code {code2} (Expected: 3 if multi-proof fails, 0 if allowed)")
    if code2 != 0:
        print(f"  Stdout: {out2.strip()}")

def test_notation_canon():
    print("\n--- Running Test: Notation Canon checks ---")
    
    # Case 1: Equation 6 regex bypass due to re.DOTALL and .*?
    # We place Equation 6 components very far apart, and put intermediate illegal notation.
    content1 = r"""\documentclass{article}
\begin{document}
% First component of Equation 6
\mathcal{A}=\mu(\mathcal{O}^{*})

% Unrelated paragraph containing illegal notation \mathcal{O} that should be flagged
This is an illegal use of \mathcal{O} that is not inside any allowed equation.

% Second component of Equation 6
\mathcal{O}^{*}=\operatorname{im}(\alpha)

% Another unrelated paragraph with illegal notation \mathcal{A}
This is an illegal use of \mathcal{A}.

% Third component of Equation 6
\alpha:\mathcal{O}\rightharpoonup\mathcal{O}^{*}\cup\{\bot\}
\end{document}
"""
    
    # Case 2: Substring matching for 'Act' in line_text for \Phi and \Psi
    # 'Act' is in 'interaction' or 'action' (the word).
    content2 = r"""\documentclass{article}
\begin{document}
This interaction is modeled by \Phi.
\end{document}
"""
    
    # Case 3: plain font A, O*, L for local ggen logs check
    # Check if 'delta' triggers on greek letter \delta or word containing delta.
    content3 = r"""\documentclass{article}
\begin{document}
We have a morphism \delta : \mathcal{O} \to \mathcal{O}.
\end{document}
"""
    
    # Case 4: Morphism signature name check
    # url: feature.ttl or arbitrary label
    content4 = r"""\documentclass{article}
\begin{document}
The resource is url: feature.ttl.
\end{document}
"""

    with tempfile.NamedTemporaryFile(suffix=".tex", mode="w", delete=False) as f:
        f.write(content1)
        path1 = f.name
    code1, out1, err1 = run_target(path1)
    os.remove(path1)
    print(f"Case 1 (Eq 6 bypass): Exit code {code1} (Expected: 4 due to illegal \mathcal{{O}} in between, Actual: 0 if bypassed)")
    if code1 != 0:
        print(f"  Stdout: {out1.strip()}")

    with tempfile.NamedTemporaryFile(suffix=".tex", mode="w", delete=False) as f:
        f.write(content2)
        path2 = f.name
    code2, out2, err2 = run_target(path2)
    os.remove(path2)
    print(f"Case 2 (\\Phi on line with 'interaction'): Exit code {code2} (Expected: 0, Actual: {code2})")
    if code2 != 0:
        print(f"  Stdout: {out2.strip()}")

    with tempfile.NamedTemporaryFile(suffix=".tex", mode="w", delete=False) as f:
        f.write(content3)
        path3 = f.name
    code3, out3, err3 = run_target(path3)
    os.remove(path3)
    print(f"Case 3 (\\delta triggering ggen context): Exit code {code3} (Expected: 0, Actual: {code3})")
    if code3 != 0:
        print(f"  Stdout: {out3.strip()}")

    with tempfile.NamedTemporaryFile(suffix=".tex", mode="w", delete=False) as f:
        f.write(content4)
        path4 = f.name
    code4, out4, err4 = run_target(path4)
    os.remove(path4)
    print(f"Case 4 (url: feature.ttl morphism check): Exit code {code4} (Expected: 0, Actual: {code4})")
    if code4 != 0:
        print(f"  Stdout: {out4.strip()}")

def test_robustness():
    print("\n--- Running Test: Robustness ---")
    
    # Case 1: Target path is a directory
    temp_dir = tempfile.mkdtemp()
    code1, out1, err1 = run_target(temp_dir)
    shutil.rmtree(temp_dir)
    print(f"Case 1 (Target is directory): Exit code {code1}")
    if code1 != 0:
        print(f"  Error output snippet: {err1.strip()[:200]}")

if __name__ == "__main__":
    test_comment_stripping()
    test_theorem_to_proof()
    test_notation_canon()
    test_robustness()
