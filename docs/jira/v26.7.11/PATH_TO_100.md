# Path to 100% — v26.7.11 Production Readiness

Milestone: v26.7.11. Synthesis of 8 read-only rail explorations conducted 2026-07-11,
session window ending ~23:02 PDT. Every figure below is cited to a source rail
exploration and, transitively, to a file:line or command+output that exploration ran.
No claim here was re-derived independently by this synthesis — where two explorations
disagree (the Erlang broker fix timeline, primarily), both timestamps are stated rather
than resolved by guessing.

Last Updated: 2026-07-11

## 1. Headline

No rail in this milestone is at 100% by the falsifiable bars each exploration defined
for its own scope. The gap is not evenly distributed and is not primarily "more logic
needs to be written" — most of the logic named in PROJ-750 through PROJ-787 already
exists, is genuinely tested, and passes. The gap consists of six distinct, concrete
things:

1. **Production reachability ("islands").** This is the single most repeated finding
   across all 8 explorations, independently re-derived each time: real, tested logic
   with zero callers outside its own test file. Confirmed present in Rail A/B
   (`admit_transition_with_external_cut`, 5 call sites, all tests), Rail F/G (the
   entire OTLP→RDF→OCEL→measurement chain, `admitted_spans_to_trig` has zero callers
   anywhere), the closure/compensation/N3/GraphLaw-authority group (four separate
   modules, each independently zero-callers), and the refusal catalog
   (`OTP_ATOMVM_SEMANTIC_DRIFT`, `COMPENSATION_REQUIRED`'s trigger, `REPLAY_DIGEST_MISMATCH`).
   No rail closes this gap by writing more logic — each needs exactly one real,
   non-test entry point wired to logic that already works.
2. **The Verification Ladder (PROJ-792–795) is effectively unstarted.** 0 of 10 chaos
   modes, 0 of 6 stress dimensions, 0 of 9 separately-reported benchmarks, 0 of 13
   verifier-report fields exist as real, run artifacts. This is not a same-session
   remaining item — see §11.
3. **Two items require genuinely new cross-language/cross-runtime infrastructure**,
   not more wiring: the AtomVM NIF-loading incompatibility (Rail E, §5) and the
   Erlang→`praxis-graphlaw` SHACL bridge for `RETURN_SEMANTIC_REFUSED` (§8). Both are
   correctly out of proportion to the tickets that currently reference them.
4. **The refusal catalog is fragmented across three uncorrelated taxonomies**
   (`chatman::abi::Refusal` in Rust/praxis-graphlaw, `CoreError` in Rust/praxis-core,
   Erlang atom codes in `apps/arazzo_runner`), only one of which has a
   completeness-checking mechanism at all (`ALL_REFUSAL_NAMES`, currently 31 of 46
   `Refusal` variants — 67%). "Wire the full PRD §18 catalog" cannot be satisfied by
   extending one array; this needs an explicit scoping decision (§8).
5. **A live reproducibility gap**: every green Rust-side test result cited by Rail A/B
   this session depends on an uncommitted working-tree diff in the sibling repo
   `/Users/sac/wasm4pm-compat`. No CI run, and no second machine, would currently
   reproduce any of it (§2).
6. **The three Erlang broker severe defects** (return-path wiring, token forgery,
   TOCTOU dedup race) were found open by five of the eight explorations checking the
   file between 22:41 and 22:57, and found fixed and independently
   re-verified (`rebar3 eunit --module=arazzo_runner_broker_test` → 17/17,
   23:02:12) by the exploration that checked latest. Treat the fix as landed per the
   most recent evidence, but re-run the three greps in §3 before relying on this if
   more time has passed — the file was under active, fast-moving edit all session.

Set against that gap, the concrete progress that is real and independently verified
this session: Rail H shipped all 9 PRD-named Lean theorem targets with zero
`sorry`/`axiom`/`admit` and a passing `lake build` (§6) — the most complete
single-ticket-set deliverable of the eight rails. Rail A/B, Rail E's differential
corpus, and the closure-law/N3-quarantine/GraphLaw-authority modules are each real,
tested libraries; none is yet a production system by its own rail's definition of
"production."

## 2. Rail A/B — Chatman Engine External-Cut Projection (PROJ-750–754, 796)

Call chain: `admit_transition_with_external_cut` → `ExternalCutCompiler::compile` →
`render_and_compile` (SPARQL → Tera → Arazzo parse/resolve/lower/normalize/compile →
WASM) → sealed digest #10 → replayable.

### 2.1 100% definition

Six independently falsifiable conditions, all required simultaneously:

| # | Bar | Falsifier |
|---|---|---|
| a | Reachability | ≥1 non-test call site of `admit_transition_with_external_cut` reachable from a `[[bin]]` target or real CLI verb |
| b | Coverage ≥90%/85% | `just coverage` (tarpaulin) on `chatman/{engine.rs,powl_projection.rs}`, `praxis-core/src/arazzo.rs`, `wasm4pm-arazzo/src/{lower,temporal,compile}.rs` |
| c | Determinism, 5x | 5 consecutive identical runs of digest #10, byte-identical (this repo's own stated bar; PROJ-796 ran only 3) |
| d | Benchmark | a `cargo bench` target measuring `render_and_compile`/`compile_to_wasm` end to end, with an O() bound documented |
| e | Clippy clean, scoped | `cargo clippy -D warnings` exits 0 for every crate in the chain, isolated from unrelated shared-dependency debt |
| f | Replay/tamper detection | a corrupted manufactured-Arazzo artifact replayed against a recorded digest #10 fails loud via `ReplayMismatch::ExternalCut`, never silently passes |

### 2.2 Current delta (live-verified 2026-07-11, this pass)

| Bar | Status | Evidence |
|---|---|---|
| a Reachability | not met | `grep -rn "\.admit_transition_with_external_cut("` → 5 hits, all in `praxis-core/tests/rail_ab_external_cut_wiring.rs` and `praxis-graphlaw/src/chatman/engine_test.rs`. Zero `[[bin]]` targets in `praxis-core`/`praxis-graphlaw`/`wasm4pm-arazzo`. `cng`/`praxis-l4` mention `ChatmanEngine` only in doc comments. |
| b Coverage | not measured | No `just coverage` run this milestone for these 4 files. |
| c Determinism | partial | 3x done for `rail_ab_external_cut_wiring`, not this repo's own 5x bar. |
| d Benchmark | not met | Zero bench targets anywhere reference `admit_transition_with_external_cut`, `project_and_compile`, `render_and_compile`, or `ChatmanRailAbCompiler`. PROJ-794 (§20 benchmark) is `BLOCKED`. |
| e Clippy | partial | `wasm4pm-arazzo` clean (0 warnings). `praxis-core` clippy fails only because it pulls in `praxis-graphlaw`'s lib, which carries 52 pre-existing clippy errors in `hooks/`, `shacl/`, `owlrl/`, and a sibling repo — zero errors in `chatman/{engine.rs,powl_projection.rs}` itself. No scoped `praxis-core-clippy-libtests` recipe exists to isolate this. |
| f Replay tamper detection | met | `ReplayMismatch::ExternalCut` real at `engine.rs:499-509`/`:980-998`, reached via `verify_replay_with_external_cut` (`:968`); closed this session, adversarially re-verified, 2 new tests. |

Test suites live-verified this pass, all green, 0 failed: `praxis-core-test` (71 lib +
55 integration across 6 suites), `wasm4pm-arazzo-test` (55 lib + 13 integration),
`powl2-decompose-test` (21 lib + 7 integration).

**Reproducibility gap**: `/Users/sac/wasm4pm-compat` — the path dependency every
green result above depends on — is 1 commit ahead of `origin/main` with an
**uncommitted** working-tree diff (257 insertions/108 deletions across `Cargo.toml`,
`src/lib.rs`, `src/petri.rs`, `src/models_hash.rs`, `tests/arazzo_roundtrip.rs`,
`wasm4pm-core/src/{models,pddl}.rs`). No CI and no clean checkout elsewhere would
currently reproduce a single green result cited above.

**Net status**: Rail A/B is real and tested as a library (every stage is genuine,
independently tested, and live-passing) and partial as a production system —
precisely 5 test-only call sites, zero production ones, resting on an uncommitted
sibling-repo dependency.

### 2.3 Sequenced remaining work

1. **W1 — Add a real production entry point** (S). New `praxis-core/src/bin/`
   CLI or wire `cng`'s existing `[[bin]]` to actually call the chain instead of only
   mentioning it in doc comments. Closes bar (a). The trait seam
   (`ExternalCutCompiler`/`ChatmanRailAbCompiler`) already exists and is tested — this
   is CLI wiring around working logic.
2. **W2 — Commit the `wasm4pm-compat` fix** (S). No downstream dependency itself, but
   blocks reproducibility of every other item's verification. Highest-leverage single
   item in this rail.
3. **W3 — Add a scoped `praxis-core-clippy-libtests` recipe** (XS), mirroring the
   existing `praxis-graphlaw-check-libtests` pattern, isolating Rail A/B's own files
   from the 52 pre-existing unrelated errors. Closes bar (e) without absorbing that
   backlog.
4. **W4 — Coverage measurement + gap-closing** (M). Baseline can run today;
   meaningful gap-closing depends on W1 landing first so numbers aren't test-tautological.
5. **W5 — Benchmark suite for Rail A/B's compile cost** (S), sequenced after W2 so
   the number is citable against a reproducible dependency state. One of PROJ-794's 9
   named benchmarks, done here as a strict subset.
6. **W6 — 5x determinism re-run** (XS), after W2, plus fixing the
   `just praxis-core-test <substring>` filter (currently matches by test *function*
   name, not binary — `rail_ab_external_cut_wiring` as a filter matches 0 of 3 tests).
7. **W7 — Verifier Report Rail A/B fields (PROJ-795)** (out of this rail's own scope,
   flagged not silently absorbed) — depends on W1 and on PROJ-791–794/770, all
   currently `BLOCKED`/`PARTIAL`.

## 3. Rail C/D — Erlang OTP Broker & Workflow (PROJ-755–758)

### 3.1 100% definition

Three severe defects from `ADVERSARIAL_DOD.md`'s Tier-1 finding, restated as
falsifiable tests:

1. **Return-path wiring**: an EUnit test drives a real `arazzo_runner_workflow`
   process only through its external event API (never calling
   `admit_return/3` from the test body) and observes the join-successor genuinely
   dispatch as a consequence — not by inspecting the broker ledger, which proves
   nothing about the workflow itself.
2. **Token security**: a negative test recomputes what `make_token/1` would produce
   from only publicly-observable values (`WorkflowId`, `StepId`,
   `IdempotencyKey`/`CorrelationId`) and asserts it does **not** equal the real minted
   token — requires a value never logged or returned in any refusal context (a
   per-node secret, or server-side random tokens).
3. **TOCTOU dedup atomicity**: ≥50 concurrent processes dispatch with an identical
   `{WorkflowId, StepId, IdempotencyKey}`; exactly one reaches the I/O side, and no
   ledger entry is left `dispatch_failed` after having reached `actuated`. Requires a
   single atomic `ets:insert_new/2`, not lookup-then-insert.

Explicitly out of scope for "100%" on these 3 defects: `RETURN_SEMANTIC_REFUSED` and
4 of the 5 `?UNENFORCED_PREACTUATION_CHECKS` — no Erlang↔GraphLaw bridge exists (see §8).

### 3.2 Current delta — timeline-sensitive, both snapshots given

**Earlier snapshot (five explorations, checks between 22:41 and 22:57)**: all three
defects open. `admit_return/3` called from nowhere but its own test file; dedup still
a plain `ets:lookup` + separate `ets:insert`; `make_token/1` still unsalted
`sha256(workflow_id|step_id|idempotency_key)`, explicitly documented in-file as
intentional ("content-addressing, not secrecy").

**Later snapshot (rail-h, checked 23:02:12, latest evidence in this synthesis)**: all
three fixed and independently tested:
- Return-path: `admit_return/3` now called from inside `do_dispatch/6`
  (`arazzo_runner_broker.erl:376`) right after I/O capture.
- Token security: `make_token` now mixes in `broker_secret()` — a 32-byte
  `crypto:strong_rand_bytes/1` secret generated once per node into `persistent_term`
  (`:778-783`).
- TOCTOU race: dedup now `ets:insert_new(arazzo_broker_dedup, {DedupKey,
  DispatchToken})` (`:249`), a single atomic CAS.
- `rebar3 eunit --module=arazzo_runner_broker_test` → **17 tests, 0 failures**
  (23:02:12), including one test per defect
  (`test_full_dispatch_correlation_return_round_trip`,
  `test_actuation_token_requires_server_secret`,
  `test_concurrent_duplicate_dispatch_claims_exactly_once`).

**Treat the fix as landed**, but this was a targeted confirmation, not a full
multi-app re-audit (Tier-2 findings such as unused `add_documents_par` were not
re-checked). Re-run the three greps below before further reliance:

```
grep -rn "admit_return" apps/*/src/*.erl | grep -v arazzo_runner_broker.erl
grep -n "insert_new" apps/arazzo_runner/src/arazzo_runner_broker.erl
grep -n "make_token\|broker_secret" apps/arazzo_runner/src/arazzo_runner_broker.erl
```

**New, orthogonal finding (unaffected by the fix)**: the git-hygiene remediation
`ADVERSARIAL_DOD.md` marked "DONE" for PROJ-755/756/757/758/760 does not hold on
re-check — `.gitignore`'s `_build/`/`*.beam` lines are unstaged working-tree content,
and `git ls-files | grep -c '_build/.*\.beam$'` returns 9 tracked compiled-artifact
paths. Every `rebar3 eunit` run this session, including the one verifying the 3
defect fixes, ran against a dirty `git status`.

### 3.3 Sequenced remaining work

Given the fix's apparent landing, remaining work shifts from "fix the 3 defects" to
"verify and harden them," plus the disclosed adjacent gap:

1. **Re-confirm the 3 fixes with a fresh timestamp check** before any downstream rail
   (E, refusal catalog, verification ladder) depends on broker behavior.
2. **Add the falsifiable tests from §3.1 verbatim** if not already exactly matching
   what landed (the 3 named tests above appear to satisfy this, but were not
   independently authored against §3.1's bar by this synthesis).
3. **Complete the git-hygiene remediation**: stage the `.gitignore` edit, `git rm
   --cached` the 9 tracked `.beam` paths, re-verify clean `git status` across ≥3
   consecutive `rebar3 eunit` runs — the bar `ADVERSARIAL_DOD.md` itself already set.
4. **Re-run PROJ-761's golden digests** (Rail E dependency, §4) once the broker fix is
   confirmed stable — the corpus's OTP path drives real broker dispatch, and a broker
   semantics change is exactly the kind of thing that silently invalidates a pinned
   golden digest.
5. **Update `tickets/index.md`/`ADVERSARIAL_DOD.md`** only after 1–4 are independently
   re-verified this session, not carried forward from a prior agent's self-report.

## 4. Rail E — AtomVM Wrapper & Differential Conformance (PROJ-760–762)

### 4.1 100% definition

The PRD names 9 reaction-event classes, but they split structurally: **Group A** (4
labels — `result`, `timeout`, `child_complete`, `child_refused` — that reach
`air_core:transition/2`, which itself has only 2 pattern-matched clauses, so Group A
is really "2 underlying shapes reached by 4 labels") and **Group B** (5 classes —
`start`, `dispatch_ready`, `acknowledgment`, `retry_due`, `admission_result` — that
never reach `transition/2` at all, confirmed by the module's own complexity comment).
The AtomVM-side wrapper has **no reaction-vocabulary layer**, so Group B classes have
no AtomVM-side representation to differentially compare against under the current
architecture — a literal 9/9 corpus is structurally impossible today, not merely
unbuilt.

- **Tier 1 (achievable today, corpus-authoring only)**: all 4 Group A labels, each
  positive and negative, across ≥2 topology shapes. Today: 1 shape, 2 of 4 labels
  (`result`, `timeout` only).
- **Tier 2 (architecture decision required first)**: is Group B in scope for Rail E
  equivalence at all? Neither answer is currently written down anywhere.

Separately, and more fundamental: **`air_core` loads a compiled Rust NIF via
`erlang:load_nif/2`** (`air_core.erl:1-2,104-112`), on the hot path of every single
transition (`eval_expr_nif`, called by `bind_outputs/3`, called by every `transition/2`).
AtomVM's supported extension model requires Nifs/Ports to be **statically compiled
into the VM binary at build time** — it does not support `erlang:load_nif/2`-style
dynamic loading at all (confirmed against AtomVM's own build docs and `nifs.c`
source). **The current code, as it stands, cannot load on real AtomVM** — a hard
code-compatibility incompatibility, not a tooling-provisioning gap. This is materially
stronger than PROJ-760's own disclosure ("no AtomVM runtime installed... out-of-scope
future work"), which frames it as tooling rather than code incompatibility.

- **Option A — real target in scope**: a pure-Erlang `eval_expr` fallback (smaller
  lift, new determinism-parity burden) or a statically-linked AtomVM-native port
  (larger lift, real toolchain + `just atomvm-build`).
- **Option B — real target explicitly out of scope for v26.7.11**: requires narrowing
  the PRD's unconditional `SHALL` (`PRD.md:67,427`) with an explicit, cited scope note
  — does not currently exist anywhere.

Neither is decided in writing. Corpus-breadth 100% (Tier 1/2 above) is well-defined
regardless of this decision; runtime-target 100% is undefined until it is made.

### 4.2 Current delta

- PROJ-760's delegation-facade claim holds (`arazzo_runner.erl` forwards to
  `arazzo_atomvm_workflow`, which calls `air_core:new/1`/`transition/2` directly). But
  zero AtomVM SDK/build integration exists anywhere in the tree; the NIF
  incompatibility above is undisclosed in `PROJ-760.md`; `Commands` from
  `transition/2` are computed but discarded on this path — PROJ-761 works around this
  via `erlang:trace/3` rather than fixing the wrapper (disclosed, but the equivalence
  proof observes the wrapper from outside, not through its own output surface).
- PROJ-761's corpus is real and non-tautological (`compare_four_dimensions/2` asserts
  genuine cross-path equality), but covers 1 topology and 2 of 4 Group-A classes —
  `child_complete`/`child_refused` are zero-covered despite already having a real,
  exploitable translation path (`handle_reaction/3:428-437`). Cheapest concrete gap
  to close.
- PROJ-762's refusal logic is real (`observe/2`, `compare_four_dimensions/2`,
  `first_mismatch/1`, correct priority ordering), but `OTP_ATOMVM_SEMANTIC_DRIFT`
  exists **only inside the test file** — zero production `src/` module exports it.
  Same island shape as the broker's pre-fix `admit_return/3` gap. Undisclosed in
  `PROJ-762.md`.
