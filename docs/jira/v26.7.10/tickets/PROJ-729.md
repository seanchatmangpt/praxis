# PROJ-729 — G13 crash-resume falsifier, byte-identity, 8-squared across engines

Status: ALIVE, scoped to the CARGO_BIN_EXE test harness, now including the literal 8² (64-leaf)
fan-out — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

Kill an engine after DISPATCHED → restart → `cng engine resume` → chain-prefix verification
(G13). Two full distributed C+H+M runs produce byte-identical evidence bundles. 8² recursion
spanning engines (DoD §9). Gates: G13 and the distributed half of G16.

## Evidence (this session)

Part of the `cargo test -p cng --features bench --test cng_multi_engine -- --test-threads=1`
run, 6/6 passed this session. `g13_crash_resume_verifies_chain_and_completes`
(`cng_multi_engine.rs:318`) — the direct target of PROJ-734's watch-loop `.ttl`-extension-only
filter fix (the pre-fix version could fire `child.kill()` on a transient `.tmp` file before a
committed ledger `.ttl` existed, making the torn-tail branch flaky).
`distributed_determinism_two_serialized_runs_byte_identical`
(`cng_multi_engine.rs:442-460+`) — two serialized C+H+M runs, every file byte-identical.
`recursion_crosses_engines_depth_two` (`cng_multi_engine.rs:470`) — fan_out=2/depth=2 smoke
test, 14 contracts. Scoped to the CARGO_BIN_EXE test harness (see PROJ-728).

## Evidence (follow-up round — literal 8² fan-out)

`recursion_crosses_engines_full_8x2_fanout`, added to `crates/cng/tests/cng_multi_engine.rs`,
reuses the existing `serialized_run` helper (unmodified) with fan_out=8/depth=2 instead of the
smoke test's fan_out=2 — the literal `8²` scale the module docs and this ticket's own title
cite ("8 + 64 children per root"). Per root: 1 root + 8 first-level children + 64 second-level
(leaf) children = 73 dispatches; two roots (H, M) = 146 total dispatches, 64 of them the
depth-2 leaves. Routing crosses real OS engine processes (`CARGO_BIN_EXE_cng engine serve`),
same mechanism as the rest of the harness — not mocked.

Command: `CARGO_TARGET_DIR=target/agent-728 cargo test -p cng --features bench --test
cng_multi_engine -- --test-threads=1 --nocapture`. Run 1:
`recursion_crosses_engines_full_8x2_fanout ... ok` in 37.19s, full 7-test suite in 45.22s. Run
2 (after a doc-comment-only edit): 32.50s, full suite 41.38s. No scaling-down was needed — the
harness handled the literal target within the `justfile`'s 1800s ceiling for this test binary.

PROJ-729 upgrades from "8² tested at fan_out=2" to "8² tested at the literal fan_out=8, depth=2
target" — closing the gap `DOD_SIGNOFF.md` §9 previously named explicitly.
