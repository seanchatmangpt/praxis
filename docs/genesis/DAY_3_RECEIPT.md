# Genesis Day 3 Receipt — Adversarial Hardening

**Date:** 2026-07-02 (Day 3 of the seven-day program)
**Program:** GENESIS Day 3 — establish correctness *without trusting any single implementation*: differential verification (independent oracles must agree on a shared, generated corpus), fuzz + proptest over every admission boundary (quarantine, config loader, receipt validator, PDDL parser), and mutation-test the receipt chain (drop / reorder / flip) — killing every survivable mutant or receipting why it survives.
**Principle:** build beyond human reading, within human verification. No claim below exceeds a mechanism you can re-run.

> This receipt is written **out of order**, after Days 4, 5, 6, and 7 were already receipted and after the Day-7 `GENESIS_SEAL.json` was cast. That is unusual and is stated plainly rather than hidden. Day 3's work (the differential oracle suite + the fuzz/mutation suites) genuinely landed, but at Day 7 it had sealed no manifest of its own — every later day (`GENESIS.md` Day 3 row, `GENESIS_SEAL.json`, and the `chain_note` of Days 4/5/6) records Day 3 as the *unsealed gap they chained around*. This closer seals Day 3 for real, restoring the predecessor the program specified. The chain consequences of sealing out of order are handled honestly in **Chain** below — nothing is back-dated and no committed record is rewritten.

---

## The day's thesis, proven in code

**Correctness by independent agreement, not by trust.** Nothing here rests on one implementation being "comprehended" as correct. Every core computation is checked against a second (often a third) implementation written from a different direction, over a *generated* corpus. Where two implementations disagreed, the disagreement was root-caused to a real bug and fixed — never papered over by loosening an assertion (one such bug is recorded in §Objective below). Where an oracle could not be wired (a dependency that does not resolve), that is a receipted BLOCKER inside the test suite, not a silent omission.

---

## What landed

Four new/edited surfaces; **46 tests across four suites, all green**, re-run live at receipt time.

| Suite | File (LoC) | Method | Result |
|---|---|---|---|
| Differential | `tests/differential.rs` (766) | 4 oracle pairs agree on a shared/generated corpus | **8 passed** (`--features proposer`) |
| Fuzz — core perimeter | `crates/praxis-core/tests/fuzz_boundaries.rs` (227) | proptest, 2048 cases/prop | **8 passed** |
| Fuzz — root ops surface | `tests/fuzz_ops.rs` (221) | proptest, 1024 cases/prop | **11 passed** (`--features proposer`) |
| Mutation — receipt chain | `crates/praxis-core/tests/mutation_chain.rs` (447) | drop/reorder/flip operators, caught-at-stage | **19 passed** (11 kills + 5 documented survivors + 3 baseline/boundary) |
| Bug fix + regression | `crates/praxis-proposer/src/objective.rs` (267) | fix surfaced by Pair-4 differential | 5 unit tests incl. backward-candidate regression |
| Bench | `benches/bench_main.rs` (148, edited) | added `admission_throughput` group | measured live, §Bench |

Per the standing constraint, **no `cargo-fuzz`/libfuzzer targets** were added — fuzzing is proptest with high case counts, and the count is documented in each file's module doc and overridable at runtime via `PROPTEST_CASES` (proptest reads it and it wins over the `with_cases` default).

---

## (1) Differential oracle pairs — wired + disagreements handled

The simdjson method: two or three independent implementations, one shared corpus, byte/decision agreement asserted. From `tests/differential.rs`:

1. **PLANNERS** — `bcinr_pddl::ground::GroundTemporalProblem` (praxis dep) vs `wasm4pm_planner::find_temporal_plan` (independent dev-dep). Identical durative-STRIPS PDDL text into both parsers/planners. A **third** oracle — a from-scratch monotone-reachability fixpoint — decides ground-truth solvability, and an independent plan-replay validates that each returned plan actually reaches the goal. Corpus: a seeded xorshift64\* generator of durative-STRIPS domains + a numeric-fluent capacity exemplar + a revenue-stage chain.
   - `pair1_planners_generated_corpus_triple_agreement`, `pair1_planners_capacity_numeric_exemplar`, `pair1_planners_revenue_stage_chain`, `pair1_scope_classical_exemplars` (SCOPE receipt: the classical `:strips`/`:adl` exemplars — `revenue.pddl`, `lawobject-capability.pddl` — are parse-anchored on the bcinr side only, recorded in-test, not silently dropped).
