#!/usr/bin/env bash
set -euo pipefail

# verify_conformance.sh — Programmatic conformance verification script.
#
# Synthesises a conforming project from the template, verifies compilation,
# and programmatically asserts conformance to Post-Chatman principles.

TEMPLATE_DIR="/Users/sac/praxis/template"
TEMP_PROJECT_DIR="/tmp/my-conforming-project"
PROJECT_NAME="my-conforming-project"
PROJECT_NAME_SNAKE="my_conforming_project"
DESCRIPTION="A dynamically generated conforming project for verification."

echo "=== Conformance Verification Protocol ==="
echo "Template Source: $TEMPLATE_DIR"
echo "Target Temp Dir: $TEMP_PROJECT_DIR"

# 1. Clean previous generation
rm -rf "$TEMP_PROJECT_DIR"
mkdir -p "$TEMP_PROJECT_DIR"

# 2. Custom mock substitution generator (since cargo-generate is not installed)
echo "Generating project from template..."
python3 -c "
import os
import shutil

src = '$TEMPLATE_DIR'
dst = '$TEMP_PROJECT_DIR'

placeholders = {
    '{{project-name}}': '$PROJECT_NAME',
    '{{project_name}}': '$PROJECT_NAME_SNAKE',
    '{{description}}': '$DESCRIPTION'
}

for root, dirs, files in os.walk(src):
    # Exclude unwanted directories
    if '.git' in dirs:
        dirs.remove('.git')
    if 'target' in dirs:
        dirs.remove('target')
    
    # Recreate folder structure
    rel_path = os.path.relpath(root, src)
    if rel_path != '.':
        os.makedirs(os.path.join(dst, rel_path), exist_ok=True)
        
    for file in files:
        src_file = os.path.join(root, file)
        dst_file = os.path.join(dst, rel_path, file) if rel_path != '.' else os.path.join(dst, file)
        
        # Binary vs Text file check
        is_binary = False
        try:
            with open(src_file, 'tr') as check_file:
                check_file.read(512)
        except Exception:
            is_binary = True
            
        if is_binary:
            shutil.copy2(src_file, dst_file)
        else:
            with open(src_file, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
            for key, val in placeholders.items():
                content = content.replace(key, val)
            with open(dst_file, 'w', encoding='utf-8') as f:
                f.write(content)
"
echo "Project generated successfully."

# 3. Compile Verification
echo "Running compilation check..."
(
    cd "$TEMP_PROJECT_DIR"
    # Ensure offline compilation is used if needed
    cargo check --all-targets --all-features
)
echo "Compilation passed."

# 4. Structural Conformance Checking
echo "Scanning generated project for structural conformance..."

TYPES_RS="$TEMP_PROJECT_DIR/src/types.rs"
LSP_RS="$TEMP_PROJECT_DIR/src/lsp.rs"

# Helper function to assert pattern presence
assert_contains() {
    local file="$1"
    local pattern="$2"
    local desc="$3"
    if grep -qF "$pattern" "$file"; then
        echo "  [PASS] $desc"
    else
        echo "  [FAIL] $desc (Pattern not found: '$pattern' in $file)"
        exit 1
    fi
}

assert_contains "$TYPES_RS" "pub struct Evidence<T, S: sealed::EvidenceState, W>" "Evidence typestate structure is present in types.rs"
assert_contains "$TYPES_RS" "pub trait Admit" "Admit trait is present in types.rs"
assert_contains "$LSP_RS" "pub struct AppLspServer" "AppLspServer struct is present in lsp.rs"
assert_contains "$LSP_RS" "impl RulePackServer for AppLspServer" "RulePackServer implementation is present in lsp.rs"

echo "=== Conformance Verification Verdict: VERIFIED ==="
