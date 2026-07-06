#!/usr/bin/env python3
import os
import sys
import argparse
import subprocess
import re
import time

# Define lists of words to audit
HYPE_WORDS = [
    "revolutionary", "groundbreaking", "paradigm shift", "unparalleled", 
    "magic", "unprecedented", "ultimate", "silver bullet", 
    "breakthrough", "game-changing", "disruptive", "pioneering"
]

OVERCLAIMS = [
    "perfectly", "infinitely", "flawless", "trivially", 
    "absolute guarantee", "100%", "solves the halting problem", 
    "impossible to fail", "unhackable"
]

# Six allowed Chatman Equations in normalized forms
# Regexes with flexible whitespace/macro/delimiter/spacing supports
MS = r'(?:\\,|\\;|\\!|\\ |\\quad|\\qquad|\\hfill|\\  |\\   |\\    |\s)*'
L_PAREN = r'(?:\\left\s*\(|\\bigl\(|\\Bigl\(|\()'
R_PAREN = r'(?:\\right\s*\)|\\bigr\)|\\Bigr\)|\))'

A_SYM = r'(?:\\mathcal\s*\{\s*A\s*\}|\\Act)'
R_SYM = r'(?:\\mathcal\s*\{\s*R\s*\}|\\Rec)'
O_SYM = r'(?:\\mathcal\s*\{\s*O\s*\}|\\Obs)'
O_STAR_SYM = r'(?:(?:\\mathcal\s*\{\s*O\s*\}|\\Obs)' + MS + r'\^' + MS + r'(?:\{\s*\*\s*\}|\*)|\\Adm)'
MU_SYM = r'(?:\\mu|\\muop)'
ALPHA_SYM = r'(?:\\alpha|\\adm)'

ALLOWED_EQUATIONS = [
    # 1. \mathcal{A}=\mu(\mathcal{O}^{*})
    A_SYM + MS + r'=' + MS + MU_SYM + MS + L_PAREN + MS + O_STAR_SYM + MS + R_PAREN,
    
    # 2. \mathcal{R}=\operatorname{receipt}(\mathcal{A})
    R_SYM + MS + r'=' + MS + r'(?:\\operatorname|\\text)' + MS + r'\{\s*receipt\s*\}' + MS + L_PAREN + MS + A_SYM + MS + R_PAREN,
    
    # 3. \mathcal{O} \xrightarrow{\alpha} \mathcal{O*} \xrightarrow{\mu} \mathcal{A} \xrightarrow{\operatorname{receipt}} \mathcal{R}
    O_SYM + MS + r'(?:\\xrightarrow' + MS + r'\{\s*' + ALPHA_SYM + r'\s*\}|\\to)' + MS + O_STAR_SYM + MS + r'(?:\\xrightarrow' + MS + r'\{\s*' + MU_SYM + r'\s*\}|\\to)' + MS + A_SYM + MS + r'(?:\\xrightarrow' + MS + r'\{\s*(?:\\operatorname|\\text)' + MS + r'\{\s*receipt\s*\}\s*\}|\\to)' + MS + R_SYM,
    
    # 4. \mu_{\mathrm{prior}}=\mu\circ\alpha
    MU_SYM + MS + r'_\s*(?:\{\s*\\(?:mathrm|text|sf)' + MS + r'\{\s*prior\s*\}' + MS + r'\}|\{\s*prior\s*\}|prior)' + MS + r'=' + MS + MU_SYM + MS + r'\\circ' + MS + ALPHA_SYM,
    
    # 5. \mathcal{A}=\mu(\mathcal{O})
    A_SYM + MS + r'=' + MS + MU_SYM + MS + L_PAREN + MS + O_SYM + MS + R_PAREN,
    
    # 6. \mathcal{A}=\mu(\mathcal{O}^{*}) \quad\text{with}\quad \mathcal{O}^{*}=\operatorname{im}(\alpha), \qquad \alpha:\mathcal{O}\rightharpoonup\mathcal{O}^{*}\cup\{\bot\}
    A_SYM + MS + r'=' + MS + MU_SYM + MS + L_PAREN + MS + O_STAR_SYM + MS + R_PAREN + r'(?:(?!\r?\n\s*\r?\n)[\s\S])*?' + O_STAR_SYM + MS + r'=' + MS + r'(?:\\operatorname|\\text)' + MS + r'\{\s*im\s*\}' + MS + L_PAREN + MS + ALPHA_SYM + MS + R_PAREN + r'(?:(?!\r?\n\s*\r?\n)[\s\S])*?' + ALPHA_SYM + MS + r':' + MS + O_SYM + MS + r'(?:\\rightharpoonup|\\to)' + MS + O_STAR_SYM + MS + r'\\cup' + MS + r'(?:\\\{\s*(?:\\bot|\\Rfsl)\s*\\\}|(?:\\bot|\\Rfsl))'
]

