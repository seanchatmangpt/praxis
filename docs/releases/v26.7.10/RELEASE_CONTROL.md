# RELEASE_CONTROL — cng Recursive Workflow Standing Benchmark v26.7.10

Single control surface for `PRD.md` and `ARD.md` in this directory. Both documents' Status lines
tie to this file. If this file and either document disagree, this file wins.

## 1. Evidentiary floor

The evidentiary floor for this release is the `e763f44` baseline campaign, verified this
session:

- `just cng-bench-build` exit 0.
- 10,000-worker, depth-2 campaign: `cng benchmark generate --out <dir> --workers 10000 --depth 2`
  (436 files) → `cng benchmark run --dir <dir>` exit 0, `MEASUREMENT_CLASS=MEASURED_CNG_RESULT`,
  `WORKERS_REPRESENTED=10000`, `WORKFLOW_INSTANCES=109`, `EXECUTED_TRANSITIONS=864`,
  `RECEIPTS_GENERATED=108`, `REFUSED_TRANSITIONS=1`, `RECURSIVE_ATTACHMENTS=8`,
  `DATALOG_DERIVED_ROLES=10000`,
  `OCEL_GRAPH_DIGEST=blake3:37dda8ff5721528cc7952c0cf94141b506fa1a5e92a1d4deabea6cf7f774c7a6` —
  run twice, byte-identical headline numbers and digests.
- `cng benchmark verify --dir <dir>` exit 0, `replayed:3, replay_passes:3,
  exported_validated:3, exported_validation_failures:0`.
- `just cng-test`: all suites pass (powl 10, cng_hierarchical 1, cng_cli_smoke 1,
  cng_negative_fixtures 5, cng_pipeline 4, no_inline_ttl_guard 2).
- `just test-bin chatman_pddl_to_powl_joseph_famine_hierarchical`: 3 passed.

Nothing in `PRD.md`/`ARD.md` may claim ALIVE beyond this floor without citing a fresh command +
output in the same breath.

## 2. Explicit exclusions (verbatim, reused identically in PRD.md and ARD.md)

1. **PQC** — REFUSED this release. Zero design precedent found in the repo this session
   (grepped for pqc/dilithium/kyber/ml-dsa/ml-kem — every hit is a typo-dictionary entry,
   unrelated Cargo.lock crate names, or `docs/forensics/` teardown notes on a rejected,
   flagged-fake external Dilithium implementation). Classical ed25519 signing exists
   (`crates/praxis-core/src/signing.rs`) but has zero wiring into `cng`.
2. **Full depth-5/4,680-attachment campaign** — UNVERIFIED at that scale. Only depth-2/8-
   attachment was actually run. Corrected arithmetic: attachment count at depth n is
   `(8^n − 8)/7`, not `8^n`. Depth 5 → 4,680, not 32,768. The `artifact_sets` cap
   (`crates/cng/src/main.rs:524`, 50,000) is a separate axis (flat set count) from tree-depth
   attachment counts.
3. **Workflow sockets** beyond the existing `attachesWorkflow` mechanism — future increment per
   `crates/cng/BENCHMARK.md`.
4. **Bounded-question/resume loop** — future increment, not in this release, per
   `crates/cng/BENCHMARK.md` and prior session notes.
5. **`verify` re-deriving `ocel_graph_digest`/`sparql_result_digest`** — not implemented this
   session; deferred to PROJ-602.

## 3. Ten verified corrections (binding)

The Claims Reconciliation table in `PRD.md` (rows 2-11) encodes ten corrections found this
session against an earlier, more optimistic stakeholder draft. These are binding: no future
session may restate the optimistic framing without re-deriving fresh evidence that supersedes
this correction. Summary (full detail in `PRD.md`):

1. `verify` re-derives the POWL manufacture digest only, not OCEL/SPARQL digests.
2. "No inline SPARQL" is true for `bench.rs` only — `pipeline.rs:135`, `shape.rs:75,82,122,133,
   146,159` still hold inline SPARQL.
