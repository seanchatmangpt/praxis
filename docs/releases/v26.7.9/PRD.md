# Chatman Engine v26.7.9 — Product Requirements Document

Status: DRAFT, tied to `RELEASE_CONTROL.md` (single control surface). Every claim in this
document cites a file, test, or receipt in this repository. Rows without evidence are marked
PLANNED or UNKNOWN, never asserted. This release additionally reconciles every press-release
marketing claim against the repo's no-overclaiming vocabulary (`.claude/rules/
no-overclaiming.md`) — see `## Claims Reconciliation` below, embedded verbatim in `ARD.md`.
Gate F verdict (`docs/chatman-engine/chicago_tdd_final_report.md`, `ADMITTED_DRY_RUN_PUBLISHABLE`)
is the evidentiary floor for the S1-S6 pipeline; nothing below claims beyond that floor without
saying so.

## Claims Reconciliation

Every marketing claim in the Chatman Engine v26.7.9 press release is reconciled below against
this repository's evidentiary vocabulary (`.claude/rules/no-overclaiming.md`). This table is
the single source of truth for claim status; narrative sections elsewhere in this document must
not assert a status stronger than the row below. Status vocabulary: **ALIVE** (verified,
executes, cited test/receipt passes), **PARTIAL** (real but narrower than the claim — gap named
explicitly), **PLANNED** (roadmap/ticket only, no code path), **UNKNOWN** (not yet investigated
to a verdict), **MOCKED** (a stand-in exists where the claim implies the real thing).

| # | Claim | Status | Scope / caveat | Evidence | Ticket |
|---|---|---|---|---|---|
| 1 | RDF/Turtle as source of truth; downstream artifacts "manufactured" | ALIVE | S1-S6 pipeline only | `crates/praxis-graphlaw/src/chatman/engine.rs` (S1-S6); `docs/chatman-engine/chicago_tdd_final_report.md` (Gate F verdict) | — |
| 2 | Direct projection to PDDL v3.1 planning models | PARTIAL | Mechanism ALIVE via `bcinr_pddl`/`crates/pddl-index`; literal PDDL v3.1 spec-version conformance not confirmed by any search this session | `engine.rs` S3 `generate_pddl_plan`; `crates/pddl-index` (tested grounder) | — |
| 3 | Direct projection to POWL v2 workflow models | PARTIAL | Mechanism ALIVE via `bcinr_powl`; `crates/powl2-decompose` cites Kourani et al. Defs 3.6-3.9 formally, but no full POWL 2.0 spec-conformance suite found | `engine.rs` S4 `admit_powl_trace`; `crates/powl2-decompose/src/powl.rs` | — |
| 4 | RDF-native process evidence, exportable to OCEL formats | PARTIAL | Core path (OCEL validation inside S4) ALIVE and tested; `crates/ocel/ocel_gap_report.md` ("No gaps found. All systems functional.") is a content-free one-liner and is NOT usable as evidence | `engine.rs` S4; `crates/praxis-core/src/ocel.rs` (tested) | — |
| 5 | Generated artifacts from graph bindings (e.g. demand letters) | UNKNOWN | No agent found this capability confirmed anywhere in `chatman/` or elsewhere; not asserted, not denied | searched `crates/praxis-graphlaw/src/chatman/`, repo-wide — no hits | UNTRACKED |
| 6 | Typed refusals for invalid/unsupported branches | ALIVE | 29 named, doc-commented variants | `abi.rs` `pub enum Refusal` (29 variants) | — |
| 7 | Receipt envelopes with ordered digest material | ALIVE | 9 digests, constitutional order, BLAKE3 root | `engine.rs` `EngineProcessReceipt` (alias `ProcessReceiptEnvelope`) | — |
| 8 | Deterministic replay | PARTIAL | Core engine `verify_replay`/`ReplayMismatch` ALIVE; WASM-surfaced `Status::HashMismatch` for full pipeline re-run not yet wired | `engine.rs` `verify_replay`; PROJ-417 (open) | PROJ-417 |
| 9 | "No unreceipted actuation" | ALIVE, scoped to S1-S6 | `trigger_knowledge_hooks` (S5) produces sealed `BoundaryRequest` before `generate_receipt` (S6); scope excludes deferred S3→S4 wiring | `engine.rs` S5/S6; Gate F Gate B | — |
| 10 | Refusal on missing evidence / unreachable goal / unlawful path / hash mismatch / tamper / unreceipted actuation / changed replay material | PARTIAL | Each maps to a named `Refusal` variant (ALIVE as typed mechanism); "projection hash mismatch" and "changed replay material" intersect the PROJ-417 gap on the WASM surface | `abi.rs` `Refusal`; PROJ-417 | PROJ-417 |
| 11 | "Dry-run publishable release" framing (Availability section) | ALIVE, exclusions required | Literal Gate F verdict string; accurate only if the six exclusions (Sec. 12 below) are carried forward with it | `docs/chatman-engine/chicago_tdd_final_report.md` | — |

