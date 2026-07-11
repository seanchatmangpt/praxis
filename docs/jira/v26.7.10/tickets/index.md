# Milestone Overview: v26.7.10 — Recursive Workflow

Two phases share this milestone. Phase 1 (PROJ-601..622) — the Fortune-5 benchmark hardening
plus the single-process autonomic loop — is **CLOSED as an interim milestone** at commit
`31c236f` (`docs/releases/v26.7.10/RELEASE_CONTROL.md` §8,
`docs/releases/v26.7.10/DEFINITION_OF_DONE_INTERIM.md`). Phase 2 (PROJ-701..731,
v26.7.10-revised) supersedes the DoD in place with the No-LLM Multi-Actor Planning +
Multi-Engine Execution doctrine (`docs/releases/v26.7.10/DEFINITION_OF_DONE.md`, PROJ-730).
`RELEASE_CONTROL.md` is the single control surface; if this index and it disagree, it wins.

Statuses use the no-overclaiming vocabulary. As of this session, PROJ-701..713/720..729/
733/734/739..745/749 are ALIVE (or ALIVE with a named PARTIAL/UNVERIFIED gap), evidenced by
`cargo test -p cng --features bench` (107 tests, 0 failures) and the specific tests cited per
ticket — including PROJ-749's own dedicated test binary
(`cng_decompose_to_dispatch_integration`, 2/2), added after that 107-test figure was measured
and not folded into it; PROJ-714 remains PLANNED as a genuinely never-built, declared cut line
(`RELEASE_CONTROL.md` §9.2); PROJ-730/731/746/747/748 are doc-track tickets, statused
individually below. Nothing flips without a cited command + output in `RELEASE_CONTROL.md`
§9/§9.1. Work is uncommitted this session — HEAD is still `1f3f9bc`.

## Phase 1 — PROJ-601..622 (interim milestone, CLOSED)

| Ticket | Scope | Status |
|---|---|---|
| [PROJ-601](PROJ-601.md) | `digests.json` path portability in `verify` | CLOSED (`40f6020`) |
| [PROJ-602](PROJ-602.md) | `cng evidence replay` verb for auditors | CLOSED (`40f6020`) |
| [PROJ-603](PROJ-603.md) | Bundle manifest schema (every digest named) | CLOSED (`40f6020`) |
| [PROJ-604](PROJ-604.md) | Close inline-SPARQL sites + guard test | CLOSED (`40f6020`) |
| [PROJ-605](PROJ-605.md) | `CNG_R11 AuditMismatch` refusal variant | CLOSED (`40f6020`) |
| [PROJ-606](PROJ-606.md) | DoD doctrine document (now interim) | CLOSED (interim) |
| [PROJ-607](PROJ-607.md) | Doc reconciliation pass | CLOSED (interim) |
| [PROJ-608](PROJ-608.md) | `benchmark workday` verb | CLOSED (interim, `31c236f`) |
| [PROJ-609](PROJ-609.md) | Interruption + planning categories (14) | CLOSED (interim, `31c236f`) |
| [PROJ-610](PROJ-610.md) | `standing-next-action.rq` + `CNG_R12` | CLOSED (interim, `31c236f`) |
| [PROJ-611](PROJ-611.md) | Bounded admission resume loop | CLOSED (interim, `31c236f`) |
| [PROJ-612](PROJ-612.md) | graphlaw hook pack actuation, `CNG_R13` | CLOSED (interim, `31c236f`) |
| [PROJ-613](PROJ-613.md) | Dialect registry + HookStanding CNG_R14 | CLOSED (interim, `31c236f`) |
| [PROJ-614](PROJ-614.md) | Graph-authoritative metrics closure | CLOSED (interim, `31c236f`) |
| [PROJ-615](PROJ-615.md) | Optional ed25519 signatures | CUT (`RELEASE_CONTROL.md` §8.1) |
| [PROJ-616](PROJ-616.md) | Verification harness + tamper negatives | CLOSED (interim, `31c236f`) |
| [PROJ-617](PROJ-617.md) | Interim release closure + DoD sign-off | CLOSED (interim, `31c236f`) |
| [PROJ-618](PROJ-618.md) | Dispatch contract + 13-state machine | CLOSED (interim, `31c236f`) |
| [PROJ-619](PROJ-619.md) | Broker dispatch + re-admission (loopback) | CLOSED (loopback-real) |
| [PROJ-620](PROJ-620.md) | Recursive closure/timeout/compensation | CLOSED (interim, `31c236f`) |
| [PROJ-621](PROJ-621.md) | Arazzo dialect | CLOSED (interim, `31c236f`) |
| [PROJ-622](PROJ-622.md) | SPARQL-derived success markers (11 TRUE) | CLOSED (interim, `31c236f`) |