3. Auditor replay has a concrete path-portability bug in `digests.json` key handling
   (`bench.rs`, `run`/`verify`), not merely a missing feature.
4. Recursion-tree attachment arithmetic is `(8^n − 8)/7`, not `8^n`; `artifact_sets` is a
   separate axis.
5. Bundle manifest (`digests.json`) is `{set_dir_path: powl_digest}` only; no single manifest
   names every digest; a prior `results.json` sample missing OCEL/SPARQL digests is UNKNOWN
   pending re-verification, not confirmed as a bug.
6. PQC has zero design precedent; REFUSED this release; ed25519 signing is the real near-term
   seam (`crates/praxis-core/src/signing.rs`), unwired in `cng`.
7. `CngRefusal` has no exhaustiveness registry; `CNG_R08 Nondeterminism` is the wrong reuse
   target for auditor-mismatch; a new `CNG_R11 AuditMismatch` is recommended, PLANNED only.
8. `crates/cng/Cargo.toml:35-50,80-85` already self-documents its registry-only-deps exceptions
   — cite as existing self-disclosure, not a newly-discovered risk.
9. `docs/releases/v26.7.10/` and `docs/jira/v26.7.10/` did not exist before this pass — first
   ticket set for this line of work.
10. Status vocabulary is the 5-value ALIVE/PARTIAL/PLANNED/UNKNOWN/MOCKED set, used exactly as
    in v26.7.9 — no 4-value variant invented.

## 4. Claims Reconciliation table — single logical table, two files

The `## Claims Reconciliation` section in `PRD.md` and `ARD.md` is one logical table maintained
in two places. Any status change requires updating both files in the same commit. PROJ ticket
numbers cited there must match tickets under `docs/jira/v26.7.10/tickets/`.

## 5. v26.7.10 scope

| Ticket | Scope item | File | Status |
|---|---|---|---|
| PROJ-601 | Fix `digests.json` path-portability bug | `docs/jira/v26.7.10/tickets/PROJ-601.md` | ALIVE |
| PROJ-602 | Add `cng evidence replay` verb for third-party auditors | `docs/jira/v26.7.10/tickets/PROJ-602.md` | ALIVE |
| PROJ-603 | Bundle manifest schema (all input/output digests + unpopulated `signatures: []`) | `docs/jira/v26.7.10/tickets/PROJ-603.md` | ALIVE |
| PROJ-604 | Close remaining inline-SPARQL sites (`pipeline.rs`, `shape.rs`), extend guard | `docs/jira/v26.7.10/tickets/PROJ-604.md` | ALIVE |
| PROJ-605 | New `CNG_R11 AuditMismatch` refusal + negative test | `docs/jira/v26.7.10/tickets/PROJ-605.md` | ALIVE |
| PROJ-606 | Definition of Done doctrine | `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` | ALIVE (doc) |

`DEFINITION_OF_DONE.md` is doctrine this control surface points to; it does not compete with
this file. If it and this file disagree, this file wins. Its ALIVE status covers only the
document's existence — the behaviors it codifies are UNVERIFIED/PLANNED (PROJ-608..622).

## 7. Final verification ladder — evidence for PROJ-601..605 ALIVE (this session)

Ran the `docs/releases/v26.7.10/IMPLEMENTATION_SPEC.md` "Final verification ladder" in order,
scratch dirs under `/private/tmp/.../scratchpad/v267.10/` (X then relocated to Y):

1. `just cng-test` — exit 0. 30 tests across powl/cng_bench_portability(0, non-bench
   build)/cng_cli_smoke/cng_hierarchical/cng_negative_fixtures/cng_pipeline/no_inline_ttl_guard,
   all pass (`no_inline_sparql_in_rust_sources` now scans all of `src/`+`tests/`, confirming
   PROJ-604's guard extension; `audit_mismatch_refusal_has_stable_code` confirms PROJ-605's
   `CNG_R11`).
2. `just cng-bench-build` — exit 0.
3. `just cng-bench benchmark generate --out X --workers 10000 --depth 2` — exit 0, 436 files,
   `artifact_sets=100`, `recursion_nodes=9`.
