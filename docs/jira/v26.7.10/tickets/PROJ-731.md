# PROJ-731 — Final closure — RELEASE_CONTROL/DOD_SIGNOFF for v26.7.10-revised

Status: CLOSED (doc) — honest sign-off produced this session (PROJ-748:
`docs/releases/v26.7.10/DOD_SIGNOFF.md`, `DOD_EVIDENCE_MAP.md`), updated in place across
three waves of evidence this session (original closure, a follow-up verification round, and
a second synthesis round); the underlying `V26_7_10_PRODUCTION_READY` claim in its FULL §16
meaning is now ALIVE for the two-bundle (workday + planning) composition and remains
UNVERIFIED only for the three-bundle (+ distributed) composition — see DoD §16; Phase 6
commit not run this session

Track: D (doctrine + closure).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

After all gates G0-G16: closure table in `RELEASE_CONTROL.md`, clause-by-clause sign-off
against the revised DoD (same pattern as the interim PROJ-617 closure), all §16 markers
derived TRUE via SPARQL on a real run — any false marker is a typed refusal + nonzero exit.
Runs last in Track D.

## Closure note (updated this session — second synthesis round)

The documentation half of this ticket (closure table in `RELEASE_CONTROL.md` §9, sign-off in
`DOD_SIGNOFF.md`/`DOD_EVIDENCE_MAP.md`) is complete and honest, and has been kept current
across three waves of evidence landing in the same session:

1. **Original closure**: each marker family (planning, `LLM_CALLS_ZERO`, distributed)
   independently verified true; `full_production_ready`'s real two-bundle invocation not yet
   exercised (only unit-tested with a hand-fabricated `workday_markers` half).
2. **Follow-up verification round**: closed that two-bundle gap —
   `full_production_ready_holds_on_real_dual_bundle_evidence` (`cng_production_ready.rs`) runs
   a REAL `workday()` bundle and a REAL `decompose()` bundle together, all 26 keys `true`. Also
   closed the literal 8² fan-out (PROJ-729), the arazzo digest-verify wiring (PROJ-745), the
   full 5x20 IPC corpus scale (PROJ-711), and `CNG_R09`'s negative test (§18 item 7).
3. **Second synthesis round**: PROJ-749 stitched a real `decompose()` output into a real
   cross-engine dispatch run for the first time this milestone (mechanism-level; no
   payload-carrying contract yet, PROJ-710 -> PROJ-723 open); §18 negative-corpus item 6 was
   closed (item 7 additionally confirmed); a workspace-wide sanity sweep (`cargo check`,
   scoped clippy, `cargo fmt`) found the tree otherwise clean.
4. **Moonshot round** (this session, after the three waves above and a further load-bearing
   `GAP_AUDIT.md` closure wave): PROJ-710 -> PROJ-723 is now CLOSED — a dispatched contract
   carries its subworkflow's actual PDDL payload, digest-verified, and the remote engine
   executes that specific plan (`dispatched_subworkflow_payload_is_the_content_the_engine_
   actually_executes`, `cng_decompose_to_dispatch_integration.rs`, 3/3 passed). PROJ-714 moved
   from 1/4 to 2/4 long-horizon scenarios. §18 negative-corpus item 5 (mutex-saturated goals)
   was upgraded from an adjacent-scenario proxy to a literal fixture, closing all 8/8 items by
   their named mechanism. See `docs/releases/v26.7.10/RELEASE_CONTROL.md` §9.2/§9.2a and
   `docs/releases/v26.7.10/DOD_SIGNOFF.md`'s "Moonshot round" section for full detail.

What remains UNVERIFIED, per this paragraph as first written (STALE — see the correction
immediately below, which post-dates both the EOD push and the moonshot round): `full_
production_ready`'s real THREE-bundle composition; a dispatched contract carrying its
subworkflow's actual PDDL payload; potato itself dispatched across H/M (it has no split);
PROJ-714's long-horizon scenarios (declared cut); any live-repo `ggen sync run`; and Phase 6.

**Correction (current, after the EOD push and the moonshot round)**: the real THREE-bundle
`full_production_ready` composition is now ALIVE
(`full_production_ready_holds_on_real_triple_bundle_evidence`,
`cng_production_ready_three_way.rs`, EOD push — see `DOD_SIGNOFF.md`'s "EOD push" section).
The payload-carrying-contract gap is now CLOSED (PROJ-710 -> PROJ-723, moonshot round — see
item 4 above). PROJ-714 stands at 2/4 (moonshot round), not fully cut. Potato dispatched
across H/M, any live-repo `ggen sync run`, and Phase 6 push remain genuinely open, unchanged.
G0-G16 status: G0-G13 code-level evidence unchanged; G14 ALIVE at full scale; G15 ALIVE
(mechanism, 2/4) / PLANNED (3-4, time-boxed cut — `RELEASE_CONTROL.md` §9.2, `PROJ-714.md`);
G16's SPARQL-derived conjunction is ALIVE for the three-bundle composition (EOD push).