def strip_comments(text):
    """Strip LaTeX comments, ignoring escaped percent signs (\%)."""
    return re.sub(r'((?<!\\)(?:\\\\)*)%.*', r'\1', text)

def get_math_spans(text):
    """Find all math mode spans in the LaTeX text."""
    spans = []
    # 1. Matches \begin{env}... \end{env}
    env_pattern = r'\\begin\{(equation|align|gather|multline|eqnarray|math|displaymath|split|cases)\*?\}([\s\S]*?)\\end\{\1\*?\}'
    for m in re.finditer(env_pattern, text):
        spans.append((m.start(), m.end()))
        
    # 2. Matches \[ ... \]
    for m in re.finditer(r'\\\[([\s\S]*?)\\\]', text):
        spans.append((m.start(), m.end()))
        
    # 3. Matches \( ... \)
    for m in re.finditer(r'\\\(([\s\S]*?)\\\)', text):
        spans.append((m.start(), m.end()))

    # 4. Matches $$...$$
    for m in re.finditer(r'\$\$([\s\S]*?)\$\$', text):
        spans.append((m.start(), m.end()))

    # 5. Matches $...$ (unescaped)
    dollar_indices = [m.start() for m in re.finditer(r'(?<!\\)\$', text)]
    i = 0
    clean_dollars = []
    while i < len(dollar_indices):
        if i + 1 < len(dollar_indices) and dollar_indices[i+1] == dollar_indices[i] + 1:
            i += 2
        else:
            clean_dollars.append(dollar_indices[i])
            i += 1
            
    for j in range(0, len(clean_dollars) - 1, 2):
        spans.append((clean_dollars[j], clean_dollars[j+1] + 1))
        
    spans.sort(key=lambda x: x[0])
    merged = []
    for start, end in spans:
        if not merged:
            merged.append((start, end))
        else:
            prev_start, prev_end = merged[-1]
            if start < prev_end:
                merged[-1] = (prev_start, max(prev_end, end))
            else:
                merged.append((start, end))
    return merged

def is_inside_math(start, end, spans):
    """Check if the given start/end range is inside any of the math mode spans."""
    for s, e in spans:
        if s <= start and end <= e:
            return True
    return False