4. `just cng-bench benchmark run --dir X` — exit 0.
5. `just cng-bench benchmark run --dir X` (byte-identical re-run) — exit 0,
   `REPLAY_RESULT=2/2`, `POWL_DIGEST=blake3:d8e8975f...`,
   `OCEL_GRAPH_DIGEST=blake3:8af70fd4...`, `SPARQL_RESULT_DIGEST=blake3:c4bbf146...` — identical
   across both runs.
6. `just cng-bench-verify X` — exit 0, `REPLAY_RESULT=3/3`,
   `{digests_on_record:108, replayed:3, replay_passes:3, exported_validated:3,
   exported_validation_failures:0}`.
7. `cp -R X Y && rm -rf X` — relocated the bundle; `X` no longer exists.
8. `just cng-bench-verify Y` — exit 0, same `REPLAY_RESULT=3/3` payload as step 6 — **PROJ-601
   proof**: `digests.json` keys are bench-dir-relative and rejoin correctly against a moved
   `--dir`.
9. `just cng-evidence-replay Y` — exit 0, `AUDIT_OBS_DIGEST_MATCH=true`,
   `AUDIT_QUERIES_VERIFIED=16`, `AUDIT_OCEL_GRAPH_DIGEST_MATCH=true`, `AUDIT_RESULT=CONFORMANT`;
   JSON report `recomputed_ocel_graph_digest == expected_ocel_graph_digest ==
   blake3:8af70fd4544bc8dc13f9824dc37caf9ad78e1da5c09742ec31655c897805a45f` — **PROJ-602/603
   proof**: independent auditor replay from `Y` alone (no producer state) recomputes evidence
   from the bundled `obs/`, `queries/`, and `evidence-manifest.json`.
10. Tamper: appended `\n# x\n` to `Y/obs/role-part-00001.ttl`, reran
    `just cng-evidence-replay Y` — exit **1**, stderr:
    `Error: ExecutionError { message: "CNG_R11: obs digest mismatch — recomputed
    blake3:639711fe... vs manifest blake3:f9d14bce..." }` — **PROJ-605 proof**: third-party
    integrity failure refuses as `CNG_R11 AuditMismatch`, not a silent pass or a `CNG_R08`
    misuse.
11. `just verify-all` — the workspace-wide `check`/`test`/`clippy`/`doctor` gate times out on
    this machine at the recipe's fixed `timeout 180s`/`timeout 600s` bounds (`justfile:84,91`)
    against the full multi-crate workspace with `--all-features`; three consecutive attempts this
    session: `error: recipe check failed on line 84 with exit code 124` (cold-cache run), then
    `error: recipe test failed on line 94 with exit code 124` (twice, including once with a fully
    warm build cache — `Finished test profile ... in 0.88s` — so the 124 is the `cargo nextest
    run --workspace --all-features` execution itself exceeding 600s, not compilation). This is a
    pre-existing environment/timeout-budget constraint of the whole-workspace recipe, not a
    regression from PROJ-601..605 — the cng-scoped `just cng-test` (item 1 above) is green.
    PROJ-601..605 ALIVE status rests on items 1-10; item 11 is UNKNOWN/BLOCKED at the
    whole-workspace scope and not claimed here.

## 6. Documents governed by this control surface

- `docs/releases/v26.7.10/PRD.md`
- `docs/releases/v26.7.10/ARD.md`
- `docs/jira/v26.7.10/tickets/PROJ-601.md` .. `PROJ-605.md`
- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-606 — doctrine pointed to by this file)
- `docs/releases/v26.7.10/DOD_EVIDENCE_MAP.md` (evidence index for the DoD)
- `docs/releases/v26.7.10/DOD_SIGNOFF.md` (PROJ-617 — clause-by-clause sign-off)
- `docs/jira/v26.7.10/tickets/PROJ-606.md` .. `PROJ-622.md` (Sec. 8 table counterparts)

## 8. PROJ-606..622 closure table (PROJ-617, 2026-07-10)

