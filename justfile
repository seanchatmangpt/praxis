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
# cargo-cicd only plans (prints "would run"); this recipe extracts the affected file paths from
# that plan, derives the owning crate package(s) (crates/<pkg>/...), and actually executes
# nextest (falling back to cargo test) scoped to those packages — same nextest/fallback pattern
# as the `test` recipe above.
test-changed:
    #!/usr/bin/env bash
    set -euo pipefail
    plan=$(timeout 30s cargo-cicd test changed --format plain)
    echo "$plan"
    pkgs=$(echo "$plan" | grep -oE 'crates/[^/]+/' | sed 's#crates/##; s#/##' | sort -u)
    if [ -z "$pkgs" ]; then
        echo "just test-changed: no affected crate packages parsed from plan; nothing to run"
        exit 0
    fi
    pkg_args=""
    for p in $pkgs; do pkg_args="$pkg_args -p $p"; done
    echo "just test-changed: running affected packages:$pkg_args"
    # 1800s, not 600s: a cold build of an affected crate (e.g. praxis-graphlaw with a new
    # dependency) can spend 8-9 minutes on compilation alone before any test executes.
    if command -v cargo-nextest >/dev/null 2>&1; then
        timeout 1800s cargo nextest run $pkg_args --all-features
    else
        timeout 1800s cargo test $pkg_args --all-features
    fi

# Check target directory size and prune
clean-stale:
    timeout 30s cargo cicd target prune

# NOTE: must invoke `cargo-cicd` (direct binary), not `cargo cicd` — the installed
# binary's clap parser rejects cargo's prepended arg.
# Refresh the praxis-standing.v1 index, standing-pack ontology, and docs/standing/REALITY_INDEX.md
standing:
    command -v ggen >/dev/null || (echo "ggen not found on PATH — run: cargo install --path crates/ggen --locked" && exit 1)
    timeout 60s cargo-cicd standing refresh
    cp target/praxis-standing/standing.ttl ../cargo-cicd/plugins/cargo-cicd-kit/standing-pack/ontology.ttl
    timeout 120s ggen sync run
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

# Run workspace benchmarks. `just bench filter="bench_name"` to scope to one benchmark target
bench filter="":
    timeout 600s cargo bench {{ if filter != "" { "--bench " + filter } else { "" } }}

# Line/branch coverage report via tarpaulin (installs it if missing). `just coverage out="Html"` for other tarpaulin --out formats
coverage out="Html":
    command -v cargo-tarpaulin >/dev/null 2>&1 || cargo install cargo-tarpaulin --locked
    timeout 600s cargo tarpaulin --out {{out}} --output-dir coverage --exclude-files "tests/*"

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

# List test names for a given package/test-binary pattern without running them, e.g.
# `just nextest-list -p praxis-graphlaw --test 'shex_validation_*'`
nextest-list *args:
    timeout 120s cargo nextest list {{args}}

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

# --- cng CLI (crates/cng) ---

# Build the cng CLI binary
cng-build:
    timeout 600s cargo build -p cng

# Run the cng CLI with arguments (e.g. `just cng-run plan generate --dir plans/`)
cng-run *args:
    timeout 300s cargo run -q -p cng -- {{args}}

# Build/run the cng CLI with the bench feature (Fortune-5 benchmark verbs)
cng-bench-build:
    timeout 900s cargo build --release -p cng --features bench

cng-bench *args:
    timeout 3600s cargo run -q --release -p cng --features bench --bin cng -- {{args}}

# Replay/verify leg of the benchmark campaign: re-manufactures against
# digests.json and re-validates exported POWL (see bench.rs verify()).
cng-bench-verify dir:
    timeout 3600s cargo run -q --release -p cng --features bench --bin cng -- benchmark verify --dir {{dir}}

# Independent auditor replay from a self-contained bundle (no producer state).
cng-evidence-replay bundle:
    timeout 3600s cargo run -q --release -p cng --features bench --bin cng -- evidence replay --bundle {{bundle}}

