# DOD_SIGNOFF_INTERIM — Clause-by-Clause DoD Sign-Off for v26.7.10 (PROJ-617, interim)

SUPERSEDED (2026-07-10): this is the prior v26.7.10 (interim) sign-off, preserved verbatim
(fix-forward, not deleted) per PROJ-748. It signs off against the superseded interim DoD
(`DEFINITION_OF_DONE_INTERIM.md`), closed at commit `31c236f` with all 11 markers derived
TRUE. The governing sign-off for v26.7.10-revised is `DOD_SIGNOFF.md` in this directory
(PROJ-748). Nothing below is discarded — the interim closure stands as evidence of the
substrate the revised DoD builds on.

Status: FINAL, tied to `RELEASE_CONTROL.md`. Every claim cites a file, test, or receipt. If
this file and `RELEASE_CONTROL.md` disagree, `RELEASE_CONTROL.md` wins.

Scope of "ALIVE" here: verified this closing session, meaning (a) the PROJ-601..605
verification ladder (`RELEASE_CONTROL.md` §7), or (b) the consolidated final build,
orchestrator-verified this session (`RELEASE_CONTROL.md` §8): `just cng-test-bench` ALL GREEN
(40 lib tests + integration suites 6/1/1/5/4/2, 0 failures) and `just cng-workday-verify`
(seed=616, ticks=8, rpm=125) — two same-seed runs byte-identical, all 11 success markers TRUE,
`evidence_chain_digest blake3:4e38a38f…0475`, `ocel_graph_digest blake3:853638…b315`,
`run_hook_hash ba8615…8ffe`. These two commands are cited below as "build item 1" and
"build item 2".

Fix-forward record (part of the evidence trail): after the first consolidated build, the
orchestrator rewrote `crates/cng/queries/markers/marker-child-closure.rq` and
`crates/cng/queries/metric-dispatch-closure.rq` to fix a SPARQL scoping bug — FILTER on
outer-bound `?law` inside UNION branches was unbound in branch scope, so `satisfiedParents`
was always 0; the closure law is now matched as a triple pattern inside each UNION arm,
mirroring `dispatch-closure.rq`. Build items 1-2 postdate this fix.

## Query-name drift resolution — VERIFIED

`DEFINITION_OF_DONE.md` §4's names are now authoritative and real:
`crates/cng/queries/metric-hook-actuations.rq` and
`crates/cng/queries/metric-dispatch-closure.rq` exist on disk (`ls` verified this session) and
are exercised by build items 1-2. The old `metric-hook-receipts.rq` is deleted (`ls` confirms
absence). `DOD_EVIDENCE_MAP.md` has been swept to the new names.

## Clause-by-clause sign-off

One line per `DEFINITION_OF_DONE.md` section, evidence via `DOD_EVIDENCE_MAP.md`.

