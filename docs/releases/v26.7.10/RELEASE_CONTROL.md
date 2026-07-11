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
- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730 — governing v26.7.10-revised
  doctrine pointed to by this file)
- `docs/releases/v26.7.10/DEFINITION_OF_DONE_INTERIM.md` (PROJ-606 — superseded interim DoD,
  preserved fix-forward)
- `docs/releases/v26.7.10/DOD_EVIDENCE_MAP.md` (PROJ-748 — governing evidence index for
  v26.7.10-revised)
- `docs/releases/v26.7.10/DOD_SIGNOFF.md` (PROJ-748 — governing clause-by-clause sign-off for
  v26.7.10-revised)
- `docs/releases/v26.7.10/DOD_EVIDENCE_MAP_INTERIM.md`, `DOD_SIGNOFF_INTERIM.md` (PROJ-617 —
  superseded interim evidence map + sign-off, preserved fix-forward, PROJ-748)
- `docs/jira/v26.7.10/tickets/PROJ-606.md` .. `PROJ-622.md` (Sec. 8 table counterparts)
- `docs/jira/v26.7.10/tickets/PROJ-701.md` .. `PROJ-749.md` (Sec. 9 scope; 715-719 skipped)

## 8. PROJ-606..622 closure table (PROJ-617, 2026-07-10)

Statuses below reflect the closing session, including the consolidated final build,
orchestrator-verified this session:

1. `just cng-test-bench` — ALL GREEN: 40 lib tests passed, 0 failed; integration suites
   6/1/1/5/4/2 passed, 0 failures anywhere. Includes the PROJ-614/616/622 tests (marker
   positive+negative, `CNG_R19`/`CNG_R20`, all 5 tamper negatives, in-process determinism
   gate).
2. `just cng-workday-verify` (seed=616, ticks=8, rpm=125) — two same-seed runs produced
   byte-identical evidence bundles; report shows all 11 success markers TRUE
   (`AUTONOMIC_LOOP_CLOSED` … `V26_7_10_PRODUCTION_READY`); telemetry 64 transitions /
   64 hook actuations / 3 dispatches sent / 3 consequences admitted / 1 refusal resumed;
   `evidence_chain_digest blake3:4e38a38f…0475`, `ocel_graph_digest blake3:853638…b315`,
   `run_hook_hash ba8615…8ffe`.

Fix-forward record: after the first consolidated build, the orchestrator rewrote
`crates/cng/queries/markers/marker-child-closure.rq` and
`crates/cng/queries/metric-dispatch-closure.rq` — a SPARQL scoping bug (FILTER on outer-bound
`?law` inside UNION branches, unbound in branch scope, so `satisfiedParents` was always 0) was
fixed by matching the closure law as a triple pattern inside each UNION arm, mirroring
`dispatch-closure.rq`. The green results above postdate this fix.

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
| PROJ-614 | Graph-authoritative metrics closure | ALIVE | consolidated build items 1-2 above; `metric-hook-actuations.rq`/`metric-dispatch-closure.rq` on disk (old `metric-hook-receipts.rq` deleted) |
| PROJ-615 | Optional ed25519 signatures | CUT | optional cut line exercised — see below |
| PROJ-616 | Verification harness | ALIVE | consolidated build item 1 (5 tamper negatives, determinism gate) + item 2 (byte-identical same-seed bundles) |
| PROJ-622 | SPARQL success markers | ALIVE | consolidated build item 2 — all 11 markers TRUE via SPARQL; marker negatives green in item 1 |
| PROJ-617 | Closure (complete) | ALIVE (doc) | this section + `DOD_SIGNOFF.md` |

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
`V26_7_10_PRODUCTION_READY` is claimed **scoped exactly as the DoD defines it**: SPARQL-derived
marker conjunction TRUE on the `just cng-workday-verify` run above, with loopback-real external
dispatch, MOCKED-HUMAN synthesized consequences, and live network endpoints out of scope
(Sec. 8.2). No unscoped production-ready claim is made.

## 9. v26.7.10-revised scope (PROJ-701..731) — No-LLM planning + multi-engine execution

