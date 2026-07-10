# cng Recursive Workflow Standing Benchmark — Product Requirements Document

Status: DRAFT, tied to `RELEASE_CONTROL.md`. Every claim cites a file, test, or receipt. Rows
without evidence are marked PLANNED or UNKNOWN, never asserted.

This release covers the `cng` benchmark surface (`crates/cng/src/bench.rs`, `bench` cargo
feature) at commit `e763f44`, which is the ALIVE evidentiary baseline for this document. It also
scopes the v26.7.10 hardening work (bundle-manifest completeness, auditor-replay portability,
inline-SPARQL closure, a new refusal variant) as PLANNED — none of it is built yet.

## Claims Reconciliation

Status vocabulary (identical to v26.7.9): **ALIVE** (verified this session, cited test/receipt
passes), **PARTIAL** (real but narrower than claimed — gap named), **PLANNED** (ticket only, no
code), **UNKNOWN** (not yet investigated), **MOCKED** (a stand-in exists where the claim implies
the real thing). This table is authoritative here; `ARD.md` points to it and must not restate a
different verdict for the same row.

| # | Claim | Status | Scope/caveat | Evidence | Open ticket |
|---|---|---|---|---|---|
| 1 | 10,000-worker, depth-2 recursive workflow campaign runs end-to-end and is measured, not modeled | ALIVE | Exact campaign only; not any other worker count/depth | `cng benchmark generate --out <dir> --workers 10000 --depth 2` (436 files) → `cng benchmark run --dir <dir>` exit 0, `MEASUREMENT_CLASS=MEASURED_CNG_RESULT`, `WORKERS_REPRESENTED=10000`, `WORKFLOW_INSTANCES=109`, `EXECUTED_TRANSITIONS=864`, `RECEIPTS_GENERATED=108`, `REFUSED_TRANSITIONS=1`, `RECURSIVE_ATTACHMENTS=8`, `DATALOG_DERIVED_ROLES=10000`, `OCEL_GRAPH_DIGEST=blake3:37dda8ff5721528cc7952c0cf94141b506fa1a5e92a1d4deabea6cf7f774c7a6`; run twice, byte-identical headline numbers and digests | — |
| 2 | Run is independently reproducible/replayable | ALIVE, scoped | `cng benchmark verify --dir <dir>` exit 0, `replayed:3, replay_passes:3, exported_validated:3, exported_validation_failures:0` — a 3-item sample, not full-corpus replay; see row 3 for the portability caveat | `crates/cng/src/bench.rs` `verify()` (~2050-2115) | PROJ-601 |
| 3 | Third-party auditor can move the bundle directory and replay it elsewhere | BLOCKED / MOCKED-adjacent | `digests.json` keys are `dir.display().to_string()` captured at `run` time (absolute/CWD-relative path strings); `verify()` (bench.rs:2064) uses them verbatim without rejoining against its own `--dir` argument — moving the bundle silently fails to resolve files rather than replaying cleanly | `crates/cng/src/bench.rs` `run`/`verify` digest-key construction | PROJ-601 |
| 4 | `benchmark verify` re-derives and checks all evidence digests (OCEL graph, SPARQL results) | PARTIAL, not ALIVE | `verify()` (bench.rs:2050-2115) re-derives and checks the POWL manufacture digest only; it never recomputes or compares `ocel_graph_digest` or `sparql_result_digest` | `crates/cng/src/bench.rs` `verify()` | PROJ-602 |
| 5 | No inline Turtle/SPARQL in Rust source | PARTIAL | True for `bench.rs` only (guard `crates/cng/tests/no_inline_ttl_guard.rs` scans all `.rs` under `src/`+`tests/`, but its needles check inline Turtle-prefix/PDDL markers, not SPARQL SELECT text); `crates/cng/src/pipeline.rs:135` (1 inline `SELECT`) and `crates/cng/src/shape.rs:75,82,122,133,146,159` (6 inline `SELECT`s) still hold inline SPARQL strings, unguarded | `crates/cng/tests/no_inline_ttl_guard.rs`; `crates/cng/src/pipeline.rs:135`; `crates/cng/src/shape.rs:75,82,122,133,146,159` | PROJ-604 |
| 6 | Recursion tree at depth n produces 8ⁿ attachments | WRONG — corrected here | Attachment/edge count of the 8-ary tree at depth n is `(8^n − 8)/7`, not `8^n`. Depth 2 → 8 (matches the measured campaign, row 1, `RECURSIVE_ATTACHMENTS=8`). Depth 5 → 4,680, not 32,768. The unrelated `artifact_sets` cap (`crates/cng/src/main.rs:524`, cap 50,000) is a separate axis (flat set count) from recursion-tree depth; 8^5=32,768 fits under that cap as a set count but is not an attachment count | `crates/cng/src/bench.rs` `write_recursion_tree` (line 331), `derive_attachments` (line 1009); `crates/cng/src/main.rs:524` | — |
| 7 | Depth-5 (4,680-attachment) campaign has been run | UNVERIFIED at that scale | Only the depth-2/8-attachment campaign (row 1) was actually run this session; no depth-5 run is cited anywhere | none found this session | — |
| 8 | Bundle manifest names every input/output digest in one file | UNKNOWN, needs re-verification | `digests.json` is `{set_dir_path: powl_digest}` only; `ocel_graph_digest`/`sparql_result_digest` exist as separate `RunReport` fields but are never bundled into a single manifest artifact; no ontology-digest or rules-digest field exists anywhere in the codebase; an on-disk `results.json` sample from a prior run session was found missing `ocel_graph_digest`/`sparql_result_digest` — flagged UNKNOWN pending a fresh run, not confirmed as a bug | `crates/cng/src/bench.rs` `run()`/`verify()` digest handling | PROJ-603 |
| 9 | Datalog role-derivation layer uses the real graphlaw engine, not a mock | ALIVE, scoped | `praxis-graphlaw` pulled only behind the optional `bench` feature, a deliberate, self-documented registry-only-deps exception (`crates/cng/Cargo.toml:35-50`); typed refusal on role/roster disagreement (`bench.rs` ~1372-1390) | `crates/cng/Cargo.toml:35-50`; `crates/cng/src/bench.rs` ~1372-1390; `just cng-test` (all suites pass, see row 10) | — |
| 10 | Hierarchical workflow manufacture is real, not flattened-only fakery | PARTIAL, disclosed by design | Hierarchy is real in the serialized POWL artifact (`pipeline::hierarchical_projection`); execution is a lawful flattening onto published bcinr-powl 26.6.25 via `runner::validate_run_hierarchical`, not native nested `PowlAstNode` composition — this is the project's own stated boundary, not a newly found gap | `crates/cng/BENCHMARK.md` lines 47-50; `crates/cng/src/pipeline.rs:244`; `crates/cng/src/runner.rs:448` | — |
| 11 | Post-quantum-crypto (PQC) signing binding exists or is planned near-term | REFUSED for this release | Zero design precedent anywhere in the repo (grepped this session for pqc/dilithium/kyber/ml-dsa/ml-kem: every hit is either a typo-dictionary entry, unrelated Cargo.lock crate names, or `docs/forensics/` teardown notes on a rejected, flagged-fake external Dilithium implementation). Classical ed25519 signing DOES exist and is real (`crates/praxis-core/src/signing.rs`, `sign_chain_hash`/`verify_chain_hash`, `Receipt.signature: Option<Vec<u8>>` behind the `signed` feature) but `cng` itself has zero signature wiring — this is the actual near-term seam if signing is wanted later | `crates/praxis-core/src/signing.rs`; repo-wide grep, this session, no PQC hits | — |
| 12 | Test suites for the benchmark/pipeline surface pass | ALIVE | `just cng-test`: powl 10, cng_hierarchical 1, cng_cli_smoke 1, cng_negative_fixtures 5, cng_pipeline 4, no_inline_ttl_guard 2 — all pass; `just test-bin chatman_pddl_to_powl_joseph_famine_hierarchical`: 3 passed; `just cng-bench-build` exit 0 | this session's run output | — |

