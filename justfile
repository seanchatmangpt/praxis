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

# Genesis Day 2 — Revenue Physics pipe, end to end in one command:
# observation -> ranked proposals -> top PDDL goal -> evidence-gated plan ->
# per-action law admit -> signed-chain receipt binding the proposal_hash (AR-9).
revenue-demo:
    cargo run --quiet --features proposer --bin revenue_demo

# The in-process conformance test for the same pipe (deterministic chain_hash,
# evidence-gate agreement, forced-admission refusal).
revenue-test:
    cargo test --features proposer --test revenue_pipe

# Append/refresh the [evidence] TOML block in Cargo.toml from a receipt file.
# Requires the `cicd-evidence-gen` CLI (crates/cicd-evidence-gen, cargo-cicd
# repo) on PATH. Mutates Cargo.toml — run manually, never in CI.
# Bump the crate-name/version pair here alongside Cargo.toml's own [package].
evidence:
    timeout 60s cicd-evidence-gen my-conforming-project 26.6.30 --receipt receipt.json --append Cargo.toml

# CI-mode: validate the existing [evidence] block against receipt.json,
# writing nothing. This is what `dod`'s soft evidence-check calls.
evidence-check:
    timeout 60s cicd-evidence-gen my-conforming-project 26.6.30 --receipt receipt.json --check