def run_compilation(target_file, engine, output_dir):
    """Compile LaTeX document using pdflatex or latexmk."""
    if not os.path.exists(target_file):
        return False, f"Target file '{target_file}' not found.", []

    os.makedirs(output_dir, exist_ok=True)
    
    # Construct subprocess command
    if engine == 'latexmk':
        cmd = ['latexmk', '-pdf', '-interaction=nonstopmode', f'-outdir={output_dir}', target_file]
    else:  # pdflatex
        cmd = ['pdflatex', '-interaction=nonstopmode', f'-output-directory={output_dir}', target_file]

    # Run compilation
    try:
        result = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=60
        )
        success = (result.returncode == 0)
    except subprocess.TimeoutExpired:
        return False, "Compilation timed out after 60 seconds.", ["TIMEOUT"]
    except Exception as e:
        return False, f"Execution failed: {str(e)}", []

    # Read log file to collect warnings/errors
    base_name = os.path.splitext(os.path.basename(target_file))[0]
    log_file = os.path.join(output_dir, f"{base_name}.log")
    log_warnings_errors = []
    
    if os.path.exists(log_file):
        try:
            with open(log_file, 'r', encoding='utf-8', errors='ignore') as f:
                for line in f:
                    line_lower = line.lower()
                    if "warning" in line_lower or "error" in line_lower or "undefined control sequence" in line_lower or "undefined" in line_lower:
                        log_warnings_errors.append(line.strip())
        except Exception as e:
            log_warnings_errors.append(f"Failed to read log file: {str(e)}")

    # Clean up temporary build artifacts by default
    temp_exts = ['.aux', '.log', '.out', '.toc', '.fls', '.fdb_latexmk', '.nav', '.snm', '.vrb']
    for ext in temp_exts:
        temp_file = os.path.join(output_dir, f"{base_name}{ext}")
        if os.path.exists(temp_file):
            try:
                os.remove(temp_file)
            except Exception:
                pass
        # Also check source directory in case engine outputted there
        source_dir = os.path.dirname(target_file)
        temp_file_source = os.path.join(source_dir, f"{base_name}{ext}")
        if os.path.exists(temp_file_source):
            try:
                os.remove(temp_file_source)
            except Exception:
                pass

    msg = f"Compilation succeeded. Command: {' '.join(cmd)}" if success else f"Compilation failed with exit code {result.returncode}. Command: {' '.join(cmd)}"
    return success, msg, log_warnings_errors

def audit_structure_and_content(file_path):
    """Verify theorem-to-proof matches, hype words, and overclaims."""
    if not os.path.exists(file_path):
        return False, ["Target file not found."], [], []

    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        content = f.read()

    content_no_comments = strip_comments(content)

    # 1. Theorem-to-proof match using stack/event-based parsing
    environments = []
    pattern = r'\\(begin|end)\{(theorem|lemma|corollary|proposition|definition|axiom|proof)\}'
    for match in re.finditer(pattern, content_no_comments):
        environments.append((match.group(1), match.group(2), match.start()))

    proof_requiring_envs = ("theorem", "lemma", "corollary", "proposition")
    all_thm_envs = ("theorem", "lemma", "corollary", "proposition", "definition", "axiom")

    thm_count = sum(1 for ev, e, _ in environments if ev == "begin" and e in all_thm_envs)
    proof_req_count = sum(1 for ev, e, _ in environments if ev == "begin" and e in proof_requiring_envs)
    proof_count = sum(1 for ev, e, _ in environments if ev == "begin" and e == "proof")

    structural_errors = []
    if proof_req_count != proof_count:
        structural_errors.append(f"Total environment count mismatch: {thm_count} theorem-like environment(s) vs {proof_count} proof(s).")

    # Sequence tracking
    thm_list = [] # list of (name, start_pos)
    proof_list = [] # list of start_pos
    
    open_stack = []
    for ev, env_name, pos in environments:
        if ev == "begin":
            open_stack.append((env_name, pos))
            if env_name in proof_requiring_envs:
                thm_list.append((env_name, pos))
            elif env_name == "proof":
                proof_list.append(pos)
        elif ev == "end":
            if open_stack and open_stack[-1][0] == env_name:
                open_stack.pop()

    proved_theorems = {}
    for proof_pos in proof_list:
        matched = False
        for thm_name, thm_pos in reversed(thm_list):
            if thm_pos < proof_pos and thm_pos not in proved_theorems:
                proved_theorems[thm_pos] = proof_pos
                matched = True
                break
        if not matched:
            line_num = content_no_comments.count("\n", 0, proof_pos) + 1
            structural_errors.append(f"Proof environment at line {line_num} has no preceding theorem-like environment.")

    for idx, (thm_name, thm_pos) in enumerate(thm_list):
        if thm_pos not in proved_theorems:
            line_num = content_no_comments.count("\n", 0, thm_pos) + 1
            if idx + 1 < len(thm_list):
                next_name = thm_list[idx+1][0]
                structural_errors.append(f"Theorem-like environment '{thm_name}' at line {line_num} has no matching proof before next theorem '{next_name}'.")
            else:
                structural_errors.append(f"Theorem-like environment '{thm_name}' at line {line_num} has no matching proof at the end of the document.")

    # 2. Hype words and overclaims check line-by-line
    hype_occurrences = []
    overclaim_occurrences = []
    
    lines = content.split('\n')
    for i, line in enumerate(lines):
        line_clean = strip_comments(line)
        line_lower = line_clean.lower()
        
        # Check hype words
        for w in HYPE_WORDS:
            if w in line_lower:
                hype_occurrences.append((i + 1, w, line.strip()))
                
        # Check overclaims
        for w in OVERCLAIMS:
            if w in line_lower:
                overclaim_occurrences.append((i + 1, w, line.strip()))

    success = (len(structural_errors) == 0 and len(hype_occurrences) == 0 and len(overclaim_occurrences) == 0)
    return success, structural_errors, hype_occurrences, overclaim_occurrences

