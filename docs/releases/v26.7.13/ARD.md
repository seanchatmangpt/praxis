# ARD — Architecture Reference Document, v26.7.13

Status: DRAFT, tied to `RELEASE_CONTROL.md` (single control surface). Every claim in this
document cites a file, test, or receipt in this repository. Rows without evidence are marked
PLANNED or UNKNOWN, never asserted. Companion to `docs/releases/v26.7.13/PRD.md`, sharing its
Claims Reconciliation table verbatim per this milestone's house template — the two documents
must not diverge on status for the same claim number. This ARD carries the architectural detail
behind the eight shipped-work themes (Sec. 2 below expands rows referenced from the table) and,
separately, the ratified Rust-only forward architecture (`ArchitectureSnapshot` carrier,
truthful `SearchOutcome` algebra, dependency-footprinted semantic caching, Datalog
specialization behind a differential promotion gate, `TraceEq`-guarded search reduction,
six-obligation cross-slice composition, `PlanWitness`/`plancheck`) — every forward item is
PLANNED or UNKNOWN in this document; nothing new claims ALIVE.

## Claims Reconciliation

Identical table to `PRD.md`, reproduced verbatim below (not summarized by reference) per this
milestone's explicit mirroring requirement — the two files must never drift on status, scope,
or evidence for the same claim number. Status vocabulary: **ALIVE** (verified, executes, cited
test/receipt passes), **PARTIAL** (real but narrower than the claim — gap named explicitly),
**PLANNED** (roadmap/ticket only, no code path), **UNKNOWN** (not yet investigated to a
verdict), **MOCKED** (a stand-in exists where the claim implies the real thing).