## 1. Product summary

Chatman Engine is a governed S1→S6 pipeline inside `crates/praxis-graphlaw/src/chatman/`:
`fetch_snapshot` → `apply_owl_closure` → `generate_pddl_plan` → `admit_powl_trace` →
`trigger_knowledge_hooks` → `generate_receipt`. Admitted RDF/Turtle is the source; PDDL plan
tapes, POWL trace admission, hook actuation records, and a 9-digest `EngineProcessReceipt` are
manufactured from it, never asserted directly. Independent Gate F audit
(`docs/chatman-engine/chicago_tdd_final_report.md`) verdict: `ADMITTED_DRY_RUN_PUBLISHABLE`,
scoped to this S1-S6 surface — not "production-ready," not unscoped.

## 2. Narrative frame

Chatman Engine is the governance layer on top of admission: it adds sealed, receipted,
per-field-replayable transitions (`AdmittedTransition`, `EngineProcessReceipt`) where prior work
had admission without a receipted transition chain. It does not claim autonomy or
self-governance beyond what the 29 typed `Refusal` boundaries (`abi.rs`) actually enforce.

## 3. Customer problem

Governed systems need transitions that are replayable and digest-verifiable, not merely logged.
Chatman Engine's answer is a typed refusal boundary (29 `Refusal` variants, `abi.rs`) plus a
per-field replay check (`verify_replay`, `ReplayMismatch` enum, `engine.rs`) rather than a
generic error string or silent pass-through.

## 4. Product position

**S1-S6 admission/planning/receipt core — explicitly not more.** Six things are out of scope
for this closure and must be disclosed before any confidence-building narrative: N3 cubic
scaling, S3→S4 `OrchestratedPlan`/`TapeBridge` wiring, PROJ-415 (SHACL `CompiledShape`),
PROJ-416 (Pattern-4 canonical receipts), PROJ-417 (WASM `HashMismatch` replay surfacing), and
crate-wide non-Chatman clippy debt. See Sec. 12.

## 5. Core equation

```
R = generate_receipt(admit_powl_trace(generate_pddl_plan(apply_owl_closure(fetch_snapshot(S)))))
```

Receipt `R` is the image of admitted snapshot `S` under the sealed S1→S6 chain
(`crates/praxis-graphlaw/src/chatman/engine.rs`); any stage refuses forward with a typed
`Refusal` variant rather than degrading (`abi.rs`).

## 6. Doctrine

Typed-refusal completeness plus computed-never-asserted receipts. Zero `.unwrap()`/`.expect(`/
`panic!(`/`unsafe`/`SystemTime`/`Instant::now`/`.ok()`-swallowing hits inside
`crates/praxis-graphlaw/src/chatman/` per Gate E's static-token scan
(`docs/chatman-engine/chicago_tdd_final_report.md`).

## 7. Primary release goal

Achieve Gate F `ADMITTED_DRY_RUN_PUBLISHABLE` for the S1-S6 core — already achieved, per report
audited against commit `7d76019` — with the six exclusions as explicit follow-on goals (tracked
as PROJ-415/416/417 and untracked items), and reconcile the PROJ-411..414 ticket-status/Gate-F
disposition mismatch (tickets still say "IN PROGRESS"; Gate F says PASS) —
`docs/jira/v26.7.8/tickets/PROJ-411-417-reconciliation.md`.