def audit_notation_canon(file_path):
    """Enforce rules from master_notation_canon.md."""
    if not os.path.exists(file_path):
        return False, ["Target file not found."]

    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
        content = f.read()

    content_no_comments = strip_comments(content)
    violations = []

    # 1. Calligraphic \mathcal{O}, \mathcal{A}, \mathcal{R} restriction check
    allowed_intervals = []
    for eq_regex in ALLOWED_EQUATIONS:
        for m in re.finditer(eq_regex, content_no_comments, re.DOTALL):
            allowed_intervals.append((m.start(), m.end()))
            
    symbol_pattern = r'\\mathcal\s*\{\s*[OAR]\s*\}|\\Obs\b|\\Act\b|\\Rec\b|\\Adm\b'
    math_spans = get_math_spans(content_no_comments)
    for m in re.finditer(symbol_pattern, content_no_comments):
        start, end = m.start(), m.end()
        # Verify if it is inside math mode
        if not is_inside_math(start, end, math_spans):
            continue
            
        line_start = content_no_comments.rfind('\n', 0, start) + 1
        line_end = content_no_comments.find('\n', end)
        if line_end == -1: line_end = len(content_no_comments)
        line_text = content_no_comments[line_start:line_end]
        
        # Ignore definitions
        if 'newcommand' in line_text or 'providecommand' in line_text or 'DeclareMathOperator' in line_text:
            continue
            
        is_allowed = False
        for ast, aend in allowed_intervals:
            if ast <= start and end <= aend:
                is_allowed = True
                break
        if not is_allowed:
            line_num = content_no_comments.count('\n', 0, start) + 1
            violations.append(f"Notation Violation: Calligraphic/Macro symbol '{m.group(0)}' at line {line_num} is used outside the 6 allowed Chatman Equations.\nContext: {line_text.strip()}")

    # 2. \Phi and \Psi check
    # Check \Phi
    for m in re.finditer(r'\\Phi\b(_o)?', content_no_comments):
        start, end = m.start(), m.end()
        line_start = content_no_comments.rfind('\n', 0, start) + 1
        line_end = content_no_comments.find('\n', end)
        if line_end == -1: line_end = len(content_no_comments)
        line_text = content_no_comments[line_start:line_end]
        
        if 'newcommand' in line_text or 'providecommand' in line_text:
            continue
            
        has_subscript_o = (m.group(1) == '_o')
        if has_subscript_o:
            if 'Act' in line_text or 'mathcal{A}' in line_text or re.search(r'\\Phi_o\s*\(\s*a\s*\)', line_text):
                line_num = content_no_comments.count('\n', 0, start) + 1
                violations.append(f"Notation Violation: \\Phi_o is used with actions/artifacts at line {line_num}. \\Phi is reserved for pipeline aggregate denial only.\nContext: {line_text.strip()}")
        else:
            if 'Act' in line_text or 'mathcal{A}' in line_text or re.search(r'\\Phi\s*\(\s*a\s*\)', line_text) or 'commitment' in line_text.lower():
                line_num = content_no_comments.count('\n', 0, start) + 1
                violations.append(f"Notation Violation: \\Phi is used for commitment mapping at line {line_num}. Commitment mapping must use \\Psi, and \\Phi is reserved for pipeline aggregate denial.\nContext: {line_text.strip()}")

    # Check \Psi
    for m in re.finditer(r'\\Psi\b', content_no_comments):
        start, end = m.start(), m.end()
        line_start = content_no_comments.rfind('\n', 0, start) + 1
        line_end = content_no_comments.find('\n', end)
        if line_end == -1: line_end = len(content_no_comments)
        line_text = content_no_comments[line_start:line_end]
        
        if 'newcommand' in line_text or 'providecommand' in line_text:
            continue
            
        if 'stage' in line_text.lower() or 'denial' in line_text.lower() or 'Stage' in line_text:
            line_num = content_no_comments.count('\n', 0, start) + 1
            violations.append(f"Notation Violation: \\Psi is used in pipeline/denial context at line {line_num}. \\Psi is reserved for commitment mapping.\nContext: {line_text.strip()}")

    # 3. plain font A, O*, L for local ggen logs check
    lines = content.split('\n')
    line_starts = []
    current_pos = 0
    for line in lines:
        line_starts.append(current_pos)
        current_pos += len(line) + 1 # +1 for newline
        
    def get_line_num(pos):
        import bisect
        return bisect.bisect_right(line_starts, pos)

    for m in re.finditer(symbol_pattern, content):
        start, end = m.start(), m.end()
        if not is_inside_math(start, end, math_spans):
            continue
        line_num = get_line_num(start)
        line_text = lines[line_num - 1]
        line_clean = strip_comments(line_text)
        if 'newcommand' in line_clean or 'providecommand' in line_clean:
            continue
        if re.search(r'\bggen\b|\bfilesystem\b|(?<!\\)\bdelta\b', line_clean, re.IGNORECASE):
            violations.append(f"Notation Violation: Calligraphic/macro notation used in local ggen context at line {line_num}. ggen logs must use plain font A, O*, L.\nContext: {line_text.strip()}")

    # 4. morphism signature name check
    OBS_PAT = r'(?:\\mathcal\{O\}\s*\^\s*\{\s*\*\s*\}|\\mathcal\{O\}\s*\^\s*\*|\\mathcal\{O\}|\\Obs|\\Adm|O\s*\^\s*\{\s*\*\s*\}|O\s*\^\s*\*|O)'
    ACT_PAT = r'(?:\\mathcal\{A\}|\\Act|A)'
    MAP_PAT = r'(?:\\to|\\rightharpoonup|\\rightarrow)'
    MAPPING_PAT = r'(?:' + OBS_PAT + r'\s*' + MAP_PAT + r'\s*' + ACT_PAT + r'|' + ACT_PAT + r'\s*' + MAP_PAT + r'\s*' + OBS_PAT + r')'
    SYMBOL_PAT = r'((?:\\?[a-zA-Z0-9_]+|\{\s*[^{}]*\s*\}|\^)+)'
    morphism_regex = SYMBOL_PAT + r'\s*:\s*' + MAPPING_PAT
    
    for m in re.finditer(morphism_regex, content_no_comments):
        symbol = m.group(1).strip()
        line_start = content_no_comments.rfind('\n', 0, m.start()) + 1
        line_end = content_no_comments.find('\n', m.end())
        if line_end == -1: line_end = len(content_no_comments)
        line_text = content_no_comments[line_start:line_end]
        
        if 'newcommand' in line_text or 'providecommand' in line_text:
            continue
            
        allowed_morphisms = [
            r'^\\mu$', r'^\\muop$',
            r'^\\mu_ggen$', r'^\\muop_ggen$',
            r'^\\mu_\{ggen\}$', r'^\\muop_\{ggen\}$',
            r'^\\mu_\{\\text\{ggen\}\}$', r'^\\muop_\{\\text\{ggen\}\}$',
            r'^\\mu_\{\\mathrm\{ggen\}\}$', r'^\\muop_\{\\mathrm\{ggen\}\}$'
        ]
        symbol_normalized = re.sub(r'\s+', '', symbol)
        is_allowed = any(re.match(p, symbol_normalized) for p in allowed_morphisms)
        if not is_allowed:
            line_num = content_no_comments.count('\n', 0, m.start()) + 1
            violations.append(f"Notation Violation: Morphism signature at line {line_num} uses symbol '{symbol}' instead of \\mu or \\mu_ggen.\nContext: {line_text.strip()}")

    success = (len(violations) == 0)
    return success, violations

