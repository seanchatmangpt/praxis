# Getting Started with cng

One command to confirm your checkout is set up correctly: `just cng-smoke`. This doc
is only that — clone, run, see green. For the command surface see `README.md`, for
crate conventions (module map, refusals, tests, determinism) see `CONTRIBUTING.md`,
for a full command-by-command reference see `CHEATSHEET.md`.

## Prerequisites

- The nightly toolchain this workspace pins (`rustup show` from the repo root; the
  `runner` feature, on by default, needs it).
- `just` on `PATH` (`brew install just` or see the project's own install docs).

## Run the smoke check

```bash
just cng-smoke
```

This runs, in order, fail-fast: a `workflow doctor` health check, a real 4-tick
`benchmark workday` run, a real `plan decompose` run against an on-disk fixture, and
the `cng_cli_smoke` integration test (`plan generate` → `workflow export` → `workflow
inspect` over `plans/joseph/`). Each step prints a `PASS:`/`FAIL:` line; the recipe
exits nonzero on the first failure instead of running the rest. A final one-line
summary confirms all four passed.

First run compiles the `bench` feature (pulls in `praxis-graphlaw`,
`wasm4pm-cognition`, `pddl-index`) from scratch into an isolated `target/agent-smoke`
directory — expect a real compile, not a sub-30-second wall clock, the first time.
Re-runs reuse that incremental cache. Reclaim the disk afterward with
`just cng-clean-isolated smoke`.

## If it fails

Read the `FAIL:` line and the command output above it — each step names the exact
`cargo`/`cng` invocation it ran. See `CONTRIBUTING.md` §2 for the concurrent-build
lock-contention note if the failure looks like a hang rather than an error.

## See also

- `README.md` — product thesis and full `cng` command surface
- `CONTRIBUTING.md` — module map, typed-refusal convention, test/determinism rules
- `CHEATSHEET.md` — one real invocation per verb, copy-paste ready
