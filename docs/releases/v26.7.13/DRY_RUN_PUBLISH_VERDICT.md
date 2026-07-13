# v26.7.13 — Dry-Run Publish Verdict

Version marker: v26.7.13 · Verdict compiled 2026-07-13 · HEAD `bf982815` at compile time.

Not a "release went live" statement in any sense — see the DRY-RUN-OVERCLAIM FENCE in
`packs/dry-run-publish-pack/ontology.ttl`. This document states DRY-RUN GATE VERIFICATION
status only: which of the DoD's 6 blocking-gate categories have real, executed evidence,
which do not, and why. It is never a statement that `cargo publish` (without `--dry-run`)
ran, or that any crate reached crates.io.

## Status

**REFUSED.** Zero of the 6 blocking-gate categories (Scope, Generate, Verify, Manufacture,
Cleanroom, Receipt) defined in `packs/dry-run-publish-pack/ontology.ttl` have executed,
passing evidence. Gate 1 (Release Scope & Identity) fails deterministically on two of its
own checkboxes, verified this session: the worktree is not clean, and the three release
documents this gate would check for sync (`PRD.md`/`ARD.md`/`RELEASE_CONTROL.md`) contain
zero content about a dry-run-publish gate — they govern an unrelated eight-theme release.
Gates 2 through 6 have no evidence at all, because the Rust harness that would execute
them (`crates/cng/src/bench/dry_run_publish.rs`, `dry_run_publish_test.rs`) does not exist
anywhere in this repository's source tree or commit history. This is a broader and more
fundamental failure than a narrow "gate 1 version-sync mismatch" — no gate 1-6 verdict of
any kind (pass or typed fail) was ever computed, because the code to compute one was never
written. **Only ALIVE satisfies the v26.7.13 dry-run publish Definition of Done, and this
run is REFUSED, not ALIVE.**

## Per-gate breakdown

| Gate | Checkbox | Real status | Evidence citation |
|---|---|---|---|
| 1: Scope & Identity | Clean worktree | FAIL | `git status --short` at HEAD `bf982815f3a6d4cefc2af4a22b42a7ef69732cc9`: ~40 modified/untracked paths unrelated to this task (`.cargo-cicd/ocel/events.jsonl`, `_build/default/lib/...`, `crates/wasm4pm-arazzo/bench_result.txt`, `src/frontier.rs`, `src/mission.rs`, `src/revenue.rs`, `src/revtac.rs`, `src/verbs/doctor.rs`, `src/verbs/frontier.rs`, `tests/frontier_matrix.rs`, `tests/transport3.rs`, `crates/praxis-lean/src/closure.rs`, `crates/praxis-lean/src/receipt_gate.rs`, `crates/praxis-lean/tests/receipt_closure_gate.rs`, `tools/paper-factory/lean-lake/*`, `tmp/`) |
| 1: Scope & Identity | PRD/ARD/RELEASE_CONTROL sync | FAIL | `grep -rn "dry-run-publish\|dry_run_publish\|Kestrel\|DRY-RUN-" docs/releases/v26.7.13/{PRD,ARD,RELEASE_CONTROL}.md` returns zero matches; the three documents' own `## Claims Reconciliation` / `## 1. Evidentiary floor` tables cover eight unrelated themes (crown-witness continuation, ~40-fix basket, ggen parity, TOGAF increment 1, SOC2 testbed, `materialize` hardening, gap-closure batch, arazzo Erlang fixes) |
| 1: Scope & Identity | Publish-set identity | NO EVIDENCE | Only declared publish set is the fictional "Kestrel Toolkit" (`kestrel-core`/`kestrel-cli`/`kestrel-macros`, `packs/dry-run-publish-pack/ontology.ttl:130-135`), explicitly disclosed there as illustrative fixture data, "chosen precisely so this fixture can never be read as this repo's own real crates.io publish set" |
| 1: Scope & Identity | Version sync | NO EVIDENCE | Real crate version exists (`crates/cng/Cargo.toml`: `version = "26.9.10"`) but no declared release-version target for this DoD exists anywhere to sync it against |
| 1: Scope & Identity | Git SHA binding | NO EVIDENCE | No SHA-binding record exists for this DoD; only fact available is current `HEAD` = `bf982815f3a6d4cefc2af4a22b42a7ef69732cc9` (`git rev-parse HEAD`, this session) |
| 2: Deterministic Generation | ggen version pin / sync idempotence / lock correctness | NOT EXECUTED | No test exists (see root cause below); not attempted |
| 3: Verification Ladder | `just verify-all` / `just standing` / no silently-ignored tests | NOT EXECUTED | No test exists (see root cause below); not attempted |
| 4: Package Manufacture | `cargo package --locked` / `cargo publish --dry-run --locked` per publish-set member | NOT EXECUTED | `cargo-package-dry-run`/`cargo-publish-dry-run` justfile recipes exist (`justfile:1306-1313`) but nothing in this session or its predecessors invoked them against any real crate for this DoD; no per-crate result recorded |
| 5: Clean-Room Verification | Unpack/build/test in fresh directory | NOT EXECUTED | No test exists (see root cause below); not attempted |
| 6: Receipt & Replay | Final receipt / OCEL evidence / byte-identical replay / `external_mutation=false` / terminal goal `dry-run-verified` | NOT EXECUTED | No test exists (see root cause below); not attempted |