def write_markdown_report(report_path, target_file, compile_status, check_status, logs, structural_errors, hype_words, overclaims, notation_violations):
    """Write comprehensive test results to a markdown file."""
    with open(report_path, 'w', encoding='utf-8') as f:
        f.write(f"# E2E LaTeX Test Report\n\n")
        f.write(f"- **Target File:** `{target_file}`\n")
        f.write(f"- **Timestamp:** `{time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}`\n")
        
        # Summary table
        f.write("\n## Summary Table\n\n")
        f.write("| Step | Status | Issues Found | Details |\n")
        f.write("| --- | --- | --- | --- |\n")
        
        c_status = "PASS" if compile_status[0] else ("SKIP" if compile_status[0] is None else "FAIL")
        
        if check_status[0] is None:
            s_status = "SKIP"
            n_status = "SKIP"
            s_issues = 0
            n_issues = 0
        else:
            s_status = "PASS" if len(structural_errors) == 0 and len(hype_words) == 0 and len(overclaims) == 0 else "FAIL"
            n_status = "PASS" if len(notation_violations) == 0 else "FAIL"
            s_issues = len(structural_errors) + len(hype_words) + len(overclaims)
            n_issues = len(notation_violations)
            
        f.write(f"| Compilation | {c_status} | {len(logs) if logs else 0} | {compile_status[1]} |\n")
        f.write(f"| Structural & Content | {s_status} | {s_issues} | {len(structural_errors)} mismatches, {len(hype_words)} hype, {len(overclaims)} overclaims |\n")
        f.write(f"| Notation Canon | {n_status} | {n_issues} | {len(notation_violations)} violations |\n")
        
        # Details Sections
        f.write("\n## 1. Compilation & Logs\n\n")
        if compile_status[0] is None:
            f.write("Compilation was skipped.\n")
        else:
            f.write(f"**Result:** {compile_status[1]}\n\n")
            if logs:
                f.write("### Warnings & Errors from Log:\n")
                for log in logs:
                    f.write(f"- `{log}`\n")
            else:
                f.write("No warnings or errors found in compilation logs.\n")
                
        f.write("\n## 2. Structural & Content Audit\n\n")
        if check_status[0] is None:
            f.write("Structural & Content audit was skipped.\n")
        else:
            if structural_errors:
                f.write("### Theorem-Proof Structural Mismatches:\n")
                for err in structural_errors:
                    f.write(f"- ❌ {err}\n")
            else:
                f.write("- ✅ All theorem-like environments have matching proofs.\n")
                
            if hype_words:
                f.write("\n### Hype Word Violations:\n")
                for line, word, context in hype_words:
                    f.write(f"- ❌ Line {line}: found '{word}' in `{context}`\n")
            else:
                f.write("- ✅ No hype words found.\n")
                
            if overclaims:
                f.write("\n### Overclaim Violations:\n")
                for line, word, context in overclaims:
                    f.write(f"- ❌ Line {line}: found '{word}' in `{context}`\n")
            else:
                f.write("- ✅ No overclaim words found.\n")

        f.write("\n## 3. Notation Canon Audit\n\n")
        if check_status[0] is None:
            f.write("Notation Canon audit was skipped.\n")
        else:
            if notation_violations:
                f.write("### Notation Canon Violations:\n")
                for viol in notation_violations:
                    f.write(f"- ❌ {viol}\n")
            else:
                f.write("- ✅ No notation canon violations found.\n")

        f.write("\n## Verdict\n\n")
        if compile_status[0] is False:
            f.write("🔴 **FAILED** (Compilation failure, Exit Code 2)\n")
        elif check_status[0] is not None and (len(structural_errors) > 0 or len(hype_words) > 0 or len(overclaims) > 0):
            f.write("🔴 **FAILED** (Structural / Content failure, Exit Code 3)\n")
        elif check_status[0] is not None and len(notation_violations) > 0:
            f.write("🔴 **FAILED** (Notation Canon failure, Exit Code 4)\n")
        else:
            f.write("🟢 **PASSED** (Exit Code 0)\n")

