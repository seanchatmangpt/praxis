# DOD_EVIDENCE_MAP — Clause-by-Clause Evidence Map for v26.7.10

Version: v26.7.10. Consumed by PROJ-617 (closure) and PROJ-622 (SPARQL success markers).
Status: DRAFT, tied to `RELEASE_CONTROL.md`. Every claim cites a file, test, or receipt. Rows
without evidence are marked PLANNED or UNKNOWN, never asserted. If this file and
`RELEASE_CONTROL.md` disagree, `RELEASE_CONTROL.md` wins. This file does not upgrade any
marker's verdict on its own. Final marker verdicts live in `RELEASE_CONTROL.md` §8 and
`DOD_SIGNOFF.md` (PROJ-617): per the consolidated final build cited there, all 11 success
markers derived TRUE via SPARQL. PLANNED rows below are the pre-build planning record, kept
for traceability; where they disagree with `DOD_SIGNOFF.md`, the sign-off wins.

## How to read this map

- **Query** — the SPARQL file under `crates/cng/queries/` expected to prove the marker over the
  emitted OCEL graph. `(exists)` = file present on disk this session (`ls` verified).
  `(PLANNED)` = named by `DEFINITION_OF_DONE.md` or ticket scope but not yet real evidence.
- **Refusal** — the `CngRefusal` variant guarding the clause. Variant doc comments and stable
  codes for `CNG_R12`..`CNG_R18` are present in `crates/cng/src/powl.rs:69-172` (read this
  session), plus `CNG_R19 EvidenceGateFailed` (`powl.rs:146-158`, graph-derived closure gate
  unclosed, PROJ-614) and `CNG_R20 MarkerFalse` (`powl.rs:159-171`, false success marker,
  PROJ-622). R12..R20 all exercised by `just cng-test-bench` this session (40 lib tests
  green, incl. `unreceipted_actuation_gate_refuses_cng_r19` and
  `forced_false_marker_refuses_cng_r20`); final verdicts live in `DOD_SIGNOFF.md`.
- **Test/recipe** — the `just` recipe or test binary expected to exercise the marker.
- **Status** — no-overclaiming vocabulary; nothing exceeds `RELEASE_CONTROL.md`.

Note on naming (resolved at PROJ-617 closure): the `DEFINITION_OF_DONE.md` §4 names are now
authoritative and on disk — `metric-hook-actuations.rq` and `metric-dispatch-closure.rq`
(`ls` verified). The old `metric-hook-receipts.rq` was folded and deleted under PROJ-614;
`dispatch-closure.rq` remains as a separate broader query. This map uses the new names.

## Success-marker evidence map

