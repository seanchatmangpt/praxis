# Build Caching and Iteration Speed

Discovered 2026-07-09 while closing out Chatman Engine v26.7.9: a `cargo test -p
praxis-graphlaw chatman --release` run stalled far longer than expected. Root causes, in
order of impact:

## 1. Never run two `cargo test`/`cargo build` invocations concurrently in this repo

They serialize on the shared `target/` lock and silently double the wall-clock cost instead of
parallelizing. Before backgrounding a new cargo invocation, check for a stray one first:

```bash
ps aux | grep -E "cargo (test|build)" | grep -v grep
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

## See also

- `/Users/sac/praxis/CLAUDE.md` — project invariants and standing policy
- `justfile` — `test-changed` recipe (existing fast inner loop)