## Phase 2 — PROJ-701..749 (v26.7.10-revised)

Numbers **PROJ-715..719 are deliberately skipped** — the gap separates Track P (planning,
701-714) from Track E (multi-engine execution, 720-729); no tickets ever existed there.

### Track P — no-LLM planning/decomposition

| Ticket | Scope | Status |
|---|---|---|
| [PROJ-701](PROJ-701.md) | `pddl-strips.ttl` ontology + closed shapes | ALIVE |
| [PROJ-702](PROJ-702.md) | Lifter: PDDL string → pddl-strips triples | ALIVE |
| [PROJ-703](PROJ-703.md) | Deterministic PDDL renderer + round-trip test | ALIVE |
| [PROJ-704](PROJ-704.md) | `decomp.dl` + `decomp-resources.dl` edge rules | ALIVE |
| [PROJ-705](PROJ-705.md) | Bounded canonical candidate enumeration | ALIVE |
| [PROJ-706](PROJ-706.md) | CONSTRUCT manufacture of helper/main problems | ALIVE |
| [PROJ-707](PROJ-707.md) | Interface state `s′` replay + `CNG_R23` | ALIVE |
| [PROJ-708](PROJ-708.md) | Non-interference `CNG_R22` + release `CNG_R24` | ALIVE |
| [PROJ-709](PROJ-709.md) | POWL nested-PartialOrder composition + powl2 | ALIVE |
| [PROJ-710](PROJ-710.md) | Selection law + typed `DecompositionOutcome` | ALIVE |
| [PROJ-711](PROJ-711.md) | IPC generators (5x20 domains, solvability gate) | ALIVE (full 5x20 scale) |
| [PROJ-712](PROJ-712.md) | Potato canonical scenario + negative corpus | ALIVE |
| [PROJ-713](PROJ-713.md) | Anti-hardcoding gate | ALIVE |
| [PROJ-714](PROJ-714.md) | 4 long-horizon scenarios (declared cut line) | PLANNED (cut line, §9.2) |
| [PROJ-733](PROJ-733.md) | `pddl-index` grounder swap (performance fix) | ALIVE |

### Track E — multi-engine execution

| Ticket | Scope | Status |
|---|---|---|
| [PROJ-720](PROJ-720.md) | 16-state dispatch machine + drift test | ALIVE |
| [PROJ-721](PROJ-721.md) | Durable ledger + idempotent consume (`DoubleAdmit`) | ALIVE |
| [PROJ-722](PROJ-722.md) | EngineIdentity + per-engine bundle layout | ALIVE |
| [PROJ-723](PROJ-723.md) | `cng engine serve` verb (remote engine loop) | ALIVE |
| [PROJ-724](PROJ-724.md) | `cng engine resume` + `--partial` prefix replay | ALIVE |
| [PROJ-725](PROJ-725.md) | Arazzo 1.1 vocab/shape delta + REMOTE_* projection | ALIVE |
| [PROJ-726](PROJ-726.md) | `packs/arazzo-pack/`: graph → arazzo/API YAML | ALIVE |
| [PROJ-727](PROJ-727.md) | Distributed evidence: OBS_KINDS, OCEL, markers | ALIVE |
| [PROJ-728](PROJ-728.md) | Multi-process harness + isolation falsifiers | ALIVE (test harness) |
| [PROJ-729](PROJ-729.md) | G13 crash-resume, byte-identity, 8² across engines | ALIVE (harness, full 8² fan-out) |
| [PROJ-734](PROJ-734.md) | G13 watch-loop race fix (`.ttl`-only filter) | ALIVE |
| [PROJ-744](PROJ-744.md) | `arazzo-pack` registered in `ggen.toml` | ALIVE |
| [PROJ-745](PROJ-745.md) | `verify_arazzo_render_digest` seam | ALIVE (fn, wired) |