| Marker | Query | Refusal | Test/recipe | Status |
|---|---|---|---|---|
| `HOOK_ACTUATION_PROVEN` | `metric-hook-actuations.rq` (exists) | `CNG_R13 UnreceiptedActuation` (code present, untested) | `just cng-workday` (PLANNED, PROJ-608); `just cng-test-bench` | PLANNED (PROJ-612) |
| `ZERO_UNRECEIPTED_ACTUATION` | `metric-hook-actuations.rq` (exists; zero-gap check vs `metric-transitions.rq`) | `CNG_R13 UnreceiptedActuation` | tamper negatives (PLANNED, PROJ-616) | PLANNED (PROJ-614) |
| `EXTERNAL_WORKFLOW_DISPATCH_PROVEN` | `metric-dispatch-closure.rq`, `dispatch-closure.rq` (exist); `ocel-dispatches.construct.rq` (exists) | `CNG_R15 DispatchContractIncomplete`, `CNG_R16 DispatchStateUnlawful` | loopback outbox/inbox run under `just cng-workday` (PLANNED) | PLANNED (PROJ-618/619; loopback-real, no live network) |
| `EXTERNAL_RESULT_READMISSION_PROVEN` | `ocel-admissions.construct.rq` (exists) + admission SELECT (PLANNED, PROJ-614) | `CNG_R17 ExternalConsequenceRefused` | admission-refusal negative tests (PLANNED, PROJ-616) | PLANNED (PROJ-619) |
| `RECURSIVE_CHILD_CLOSURE_PROVEN` | `attachments-with-parent.rq` (exists); `metric-recursive-attachments.rq` (exists) | `CNG_R16 DispatchStateUnlawful` (unlawful parent close) | closure-law scenarios (PLANNED, PROJ-620) | PLANNED (attachment substrate ALIVE at depth 2: `RECURSIVE_ATTACHMENTS=8`, `RELEASE_CONTROL.md` §1) |
| `TIMEOUT_ESCALATION_PROVEN` | timeout-escalation SELECT (PLANNED, PROJ-614) | `CNG_R16 DispatchStateUnlawful` (`TIMED_OUT` path) | deterministic-tick timeout scenario (PLANNED, PROJ-620) | PLANNED |
| `COMPENSATION_WORKFLOW_PROVEN` | compensation SELECT over OCEL (PLANNED, PROJ-614) | `CNG_R17 ExternalConsequenceRefused` triggering compensation | compensation scenario (PLANNED, PROJ-620) | PLANNED |
| `AUTONOMIC_LOOP_CLOSED` | `standing-next-action.rq` (exists) + `metric-transitions.rq` (exists) | `CNG_R12 StandingAmbiguous` (code present, untested) | full `just cng-workday` loop run (PLANNED, PROJ-608) | PLANNED (manufacture→receipt→replay arc ALIVE per `RELEASE_CONTROL.md` §1/§7) |
| `ONE_PERSON_RECURSIVE_WORKFLOW_PROVEN` | conjunction of all workday SELECTs (PROJ-622) | all of R12..R17 | `just cng-workday` end-to-end (PLANNED, PROJ-608) | PLANNED |
| `GRAPHLAW_DIALECT_CLOSURE` | `registry-closed-violations.rq`, `registry-missing-fields.rq` (both exist) | `CNG_R14 DialectRegistryRefused`; `CNG_R18 ArazzoProfileRefused` | registry negative fixtures (PLANNED, PROJ-613/621) | PLANNED |
| `V26_7_10_PRODUCTION_READY` | conjunction of every row above (PROJ-622 marker query set) | n/a (derived) | PROJ-617 closure checklist | UNVERIFIED — conjunction; may not be claimed until every row is ALIVE |

## Baseline evidence already ALIVE (floor, not markers)

These rows are the `RELEASE_CONTROL.md` §1/§7 floor the markers build on. They are ALIVE for
the baseline benchmark only, not for any v26.7.10 marker.

| Clause | Query | Test/recipe | Status |
|---|---|---|---|
| OCEL authority, receipts | `metric-receipts.rq` (exists) | `just cng-bench-verify` — `replay_passes:3` | ALIVE (`RELEASE_CONTROL.md` §7 items 4-8) |
| Replay | `metric-replay.rq` (exists; made real under PROJ-614) | `just cng-evidence-replay` — `AUDIT_RESULT=CONFORMANT` | ALIVE for evidence replay (§7 item 9); metric query realness PLANNED (PROJ-614) |
| Datalog roles | `metric-derived-roles.rq` (exists) | benchmark run — `DATALOG_DERIVED_ROLES=10000` | ALIVE (§1) |
| Conformance/refusals | `metric-conformance.rq`, `metric-refusals.rq` (exist) | benchmark run — `REFUSED_TRANSITIONS=1` | ALIVE (§1) |
| Audit-integrity refusal | n/a | tamper test — exit 1, `CNG_R11 AuditMismatch` (§7 item 10) | ALIVE |

## Behavioral "operator never..." clauses → mechanisms

`DEFINITION_OF_DONE.md` §2, clauses 1-12. All PLANNED as behaviors (PROJ-608..622); mechanism
column names the intended enforcement, not achieved state.