2. **CONFORMANCE** — praxis's `PowlReplayVerifier` (POWL token-passing) vs an independent Petri-net token-game replay. `pair2_conformance_powl_vs_petri_agreement`. The from-scratch Petri oracle dteam (`NetBitmask64`) **does not resolve as a praxis dep** — recorded as a receipted BLOCKER inside `pair2_blocker_dteam_dep`, so the pair runs against the in-test Petri reimplementation rather than pretending the external oracle was wired.
3. **CHAIN** — praxis `chain::recompute_chain` vs a ~15-line from-scratch `BLAKE3(prev_hex || frame)` reimplementation, byte-for-byte over 100 random records. `pair3_chain_recompute_vs_independent_100_records`.
4. **OBJECTIVE** — `praxis_proposer::objective` scoring vs a naive in-test dot product, **bit-exact f64 agreement**. `pair4_objective_score_bit_exact` (`--features proposer`).

**Dependency probe (why the dev-dep resolves).** `wasm4pm-planner` (path, 26.7.1) is a dev-dependency of the root crate. It resolves cleanly against praxis's stable graph — rmcp 2.0 / schemars 0.8 coexist as separate versions; the planner shares the 26.7.1 wasm4pm workspace praxis already uses via `prolog8`/`wasm4pm-cognition`. Both planners parse the *same* durative PDDL text, which is what makes their agreement meaningful rather than an artifact of two different front-ends.

### Objective — the disagreement Pair 4 surfaced, root-caused and fixed

Pair-4 differential exposed a real defect in `praxis_proposer::objective::compute_fluents`. For a *backward* candidate (`target < account.stage`) the `advance` fluent was computed as `target.index() - account.stage.index()` in **`u8`**, which **underflows** — a panic in debug, a wrap to a huge positive value in release. A backward candidate is a legitimate query for this public function and must yield a *negative* advance. Fix (`objective.rs`): subtract in `f64`, not `u8` —

```rust
let advance = f64::from(target.index()) - f64::from(account.stage.index());
```

Regression test `compute_fluents` (backward candidate → negative advance, no panic/wrap) locks it. This is the day's proof that differential testing pays: the bug was invisible to any single-implementation test and only appeared when a naive oracle disagreed.

---

## (2) Fuzz — properties + case counts

Two properties, held across every admission surface: **total absence of panics** on arbitrary input, and **every rejection carries a reason** (no bare `false`).

**`crates/praxis-core/tests/fuzz_boundaries.rs`** — 2048 cases/property (`PROPTEST_CASES` override), 8 props over the core perimeter:
`quarantine_never_panics_on_arbitrary_bytes`, `…_on_arbitrary_strings`(blank-reject), `…_on_json_like`, `quarantine_typed_admits_wellformed`, `quarantine_typed_predicate_never_panics`, `quarantine_value_schema_never_panics`, `recompute_chain_hash_never_panics_on_arbitrary_hex`, `validate_never_panics_on_arbitrary_records`.

**`tests/fuzz_ops.rs`** — 1024 cases/property (`PROPTEST_CASES` override), 11 props over the root ops + config + PDDL + mission surface (all under `--features proposer`, which gates `revtac::Mission`):
every `ops::*_payload` (`judge`/`admit`/`receipt`/`promote`) never panics on arbitrary/JSON-like strings; `ops_receipt_pipeline_seals_wellformed_input`; `PraxisConfig` TOML admission via the real `star_toml::TrustedLoader` path (`config_admission_never_panics_on_arbitrary_toml` / `…_on_toml_like` / `config_rejects_out_of_range_capacity`); `promote_never_panics_on_arbitrary_standing`; `mission_parse_never_panics` / `…_on_toml_like`; and PDDL via `bcinr_pddl` (`pddl_parsers_never_panic_on_arbitrary_text`, `pddl_solve_pipeline_never_panics_on_pddl_like`).

---

## (3) Mutation testing — receipt-chain kill matrix

`crates/praxis-core/tests/mutation_chain.rs` applies affidavit-style operators (drop / reorder / field-flip) to a lawful receipt chain and asserts the validator **rejects** each mutant — recording *which validation stage* catches it (kill localization, not just kill/no-kill). Baseline `baseline_lawful_chain_passes_every_stage` proves the unmutated chain passes all stages, so a kill is attributable to the mutation and not to a broken fixture.

**Killed mutants (11) — caught-at-stage:**

| Mutant | Operator | Caught at stage |
|---|---|---|
| `mutant_version_bump` | field-flip (schema version) | schema |
| `mutant_hash_truncate` | field-flip (truncated hash) | schema |
| `mutant_event_drop_interior` | drop (interior event) | chain linkage |
| `mutant_event_reorder` | reorder | monotonic + linkage |
| `mutant_instruction_id_regression` | field-flip (id goes backward) | monotonic |
| `mutant_timestamp_future` | field-flip (ts past bounded clock) | monotonic (bounded clock) |
| `mutant_timestamp_skew_resealed` | field-flip + reseal | monotonic |
| `mutant_field_flip_node_kind` | field-flip | chain recompute |
| `mutant_field_flip_ts_ns_without_reseal` | field-flip | chain recompute |
| `mutant_field_flip_payload_hash` | field-flip | chain recompute |
| `mutant_field_flip_activity_idx` | field-flip | chain recompute |

