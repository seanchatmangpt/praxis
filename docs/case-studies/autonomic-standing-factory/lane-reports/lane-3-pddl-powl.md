# Lane 3 — PDDL Repair Planner and POWL Process Model

Status: DONE.

## Lane name

Lane 3 — PDDL Repair Planner and POWL Process Model
(`docs/case-studies/autonomic-standing-factory/lane-reports/lane-3-pddl-powl.md`).

## Files inspected

- `examples/v26_7_6_after_neon/goal.ttl` — the `pdl:` vocabulary exemplar
  mirrored exactly (Domain/Type/Predicate/Action/Problem shape, `pdl:param`
  with `pdl:index`/`pdl:var`/`pdl:ofType`, `pdl:pre`/`pdl:add`/`pdl:del` as
  string-literal atoms).
- `src/mfg.rs` — `enforce_pddl8` (arity/params/conjuncts bounds, confirmed
  against `wasm4pm-compat::pddl::PDDL8_MAX_*` = 8/8/8, plan depth 64, ground
  4096) and the manufacture/validate pipeline.
- `src/plan_run.rs` — the `plan_run_payload` vertical slice (graph -> mfg ->
  solve -> POWL compile -> receipted execute -> artifact write -> ledger
  receipt); its determinism contract (invariant 3, `ts_ns: 0` everywhere).
- `src/bin/ocel_process_validate.rs` (957 lines, read in full) — the
  hardcoded `CHILD_SPECS`/`ORDER_LABEL_PAIRS` release-loop POWL model, its
  `ModelReport{alphabet,children,order_pairs}` projection, and its 8-test
  suite.
- `src/verbs/receipt.rs`, `docs/releases/v26.7.6/RECEIPT_VERIFY_OCEL.md` —
  confirmed `receipt validate --dir <path>` accepts an arbitrary receipts
  directory (not just the configured default).
- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md` —
  read first per instructions; Phase 0 confirmed done, directories present.
- `docs/case-studies/autonomic-standing-factory/lane-reports/lane-1-cargo-cicd.md`
  — confirmed real evidence already exists
  (`target/praxis-standing/standing.json`, `standing.ocel.json`,
  `docs/releases/v26.7.6/ocel/*.json`) before authoring the domain's init
  state, so `(has-evidence repo-praxis)` reflects reality, not fabrication.

## Files changed

- `docs/case-studies/autonomic-standing-factory/case-study/pddl/goal.ttl`
  (new) — the 16-action PDDL8 repair domain (see below).
- `docs/case-studies/autonomic-standing-factory/case-study/pddl-out/{domain.pddl,problem.pddl,plan.json}`
  (new) — `plan run` output.
- `docs/case-studies/autonomic-standing-factory/case-study/pddl_{domain.pddl,problem.pddl,plan.json}`
  (new) — canonical manifest-name copies of the same three files.
- `docs/case-studies/autonomic-standing-factory/case-study/pddl-receipts/receipts.jsonl`
  (new) — the ledger receipt from the `plan run` invocation used for the
  canonical-copy artifacts.
- `docs/case-studies/autonomic-standing-factory/case-study/powl_model.json`
  (new) — the case-study POWL model's `ModelReport` projection.
- `src/bin/ocel_process_validate.rs` — added `ModelKind` enum, `parse_args`,
  `CASE_STUDY_CHILD_SPECS`/`CASE_STUDY_ORDER_LABEL_PAIRS` tables (parallel to
  the untouched v26.7.6 `CHILD_SPECS`/`ORDER_LABEL_PAIRS`), refactored
  `release_loop_model`'s body into a shared `build_loop_model(specs, pairs)`
  used by both `release_loop_model()` and the new `case_study_loop_model()`,
  and added `run_case_study()` + a `--model` dispatch in `main()`.
- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md` —
  updated phase rows 10 and 11 only (PDDL repair domain, POWL process
  model), from PENDING to DONE with evidence pointers. No other rows
  touched. No merge conflict encountered (file had no concurrent diff at
  edit time).

Explicitly NOT touched (Lane 2's ownership, confirmed by directory listing
before editing): `case-study/shapes/`, `case-study/shex/`, `case-study/rules/`,
`case-study/graphlaw_judgment.ttl`, `src/bin/case_study_judge.rs`,
`crates/praxis-graphlaw/tests/zz_case_study_sanity_check_temp.rs`,
`CASE_STUDY.md`, `PRODUCTION_READINESS.md` — these were present as untracked
files in the working tree when this lane started/finished (Lane 2 running
concurrently) and were left alone, not staged, not committed by this lane.

## Commands run

```
cargo build --features ggen --bin my-conforming-project

cargo run --features ggen --bin my-conforming-project -- plan run \
  --goal docs/case-studies/autonomic-standing-factory/case-study/pddl/goal.ttl \
  --out-dir docs/case-studies/autonomic-standing-factory/case-study/pddl-out \
  --receipts-dir docs/case-studies/autonomic-standing-factory/case-study/pddl-receipts

# determinism proof (two independent out-dirs)
cargo run --features ggen --bin my-conforming-project -- plan run \
  --goal docs/case-studies/autonomic-standing-factory/case-study/pddl/goal.ttl \
  --out-dir /tmp/pdl_det_run_a --receipts-dir /tmp/pdl_det_receipts_a
cargo run --features ggen --bin my-conforming-project -- plan run \
  --goal docs/case-studies/autonomic-standing-factory/case-study/pddl/goal.ttl \
  --out-dir /tmp/pdl_det_run_b --receipts-dir /tmp/pdl_det_receipts_b

cargo run --features ggen --bin my-conforming-project -- receipt validate \
  --dir docs/case-studies/autonomic-standing-factory/case-study/pddl-receipts

cargo build --bin ocel_process_validate
cargo test --bin ocel_process_validate
cargo run --bin ocel_process_validate                      # default (release, unchanged)
cargo run --bin ocel_process_validate -- --model case-study # new case-study model

cargo build --workspace --features ggen
cargo test --features ggen --lib --bins
cargo test --features ggen --test plan_run_e2e
```

cwd for all commands: `/Users/sac/praxis` (repo root).

## Artifacts produced

- `case-study/pddl/goal.ttl` — domain: 11 types, 18 predicates, 16 action
  schemas (each within PDDL8 bounds: <=3 params, <=3 pre-conjuncts, <=2
  effect-conjuncts, arity 1 throughout — all well under the 8/8/8 limits);
  problem: 11 objects, 3 init facts, 1 goal atom.
- `case-study/pddl-out/{domain.pddl,problem.pddl,plan.json}` and the
  canonical-name copies `case-study/pddl_{domain,problem}.pddl`,
  `case-study/pddl_plan.json`.
- `case-study/pddl-receipts/receipts.jsonl` — one ledger receipt for the
  canonical `plan run` invocation.
- `case-study/powl_model.json` — `ModelReport{alphabet: 16, children: 16,
  order_pairs: 114}` for the case-study process model.
- `src/bin/ocel_process_validate.rs` diff (case-study `--model` support).

## PDDL result

- **admitted: true**
- **grounder: naive**
- **plan_len: 16** (all 16 domain actions fire, one ground instance each)
- **plan (in solved order)**:
  `classify-external-side-effect, demote-claim, emit-standing,
  materialize-n3, validate-shacl, validate-shex, compute-datalog-closure,
  solve-pddl-plan, compile-powl-model, attach-benchmarks, smoke-client,
  verify-receipts, record-ocel, validate-wasm4pm-process, promote-claim,
  render-final-verdict`
- **powl_chain_hash**:
  `blake3:d9d50a2f561f0c54fd9e655cac6ef4c96b99b91bbb09d2148a804a68608cb658`

This is not a trivial 1-step goal satisfaction. The domain's init state
honestly encodes a bad prior state — `(claim-promoted
autonomic-standing-factory-local-first)` is true from the start, modelling
an earlier evidence-less/unlawful promotion of the scope's own claim. The
only path to the goal `(ready-for-scope
autonomic-standing-factory-local-first)` requires the planner to
`demote-claim` that premature promotion, independently `classify-external
-side-effect` on the fleet's real open cargo-cicd receipt writes, build the
full evidence chain (standing -> SHACL/ShEx/N3 -> Datalog closure -> PDDL
plan -> POWL compile -> client smoke + receipt verify + benchmarks -> OCEL
record -> wasm4pm validate), and only then `promote-claim` lawfully
(gated on `claim-demoted`, `datalog-closed`, and `wasm4pm-validated` all
holding on the correct objects) before `render-final-verdict` can fire
(gated on both the lawful promotion and the side-effect classification).

## Determinism proof

Two independent `plan run` invocations over the same `goal.ttl`, into two
different `--out-dir`/`--receipts-dir` pairs (`/tmp/pdl_det_run_a` and
`/tmp/pdl_det_run_b`):

- run A `powl_chain_hash`: `blake3:d9d50a2f561f0c54fd9e655cac6ef4c96b99b91bbb09d2148a804a68608cb658`
- run B `powl_chain_hash`: `blake3:d9d50a2f561f0c54fd9e655cac6ef4c96b99b91bbb09d2148a804a68608cb658`

Identical. Matches invariant 3 (no wall clock in the hash path) and the
existing `tests/plan_run_e2e.rs::two_runs_identical_chain_hashes` contract
for a different fixture.

## Receipt verification

`receipt validate --dir docs/case-studies/autonomic-standing-factory/case-study/pddl-receipts`:

```json
{
  "verdict": {
    "ok": true,
    "stages": [
      { "stage": "schema", "outcome": "Pass" },
      { "stage": "chain_recompute", "outcome": "Pass" },
      { "stage": "chain_linkage", "outcome": "Pass" },
      { "stage": "monotonic", "outcome": "Pass" },
      { "stage": "token_replay", "outcome": "Pass" }
    ],
    "records_checked": 1
  }
}
```

All five stages pass.

## POWL process model

Extended `src/bin/ocel_process_validate.rs` with a `--model
{release-v26.7.6,case-study}` flag (default `release-v26.7.6`, i.e. omitting
the flag is byte-for-byte the old behavior — same `DEFAULT_LOG`, same
`CHILD_SPECS`/`ORDER_LABEL_PAIRS`, same full integrity+UTC+conformance+
participation pipeline). `release_loop_model()`'s construction logic was
factored out into a shared `build_loop_model(specs, pairs)` helper; the
v26.7.6 `CHILD_SPECS`/`ORDER_LABEL_PAIRS` **values** are untouched (same
array literals, same order, same `ChildSpec` variants).

`--model case-study` builds a new, parallel `CASE_STUDY_CHILD_SPECS`
(16 `Once` leaves) / `CASE_STUDY_ORDER_LABEL_PAIRS` table with exactly the
events and partial order specified:

```
case_study_started < standing_emitted <
  {shacl_validated, shex_validated, n3_materialized < datalog_closed} <
  pddl_plan_generated < powl_model_compiled <
  {client_smoked, receipts_verified, benchmarks_attached} <
  graphlaw_judgment_emitted < ocel_log_written <
  wasm4pm_process_validated < final_verdict_rendered < case_study_finished
```

Because this case study's own OCEL log does not exist yet (Lane 4 produces
`case-study/ocel_*.json`), `--model case-study` does not attempt a full
integrity/UTC/conformance/participation run (there is nothing to validate
against) — it builds the model, classifies it through the same
`model_view`/`model_report` machinery as the release path, and writes the
`ModelReport{alphabet,children,order_pairs}` projection to
`case-study/powl_model.json`: 16 children, 114 order pairs (after
transitive closure), alphabet size 16. This is an honest scope boundary,
not a shortcut: the model is real and usable by Lane 4/6, but no
conformance verdict is claimed without a log to check it against.

**Deviation note for Lane 6's `PROCESS_MODEL.md`**: the partial order above
is asserted from this ticket's specification, not mined from an observed
trace (none exists yet). If Lane 4's actual OCEL capture shows
`ocel_log_written` occurring earlier than this model implies (e.g. the log
is opened/appended-to before `graphlaw_judgment_emitted` in real execution
order), that is a genuine model/reality mismatch Lane 6 must resolve
explicitly in `PROCESS_MODEL.md` — do not silently edit
`CASE_STUDY_ORDER_LABEL_PAIRS` to hide it.

## Tests passed

- `cargo test --bin ocel_process_validate`: **8/8 passed before this
  change, 8/8 passed after** (`canonical_trace_is_a_member`,
  `missing_required_event_is_rejected`, `order_violation_is_rejected`,
  `repeated_once_event_is_rejected`, `broken_benchmark_pattern_is_rejected`,
  `project_dedupe_drops_foreign_and_collapses_repeats`,
  `utc_parser_accepts_z_and_rejects_offsets_and_regressions`,
  `membership_agrees_with_language_upto`) — the v26.7.6 model's regression
  gate is unaffected, confirming the hard requirement.
- `cargo test --features ggen --lib --bins`: 101 lib tests + 13
  `my-conforming-project` bin tests + 8 `ocel_process_validate` bin tests,
  all passed.
- `cargo test --features ggen --test plan_run_e2e`: 3/3 passed
  (`dry_run_solve_and_powl_compile`, `full_loop_after_neon_fixture`,
  `two_runs_identical_chain_hashes`) — the pre-existing golden fixture
  (`examples/v26_7_6_after_neon/goal.ttl`) is untouched and still green.
- `cargo build --workspace --features ggen`: green.

## Failures found

None. The PDDL domain was authored to be reachable on the first attempt
(bounds were checked against `wasm4pm-compat::pddl::PDDL8_MAX_*` before
writing the TTL); no infeasible intermediate version was produced or
discarded.

## Repairs made

None required in shared code beyond the additive `--model` flag. One
self-correction during this lane's own work: the first `cargo run --bin
ocel_process_validate` invocation (default/release path, run to establish a
baseline before adding `--model`) rewrote
`docs/releases/v26.7.6/ocel/wasm4pm-process-validation.json`'s
`validated_at_utc` field (evidence-time only, not a hash input, per the
validator's own documented closure rule) — this out-of-scope side effect
was reverted with `git checkout --` before committing, so the v26.7.6
release evidence file carries no unrelated diff from this lane.

## Remaining external side effects

None from this lane. All commands ran against the local praxis checkout;
no network calls, no publishes, no pushes. The `/tmp/pdl_det_run_{a,b}` and
`/tmp/pdl_det_receipts_{a,b}` directories used for the determinism proof are
outside the repo and were not copied into tracked output.

## Handoff to next lane

- Lane 4 (OCEL v2 + wasm4pm): the case-study POWL model
  (`case-study/powl_model.json`, alphabet of 16 event-type names) is the
  event-type vocabulary Lane 4's own OCEL log should emit for a genuine
  conformance run of the case-study model itself (as opposed to the PDDL
  repair plan's action names, which are a separate vocabulary already
  captured in `case-study/pddl_plan.json`). Lane 4 can re-run
  `ocel_process_validate --model case-study <log-path>` once that log
  exists — the current binary accepts a positional log-path arg alongside
  `--model case-study` but only records it (does not validate it) per this
  lane's documented scope boundary; extending `run_case_study` to perform a
  full validation pass once a real log exists is Lane 4/6's call, not
  pre-empted here.
- Lane 6 (reports): see the deviation note above for `PROCESS_MODEL.md`
  regarding the asserted-not-mined case-study order pairs.
- Lane 2 (GraphLaw): no dependency in either direction — this lane's
  `datalog-closed` predicate in the PDDL domain models a precondition on
  Lane 2's actual Datalog closure existing, but the PDDL domain itself does
  not consume Lane 2's `case-study/rules/readiness.dl.n3` output directly
  (that wiring, if wanted, is Lane 6's evidence-manifest job).

## Evidence paths

- `docs/case-studies/autonomic-standing-factory/case-study/pddl/goal.ttl`
- `docs/case-studies/autonomic-standing-factory/case-study/pddl-out/domain.pddl`
- `docs/case-studies/autonomic-standing-factory/case-study/pddl-out/problem.pddl`
- `docs/case-studies/autonomic-standing-factory/case-study/pddl-out/plan.json`
- `docs/case-studies/autonomic-standing-factory/case-study/pddl_domain.pddl`
- `docs/case-studies/autonomic-standing-factory/case-study/pddl_problem.pddl`
- `docs/case-studies/autonomic-standing-factory/case-study/pddl_plan.json`
- `docs/case-studies/autonomic-standing-factory/case-study/pddl-receipts/receipts.jsonl`
- `docs/case-studies/autonomic-standing-factory/case-study/powl_model.json`
- `src/bin/ocel_process_validate.rs`
- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md`
  (phase rows 10-11)
- git commit `9081a8f` (`/Users/sac/praxis`, branch `main`) — "feat(case
  -study): add PDDL repair domain and case-study POWL process model -
  v26.7.6 model unchanged"
