set shell := ["bash", "-uc"]

# Run only tests affected by changes
test-changed:
    timeout 30s cargo cicd test changed

# Check target directory size and prune
clean-stale:
    timeout 30s cargo cicd target prune

# Compile the praxis-standing.v1 index (target/praxis-standing/standing.json),
# copy it into the standing-pack for ggen, regenerate docs/standing/REALITY_INDEX.md.
# NOTE: invoked as `cargo-cicd ...` (direct binary name), not `cargo cicd ...` —
# the installed binary's clap parser rejects the `cicd` arg cargo's subcommand
# dispatch prepends (same issue affects `test-changed`/`clean-stale` above with
# this binary version; tracked separately, not fixed by this recipe).
# NOTE: `cargo-cicd standing refresh` used to embed a fresh `generated_at_utc`
# timestamp directly in standing.ttl on every run, so its content hash never
# matched a prior ggen.lock entry (ggen.lock's own [FM-PACK-008] error names
# "delete ggen.lock to intentionally re-lock" as the remediation for exactly
# this case) — forcing an `rm -f ggen.lock` before every invocation just to
# avoid a spurious lock mismatch. Fixed upstream in cargo-cicd
# (crates/cargo-cicd-core/src/standing/emit.rs::render_standing_ttl no longer
# includes generated_at_utc; the timestamp still lives in standing.json), so
# standing.ttl is now byte-identical across runs with unchanged artifact
# state and the `rm -f ggen.lock` workaround is no longer needed.
standing:
    timeout 60s cargo-cicd standing refresh
    cp target/praxis-standing/standing.ttl ../cargo-cicd/plugins/cargo-cicd-kit/standing-pack/ontology.ttl
    timeout 120s cargo run --quiet -p ggen --bin ggen -- sync run
    timeout 60s cargo-cicd claude_context show
    @echo "just standing: refreshed target/praxis-standing/standing.json, regenerated docs/standing/REALITY_INDEX.md and target/praxis-standing/CLAUDE_CODE_CONTEXT.md"

# Build the workspace
build:
    timeout 120s cargo build

# Build the release ggen binary and install it to ~/.cargo/bin/ggen, so the
# global `ggen` on PATH tracks this checkout instead of silently drifting to
# whatever version was last `cargo install`ed. Downstream consumers that pin
# an absolute path (e.g. mfact's justfile -> target/debug/ggen) are NOT
# updated by this recipe on purpose — it only fixes the global command.
# Run this after any change to crates/ggen you want reflected outside praxis.
install-ggen:
    timeout 180s cargo install --path crates/ggen --force
    @ggen --version

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

# Build the GraphLaw WASM bridge for the browser playground: Rust
# (crates/praxis-graphlaw-wasm) -> wasm32 -> wasm-bindgen JS/TS glue, output
# to crates/praxis-graphlaw-wasm/pkg/ (gitignored). This is the Rust->WASM
# half of the end-to-end chain; playground/web/justfile's `wasm`/`build-e2e`
# recipes call this one, then re-link pkg/ into playground/web/node_modules
# and build the Next.js app on top of it. wasm-opt is disabled (see
# crates/praxis-graphlaw-wasm/Cargo.toml's [package.metadata.wasm-pack]) --
# the emitted bulk-memory ops aren't accepted by the installed wasm-opt, and
# skipping it doesn't affect correctness, only binary size.
wasm-playground:
    cd crates/praxis-graphlaw-wasm && wasm-pack build --target web --out-dir pkg --release
    @echo "just wasm-playground: built crates/praxis-graphlaw-wasm/pkg (consumed by playground/web via a file: dependency -- see playground/web/justfile)"

# Append/refresh the [evidence] TOML block in Cargo.toml from a receipt file (requires cicd-evidence-gen on PATH; run manually, never in CI)
evidence:
    timeout 60s cicd-evidence-gen my-conforming-project 26.6.30 --receipt receipt.json --append Cargo.toml

# CI-mode: validate the existing [evidence] block against receipt.json, writing nothing (what `dod`'s soft evidence-check calls)
evidence-check:
    timeout 60s cicd-evidence-gen my-conforming-project 26.6.30 --receipt receipt.json --check