| # | Claim | Status | Scope / caveat | Evidence | Ticket |
|---|---|---|---|---|---|
| 1 | Theme A — Crown-witness composition continuation (LOCAL/EXTERNAL) | PARTIAL | LOCAL 9/11 `REAL_EDGE` + 2 `PARTIAL_REAL_EDGE` (`F08→F09`, `F18→F19`); EXTERNAL `MISSING_EDGE_COUNT`=0 but `F10→F12` `PARTIAL_REAL_EDGE`; union 20 REAL / 3 PARTIAL of 23 distinct edges. `LOCAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` = **false**, `EXTERNAL_OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` = **false**, `OBSERVATION_TO_REPLAY_CONTIGUOUS_PATH` = **false** — stated as-is, never rounded up. Repair R8 (`F08→F09`, `F18→F19`, `F10→F12`) unstarted. | `docs/jira/v26.7.12/CROWN_STATUS.md` (executive-summary table); LOCAL driver `crown_local::drive_local_witness_prefix` (commits `3322bf2d`, `d60f2036`, `eeca952a`, `66d8732e`, `0815680a`, `217dc37d`, `66cb59b1`); EXTERNAL tail `crown_external::drive_external_witness_tail` (`4ce20102` `F16→F18`, `1e1ce976` `F18→F20`) | R8 (v26.7.12 repair backlog, unstarted) |
| 2 | Theme B — ~40 production-issue fixes (determinism, typed refusals, mutex poisoning, security/injection) | ALIVE per fix | Count is approximate (~40 across the milestone), not an exact commit tally. This row cites a representative, non-exhaustive sample spanning the four named categories; each cited commit was independently verified this session per its own commit message, not re-verified here. | mutex poisoning: `0767bcc2`, `35d991ca` (recover from poisoned `GLOBAL_ENCODER`/registry locks), `aee200c7` (`r2s_operator` poison regression test); determinism: `967ad485` (DSTREAM deletion order), `4de48d54` (SPARQL `GROUP BY` row order), `f08b4e41` (`sh:closed`/ShEx `CLOSED` order), `89ba964c` (SHACL report order), `bba62242` (empty-ruleset false-positive cycle); security/injection: `035bf0ec`, `f789c6db` (refuse `"""` before Turtle embedding), `4b2774ba` (validate `engine_id`, block path relocation); typed refusals: `46dcee68` (`RSPBuilder::build`), `d01d2904` (CSprite panic), `06ee9f81` (`kh:window` underflow), `213ebcbe` (`substitute_triple_with_bindings`), `64048f43` (`Parser::parse` bounds-check) | UNTRACKED (informal swarm findings #3–#33, no single PROJ ticket covers this bucket) |
| 3 | Theme C — ggen 26.7.13 (`extra_ontologies`, `lock:bool`, pack migrations, GGEN_PARITY.md) | ALIVE | `extra_ontologies` mechanism + first consumer (`togaf-adm-pack`) ALIVE; `lock:bool` opt-out ALIVE; version bump to 26.7.13 + clean sync ALIVE — all three superseding `docs/GGEN_PARITY.md`'s own BLOCKED/PARTIAL disclosure, which reflected an earlier, since-resolved session state. `GGEN_PARITY.md`'s disclosed `EXPECTED_FACTORY_HEAD` pinning risk remains open (Sec. 12, register item). | `4f1c4428` (`lock:bool`; `pack_e2e` 12/12, `ggen_toml_schema_match` 5/5, `graph_config_test` 8/8, `ggen_toml_semantic_validation` 7/7); `5639123b` (version bump, clean `ggen sync run`, `ggen 26.7.13` installed live, idempotent second run, adversarial fail-closed tamper check restored); `89279221` (3-pack `extra_ontologies` migration, sync clean and idempotent); `6342e2d6` (`docs/GGEN_PARITY.md`, 5-agent survey verdict doc) | #102–#105, #111 |
| 4 | Theme D — TOGAF ADM increment 1 | ALIVE, scoped to increment 1 only | Increment 1 (togaf-adm-pack + phase fixtures + full-cycle projection test) only; increments 2 and 3 are zero-commit PLANNED (Sec. 12). `togaf.rs`'s own module doc stamps `(v26.7.13)` though only increment 1 has landed — the ticket status below is authoritative, not the doc-comment stamp. | `c0e7cd71` (TOGAF ADM full-lifecycle increment 1 + ggen `extra_ontologies` unions; `ggen-test-isolated togaf1 pack_e2e` 10/10 incl. 2 new, `graph_config_test` 8/8, chicago `test!` 4 new, cng lib 173/173); `67106c39` (module-doc private-namespace undercount fix, dogfood w6a5iv25g) | #99 |
| 5 | Theme E — SOC2 testbed stages 1–4 + Arclight rescale + hook-actuation tests | PARTIAL, explicitly not a compliance claim | This is a workflow-domain-modeling exercise, not a SOC2 attestation of any system, including this repo — see Doctrine Sec. 6 for the compliance-overclaim fence. Stage 2 disclosed a concurrent-edit collision against the live shared tree (176/3 vs. 179/0 isolated). Stage 3's 3 full-lib failures were pre-existing (Arclight-naming drift), not a Stage-3 regression, resolved by the reconciliation commit. | Stage 1 `756a2584` (`soc2` filter 4/0, full lib 177/0); Stage 2 `3909f89d` (`bench::soc2` 5p, `bench::roles` 2p, `bench::hooks` 5p; isolated 179/0, live tree 176/3 — 3 unrelated pre-existing fixed forward same commit); Fortune-5 rescale `f8c9e3dd`; Stage 3 `fd504676` (`soc2_growth` filter 5/0, full lib 181/3 at Stage-3 `HEAD`); Arclight reconciliation `8f461232` (isolated `bench::soc2` 10/0, full lib suite **184/0**, background task `b02woqay0`); fence doc `2a9f90a1`; stale-claims correction `7b6a08e0` (dogfood wl8n77q65) | #107–#119 |
| 6 | Theme F — `materialize` `catch_unwind` hardening | ALIVE, scoped | Hardens a panic path with no currently-demonstrated production trigger — proactive, not a reproduced-crash fix. The follow-on `verdicts`-not-restored bug (a real, demonstrated gap) is fixed in the same theme's second commit. `Binding::len()`'s HashMap-iteration-order column-length issue (`bindings.rs:22`) was found but reachability via real query-constructed bindings is unconfirmed (open disclosure item). | `70f2faba` (`catch_unwind` rollback hardening; full `--lib` suite isolated 428/0, 7 ignored); `bf982815` (restores `verdicts` in both rollback arms + `strip_comments` escaped-quote fix; `materialize_clears_verdicts_on_rollback_not_just_triple_index` 1/0; `strip_comments` 4/0; full `--lib` 433/0, 7 ignored) | #120, #121 |
| 7 | Theme G — gap-closure coverage commits | ALIVE, narrow scope | Test/disclosure coverage only, no new features. 6-commit fixer-candidate batch: two close reachability gaps with new tests, one discloses a target as structurally untestable rather than fabricating coverage, one retracts a stale doc claim rather than leaving it uncorrected. | `d0372c67` (`ProfileSymbolTable::project_quad` round-trip); `d969119a` (`consult_breed`/`into_authority` under `--all-features`); `74b63c63` (`ReplayMismatch::ReplayRefused` in `chatman_receipts_chain`); `f5b59a56` (F04 `IoRefused` disclosed structurally untestable); `3e582bf9` (F07 `ShexValidatorError` reachability closed; `ShaclValidatorError` disclosed open); `581db137` (retract stale bracket-workaround claim in `project_residue` doc) | #106 |
| 8 | Theme H — arazzo Erlang fixes | ALIVE, scoped to finding #13's siblings | Covers finding #13's OTP-runner broker-dispatch, OTP-runner reaction-dispatch, and AtomVM transition siblings. A fourth, structurally identical sibling (`loop_waiting_for_io/2`) is deliberately left unwrapped as confirmed dead code, not overlooked. Task #85's originally suspected `arazzo_runner_broker_test` hang is NOT supported by the repo's own test record and is closed as unsupported-by-record, not fixed. `admit_return/3` remains a disclosed, un-fixed production dead end (broker-path completions never re-enter `air_core:transition`). | `2966330f` (`apply_transition/4` broker-dispatch catch; `erlang-test-workflow` 5 tests/1 pre-existing unrelated failure); `f22c1db4` (AtomVM transition catch; `erlang-test-atomvm-workflow` 3/0, `erlang-test-atomvm-differential` 4/0); `3d37cf24` (`handle_reaction/3` catch-all; 5 tests/1 pre-existing unrelated failure); `df634bb8` (3 `dispatch_event` coverage gaps, dogfood w6a5iv25g) | finding #13 (informal); #85 (closed-as-unsupported) |

### Forward-architecture claims (ARD addendum, not present in `PRD.md`'s table)

The ratified Rust-only forward architecture has zero code in this repository as of this
writing — confirmed this session by `grep -rln "plancheck\|PlanWitness\|SearchOutcome\|
ArchitectureSnapshot" crates --include='*.rs'` (zero hits) and `find crates/praxis-graphlaw/src
-maxdepth 1 -iname "architecture*"` (zero hits, the proposed module home does not exist yet).
Every row below is PLANNED or UNKNOWN; none is a `PRD.md` claim number and none rounds up.

| # | Forward-architecture item | Status | Blocking dependency / note |
|---|---|---|---|
| A | `ArchitectureSnapshot` carrier | PLANNED | Proposed home `crates/praxis-graphlaw/src/architecture/snapshot.rs`; zero commits, zero grep hits (Sec. 4, 9) |
| B | Truthful `SearchOutcome` algebra (Found/Bounded/Exhausted) | PLANNED | Motivated directly by the Exhausted-vs-Bounded conflation finding (Sec. 7); zero code |
| C | Dependency-footprinted semantic caching (`CacheEntry`/`DependencyFootprint`) | PLANNED | Capability fence: semantic caches MISSING — only content-hash receipts + one `moka` HTTP cache behind the optional `mcp` feature (`Cargo.toml:54,91`) |
| D | Datalog specialization behind a differential promotion gate | PLANNED | Capability fence: stratified semi-naive Datalog ALIVE, specialization itself MISSING (zero magic-set/adornment hits); the gate is an explicit blocking step, not a formality (Sec. 10) |
| E | `TraceEq`-guarded search reduction | PLANNED/UNKNOWN | Blocked on `TraceEq` itself — capability fence: **MISSING in this workspace**, contradicting an earlier design doc's ALIVE claim; disclosed, not silently resolved |
| F | Six-obligation cross-slice composition (`SlicePlan`) | PLANNED | Builds on `compose_two` (`crates/cng/src/bench/decomp/compose.rs:48`), itself PARTIAL — composed POWL is admissible-not-executable today |
| G | `PlanWitness`/`plancheck` verifier | PLANNED | Kernel-level proof authority explicitly DEFERRED/EXCLUDED by the ratified design; nearest existing kin `bcinr_pddl::execute_tape`, `validate_powl_store`, f25 receipt-replay (Sec. 7) |

## 1. Architecture summary

v26.7.13 is a hardening-and-disclosure release across eight independently shipped surfaces
(Themes A–H, table above) — no theme modifies the Chatman Engine S1–S6 pipeline or the
`A = μ(O*)` equation (`docs/CHATMAN_EQUATION.md`); Themes A, E, F, and G touch machinery
adjacent to μ's inputs and receipts (crown witnesses, hook actuation, `materialize` rollback,
coverage of receipt-adjacent replay paths) without changing the equation itself. Separately,
this document is the architectural record for a ratified-but-unbuilt Rust-only forward design:
a canonical `ArchitectureSnapshot` carrier, a truthful `SearchOutcome` algebra distinguishing
`Found`/`Bounded`/`Exhausted` (Sec. 7), dependency-footprinted semantic caching, Datalog
specialization gated behind differential promotion, `TraceEq`-guarded search reduction,
six-obligation cross-slice composition, and a `PlanWitness`/`plancheck` verifier with
kernel-level proof authority explicitly DEFERRED/EXCLUDED. Every forward item is PLANNED or
UNKNOWN (addendum table above); this ARD documents the design and its grounding in existing
code, not a delivered capability.