## 1. Product summary

`cng` (crates/cng, v26.9.10) is a noun-verb CLI whose `bench` feature (`crates/cng/src/bench.rs`)
manufactures a Recursive Workflow standing benchmark: an 8-ary tree of admitted PDDL/POWL
artifact sets, executed on the real bcinr-powl scheduler, with every headline number read back
out of an OCEL 2.0 evidence graph rather than asserted from Rust counters. At commit `e763f44`
a 10,000-worker, depth-2 campaign ran end-to-end, measured (row 1 above), byte-identical across
two runs, and independently sample-replayed (row 2).

## 2. Narrative frame

This is a standing benchmark, not a demo: every headline number (`WORKERS_REPRESENTED`,
`WORKFLOW_INSTANCES`, `EXECUTED_TRANSITIONS`, `RECEIPTS_GENERATED`, `REFUSED_TRANSITIONS`,
`RECURSIVE_ATTACHMENTS`, `DATALOG_DERIVED_ROLES`, `OCEL_GRAPH_DIGEST`) is a SPARQL SELECT result
over a CONSTRUCT-manufactured evidence graph, and a telemetry/graph mismatch is a typed refusal
(`CngRefusal::HardcodingSuspicion`, `crates/cng/src/bench.rs` ~1860-1887), not a warning. `cng`
is a concrete instance of `A = μ(O*)`, `R = receipt(A)` (`docs/CHATMAN_EQUATION.md:4`): admitted
observation facts (`O*`) are lawfully manufactured (`μ`) into workflow artifacts (`A`), and the
benchmark's receipts (`R`) prove that consequence — this document introduces the `G_OCEL =
CONSTRUCT_P(G_OBS)` formalism for the specific manufacture step (see `ARD.md` Sec. 3, originated
in this document).

## 3. Customer problem

Fortune-5-scale workflow claims need a benchmark that cannot silently pass with canned or
hardcoded output. The reconcile gate (`bench.rs` ~1860-1887, message: "the SPARQL evidence graph
is the authority") refuses the whole run if telemetry counters disagree with the graph-derived
numbers — this is the mechanism that lets row 1's headline numbers be trusted as measured, not
asserted.

## 4. Product position

**A measured 10,000-worker/depth-2 campaign plus a set of PLANNED hardening tickets — explicitly
not more.** No depth-5/4,680-attachment campaign has been run (row 7); no PQC signing exists or
is planned this release (row 11); `verify` does not re-check OCEL/SPARQL digests (row 4); the
bundle is not portable across machines/directories without a fix (row 3).

## 5. Core equation

```
G_OCEL = CONSTRUCT_P(G_OBS)
```

Introduced in this document (see `ARD.md` Sec. 3 for the full formalism and origination note).
`G_OBS` is the disk-template-fed observation store; `P` is the fixed set of `crates/cng/queries/
*.construct.rq` CONSTRUCT queries (generated by `packs/ocel-bench-pack` from
`crates/praxis-graphlaw/ontologies/core/ocel2.ttl`); `G_OCEL` is the resulting OCEL evidence
graph, whose sorted N-Triples serialization is `ocel_graph_digest`. Every `RunReport` headline
field is then a SELECT over `G_OCEL`, never an assertion.

## 6. Doctrine

Evidence graph is the authority, Rust counters are telemetry only (`bench.rs` reconcile gate,
row 3 of the Product summary evidence). Typed refusal on any disagreement
(`CngRefusal::HardcodingSuspicion`), never a silent pass. Registry-only dependencies by default;
non-registry exceptions (`praxis-graphlaw` for `bench`, `chicago-tdd-tools` dev-dep) are
deliberately self-documented in `crates/cng/Cargo.toml:35-50,80-85`, not hidden.

## 7. Primary release goal

Establish the e763f44 10,000-worker/depth-2 campaign as the ALIVE evidentiary floor for this
release, and stage five hardening tickets (PROJ-601..505, `docs/jira/v26.7.10/tickets/`) as
PLANNED work closing the gaps found this session (auditor-replay portability, verify-digest
completeness, bundle-manifest schema, inline-SPARQL closure, a new `CNG_R11 AuditMismatch`
refusal). None of PROJ-601..505 is implemented in this release.

## 8. MVP definition

The MVP for this release is the reconciled baseline itself, not new code:

1. Baseline campaign measured and byte-identical across 2 runs (row 1).
2. Sample replay verified via `cng benchmark verify` (row 2).
3. All ten corrections (Claims Reconciliation rows above) captured accurately, not softened.
4. Five PLANNED ticket stubs staged under `docs/jira/v26.7.10/tickets/` for the next increment.

## 9. Personas

- **Benchmark operator.** Runs `cng benchmark generate`/`run`/`verify` and needs to know exactly
  what `verify` does and does not check (row 4) before trusting a "verified" claim.
- **Third-party auditor.** Currently blocked from relocating a bundle directory and replaying it
  (row 3) — PROJ-601 is the fix.
- **AI agent / Datalog role consumer.** Depends on `crates/cng/rules/bench-roles.dl` via the real
  `praxis-graphlaw` engine (row 9), with a typed refusal on any role/roster mismatch.
- **Enterprise buyer.** Must be told, not left to discover, that the depth-5/4,680-attachment
  claim is UNVERIFIED (row 7) and that the 8ⁿ arithmetic used in prior circulated drafts was
  wrong (row 6).

## 10. Functional requirements

| # | Requirement | Evidence surface |
|---|---|---|
| F1 | Deterministic corpus generation | `cng benchmark generate --out DIR --workers N --depth D` |
| F2 | Measured execution with graph-authoritative headline numbers | `cng benchmark run --dir DIR`; `bench.rs` reconcile gate |
| F3 | Independent sample replay + export re-validation | `cng benchmark verify --dir DIR`; `bench.rs` `verify()` |
| F4 | Datalog role derivation with typed mismatch refusal | `crates/cng/rules/bench-roles.dl`; `bench.rs` ~1372-1390 |
| F5 | Hierarchical manufacture via lawful flattening | `pipeline::hierarchical_projection`; `runner::validate_run_hierarchical` |
| F6 | Recursive attachment tree, parent-IRI-preserving | `bench.rs` `write_recursion_tree` (line 331), `derive_attachments` (line 1009) |

## 11. Non-functional requirements

1. **Determinism.** Headline numbers and `OCEL_GRAPH_DIGEST` byte-identical across 2 consecutive
   runs of the same campaign (row 1).
2. **No inline Turtle/PDDL in `bench.rs`.** Enforced by
   `crates/cng/tests/no_inline_ttl_guard.rs` (2 tests pass, `just cng-test`).
3. **Registry-only deps by default.** `cargo publish --dry-run` unaffected with `bench`/
   `otel-live` off (`Cargo.toml:35-50,80-85`).
4. **Correct recursion arithmetic.** `(8^n − 8)/7` attachments at depth n, not `8^n` — see row 6
   and Sec. 12 below; this must not recur as a future claim.

## 12. Out of scope (non-goals)

1. **PQC signing** — explicitly REFUSED this release (row 11); no design precedent exists.
2. **Full depth-5/4,680-attachment campaign** — UNVERIFIED at that scale; only depth-2/8-
   attachment was actually run. Restated arithmetic: attachment count at depth n of the 8-ary
   tree is `(8^n − 8)/7`, NOT `8^n`. Depth 5 → 4,680, not 32,768. The `artifact_sets` cap
   (`main.rs:524`, 50,000) is a separate flat-set-count axis, unrelated to tree-depth attachment
   counts — 8^5=32,768 fits under that cap as a set count but is not an attachment total. Any
   future claim conflating these two axes is wrong and must be corrected, not softened.
3. **Workflow sockets** beyond the existing `attachesWorkflow` parent-child mechanism — future
   increment per `crates/cng/BENCHMARK.md`.
4. **Bounded-question/resume loop** — future increment, not in this release.
5. Any verify-time re-derivation of `ocel_graph_digest`/`sparql_result_digest` (row 4) — deferred
   to PROJ-602.

## 13. Day-one finish plan

1. Confirm `docs/releases/v26.7.10/{PRD.md,ARD.md,RELEASE_CONTROL.md}` and
   `docs/jira/v26.7.10/tickets/{PROJ-601..505}.md` are all present and consistent (this pass).
2. File PROJ-601..505 as tracked tickets when the next increment begins implementation.
3. Re-run the baseline campaign fresh before any future claim upgrades a PARTIAL/UNKNOWN row
   above to ALIVE.

## 14. Acceptance criteria

| # | Criterion | Proof required | Status |
|---|---|---|---|
| 1 | Baseline campaign measured | `cng benchmark run` twice, byte-identical headline numbers + `OCEL_GRAPH_DIGEST` | PASS (row 1) |
| 2 | Sample replay | `cng benchmark verify` exit 0, `replayed:3, replay_passes:3` | PASS (row 2) |
| 3 | Test suites green | `just cng-test` all pass; `just test-bin chatman_pddl_to_powl_joseph_famine_hierarchical` 3 passed | PASS (row 12) |
| 4 | All 10 corrections captured without softening | Claims Reconciliation table rows 2-11 | PASS (this document) |
| 5 | v26.7.10 scope items staged as PLANNED tickets | `docs/jira/v26.7.10/tickets/PROJ-601..505.md` | PASS (stubs created) |
| 6 | No overclaim of depth-5 scale or PQC | Sec. 12 non-goals | PASS (explicit) |
