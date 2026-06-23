#!/usr/bin/env bash
set -euo pipefail

TEST_TEMP_DIR="/tmp/hollow-gate-test-$RANDOM"
mkdir -p "$TEST_TEMP_DIR/src"

# Compile hollow-gate in a clean workspace first
echo "Compiling hollow-gate..."
cp -r /Users/sac/praxis/template/tools/hollow-gate /tmp/hollow-gate-compile-test
(
    cd /tmp/hollow-gate-compile-test
    cargo build --release -j 1
)

HOLLOW_GATE_BIN="/tmp/hollow-gate-compile-test/target/release/hollow-gate"

# Helper to satisfy structural conformance checks
STRUCTURAL_BOILERPLATE="// PhantomData struct Raw; struct Validated; struct Admitted; struct Evidence trait Admit impl RulePackServer for"

# 1. Test case: todo!("custom message");
cat << EOF > "$TEST_TEMP_DIR/src/dummy.rs"
fn test() {
    todo!("custom message");
}
$STRUCTURAL_BOILERPLATE
EOF

echo "Running hollow-gate on todo!(\"custom message\")..."
if "$HOLLOW_GATE_BIN" "$TEST_TEMP_DIR" > /dev/null 2>&1; then
    echo "FAIL: hollow-gate allowed todo!(\"custom message\")"
    rm -rf "$TEST_TEMP_DIR" /tmp/hollow-gate-compile-test
    exit 1
else
    echo "SUCCESS: hollow-gate blocked todo!(\"custom message\")"
fi

# 2. Test case: unimplemented! { "custom" }
cat << EOF > "$TEST_TEMP_DIR/src/dummy.rs"
fn test() {
    unimplemented! { "custom" }
}
$STRUCTURAL_BOILERPLATE
EOF

echo "Running hollow-gate on unimplemented! { \"custom\" }..."
if "$HOLLOW_GATE_BIN" "$TEST_TEMP_DIR" > /dev/null 2>&1; then
    echo "FAIL: hollow-gate allowed unimplemented! { \"custom\" }"
    rm -rf "$TEST_TEMP_DIR" /tmp/hollow-gate-compile-test
    exit 1
else
    echo "SUCCESS: hollow-gate blocked unimplemented! { \"custom\" }"
fi

# 3. Test case: commented-out todo!
cat << EOF > "$TEST_TEMP_DIR/src/dummy.rs"
fn test() {
    // todo!("this is fine because it is commented out");
}
$STRUCTURAL_BOILERPLATE
EOF

echo "Running hollow-gate on commented-out todo!..."
if "$HOLLOW_GATE_BIN" "$TEST_TEMP_DIR" > /dev/null 2>&1; then
    echo "SUCCESS: hollow-gate allowed commented-out todo!"
else
    echo "FAIL: hollow-gate blocked commented-out todo!"
    rm -rf "$TEST_TEMP_DIR" /tmp/hollow-gate-compile-test
    exit 1
fi

rm -rf "$TEST_TEMP_DIR" /tmp/hollow-gate-compile-test
echo "All hollow-gate tests PASSED!"