Statuses below reflect the closing session. ALIVE rows cite this session's green
`just cng-test-bench` run (31 lib tests + integration suites passing, recorded in the session
logs for the PROJ-608..613 and PROJ-618..621 waves). IN_PROGRESS rows are UNVERIFIED pending
the consolidated final build — the orchestrator flips them to ALIVE only after that build runs
green; no such build has run at the time of this writing, and this table does not claim it.

| Ticket | Scope item | Status | Evidence |
|---|---|---|---|
| PROJ-606 | `DEFINITION_OF_DONE.md` document | ALIVE (doc) | file exists; `tickets/PROJ-606.md` CLOSED |
| PROJ-607 | Doc reconciliation pass | ALIVE (doc) | `tickets/PROJ-607.md` CLOSED this session |
| PROJ-608 | `workday` verb | ALIVE | `just cng-test-bench` green this session (session log) |
| PROJ-609 | Interruption + planning categories | ALIVE | same `just cng-test-bench` run |
| PROJ-610 | Standing-next-action query | ALIVE | same run; `standing-next-action.rq` on disk |
| PROJ-611 | Bounded admission resume | ALIVE | same run |
| PROJ-612 | graphlaw hook pack actuation | ALIVE | same run — hook actuations 64/64 receipted |
| PROJ-613 | Dialect registry + `HookStanding` | ALIVE | same run — `CNG_R14` registry gate green |
| PROJ-618 | Dispatch contract / state machine | ALIVE | same run (dispatch wave suites green) |
| PROJ-619 | Broker dispatch + re-admission (loopback) | ALIVE | same run; loopback-real only, see boundary below |
| PROJ-620 | Recursive dispatch / closure / compensation | ALIVE | same run |
| PROJ-621 | Arazzo dialect | ALIVE | same run |
| PROJ-614 | Graph-authoritative metrics closure | IN_PROGRESS — UNVERIFIED pending final build | agent wave running; no green build cited yet |
| PROJ-615 | Optional ed25519 signatures | CUT | optional cut line exercised — see below |
| PROJ-616 | Verification harness | IN_PROGRESS — UNVERIFIED pending final build | agent wave running; no green build cited yet |
| PROJ-622 | SPARQL success markers | IN_PROGRESS — UNVERIFIED pending final build | agent wave running; no green build cited yet |
| PROJ-617 | Closure (this pass) | ALIVE (doc) | this section + `DOD_SIGNOFF.md` |

ChatmanEngine adoption is DEFERRED to a future increment — v26.7.10 uses the `TripleStore` hook
surface, not `ChatmanEngine`.

### 8.1 PROJ-615 cut record

The optional cut line was exercised: ed25519 signatures on workday evidence manifests are
deferred out of v26.7.10. `EvidenceManifest.signatures` remains a deliberately empty
`signatures: []` field — PARTIAL by design, named here so no future session reads the empty
array as an omission or a bug. The near-term seam remains
`crates/praxis-core/src/signing.rs`, unwired into `cng` (Sec. 3 item 6).

### 8.2 Loopback-vs-network boundary (binding honesty note)

External dispatch in v26.7.10 is **loopback-real**: a deterministic filesystem outbox/inbox
that fully exercises the dispatch contract, the 13-state machine, correlation, admission,
timeout, escalation, and compensation paths. The dispatch **mechanism** is ALIVE per the
`just cng-test-bench` evidence above. Live network endpoints are declared out of scope:
**third-party endpoints are UNVERIFIED** and may not be claimed. Synthesized human
consequences are labeled MOCKED-HUMAN wherever they appear.

### 8.3 DoD sign-off

Clause-by-clause sign-off against `DEFINITION_OF_DONE.md`'s 15 sections lives in
`DOD_SIGNOFF.md` (this directory), indexed through `DOD_EVIDENCE_MAP.md`. Per that sign-off,
`V26_7_10_PRODUCTION_READY` is **not** claimed: its conjunction depends on PROJ-614/616/622,
which are UNVERIFIED pending the consolidated final build.