## 2. Components

**Shipped-theme components** (architectural detail behind the Claims Reconciliation table):

| Theme | Component | Location | Status |
|---|---|---|---|
| A | LOCAL/EXTERNAL crown-witness drivers | `crates/multifractal-workflow/src/{crown_local,crown_external}.rs` | PARTIAL |
| B | ~40 scattered fixes (mutex, determinism, refusal, injection) | see Claims row 2 evidence column | ALIVE per fix |
| C | `extra_ontologies` union + `lock:bool` opt-out | `crates/ggen/src/config.rs` | ALIVE |
| D | `togaf-adm-pack` + phase fixtures | `packs/togaf-adm-pack/` | ALIVE, scoped to increment 1 |
| E | SOC2 bench category + Mycin/Datalog roles | `crates/cng/src/bench/{soc2,soc2_growth}.rs` | PARTIAL, fenced |
| F | `TripleStore::materialize` checkpoint rollback | `crates/praxis-graphlaw/src/` (materialize path) | ALIVE |
| G | reachability/disclosure coverage commits | see Claims row 7 evidence column | ALIVE, narrow |
| H | broker-dispatch/reaction-dispatch/AtomVM transition catches | `apps/arazzo_runner/`, `apps/arazzo_atomvm/` | ALIVE, scoped |

**Proposed forward-architecture components** (PLANNED; full struct shapes in Sec. 4):

| Component | Proposed location | Builds on |
|---|---|---|
| `ArchitectureSnapshot` | `crates/praxis-graphlaw/src/architecture/snapshot.rs` | `ProfileSymbolTable`, `canonicalize_quads`, RDFC-1.0 N-Quads discipline |
| `RuleProgram` | `crates/praxis-graphlaw/src/architecture/` | today's `Rule`/`BodyLiteral`/`CompiledRule` (`rule.rs`) |
| `CacheEntry`/`DependencyFootprint` | `crates/praxis-graphlaw/src/architecture/` | `EngineProcessReceipt`'s 9-digest tag-prefixed `blake3_combined` scheme |
| `SearchLimits`/`SearchOutcome` | `crates/praxis-graphlaw/src/architecture/` (or `pddl-index`) | `CHATMAN_CONSTANT=8`, `DescentMeter`, `Pddl8` bounds |
| `SlicePlan` | `crates/praxis-graphlaw/src/architecture/` | `compose_two` (`crates/cng/src/bench/decomp/compose.rs`) |
| `PlanWitness`/`PlanCheckOutcome` | `crates/praxis-graphlaw/src/architecture/` | `bcinr_pddl::execute_tape`, `validate_powl_store`, f25 receipt-replay |