# Single-operator workday benchmark (PROJ-608): deterministic logical-tick
# day with the standing-next-action law and bounded admission → resume.
cng-workday *args:
    timeout 3600s cargo run -q --release -p cng --features bench --bin cng -- benchmark workday {{args}}

# PROJ-616 verification harness: run the SAME-SEED workday twice into two
# fresh directories and byte-compare the full evidence bundles — every file
# under both output trees (obs/, roster/, evidence/ocel.nt, admissions/,
# dispatch/, generated/, ticks/, results/) except workday-report.json, which
# is compared after removing its path-derived "out_dir" line. Any drift
# exits nonzero. Debug profile on purpose (no --release): this is a
# correctness gate, not a throughput benchmark (see docs/BUILD_CACHING.md).
cng-workday-verify seed="616" ticks="8" rpm="125":
    #!/usr/bin/env bash
    set -euo pipefail
    root="target/cng-workday-verify"
    rm -rf "$root/a" "$root/b"
    timeout 3600s cargo run -q -p cng --features bench --bin cng -- \
        benchmark workday --out "$root/a" --seed {{seed}} --ticks {{ticks}} --refusal-per-mille {{rpm}}
    timeout 3600s cargo run -q -p cng --features bench --bin cng -- \
        benchmark workday --out "$root/b" --seed {{seed}} --ticks {{ticks}} --refusal-per-mille {{rpm}}
    diff -r --exclude=workday-report.json "$root/a" "$root/b"
    diff <(grep -v '"out_dir"' "$root/a/results/workday-report.json") \
         <(grep -v '"out_dir"' "$root/b/results/workday-report.json")
    echo "cng-workday-verify: byte-identical evidence bundles (seed={{seed}}, ticks={{ticks}}, rpm={{rpm}})"

# Multi-engine worker process (PROJ-723): bounded receipted poll loop over
# <root>/engines/<id>/inbox; consequences to its outbox; quiesce.ttl ends it.
# e.g. `just cng-engine-serve --root target/engines --engine-id H --seed 42 --max-polls 64`
cng-engine-serve *args:
    timeout 3600s cargo run -q -p cng --features bench --bin cng -- engine serve {{args}}

# Crash-resume (PROJ-724): reload ledger tail + processed set, verify the
# receipt-chain prefix (torn tail refuses CNG_R11), continue the serve loop.
cng-engine-resume *args:
    timeout 3600s cargo run -q -p cng --features bench --bin cng -- engine resume {{args}}

# Run the cng test suite
cng-test:
    timeout 600s cargo test -p cng

# Run the cng test suite with the bench feature (portability/audit tests)
cng-test-bench:
    timeout 900s cargo test -p cng --features bench

# Type-check the cng crate + its tests with the bench feature (fast inner
# loop before cng-test-one / cng-test-bench; catches compile errors without
# running anything).
cng-check:
    timeout 300s cargo check -p cng --features bench --tests

# Run ONE cng integration-test binary by exact name (scoped inner loop; avoids
# rebuilding/running the whole bench suite when investigating a single binary)
cng-test-one binary *args:
    timeout 1200s cargo test -p cng --features bench --test {{binary}} {{args}}

# Run ONLY the cng crate's in-crate unit tests (dispatch_test.rs, engine_test.rs,
# decomp_test.rs, etc. via #[cfg(test)] modules) — skips the heavier standalone
# integration-test binaries (cng_long_horizon_scenario and friends) entirely.
cng-test-lib *args:
    timeout 600s cargo test -p cng --features bench --lib {{args}}

# PROJ-728/729 multi-engine harness: coordinator + REAL engine OS processes
# over the filesystem transport — isolation falsifiers, G13 crash-resume,
# distributed determinism, cross-engine recursion. Exact --test scope;
# single-threaded because each test spawns its own engine processes.
cng-multi-engine:
    timeout 1800s cargo test -p cng --features bench --test cng_multi_engine -- --test-threads=1

