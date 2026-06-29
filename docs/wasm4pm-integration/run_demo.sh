#!/bin/bash
set -euo pipefail

# wasm4pm-integration Live Demo Testbed Script
# Runs validation, keygen, and conformance checking against Praxis artifacts.

WPM_CLI="node /Users/sac/wasm4pm/apps/wasm4pm/dist/bin/wpm.js"
OCEL_LOG="/Users/sac/praxis/ocel/anti_llm_cheat_lsp_ocel.json"
PNML_MODEL="/Users/sac/wasm4pm/petri_net_lawful_dispatch.pnml"
KEYS_DIR="/Users/sac/praxis/docs/wasm4pm-integration/keys"

echo "======================================================================"
echo "WASM4PM-INTEGRATION DEMO: Validation & Conformance Verification Loop"
echo "======================================================================"
echo "Target Log:   $OCEL_LOG"
echo "Target Model: $PNML_MODEL"
echo "======================================================================"

# Step 1: Execute Schema & Referential Integrity Validation
echo ""
echo "[Step 1] Running Schema & Referential Integrity Validation..."
echo "Command: $WPM_CLI validate $OCEL_LOG --format ocel"
echo "----------------------------------------------------------------------"

# We capture the output and ensure it passes validation.
# We do not fail on warnings (since sparse type warnings are expected).
set +e
VALIDATE_OUT=$($WPM_CLI validate "$OCEL_LOG" --format ocel 2>&1)
VALIDATE_STATUS=$?
set -e

echo "$VALIDATE_OUT"
echo "----------------------------------------------------------------------"
if [ $VALIDATE_STATUS -eq 0 ]; then
    echo "✔ Validation Verdict: PASSED (Referential integrity OK)"
else
    echo "✘ Validation Verdict: FAILED (Exit Code: $VALIDATE_STATUS)"
    exit 1
fi

# Step 2: Demonstrate Cryptographic Key Pair Generation
echo ""
echo "[Step 2] Generating ed25519 Cryptographic Keys..."
echo "Command: $WPM_CLI receipt keygen --dir $KEYS_DIR"
echo "----------------------------------------------------------------------"

if [ -d "$KEYS_DIR" ]; then
    rm -rf "$KEYS_DIR"
fi

KEYGEN_OUT=$($WPM_CLI receipt keygen --dir "$KEYS_DIR" 2>&1)
echo "$KEYGEN_OUT"
echo "----------------------------------------------------------------------"

if [ -f "$KEYS_DIR/signing.key" ] && [ -f "$KEYS_DIR/signing.pub" ]; then
    echo "✔ Key Generation Verdict: SUCCESS (Keys generated in $KEYS_DIR)"
else
    echo "✘ Key Generation Verdict: FAILED"
    exit 1
fi

# Step 3: Run Conformance Checking (Conforming Trace Prefix)
echo ""
echo "[Step 3] Running Conformance check for Conforming Trace Prefix..."
echo "Prefix: [guard_check]"
echo "Command: $WPM_CLI prefix-conformance -m $PNML_MODEL -p guard_check"
echo "----------------------------------------------------------------------"

set +e
CONFORM_OUT_1=$($WPM_CLI prefix-conformance -m "$PNML_MODEL" -p guard_check 2>&1)
CONFORM_STATUS_1=$?
set -e

echo "$CONFORM_OUT_1"
echo "----------------------------------------------------------------------"

# In prefix-conformance, a sequence that is valid but hasn't reached the end returns exit code 6
# with Report: FAKE-LIVE or similar, but "Valid so far?: Yes". Let's verify.
if echo "$CONFORM_OUT_1" | grep -q "Valid so far?:       Yes"; then
    echo "✔ Conformance Verdict: CONFORMING SO FAR (Sequence is valid under model constraints)"
else
    echo "✘ Conformance Verdict: NON-CONFORMING"
    exit 1
fi

# Step 4: Run Conformance Checking (Non-Conforming Trace Prefix)
echo ""
echo "[Step 4] Running Conformance check for Non-Conforming Trace Prefix..."
echo "Prefix: [guard_check, constraint_eval] (Illegal: admissible_mask token consumed)"
echo "Command: $WPM_CLI prefix-conformance -m $PNML_MODEL -p guard_check,constraint_eval"
echo "----------------------------------------------------------------------"

set +e
CONFORM_OUT_2=$($WPM_CLI prefix-conformance -m "$PNML_MODEL" -p guard_check,constraint_eval 2>&1)
CONFORM_STATUS_2=$?
set -e

echo "$CONFORM_OUT_2"
echo "----------------------------------------------------------------------"

if echo "$CONFORM_OUT_2" | grep -q "Valid so far?:       No"; then
    echo "✔ Non-conformance Detection Verdict: SUCCESS (Correctly blocked illegal sequence)"
else
    echo "✘ Non-conformance Detection Verdict: FAILED (Illegal sequence was not blocked)"
    exit 1
fi

echo ""
echo "======================================================================"
echo "WASM4PM-INTEGRATION DEMO: ALL STEPS EXECUTED SUCCESSFULLY"
echo "======================================================================"
exit 0