The v26.7.10 DoD is superseded in place (2026-07-10): governing doctrine is the rewritten
`DEFINITION_OF_DONE.md` (PROJ-730); the prior DoD is preserved at
`DEFINITION_OF_DONE_INTERIM.md`. The interim closure of PROJ-606..622 (Sec. 8, commit
`31c236f`, all 11 markers derived TRUE) **stands as evidence of the substrate** — the
single-process autonomic loop, loopback dispatch, 13-state machine, hooks, receipts, replay,
and marker machinery the revised scope builds on. Nothing in Sec. 8 is reopened or downgraded.

Prior wording here read "all rows below are PLANNED — ticket files exist, no code exists."
That header is now FALSE and corrected: Wave 1 + wave 2 landed ~4k LOC across Tracks P and E
plus arazzo-pack, but it had never compiled or run until this session's Phase 1-2 fixes
(PROJ-733/734). Rows below reflect what actually ran green this session (`cargo test -p cng
--features bench`: 107 tests total, 0 failures — see the ladder below); PROJ-714 remains
PLANNED as a genuinely never-built, declared cut line (§9.2). No row flips to ALIVE without a
fresh command + output cited here or in the ticket file it points to. Ticket numbers
PROJ-715..719 are deliberately skipped (track separator; no tickets ever existed there); this
session added PROJ-733/734 (Phase 1 fixes, beyond the original plan's ticket range),
PROJ-739..748 (Phase 3-5), a follow-up verification round (closing the literal 8² fan-out,
the real two-bundle `full_production_ready` composition, the arazzo digest-verify wiring, the
full 5x20 IPC corpus scale, and `CNG_R09`'s negative test), and a second, separate synthesis
round that added PROJ-749 (the decompose-to-dispatch bridge, closing the Track P/Track E
integration gap at the mechanism level) plus a workspace-wide sanity sweep and the two
remaining §18 negative-corpus items — see §9.1 items 7-8 and §9.3 below.

### 9.1 Verification ladder — evidence for PROJ-701..713/720..729/733/734/739..742/749 ALIVE

Commands run this session (cited exactly; not re-derived):

1. `cargo test -p cng --features bench --test cng_decomp`: 3/3 passed, 0.18s (was 60s+ hang
   before PROJ-733's grounder swap).
2. `cargo test -p cng --features bench --test cng_ipc_corpus`: 10/10 passed, 1.79s (was
   minutes/OOM-risk; two negative fixtures corrected — see PROJ-712).
3. `cargo test -p cng --features bench --test cng_multi_engine -- --test-threads=1`: 6/6
   passed, including `g13_crash_resume_verifies_chain_and_completes` (confirms PROJ-734's
   watch-loop race fix holds).
4. `cargo test -p cng --features bench` (full suite): 107 tests total, 0 failures (67 lib + 6
   cng_bench_portability + 1 cng_cli_smoke + 3 cng_decomp + 1 cng_hierarchical + 10
   cng_ipc_corpus + 6 cng_multi_engine + 5 cng_negative_fixtures + 4 cng_pipeline + 2
   cng_workday_verify + 2 no_inline_ttl_guard), ~109s wall time (was `exit 124` timeout at
   900s before PROJ-733's fix). Predates the follow-up round's and second synthesis round's
   new test binaries (`cng_production_ready.rs`, `cng_ipc_corpus_full_scale.rs`,
   `cng_decompose_to_dispatch_integration.rs`, `cng_decomp_negative_corpus_completeness.rs`)
   — this figure is not re-stated as a new combined total anywhere in this document; each new
   binary's own count is cited separately (items 7-8 below and §9's PROJ-742/711/749 rows).
5. `cargo check -p cng` (no `--features bench`, the default/publishable surface): compiles
   clean, 0 warnings — confirms `pddl-index` (bench-only optional path dependency,
   `Cargo.toml:42-50`) does not participate in the default build.
6. `cargo test -p cng --features bench --lib`: 67/67 passing, re-confirmed after gating two
   dead-code warnings (`DEFINITION_OF_DONE.md` §16 verb-related consts) `#[cfg(feature =
   "bench")]`.
7. (Second synthesis round) `cargo test -p cng --features bench --test
   cng_decompose_to_dispatch_integration`: 2/2 passed, 1.76s — PROJ-749, the decompose-to-
   dispatch bridge; see §2/§8 of `DOD_SIGNOFF.md` for the reconciled clause status and the
   honest boundary (no PDDL-payload-carrying contract yet).
8. (Second synthesis round) `just cng-test-one cng_decomp_negative_corpus_completeness --
   --nocapture`: 2/2 passed, run twice, 0.05s — closes DoD §18 negative-corpus items 6 and 7
   (item 7 additionally, alongside the follow-up round's own `CNG_R09` test); see PROJ-712/713.

Per-ticket evidence citations (exact test names, file:line) live in each ticket file's own
"Evidence (this session)" section — not restated here to avoid drift between two copies.

| Ticket | Scope item | Status |
|---|---|---|
| PROJ-701 | `pddl-strips.ttl` ontology + closed shapes | ALIVE |
| PROJ-702 | Lifter: PDDL string literal → pddl-strips triples | ALIVE |
| PROJ-703 | Deterministic PDDL renderer + round-trip property test | ALIVE |
| PROJ-704 | `rules/decomp.dl` + `decomp-resources.dl` edge derivation | ALIVE |
| PROJ-705 | Bounded canonical candidate enumeration (single-actor = #0) | ALIVE |
| PROJ-706 | CONSTRUCT manufacture of helper/main problem graphs | ALIVE |
| PROJ-707 | Interface state `s′` replay + `CNG_R23 InterfaceStateMismatch` | ALIVE |
| PROJ-708 | Non-interference `CNG_R22` + release closure `CNG_R24` | ALIVE |
| PROJ-709 | POWL nested-PartialOrder composition + powl2 emission | ALIVE |
| PROJ-710 | Selection law, candidate receipts, typed `DecompositionOutcome` | ALIVE |
| PROJ-711 | IPC generators (5 domains × 20, solvability gate) | ALIVE (full 5x20=100 scale run, follow-up round) |
| PROJ-712 | Potato canonical scenario + negative corpus | ALIVE |
| PROJ-713 | Anti-hardcoding gate | ALIVE |
| PROJ-714 | 4 long-horizon scenarios | ALIVE (mechanism, 1/4) / PLANNED (2-4, time-boxed — §9.2 below) |
| PROJ-720 | 16-state dispatch machine everywhere + drift test | ALIVE |
| PROJ-721 | Durable dispatch ledger + idempotent consume (`DoubleAdmit`) | ALIVE |
| PROJ-722 | Deterministic `EngineIdentity` + per-engine bundle layout | ALIVE |
| PROJ-723 | `cng engine serve` verb (bounded receipted poll loop) | ALIVE |
| PROJ-724 | `cng engine resume` + `--partial` prefix replay | ALIVE |
| PROJ-725 | Arazzo 1.1 vocab/shape delta + REMOTE_* projection | ALIVE |
| PROJ-726 | `packs/arazzo-pack/`: graph → arazzo/openapi/asyncapi YAML | ALIVE |
| PROJ-727 | Distributed evidence: OBS_KINDS, OCEL construct, markers | ALIVE |
| PROJ-728 | Multi-process harness + isolation falsifiers | ALIVE, scoped to CARGO_BIN_EXE test harness |
| PROJ-729 | G13 crash-resume, byte-identity, 8² across engines | ALIVE, scoped to CARGO_BIN_EXE test harness (literal 8²=64-leaf fan-out achieved, follow-up round) |
| PROJ-730 | Revised DoD doctrine + ticket set | IN PROGRESS (§16 reconciled this session; closes when committed) |
| PROJ-731 | Final v26.7.10-revised closure + sign-off | CLOSED (doc) — see DOD_SIGNOFF.md; two-way (workday+planning) full-conjunction claim ALIVE, follow-up round; three-way (+distributed) UNVERIFIED |
| PROJ-733 | `pddl-index` grounder swap (performance fix) | ALIVE |
| PROJ-734 | G13 watch-loop race fix (`.ttl`-only filter) | ALIVE |
| PROJ-739 | 6 planning marker queries + `PLANNING_MARKER_MAP` | ALIVE |
| PROJ-740 | 3 `LLM_CALLS_ZERO` family markers | ALIVE |
| PROJ-741 | `cng plan decompose` verb | ALIVE |
| PROJ-742 | `full_production_ready` conjunction combinator | ALIVE (pure function) / ALIVE (real two-bundle invocation, follow-up round) / UNVERIFIED (real three-bundle, +distributed) |
| PROJ-743 | DoD §16 marker-name reconciliation | DONE (doc) |
| PROJ-744 | `arazzo-pack` registered in `ggen.toml` | ALIVE |
| PROJ-745 | `verify_arazzo_render_digest` seam | ALIVE (function, wired into `arazzo::run_arazzo_projection`, follow-up round) |
| PROJ-746 | Ticket status flips + this table + `index.md` sync | DONE (doc) |
| PROJ-747 | PROJ-714 cut-line record (this subsection) | DONE (doc) |
| PROJ-748 | Revised `DOD_SIGNOFF.md`/`DOD_EVIDENCE_MAP.md` | DONE (doc) |
| PROJ-749 | Decompose-to-dispatch bridge (Track P/E integration, second synthesis round) | ALIVE (mechanism, non-potato fixture) |

Marker note: `V26_7_10_PRODUCTION_READY` keeps its name but its meaning is revised to the
`DEFINITION_OF_DONE.md` §16 conjunction (`LLM_CALLS_ZERO` family + planning set + distributed
set — names reconciled to on-disk identifiers at PROJ-743), scoped by the §20 honest
boundaries (filesystem transport; HTTP binding declared via generated OpenAPI/AsyncAPI docs
but UNVERIFIED as a live network path; long-horizon set is the cut line — see
`DEFINITION_OF_DONE.md` §20 item 1 for the current, precise Arazzo-vs-OpenAPI/AsyncAPI
render-verification split; the load-bearing closure wave's own investigation, at the time it
ran, found zero Rust consumption of either document — `GAP_AUDIT.md` §7 item 8 — narrowing the
DoD's claim accordingly; DoD §20's text may since have moved further, not independently
re-verified by this doc). Each marker family
is independently ALIVE; a follow-up verification round then invoked the two-run conjunction
end-to-end — `full_production_ready` combining a REAL `workday()` bundle and a REAL
`decompose()` bundle, all 26 keys `true` (PROJ-742, `cng_production_ready.rs`) — so the §16
conjunction is now ALIVE for the two-way (workday + planning) composition. The three-way
composition (+ a real distributed bundle) remains UNVERIFIED: it requires
`cng_multi_engine.rs`'s private harness helpers, not importable from a separate test crate.

### 9.2 PROJ-714 cut record (G14/G15) — updated, EOD push

Originally: PROJ-714 (4 long-horizon scenarios, G14/G15, `DEFINITION_OF_DONE.md` §19/§20 item
3) was never built, by design, and this subsection recorded it as the release's declared cut
line. That has partially changed: an EOD push built one real, non-stubbed long-horizon
scenario — `tests/cng_long_horizon_scenario.rs`
(`long_horizon_logistics_scenario_decomposes_and_plans_end_to_end`, run this session, 1/1
passed, 0.24s/0.27s across two independent runs) — a two-package, 16-room-corridor logistics
domain whose single-actor plan is 30 real steps and whose helper/main decomposition genuinely
wins the selection law (makespan 15 vs. 30), proving the full pipeline (grounding → Datalog
edge derivation → candidate search → planning → interference/release proofs → selection →
receipt) holds at this length without the grounding-blowup cliff PROJ-733 fixed. `PROJ-714.md`
now reads `ALIVE (mechanism, 1/4)` / `PLANNED (2-4, time-boxed cut)`. Scenarios 2-4, per
`PROJ-714.md`'s own revised scope, are three parameter variations of the existing 5-domain IPC
generator family at extended plan length — a time-boxed cut this session (2.2h EOD window),
not silently dropped, and not the same gap as PROJ-714's original "nothing exists" state. Do
not cite this subsection as "PROJ-714 = cut line, nothing built" — that framing is now stale;
cite `PROJ-714.md` directly for the precise, current 1-of-4 scope. Distinct from PROJ-711's IPC
corpus (a separate ticket, ALIVE at full 5x20 scale) — the two remain not to be conflated.

### 9.3 Session sanity sweep (second synthesis round): workspace check, clippy, fmt

Three of this session's second-round agents ran read-only-scoped sanity passes over the
touched crates, not directly named in the §9 table above (no code changes to Track P/E
tickets resulted; two of the three found and fixed unrelated pre-existing lint debt).

1. **Workspace-wide build — clean.**
   `CARGO_TARGET_DIR=target/agent-workspace-check cargo check --workspace --all-features`:
   `Finished` in 5m 38s, zero errors; only pre-existing warnings unrelated to this session
   (`ggen`, `cng/src/bench/{dispatch,engine}.rs`, `ggen/src/bin/mcp_server.rs`).
   `cargo test --workspace --all-features --no-run`: exit 0, every workspace test binary
   compiled. Confirms `pddl-index`'s addition to `cng`'s bench-only feature surface
   (`crates/cng/Cargo.toml`) does not affect `praxis-synthesis` (the other consumer of
   `pddl-index`) or any other workspace member.
2. **Scoped clippy sweep (`praxis-graphlaw` + `pddl-index`) — 6 real items fixed, ~60
   pre-existing unrelated errors named and left untouched.** Removed two unused private
   functions (`hooks/delta_query.rs::delta_touches`, `shacl/index_utils.rs::contains_triple`,
   both exact duplicates of actively-used code elsewhere), added a documented
   `#[allow(dead_code)]` to `shacl/closure.rs`'s `dense_to_global` (a deliberately-reserved
   PROJ-416 seam per its own doc comment, not vestigial code), and added missing `///` doc
   comments to `pddl-index/src/ground.rs`'s `IndexedGroundProblem` public fields (satisfying
   `#![warn(missing_docs)]`). Confirmation command:
   `CARGO_TARGET_DIR=target/agent-clippy cargo clippy -p praxis-graphlaw -p pddl-index
   --all-targets --all-features -- -D warnings` — exit 101, NOT a clean pass and not claimed
   as one. All 6 assigned items are confirmed gone by grep; `pddl-index` alone lints clean;
   the remaining ~60 errors span 22 OTHER `praxis-graphlaw` files (`tripleindex.rs`,
   `hooks/quads.rs`, `reasoner/mod.rs`, `sparql/mod.rs`, `owlrl/mod.rs`, and others),
   last touched at commit `2dd4f04` ("PROJ-411..417: Chatman Engine v26.7.9"), predating this
   session — explicitly out of the assigned scope and not fixed here. A future session should
   file this pre-existing backlog as its own ticket if it is to be closed.
3. **`cargo fmt --all --check` — clean, zero files flagged.** Run via `just fmt-check`
   (direct `cargo fmt` is blocked by `.claude/hooks/block-direct-cargo.sh`); confirmed the
   toolchain (`nightly-2026-06-22`, pinned in `rust-toolchain.toml`) is actually active, not a
   silent no-op. Exit 0 across all 17 workspace members, including every file in this
   session's "hot" concurrently-edited set (`crates/cng/src/bench/{dispatch,arazzo,workday,
   decomp,engine}*`, `crates/pddl-index/src/ground.rs`, `crates/praxis-graphlaw/src/{hooks/
   delta_query.rs,shacl/closure.rs,shacl/index_utils.rs}`). Point-in-time caveat, stated by
   the checking agent itself and repeated here honestly: the negative-corpus completeness
   file and the PROJ-749 bridge/test files landed after this check ran, so their formatting
   was not directly confirmed by this specific pass — a final `just fmt-check` immediately
   before any release gate is prudent, not because a problem is known or expected, but
   because those specific files predate this check by only minutes and were not re-verified.

## 10. Roadmap — deferred deployment increments (not in v26.7.10-revised scope)

1. **AtomVM-wrapped Chatman Engine deployments.** Target: run the Chatman Engine's WASM build
   under AtomVM (the lightweight BEAM implementation that compiles to WebAssembly), so engine
   instances deploy as BEAM-supervised WASM nodes — browser, Node.js, or microcontroller-class
   hosts — with Erlang/OTP supervision as the fault-tolerance layer around the deterministic
   core. Fit notes recorded now, unverified: the engine's logical-tick discipline (Chatman
   Constant, no wall clock) and WASM target were designed for exactly this class of host; OTP
   supervision would wrap, never replace, typed refusals — a crashed engine process restarts
   from its receipt chain/ledger (the v26.7.10-revised G13 resume machinery is the intended
   substrate). Status: UNVERIFIED research direction; no AtomVM artifact exists in this repo.
   Prereqs: ChatmanEngine adoption in `cng` (itself deferred, Sec. 8), a wasm32 build gate,
   and a transport binding beyond the filesystem loopback (Sec. 8.2 boundary).
2. **ChatmanEngine adoption in `cng`** (carried from Sec. 8) — precedes item 1.
3. **Live network transport binding** for the generated OpenAPI/AsyncAPI contracts (Sec. 8.2 /
   DoD §20 boundary) — precedes any distributed AtomVM fleet claim.