# Package-surface smoke: install the cng binary from the crate and invoke it
cng-install-smoke:
    timeout 600s cargo install --path crates/cng --debug --root target/install-smoke --force
    target/install-smoke/bin/cng workflow doctor

# --- Isolated-target cargo recipes (concurrent-agent-safe) ---
#
# WHY: cargo takes an exclusive lock on `target/.cargo-lock` scoped to CARGO_TARGET_DIR (see
# the lock-contention NOTE at the top of this file). Concurrent Claude Code agents/terminals
# all hitting the default `target/` serialize on that lock rather than running in parallel --
# looks like a hang, isn't. This session hand-typed `CARGO_TARGET_DIR=target/agent-<name>
# cargo test ...` 20+ times, always slightly differently, to work around it; these recipes
# formalize that pattern so it's discoverable and consistent going forward.
#
# WHEN: any time you're running cargo alongside other concurrent Claude Code agent work in
# this repo. Pick a short, descriptive `name` (your ticket or feature slug) so `du -sh
# target/agent-*` stays legible later. Isolated dirs trade a slower first build (no shared
# incremental cache) for true concurrency -- and they are not self-cleaning, so run
# `cng-clean-isolated` when you're done with yours; they accumulate multi-GB each otherwise.

# Run one cng integration-test binary in an isolated target dir (concurrent-agent-safe; see
# note above), e.g. `just cng-test-isolated my-feature cng_decomp -- --nocapture`
cng-test-isolated name binary *args:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 1200s cargo test -p cng --features bench --test {{binary}} {{args}}

# Type-check the cng crate + its tests in an isolated target dir (concurrent-agent-safe; fast
# compile-only sanity check mid-edit, before cng-test-isolated)
cng-check-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 300s cargo check -p cng --features bench --tests

# Remove one isolated target dir (cleanup after an agent/dev is done with it)
cng-clean-isolated name:
    #!/usr/bin/env bash
    set -euo pipefail
    dir="target/agent-{{name}}"
    if [ -d "$dir" ]; then
        du -sh "$dir"
        rm -rf "$dir"
        echo "cng-clean-isolated: removed $dir"
    else
        echo "cng-clean-isolated: $dir does not exist, nothing to remove"
    fi

# Remove every isolated target dir in one shot (periodic cleanup). Only removes your own work
# if run mid-session by other agents too -- check `du -sh target/agent-*` first if you want to
# know what you're about to reclaim, since this takes everyone's isolated dirs at once.
cng-clean-all-isolated:
    rm -rf target/agent-*

# Search crates.io for a crate name (publishability checks)
crates-search name:
    timeout 60s cargo search {{name}} --limit 3

# Dry-run publish one workspace crate (no upload; packaging + verify build)
publish-dry-run crate:
    timeout 600s cargo publish -p {{crate}} --dry-run --allow-dirty

# Run one exact praxis-graphlaw integration-test binary (e.g. `just test-bin chatman_pddl_to_powl_projection`).
# --nocapture so artifact-path markers (GENERATED_POWL_TTL_PATH=, IMPORTED_PDDL_TTL_PATH=) stay visible.
test-bin binary:
    timeout 600s cargo test -p praxis-graphlaw --test {{binary}} -- --nocapture

# Slow quality gates: mutation score, line coverage, dylint (requires cargo-mutants/llvm-cov/dylint)
chatman-quality:
    cargo mutants -p praxis-graphlaw --file 'src/chatman/*'
    cargo llvm-cov nextest -p praxis-graphlaw --fail-under-lines 85
    cargo dylint --all --workspace

# Idempotence check: ggen sync twice must leave generated chatman paths unchanged
chatman-sync-verify:
    command -v ggen >/dev/null || (echo "ggen not found on PATH — run: cargo install --path crates/ggen --locked" && exit 1)
    timeout 120s ggen sync run
    timeout 120s ggen sync run
    git diff --exit-code -- 'crates/praxis-graphlaw/src/chatman' 'docs/chatman-engine'