- Two documentation-staleness gaps: `tickets/index.md:175-177`'s Rail E summary is
  stale (contradicted by the same file's own PROJ-760/761/762 rows); `PROJ-762.md`
  itself still says `Status: PLANNED`, not synced with `index.md`'s ALIVE row.
- If/when the broker fix (§3) is confirmed stable, PROJ-761's golden digests should be
  re-run — they were pinned against pre-fix broker dispatch behavior.

### 4.3 Sequenced remaining work

1. **Resolve the AtomVM runtime-target scope question in writing** (Option A vs B) —
   a decision, zero code cost, gates step 2.
2. **If A**: build the smallest real target (pure-Erlang `eval_expr` fallback
   preferred over a full native port) plus a `just atomvm-build`/`atomvm-test`
   recipe. **If B**: write the narrowing scope note and cite it from `PROJ-760.md`.
3. **Extend PROJ-761's corpus to `child_complete`/`child_refused`** — lowest risk,
   immediately actionable, no architecture change (S).
4. **Decide and document Group B's scope** — out-of-scope-with-reason, or scope the
   materially larger AtomVM-side reaction-vocabulary layer.
5. **Move `OTP_ATOMVM_SEMANTIC_DRIFT`/`observe`/`compare_four_dimensions` into a
   production `src/` module** with ≥1 real non-test caller, matching the bar already
   set for `admit_return/3`; wire into PROJ-786's refusal catalog.
6. **Multi-topology stress** (fan-out/depth, concurrent AND-joins, duplicate-delivery
   interaction) — after 3–4, feeds PROJ-792's chaos suite.
7. **Fix the two stale doc items** (§4.2) — trivial, zero code risk.
8. **Re-verify PROJ-761's golden digests** once the broker fix (§3) is confirmed
   stable.

## 5. Rail F/G — OTLP→RDF→OCEL→PROV-O→Measurement (PROJ-763–767)

### 5.1 100% definition

Five falsifiable criteria: (a) every function in
`otel_rdf::admit → otel_ocel::project_otel_to_ocel → otel_receipt::receipt_otel_to_ocel
→ measurement::{compute_execution_measure,build_measurement_profile,project_measurement_profile}`
has ≥1 call site reachable from `fn main`/a `[[bin]]`/a `#[verb]` handler; (b) a single
real command leaves both `G_RESULT`/`urn:graph:results` and
`G_RECEIPT`/`urn:graph:receipts` non-empty; (c) each of the 11 PRD-named
`DeclaredProcessScale` variants has either a real query+test or an individually
distinct, falsifiable refusal reason; (d) the composed PRD §16 pipeline (`μ_x` →
`Z(q,ε)` → `τ(q)` → `D(q)` → `f(α)`) runs as one chain at least once; (e) a typed
`MeasurementStanding` enum with the PRD's exact 7 values is constructed from a real
computed profile.

None of (a)–(e) is met today.

### 5.2 Current delta

- **(a) Reachability — worse than a two-hop chain, it's a closed loop.**
  `otel_rdf::admit` → `project_admitted_spans` → `admitted_spans_to_trig`, which has
  **zero callers anywhere**. `otel_ocel::project_otel_to_ocel` → `receipt_otel_to_ocel`,
  zero non-test callers. `measurement::{compute_execution_measure,
  build_measurement_profile,project_measurement_profile}`, zero non-test callers.
  `bench::multifractal` is a **private** module (`mod multifractal;`, not `pub mod`),
  zero callers outside its own test. The one real `[[bin]]` (`otel-live`) never
  imports any of these modules. None of the 18 `#[verb]` handlers in `main.rs`
  reference them. The Round-6 `ADVERSARIAL_DOD.md` finding that this was "resolved by
  PROJ-764" is confirmed still open — the chain calls itself in a circle, never
  reached from `main.rs`/`pipeline.rs`/`runner.rs`/`otel-live.rs`.
- **(b) Both graphs — real quad-producing code, both populated in zero real runs.**
  `receipt_otel_to_ocel` genuinely writes PROV-O quads into `RECEIPT_GRAPH_IRI`, but
  has zero non-test callers. `project_measurement_profile` genuinely turns a computed
  profile into `G_RESULT` quads, but its own doc comment discloses it doesn't insert
  into a store — no production code even attempts that materialization step.
- **(c) 11 scales — 3 real (`Workflow`, `Activity`, `ObjectCentricAggregationLevel`),
  8 refused with individually distinct, verified-genuine reasons** (each a specific
  missing OTLP attribute or architectural boundary, not a generic placeholder) —
  confirmed matching `index.md`'s PROJ-766 entry exactly, no drift.
- **(d) New finding**: `measurement.rs`'s own module doc discloses PROJ-766 and
  PROJ-767 are "two independent halves" and wiring them is "a distinct,
  not-yet-scoped follow-up." Confirmed by grep: zero cross-references either
  direction. The estimator's only real data point
  (`track2b_real_workday_tape_ops_measurement`) is fed by a synthetic tape-ops proxy,
  never by `μ_x` from real admitted `G_OCEL` evidence. PRD §16's literal pipeline has
  never executed as one chain, in test or production — disclosed in source, but not
  named as a top-level gap in `index.md`/`ADVERSARIAL_DOD.md`.
- **(e) No typed standing enum exists** — zero hits for
  `DECLARED|PARTIAL_ALIVE|BUILD_BROKEN|MeasurementStanding` outside doc-comment prose.
- **Reconfirmed safety defect**: `otel_ocel::graph_content_digest` passes an
  unrestricted runtime `&str` into `named_graph`'s `.expect(...)` — a malformed
  `graph_iri` panics rather than returning `CngRefusal`. Harmless only because nothing
  outside tests calls it today; step 1 below makes this reachable for the first time.

### 5.3 Sequenced remaining work

1. **Add one real CLI entry point calling the whole chain** (M) — closes (a) and (b)
   simultaneously: spans → `project_admitted_spans` → `G_OTEL` →
   `project_otel_to_ocel` → `G_OCEL` → `receipt_otel_to_ocel` → `G_RECEIPT` →
   `build_measurement_profile`+`project_measurement_profile` → `G_RESULT`.
2. **Fix `graph_content_digest`'s panic-on-external-input** (XS) — before step 1 makes
   this reachable from a live entry point for the first time.
3. **Wire `measurement`'s real `μ_x` output into `bench::multifractal`'s estimator**
   (S) as an alternative mass source, run the estimator against it once — closes (d).
4. **Add the typed `MeasurementStanding` enum** (XS), constructed from whichever of
   1/3 ran — closes (e).
5. **Stabilize the Track2b output location per PROJ-797** (S, lower priority) — a real
   receipted-path win but doesn't move any of §5.1's criteria.
6. **Out of this milestone's control**: revisiting whether any of the 8 refused
   scales can gain a real data source requires new OTLP producer attributes in
   `registry/otel/praxis-events.yaml` — a Weaver-registry change, not a `cng`-side
   change. Correctly out of scope for PROJ-763–767 as drafted.

## 6. Rail H — Lean/Lake Formal Standing (PROJ-768–770)

### 6.1 100% definition

Not "all Lean proofs done" — 8 specific commands and pass conditions:

| # | Criterion | Command | Pass condition |
|---|---|---|---|
| a | 9/9 PRD targets have a real theorem | source read | every target has ≥1 `theorem`, not just `def` |
| b | Whole package builds | `lake build` | exit 0 |
| c | Zero sorry/admit/unauthorized-axiom in the 9 | `praxis-l4 no-sorry --root .../V26711` | `finding_count: 0` |
| d | Each of the 9 kernel-verifies | `praxis-l4 verify --lake-env --root <pkg>` | every file `verified`, zero `kernel_rejected` |
| e | Declaration index has no gaps | `index-build` + `report` via `build_with_root` | `missing_files` empty for all 9 |
| f | Negative fixtures exist and are correctly rejected | audit run against 3 fixtures | all 3 flagged, proving the audit isn't vacuous |
| g | Verifier Report carries a real Lean/Lake field | PROJ-795, once it exists | sourced from (b)–(e), not hand-written |
| h | Scope of any "100%" claim stated | — | 71 pre-existing thesis-corpus axioms classified, or claim explicitly scoped to the 9 v26.7.11 targets only |

### 6.2 Current delta (live-verified this session)

**(a), (b), (c), (d) are met today for the 9 milestone targets** — the state has
moved materially since the pre-dispatch snapshot ("Zero existing .lean content
matches any of these 9 targets", PROJ-769 `PLANNED`).

- All 9 targets landed (`tools/paper-factory/lean-lake/Praxis/Milestone/V26711/`, 10
  files including a shared `ClosureModel.lean`, still untracked in git), each
  self-labeled "Target N of 9," 1:1 against `PRD.md:1035-1043`, each citing the exact
  PRD range and source file:line it models. Real theorems with real tactics
  (`simp`, `rfl`, structural induction, `Finset`/`WellFounded`/`Prod.lex`), not
  `def`-only scaffolding.
- Zero `sorry`/`axiom`/`admit` confirmed three independent ways: targeted grep (0
  hits excluding false positives), a live `no-sorry` run (`finding_count: 0`), and
  10 non-trivial `.olean` artifacts (33–389 KB) timestamped after every source
  edit, meaning a real `lake build` succeeded.
- 4 files with real Mathlib/cross-file imports individually re-verified live:
  `lake env lean` on each → exit 0, empty output.
- A related fix landed: `detect_lake_env()` auto-detects a Lake-managed root, OR'd
  into `--lake-env` inside `run_cli`'s verify handler, closing the documented
  `lake_env`-defaults-false footgun. 4 new tests, live-run: 4/4 passed.

**(e), (f), (g), (h) do not hold**, plus two new findings from this session:

- The `detect_lake_env` fix only lives in `run_cli()` (`standalone-cli` feature) —
  the **default** build path (`verbs/l4.rs` → `cli::verify()` directly) still takes
  a raw `lake_env: bool` with no fallback.
- **New**: `cli::verify()`'s file walk has zero exclusion for `.lake/`. The package
  now contains 9,542 total `.lean` files, of which only 201 are this repo's own
  corpus — the remaining 9,341 are vendored Mathlib/Batteries/Aesop under
  `.lake/packages/`. A verify run against the actual package root (required for
  imports to resolve) also attempts `lake env lean` on all 9,341 dependency files.
  Live-observed: killed after ~6 minutes at 61/201 own-corpus files (48 verified, 11
  `axiom_unauthorized`, 2 `kernel_rejected` — roughly consistent with PROJ-768's
  disclosed 71-axiom baseline). Unbounded once it reaches `.lake/packages/mathlib`.
  Directly blocks a bounded-time Lean/Lake field for PROJ-795.
- `cli::report()` still calls root-less `VerificationReport::build`, never
  `build_with_root` — `missing_file_records()` remains a dead end.
- RDF declaration index has zero records for any of the 9 new targets.
- Zero negative fixtures exist anywhere for PROJ-770.
- `justfile` still has no Rail-H/Milestone-specific recipe.
- Two stale doc comments: `rail_h_existing_corpus_audit.rs`'s header still says
  PROJ-769 is "deliberately not attempted here"; `tickets/index.md`'s PROJ-769 row
  still says `PLANNED` — both now false.

### 6.3 Sequenced remaining work

1. **Fix `cli::verify()`'s `.lake/` scope bug** (S) — exclude `.lake/` from the
   `WalkDir` loop. Without this, no bounded-time full-package verify run is
   possible, blocking step 7.
2. **Move `detect_lake_env` into `cli::verify()` itself** (XS) — so both entry
   points benefit, not just `standalone-cli`.
3. **Wire `cli::report()` to `build_with_root`** (XS).
4. **Add a justfile recipe scoped to `Praxis/Milestone/V26711/`** (XS).
5. **Extend `docs/thesis/rdf/corpus.ttl`** with `math:Statement` declarations for
   the 9 targets (S), then run `index-build` — closes criterion (e).
6. **Build PROJ-770's negative fixtures**: one `sorry`-laden proof, one
   unauthorized-`axiom` proof, one PID-dependent identity claim (S).
7. **Run the now-unblocked (post step 1) full corpus verify**, classify each of the
   71 pre-existing unauthorized axioms — required for an unscoped "100%" claim to be
   honest (M).
8. **Wire a Lean/Lake build-status field into PROJ-795** once it exists, sourced from
   1–7's real output.
9. **Fix the two stale doc comments** (XS).

## 7. Closure, Compensation, N3 Quarantine, GraphLaw Authority (PROJ-759, 771–780)

Snapshot time ~22:56 PDT — this section's broker-adjacent findings are checked
against the earlier (pre-fix-confirmation) state; re-read §3 for the current broker
status.

### 7.1 100% definition (per lane, build axis vs wiring axis)

- **Closure law**: built = all 6 `ClosureLaw` variants real (confirmed, `closure.rs:488-520`).
  Wired = `ChatmanEngine::admit_transition` calls `RecursiveSocketClosure` for ≥1 real
  S1–S6 step, provable from `ChatmanEngine::new()`, not `RecursiveSocketClosure::declare()`
  directly. Falsifier: `grep -n "closure::" engine.rs` → 0 today.
- **Compensation catalog**: built = `CompensationKind` has all 7 PRD variants with
  per-kind validation (confirmed). Wired (manufacture path) = a real trigger calls
  `manufacture_compensation_workflow` without a human/test hand-assembling the
  `PriorActuationRef`. Falsifier: 0 non-test callers today; PROJ-775 is honestly
  `PLANNED`.
- **Child-completion gate**: built = `promote_observed_to_admitted` takes a real
  `&shacl::ValidationReport`, only promotes on genuine conformance (confirmed). Wired
  = called from a real remote-result admission path. Falsifier: 0 today.
- **N3 quarantine**: built = capability gate → builtin whitelist → cost-bound
  ordering, all real and tested (confirmed). Wired = `ChatmanEngine`'s dialect-routing
  sets `requires_n3_builtins: true` for a real N3-shaped input. Falsifier: hardcoded
  `false` at `engine.rs:1148` today.
- **GraphLaw authority registry**: module-wired (confirmed, `lib.rs:8`, 5 tests).
  Decision-wired = a caller reachable from something other than the crate's own unit
  tests consults `authority_for()`. Falsifier: 0 non-test callers of
  `admit_manufactured_arazzo_for_dialect`/`authority_for` today.
- **Full PRD §18 refusal catalog**: 100% means every named code (a) exists as a
  distinct typed value, (b) has ≥1 real triggering test, (c) appears in whatever
  catalog-completeness construct that taxonomy uses. Only `chatman::abi::Refusal` has
  such a construct (`ALL_REFUSAL_NAMES`); `CoreError` and the Erlang atom codes have
  none — "wire the catalog" cannot be one mechanical pass across all three.

### 7.2 Current delta

- **Closure/compensation/child-completion gate**: `ChatmanEngine` is still exactly
  the 5 private fields PROJ-771 documented — no closure/compensation handle exists to
  call through even if a call site were added elsewhere. Zero matches for
  `closure::|compensation::|RecursiveSocketClosure|CompensationWorkflow|
  manufacture_compensation_workflow|ChildCompletionState|ClosureLaw` in `engine.rs`.
  `promote_observed_to_admitted` has exactly one non-module reference anywhere, and
  it's a doc-comment cross-reference, not a call. This is confirmed-stable, not new —
  but PROJ-774's own "ALIVE" ticket entry doesn't state it's still 0-callers-outside-tests.
- **New finding — compensation's "6 new Refusal variants" claim doesn't match the
  code.** PROJ-759's ticket text states 6 new variants, deliberately deferred from
  the catalog. In fact `compensation.rs` constructs exactly **one** `Refusal`
  variant, the pre-existing generic `Refusal::ValidationFailed(String)`, at all 9 of
  its error sites — zero `CompensationXyz` variant exists anywhere. Nothing
  catalog-worthy to add yet on the compensation side, distinct from closure, which
  genuinely did build 8 distinct variants correctly awaiting catalog wiring.
- **Cross-lane catalog count, freshly computed**: `Refusal` enum has **46** variants,
  `ALL_REFUSAL_NAMES` lists **31** (67.4%). 15 missing, named:
  `ChildCompletionUnadmitted`, `ChildConformanceRefused`, `ClosureLawNoChildren`,
  `ClosureLawOrderedSubsetInvalid`, `ClosureLawPolicyNotDeclared`,
  `ClosureLawQuorumOutOfRange`, `ClosureLawUnknownChild`,
  `ExternalCutAuthorityMismatch`, `ExternalCutTypeMismatch`, `ExternalCutUndeclared`,
  `N3BuiltinRefused`, `N3CostBoundExceeded`, `N3DirectActuationRefused`,
  `ParentClosureUnsatisfied`, `PowlRegionNotAdmitted`. Matches documented precedent
  (PROJ-779/780/783 deferred this on purpose) but the exact count wasn't previously
  stated anywhere.
- **Structural finding**: no single "PRD §18 refusal catalog" artifact exists. It's
  fragmented across `chatman::abi::Refusal` (partial completeness gate),
  `praxis-core::CoreError` (zero catalog construct), and Erlang atom codes in
  `apps/arazzo_runner` (zero catalog construct).
- **N3Executor**: still zero production callers — `requires_n3_builtins: false`
  hardcoded, unchanged from `SYSTEM_DIAGRAMS.md`'s prior finding.
- **GraphLaw authority**: module-wired, decision-wired at zero. Unchanged.
- **Dangling intra-doc link**: `abi.rs:579` still references
  `[Refusal::ClosureLawNotImplemented]`, a variant PROJ-773 genuinely removed —
  cosmetic, breaks `cargo doc`'s intra-doc-link lint.

### 7.3 Sequenced remaining work

1. **Fix the compensation refusal-variant gap first** (XS) — a correctness-of-claim
   fix, blocks a correct PROJ-786 scope. Add real distinct variants, or explicitly
   document the fold into `ValidationFailed` by contract rather than let the ticket
   text claim 6 that don't exist.
2. **PROJ-786/787: wire the 15 missing chatman variants** into `ALL_REFUSAL_NAMES` +
   `gate_refusal_name_matches_const_list` + the 8 acceptance schemas (S).
3. **Decide the cross-taxonomy catalog question explicitly** (XS, scoping decision)
   before claiming "full PRD §18 catalog wired."
4. **PROJ-774 wiring**: give `promote_observed_to_admitted` one real non-test caller
   (S) — sequence after the broker's `admit_return/3` is confirmed stable (§3), so
   the caller is a real Erlang→Rust bridge path, not another synthetic Rust test.
5. **PROJ-775: build the actual compensation trigger** (M) — currently `PLANNED`
   correctly; the one piece of this section with no code at all yet.
6. **Chatman engine wiring for closure/compensation into S1–S6** (M) — once 4–5
   exist, add the engine-side call edge, reusing PROJ-796's
   byte-identity-preserving-when-absent test pattern rather than inventing a new one.
7. **N3Executor wiring** (S, lowest priority of the five) — needs a real N3-shaped
   admission path to exist first; no PRD §19 scenario currently requires it.
8. **Verifier report generator (PROJ-795)**: once it exists, feed this section's
   findings into it rather than re-deriving them — re-run the greps first, this
   snapshot ages fast.
9. **Fix the dangling doc link** (XS).

## 8. Refusal Catalog & Receipt Chain (PROJ-781, 782, 785–787)

### 8.1 PROJ-781 — Receipt Chain Minimum Fields

**100% definition**: 4 emission sites (step-dispatching, step-completing via
consequence capture, step-completing via return-admission, reaction-firing across 8
reaction classes), each wired with a test driving the real runtime path. Falsifier:
`grep -c "arazzo_runner_event_receipt:emit(" apps/arazzo_runner/src/*.erl` == 4.

**Current delta**: infrastructure real and complete (`#event_receipt{}` implements
all 10 PRD fields; `arazzo_runner_blake3.erl` shells real `b3sum`; 10 real EUnit tests
across two files). Only **1 of 4** emission sites wired (step-dispatching, at
`arazzo_runner_broker.erl:281`). **New gap**: the receipt log is ETS-only, no `dets:`
calls anywhere — does not survive a VM/process restart, unlike `#workflow_identity{}`
which is DETS-durable. Matters directly for PROJ-782.

### 8.2 PROJ-782 — Replay Verifier

**100% definition**: 5-step PRD algorithm (resolve by digest → restore admitted
state → apply admitted event corpus → recompute digests → verify receipt-head
equivalence), mismatch a typed refusal never a log line.

**Current delta**: essentially zero, genuinely gated on 3 things: (1) PROJ-781 must
be 4/4; (2) the receipt-log durability gap above; (3) "resolve the AIR artifact by
digest" has no concrete referent on the Erlang side — no digest-addressable artifact
store exists. Recommended scope: define "the AIR artifact" as `#runner_state.workflow_def`
+ `source_digest`/`projection_digest` already on `#workflow_identity{}`, checking
`blake3(workflow_def)` equals the recorded digest — explicitly not re-verifying the
Rail A/B manufacture chain (already done separately, Rust-side). **Status:
correctly `BLOCKED`**, not fabricatable smaller.

### 8.3 PROJ-785 — Return/Correlation/Broker Refusal Codes (8 codes)

7/8 live and tested. `RETURN_SEMANTIC_REFUSED` is the sole remaining gap.

**Sizing the bridge, resolved**: a smaller fix is correct; the full Erlang→
praxis-graphlaw SHACL bridge is out of proportion to this ticket. The repo's only
Erlang↔Rust bridge anywhere is `air_core`'s narrow `eval_expr_nif`. Reaching the
SHACL admission layer means either extending that NIF to run SHACL validation
in-process (pulling `oxigraph`/`spargebra`, two out-of-tree path deps, and the
nightly toolchain pin into a BEAM-loaded NIF) or a port/socket bridge (zero precedent
in this codebase — confirmed zero `open_port`/`gen_tcp`/`httpc` hits). **Recommended:
leave `RETURN_SEMANTIC_REFUSED` in `?UNENFORCED_RETURN_STAGES` (already disclosed,
not silent), close PROJ-785 at 7/8, and file the bridge as its own separately-sized
ticket.**

### 8.4 PROJ-786 — Closure/Compensation/Replay/Equivalence Codes (5 codes)

Not a single mechanical pass — bundles three structurally different kinds of work:

| Code | State |
|---|---|
| `CHILD_COMPLETION_UNADMITTED` | Real, tested (`abi.rs:514-526`) — mechanical wiring only |
| `PARENT_CLOSURE_UNSATISFIED` | Real, tested (`abi.rs:527-536`) — mechanical wiring only |
| `COMPENSATION_REQUIRED` | Does not exist anywhere — blocked on PROJ-775 (§7.3 item 5) |
| `REPLAY_DIGEST_MISMATCH` | Does not exist anywhere — blocked on PROJ-782 (§8.2) |
| `OTP_ATOMVM_SEMANTIC_DRIFT` | Real, tested, but lives in a test file, not `src/` — needs promotion (§4.2) or an explicit verification-only-scope decision |

**Recommended framing**: PARTIAL, 2/5 mechanically closeable now, 1/5 needs a design
decision, 2/5 hard-blocked on other PLANNED/BLOCKED tickets. Not a single-pass ticket.

### 8.5 PROJ-787 — N3/Measurement Codes (7 codes)

Cheapest of the three catalog tickets — dependencies (780, 766, 767) are all ALIVE —
but spans two unrelated `Refusal` type hierarchies (`praxis-graphlaw` and `cng`),
undisclosed in the ticket text.

- 3 real, tested `praxis-graphlaw` variants: mechanical wiring only.
- `N3_CAPABILITY_MISSING`: ambiguous — recommend mapping onto the existing
  `Refusal::N3UnavailableByProfile` rather than adding a duplicate, same pattern
  PROJ-784 already used.
- `MEASUREMENT_EVIDENCE_INSUFFICIENT`: real in `cng` (`CngRefusal::MeasurementEvidenceInsufficient`,
  `CNG_R29`) — different crate, needs `cng`'s own catalog mechanism confirmed to
  exist before extending.
- `MEASUREMENT_PROFILE_MISSING`: does not exist anywhere — genuinely new code needed
  (distinct triggering condition from `MeasurementEvidenceInsufficient`).
- `MULTIFRACTAL_CLASSIFICATION_UNADMITTED`: doesn't exist under that name; the
  underlying invariant is satisfied via `Ok(multifractal:false)` /
  `CngRefusal::MultifractalFitDegenerate`. Needs a judgment call — is a distinct
  `Err` variant required, or is the honest `Ok(false)` the correct representation of
  "not admitted"? Flag to the PRD owner rather than assume.

### 8.6 Sequenced remaining work (dependency-ordered, effort-tiered)

Effort tiers per this exploration's own calibration against already-closed tickets:
XS = one test, existing infra verbatim. S = one new mechanism, no new subsystem. M =
new subsystem with real state. L = full §20-scale ticket, M-tier repeated per item.

**Small, mechanical, do first (hours):**
1. PROJ-787's 3 real N3 variants → catalog + schemas.
2. PROJ-786's 2 real closure variants → catalog + schemas.
3. Document the `N3_CAPABILITY_MISSING` and `MULTIFRACTAL_CLASSIFICATION_UNADMITTED`
   mappings.
4. PROJ-785: close at 7/8, file `RETURN_SEMANTIC_REFUSED` as its own ticket.

**Medium, new code, no new architecture (a day or two each):**
5. PROJ-781: wire emission sites 2–4, one test per site through the real call chain.
6. Add a DETS mirror for the event-receipt log, or explicitly scope PROJ-782 to
   same-process replay only.
7. PROJ-787's `MEASUREMENT_PROFILE_MISSING` — new triggering condition + test.
8. PROJ-786's `OTP_ATOMVM_SEMANTIC_DRIFT` — promote to a real callable module, or
   document verification-only scope.

**Larger, real dependency chain, correctly sequenced after the above:**
9. PROJ-782 itself — once unblocked, the algorithm is bounded and well-specified,
   given DETS-backed `#runner_state{}` already exists (M).
10. PROJ-786's `COMPENSATION_REQUIRED` — blocked on PROJ-775.
11. PROJ-786's `REPLAY_DIGEST_MISMATCH` — blocked on step 9.

**Genuinely large, out of proportion to any of these five tickets:**
12. The Erlang→`praxis-graphlaw` SHACL bridge for `RETURN_SEMANTIC_REFUSED` (§8.3) —
    should be its own ticket with its own sizing.

## 9. Verification Ladder (PROJ-792–795)

### 9.1 100% definition

**PROJ-792 (chaos, `PRD.md:965-980`)** — exactly 10 named failure modes plus one hard
invariant ("no chaos case may create unreceipted actuation or false parent closure"),
each requiring a real injected fault against the live runtime (OTP process death,
remote engine restart, duplicate delivery, event reordering, delayed acknowledgment,
timeout, partition, stale result, malformed result, receipt corruption), run ≥3x
consecutively with identical outcome. A suite implementing only the modes with
existing test scaffolding and silently skipping the rest is not 100% — it must
disclose the subset by name. **Structural precondition unresolved**: the live
runtime's real parent-closure logic is `air_core`'s AND-join `pred_mask`/
`completed_mask`, not `chatman::closure.rs`'s `RecursiveSocketClosure` — that Rust
module has zero production callers from anything Erlang-side. PROJ-792 must state
explicitly which closure implementation is under test or it will silently test the
wrong thing.

**PROJ-793 (stress profile, `PRD.md:982-993`)** — 6 dimensions (concurrent workflow
instances, dispatch fan-out, child depth, receipt-chain length, event corpus replay
size, OCEL object/event volume), each requiring a declared, checked-in limit (number
+ exact command + environment + failure signature just past the limit) — never
"it worked once locally."

**PROJ-794 (benchmarks, `PRD.md:995-1009`)** — 9 separately-reported numbers, never
an aggregate: BCINR local transition latency, wasm4pm Arazzo-to-AIR compile cost,
Erlang AIR transition cost, OTP supervision/recovery cost, AtomVM transition cost,
broker dispatch overhead, replay throughput, RDF-to-OCEL construction cost,
multifractal measurement cost. Rust-side benchmarks (1/2/8/9) have real `criterion`
infra available; Erlang-side (3/4/5/6/7) have no timing harness anywhere in `apps/`.

**PROJ-795 (verifier report, `PRD.md:1011-1027`)** — a real program (explicitly not
a hand-written doc) producing all 13 fields from real command output: declared
artifacts, manufactured artifacts, admitted artifacts, refused fixtures, orphan
counts, projection digest consistency, AIR conformance corpus result, OTP/AtomVM
differential result, broker bypass search result, replay equivalence result, OCEL
transformation equivalence result, measurement rail status, Lean/Lake build status.

### 9.2 Current delta

| Ticket | Real progress | Evidence |
|---|---|---|
| PROJ-792 | 0 of 10 modes meet the bar. 2 reusable mechanisms exist (crash-restart, duplicate-dispatch dedup) but assert different things than the required invariants. | `arazzo_runner_workflow_test.erl:164,287-288`; `arazzo_runner_broker_test.erl` |
| PROJ-793 | 0 of 6 dimensions have a declared, checked-in limit. PROJ-757's "300-iteration" result is cited in prose only — the script itself is not in the tree. | `find` for `*stress_restart*`/`*kill_restart*` → nothing |
| PROJ-794 | 0 of 9 named benchmarks exist. Adjacent infra exists (`criterion` in `Cargo.toml`, 7 existing but unrelated bench files) but none target these 9. No Erlang-side timing harness anywhere. | directory listings this session |
| PROJ-795 | 0 of 13 fields wired. No script, binary, or `just` recipe exists anywhere. One adjacent building block (`praxis-lean/src/report.rs`) covers exactly 1 of 13 fields. | `grep -n "verifier" justfile` → nothing |

Upstream blockers unchanged: PROJ-791 `PARTIAL` (19.11 replay-of-OTLP-derived-RDF gap
open), PROJ-770 `PLANNED` (zero content), PROJ-769 landing concurrently (confirmed
ALIVE per §6, not yet reflected in `tickets/index.md`, which still shows `PLANNED` —
a live discrepancy to reconcile, not resolved by this synthesis).

A structural sequencing point: PROJ-758 (dependency of both 792 and 794) had a
production dead end (`admit_return/3` never called) that, per §3's later evidence,
appears fixed as of 23:02:12. Chaos/benchmark work built before that fix landed would
only exercise the outbound half of the broker's loop — confirm the fix is stable (§3)
before building PROJ-792/794 against the broker's return path.

### 9.3 Effort calibration (against this milestone's own already-closed tickets)

- **XS** — one new test reusing existing infra verbatim (comparable to PROJ-779's
  zero-cost-rule test).
- **S** — one new mechanism, no new subsystem (comparable to PROJ-758's
  `check_required_prior_receipts/5`, or PROJ-796's `verify_replay_with_external_cut`).
- **M** — a new subsystem with real state, real bugs found along the way
  (comparable to PROJ-757's supervision behavior + DETS + stress script — 2 real bugs
  found and fixed).
- **L** — the full §20 bar, i.e., M-tier effort **repeated once per named item** (10x
  for chaos, 9x for benchmark, 6x for stress) plus cross-item regression and the
  adversarial-review correction round every "ALIVE" claim in `ADVERSARIAL_DOD.md` has
  needed this session.

Under this calibration: **PROJ-792 and PROJ-794 in full are L-tier**; **PROJ-793 in
full is M-to-L**; **PROJ-795 in full is M**, but only after 791/792/793/794/770
produce real results to consume. See §11.

### 9.4 Recommended smallest valuable slices (sequenced)

1. **Resolve/confirm the PROJ-758 broker dead end status** (XS, decision) — before
   building anything against it.
2. **PROJ-793's smallest slice**: declare one real limit — receipt-chain length,
   since PROJ-781's `emit/1` chaining already exists (S).
3. **PROJ-792's smallest slice**: 2 of 10 modes end-to-end to the real bar — OTP
   process death and duplicate delivery, both with reusable mechanisms already
   present (S for the pair, once step 1 is decided).
4. **PROJ-794's smallest slice**: one Rust-side benchmark using existing `criterion`
   infra — multifractal measurement cost, since `bench::multifractal` already has
   timing-adjacent code and a real data source (XS–S).
5. **PROJ-795's smallest slice — build the skeleton now, not last.** Emit the
   13-key structure with real values for what's measurable today (Lean/Lake status
   via `praxis-l4`'s existing `report.rs`; artifact counts via existing test-suite
   output) and an explicit `NOT_YET_MEASURED: blocked on PROJ-79N` marker for the
   rest — never a fabricated pass. One field is unusually cheap to populate for real
   right now: "broker bypass search result" already has a documented, reproducible
   finding (the token-forgery gap, §3) — this field can report a real result today,
   independent of any other ticket landing (S for the skeleton + that one field; M
   to wire in the remaining 12 once their sources exist).

## 10. Cross-Rail Prioritized Roadmap

Ordered by dependency and leverage, not by ticket number. Sizes use the §9.3
XS/S/M/L calibration throughout, since it is the only one of the 8 explorations that
grounded its tiers against this milestone's own already-closed comparables.

1. **Confirm the Erlang broker fix is stable** (XS) — re-run the 3 greps in §3.2
   against current file state. Gates Rail C/D closure, Rail E's golden-digest
   re-verification, closure/compensation's PROJ-774 wiring, and PROJ-792/794's
   broker-dependent items. Highest-leverage single check in this roadmap because five
   of eight explorations found this file mid-edit.
2. **Commit the `wasm4pm-compat` fix** (S, Rail A/B) — undermines the reproducibility
   of every Rail A/B green result today, including results already claimed ALIVE
   across five adversarial-review rounds this session.
3. **Complete the Erlang git-hygiene remediation** (XS, Rail C/D) — 9 tracked
   `.beam` paths, unstaged `.gitignore` edit; every `rebar3 eunit` run this session,
   including the broker fix verification, ran against a dirty tree.
4. **Add one real production entry point per island, in parallel** (S each, ~5
   independent islands): Rail A/B's `admit_transition_with_external_cut` (§2.3 W1);
   Rail F/G's full OTLP→RDF→OCEL→measurement chain (§5.3 step 1); closure's
   `promote_observed_to_admitted` (§7.3 step 4, sequence after step 1 above so the
   caller is a real bridge path); N3Executor's `requires_n3_builtins` flip (§7.3 step
   7, lowest priority of this group). These are structurally the same fix repeated —
   wiring, not new algorithm work — and are independently parallelizable.
5. **Mechanical refusal-catalog wiring** (S, §8.6 items 1–4) — 5 already-real
   variants into `ALL_REFUSAL_NAMES` + schemas, the compensation-claim correction,
   the cross-taxonomy scoping decision, PROJ-785 closed at 7/8 with the SHACL bridge
   spun out as its own ticket.
6. **Rail H's remaining wiring** (S–M, §6.3) — `.lake/` scope fix first (blocks
   bounded-time verification entirely), then the mechanical items (2–4), then the
   negative fixtures and full-corpus axiom classification.