### Track P/E integration

| Ticket | Scope | Status |
|---|---|---|
| [PROJ-749](PROJ-749.md) | Decompose-to-dispatch bridge (`decomp/dispatch_bridge.rs`) | ALIVE (mechanism, non-potato fixture) |

### Track D — doctrine + closure

| Ticket | Scope | Status |
|---|---|---|
| [PROJ-730](PROJ-730.md) | Revised DoD doctrine + ticket set (these docs) | IN PROGRESS |
| [PROJ-731](PROJ-731.md) | Final closure — control surface + sign-off | CLOSED (doc) |
| [PROJ-739](PROJ-739.md) | 6 planning marker queries + `PLANNING_MARKER_MAP` | ALIVE |
| [PROJ-740](PROJ-740.md) | 3 `LLM_CALLS_ZERO` family markers | ALIVE |
| [PROJ-741](PROJ-741.md) | `cng plan decompose` verb | ALIVE |
| [PROJ-742](PROJ-742.md) | `full_production_ready` combinator | ALIVE (fn) / ALIVE (2-run) / UNVERIFIED (3-run) |
| [PROJ-743](PROJ-743.md) | DoD §16 marker-name reconciliation | DONE (doc) |
| [PROJ-746](PROJ-746.md) | Ticket status flips + `RELEASE_CONTROL.md` §9 sync | DONE (doc) |
| [PROJ-747](PROJ-747.md) | PROJ-714 cut-line record | DONE (doc) |
| [PROJ-748](PROJ-748.md) | Revised `DOD_SIGNOFF.md`/`DOD_EVIDENCE_MAP.md` | DONE (doc) |

## Execution sequence (as actually run this session)

```text
Phase 1 (perf/harness fix): 733 -> 734.
Phase 2 (isolated verification): cng_decomp -> cng_ipc_corpus -> cng_multi_engine -> full suite.
Phase 3 (markers, parallel): 739, 740, 741, 742, 743.
Phase 4 (arazzo wiring, parallel): 744, 745.
Phase 5 (doc closure, sequential): 746 -> 747 -> 748.
Phase 6 (commit): NOT run this session — HEAD still `1f3f9bc`, `git status` not clean.
```

