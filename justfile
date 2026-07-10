set shell := ["bash", "-uc"]

# just recipes don't source ~/.zshrc, so RUSTC_WRAPPER=sccache (set there per docs/BUILD_CACHING.md)
# wouldn't reach cargo invocations run via `just` otherwise. `export` makes this an env var for
# every recipe below.
export RUSTC_WRAPPER := "sccache"

# NOTE: cargo takes an exclusive lock on `target/.cargo-lock` per invocation, scoped to
# CARGO_TARGET_DIR. Concurrent `just` invocations (e.g. multiple agents/terminals) against
# the same target dir serialize on that lock rather than running in parallel — this looks
# like a hang but is Cargo's own build-directory mutex, not a justfile bug. To run build/
# check/test/clippy recipes concurrently without queuing, override the target dir per
# invocation: `CARGO_TARGET_DIR=target/agent-2 just check`. Isolated dirs trade a slower
# first build (no shared incremental cache) for true concurrency.

# Check for a stray concurrent cargo build/test/check holding the target/ lock before
# starting a new one -- concurrent invocations serialize and silently double wall-clock time
check-lock:
    @ps aux | grep -E "cargo (test|build|check)" | grep -v grep || echo "no cargo build/test/check currently running"

# One-time: install sccache and print the shell-profile line to wire it up. Speeds up
# repeated compiles across this crate's many separate test binaries (shared deps like
# oxigraph/praxis_graphlaw get cached at the object level instead of recompiled per binary).
setup-sccache:
    command -v sccache >/dev/null 2>&1 || brew install sccache
    @echo "sccache installed. Add this to your shell profile (~/.zshrc):"
    @echo '    export RUSTC_WRAPPER=sccache'
    @echo "Then open a new shell and re-run your build; verify with: sccache --show-stats"

# Run only tests affected by changes
test-changed:
    timeout 30s cargo cicd test changed

# Check target directory size and prune
clean-stale:
    timeout 30s cargo cicd target prune

# NOTE: must invoke `cargo-cicd` (direct binary), not `cargo cicd` — the installed
# binary's clap parser rejects cargo's prepended arg.
# Refresh the praxis-standing.v1 index, standing-pack ontology, and docs/standing/REALITY_INDEX.md
standing:
    timeout 60s cargo-cicd standing refresh
    cp target/praxis-standing/standing.ttl ../cargo-cicd/plugins/cargo-cicd-kit/standing-pack/ontology.ttl
    timeout 120s cargo run --quiet -p ggen --bin ggen -- sync run
    timeout 60s cargo-cicd claude_context show
    @echo "just standing: refreshed target/praxis-standing/standing.json, regenerated docs/standing/REALITY_INDEX.md and target/praxis-standing/CLAUDE_CODE_CONTEXT.md"

# Build the workspace
build:
    timeout 120s cargo build

# NOTE: consumers pinning absolute paths (e.g. mfact -> target/debug/ggen) are
# intentionally not updated by this recipe.
# Install the release ggen binary to ~/.cargo/bin so the global `ggen` tracks this checkout
install-ggen:
    timeout 180s cargo install --path crates/ggen --force
    @ggen --version

# Type-check the whole workspace with every feature enabled (what `doctor check` itself shells out to)
check:
    timeout 180s cargo check --workspace --all-features

# Run the full test suite across the workspace with every feature enabled (matches CI's `test` job)
# nextest runs test binaries in parallel (vs. cargo test's serial-by-binary execution); falls
# back to cargo test if nextest isn't on PATH (see chatman-verify for the same pattern)
test:
    if command -v cargo-nextest >/dev/null 2>&1; then \
        timeout 600s cargo nextest run --workspace --all-features; \
    else \
        timeout 600s cargo test --workspace --all-features; \
    fi

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

# Drive the full Day 2 revenue pipe through the MCP server over raw JSON-RPC (membrane-only access)
membrane-demo:
    ./scripts/membrane_demo.sh

# CI-runnable form of the membrane demo (spawns the server binary, no Python).
membrane-test:
    cargo test --features mcp,proposer --test membrane_mcp

# NOTE: wasm-opt is disabled in the crate's [package.metadata.wasm-pack] — the emitted
# bulk-memory ops aren't accepted by the installed wasm-opt (size cost only).
# Build the GraphLaw WASM bridge to crates/praxis-graphlaw-wasm/pkg/ (consumed by playground/web)
wasm-playground:
    cd crates/praxis-graphlaw-wasm && wasm-pack build --target web --out-dir pkg --release
    @echo "just wasm-playground: built crates/praxis-graphlaw-wasm/pkg (consumed by playground/web via a file: dependency -- see playground/web/justfile)"

# Append/refresh the [evidence] TOML block in Cargo.toml from a receipt file (requires cicd-evidence-gen on PATH; run manually, never in CI)
evidence:
    timeout 60s cicd-evidence-gen my-conforming-project 26.6.30 --receipt receipt.json --append Cargo.toml

# CI-mode: validate the existing [evidence] block against receipt.json, writing nothing (what `dod`'s soft evidence-check calls)
evidence-check:
    timeout 60s cicd-evidence-gen my-conforming-project 26.6.30 --receipt receipt.json --check

# --- Chatman Engine v26.7.9 ---

# Fast chatman verification: tests (incl. static gates) + diagram atlas
chatman-verify:
    if command -v cargo-nextest >/dev/null 2>&1; then \
        timeout 600s cargo nextest run -p praxis-graphlaw -E 'test(chatman)'; \
    else \
        timeout 600s cargo test -p praxis-graphlaw chatman; \
    fi
    timeout 300s cargo test -p praxis-graphlaw --test chatman_static_gates
    python3 docs/chatman-engine/diagrams/atlas/verify_atlas.py

# Slow quality gates: mutation score, line coverage, dylint (requires cargo-mutants/llvm-cov/dylint)
chatman-quality:
    cargo mutants -p praxis-graphlaw --file 'src/chatman/*'
    cargo llvm-cov nextest -p praxis-graphlaw --fail-under-lines 85
    cargo dylint --all --workspace

# Idempotence check: ggen sync twice must leave generated chatman paths unchanged
chatman-sync-verify:
    timeout 120s cargo run --quiet -p ggen --bin ggen -- sync run
    timeout 120s cargo run --quiet -p ggen --bin ggen -- sync run
    git diff --exit-code -- 'crates/praxis-graphlaw/src/chatman' 'docs/chatman-engine'