Module home rationale: `crates/praxis-graphlaw/src/architecture/{snapshot,identity,delta}` sits
below both `cng` and `multifractal-workflow` in the dependency graph, adds no new dependency
edges, and owns the primitives the carrier needs. A new leaf crate is warranted only if `ggen`
itself must import the carrier types — not the case for any item in the addendum table above.

## 3. Core invariant

The forward architecture, wherever it lands, must satisfy this repo's existing eight invariants
(`CLAUDE.md`) without exception; three are singled out here because the proposed design
introduces new surfaces that could otherwise erode them silently:

1. **Receipts computed, never asserted.** Any `SnapshotId`/`CacheKey` design must reuse the
   tag-length-prefixed `blake3_combined` discipline `EngineProcessReceipt` already uses across
   its 9 digests (one version tag per digest scheme; 11 distinct content-addressing schemes
   already inventoried, all BLAKE3) — or explicitly justify a new tag. Inventing a parallel
   hashing scheme is a Sec. 16 gate failure, not a style choice.
2. **No wall clock in hash/receipt paths.** `SearchLimits`/`SearchOutcome` and any cache
   eviction policy must express bounds as declared step/tick counts (`CHATMAN_CONSTANT=8`,
   `DescentMeter`, `TickBudget` — Sec. 7), never `SystemTime`/`Instant::now`.
3. **Byte-vs-semantic hashing must be stated honestly, not blurred.** `ggen` pack content
   hashes are BYTE-level (serialization-sensitive); `graph_hash` IS semantic (canonical
   N-Quads, bounded 5-iteration color-refinement bnode c14n); chatman receipts use RDFC-1.0.
   The `ArchitectureSnapshot` carrier's own hash must declare which regime it is in — it must
   not claim `ggen`'s input addressing is semantic when only the graph-state layer is.

The differential promotion gate (Sec. 10, addendum row D) is itself a corollary of invariant 5
(deterministic under fixed seed, no algorithmic surprises): specialization must not feed
planner/composer/authoritative caches until oracle-equivalent across cold/warm/incremental
execution and metamorphic reorderings — an untested specialization path silently promoted to
authority would be exactly the "algorithmic surprise" invariant 6 forbids.

## 4. Object model

All structs below are **proposed shapes, not implemented code** — PLANNED per the addendum
table. Each is annotated with the existing type(s) it builds on, per the grounding facts this
document is sourced from.

```rust
// PLANNED — crates/praxis-graphlaw/src/architecture/snapshot.rs
// Builds on: ProfileSymbolTable (chatman/triple8.rs, the closed-world interner),
// canonicalize_quads (`_:c14n{idx}` relabeling), EngineProcessReceipt digest #1's
// RDFC-1.0 canonical-N-Quads-hash-of-input pattern (chatman/engine.rs).
pub struct ArchitectureSnapshot {
    pub snapshot_id: Digest,        // tag-prefixed blake3_combined, reusing the 9-digest scheme
    pub symbol_table: ProfileSymbolTable,
    pub canonical_nquads: String,   // RDFC-1.0, sorted — never a struct dump
    pub graph_hash: Digest,         // semantic (color-refinement c14n), distinct from byte hash
}

// PLANNED — lowers today's Rule/BodyLiteral (rule.rs:7,30) into a typed program.
// Builds on: Rule { body: Vec<BodyLiteral>, head: Triple } (rule.rs:30, 421 lines total),
// CompiledRule's PatternStep/Selectivity ordering (rule.rs:132) as the pre-lowering baseline.
pub struct RuleProgram {
    pub rules: Vec<CompiledRule>,
    pub stratification: Result<Vec<Stratum>, StratificationCycle>, // typed, not bare String
}
// StratificationCycle replaces today's bare-String cycle error (Sec. 6) — the RuleProgram
// lowering is the proposed opportunity to type it, not a separate follow-on change.

// PLANNED — dependency-footprinted semantic cache entry.
// Builds on: EngineProcessReceipt's 9 constitutional digests + tag-prefixed blake3_combined
// root (chatman/engine.rs); today only content-hash receipts and one moka HTTP cache
// (Cargo.toml, `mcp` feature) exist — no semantic cache of any kind today (addendum row C).
pub struct DependencyFootprint {
    pub inputs: Vec<Digest>,        // the exact upstream digests this result depends on
}
pub struct CacheEntry<T> {
    pub key: Digest,                // reuses the same tag-prefixed blake3_combined scheme
    pub footprint: DependencyFootprint,
    pub value: T,
}

// PLANNED — truthful search-outcome algebra (Sec. 7 motivates this directly).
// Builds on: CHATMAN_CONSTANT: u64 = 8 (praxis-synthesis/src/budget.rs:28),
// DescentMeter (typed DescentBudgetExhausted; multifractal-workflow/crown_local.rs and
// others), Pddl8 n<=256 width + depth 64 (PDDL8_MAX_PLAN_DEPTH), max_hot_constraints<=8.
pub struct SearchLimits {
    pub max_depth: u32,             // aligns with PDDL8_MAX_PLAN_DEPTH=64, not a new constant
    pub max_ticks: u64,             // CHATMAN_CONSTANT-denominated, never wall-clock
}
pub enum SearchOutcome<T> {
    Found(T),                       // re-checkable — PlanWitness applies
    Bounded,                        // UNKNOWN by definition: search stopped, goal status unknown
    Exhausted,                      // trusted to the exact finite search actually performed
}
// Today: both over-depth and true no-plan drain to one Refusal::NoAdmissiblePlan /
// CngRefusal::PlanUnsolvable (Sec. 7) — SearchOutcome's whole purpose is to stop collapsing
// these two distinct claims into one bit.

// PLANNED — six-obligation cross-slice composition.
// Builds on: compose_two (crates/cng/src/bench/decomp/compose.rs:48), which exists today but
// produces a composed POWL that is admissible-not-executable (capability fence, addendum F).
pub struct SlicePlan {
    pub slices: Vec<SlicePlanEntry>,
    pub obligations: [ObligationCheck; 6], // six-obligation composition, per the ratified design
}

// PLANNED — plan-witness verifier; kernel-level proof authority DEFERRED/EXCLUDED.
// Nearest existing kin (none is a witness checker): bcinr_pddl::execute_tape (replays tape,
// sets a goal_reached flag), validate_powl_store (shape only), f25 receipt-replay (digest
// only, no semantic re-check).
pub struct PlanWitness {
    pub outcome: SearchOutcome<Digest>, // Digest references the plan/tape this witnesses
}
pub enum PlanCheckOutcome {
    Verified,                       // Found re-checked independently
    UnknownByDesign,                // Bounded — never asserted true or false
    TrustedToSearch,                // Exhausted — trusted to the exact finite search, not proved
    // No kernel-level proof variant exists here by design (DEFERRED/EXCLUDED).
}
```

