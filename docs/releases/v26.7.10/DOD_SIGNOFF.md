# DOD_SIGNOFF — Clause-by-Clause DoD Sign-Off for v26.7.10 (PROJ-617)

Status: DRAFT, tied to `RELEASE_CONTROL.md`. Every claim cites a file, test, or receipt. Rows
without evidence are marked PLANNED or UNKNOWN, never asserted. If this file and
`RELEASE_CONTROL.md` disagree, `RELEASE_CONTROL.md` wins.

Scope of "ALIVE" here: verified this closing session, meaning either (a) the PROJ-601..605
verification ladder (`RELEASE_CONTROL.md` §7), or (b) this session's green
`just cng-test-bench` run (31 lib tests + integration suites passing, recorded in the session
logs for the PROJ-608..613 and PROJ-618..621 waves; see `RELEASE_CONTROL.md` §8). Anything
resting on PROJ-614/616/622 is UNVERIFIED pending the consolidated final build — the
orchestrator flips those statuses after that build, not this document.

## Query-name drift resolution

`DEFINITION_OF_DONE.md` §4 names `metric-hook-actuations.rq` and `metric-dispatch-closure.rq`;
`DOD_EVIDENCE_MAP.md` recorded the on-disk names as `metric-hook-receipts.rq` and
`dispatch-closure.rq`. Resolution: the DoD names are being made authoritative by the PROJ-614
agent (the old names are folded into `metric-hook-actuations.rq` and
`metric-dispatch-closure.rq`). Because PROJ-614 is IN_PROGRESS and unbuilt, the rename is
UNVERIFIED pending final build; this sign-off cites the DoD names as target-authoritative and
does not claim they exist on disk yet.

## Clause-by-clause sign-off

One line per `DEFINITION_OF_DONE.md` section, evidence via `DOD_EVIDENCE_MAP.md`.

| § | Clause | Status | Evidence pointer |
|---|---|---|---|
| 1 | Core sentence (full autonomic chain) | PARTIAL | Manufacture/receipt/replay + hook actuation + loopback dispatch ALIVE (`RELEASE_CONTROL.md` §1/§7/§8); autonomic continuation markers UNVERIFIED pending PROJ-622 |
| 2 | Behavioral "operator never..." (12 clauses) | PARTIAL | Mechanisms landed for 1-12 per §8 ALIVE rows (PROJ-608..613, 618..621); behavioral workday-mode marker run UNVERIFIED pending PROJ-616/622 |
| 3 | Governing autonomic loop | PARTIAL | Manufacture→OCEL→receipt→replay ALIVE (§1/§7); Dispatch/Observe/Admit code ALIVE via `just cng-test-bench` (§8); loop-closure marker `AUTONOMIC_LOOP_CLOSED` UNVERIFIED pending PROJ-622 |
| 4 | Hook-morphism law (zero unreceipted actuation) | PARTIAL | Hook actuation 64/64 receipted in this session's `just cng-test-bench` (PROJ-612 row, §8); the SPARQL zero-gap check (`metric-hook-actuations.rq`) UNVERIFIED pending PROJ-614 |
| 5 | Dialect Registry Invariant | PARTIAL | Registry + `CNG_R14` gate green in `just cng-test-bench` (PROJ-613, §8); Arazzo dialect green (PROJ-621, §8); registry-as-executable-law marker query UNVERIFIED pending PROJ-622 |
| 6 | HookStanding lifecycle | ALIVE (code path) | PROJ-613 row, §8 — `just cng-test-bench` green this session; ChatmanEngine adoption DEFERRED as recorded |
| 7 | External dispatch doctrine (20-field contract, broker exclusivity) | ALIVE (loopback-real) | PROJ-618/619 rows, §8; `CNG_R15`/`CNG_R16` negatives in the same run. Live network endpoints UNVERIFIED, out of scope (§8.2); human consequences MOCKED-HUMAN |
| 8 | Dispatch state machine + readmission | ALIVE (loopback-real) | PROJ-618/619 rows, §8; `CNG_R17` admission refusal path in the same run; third-party endpoints UNVERIFIED (§8.2) |
| 9 | Callback/polling law | PARTIAL | Loopback collection surface exercised (PROJ-619, §8); bounded-polling marker query UNVERIFIED pending PROJ-614/622 |
| 10 | Recursive dispatch + parent-child closure | ALIVE (loopback-real) | PROJ-620 row, §8; closure/timeout/compensation suites in this session's `just cng-test-bench`; `RECURSIVE_CHILD_CLOSURE_PROVEN` marker UNVERIFIED pending PROJ-622 |
| 11 | Compensation | ALIVE (loopback-real) | PROJ-620 row, §8; `COMPENSATION_WORKFLOW_PROVEN` marker UNVERIFIED pending PROJ-622 |
| 12 | LLM edge-only | UNVERIFIED as enforced boundary | Doctrine binding; no enforcement test cited this session (`DOD_EVIDENCE_MAP.md` has no row for it) |
| 13 | Autonomic completion criterion (production chain) | PARTIAL | Chain segments through dispatch/admission ALIVE (§8); manifest `signatures: []` deliberately empty — PROJ-615 CUT (§8.1); end-to-end byte-identity + tamper harness UNVERIFIED pending PROJ-616 |
| 14 | Success markers (SPARQL-derived) | UNVERIFIED pending final build | Marker query set is PROJ-622 (IN_PROGRESS); no marker may flip until queries run over a real workday bundle. `V26_7_10_PRODUCTION_READY` NOT claimed |
| 15 | Current status summary | ALIVE (doc) | Superseded in detail by `RELEASE_CONTROL.md` §8 and this file; loopback/MOCKED-HUMAN boundaries restated there |

## Refusal-variant ledger delta

`DOD_EVIDENCE_MAP.md` recorded `CNG_R12`..`CNG_R18` as "code present, no test cited."
Forward-pointing PARTIAL note: this session's `just cng-test-bench` run (§8) exercised
`CNG_R12`/`CNG_R13`/`CNG_R14` (PROJ-610/612/613 rows) and the dispatch-wave negatives for
`CNG_R15`/`CNG_R16`/`CNG_R17`/`CNG_R18` (PROJ-618..621 rows) per session logs. The shared
ledger in `DOD_EVIDENCE_MAP.md` is not upgraded here — the consolidated final build has not
run, and the orchestrator owns the flip.

## What is explicitly not claimed

1. `V26_7_10_PRODUCTION_READY` — conjunction unresolved (PROJ-614/616/622 UNVERIFIED).
2. Live third-party network dispatch — out of scope, UNVERIFIED (`RELEASE_CONTROL.md` §8.2).
3. Real human consequences — synthesized ones are MOCKED-HUMAN everywhere they appear.
4. ed25519 manifest signatures — PROJ-615 CUT; `signatures: []` deliberately empty, PARTIAL.
5. Whole-workspace `just verify-all` — BLOCKED at recipe timeout per `RELEASE_CONTROL.md` §7
   item 11; not re-attempted this pass.

## See Also

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface; wins on disagreement
- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` — the doctrine signed off here (PROJ-606)
- `docs/releases/v26.7.10/DOD_EVIDENCE_MAP.md` — clause → query/test/refusal index
- `docs/jira/v26.7.10/tickets/index.md` — per-ticket status counterparts
- `.claude/rules/no-overclaiming.md` — status vocabulary used throughout