A follow-up verification round (4 targeted agents, isolated `CARGO_TARGET_DIR=target/agent-7xx`
builds, after Phase 5's initial doc closure) closed five further gaps: the literal 8² fan-out
(PROJ-729), the real two-bundle `full_production_ready` invocation (PROJ-742), the arazzo
digest-verify wiring (PROJ-745), the full 5x20 IPC corpus scale (PROJ-711), and `CNG_R09`'s
negative test in `decomp/`. See `DOD_SIGNOFF.md`'s "What changed in the follow-up verification
round" section for the full command+output citations. This table and `RELEASE_CONTROL.md` §9.1
reflect that follow-up round's evidence.

A second, separate synthesis round (5 more agents) then added: a clean workspace-wide `cargo
check`/`cargo test --no-run`, a scoped `praxis-graphlaw`/`pddl-index` clippy sweep, a clean
`cargo fmt --all --check`, DoD §18 negative-corpus item 6 closed (item 7 additionally
confirmed), and PROJ-749 — the decompose-to-dispatch bridge that closes the Track P/Track E
integration gap named below. See `DOD_SIGNOFF.md`'s "What changed in this session's second
synthesis round" section and `RELEASE_CONTROL.md` §9.1 items 7-8 / §9.3 for the full
command+output citations.

Original Phase 2 per-ticket sequence (`D: 730 first, 731 last; P: 701->...->713->(714);
E: 720->721->722->723->724->725->726->727->728->729`) is superseded by the above — the code
for Tracks P/E had already landed before this session (wave 1+2); this session's work was
fixing the two real bugs blocking verification (733/734), running everything, and reconciling
the doctrine to what actually ran, not re-implementing in ticket order. Tracks P and E were
independent until integration through Phase 5 and the follow-up round: PROJ-710's
`DecompositionResult` graph fed PROJ-723/725's dispatch surface only in the sense that both
existed on disk, with no code stitching a real `decompose()` output through to a real
cross-engine dispatch run. PROJ-749 (second synthesis round) closes that specific gap at the
mechanism level, via a new bridge module (`decomp/dispatch_bridge.rs`) and a new dedicated
integration test — not by modifying `cng_multi_engine.rs` — on a fixture built for the
purpose (the canonical potato scenario's own `decompose()` output is single-actor and has
nothing to dispatch). See `DOD_SIGNOFF.md` §2/§8 for the exact scope of what PROJ-749 proves
and what it does not (no PDDL-payload-carrying contract yet, PROJ-710 -> PROJ-723 open).

## Standing boundaries (honesty notes)

- Multi-engine transport is filesystem inbox/outbox between separate OS processes; HTTP
  binding is declared via generated OpenAPI/AsyncAPI docs and UNVERIFIED as a live network
  path (`DEFINITION_OF_DONE.md` §20).
- Long-horizon scenarios (PROJ-714) are the declared cut line — may be CUT, never faked; see
  `RELEASE_CONTROL.md` §9.2 for the recorded cut subsection (PROJ-747).
- Synthesized human consequences remain MOCKED-HUMAN wherever they appear.
- Phase 1 boundaries (loopback-real dispatch, TripleStore hook surface, deferred
  ChatmanEngine) are recorded in `DEFINITION_OF_DONE_INTERIM.md`.
- `V26_7_10_PRODUCTION_READY` in its full DoD §16 meaning requires composing a `workday()`
  bundle with a `cng plan decompose` bundle via `full_production_ready` (PROJ-742); a follow-up
  verification round ran that two-bundle composition end-to-end (`cng_production_ready.rs`, all
  26 keys `true`) — see `DEFINITION_OF_DONE.md` §16 and `RELEASE_CONTROL.md` §9.1. The
  three-bundle composition (+ a real distributed bundle) remains UNVERIFIED.
- PROJ-749's decompose-to-dispatch bridge proves the mechanism (a real `decompose()` output
  reaches a real cross-engine dispatch run), not payload fidelity: the remote engine still
  executes its OWN dispatch-id-seeded synthetic PDDL artifact set, not the dispatched
  subworkflow's actual plan — no payload-carrying contract exists yet (PROJ-710 -> PROJ-723).
  It also does not cover the potato scenario itself, which selects single-actor and has
  nothing to dispatch — see `DOD_SIGNOFF.md` §2/§8.
- This session's work is uncommitted — `git status` is not clean, HEAD is still `1f3f9bc`.
  Nothing in this index or `RELEASE_CONTROL.md` claims the increment is committed.

## See Also

- `docs/releases/v26.7.10/RELEASE_CONTROL.md` — single control surface
- `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` — governing doctrine (v26.7.10-revised)
- `docs/releases/v26.7.10/DEFINITION_OF_DONE_INTERIM.md` — superseded interim DoD
- `docs/releases/v26.7.10/PRD.md`, `docs/releases/v26.7.10/ARD.md`
