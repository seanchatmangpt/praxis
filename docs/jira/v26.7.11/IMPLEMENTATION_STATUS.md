# Implementation Status — PRD v26.7.11, Layers 1-10

Generated 2026-07-11 from a 10-agent reconciliation workflow (5 agents auditing PRD §7.1-7.10
against real codebase evidence — commands run, files read, this session — then 5 agents
drafting the `tickets/index.md` breakdown from those findings). Status vocabulary per
`.claude/rules/no-overclaiming.md`. This is a snapshot; re-run the reconciliation before
trusting it as current if significant time has passed or commits have landed since.

**Read `SAFETY_FINDINGS.md` first** — two of the layers below (7.8, 7.9) surfaced a
fabricated-capability document and a live LLM-code-injection pathway, not just missing
features. That finding is more urgent than this status table.

| Layer | Name | Status | One-line finding |
|-------|------|--------|-------------------|
| 7.1 | Admitted Graph | PARTIAL | Oxigraph-backed truth substrate and explicit dialect routing are ALIVE and tested (34 passing tests cited); the negative claim "no runtime layer infers ambient authority from syntax" was only traced through one call site, not certified repo-wide. |
| 7.2 | PDDL | ALIVE | Real parse→ground→plan flow via `bcinr_pddl`, consumed identically by `praxis-graphlaw` and `cng`; confirmed zero execution-side coupling (PDDL doesn't own runtime semantics) via exhaustive grep. |
| 7.3 | POWL v2 | **ALIVE (library)** | **Updated later this session.** Sockets/closure now built and independently double-verified: `WorkflowSocketId`/`SocketPath`/`ParentChildEdge`/`ParentChildClosure` in `powl2-decompose/src/powl.rs` (PROJ-750). Still zero downstream consumers wiring these into `decompose.rs`/`recompose.rs`/emission — structurally real, not yet used. |
| 7.4 | External Cut Projection | **ALIVE (library), PARTIAL (production)** | **Updated later this session — largest status change in this table.** `validate_external_cut` now gates the real admission path; the SPARQL query's vocabulary now fully matches the real RDF emitter (the "would match nothing if run" finding is no longer true — independently re-verified by direct predicate comparison); real oxigraph execution returns real, exact-value rows against a real fixture; real Tera rendering produces a genuine Arazzo document that round-trips through the real parser; the receipt hashes real material digests (independently re-verified by recomputing them). Gap: none of it is called from `ChatmanEngine::admit_transition` (PROJ-751/752, verified; PROJ-796 tracks the wiring gap). See `RAIL_A_B_STATUS.md`'s "Update" section. |
| 7.5 | Arazzo | **ALIVE** | Parsing was already real; manufacturing (the other half) is now also real — see 7.4/7.6. |
| 7.6 | wasm4pm AIR Compiler | **ALIVE (library), PARTIAL (production)** | **Updated later this session — the load-bearing gap named here is closed.** `crates/wasm4pm-arazzo/src/lower.rs` now genuinely lowers a parsed `ArazzoDescription` into `AirProgram`; proven end-to-end with real source content (step ids, URLs, routing names) surviving into compiled WASM bytes (PROJ-753, independently double-verified). Typed refusal coverage added for cyclic dependencies, unsupported criteria, malformed retry policy (PROJ-754). Still true, unchanged: no WASM host-import/execution runtime exists anywhere in the repo, and the composed pipeline has no production caller (PROJ-796). |
| 7.7 | Shared Erlang Transition Core | PARTIAL | Real, compiling Erlang that actually runs (verified: compiled fresh, ran its eunit test, confirmed real Rust-NIF arithmetic). But `transition/2` returns a bare state, never the PRD-required `{S', C}` pair; join/AND-dependency readiness is manually faked in the shipped test itself; the module performs direct I/O (`dispatch_http/2`, `dispatch_rdma/3`) that PRD line 388 explicitly forbids in this layer; seven exported functions are permanently dead stubs. See `SAFETY_FINDINGS.md` §3-4 for the stale-binary and dead-stub findings. |
| 7.8 | OTP Outer Runner | MOCKED | The real supervisor (`arazzo_runner_sup.erl`) is structurally correct OTP. But: `restart => temporary` contradicts "SHALL survive restart"; none of the 10 required identity fields or 9 required reaction events are implemented; a confirmed dead-code bug means every workflow starts with `undefined` core state (`air_core:initial_state/0` doesn't exist); the runner's pattern-match against the transition core's return shape can never succeed. **See `SAFETY_FINDINGS.md` — this layer also contains the LLM-code-injection pathway and the unrelated `otp_runner.erl` fictional module.** |
| 7.9 | AtomVM Runner | MOCKED | The module under this name implements no AIR semantics at all — it is unrelated filler (see `SAFETY_FINDINGS.md` §3). The equivalence "proof" is prose contradicted by the actual implementation; its only test always passes regardless of behavior. No `rebar.config` exists for this app — it isn't part of any tracked build. |
| 7.10 | BCINR Local Runner | PARTIAL | Real `bcinr-*` crate dependencies exist and are genuinely used — but only for PDDL/STRIPS planning (a different concern), not for the process-cell responsibilities this layer specifies (activity eligibility, dependency satisfaction, socket attachment, child closure, next local transition). Zero implementation of those five responsibilities found. |

## Rail-level read (PRD §22 Delivery Order)

**Updated later this session.** Per `PRD.md:1083` ("No later rail SHALL be used to backfill
authority missing from an earlier rail"), Rail A and B (7.3-7.6) were both incomplete when this
table was first generated. A subsequent 10-agent build pass (PROJ-750 through PROJ-754, each
independently double-verified with fresh commands) closed every named gap except one: the
composed pipeline (POWL admission → SPARQL projection → Tera render → Arazzo manufacture → AIR
lowering → WASM compile) is real and tested end-to-end, but `ChatmanEngine::admit_transition`
still calls none of it — every stage is only ever exercised inside test code. That gap is
tracked as PROJ-796.

This changes, but does not eliminate, the original finding: Rail C-E's apparent progress
(real-looking Erlang, a supervisor, an AtomVM app — see 7.7-7.9 below, still MOCKED/PARTIAL as
found) is now standing on a Rail A/B foundation that is genuinely real *as a library*, but that
foundation still has no production entry point. Until PROJ-796 lands, none of Rails C-E's work
can be honestly counted as receiving real admitted input, even though Rail A/B itself is no
longer the blocker it was — the blocker moved from "the code doesn't exist" to "the code exists
but nothing calls it."

## See also

- `SAFETY_FINDINGS.md` — read first.
- `tickets/index.md` — PROJ-750 through PROJ-770, the actionable breakdown drafted from this
  reconciliation.
- `RAIL_A_B_STATUS.md` — deep-dive on the two blocking layers.
- `RAIL_G_MEASUREMENT_DESIGN.md` — instrumentation plan for the multifractal measurement rail.
- `PRD.md` — the specification itself.
