set shell := ["bash", "-uc"]

# Run only tests affected by changes
test-changed:
    timeout 30s cargo cicd test changed

# Check target directory size and prune
clean-stale:
    timeout 30s cargo cicd target prune

# Build the workspace
build:
    timeout 120s cargo build

# Type-check the whole workspace with every feature enabled (what `doctor check` itself shells out to)
check:
    timeout 180s cargo check --workspace --all-features

# Run the full test suite across the workspace with every feature enabled (matches CI's `test` job)
test:
    timeout 600s cargo test --workspace --all-features

# Lint with the exact flags CI's `clippy` job uses — fails on any warning (.github/workflows/ci.yml)
clippy:
    timeout 300s cargo clippy --all-targets --all-features -- -D warnings

# Format the whole workspace in place
fmt:
    cargo fmt --all

# Format check only, no writes (matches CI's `fmt` job)
fmt-check:
    cargo fmt --all --check

# Holistic health check: build, config witness, frontier, tools, receipts, features. `just doctor format=json` for machine output
doctor format="text":
    cargo run --quiet --bin my-conforming-project --all-features -- doctor check --format {{format}}

# Build the capability-frontier DfCM matrix and print its summary + full report
frontier:
    cargo run --quiet --bin my-conforming-project --all-features -- frontier matrix

# The full local Definition-of-Done gate in CI order: check, test, clippy, then doctor (stops at first failure)
verify-all: check test clippy doctor
    @echo "verify-all: check + test + clippy + doctor all passed"

# Genesis Day 2 Revenue Physics pipe end to end: observation -> proposals -> goal -> plan -> admit -> receipt
revenue-demo:
    cargo run --quiet --features proposer --bin revenue_demo

# In-process conformance test for the revenue pipe (chain_hash determinism, evidence-gate agreement)
revenue-test:
    cargo test --features proposer --test revenue_pipe

# Membrane demo: drive the COMPLETE Day 2 revenue pipe through the MCP server over
# raw JSON-RPC (propose_revenue -> propose_goal -> plan_solve -> judge -> admit ->
# receipt -> whoami), proving an external agent completes a receipted mission with
# only membrane access. Ends with the receipt chain_hash + RECEIPTED AgentByte.
membrane-demo:
    ./scripts/membrane_demo.sh

# CI-runnable form of the membrane demo (spawns the server binary, no Python).
membrane-test:
    cargo test --features mcp,proposer --test membrane_mcp

# Append/refresh the [evidence] TOML block in Cargo.toml from a receipt file (requires cicd-evidence-gen on PATH; run manually, never in CI)
evidence:
    timeout 60s cicd-evidence-gen my-conforming-project 26.6.30 --receipt receipt.json --append Cargo.toml

# CI-mode: validate the existing [evidence] block against receipt.json, writing nothing (what `dod`'s soft evidence-check calls)
evidence-check:
    timeout 60s cicd-evidence-gen my-conforming-project 26.6.30 --receipt receipt.json --check