## 8. MVP definition

The MVP is the S1-S6 core, gated as follows (Gate C folded into Gate B per the source report):

1. **Gate A — Substrate.** PASS. Toolchain pin `nightly-2026-06-22`, single `oxrdf`/`oxigraph`
   version, `rdf-12` feature on, `TripleTermInSnapshot` enforced at text boundary (rdf-12 not
   enabled workspace-wide).
2. **Gate B — Code.** PASS. Six chatman source files present (`abi`, `triple8`, `admission8`,
   `router`, `engine`, `bridge`); `Refusal` enum has 29 variants; `AdmittedTransition` sealed/
   read-only; `EngineProcessReceipt` carries 9 digests + `receipt_root`; `verify_replay` checks
   each field independently; crate compiles. 123 tests passed (5 skipped, documented fail-loud
   ignores, not vacuous passes), 11 static gates passed.
3. **Gate D — Evidence.** PASS. 8 OCEL suites regenerated twice with byte-identical receipt
   hashes; 5 consecutive e2e runs byte-identical on `receipt_root`. Disclosed non-gating
   finding: raw `.ocel.json` bodies are NOT byte-deterministic — only the sealed receipt digest
   is, which is what the DoD actually gates.
4. **Gate E — Quality (Chatman-scoped).** PASS, with mutation score and line coverage explicitly
   marked **UNVERIFIED/advisory** — not claimed as passing. `cargo fmt` clean, zero clippy
   findings inside `src/chatman`/`tests/chatman_*` (crate-wide clippy failures are pre-existing
   debt outside the Chatman surface), zero forbidden tokens, no duplicate canonical types.

## 9. Personas

- **Founder-operator.** Needs the Gate F report as a single audit artifact rather than
  re-deriving standing by hand (`docs/chatman-engine/chicago_tdd_final_report.md`).
- **AI agent.** Consumes `ChatmanEngine::{in_memory, open, load_snapshot, admit_transition,
  actuate}` (`engine.rs`) as the typed API surface; no dedicated CLI verb was found this session
  — CLI access is PLANNED/UNKNOWN (see `ARD.md` Sec. 8).
- **Adversarial reviewer.** Served by the Gate F process itself: an independent, non-authoring
  audit session that re-derived every command against a clean `git status` precondition.
- **Enterprise buyer.** Must be told, not left to discover, that the compiled
  `standing.json`/`REALITY_INDEX.md` still shows the relevant crate at ladder `0`,
  `DISCOVERED` — a schema gap, not a refutation, but disclosed either way (Sec. 3 of
  `RELEASE_CONTROL.md`).

## 10. Functional requirements

