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

# Check for a stray concurrent cargo/rustc build or rebar3 (Erlang side, apps/arazzo_*) holding
# a build lock before starting a new one -- concurrent invocations serialize and silently double
# wall-clock time. Widened from a cargo-only grep after this session repeatedly needed the same
# check against rebar3 compiles too (apps/arazzo_atomvm, apps/arazzo_runner).
check-lock:
    @ps aux | grep -E "cargo|rustc|rebar3" | grep -v grep || echo "no cargo/rustc/rebar3 build currently running"

# Report every isolated CARGO_TARGET_DIR under target/agent-* with its size, and warn (not
# delete) if any cargo/rustc/rebar3 process is currently running -- a stray agent-* dir might
# still be in active use by it. Read-only; pair with clean-stale-isolated to actually remove.
list-isolated:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! ls -d target/agent-* >/dev/null 2>&1; then
        echo "no target/agent-* dirs present"; exit 0
    fi
    du -sh target/agent-* 2>/dev/null | sort -rh
    if ps aux | grep -E "cargo|rustc|rebar3" | grep -v grep >/dev/null; then
        echo "WARNING: a cargo/rustc/rebar3 process is currently running -- do not blindly remove all of the above, one may be held by it"
    fi

# Remove every isolated target/agent-* dir not currently held by a running cargo/rustc/rebar3
# process (regenerable, zero risk once confirmed unheld). Formalizes a pattern this session's
# own agents hand-ran 15+ times (`ps aux | grep -E "cargo|rustc|rebar3"` then `rm -rf
# target/agent-*`) because target/agent-* dirs are NOT self-cleaning and repeatedly filled disk
# toward 100% this session. Refuses (does not delete anything) if a matching process is found --
# re-run once it finishes. See also the older, cng-scoped `cng-clean-all-isolated` (identical
# unconditional `rm -rf target/agent-*`, no process check) and single-dir `cng-clean-isolated
# <name>`; this recipe is the crate-agnostic, safety-checked version those two predate.
clean-stale-isolated:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! ls -d target/agent-* >/dev/null 2>&1; then
        echo "clean-stale-isolated: no target/agent-* dirs present, nothing to do"; exit 0
    fi
    if ps aux | grep -E "cargo|rustc|rebar3" | grep -v grep >/dev/null; then
        echo "clean-stale-isolated: refusing -- a cargo/rustc/rebar3 process is currently running"
        echo "check 'just list-isolated' and re-run once it finishes"
        exit 1
    fi
    du -sh target/agent-* 2>/dev/null
    rm -rf target/agent-*
    echo "clean-stale-isolated: removed all target/agent-* dirs listed above"