**Root cause common to Gates 2-6 (and most of Gate 1):** `crates/cng/src/bench/dry_run_publish.rs`
and `crates/cng/src/bench/dry_run_publish_test.rs` do not exist. Confirmed this session by four
independent checks:

1. `find crates/cng/src/bench -iname "*dry*run*"` — empty.
2. `grep -rln "dry_run_publish\|Kestrel\|DRY-RUN-SCOPE\|dry-run-verified" --include="*.rs" .`
   (excluding `_build/`, `target/`, `.git/`, `worktrees/`) — zero hits in any `.rs` file.
3. `crates/cng/src/bench/mod.rs` declares 65 `mod`/`pub mod` entries; none is named
   `dry_run_publish`.
4. `git log --diff-filter=A --oneline --all -- "crates/cng/src/bench/dry_run_publish*"` — empty;
   these two files were never added in any commit in this repository's history.

The DoD's own specified command, re-run fresh this session per instruction (not assumed from
any prior context, since none was available to this session):

```text
$ just cng-test-lib-isolated dryrun-verdict-check bench::dry_run_publish -- --nocapture
   Compiling cng v26.9.10 (/Users/sac/praxis/crates/cng)
warning: `cng` (lib test) generated 1 warning (1 duplicate)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 22s
     Running unittests src/lib.rs (target/agent-dryrun-verdict-check/debug/deps/cng-ca5d8ee13d2d21f7)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 184 filtered out; finished in 0.00s
```

Exit code 0. The `cng` lib built cleanly (warnings are pre-existing, in the `wasm4pm-cognition`
and `multifractal-workflow` dependency crates, unrelated to this gate). Zero of the `cng` lib's
184 tests match the `bench::dry_run_publish` path — a substring-filtered `0 passed; 0 failed`
result is a vacuous match, not a passing gate; it must not be read as evidence any gate ran.

## Required falsifiers

Each row states a claim the task's own framing implicitly required to be true, and whether this
session's direct verification falsified it.

| # | Claim | Result |
|---|---|---|
| 1 | `crates/cng/src/bench/dry_run_publish.rs` exists on disk | FALSIFIED — absent (see Root cause, checks 1-4) |
| 2 | The Gate 1-6 harness is wired into `crates/cng/src/bench/mod.rs` | FALSIFIED — module list has no `dry_run_publish` entry |
| 3 | `PRD.md`/`ARD.md`/`RELEASE_CONTROL.md` already define or reference the dry-run-publish 6-gate DoD | FALSIFIED — zero string matches for the gate vocabulary in any of the three files |
| 4 | `packs/dry-run-publish-pack/templates/` contains rendered PDDL/POWL fragments, per the ontology's own forward reference to them | FALSIFIED — directory exists, 0 files (`ls -la packs/dry-run-publish-pack/templates/`) |
| 5 | Re-running the DoD's specified command produces a gate result (pass or typed fail) rather than zero collected tests | FALSIFIED — fresh run this session: `0 passed; 0 failed; ...; 184 filtered out`, exit 0 |
| 6 | The worktree is clean, satisfying Gate 1's own "clean worktree" checkbox | FALSIFIED — `git status --short` dirty, ~40 paths (see Gate 1 row above) |

All six required falsifiers were successfully falsified — none of the optimistic readings of
"the harness and gates already exist and were already run" survives direct verification against
the live repository.

## Final outcome algebra verdict

Rule (as given): ALIVE only if every blocking gate passed; REFUSED if a deterministic gate
failed; BOUNDED/UNSUPPORTED/INCONSISTENT are also legitimate depending on what the real run
found.

Applying it: ALIVE is impossible — Gates 2 through 6 have no executed evidence of any kind, so
"every blocking gate passed" cannot be asserted. Gate 1 supplies the deterministic failure the
rule requires for REFUSED independent of the missing harness: its "clean worktree" and
"PRD/ARD/RELEASE_CONTROL sync" checkboxes were checked directly against the live repository
this session (falsifiers 3 and 6 above) and both fail, reproducibly, without needing any Rust
test to exist. This is broader than a narrow "gate 1 version-sync" finding — the version-sync
checkbox itself cannot even be evaluated (falsifier-table NO EVIDENCE rows), because no declared
release-version target exists for this DoD to sync against. The dominant, larger-scope finding
is that Gates 2-6's entire verification harness was never implemented (falsifiers 1, 2, 4, 5).