| # | Requirement | Evidence surface |
|---|---|---|
| F1 | S1 fetch_snapshot — RDFC-1.0 canonicalize, hash canonical N-Quads | `engine.rs` S1 |
| F2 | S2 apply_owl_closure — route via `DialectRouter`, materialize into sibling `<snapshot#closure>` graph, input never mutated | `engine.rs` S2; `router.rs` |
| F3 | S3 generate_pddl_plan — `bcinr_pddl` → `Pddl8Tape` | `engine.rs` S3; `crates/pddl-index` (own, tested, separate pipeline from `docs/PDDL_INTEGRATION_SUMMARY.md`'s "schema only" RDF→PDDL sketch — do not conflate) |
| F4 | S4 admit_powl_trace — OCEL validation (`wasm4pm_compat`) + tape conformance (`bcinr_powl`) + causal-frame chaining (`bcinr_powl_receipt`); violations → `Refusal::TraceUnlawful` | `engine.rs` S4 |
| F5 | S5 trigger_knowledge_hooks — sealed `BoundaryRequest`, not constructible outside module | `engine.rs` S5 |
| F6 | S6 generate_receipt — 9 digests in constitutional order + BLAKE3 root; S1 hash re-verified before sealing (TOCTOU guard) | `engine.rs` S6 |
| F7 | Typed refusal taxonomy | `abi.rs` `Refusal` (29 variants) |
| F8 | Per-field replay verification | `engine.rs` `verify_replay`, `ReplayMismatch` |

## 11. Non-functional requirements

1. **Determinism.** `receipt_root` byte-identical across 5 consecutive runs and across 2 clean
   OCEL-suite regenerations; raw `.ocel.json` bodies are not byte-deterministic — only the
   sealed digest is (Gate D).
2. **Typed refusal completeness.** 29 `Refusal` variants, zero forbidden-token hits (`abi.rs`,
   Gate E).
3. **Sealed construction.** `AdmittedTransition` and `BoundaryRequest` have private fields, only
   constructible via `ChatmanEngine::admit_transition` (Gate B).
4. **Computed evidence.** `EngineProcessReceipt` BLAKE3, 9 digests, never asserted-in.
5. **No wall clock in receipt paths.** Zero `SystemTime`/`Instant::now` hits (Gate E).

## 12. Out of scope

1. N3 cubic-scaling work — untouched by this release.
2. Deferred S3→S4 `OrchestratedPlan`/`TapeBridge` engine-side wiring (`bridge.rs` exists, full
   wiring not complete).
3. PROJ-415 — SHACL `CompiledShape` population.
4. PROJ-416 — Pattern-4 canonical renders not wired to receipt hashing.
5. PROJ-417 — WASM `Status::HashMismatch` replay surfacing.
6. Crate-wide non-Chatman clippy debt.
7. Mutation testing and line coverage (Gate E items 4-5, explicitly advisory/UNVERIFIED).
8. Standing-index milestone representation (schema gap, no `MilestoneArtifact` kind — see
   `ARD.md` Sec. 5).

## 13. Day-one finish plan

1. Sync PROJ-411..414 in-file ticket status headers from "IN PROGRESS" to reflect the Gate F
   PASS disposition (`docs/jira/v26.7.8/tickets/PROJ-411-417-reconciliation.md`).
2. Confirm PROJ-415/416/417 remain OPEN and unaffected by this closure.
3. Decide whether to extend the `cicd-standing.v1` schema with a `MilestoneArtifact` kind so
   Gate F verdicts can be represented in `REALITY_INDEX.md` — PLANNED, explicitly out of scope
   for this pass per `docs/standing/STANDING_SCHEMA_MILESTONE_GAP.md`.
4. If mutation/coverage tooling is wanted to upgrade Gate E items 4-5 from UNVERIFIED, run it
   and record output here.

## 14. Acceptance criteria

Reproduces Gate A/B/D/E as the acceptance table; each row's status is the literal Gate F
disposition, not a paraphrase.

| # | Criterion | Proof required | Status |
|---|---|---|---|
| 1 | Gate A — substrate | toolchain pin, single oxrdf/oxigraph version, rdf-12 on, TripleTermInSnapshot enforced | PASS |
| 2 | Gate B — code | 6 files present, 29 Refusal variants, sealed AdmittedTransition, 9-digest receipt, per-field verify_replay, cargo check clean | PASS |
| 3 | Gate C (folded into B) — tests | 123 passed / 5 skipped (documented), 11 static gates, diagram atlas, hotpath tick gate | PASS |
| 4 | Gate D — evidence | sync-verify exit 0, 8 OCEL suites byte-identical across 2 regenerations, 5x determinism run byte-identical `receipt_root` | PASS |
| 5 | Gate E — quality | fmt clean, zero clippy in chatman surface, zero forbidden tokens, no duplicate canonical types | PASS; mutation/coverage UNVERIFIED |
| 6 | Six named exclusions | tracked as EXCLUDED, not FAILED | EXCLUDED (Sec. 12) |
| 7 | Standing index reflects milestone | `MilestoneArtifact` kind in `cicd-standing.v1` schema | PLANNED/GAP — does not invalidate rows 1-5 |

Verdict: `ADMITTED_DRY_RUN_PUBLISHABLE`, scoped exactly to the S1-S6 surface with the exclusions
in Sec. 12 (`docs/chatman-engine/chicago_tdd_final_report.md`).