**Survivors (5) — documented gaps, receipted not hidden.** Each is a mutation the *current* validator does **not** catch, with the reason it is out of the chain-integrity contract:
`survivor_head_drop`, `survivor_tail_drop` (dropping the boundary event yields a still-internally-consistent shorter chain — length is not attested against an external count here), `survivor_andon_flip`, `survivor_obligation_count_flip`, `survivor_object_ids_mutation` (fields that are not folded into the chain hash / not range-checked by the validator). These are named tests asserting the survival explicitly, so any future validator tightening will flip them to kills and the test will demand updating — the gap cannot rot silently.

**Boundary tests (3):** `boundary_ts_equal_to_now_is_not_future`, `boundary_equal_consecutive_ts_is_allowed` (equal consecutive timestamps are lawful; only strict regression is a kill), plus the baseline above.

Kill rate on in-scope operators: **11/11**; 5 explicitly-out-of-scope survivors receipted. No survivable in-contract mutant left un-killed.

---

## (4) Bench — measured live vs. target

`benches/bench_main.rs` gained the `admission_throughput` Criterion group (`Throughput::Elements(1)` → calls/sec), timing the *green* admission fast lane (scalar `value`, no obligations → `Validated`/`Admitted`) — a clean throughput ceiling, not a mixed path. A `debug_assert!` guards that the payload is actually on the green path so the bench cannot silently time a parse error. Measured live at receipt time (`--warm-up-time 1 --measurement-time 3`):

```
admission_throughput/judge_payload/green   time: [1.3619 µs 1.3635 µs 1.3653 µs]   thrpt: ~733 Kelem/s
admission_throughput/admit_payload/green   time: [1.1691 µs 1.1717 µs 1.1749 µs]   thrpt: ~853 Kelem/s
```

**Target reconciliation (honest).** Day 3 set no hard-coded numeric admission-latency target in code — the bench is a *measurement ceiling* the group exists to track over time, and the module doc says so. Both surfaces admit in **~1.2–1.4 µs single-threaded** (≈ 0.73–0.85 M calls/s), comfortably sub-10 µs, i.e. admission is not a bottleneck against any plausible MCP/CLI request rate. No claim in this receipt rests on a specific throughput figure; the numbers above are what this machine measured at close time and are reproducible via `cargo bench --bench bench_main -- admission_throughput`.

---

## Full test sweep — green tail (re-run at receipt time)

```
tests/differential.rs           running 8 tests  … test result: ok. 8 passed; 0 failed   (--features proposer)
tests/fuzz_ops.rs               running 11 tests … test result: ok. 11 passed; 0 failed  (--features proposer)
praxis-core fuzz_boundaries.rs  running 8 tests  … test result: ok. 8 passed; 0 failed
praxis-core mutation_chain.rs   running 19 tests … test result: ok. 19 passed; 0 failed
```

Total for the day's owned surfaces: **46 passed, 0 failed.** (`fuzz_ops` and `differential` require `--features proposer`; without it, `fuzz_ops` correctly fails to compile because `revtac::Mission` is feature-gated — recorded in Refusals §2.)

---

## Refusals / gaps (receipted, not papered over)

1. **Sealed out of order, after the Day-7 seal — not hidden.** `GENESIS_SEAL.json` (committed Day-7 artifact, `seal_hash 9c666317…`) recorded Day 3 as *unsealed* and covered only the two contiguous links 1→2. Days 4, 5, and 6 each chained Day 2 and named Day 3 as the gap they routed around. This receipt does **not** rewrite that seal, nor the Day-4/5/6 `chain_note`s, to retroactively claim Day 3 was sealed at their emission. Day 3 now has a genuine manifest chaining Day 2 (`prev = cb184872…`); the canonical week-seal remains the true Day-7-state record. Re-casting the week-seal to fold in the now-genuine Days 3/4/6 links is a follow-up for whoever next seals over a quiescent tree.
2. **`tests/fuzz_ops.rs` requires `--features proposer` to compile.** It imports `my_conforming_project::revtac::Mission`, which is gated behind the `proposer` feature. Run without the feature, it is a compile error (`E0432: unresolved import … configured out`), not a silent skip. This is by design — the mission-fuzz property is only meaningful when the mission surface is compiled in — and is stated so the correct invocation (`cargo test --test fuzz_ops --features proposer`) is on the record.
3. **CONFORMANCE external oracle (dteam / `NetBitmask64`) not wired as a dep.** It does not resolve as a praxis dependency; Pair 2 runs against an in-test Petri reimplementation and records the blocker in `pair2_blocker_dteam_dep` rather than claiming the external oracle agreed.
4. **PLANNERS classical exemplars are single-oracle.** The `:strips`/`:adl` exemplars (`revenue.pddl`, `lawobject-capability.pddl`) are parse-anchored on the bcinr side only — `wasm4pm_planner` targets durative problems — so they are a SCOPE note (`pair1_scope_classical_exemplars`), not a differential claim. Only the durative corpus + numeric/stage exemplars carry the triple-agreement guarantee.
5. **Mutation survivors are a real coverage boundary.** The 5 documented survivors (head/tail drop, andon/obligation-count/object-ids flips) are mutations outside the chain-integrity contract the validator currently enforces. They are asserted-as-surviving so the gap is visible and will break loudly if a future validator tightens — but as of Day 3 they are genuinely un-killed and named as such.
6. **Non-quiescent tree; live HEAD drift across the constellation.** Because this seal is out-of-order (post-Day-7), sibling HEADs have advanced past their Day-2 values; the manifest records live branch/HEAD/dirty_files while carrying each sibling's Day-2 crate-version map (Day-3 work touched only praxis), with the drift flagged in the manifest `chain_note`. `praxis` has 115 dirty files at close (concurrent agents live-editing). Push / tag / publish of the constellation remain out of scope for this additive closer.