7. **Rail E's AtomVM scope decision** (XS decision, §4.3 step 1) — before any more
   corpus-authoring work, since it determines whether PROJ-761 needs to grow into a
   Group B reaction layer or can stay corpus-only.
8. **Rail E's cheap corpus extension** (S, §4.3 step 3) — `child_complete`/
   `child_refused`, no architecture change, doable independent of step 7's answer.
9. **Coverage/benchmark/determinism baselines** (M, Rail A/B §2.3 W4–W6; Rail F/G
   §5.3 steps 3–4) — run once reachability (step 4) makes the numbers meaningful
   rather than test-tautological.
10. **Verification Ladder skeleton and smallest slices** (§9.4, S–M each) — start
    the PROJ-795 skeleton in parallel with everything above, not after; it is the
    one artifact that turns every other item's status into a machine-checked field
    instead of a hand-transcribed one. Do not attempt the full L-tier scope of
    PROJ-792/794 in this pass — see §11.
11. **Genuinely new-architecture items, scheduled last and separately estimated**:
    the AtomVM native/fallback target (§4.1, if Option A is chosen); the Erlang→
    praxis-graphlaw SHACL bridge (§8.3/§8.6 item 12); PROJ-775's compensation
    trigger and PROJ-782's replay verifier (both correctly `BLOCKED`/`PLANNED`, not
    artificially acceleratable).