| § | Clause | Status | Evidence pointer |
|---|---|---|---|
| 1 | Core sentence (full autonomic chain) | ALIVE (scoped) | build item 2 — all 11 markers TRUE over a real workday bundle; loopback-real dispatch, MOCKED-HUMAN consequences (§8.2) |
| 2 | Behavioral "operator never..." (12 clauses) | ALIVE (scoped) | build item 2 — `ONE_PERSON_RECURSIVE_WORKFLOW_PROVEN` TRUE; mechanisms per `DOD_EVIDENCE_MAP.md` behavioral table |
| 3 | Governing autonomic loop | ALIVE | build item 2 — `AUTONOMIC_LOOP_CLOSED` TRUE; 1 refusal resumed in telemetry |
| 4 | Hook-morphism law (zero unreceipted actuation) | ALIVE | build item 2 — 64/64 hook actuations receipted; `metric-hook-actuations.rq` on disk; `ZERO_UNRECEIPTED_ACTUATION` TRUE |
| 5 | Dialect Registry Invariant | ALIVE | build items 1-2 — `CNG_R14` gate green, `GRAPHLAW_DIALECT_CLOSURE` TRUE; Arazzo registered (PROJ-621) |
| 6 | HookStanding lifecycle | ALIVE | build item 1 (PROJ-613 suites); ChatmanEngine adoption DEFERRED as recorded |
| 7 | External dispatch doctrine (20-field contract, broker exclusivity) | ALIVE (loopback-real) | build items 1-2 — `EXTERNAL_WORKFLOW_DISPATCH_PROVEN` TRUE, 3 dispatches sent; live network UNVERIFIED, out of scope (§8.2); MOCKED-HUMAN |
| 8 | Dispatch state machine + readmission | ALIVE (loopback-real) | build item 2 — `EXTERNAL_RESULT_READMISSION_PROVEN` TRUE, 3 consequences admitted; third-party endpoints UNVERIFIED (§8.2) |
| 9 | Callback/polling law | ALIVE (loopback-real) | build items 1-2 — loopback collection surface exercised through the admission pipeline; no unbounded polling path exists in scope |
| 10 | Recursive dispatch + parent-child closure | ALIVE | build item 2 — `RECURSIVE_CHILD_CLOSURE_PROVEN` TRUE (post fix-forward on `marker-child-closure.rq`) |
| 11 | Compensation | ALIVE | build item 2 — `COMPENSATION_WORKFLOW_PROVEN` and `TIMEOUT_ESCALATION_PROVEN` TRUE |
| 12 | LLM edge-only | UNVERIFIED as enforced boundary | doctrine binding; no enforcement test cited this session (`DOD_EVIDENCE_MAP.md` has no row for it) |
| 13 | Autonomic completion criterion (production chain) | ALIVE (scoped) | build items 1-2 end to end; manifest `signatures: []` deliberately empty — PROJ-615 CUT, PARTIAL by design (§8.1) |
| 14 | Success markers (SPARQL-derived) | ALIVE — VERIFIED-TRUE | build item 2 — all 11 markers TRUE via SPARQL over the emitted OCEL graph; marker negatives green in build item 1 |
| 15 | Current status summary | ALIVE (doc) | refreshed this pass; detail owned by `RELEASE_CONTROL.md` §8 and this file |

## V26_7_10_PRODUCTION_READY — scoped claim

`V26_7_10_PRODUCTION_READY` derived TRUE as the marker conjunction on build item 2. The claim
is scoped exactly as the DoD defines it: loopback-real external dispatch (deterministic
filesystem outbox/inbox), MOCKED-HUMAN synthesized human consequences, live network endpoints
out of scope and UNVERIFIED (`RELEASE_CONTROL.md` §8.2). No unscoped production-ready claim is
made or permitted.

## What remains explicitly not claimed

1. Live third-party network dispatch — out of scope, UNVERIFIED (`RELEASE_CONTROL.md` §8.2).
2. Real human consequences — synthesized ones are MOCKED-HUMAN everywhere they appear.
3. ed25519 manifest signatures — PROJ-615 CUT; `signatures: []` deliberately empty, PARTIAL.
4. LLM edge-only as an *enforced* runtime boundary (§12 above) — doctrine only this release.
5. Whole-workspace `just verify-all` — BLOCKED at recipe timeout per `RELEASE_CONTROL.md` §7
   item 11; not re-attempted this pass.

## See Also

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface; wins on disagreement
- `docs/releases/v26.7.10/DEFINITION_OF_DONE_INTERIM.md` — the doctrine signed off here (PROJ-606)
- `docs/releases/v26.7.10/DOD_EVIDENCE_MAP_INTERIM.md` — clause → query/test/refusal index
- `docs/releases/v26.7.10/DOD_SIGNOFF.md` — governing sign-off for v26.7.10-revised (PROJ-748)
- `docs/jira/v26.7.10/tickets/index.md` — per-ticket status counterparts
- `.claude/rules/no-overclaiming.md` — status vocabulary used throughout
