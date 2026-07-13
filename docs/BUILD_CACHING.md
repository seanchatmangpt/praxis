# Build Caching and Iteration Speed

Discovered 2026-07-09 while closing out Chatman Engine v26.7.9: a `cargo test -p
praxis-graphlaw chatman --release` run stalled far longer than expected. Root causes, in
order of impact:

## 1. Never run two `cargo test`/`cargo build` invocations concurrently in this repo

They serialize on the shared `target/` lock and silently double the wall-clock cost instead of
parallelizing. Before backgrounding a new cargo invocation, check for a stray one first — widen
the grep to `rustc`/`rebar3` too (the Erlang side, `apps/arazzo_*`, has the same lock-contention
failure mode against its own `_build/` dir):

```bash
just check-lock   # ps aux | grep -E "cargo|rustc|rebar3"
```

## 2. `--release` here means full LTO, codegen-units=1 — do not use it for iteration

`praxis-graphlaw`'s release profile is `-C lto -C codegen-units=1` (correctness/benchmark
config, not a dev-loop config). Every one of the ~30+ separate integration-test binaries under
`tests/*.rs` pays full-LTO link cost on any change to `praxis_graphlaw`, even though
determinism and correctness do not depend on optimization level. For iteration:

```bash
# fast inner loop — no LTO, no cross-binary link cost
cargo test -p praxis-graphlaw --test chatman_acceptance_admission

# only reach for --release when actually measuring perf/benchmarks
```

## 3. Scope by exact test binary, not by substring

`cargo test -p praxis-graphlaw chatman` matches every test binary whose *name or contents*
contain "chatman" — this compiles and links dozens of binaries even when only one is of
interest. Prefer `--test <exact_binary_name>` (see `just test-changed` for the fast-path
recipe already in the justfile).

## 4. No `sccache` installed at time of writing

`target/` was 156GB with no `RUSTC_WRAPPER` set — nothing caches object-level compilation
across the many test binaries that share `oxigraph`/`praxis_graphlaw`/etc. as a common
dependency. Installing it is a one-time, local, no-credential action:

```bash
brew install sccache
export RUSTC_WRAPPER=sccache   # add to shell profile to persist
sccache --show-stats           # verify it's engaging after a build
```

Expected win: repeated rebuilds of the many chatman/shacl/n3/etc. test binaries reuse shared
generic-instantiation object code instead of recompiling it per binary.

## 5. Isolated `CARGO_TARGET_DIR=target/agent-*` dirs need active cleanup — they are not self-cleaning

Concurrent agents/terminals against the shared `target/` lock (see §1) can instead each use their
own isolated target dir: `CARGO_TARGET_DIR=target/agent-<name> cargo ...`, or the equivalent
`just <crate>-check-isolated <name>` / `<crate>-test-isolated <name>` recipes (full rationale in
the "Isolated-target cargo recipes" comment block in `justfile`, above `cng-check-isolated`).
This buys real concurrency at the cost of a slower first build per isolated dir (no shared
incremental cache) — and every one of those dirs (several GB each, full dependency tree
recompiled) sits on disk until something removes it. A single agentic session doing this
repeatedly across many crate families (`multifractal-workflow`, `praxis-graphlaw`, `cng`, ...)
has, in practice, filled disk toward 100% more than once this way.

```bash
just list-isolated          # size of every target/agent-* dir, warns if a build is running
just clean-stale-isolated   # removes them all, but refuses (exit 1) if cargo/rustc/rebar3 is running
```

`clean-stale-isolated` is crate-agnostic and safety-checked (see `just check-lock`'s pattern) —
prefer it over the older, cng-scoped `cng-clean-all-isolated` (identical `rm -rf target/agent-*`,
but no running-build check) or hand-typing `rm -rf target/agent-*`. Run it periodically during any
session that uses isolated builds, not only when `df -h` finally complains.

## See also

- `/Users/sac/praxis/CLAUDE.md` — project invariants and standing policy
- `justfile` — `test-changed` recipe (existing fast inner loop)