---

## Chain

Manifest algorithm (matches Days 1, 2, 4, 5, 6 exactly): `manifest_hash = blake3(json.dumps(obj, sort_keys=True, separators=(",",":")))` with the `manifest_hash` field removed. **Verified reproducible at receipt time:** recomputing over `MANIFEST_DAY_2.json` reproduces `cb184872…` (the same script that produced this Day-3 hash), and reloading the written `MANIFEST_DAY_3.json` reproduces its own stated hash. Constellation = the same 11 repos Days 1–2 recorded; every sibling HEAD was re-scanned **live** at seal time (drift recorded in the manifest `chain_note`); `praxis` crate versions were re-derived live from `[package]` sections and include the post-Day-3 additions `agent8`/`pddl-index`.

- **prev (the genuine predecessor — Day 2)** `manifest_hash` = `cb184872b7c8bd5030b524a5430bd44d208db5b151f6f21066514293dea4e8c9`
  (`docs/genesis/MANIFEST_DAY_2.json`).
- **this day (Day 3)** `manifest_hash` = `5686fe5f07a2ba92c5938dd58bd90c7284be9cbc2fc4d33181b8c5cb94a5afbe`
  (`docs/genesis/MANIFEST_DAY_3.json`; `prev_day_hash =` the Day-2 hash above).

Day-3 `praxis` HEAD recorded in the manifest: `d8e32609d08a4ff05933298d9e20d88f2007274f` (the HEAD immediately before this receipt commit). The week now has, in sealed form, links 1→2 (contiguous) plus **3→2**, 4→2, and 6→2 — Day 3 is no longer the silent gap Days 4/5/6 chained around; it is a genuine, independently-reproducible link.

---

## What Day 4 inherits

> Days 4, 5, 6, 7 already ran. What follows is what a **re-seal / release pass over a quiescent tree** now inherits from a genuinely-sealed Day 3.

- **The Day-3 gap is closed.** The debt the Day-4 receipt listed first — "`MANIFEST_DAY_3.json` — Day 3's fuzz/mutation work is still unsealed" — is now discharged. A future `GENESIS_SEAL` recomputation can fold Days 3, 4, and 6 in honestly (all chaining Day 2); the current `seal_hash 9c666317…` remains the true Day-7-state record and must not be silently overwritten.
- **A correctness substrate, not just tests.** Day 3 leaves a *reusable* verification method: differential oracle pairs (`tests/differential.rs`) with a seeded, byte-reproducible corpus generator; a fuzz harness pattern (`PROPTEST_CASES`-tunable, panic-freedom + reason-carrying rejection) covering the whole admission perimeter; and a mutation kill-matrix with explicit caught-at-stage localization and named survivors. Any new admission surface (e.g. the Day-4 membrane / `agent8` fleet path) can be dropped into the same harnesses.
- **One real bug fixed, with a regression lock.** `praxis_proposer::objective::compute_fluents` no longer underflows on backward candidates; the `f64` fix + `compute_fluents` regression test are permanent. This is the concrete return on differential testing.
- **Known debts still open for the release pass:** (1) the `PRAXIS_SIGNING_KEY` parallel-test race in `praxis-core` / sibling receipt tests (documented Days 4–7); (2) the `FleetStatusParams` visibility lint on the MCP bin; (3) tree quiescence — push, tag, and `cargo publish` remain correctly refused until the constellation is committed and green; (4) the 5 documented mutation survivors are candidates to convert to kills if the receipt-validator contract is widened (length attestation, andon/obligation range checks).