# Verify each hardcoded external path dependency (wasm4pm-compat, bcinr-pddl/bcinr-powl/
# bcinr-powl-receipt, lsp-max, affidavit -- root Cargo.toml [dependencies] and
# [patch.crates-io]) actually exists on disk, report its git branch/HEAD (flagging a
# detached HEAD), and check whether the sibling's own Cargo.toml [package] version
# satisfies this workspace's declared version requirement (Cargo caret semantics: same
# major, sibling >= required). A detached-HEAD sibling checkout or a version drift here
# is exactly the failure class that cost 5+ agents a large stretch of an earlier session
# (a detached HEAD plus an unwired subcrate extraction in wasm4pm-compat) -- this turns
# that multi-agent debugging saga into a 2-second check. Nonzero exit if any sibling is
# missing, detached, or version-mismatched.
check-sibling-deps:
    #!/usr/bin/env bash
    set -uo pipefail
    status=0

    # Cargo's default caret compatibility: same major, then (minor, patch) of the
    # sibling's actual version must be >= the workspace's required version. All deps
    # here are major 26, but the 0.x.y stricter case (pin minor too) is handled for
    # completeness.
    semver_ok() {
        local req="$1" act="$2"
        local req_maj req_min req_pat act_maj act_min act_pat
        IFS='.' read -r req_maj req_min req_pat <<< "$req"
        IFS='.' read -r act_maj act_min act_pat <<< "$act"
        req_min=${req_min:-0}; req_pat=${req_pat:-0}
        act_min=${act_min:-0}; act_pat=${act_pat:-0}
        [ "$req_maj" = "$act_maj" ] || return 1
        if [ "$req_maj" = "0" ]; then
            [ "$req_min" = "$act_min" ] || return 1
            [ "$act_pat" -ge "$req_pat" ]
            return $?
        fi
        if [ "$act_min" -gt "$req_min" ]; then return 0; fi
        if [ "$act_min" -lt "$req_min" ]; then return 1; fi
        [ "$act_pat" -ge "$req_pat" ]
    }

    deps=(
        "wasm4pm-compat:/Users/sac/wasm4pm-compat"
        "bcinr-pddl:../bcinr/crates/bcinr-pddl"
        "bcinr-powl:../bcinr/crates/bcinr-powl"
        "bcinr-powl-receipt:../bcinr/crates/bcinr-powl-receipt"
        "lsp-max:/Users/sac/lsp-max"
        "affidavit:/Users/sac/affidavit"
    )

    for entry in "${deps[@]}"; do
        name="${entry%%:*}"
        path="${entry#*:}"
        echo "=== $name ($path) ==="

        if [ ! -d "$path" ]; then
            echo "  MISSING: directory does not exist on disk"
            status=1
            continue
        fi

        if git -C "$path" rev-parse --git-dir >/dev/null 2>&1; then
            branch=$(git -C "$path" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
            sha=$(git -C "$path" rev-parse --short HEAD 2>/dev/null || echo "unknown")
            if [ "$branch" = "HEAD" ]; then
                echo "  DETACHED HEAD at $sha -- sibling checkout is not on a named branch"
                status=1
            else
                echo "  branch: $branch @ $sha"
            fi
        else
            echo "  not a git repository"
        fi

        sibling_toml="$path/Cargo.toml"
        if [ ! -f "$sibling_toml" ]; then
            echo "  MISSING: no Cargo.toml at $sibling_toml"
            status=1
            continue
        fi
        sibling_ver=$(grep -m1 -E '^version[[:space:]]*=[[:space:]]*"' "$sibling_toml" \
            | sed -E 's/^version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')

        required_ver=$(grep -m1 -E "^${name}[[:space:]]*=.*version[[:space:]]*=[[:space:]]*\"" Cargo.toml \
            | sed -E 's/.*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/')

        if [ -z "$sibling_ver" ]; then
            echo "  sibling Cargo.toml has no literal [package] version (workspace-inherited and unresolved)"
            status=1
        elif [ -z "$required_ver" ]; then
            echo "  version: $sibling_ver (workspace has no version constraint on this dep; patch-only reference)"
        elif semver_ok "$required_ver" "$sibling_ver"; then
            echo "  version: $sibling_ver satisfies workspace requirement ^$required_ver"
        else
            echo "  VERSION MISMATCH: sibling=$sibling_ver does not satisfy workspace requirement ^$required_ver"
            status=1
        fi
    done

    exit $status

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

# Full cargo clean (wipes target/ entirely) — check `ps aux | grep cargo` first;
# this will corrupt any concurrently running build, not just slow it down.
clean:
    cargo clean

# NOTE: must invoke `cargo-cicd` (direct binary), not `cargo cicd` — the installed
# binary's clap parser rejects cargo's prepended arg.
# Refresh the praxis-standing.v1 index, standing-pack ontology, and docs/standing/REALITY_INDEX.md
#
# Swarm audit wnl2yhbgm findings #16/#17: the sibling-repo ontology publish below used to be a
# bare `cp` into `../cargo-cicd/plugins/cargo-cicd-kit/standing-pack/ontology.ttl` -- a hardcoded
# sibling checkout that exists on this developer's machine but not in a fresh clone or CI. Since
# `just` recipe lines run as separate shell invocations and abort the recipe on the first nonzero
# exit, a missing sibling made `cp` fail and the recipe stop right there.
#
# CORRECTION (dogfood cycle 13, workflow wr71ue3z9, caught an overstated claim in this comment's
# first version): `target/praxis-standing/standing.json` is produced by `cargo-cicd standing
# refresh` on the line ABOVE the cp step and was never actually at risk from a cp failure, pre- or
# post-fix. Only `ggen sync run` + `cargo-cicd claude_context show` (which produces
# `docs/standing/REALITY_INDEX.md`) were genuinely gated by the old hard-abort. The sibling publish
# is now best-effort -- skips with a disclosed message if the sibling directory isn't present, so
# that specific failure mode (an opaque hard-abort on a missing sibling) no longer happens. This
# used to mean `just standing` did not complete end to end on every machine: `ggen sync run` had
# its OWN separate failure mode this fix did not touch -- `standing.ttl`'s content changes on every
# refresh while `ggen.lock` pins a specific prior hash, so `ggen sync run` genuinely refused with a
# content-hash mismatch (`FM-PACK-008`) on this developer's own machine (workflow wr71ue3z9 ran the
# ORIGINAL unconditional cp standalone and got the identical error).
#
# FIXED: `PackRef::Path` gained a `lock: bool` opt-out field (default `true`, unchanged behavior
# for every other pack). The root `ggen.toml`'s `standing-pack` entry should set `lock = false` --
# `standing-pack/ontology.ttl` is a regenerated output projection (rewritten by every `just
# standing` run above), not a stable source, so content-hash pinning was fundamentally the wrong
# contract for it. An unlocked pack is never checked against `ggen.lock` and never written to it
# (`crate::pack::lock_entries` skips it outright). See `crates/ggen/src/pack.rs` and
# `crates/ggen/src/config.rs` for the implementation; `ggen.toml`'s `standing-pack` line itself is
# updated in a separate step to avoid a concurrent-edit race with other in-flight agents.
standing:
    command -v ggen >/dev/null || (echo "ggen not found on PATH — run: cargo install --path crates/ggen --locked" && exit 1)
    timeout 180s cargo-cicd standing refresh
    if [ -d "../cargo-cicd/plugins/cargo-cicd-kit/standing-pack" ]; then cp target/praxis-standing/standing.ttl ../cargo-cicd/plugins/cargo-cicd-kit/standing-pack/ontology.ttl && echo "just standing: published standing.ttl to sibling checkout"; else echo "just standing: sibling checkout not found at ../cargo-cicd/plugins/cargo-cicd-kit/standing-pack -- skipping the dev-convenience ontology publish (not required for core standing refresh; expected in a fresh clone or CI)"; fi
    timeout 120s ggen sync run
    timeout 60s cargo-cicd claude_context show
    @echo "just standing: refreshed target/praxis-standing/standing.json, regenerated docs/standing/REALITY_INDEX.md and target/praxis-standing/CLAUDE_CODE_CONTEXT.md"

# Build the workspace
build:
    timeout 120s cargo build

# NOTE: consumers pinning absolute paths (e.g. mfact -> target/debug/ggen) are
# intentionally not updated by this recipe.
# Install the release ggen binary to ~/.cargo/bin so the global `ggen` tracks this checkout.
# `cargo install` builds in its own isolated target dir by default (not the workspace
# target/), so a cold install recompiles the full dependency tree -- 180s was observed to
# time out (exit 124) on a cold build; 900s covers that plus headroom, matching the other
# from-scratch build timeouts in this file (e.g. ci-clean-verify's 1800s cng rebuild).
install-ggen:
    timeout 900s cargo install --path crates/ggen --force
    @ggen --version

# Type-check the whole workspace with every feature enabled (what `doctor check` itself shells out to)
check:
    timeout 180s cargo check --workspace --all-features

# Run the full test suite across the workspace with every feature enabled (matches CI's `test` job)
# nextest runs test binaries in parallel (vs. cargo test's serial-by-binary execution); falls
# back to cargo test if nextest isn't on PATH (see chatman-verify for the same pattern)
test:
    if command -v cargo-nextest >/dev/null 2>&1; then \
        cargo nextest run --workspace --all-features -j 4; \
    else \
        cargo test --workspace --all-features -- --test-threads=1; \
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

# Format (write) just the named package -- scoped so a change to one crate doesn't touch
# unrelated in-flight edits elsewhere in the workspace (e.g. concurrent agent sessions).
fmt-pkg pkg:
    cargo fmt -p {{pkg}}

# Format-check just the named package, scoped (same rationale as fmt-pkg)
fmt-check-pkg pkg:
    cargo fmt -p {{pkg}} -- --check

# Holistic health check: build, config witness, frontier, tools, receipts, features. `just doctor format=json` for machine output
doctor format="text":
    cargo run --quiet --bin my-conforming-project --all-features -- doctor check --format {{format}}

# Build the capability-frontier DfCM matrix and print its summary + full report
frontier:
    cargo run --quiet --bin my-conforming-project --all-features -- frontier matrix

# The full local Definition-of-Done gate in CI order: check, test, clippy, then doctor (stops at first failure)
verify-all: check test clippy doctor lean-receipt-gate
    @echo "verify-all: check + test + clippy + doctor + lean-receipt-gate all passed"

# PROJ-795 (v26.7.11): FIRST-SLICE Verifier Report (PRD.md sec.20 "Verifier Report", 13
# required fields). NOT the full 13-field instrument -- answers only the fields today's
# real, already-built artifacts can back with a live command or a real parse of
# docs/jira/v26.7.11/tickets/index.md; every other field is printed as an explicit,
# visible NOT_YET_AVAILABLE row naming the blocking ticket, never a guessed value.
# Re-runs every underlying check live (no cached results). See scripts/verifier_report.py
# for the field-by-field design notes.
verifier-report:
    python3 scripts/verifier_report.py

# Run workspace benchmarks. `just bench filter="bench_name"` to scope to one benchmark target
bench filter="":
    cargo bench {{ if filter != "" { "--bench " + filter } else { "" } }}

# Line/branch coverage report via tarpaulin (installs it if missing). `just coverage out="Html"` for other tarpaulin --out formats
coverage out="Html":
    command -v cargo-tarpaulin >/dev/null 2>&1 || cargo install cargo-tarpaulin --locked
    cargo tarpaulin --out {{out}} --output-dir coverage --exclude-files "tests/*"

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
        cargo nextest run -p praxis-graphlaw -E 'test(chatman)'; \
    else \
        cargo test -p praxis-graphlaw chatman; \
    fi
    timeout 300s cargo test -p praxis-graphlaw --test chatman_static_gates
    python3 docs/chatman-engine/diagrams/atlas/verify_atlas.py

# --- cng CLI (crates/cng) ---

# Golden-path onboarding check for cng first-time contributors: doctor check, a real
# small-scale `benchmark workday` run, a real `plan decompose` run against the on-disk
# fixture CHEATSHEET.md documents, and the `cng_cli_smoke` integration test (plan
# generate -> workflow export -> inspect over plans/joseph/). One command, green output,
# you're set up -- see crates/cng/GETTING_STARTED.md. Fails fast (set -euo pipefail,
# each cargo call has its own timeout) and isolates its cargo calls in
# target/agent-smoke so it never collides with a concurrent agent's lock on the default
# target/ dir (see the CARGO_TARGET_DIR note at the top of this file); the last step
# reuses cng-test-isolated against the SAME isolated dir so the whole recipe shares one
# incremental build. Not a throughput benchmark -- deliberately the smallest
# already-proven scenario per step, not an invented one: 4-tick workday,
# CHEATSHEET.md's decompose fixture, the existing plan/export/inspect CLI smoke test.
# Clean up the isolated dir afterward with `just cng-clean-isolated smoke`.
cng-smoke:
    #!/usr/bin/env bash
    set -euo pipefail
    export CARGO_TARGET_DIR="target/agent-smoke"
    scratch="$CARGO_TARGET_DIR/cng-smoke"

    echo "=== [1/4] doctor check ==="
    doctor_out=$(timeout 120s cargo run --quiet -p cng --features bench --bin cng -- workflow doctor)
    echo "$doctor_out"
    if echo "$doctor_out" | grep -q "FAILED"; then
        echo "FAIL: doctor check reported a FAILED runtime check"
        exit 1
    fi
    echo "PASS: doctor check"

    echo "=== [2/4] benchmark workday (smoke scale: 4 ticks) ==="
    rm -rf "$scratch/workday"
    timeout 120s cargo run --quiet -p cng --features bench --bin cng -- \
        benchmark workday --out "$scratch/workday" --seed 1 --ticks 4
    echo "PASS: benchmark workday"

    echo "=== [3/4] plan decompose (fixture: tests/fixtures/decomp-negative/actor-lacks-capability) ==="
    rm -rf "$scratch/decompose"
    timeout 120s cargo run --quiet -p cng --features bench --bin cng -- \
        plan decompose \
        --domain crates/cng/tests/fixtures/decomp-negative/actor-lacks-capability.domain.pddl \
        --problem crates/cng/tests/fixtures/decomp-negative/actor-lacks-capability.problem.pddl \
        --out "$scratch/decompose"
    echo "PASS: plan decompose"

    echo "=== [4/4] cng_cli_smoke integration test ==="
    just cng-test-isolated smoke cng_cli_smoke
    echo "PASS: cng_cli_smoke"

    echo "cng-smoke: PASS -- doctor + workday(4 ticks) + decompose(fixture) + cng_cli_smoke all green; you're set up"

# Build the cng CLI binary
cng-build:
    cargo build -p cng

# Run the cng CLI with arguments (e.g. `just cng-run plan generate --dir plans/`)
# --bin cng is required now that the crate ships 3 binaries (cng, otel-live,
# otel-rdf-demo); `cargo run` refuses to guess once a crate has more than one.
cng-run *args:
    timeout 300s cargo run -q -p cng --bin cng -- {{args}}

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
    cargo test -p cng

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
    cargo test -p cng --features bench --test {{binary}} {{args}}

# Run ONLY the cng crate's in-crate unit tests (dispatch_test.rs, engine_test.rs,
# decomp_test.rs, etc. via #[cfg(test)] modules) — skips the heavier standalone
# integration-test binaries (cng_long_horizon_scenario and friends) entirely.
cng-test-lib *args:
    cargo test -p cng --features bench --lib {{args}}

# Rail G Track 2b: run the real-workday multifractal measurement test
# (`track2b_real_workday_tape_ops_measurement`, crates/cng/src/bench/multifractal_test.rs)
# and print its report. Previously only reachable via a bare `cargo test`; its output
# directory was already wiped once by `cargo clean` earlier this session and had to be
# regenerated by hand. Writes (and here, prints)
# target/chatman/cng-tests/multifractal/track2b_real/track2b-measurement.txt -- that path
# is fixed relative to CARGO_MANIFEST_DIR by the test itself (scratch_dir()), so it lands
# there regardless of any CARGO_TARGET_DIR override used for the build.
cng-track2b-report:
    cargo test -p cng --features bench --lib track2b_real_workday_tape_ops_measurement -- --nocapture
    @echo "--- target/chatman/cng-tests/multifractal/track2b_real/track2b-measurement.txt ---"
    @cat target/chatman/cng-tests/multifractal/track2b_real/track2b-measurement.txt

# PROJ-728/729 multi-engine harness: coordinator + REAL engine OS processes
# over the filesystem transport — isolation falsifiers, G13 crash-resume,
# distributed determinism, cross-engine recursion. Exact --test scope;
# single-threaded because each test spawns its own engine processes.
cng-multi-engine:
    timeout 1800s cargo test -p cng --features bench --test cng_multi_engine -- --test-threads=1

# Package-surface smoke: install the cng binary from the crate and invoke it
cng-install-smoke:
    cargo install --path crates/cng --debug --root target/install-smoke --force
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
    CARGO_TARGET_DIR=target/agent-{{name}} cargo test -p cng --features bench --test {{binary}} {{args}}

# Run the FULL cng test suite (lib + every integration-test binary) with the bench
# feature, in an isolated target dir (concurrent-agent-safe; see note above). Same
# command as cng-test-bench, just isolated -- for whole-crate regression sweeps where
# cng-test-isolated's single-binary scoping would miss cross-binary interactions.
cng-test-bench-isolated name *args:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 900s cargo test -p cng --features bench {{args}}

# Run cng's in-crate unit tests (cng-test-lib) in an isolated target dir
# (concurrent-agent-safe; see note above), e.g.
# `just cng-test-lib-isolated my-feature otel_rdf`
cng-test-lib-isolated name *args:
    CARGO_TARGET_DIR=target/agent-{{name}} cargo test -p cng --features bench --lib {{args}}

# Type-check the cng crate + its tests in an isolated target dir (concurrent-agent-safe; fast
# compile-only sanity check mid-edit, before cng-test-isolated)
cng-check-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 300s cargo check -p cng --features bench --tests

# Lint the cng crate (lib + tests, bench feature) in an isolated target dir
# (concurrent-agent-safe; see note above), same flags as CI's clippy job.
cng-clippy-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 300s cargo clippy -p cng --features bench --tests -- -D warnings

# Lint ONLY the cng crate's own code (--no-deps), skipping clippy's lint pass on path
# dependencies like praxis-graphlaw. Use this when cng-clippy-isolated fails inside a
# dependency crate you didn't touch -- that failure blocks clippy from ever reaching cng's
# own code, so this is the only way to get a clean signal on a cng-only change until the
# dependency's own debt is addressed separately.
cng-clippy-own-code-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 300s cargo clippy -p cng --features bench --tests --no-deps -- -D warnings

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
    cargo publish -p {{crate}} --dry-run --allow-dirty

# Run one exact praxis-graphlaw integration-test binary (e.g. `just test-bin chatman_pddl_to_powl_projection`).
# --nocapture so artifact-path markers (GENERATED_POWL_TTL_PATH=, IMPORTED_PDDL_TTL_PATH=) stay visible.
test-bin binary:
    cargo test -p praxis-graphlaw --test {{binary}} -- --nocapture

# Run praxis-graphlaw's in-crate `#[cfg(test)]` lib unit tests, filtered by a nextest test-name
# expression (e.g. `just praxis-graphlaw-test-lib 'test(chatman::router)'`). Falls back to
# `cargo test`'s substring filter if nextest isn't on PATH. Scoped to `--lib` so a filter that
# only matches lib-module tests (like `chatman::router::tests::*`) doesn't pull in the
# integration-test binaries `test-bin` already covers individually.
praxis-graphlaw-test-lib filter:
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v cargo-nextest >/dev/null 2>&1; then
        timeout 300s cargo nextest run -p praxis-graphlaw --lib -E '{{filter}}'
    else
        timeout 300s cargo test -p praxis-graphlaw --lib '{{filter}}'
    fi

# Type-check the praxis-graphlaw crate + its tests, scoped (the workspace-wide `just check`
# pulls in every crate; this isolates praxis-graphlaw from unrelated in-flight breakage
# elsewhere in the workspace)
praxis-graphlaw-check:
    timeout 180s cargo check -p praxis-graphlaw --all-targets --all-features

# Type-check the praxis-graphlaw crate's lib + tests only, excluding benches. Exists because
# `benches/owlrl.rs` has a pre-existing, unrelated break (`TripleStore::from` returns
# `TripleStore`, not `Result`, so its `.expect(...)` calls don't compile -- confirmed via `git
# log` to predate this session's work) that would otherwise block `praxis-graphlaw-check` for
# anyone touching only src/tests.
praxis-graphlaw-check-libtests:
    timeout 180s cargo check -p praxis-graphlaw --lib --tests --all-features

# Run praxis-graphlaw's `--lib` unit tests in an isolated target dir (concurrent-agent-safe;
# see the "Isolated-target cargo recipes" note above cng-check-isolated), scoped to --lib
# (excludes benches, same pre-existing owlrl.rs break as praxis-graphlaw-check-libtests). Pass
# a substring/nextest filter as extra args, e.g.
# `just praxis-graphlaw-test-lib-isolated my-feature parser::test`
praxis-graphlaw-test-lib-isolated name *args:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 180s cargo test -p praxis-graphlaw --lib {{args}}

# Run one praxis-graphlaw `tests/*.rs` integration-test binary in an isolated target dir
# (concurrent-agent-safe; see the "Isolated-target cargo recipes" note above
# cng-check-isolated). `binary` is the file stem under tests/ (e.g. `soc2_hook_actuation`),
# not the crate name. Pass a substring filter as extra args, e.g.
# `just praxis-graphlaw-test-integration-isolated my-feature soc2_hook_actuation test_cuec_gate`
praxis-graphlaw-test-integration-isolated name binary *args:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 180s cargo test -p praxis-graphlaw --test {{binary}} {{args}}

# Lint praxis-graphlaw's lib + tests in an isolated target dir (concurrent-agent-safe;
# same --lib --tests scope as praxis-graphlaw-clippy-libtests, same pre-existing
# owlrl.rs bench break excluded).
praxis-graphlaw-clippy-libtests-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 180s cargo clippy -p praxis-graphlaw --lib --tests --all-features -- -D warnings

# Lint the praxis-graphlaw crate with the same flags CI's clippy job uses, scoped (same
# isolation rationale as praxis-graphlaw-check)
praxis-graphlaw-clippy:
    timeout 180s cargo clippy -p praxis-graphlaw --all-targets --all-features -- -D warnings

# Lint the praxis-graphlaw crate's lib + tests only, excluding benches -- same pre-existing
# `benches/owlrl.rs` break as `praxis-graphlaw-check-libtests` above.
praxis-graphlaw-clippy-libtests:
    timeout 180s cargo clippy -p praxis-graphlaw --lib --tests --all-features -- -D warnings

# Type-check the wasm4pm-arazzo crate + its tests
wasm4pm-arazzo-check:
    timeout 180s cargo check -p wasm4pm-arazzo --tests

# Run the wasm4pm-arazzo crate's unit + integration tests
wasm4pm-arazzo-test *args:
    cargo test -p wasm4pm-arazzo {{args}}

# Type-check the wasm4pm-arazzo crate + its tests in an isolated target dir
# (concurrent-agent-safe; see the "Isolated-target cargo recipes" note above
# cng-check-isolated), e.g. `just wasm4pm-arazzo-check-isolated my-feature`
wasm4pm-arazzo-check-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 180s cargo check -p wasm4pm-arazzo --tests

# Run the wasm4pm-arazzo crate's unit + integration tests in an isolated
# target dir (concurrent-agent-safe), e.g.
# `just wasm4pm-arazzo-test-isolated my-feature`
wasm4pm-arazzo-test-isolated name *args:
    CARGO_TARGET_DIR=target/agent-{{name}} cargo test -p wasm4pm-arazzo {{args}}

# Type-check + test the praxis-core crate (isolate with CARGO_TARGET_DIR=target/agent-<name>
# when running alongside other concurrent cargo work -- see the lock-contention NOTE above)
praxis-core-test *args:
    timeout 300s cargo test -p praxis-core {{args}}

# Type-check the praxis-core crate + its tests, scoped (same isolation rationale as
# praxis-graphlaw-check: praxis-core now depends on praxis-graphlaw for PROJ-752's
# render_arazzo_document, so a workspace-wide `just check` would conflate the two)
praxis-core-check:
    timeout 180s cargo check -p praxis-core --all-targets

# Lint the praxis-core crate with the same flags CI's clippy job uses
praxis-core-clippy:
    timeout 180s cargo clippy -p praxis-core --all-targets -- -D warnings

# PROJ-796 reachability closure (docs/jira/v26.7.11/PATH_TO_100.md sec.2.3 W1):
# real, non-test entry point for
# ChatmanEngine::admit_transition_with_external_cut -- drives a real Rail A/B
# admission (SPARQL projection -> Tera render -> Arazzo parse/resolve/lower/
# normalize/compile -> WASM) through ChatmanRailAbCompiler and prints the
# sealed EngineProcessReceipt. Pass --snapshot <path.ttl> to use a real
# snapshot file instead of the embedded PROJ-796 fixture. `*args` already
# forwards everything after the recipe name to the binary (this recipe body
# inserts the `--` separator itself) -- e.g. `just admit-external-cut --help`,
# not `just admit-external-cut -- --help`.
admit-external-cut *args:
    timeout 120s cargo run -p praxis-core --bin admit-external-cut -- {{args}}

# Lint the wasm4pm-arazzo crate with the same flags CI's clippy job uses
wasm4pm-arazzo-clippy:
    timeout 180s cargo clippy -p wasm4pm-arazzo --all-targets -- -D warnings

# Lint the wasm4pm-arazzo crate in an isolated target dir (concurrent-agent-safe;
# same flags as wasm4pm-arazzo-clippy), e.g. `just wasm4pm-arazzo-clippy-isolated my-feature`
wasm4pm-arazzo-clippy-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 180s cargo clippy -p wasm4pm-arazzo --all-targets -- -D warnings

# Type-check the powl2-decompose crate + its tests
powl2-decompose-check:
    timeout 180s cargo check -p powl2-decompose --tests

# Run the powl2-decompose crate's unit + integration tests
powl2-decompose-test *args:
    cargo test -p powl2-decompose {{args}}

# Lint the powl2-decompose crate with the same flags CI's clippy job uses
powl2-decompose-clippy:
    timeout 180s cargo clippy -p powl2-decompose --all-targets -- -D warnings

# Type-check the multifractal-workflow crate + its tests (v26.7.12 architecture-atlas
# scaffolding crate: 30 family modules, Wire-phase-0 skeletons only as of the crate's
# creation -- this recipe just confirms the skeleton compiles, it proves nothing about
# any family's real logic since none exists yet).
multifractal-workflow-check:
    timeout 180s cargo check -p multifractal-workflow --tests

# Lint the multifractal-workflow crate with the same flags CI's clippy job uses
multifractal-workflow-clippy:
    timeout 180s cargo clippy -p multifractal-workflow --all-targets -- -D warnings

# Lint multifractal-workflow in an isolated target dir (concurrent-agent-safe; see the
# "Isolated-target cargo recipes" note above cng-check-isolated), e.g.
# `just multifractal-workflow-clippy-isolated my-feature`
multifractal-workflow-clippy-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 180s cargo clippy -p multifractal-workflow --all-targets -- -D warnings

# Lint ONLY multifractal-workflow's own code (--no-deps), skipping clippy's lint pass on path
# dependencies like praxis-graphlaw. Use this when multifractal-workflow-clippy-isolated fails
# inside a dependency crate you didn't touch -- same rationale as cng-clippy-own-code-isolated.
multifractal-workflow-clippy-own-code-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 180s cargo clippy -p multifractal-workflow --all-targets --no-deps -- -D warnings

# Type-check multifractal-workflow in an isolated target dir (concurrent-agent-safe;
# see the "Isolated-target cargo recipes" note above cng-check-isolated), e.g.
# `just multifractal-workflow-check-isolated my-feature`
multifractal-workflow-check-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 180s cargo check -p multifractal-workflow --tests

# Type-check only the multifractal-workflow lib target (no --tests), isolated target
# dir. Narrower than multifractal-workflow-check-isolated: useful while this crate has
# 30 family modules being wired concurrently by different agents and one family's
# #[cfg(test)] code may be mid-edit/broken without that meaning every other family's
# non-test code is broken too. Does NOT compile or run any module's tests -- it is not
# a substitute for multifractal-workflow-test-isolated, only a narrower compile signal.
multifractal-workflow-check-lib-isolated name:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 180s cargo check -p multifractal-workflow --lib

# Run the multifractal-workflow crate's unit tests (per-family modules that have
# real wired logic + tests as of when this runs; modules still at Wire-phase-0
# skeleton contribute zero tests, not failures).
multifractal-workflow-test *args:
    timeout 300s cargo test -p multifractal-workflow {{args}}

# Longer-timeout variant of multifractal-workflow-test for the full suite (the crate's
# first shared-target link alone can exceed the 300s recipe above once enough family
# modules have real tests wired).
multifractal-workflow-test-long *args:
    timeout 900s cargo test -p multifractal-workflow {{args}}

# Run multifractal-workflow's tests in an isolated target dir (concurrent-agent-safe;
# see the "Isolated-target cargo recipes" note above cng-check-isolated), e.g.
# `just multifractal-workflow-test-isolated my-feature`
multifractal-workflow-test-isolated name *args:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 300s cargo test -p multifractal-workflow {{args}}

# --- Mutation testing: PILOT, one module only ---
# PILOT SCOPE, explicit: this recipe runs cargo-mutants against exactly ONE
# small, well-tested module --
# crates/multifractal-workflow/src/f16_otp_runner/bridge.rs -- as a
# feasibility pilot for mutation testing in this repo. It is NOT a
# crate-wide or workspace-wide mutation-testing rollout; extending scope to
# more files/crates (and wiring a CI gate on mutation score) is a separate,
# deliberate follow-up decision this recipe does not make for you.
#
# Erlang-side mutation testing (e.g. a PropEr-based equivalent for
# apps/arazzo_runner, apps/air_core, apps/arazzo_atomvm) is a SEPARATE,
# UNATTEMPTED follow-up -- not built here, not implied by this recipe.
#
# Uses `--in-place` (mutates the source file directly in the real working
# tree, then restores it after every mutant -- cargo-mutants' own behavior;
# verified this session by `git diff --stat` on the target file before/after
# a full run showing no residual mutation) rather than the tool's default
# copy-the-whole-workspace-per-mutant mode: this is a large monorepo and a
# full per-mutant workspace copy would be needlessly slow/disk-heavy for a
# single-file pilot.
#
# Builds in an isolated CARGO_TARGET_DIR (target/agent-mutants-pilot, per
# this file's CARGO_TARGET_DIR-isolation convention -- see the note at the
# top of this file) so a run of this recipe never lock-contends with any
# other concurrent `just` build/test/check in this repo. On first use it
# seeds that isolated dir from the existing shared target/debug via an APFS
# copy-on-write clone (`cp -Rc`, falling back to a plain recursive copy on
# non-APFS filesystems) so the baseline build doesn't recompile this crate's
# full dependency graph from scratch; the seed step is skipped on later runs
# once target/agent-mutants-pilot already exists.
#
# Real result, this pilot's first run (crates/multifractal-workflow @
# 26.7.12, cargo-mutants 27.0.0): 12 mutants generated, 4 unviable (failed
# to even compile -- `Ok(Default::default())` against a return type with no
# `Default` impl -- excluded from scoring by cargo-mutants itself), 1
# caught, 7 missed. Mutation score over viable mutants: 1/8 (12.5%). The 7
# missed mutants are a real, disclosed test-coverage gap, not a tooling
# artifact: (a) the `+`/`>=` arithmetic mutants in `wait_with_timeout`
# survive because the only unit test of that function asserts the
# *eventual* TimedOut outcome for a hung child, never that the wait took
# approximately the requested duration (not near-instant) nor exercises the
# "child exits before the timeout" success path at all; (b) the `!`-deletion
# and match-arm-deletion mutants in `call_dispatch_statem_bridge`/
# `parse_dispatch_statem_stdout` survive because the module's `proptest`
# suite only asserts "never panics" on arbitrary/malformed input, never that
# a well-formed response parses to the *correct* value -- that positive-
# value check exists only in the two `#[ignore]`d integration tests, which
# require a real `escript`+compiled `apps/arazzo_runner` and are not part of
# a default `cargo test` run. This recipe does not attempt to fix that gap;
# it reports it.
mutants-pilot:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-mutants >/dev/null || { echo "cargo-mutants not found on PATH -- run: cargo install cargo-mutants --locked"; exit 1; }
    target_dir="target/agent-mutants-pilot"
    if [ ! -d "$target_dir" ]; then
        mkdir -p "$target_dir"
        if [ -d target/debug ]; then
            cp -Rc target/debug "$target_dir/debug" 2>/dev/null || cp -R target/debug "$target_dir/debug"
        fi
    fi
    CARGO_TARGET_DIR="$target_dir" timeout 1800s cargo mutants \
        -p multifractal-workflow \
        --file 'crates/multifractal-workflow/src/f16_otp_runner/bridge.rs' \
        --in-place \
        --no-times \
        -o target/agent-mutants-pilot-report

# Type-check the praxis-lean crate (standalone-cli feature, the plain-clap
# `praxis-l4` entry point) + its tests
praxis-lean-check:
    timeout 180s cargo check -p praxis-lean --no-default-features --features standalone-cli --all-targets

# Run the praxis-lean crate's unit + integration tests
praxis-lean-test *args:
    timeout 300s cargo test -p praxis-lean {{args}}

# Lint the praxis-lean crate with the same flags CI's clippy job uses
praxis-lean-clippy:
    timeout 180s cargo clippy -p praxis-lean --no-default-features --features standalone-cli --all-targets -- -D warnings

# Run the praxis-l4 CLI binary (standalone-cli feature) with arbitrary args, e.g.
# `just praxis-lean-run -- verify --root tools/paper-factory/lean-lake`
praxis-lean-run *args:
    cargo run -q -p praxis-lean --bin praxis-l4 --no-default-features --features standalone-cli -- {{args}}

# Permanent gate against "claimed verified, never actually compiled": computes the real
# transitive import closure of tools/paper-factory/lean-lake/Praxis.lean (static import-line
# parsing, not .lake/build/ artifact inspection -- see crates/praxis-lean/src/closure.rs doc
# comment for why) and fails non-zero if mathlib_migration_receipts.jsonl claims
# "status": "verified" for any label whose file is outside that closure. Exits 0 with a JSON
# summary when clean; prints the exact offending labels and exits non-zero otherwise.
lean-receipt-gate:
    cargo run -q -p praxis-lean --bin praxis-l4 --no-default-features --features standalone-cli -- \
        receipt-closure-gate \
        --root tools/paper-factory/lean-lake \
        --entry Praxis.lean \
        --receipts tools/paper-factory/lean-lake/mathlib_migration_receipts.jsonl

# Type-check the air_core Erlang NIF (apps/air_core/native/air_core_nif)
air-core-nif-check:
    timeout 180s cargo check -p air_core_nif

# Release-build the air_core Erlang NIF (air_core.erl's -on_load loads from target/release)
air-core-nif-build:
    timeout 300s cargo build -p air_core_nif --release

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

# Regenerate jira-tracking-pack: real-parses docs/jira/v26.7.11/tickets/index.md into
# packs/jira-tracking-pack/instances.ttl + ontology.ttl (+ the compiled-in
# crates/cng/src/jira-data.ttl copy), then runs ggen sync to emit the `jira`
# CLI routes + SPARQL queries into crates/cng/src. The Tera template output
# is not guaranteed rustfmt-clean (line-wrap rules are impractical to
# replicate in a template), so this runs `cargo fmt -p cng` afterward —
# whitespace-only, deterministic, so double-render byte-identity still
# holds — rather than hand-editing the generated `jira_routes.rs`.
jira-tracking-generate:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v ggen >/dev/null || { echo "ggen not found on PATH — run: just install-ggen"; exit 1; }
    python3 packs/jira-tracking-pack/make-ontology.py
    timeout 120s ggen sync run
    cargo fmt -p cng

# Idempotence check: ggen sync twice (after a fresh real-parse) must leave
# generated jira-tracking-pack outputs byte-identical (mirrors chatman-sync-verify).
# Re-runs `cargo fmt -p cng` after this second raw sync too — same reason
# jira-tracking-generate runs it after its own sync (Tera output isn't
# rustfmt-clean), so this check compares fmt'd-to-fmt'd instead of flagging
# the sync-vs-fmt line-wrap delta as if it were real pack drift.
jira-tracking-verify: jira-tracking-generate
    timeout 120s ggen sync run
    cargo fmt -p cng
    git diff --exit-code -- 'crates/cng/src/jira_routes.rs' 'crates/cng/src/queries/jira-list.rq' 'crates/cng/src/queries/jira-evidence.rq' 'crates/cng/src/queries/jira-deps.rq' 'crates/cng/src/queries/jira-report.rq'

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
    echo "G11_OTEL_TO_RDF_MAPPER=ALIVE (crates/cng/src/otel_rdf.rs, PROJ-763; see docs/otel-rdf-handoff.md)"

# Rail F/G reachability demo (PATH_TO_100.md §5.2(a)): real, non-test entry point wiring
# otel_rdf::admit -> otel_rdf::project_admitted_spans/admitted_spans_to_trig ->
# otel_ocel::project_otel_to_ocel -> otel_receipt::receipt_otel_to_ocel over one fixture
# span, printing cngr:receiptHead plus a full TriG dump of G_OTEL/G_OCEL/G_RECEIPT. No
# otel-live feature needed (oxigraph/blake3 are unconditional deps); isolated target dir
# so it never collides with a concurrent agent's default-target build (see the
# CARGO_TARGET_DIR note at the top of this file).
cng-otel-rdf-demo *args:
    CARGO_TARGET_DIR=target/agent-otel-rdf-demo timeout 300s cargo run -q -p cng --bin otel-rdf-demo -- {{args}}

# Finds code that looks complete but isn't wired to anything real: orphaned modules (a .rs
# file that compiles clean but is never `mod`-declared from the crate root, so it's excluded
# from the binary entirely — PROJ-777/778's bug), zero-production-caller pub fns (real, tested,
# reachable only from test code), and doc comments claiming enforcement/verification over a
# function body with no actual branching logic. Heuristic (regex-based, not a real Rust
# parser) — read tools/rigor-gap-scanner/scan.py's own LIMITATIONS section before trusting a
# clean run, and treat every finding as a candidate for a human to look at, not a proof.
# Defaults to scanning the whole workspace; pass a path to scope it, e.g.
# `just rigor-gap-scan crates/praxis-graphlaw`.
rigor-gap-scan path=".":
    python3 tools/rigor-gap-scanner/scan.py {{path}}

# --- Erlang/OTP umbrella (apps/, rebar.config at repo root) ---
# air_core, arazzo_runner, arazzo_atomvm, atomvm_runner. Never invoke `rebar3` directly
# (same rule as `cargo` above) -- use these recipes instead.

# Remove all compiled Erlang/OTP artifacts (_build/{default,test}/lib/*/ebin) so the next
# erlang-compile is a full rebuild from source, not an mtime-based incremental one. Use this
# when a beam may be stale relative to its .erl source despite a normal recompile reporting
# nothing to do -- rebar3's incremental compile is mtime-based, so a source file whose mtime
# was set backward (e.g. by a concurrent session's git checkout/restore on a shared working
# tree) can be silently skipped even though its compiled beam is missing exports the current
# source declares.
#
# CORRECTED (caught by a real ci-clean-verify run): `-a` on its own is `--all` ("clean all
# apps, including deps"), scoped to whatever profile is active -- `default` when no `-p` is
# given (`rebar3 help clean`). It does NOT also cover the `test` profile, despite this
# recipe's own prior comment claiming otherwise. `rebar3 eunit` compiles under the `test`
# profile into a SEPARATE `_build/test/lib/*/ebin` tree; a plain `rebar3 clean -a` never
# touches it, so a stale `_build/test/.../*.beam` can survive indefinitely, get
# reported "nothing to do" by the next incremental `rebar3 eunit`, and diverge from its own
# current source -- reproduced live: `_build/test/lib/arazzo_atomvm/ebin/
# arazzo_atomvm_workflow.beam` was missing the `start/2` export apps/arazzo_atomvm/src/
# arazzo_atomvm_workflow.erl declares, causing 9 real eunit failures (`undef`) even though
# `_build/default`'s freshly-recompiled copy of the same module was correct. Both profiles
# must be cleaned explicitly for this recipe to deliver genuine clean-room.
erlang-clean:
    rebar3 clean -a
    rebar3 clean -a -p test

# Compile the Erlang/OTP umbrella (rebar.config's apps/* + lib/* discovery) from the repo root
erlang-compile:
    timeout 300s rebar3 compile

# Compile, then run the eunit suite across the Erlang/OTP umbrella from the repo root
erlang-test: erlang-compile
    rebar3 eunit

# Compile, then run ONLY the OTP/AtomVM differential comparator eunit module
# (arazzo_runner_atomvm_differential_test -- PROJ-761/PROJ-762, the F17/V12-017
# "AtomVM Edge Runtime" family's Differential Comparator evidence). Scoped with
# `-m` (rebar3's documented module filter, `rebar3 help eunit`) rather than the
# full-umbrella `erlang-test` so crates/multifractal-workflow's F17 module can
# gather real, targeted evidence without paying for the whole 55-test suite on
# every call. Machine-parseable stdout tail: "N tests, M failures" or "All N
# tests passed.", matching what erlang-test's own summary line looks like.
erlang-test-atomvm-differential: erlang-compile
    timeout 60s rebar3 eunit -m arazzo_runner_atomvm_differential_test

# Compile, then run ONLY the arazzo_atomvm_workflow eunit module (swarm audit
# wnl2yhbgm finding #13's AtomVM sibling -- loop/2 and start/2 crash-safety on
# malformed caller-supplied Event/InitOpts). Scoped with `-m` (same convention as
# erlang-test-atomvm-differential above).
erlang-test-atomvm-workflow: erlang-compile
    timeout 60s rebar3 eunit -m arazzo_atomvm_workflow_test

# Compile, then run ONLY the F16 dispatch_statem/dispatch_sup eunit module
# (arazzo_runner_dispatch_statem_test -- V12-016), including its repeated
# start/kill supervisor fault-injection soak test. Scoped with `-m` (same
# convention as erlang-test-atomvm-differential above) so this module's
# supervision-tree churn proof can be re-run in isolation without paying for
# the whole umbrella suite on every call.
erlang-test-dispatch-statem: erlang-compile
    timeout 60s rebar3 eunit -m arazzo_runner_dispatch_statem_test

# Compile, then run ONLY the arazzo_runner_blake3 eunit module (real BLAKE3
# hashing via b3sum, PROJ-781; swarm audit wnl2yhbgm finding #10's
# cross-VM tmp_file_path collision fix). Scoped with `-m` (same convention
# as erlang-test-dispatch-statem above).
erlang-test-blake3: erlang-compile
    timeout 60s rebar3 eunit -m arazzo_runner_blake3_test

# Compile, then run ONLY the arazzo_runner_workflow eunit module -- PROJ-757's
# real supervisor-driven crash+restart/reaction/DETS-reconstruction proof suite,
# plus swarm audit wnl2yhbgm finding #13's uncaught-broker-dispatch-exception
# regression coverage. Scoped with `-m` (same convention as
# erlang-test-dispatch-statem above).
erlang-test-workflow: erlang-compile
    timeout 90s rebar3 eunit -m arazzo_runner_workflow_test

# Compile, then run ONLY the arazzo_runner_broker eunit module -- the real
# production consumer of arazzo_runner_blake3:hex/1 (do_dispatch/6), so
# this is the narrowest scoped check that a blake3.erl change didn't
# regress its actual caller without paying for the whole umbrella suite
# (which includes unrelated soak/differential tests).
erlang-test-broker: erlang-compile
    timeout 300s rebar3 eunit -m arazzo_runner_broker_test

# --- Clean-room CI verification (spans the Erlang umbrella and cng/Rust) ---
#
# WHY THIS EXISTS: rebar3's incremental compile is mtime-based (see the erlang-clean
# comment above), and cargo's incremental compile is likewise reuse-by-default. Both can
# report "nothing to do" and hand back a compiled artifact (a `.beam`, a test binary) that
# no longer matches its own current source -- e.g. a stale `.beam` whose exported
# functions silently drifted from the `.erl` it was built from, invisible to the build
# tool itself and only caught by asking the *runtime* directly (Erlang's `code:which/1` +
# `module_info(exports)`) whether what's loaded actually matches what's on disk. That
# defect class is exactly what a normal `just erlang-test` or `just cng-test` cannot catch,
# because both trust their own incremental-build bookkeeping. This recipe exists so a CI/
# pre-merge gate gets a real clean-room rebuild instead of an incrementally-warm one:
#   1. Erlang/OTP umbrella: `erlang-clean` wipes every profile's `_build/*/lib/*/ebin`
#      before `erlang-compile` (full rebuild from source) and `erlang-test` (full eunit
#      suite) run.
#   2. cng (Rust): a dedicated, disposable `CARGO_TARGET_DIR` (target/ci-clean-verify) is
#      removed before the build so cargo cannot reuse any prior incremental state for this
#      run. This is NOT the developer's normal shared `target/` dir used by other
#      concurrent `just` invocations (see the CARGO_TARGET_DIR note at the top of this
#      file) -- only the dedicated isolated dir this recipe owns is ever removed.
# Prints one unambiguous "ci-clean-verify: PASS" line at the end; `set -euo pipefail`
# means any failing step (Erlang compile/test, or cargo test) aborts the script and
# `just` reports a nonzero exit -- there is no partial-pass ambiguity.
ci-clean-verify:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== ci-clean-verify [1/2]: Erlang/OTP umbrella clean rebuild ==="
    just erlang-clean
    just erlang-compile
    just erlang-test
    echo "PASS: erlang-clean + erlang-compile + erlang-test (full eunit suite)"

    echo "=== ci-clean-verify [2/2]: cng (Rust) clean rebuild ==="
    rust_target="target/ci-clean-verify"
    rm -rf "$rust_target"
    CARGO_TARGET_DIR="$rust_target" timeout 1800s cargo test -p cng --features bench
    echo "PASS: cng clean rebuild + test suite (CARGO_TARGET_DIR=$rust_target)"

    echo "ci-clean-verify: PASS -- genuine clean-room rebuild + test on both the Erlang umbrella and cng (Rust)"

# --- v26.7.13 Dry-Run Publish gate (docs/releases/v26.7.13/, packs/dry-run-publish-pack/) ---
#
# `cargo publish`/`cargo package` are blocked from direct invocation by
# .claude/hooks/block-direct-cargo.sh (CLAUDE.md: route cargo through just) -- these two
# recipes are that routing. Each is scoped to ONE crate (dependency-order fan-out lives in
# the harness, crates/cng/src/bench/dry_run_publish.rs, not in justfile), and uses a
# per-crate isolated CARGO_TARGET_DIR so a package/publish attempt for one crate never
# collides with another concurrent `just` invocation's build (see the "Isolated-target
# cargo recipes" note above cng-check-isolated).

# `cargo package --locked` for one workspace member, isolated target dir. Verifies the
# package file list assembles and the manifest is well-formed; does NOT build the crate
# (that's cargo-publish-dry-run below). `name` is a caller-chosen isolation tag (unique per
# concurrent run), `crate` is the -p package name.
cargo-package-dry-run name crate:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 300s cargo package --locked -p {{crate}}

# `cargo publish --dry-run --locked` for one workspace member, isolated target dir. This
# DOES build the crate in an isolated environment (closer to what crates.io's build farm
# would see) but never uploads anything -- `--dry-run` is load-bearing, never omit it.
cargo-publish-dry-run name crate:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 600s cargo publish --dry-run --locked -p {{crate}}

# Run one ggen integration-test binary in an isolated target dir
# (concurrent-agent-safe, same convention as cng-test-isolated).
ggen-test-isolated name binary *args:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 600s cargo test -p ggen --test {{binary}} {{args}}

# Run the FULL ggen test suite (lib + every integration-test binary, including
# cross_pack_matrix and framework_packs_e2e which sweep every pack under packs/) in an
# isolated target dir (concurrent-agent-safe; mirrors cng-test-bench-isolated). Use this
# for whole-crate regression sweeps where ggen-test-isolated's single-binary scoping
# would miss cross-binary interactions -- e.g. after a ggen.toml pack-wiring change.
ggen-test-all-isolated name *args:
    CARGO_TARGET_DIR=target/agent-{{name}} timeout 600s cargo test -p ggen {{args}}