## 11. Multi-Session Undertakings — Explicit Call-Outs

The following are not next-session remaining work. Each is sized against this
milestone's own demonstrated pace (§9.3) and should not be scheduled as if it were a
short remaining item:

- **PROJ-792 (chaos suite) in full**: 10 named failure modes, each S-to-M tier once
  the broker dead end is accounted for, plus cross-item regression and at least one
  adversarial-review correction round per this session's own demonstrated pattern
  (every ALIVE claim in `ADVERSARIAL_DOD.md` needed at least one correction round
  before it held). **L-tier: several M-tier efforts, not one.**
- **PROJ-794 (benchmark suite) in full**: 9 separately-reported numbers, 5 of which
  (Erlang-side) require building a timing harness from scratch where none exists
  today. **L-tier**, comparable in scope to PROJ-792.
- **PROJ-793 (stress profile) in full**: 6 dimensions. **M-to-L tier** — closer to S
  per item since most reuse existing scaling knobs, but still 6 independent
  measurement efforts with checked-in artifacts, not a single script.
- **PROJ-795 (verifier report) in full**: correctly buildable as a skeleton now
  (§9.4 step 5), but full 13-field population is gated on PROJ-791–794/770 all
  producing real results first — building it early gets the skeleton, not the
  finished artifact. **M tier for the skeleton, additional M tier once the other
  four ladder tickets land.**