| # | Operator never... | Mechanism | Status |
|---|---|---|---|
| 1 | Wonders what to do next | `standing-next-action.rq` (exists) + `CNG_R12 StandingAmbiguous` | PLANNED (PROJ-610) |
| 2 | Manually connects plans to tasks | graph-derived `attachesWorkflow` attachments (`attachments-with-parent.rq`) | PLANNED as behavior; attachment substrate ALIVE (§1) |
| 3 | Reconstructs provenance | OCEL CONSTRUCT chain (`ocel-*.construct.rq`) + receipts | PLANNED as behavior; receipt/replay floor ALIVE (§7) |
| 4 | Redraws workflows | POWL manufacture from PDDL (deterministic re-manufacture) | PLANNED as behavior; manufacture digest ALIVE (§7) |
| 5 | Determines ownership | Datalog role derivation (`metric-derived-roles.rq`) | PLANNED as behavior; role derivation ALIVE (§1) |
| 6 | Routes evidence | broker exclusivity + admission pipeline (`ocel-admissions.construct.rq`) | PLANNED (PROJ-619) |
| 7 | Reconciles completed work | parent-child closure laws + `metric-dispatch-closure.rq` | PLANNED (PROJ-620) |
| 8 | Loses replay | evidence bundle + `just cng-evidence-replay` + `CNG_R11` | ALIVE for baseline bundle (§7); workday-mode PLANNED |
| 9 | Performs semantic glue | SPARQL CONSTRUCT materialization, no inline SPARQL (guard test) | PARTIAL — guard ALIVE (§7 item 1); workday-mode PLANNED |
| 10 | Remembers open loops | dispatch state machine, no implicit completion (`CNG_R16`) | PLANNED (PROJ-618) |
| 11 | Polls without declared workflow | polling as registered workflow activity, bounded, receipted | PLANNED (PROJ-618/619) |
| 12 | Decides which compensation follows | declared compensation law in graph, evaluated via dialect | PLANNED (PROJ-620) |

Bounded admission ("resume loop") is the admission-resume mechanism of PROJ-611 — PLANNED;
`RELEASE_CONTROL.md` §2 item 4 records the bounded-question/resume loop as a future increment.

## Refusal-variant coverage ledger

| Code | Variant | Guards | Test evidence this session |
|---|---|---|---|
| `CNG_R11` | `AuditMismatch` | evidence-bundle integrity | ALIVE — tamper test, exit 1 (§7 item 10); `powl_test.rs:62-64` |
| `CNG_R12` | `StandingAmbiguous` | exactly-one next action | code present (`powl.rs:69`); PLANNED, no test cited |
| `CNG_R13` | `UnreceiptedActuation` | zero unreceipted actuation | code present (`powl.rs:81`); PLANNED, no test cited |
| `CNG_R14` | `DialectRegistryRefused` | registry closed-shape law | code present (`powl.rs:92`); PLANNED, no test cited |
| `CNG_R15` | `DispatchContractIncomplete` | 20-field dispatch contract | code present (`powl.rs:103`); PLANNED, no test cited |
| `CNG_R16` | `DispatchStateUnlawful` | 13-state dispatch machine | code present (`powl.rs:113`); PLANNED, no test cited |
| `CNG_R17` | `ExternalConsequenceRefused` | admission pipeline | code present (`powl.rs:126`); PLANNED, no test cited |
| `CNG_R18` | `ArazzoProfileRefused` | Arazzo bounded profile | code present (`powl.rs:138`); PLANNED, no test cited |

Per repo law, every variant needs ≥ 1 end-to-end negative test before its row can go ALIVE
(PROJ-616 harness; see `.claude/rules/rust-agi-core-team.md` §5).

## See Also

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface; wins on disagreement
- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` — the doctrine this map indexes (PROJ-606)
- `docs/releases/v26.7.10/PRD.md` — authoritative Claims Reconciliation table
- `docs/releases/v26.7.10/ARD.md` — architecture requirements
- `crates/cng/queries/` — on-disk SPARQL evidence sources named above
- `.claude/rules/no-overclaiming.md` — status vocabulary used throughout