# --- OTel Weaver live-check ---
# Campaign contracts: registry/otel/*.yaml (weaver 0.22.1 registry), cng --features otel-live
# --bin otel-live (positive prints OTEL_SPANS_EMITTED=1 / exit 0; negative omits process.outcome,
# exits nonzero, prints NEGATIVE_REFUSAL_CODE=WEAVER_SEMANTIC_CONVENTION_REFUSED).
# Ports overridable: OTEL_GRPC_PORT (default 4317), OTEL_ADMIN_PORT (default 4320).
# Handoff boundary after admission: docs/otel-rdf-handoff.md (G10 doc-only, G11 BLOCKED).

# Regenerate via ggen (same command `just standing` uses), then print a combined registry digest
otel-weaver-generate:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v ggen >/dev/null || { echo "ggen not found on PATH — run: cargo install --path crates/ggen --locked"; exit 1; }
    timeout 120s ggen sync run
    files=$(ls registry/otel/*.yaml 2>/dev/null | sort) || true
    if [ -z "${files:-}" ]; then
        echo "otel-weaver-generate: no registry/otel/*.yaml files found (registry agent not landed yet)"; exit 1
    fi
    if command -v b3sum >/dev/null 2>&1; then
        digest=$(cat $files | b3sum | awk '{print $1}')
    else
        digest=$(cat $files | shasum -a 256 | awk '{print $1}')
    fi
    echo "WEAVER_REGISTRY_DIGEST=$digest"

# Static registry check against weaver 0.22.1 semantic-convention schema
otel-weaver-check:
    timeout 120s weaver registry check -r registry/otel --future

# Build the feature-gated otel-live emitter binary
otel-weaver-build:
    timeout 900s cargo build -p cng --features otel-live --bin otel-live

# Start weaver live-check backgrounded; PID in target/weaver-live/weaver.pid; TCP-poll readiness
otel-weaver-live-start outdir="target/weaver-live":
    #!/usr/bin/env bash
    set -euo pipefail
    grpc_port="${OTEL_GRPC_PORT:-4317}"; admin_port="${OTEL_ADMIN_PORT:-4320}"
    for p in "$grpc_port" "$admin_port"; do
        if nc -z 127.0.0.1 "$p" 2>/dev/null; then
            echo "otel-weaver-live-start: port $p already occupied on 127.0.0.1 — stop the holder or override OTEL_GRPC_PORT/OTEL_ADMIN_PORT"; exit 1
        fi
    done
    mkdir -p "{{outdir}}"
    weaver registry live-check -r registry/otel \
        --otlp-grpc-address 127.0.0.1 --otlp-grpc-port "$grpc_port" \
        --admin-port "$admin_port" --inactivity-timeout 60 \
        --format json --output "{{outdir}}/" > "{{outdir}}/weaver.stdout.log" 2>&1 &
    pid=$!
    echo "$pid" > "{{outdir}}/weaver.pid"
    # Readiness = admin port accepting TCP, not a bare sleep (max ~15s @ 0.5s steps)
    for _ in $(seq 1 30); do
        if nc -z 127.0.0.1 "$admin_port" 2>/dev/null; then
            echo "otel-weaver-live-start: weaver ready (pid $pid, grpc $grpc_port, admin $admin_port)"; exit 0
        fi
        if ! kill -0 "$pid" 2>/dev/null; then
            echo "otel-weaver-live-start: weaver exited before becoming ready; log follows:"; cat "{{outdir}}/weaver.stdout.log"; exit 1
        fi
        sleep 0.5
    done
    echo "otel-weaver-live-start: admin port $admin_port never opened within 15s"; kill "$pid" 2>/dev/null || true; exit 1

# Emit one positive ActivityExecuted batch at the running live-check endpoint
otel-weaver-production-run:
    #!/usr/bin/env bash
    set -euo pipefail
    grpc_port="${OTEL_GRPC_PORT:-4317}"
    timeout 300s cargo run -p cng --features otel-live --bin otel-live -- \
        --endpoint "http://127.0.0.1:$grpc_port" --mode positive

# Stop live-check via admin POST /stop, pidfile kill fallback, wait for exit
otel-weaver-live-stop outdir="target/weaver-live":
    #!/usr/bin/env bash
    set -euo pipefail
    admin_port="${OTEL_ADMIN_PORT:-4320}"
    curl -s -X POST "http://127.0.0.1:$admin_port/stop" >/dev/null 2>&1 || true
    pidfile="{{outdir}}/weaver.pid"
    if [ -f "$pidfile" ]; then
        pid=$(cat "$pidfile")
        # Wait up to 15s for graceful exit after /stop before escalating
        for _ in $(seq 1 30); do
            kill -0 "$pid" 2>/dev/null || { echo "otel-weaver-live-stop: weaver (pid $pid) exited"; rm -f "$pidfile"; exit 0; }
            sleep 0.5
        done
        echo "otel-weaver-live-stop: /stop did not terminate pid $pid; sending SIGTERM"
        kill "$pid" 2>/dev/null || true
        for _ in $(seq 1 10); do
            kill -0 "$pid" 2>/dev/null || break
            sleep 0.5
        done
        kill -0 "$pid" 2>/dev/null && { echo "otel-weaver-live-stop: pid $pid survived SIGTERM"; exit 1; }
        rm -f "$pidfile"
    else
        echo "otel-weaver-live-stop: no pidfile at $pidfile (nothing to stop)"
    fi

# Assert the live-check report: >0 received signals, 0 violations. Fails loudly with the
# report head if the weaver 0.22 schema fields are absent — never silently passes.
otel-weaver-live-verify outdir="target/weaver-live":
    #!/usr/bin/env bash
    set -euo pipefail
    report="{{outdir}}/live_check.json"
    [ -f "$report" ] || { echo "otel-weaver-live-verify: $report missing"; ls -la "{{outdir}}" || true; exit 1; }
    # Weaver 0.22 report: one JSON object { samples: [...], statistics: {...} }.
    # statistics.total_entities counts received entities; each policy finding
    # inside samples carries "level": "violation" when non-conformant.
    received=$(jq '.statistics.total_entities // empty' "$report" 2>/dev/null || true)
    if [ -z "${received:-}" ] || [ "$received" = "null" ]; then
        # Fallback: count sample entries
        received=$(jq '.samples | length' "$report" 2>/dev/null || true)
    fi
    if [ -z "${received:-}" ] || [ "$received" = "null" ]; then
        echo "otel-weaver-live-verify: no recognizable count field (statistics.total_entities/samples) in report; head follows:"
        head -c 2000 "$report"; echo; exit 1
    fi
    violations=$(jq '[.. | objects | select(.level? == "violation")] | length' "$report")
    echo "WEAVER_LIVE_CHECK_RECEIVED_SIGNALS=$received"
    echo "WEAVER_LIVE_CHECK_VIOLATIONS=$violations"
    if [ "$received" -gt 0 ] && [ "$violations" -eq 0 ]; then
        echo "WEAVER_LIVE_CHECK_CONFORMS=true"
    else
        echo "WEAVER_LIVE_CHECK_CONFORMS=false"
        echo "otel-weaver-live-verify: FAILED (received=$received violations=$violations); report head:"
        head -c 2000 "$report"; echo; exit 1
    fi

# Negative campaign leg: emit telemetry missing process.outcome, expect >=1 violation
otel-weaver-live-negative:
    #!/usr/bin/env bash
    set -euo pipefail
    outdir="target/weaver-live-negative"
    grpc_port="${OTEL_GRPC_PORT:-4317}"
    just otel-weaver-live-start "$outdir"
    # Negative mode is REQUIRED to exit nonzero (NEGATIVE_REFUSAL_CODE line) — capture, don't abort
    neg_rc=0
    timeout 300s cargo run -p cng --features otel-live --bin otel-live -- \
        --endpoint "http://127.0.0.1:$grpc_port" --mode negative || neg_rc=$?
    echo "otel-weaver-live-negative: otel-live --mode negative exit code = $neg_rc (nonzero expected)"
    just otel-weaver-live-stop "$outdir"
    report="$outdir/live_check.json"
    [ -f "$report" ] || { echo "otel-weaver-live-negative: $report missing"; exit 1; }
    violations=$(jq '[.. | objects | select(.level? == "violation")] | length' "$report")
    outcome_violations=$(jq '[.. | objects | select(.level? == "violation") | select(((.message // "") | test("process\\.outcome")) or ((.context.attribute_name // "") == "process.outcome"))] | length' "$report")
    echo "NEGATIVE_LIVE_CHECK_VIOLATIONS=$violations"
    if [ "$violations" -ge 1 ] && [ "$outcome_violations" -ge 1 ] && [ "$neg_rc" -ne 0 ]; then
        echo "NEGATIVE_LIVE_CHECK_CONFORMS=false"
        echo "NEGATIVE_REFUSAL_CODE=WEAVER_SEMANTIC_CONVENTION_REFUSED"
    else
        echo "otel-weaver-live-negative: FAILED (violations=$violations process.outcome-violations=$outcome_violations emitter_rc=$neg_rc); report head:"
        head -c 2000 "$report"; echo; exit 1
    fi

# Full campaign: generate -> check -> build -> 3x (digest-stable live loop) -> negative -> markers
otel-weaver-live:
    #!/usr/bin/env bash
    set -euo pipefail
    cleanup() {
        for d in target/weaver-live target/weaver-live-negative; do
            if [ -f "$d/weaver.pid" ]; then
                kill "$(cat "$d/weaver.pid")" 2>/dev/null || true
            fi
        done
    }
    trap cleanup EXIT INT ERR
    just otel-weaver-generate
    just otel-weaver-check
    just otel-weaver-build
    digests=()
    for i in 1 2 3; do
        echo "--- otel-weaver-live loop $i/3 ---"
        d=$(just otel-weaver-generate | grep '^WEAVER_REGISTRY_DIGEST=' | cut -d= -f2)
        digests+=("$d")
        just otel-weaver-live-start
        just otel-weaver-production-run
        just otel-weaver-live-stop
        just otel-weaver-live-verify
    done
    if [ "${digests[0]}" != "${digests[1]}" ] || [ "${digests[1]}" != "${digests[2]}" ]; then
        echo "otel-weaver-live: registry digest NOT stable across 3 generations: ${digests[*]}"; exit 1
    fi
    just otel-weaver-live-negative
    echo "=== OTel Weaver campaign markers ==="
    echo "WEAVER_VERSION=$(weaver --version)"
    echo "WEAVER_REGISTRY_DIGEST=${digests[0]}"
    echo "G0_REGISTRY_GENERATED=PASS"
    echo "G1_REGISTRY_CHECK=PASS"
    echo "G2_BINARY_BUILD=PASS"
    echo "G3_LIVE_RUN_1=PASS"
    echo "G4_LIVE_RUN_2=PASS"
    echo "G5_LIVE_RUN_3=PASS"
    echo "G6_DIGEST_STABLE=PASS"
    echo "G7_POSITIVE_CONFORMS=PASS"
    echo "G8_NEGATIVE_REFUSED=PASS"
    echo "G9_RECEIPT_MARKERS=PASS"
    echo "G10_OTEL_RDF_BOUNDARY=ALIVE_AS_DOC (docs/otel-rdf-handoff.md)"
    echo "G11_OTEL_TO_RDF_MAPPER=BLOCKED (not implemented; see docs/otel-rdf-handoff.md)"