**Verdict: REFUSED.** Only ALIVE satisfies the v26.7.13 dry-run publish Definition of Done, and
this run does not reach it — Gate 1 fails two checkboxes with direct, reproducible evidence, and
Gates 2-6 have zero executed evidence because their harness was never written.

## What would need to change for this to reach ALIVE

This session does not re-derive a remediation roadmap for the underlying crates.io publish
blockers — `docs/PUBLISH_ALL_PRAXIS_PLAN.md`'s B1-B7 taxonomy is the existing, authoritative
one and is cited here by reference:

- **B1** — external, unversioned path deps (`bcinr-pddl`, `bcinr-powl`, `bcinr-powl-receipt`,
  `wasm4pm-compat`) gate the whole `praxis-graphlaw` subtree.
- **B2** — `BUSL-1.1`-licensed `wasm4pm` deps enter MIT/Apache-declared crates non-optionally.
- **B3** — four crates (`audit-tools`, `air_core_nif`, `tmp_sparql2`, `mfact-core`) have no
  `license` field at all.
- **B4** — `tmp_sparql2` is entirely git-ignored, zero packageable files.
- **B5** — no root `LICENSE*` file backs the declared SPDX strings.
- **B6** — ~334 hardcoded `/Users/sac/...` path references would ship publicly.
- **B7** — `praxis-lean` has 3 untracked-but-not-ignored files that would ship; **still open
  today**, confirmed by this session's own `git status --short`
  (`crates/praxis-lean/src/closure.rs`, `crates/praxis-lean/src/receipt_gate.rs`,
  `crates/praxis-lean/tests/receipt_closure_gate.rs`).

Specific to this DoD's own gate structure (not itself part of the B1-B7 crates.io taxonomy, but
a precondition for ever exercising it through this pipeline):

1. Write `crates/cng/src/bench/dry_run_publish.rs` and `dry_run_publish_test.rs`, wire
   `dry_run_publish` into `crates/cng/src/bench/mod.rs`, following the `soc2.rs`/`soc2_test.rs`
   pattern of mechanical assertion over a parsed domain plus adversarial mutants (the
   `packs/dry-run-publish-pack/ontology.ttl` doc comments already forward-reference this file by
   path — it is PLANNED there, not built).
2. Render `packs/dry-run-publish-pack/templates/` from the ontology (currently 0 files) — PDDL
   fragments per phase, matching `soc2-audit-pack`'s 13-template precedent.
3. Clean the worktree, or have the harness scope its "clean worktree" check to an explicit
   path-set — the DoD text does not currently specify a scope, so today any concurrent, unrelated
   dirty file anywhere in the repo would fail Gate 1.
4. Give Gate 1's "PRD/ARD/RELEASE_CONTROL sync" checkbox real content to check against — either a
   dedicated ninth theme in the existing eight-theme `PRD.md`/`ARD.md`, or a standalone
   dry-run-publish release-doc set, since the current three documents do not mention this gate at
   all.
5. Point Gate 4 (Package Manufacture) at this repo's real crates, not the fictional "Kestrel
   Toolkit" — which requires clearing B1/B2 first (or scoping Gate 4 to the crates
   `docs/PUBLISH_ALL_PRAXIS_PLAN.md` §3 already marks non-blocked-on-external, e.g. `praxis-lean`
   once B7 clears).
6. Re-run `just cng-test-lib-isolated <name> bench::dry_run_publish -- --nocapture` and confirm it
   collects more than zero tests, and that every collected test passes.

## Evidence references

- `packs/dry-run-publish-pack/ontology.ttl` — the 6-gate SKOS scheme and fictional publish set.
- `docs/PUBLISH_ALL_PRAXIS_PLAN.md` — B1-B7 blocker taxonomy (§1), per-crate readiness table (§3).
- `docs/releases/v26.7.13/PRD.md`, `ARD.md`, `RELEASE_CONTROL.md` — the unrelated eight-theme
  release these documents actually govern.
- This session's commands: `git status --short`; `git rev-parse HEAD`; `find crates/cng/src/bench
  -iname "*dry*run*"`; `grep -rln "dry_run_publish\|Kestrel\|DRY-RUN-SCOPE\|dry-run-verified"
  --include="*.rs" .`; `git log --diff-filter=A --oneline --all -- "crates/cng/src/bench/
  dry_run_publish*"`; `just cng-test-lib-isolated dryrun-verdict-check bench::dry_run_publish --
  --nocapture`.