- **The AtomVM real-runtime target (Option A, §4.1)**: either a second
  implementation of expression evaluation that must be kept in lockstep with the
  Rust NIF (new determinism-parity burden with no existing precedent in this repo),
  or a full ESP32/AtomVM toolchain provisioning plus a static-link native port.
  Neither is a wiring fix — both are new infrastructure this repo has never built.
- **The Erlang→praxis-graphlaw SHACL bridge (§8.3/§8.6 item 12)**: a new
  Rustler NIF pulling `oxigraph`/`spargebra` and two out-of-tree path dependencies
  into a BEAM-loaded library, or a port/socket protocol with zero existing precedent
  anywhere in this codebase. Genuinely out of proportion to PROJ-785 as a ticket.
- **Rail H's full-corpus axiom classification (§6.3 step 7)**: 71 pre-existing
  thesis-corpus axioms, each individually needing a crypto-assumption-allowlist vs.
  real-gap determination — this is a review task across the inherited thesis corpus,
  not a code change, and its own bounded-time execution is blocked on the `.lake/`
  scope fix landing first.
- **Full refusal-catalog cross-taxonomy unification (§8.4, §7.2 structural
  finding)**: if the scoping decision in §10 step 5 comes back "unify across all
  three taxonomies" rather than "scope per-taxonomy," that is a new registry design
  and build, not an extension of the existing `ALL_REFUSAL_NAMES` array.

## References

- `tickets/index.md` — per-ticket status table, dependency graph
- `ADVERSARIAL_DOD.md` — Tier-1/Tier-2 findings this document's deltas were checked against
- `SYSTEM_DIAGRAMS.md` — sequence diagrams for the wiring gaps named throughout
- `PRD.md` — source of every falsifiable bar cited in §2–§9 (§15 receipt chain,
  §16 measurement pipeline, §18 refusal catalog, §19 acceptance scenarios,
  §20 verification ladder)
- `RAIL_A_B_STATUS.md` — prior Rail A/B status, superseded in detail by §2 above
- `RAIL_G_MEASUREMENT_DESIGN.md` — measurement profile design referenced in §5
- `SAFETY_FINDINGS.md` — safety-relevant findings referenced in §4, §6
- `docs/CORE_TEAM_DISCIPLINE.md` — engineering standards this document's claims
  follow (no unearned status without a command+output cited in the same breath)
- `.claude/rules/no-overclaiming.md` — status vocabulary this document's per-rail
  verdicts use (ALIVE, PARTIAL, BLOCKED, MOCKED, REFUSED/UNSUPPORTED, UNVERIFIED)