## 5. Standing model

This ARD was authored from the ratified-design grounding facts and a targeted set of this-session
greps/`wc -l` checks against the live tree (Sec. 2, 4, 9) — it was **not** verified against a
freshly re-run `just standing` in this authoring session. Per `docs/standing/CLAUDE_CODE_POLICY.md`
("if they disagree, the index wins and the doc/comment is out of date"), `target/
praxis-standing/standing.json` and `docs/standing/REALITY_INDEX.md` are authoritative over any
standing claim in this document if the two diverge; this document does not itself claim a ladder
level for any v26.7.13 theme or for the forward architecture.

Standing-policy vocabulary for this and all v26.7.13 release docs: the ladder rungs are
DISCOVERED → BUILDS → TESTED → RECEIPTED → … (per-artifact, quoted from the compiled index, not
paraphrased); "production-ready" (or pilot/publish/publication-ready) is never used unscoped —
every readiness claim requires a stated scope (`ANTI-LLM-STANDING-001`). The forward-architecture
addendum table (top of this document) has no ladder entries at all, because no artifact exists
yet to rung — PLANNED/UNKNOWN is the correct and complete standing statement for those seven
rows, not a placeholder for a ladder level to be filled in later.

## 6. Rule model

Today's Datalog evaluator is the baseline the proposed `RuleProgram` lowering (Sec. 4) would
replace — stated honestly, not as a strawman:

- `Rule` is `{ body: Vec<BodyLiteral { negated, pattern }>, head: Triple }` (`rule.rs:7,30`,
  421 lines total) with `usize`-interned variables. Denial/consistency-check rules
  (`{ body } => false.`) reuse `Rule.head` with a reserved sentinel predicate
  (`DENIAL_HEAD_MARKER`, `rule.rs:45`) rather than a dedicated enum variant — a deliberate
  choice documented in-file to avoid touching ~18 existing `Rule`-literal call sites.
- `CompiledRule` (`rule.rs:132`) carries a static selectivity ordering
  (`Selectivity`/`PatternStep`, `rule.rs:78-122`) computed at load time, but this ordering is
  largely vestigial in practice: the live evaluator re-derives a greedy most-bound-first join
  order at runtime, with no statistics and no cost model informing either pass.
- Stratification uses Bellman-Ford relaxation (`datalog.rs`, 306 lines). The
  non-stratifiable-cycle refusal is a **bare `String`**, not a typed variant — the `RuleProgram`
  lowering proposed in Sec. 4 (`Result<Vec<Stratum>, StratificationCycle>`) is the opportunity
  to type this, not a separate follow-on change.
- Semi-naive delta evaluation is counter-range based over an append-only triple `Vec`. A
  `FactStore` delta/all struct exists but is test-only. Every `materialize()` call is a full
  fixpoint from scratch; `dred.rs` (delete/rederive, 246 lines) exists in the tree but is **not
  wired in** — there is no incremental evaluation today.
- Differential-gate precedent already exists and is the pattern any future differential
  promotion gate (Sec. 10, addendum row D) should extend: `datalog_stratification_fuzz.rs`
  (2000 proptest cases against an independent Tarjan-SCC oracle) and `n3_implies_fuzz.rs`. No
  existing test checks fact-order/rule-order permutation invariance — that is a genuinely novel
  gate component, not an extension of an existing one.
- The bench harness is `bencher`, not `criterion`; `materialize` benches already exist
  (`blue_river_dam`, `daily_standing`) and are the natural before/after baseline for any
  specialization work.

## 7. Planner domain

The strongest concrete motivation for `SearchOutcome` (Sec. 4, addendum row B) is a specific,
named conflation in the current planner surface, not a general architectural preference:

`PDDL8_MAX_PLAN_DEPTH` bounds search at depth 64; paths that exceed this bound silently
`continue` rather than being distinguished from a genuinely exhausted search. **Exhausted and
Bounded are not distinguished anywhere in this codebase today**: both drain into one
`NoAdmittedPlan` result ("bounded plan search exhausted without goal"), which `f08` further
collapses (alongside a dead-code `GoalNotReached` variant) into a single
`Refusal::NoAdmissiblePlan`, and `cng` collapses again into one `CngRefusal::PlanUnsolvable`.
`BoundExceeded` exists but covers only grounding count, pre-search — it says nothing about
whether the search itself ran to true exhaustion or hit the depth wall. This three-level
collapse (planner → `f08` → `cng`) is exactly the failure mode `SearchOutcome::{Found, Bounded,
Exhausted}` (Sec. 4) is designed to stop.

Supporting facts:

- Two BFS planner implementations exist — external `bcinr-pddl` 26.6.26 and a workspace
  `pddl-index` twin — structurally identical to each other, both subject to the same collapse.
- Frontier ordering IS total and deterministic (`BTreeSet` + FIFO, documented in-code); no
  checkpoint/resume capability exists.
- Reusable budget machinery `SearchLimits` (Sec. 4) should align with: `DescentMeter` (typed
  `DescentBudgetExhausted`) and `TickBudget` (`praxis-synthesis`). `ValidatedTickBudget` does
  **not** exist anywhere in this workspace and must not be cited as a precedent.
- No plan-witness checker exists today (motivating `PlanWitness`, Sec. 4, addendum row G).
  Nearest kin: `bcinr_pddl::execute_tape` (replays a tape, sets a `goal_reached` flag —
  execution replay, not independent verification), `validate_powl_store` (shape validation
  only), and f25 receipt-replay (digest match only, no semantic re-check).

Capability-fence standing for the two adjacent planner surfaces (verified this session against
the live tree per the ratified design's grounding, correcting an earlier design doc's optimism):
PDDL8 bounded planning is PARTIAL — bounds live in the external `bcinr_pddl` crate, and
depth-exhaustion is conflated with no-plan exactly as described above; PDDL→POWL projection is
also PARTIAL — 3–4 parallel implementations exist with zero cross-implementation differential
tests between them.

## 8. CLI architecture

