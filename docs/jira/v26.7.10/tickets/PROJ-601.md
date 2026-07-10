# PROJ-601 — Fix `digests.json` path-portability bug in `cng benchmark verify`

Status: CLOSED

Closed by commit `40f6020`. Verification evidence recorded in
`docs/releases/v26.7.10/RELEASE_CONTROL.md` Sec. 7 (ladder items 7-8: bundle relocated via
`cp -R X Y && rm -rf X`, `just cng-bench-verify Y` exit 0, `REPLAY_RESULT=3/3` identical to
the pre-move result) — this ticket cites that record rather than re-asserting it.

`digests.json` keys are `dir.display().to_string()` captured at `run` time (absolute/
CWD-relative path strings, `crates/cng/src/bench.rs`). `verify()` (`bench.rs:2064`) reads these
keys back verbatim without rejoining against its own `--dir` argument, so copying a benchmark
bundle to a different machine or directory silently fails to resolve files instead of replaying
cleanly. Fix: store paths relative to `bench_dir` at write time, or rejoin recorded keys against
the `--dir` argument passed to `verify`. Links back to `docs/releases/v26.7.10/PRD.md` (Claims
Reconciliation row 3) and `RELEASE_CONTROL.md` Sec. 5.

Implementation detail: `docs/releases/v26.7.10/IMPLEMENTATION_SPEC.md` (exact edits,
anchors, tests, and acceptance commands for this ticket).
