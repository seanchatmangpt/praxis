#!/usr/bin/env bash
set -euo pipefail

# verify_conformance.sh — Programmatic conformance verification script.
#
# Synthesises a conforming project from the template, verifies compilation,
# and programmatically asserts conformance to Post-Chatman principles.

TEMPLATE_DIR="/Users/sac/praxis/backup_template"
TEMP_PROJECT_DIR="/tmp/my-conforming-project-$RANDOM"
PROJECT_NAME="my-conforming-project"
PROJECT_NAME_SNAKE="my_conforming_project"
DESCRIPTION="A dynamically generated conforming project for verification."

# Register cleanup function on exit
cleanup() {
    echo "Cleaning up temp project directory: $TEMP_PROJECT_DIR"
    rm -rf "$TEMP_PROJECT_DIR"
}
trap cleanup EXIT

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
echo "Running cargo check..."
(
    cd "$TEMP_PROJECT_DIR"
    cargo check -j 1
)
echo "cargo check passed."

echo "Running cargo check --all-features..."
(
    cd "$TEMP_PROJECT_DIR"
    cargo check --all-features -j 1
)
echo "cargo check --all-features passed."

# 4. Conformance checking logic (running updated hollow-gate tool)
echo "Running hollow-gate conformance verifier..."
(
    cd "$TEMP_PROJECT_DIR"
    cargo run -j 1 --manifest-path "tools/hollow-gate/Cargo.toml" -- "$TEMP_PROJECT_DIR"
)

# 5. CLI Execution Verification
echo "Running generated CLI output verification..."
(
    cd "$TEMP_PROJECT_DIR"
    cargo run -j 1 --bin "$PROJECT_NAME" -- --help
)
echo "CLI execution passed."

echo "=== Conformance Verification Verdict: VERIFIED ==="
