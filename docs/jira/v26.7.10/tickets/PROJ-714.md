# PROJ-714 — Long-horizon scenarios (EOD-finishable scope)

Status: ALIVE (mechanism, 1 of 4 declared scenarios) / PLANNED (scenarios 2-4, time-boxed cut
this session)

Track: P (planning/decomposition).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730), §19/§20 item 3;
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

## Doctrine re-read (this revision)

DoD §20 item 3's exact text: "Long-horizon scenarios (4, G15/PROJ-714) are the declared cut
line: they may be CUT with a record, never quietly dropped or faked." §19's exact text: "5 IPC
domains ... × 20 seeded problems each ... plus 4 long-horizon scenarios." Neither clause
specifies that the 4 scenarios must be 4 independently hand-designed domains — the binding
requirements are (a) not faked/stubbed, (b) if cut, honestly recorded. This revision reads
that as permitting a narrower-but-real implementation, precisely labeled as such (below),
rather than requiring the CUT line to stay taken or requiring 4 maximally-distinct domains.

## Revised scope — reuse harness, vary domain, hold the same bar

Original framing (4 independently hand-authored long-horizon domains) is not finishable in
the remaining EOD window. Revised, real, non-stubbed scope:

1. **Scenario #1 (in flight)**: one clean-room domain requiring ~20-40+ genuine plan steps,
   structured so a real helper/main split is derivable (not a renamed short scenario) —
   `tests/cng_long_horizon_scenario.rs` + a new fixture, mirroring the potato/kitchen
   convention. Proves the pipeline (grounding → Datalog edge derivation → candidate search →
   planning → interference/release proofs → selection → receipt) holds at genuine length
   without reintroducing the grounding-blowup performance cliff this session already fixed
   (PROJ-733). This is the scaffolding every other scenario reuses — do not re-derive it.
2. **Scenarios #2-4**: do **not** hand-author three more novel domains. Draw three domains
   from the existing, already-proven-correct-and-fast 5-domain IPC generator family
   (`src/bench/ipc/{barman,blocksworld,grippers,termes,tyreworld}.rs`), run each at its
   largest generator size with a forced/extended plan chain so the resulting plan length
   clears the SAME ~20-40+ step bar scenario #1 established (not a lower bar — the point is
   proving the pipeline holds at length, not proving three more short scenarios). Each is one
   small test function (not a new file, not new scaffolding) added alongside scenario #1's
   test, following the identical shape: `decompose()` → assert typed outcome → assert
   `tape.ops.len()` clears the threshold → assert reasonable wall-clock. No new refusal codes,
   no new marker queries — this is proof-of-mechanism-at-scale, not new capability surface.

## Honest labeling (binding — do not let this round up)

This satisfies DoD §20 item 3's actual requirement ("not faked/stubbed") — every one of the 4
scenarios is a real `decompose()` call on a genuinely long, non-gamed plan, receipted like any
other. It does **not** satisfy a stronger reading some readers might assume from "4 scenarios"
— namely four maximally-distinct, independently-designed long-horizon domains. The record must
say precisely: "4/4 long-horizon scenarios proven (1 novel clean-room domain + 3 parameter
variations of the existing IPC generator family at extended plan length, same pipeline, same
step-count bar)" — never "4 independent long-horizon domains" or an unqualified "ALIVE" that
implies the stronger reading. If a future session wants the stronger reading, that is new,
separately-scoped work, not a gap in this ticket's honest closure.

## Verification

Each of the 4 scenario tests run for real, isolated `CARGO_TARGET_DIR` per agent to avoid lock
contention with concurrently-running work, exact command+output cited in `RELEASE_CONTROL.md`/
`DOD_SIGNOFF.md` per this repo's no-overclaiming rule — no status flip without a citation.