No `plancheck` verb, and no CLI surface for any forward-architecture item, exists in this
session's search (`grep -rln "plancheck\|PlanWitness\|SearchOutcome\|ArchitectureSnapshot"
crates --include='*.rs'` — zero hits). Status: **PLANNED/UNKNOWN**. If and when a CLI surface is
added, the natural convention to extend is `cng`'s existing noun-verb clap structure (`crates/
cng`, v26.9.10) rather than a new dedicated binary — but no scaffolding for this exists yet and
none is asserted here.

## 9. File architecture

Existing files the forward architecture builds on (line counts confirmed this session via
`wc -l`):

```text
crates/praxis-graphlaw/src/rule.rs              421 lines  Rule, BodyLiteral, CompiledRule
crates/praxis-graphlaw/src/datalog.rs           306 lines  stratification (Bellman-Ford)
crates/praxis-graphlaw/src/dred.rs              246 lines  delete/rederive, present, NOT wired
crates/praxis-graphlaw/src/bindings.rs          226 lines  Binding::len() (open issue, Sec. 7 PRD)
crates/praxis-graphlaw/src/chatman/triple8.rs   482 lines  ProfileSymbolTable, Term8
crates/praxis-graphlaw/src/chatman/engine.rs   2174 lines  EngineProcessReceipt, S1-S6 pipeline
crates/praxis-synthesis/src/budget.rs           193 lines  CHATMAN_CONSTANT=8, TickBudget
crates/cng/src/bench/decomp/compose.rs                     compose_two (cross-slice precedent)
```

Proposed forward-architecture module (does **not** exist yet — confirmed this session via
`find crates/praxis-graphlaw/src -maxdepth 1 -iname "architecture*"`, zero results):

```text
crates/praxis-graphlaw/src/architecture/    PLANNED, zero files today
├── snapshot.rs   ArchitectureSnapshot carrier (Sec. 4)
├── identity.rs   dependency-footprint / cache-key identity primitives (Sec. 4)
└── delta.rs      RuleProgram / SlicePlan / PlanWitness lowering (Sec. 4)
```

Shipped-theme file surfaces are enumerated per-theme in Sec. 2's components table; not
duplicated here to avoid drift between the two sections.

## 10. Dataflow

No v26.7.13 theme touches the Chatman Engine S1–S6 dataflow (Sec. 1). The forward
architecture's proposed dataflow anchors to existing S1–S6 stages without altering them,
per the ratified design's amended delivery order:

1. **`ArchitectureSnapshot` carrier lands first** — the grounding primitive every later item
   depends on, conceptually adjacent to S1 `fetch_snapshot`'s canonical-hash-of-input pattern.
2. **`RuleProgram` lowering** — types today's bare-`String` stratification-cycle refusal
   (Sec. 6) as part of the same change, not a follow-on.
3. **Dependency-footprinted semantic caching** (`CacheEntry`/`DependencyFootprint`) — reuses
   the receipt digest tag-prefix discipline (Sec. 3); keys off whichever S2/S3 inputs a cached
   result actually depended on, per `DependencyFootprint`.
4. **Datalog specialization** — conceptually adjacent to S2 `apply_owl_closure`'s rule routing.
5. **Differential promotion gate — explicit blocking step 5 in the ratified design.**
   Specialization must not feed planner/composer/authoritative caches until oracle-equivalent
   across cold/warm/incremental execution and metamorphic reorderings (Sec. 3). This is a hard
   gate, not an aspiration: nothing downstream of it may consume specialized output until it
   passes.
6. **One deterministic best-first planner first** — the ratified design explicitly rejects
   standing up a simultaneous A*/HTN/regression zoo; a single planner ships before any
   trait-boundary abstraction is introduced, conceptually adjacent to S3 `generate_pddl_plan`,
   and is the first consumer of `SearchOutcome` (Sec. 4, 7).
7. **`TraceEq`-guarded search reduction** — blocked on `TraceEq` itself, which the capability
   fence found **MISSING in this workspace** (addendum row E), contradicting an earlier design
   doc's ALIVE assumption. This step cannot start until that gap is independently closed or the
   dependency is redesigned around.
8. **Six-obligation cross-slice composition** (`SlicePlan`) — builds on `compose_two` (Sec. 4),
   conceptually adjacent to S4 `admit_powl_trace`'s composed-tape admission.
9. **`PlanWitness`/`plancheck` as level-1 independent standing** — `Found` is re-checkable,
   `Bounded` is UNKNOWN by definition (never asserted true or false), `Exhausted` is trusted to
   the exact finite search actually performed (not proved); kernel-level proof authority is
   explicitly DEFERRED/EXCLUDED by the ratified design, not silently dropped.
10. **Trait frameworks only after two concrete implementations need the boundary** — no
    speculative `trait Planner`/`trait Cache` abstraction is introduced ahead of a second real
    consumer forcing the question.

None of the ten items above has landed code (Sec. 1, 2, 9); this section documents the intended
order, not completed work.

## 11. Design system

The forward-architecture vocabulary extends, rather than replaces, this repo's existing
legal-industrial register (admission, standing, receipt, refusal, replay — established in
`docs/releases/v26.7.6/ARD.md` Sec. 11, carried through v26.7.9's ARD Sec. 11): snapshot
(`ArchitectureSnapshot`), cache entry / dependency footprint, search outcome
(`Found`/`Bounded`/`Exhausted`), slice / obligation (`SlicePlan`), and plan witness / plancheck.
None of these terms displaces an existing one; `SearchOutcome` in particular is chosen precisely
because the existing vocabulary (`Refusal::NoAdmissiblePlan`, `CngRefusal::PlanUnsolvable`)
already collapses two distinct claims into one (Sec. 7) — the new vocabulary exists to restore
that distinction, not to relabel it. The no-overclaiming vocabulary itself
(ALIVE/PARTIAL/PLANNED/UNKNOWN/MOCKED, `.claude/rules/no-overclaiming.md`) is the design system
for every standing claim in this document, including the SOC2 fence's own restricted vocabulary
(evidenced / exception-identified / remediation-applied / evidence-bundle-assembled — never
compliant / passed the audit / SOC2-ready, Doctrine Sec. 6 of `PRD.md`), which this ARD treats
as a worked example of the same discipline applied to a shipped theme.

## 12. Demo architecture

No demo or example fixture exists for any forward-architecture item — PLANNED/UNKNOWN, same
disposition as the addendum table. For the shipped themes, the closest existing demo-shaped
artifacts are: the TOGAF Meridian increment 1 cycle (30-step ADM cycle, 10 POWL children,
Claims row 4) and the Solace Cloud / Arclight SOC2 case-study bundle (10/10 phases, 5/5 roles
Mycin/Datalog parity, 0 SHACL violations, 0 fence violations) — but the SOC2 bundle's numbers
were verified only against pre-rescale Stage-1 fixtures via an isolated scratchpad copy; the
live tree diverged mid-generation because of the Arclight rescale (Theme E), and this divergence
must be disclosed alongside the bundle, not silently dropped, per Claims row 5.

## 13. Market architecture

Architecture behind any external-facing claim about this release, scoped per
`.claude/rules/no-overclaiming.md`:

- **rwai-bench discipline** (`BENCHMARK.md`): "Rust counters are telemetry, never authority."
  ALIVE scope is a 10,000-worker depth-2 conformant run, replayed 3/3. Any 5M-worker claim is
  **UNVERIFIED** — generation is capped at 50,000 sets in this workspace. Any external
  reference to rwai-bench must use the modeled/derived measurement-class tags, never the
  headline number bare.
- **TOGAF "(v26.7.13)" module-doc stamp**: `togaf.rs`'s own doc comment stamps the full
  milestone version though only increment 1 has landed (Claims row 4) — any external claim
  must cite increment 1 specifically, not the doc-comment stamp.
- **SOC2 compliance fence** (Doctrine Sec. 6 of `PRD.md`): never "compliant," "passed the
  audit," or "SOC2-ready" — structurally enforced by `verify_no_compliance_or_opinion_effects()`
  and two adversarial mutants that both refused typed (Sec. 14).
- **`ggen` 26.7.13 parity**: ALIVE scope is `extra_ontologies` + `lock:bool` + the version bump
  (Claims row 3); the `EXPECTED_FACTORY_HEAD` pinning risk in `run-evidence-pass.mjs` remains
  open and must not be implied fixed by the parity claim.
- **Forward-architecture items**: no external claim may reference `ArchitectureSnapshot`,
  `SearchOutcome`, semantic caching, `TraceEq` reduction, `SlicePlan`, or `PlanWitness` as
  shipped capability — every one is PLANNED or UNKNOWN (addendum table).

## 14. Adversarial architecture

This release's adversarial-review mechanism is the recurring dogfood-audit cycle itself, not a
single external gate: workflow `wl1quccmk` independently found the `materialize()`
`verdicts`-not-restored bug and the `strip_comments` escaped-quote truncation bug, both fixed in
Theme F (Claims row 6); an earlier session's incorrect `true` LOCAL crown-witness reading was
independently re-audited down to the honest `false` (Theme A, Claims row 1); and the SOC2
structural fence (Theme E) was itself adversarially tested — two mutants renaming the effect
predicate to `audit-compliant`/`auditor-opinion-issued` both refused typed
(`CNG_R05 UnsupportedConstruct`) rather than silently passing (`docs/SOC2_TESTBED.md`).

For the forward architecture, the existing adversarial precedent any future differential
promotion gate (Sec. 3, 10) must extend is `datalog_stratification_fuzz.rs` (2000 proptest
cases against an independent Tarjan-SCC oracle) and `n3_implies_fuzz.rs`. The gap this precedent
does **not** yet cover — fact-order/rule-order permutation invariance — is the genuinely novel
adversarial-test component the gate requires; it does not exist today and is not claimed to.

## 15. Final-day outputs

| Output | Where | Status at authoring |
|---|---|---|
| Eight shipped-theme commits | see Claims Reconciliation table, Evidence column | cited, per-theme verified this milestone |
| `docs/releases/v26.7.13/PRD.md` | this directory | exists, DRAFT |
| `docs/releases/v26.7.13/ARD.md` | this directory | this document, DRAFT |
| `docs/releases/v26.7.13/RELEASE_CONTROL.md` | this directory | does not yet exist this session |
| Forward-architecture code | `crates/praxis-graphlaw/src/architecture/` | zero files, PLANNED (Sec. 2, 9) |
| Forward-architecture CLI surface | n/a | zero hits, PLANNED/UNKNOWN (Sec. 8) |
| Standing-index milestone representation | `target/praxis-standing/standing.json` | not freshly re-run this session (Sec. 5) |
| Crown-witness repair R8 | `docs/jira/v26.7.12/CROWN_STATUS.md` | unstarted (Claims row 1) |
| TOGAF increments 2/3 | tickets #100, #101 | zero commits, PLANNED |

## 16. Definition of done

1. Each of the eight shipped themes is independently gated per the Claims Reconciliation table
   above — its own commit and test evidence, no theme rounded up beyond its stated status.
2. Every claim in this document cites a specific file, test, or commit that exists; forward-
   architecture citations (Sec. 4, 6, 7, 9) were checked this session against the live tree
   (`wc -l`, targeted `grep`, `find`) rather than assumed from the source planning document.
3. No invariant of Sec. 3 is violated by forward-architecture code landed this release —
   trivially true because no such code exists yet (Sec. 1, 2, 9, 15); this is stated
   explicitly rather than left to be inferred.
4. Per `docs/standing/CLAUDE_CODE_POLICY.md`: this document was not verified against a
   freshly re-run `just standing` in this authoring session (Sec. 5) — the compiled
   `standing.json`/`REALITY_INDEX.md`, not this ARD, is authoritative if the two diverge.
5. Receipts remain computed and chained, never asserted (`EngineProcessReceipt`, unchanged by
   this release); the proposed `CacheEntry`/`ArchitectureSnapshot` design (Sec. 3, 4) commits
   to reusing this discipline, not inventing a parallel one.

**Supersession note:** the five points above are this section's own definition of "done" for the
eight-theme release this ARD otherwise documents. For release-readiness purposes specifically
(i.e., whether v26.7.13 may be dry-run published), the separate, later-landed v26.7.13 Dry-Run
Publish Definition of Done — 6 blocking gates, required falsifiers, and a final outcome
algebra (ALIVE/REFUSED/BOUNDED/UNSUPPORTED/INCONSISTENT) — supersedes this section's narrower
5-point list. That DoD's authoritative, evidence-cited verdict is REFUSED; see
`DRY_RUN_PUBLISH_VERDICT.md` for the full gate-by-gate breakdown. This section's 5 points remain
correct for what they were written to gate (the eight themes) and are not retracted; they are
simply not sufficient, on their own, to call v26.7.13 dry-run-publish-ready.

**Hard exclusions** (not gate failures — scope boundaries, matching `PRD.md` Sec. 12): kernel-
level proof authority for `PlanWitness` (DEFERRED/EXCLUDED by the ratified design, Sec. 4, 10);
`TraceEq`-guarded search reduction, blocked on `TraceEq` itself being MISSING in this workspace
(Sec. 10 step 7); TOGAF increments 2 and 3 (#100, #101); crown-witness repair R8; the
`Binding::len()` HashMap-order column-length issue (`bindings.rs:22`); 67 pre-existing `cng`
clippy findings; the pinned `EXPECTED_FACTORY_HEAD` risk in `run-evidence-pass.mjs`; tier2/tier3
residual `knowledge_hooks_e2e` failures. Full itemization of these lives in `PRD.md` Sec. 12,
not restated here to avoid drift between the two documents.

Anything short of the five points above stays UNKNOWN in `RELEASE_CONTROL.md`. That file, not
this ARD alone, is the single control surface for what "done" means for v26.7.13.