def main():
    parser = argparse.ArgumentParser(description="E2E LaTeX test runner and auditor.")
    parser.add_argument("--compile", action="store_true", help="Enable LaTeX compilation check.")
    parser.add_argument("--check", action="store_true", help="Enable structural and notation check.")
    parser.add_argument("--engine", choices=['latexmk', 'pdflatex'], default='pdflatex', help="LaTeX engine to use.")
    parser.add_argument("--target", required=True, help="Path to the target LaTeX file or directory.")
    parser.add_argument("--output-dir", default="docs/thesis/build", help="Output directory for compilation.")
    parser.add_argument("--report", default="docs/thesis/test_report.md", help="Markdown report file path.")

    args = parser.parse_args()

    # If neither flag is explicitly passed, run both checks by default
    run_compile = args.compile
    run_check = args.check
    if not args.compile and not args.check:
        run_compile = True
        run_check = True

    # Handle directory inputs cleanly by recursively scanning for LaTeX files
    if os.path.isdir(args.target):
        tex_files = []
        for root, _, files in os.walk(args.target):
            for f in files:
                if f.endswith('.tex'):
                    tex_files.append(os.path.join(root, f))
        if not tex_files:
            print(f"No LaTeX (.tex) files found in directory '{args.target}'.")
            sys.exit(1)
        tex_files.sort()
    else:
        tex_files = [args.target]

    def get_report_path(target_file, default_report):
        if len(tex_files) == 1:
            return default_report
        base = os.path.basename(target_file)
        dir_name = os.path.dirname(target_file)
        if base.startswith("stub_") and base.endswith(".tex"):
            rep_name = base.replace("stub_", "report_").replace(".tex", ".md")
        else:
            rep_name = base.replace(".tex", "_report.md")
        return os.path.join(dir_name, rep_name)

    overall_exit_code = 0

    for target_file in tex_files:
        compile_status = (None, "Skipped compilation.")
        logs = []
        
        structural_errors = []
        hype_words = []
        overclaims = []
        notation_violations = []
        
        check_status = (None, "Skipped audits.")
        file_exit_code = 0

        # 1. Run compilation
        if run_compile:
            success, msg, logs = run_compilation(target_file, args.engine, args.output_dir)
            compile_status = (success, msg)
            if not success:
                file_exit_code = 2
                check_status = (None, "Skipped due to compile failure")
            else:
                compile_status = (True, msg)
        else:
            success = True

        # 2. Run structural & notation audits if compilation succeeded or was skipped
        if success and run_check:
            struct_ok, structural_errors, hype_words, overclaims = audit_structure_and_content(target_file)
            notation_ok, notation_violations = audit_notation_canon(target_file)
            check_status = (struct_ok and notation_ok, "Checks executed.")
            if not struct_ok or len(hype_words) > 0 or len(overclaims) > 0:
                file_exit_code = 3
            elif not notation_ok:
                file_exit_code = 4

        report_file = get_report_path(target_file, args.report)
        write_markdown_report(
            report_file,
            target_file,
            compile_status,
            check_status,
            logs,
            structural_errors,
            hype_words,
            overclaims,
            notation_violations
        )

        print(f"=== E2E LaTeX Test Results for {target_file} ===")
        if run_compile:
            print(f"Compilation: {'PASS' if compile_status[0] else ('SKIP' if compile_status[0] is None else 'FAIL')}")
        if run_check and check_status[0] is not None:
            print(f"Structural Mismatches: {len(structural_errors)}")
            print(f"Hype words found: {len(hype_words)}")
            print(f"Overclaims found: {len(overclaims)}")
            print(f"Notation Violations: {len(notation_violations)}")

        if file_exit_code != 0 and overall_exit_code == 0:
            overall_exit_code = file_exit_code

    sys.exit(overall_exit_code)

if __name__ == "__main__":
    main()

