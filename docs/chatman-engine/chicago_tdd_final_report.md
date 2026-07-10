# Chatman Engine v26.7.9 — Gate F Final Report

ADMITTED_DRY_RUN_PUBLISHABLE

## Audit basis

- Commit: `git log -1 --oneline` → `7d76019 PROJ-411..417: Chatman Engine v26.7.9 closure —
  RDF-native fixtures, deterministic OCEL, praxis-local snapshot, gates A–E evidence`
- `git status --porcelain` → clean at audit start; no cargo processes running.
- All conclusions below re-derived by command in this session (non-authoring auditor).

## Scope

Chatman Engine v26.7.9 S1–S6 core admission, planning, workflow-check, receipt, replay, and
evidence pipeline — with these named exclusions (not covered by this verdict):

1. N3 cubic-scaling work (commit `7765777`) — analysis only, untouched.
2. Deferred S3→S4 OrchestratedPlan/TapeBridge engine-side projection wiring — verified absent:
   `grep -c orchestrated_plan crates/praxis-graphlaw/src/chatman/engine.rs` → `0`
   (the committed `bridge.rs` types remain in scope).
3. PROJ-415 (SHACL CompiledShape population) — separate OPEN ticket.
4. PROJ-416 (Pattern-4 canonical-render receipt consumers) — separate OPEN ticket.
5. PROJ-417 (WASM full-pipeline HashMismatch replay) — separate OPEN ticket.
6. Crate-wide non-Chatman clippy debt (predates `2dd4f04`) — see Gate E evidence.

## Gate A — Substrate: PASS

- `grep -n "nightly-2026-06-22" rust-toolchain.toml` → `channel = "nightly-2026-06-22"` (line 11).
- `grep -nE "oxigraph-local|oxrdf-patched" Cargo.toml` → no matches (no local patches).
- `cargo tree -e normal -p praxis-graphlaw | grep -E "oxrdf v|oxigraph v" | sort -u` →
  only `oxrdf v0.3.3` (single version) and `oxigraph v0.5.9`.
- `cargo tree -e features -p praxis-graphlaw | grep rdf-12` →
  `oxigraph feature "rdf-12"` (plus oxrdfio/oxrdf/oxjsonld rdf-12) — feature ON.
- `grep -rn "TripleTermInSnapshot" crates/praxis-graphlaw/src/chatman/` → variant defined
  (`abi.rs:334`), named (`abi.rs:369,411`), enforced in `engine.rs` (refusal at the receipt
  boundary, `engine.rs:512` doc + implementation).

## Gate B — Code: PASS

- `ls crates/praxis-graphlaw/src/chatman/{abi,triple8,admission8,router,engine,bridge}.rs` →
  all six files exist.
- Enum-scoped variant count:
  `awk '/pub enum Refusal/,/^}/' crates/praxis-graphlaw/src/chatman/abi.rs | grep -cE
  "^\s+[A-Z][A-Za-z0-9]*\s*[({,]"` → `29`.
- `grep -n "struct AdmittedTransition" .../chatman/engine.rs` → `engine.rs:284`; all fields
  private, read-only accessors only; doc states the only constructor is
  `ChatmanEngine::admit_transition` (sealed, receipt-bearing).
- `EngineProcessReceipt` (alias `ProcessReceiptEnvelope`, `engine.rs:244`) carries exactly 9
  constitutional digests (graph_snapshot, profile, symbol_table, projection, admission_table,
  route_decision, tape, hook_event, engine_version) plus the derived `receipt_root`.
- `verify_replay` (`engine.rs:687`) re-checks all 9 fields independently via a
  `[(&Digest, &Digest, Ctor); 9]` per-field scan with a distinct `ReplayMismatch` variant per
  field, then recomputes `receipt_root` — per-field, not whole-envelope.
- `cargo check -p praxis-graphlaw` → `Finished dev profile ... in 15.52s` (compiles; 16 lib
  warnings are outside the Chatman surface, see Gate E).

## Gate C — Tests: PASS

- `grep -rL "assert" crates/praxis-graphlaw/tests/chatman_acceptance_*.rs` → no assert-free
  generated file.
- Full suite: `cargo nextest run -p praxis-graphlaw -E 'binary(~chatman)'` →
  `Summary [0.362s] 123 tests run: 123 passed, 5 skipped`.
- `cargo test -p praxis-graphlaw --test chatman_static_gates` →
  `ok. 11 passed; 0 failed` (includes duplicate-canonical-types gate).
- Diagram atlas: `python3 docs/chatman-engine/diagrams/atlas/verify_atlas.py` →
  `VERIFICATION RESULT: PASS`.
- Tick gate: `tests/chatman_hotpath.rs` exercises `AdmissionTable8::lookup` across all 256
  states as a pure single-indexed load (`lookup_is_single_indexed_load_across_all_256_states`,
  passing within the 123).
- Gate C adjudication (`evidence/gate_c_adjudication.md`) judged independently: the current
  DoD Gate C carve-out matches the actual repo layout (8 ggen dispatch files, static gates,
  spec theorems, structural/governance tests, proptest-internal items); the raw-`#[test]`
  census matches what I observe; declining a mass macro conversion is sound — it would rewrite
  passing test mechanics for cosmetic conformance, which the DoD itself forbids. Adjudication
  accepted.
- Note on the 5 skips: `tests/chatman_engine_acceptance/properties.rs` carries five
  `#[ignore = "CENG: engine lands in next phase; ... doc-stub module today"]` properties,
  documented fail-loud (never pass vacuously). The stated rationale is stale — `chatman::engine`
  et al. are now implemented — but ignored lane sketches are not a Gate C criterion. Recorded
  as a follow-up, not a gate failure.

## Gate D — Evidence: PASS

- `just chatman-sync-verify` (double `ggen sync` + `git diff --exit-code -- 'crates/praxis-
  graphlaw/src/chatman' 'docs/chatman-engine'`) run from the clean committed tree → exit `0`
  (verified via `just`'s own exit status, not a piped tail). Only `.ggen-v2/receipt*.json`
  bookkeeping changed, outside the gated paths; restored via `git checkout`.
- OCEL: 8 suites regenerated from scratch twice
  (`rm -rf .cargo-cicd/ocel/chatman` between runs; `cargo test -p praxis-graphlaw --test
  chatman_acceptance_<suite>` for admission agents hooks receipts replay routing static
  triple8). `shasum -a 256 .cargo-cicd/ocel/chatman/*.receipt.json` byte-identical across both
  runs (e.g. `hook.receipt.json` =
  `b74d420b8cf8ee9acc9ae619f203e9cae6c4a1afddd91ed9cabc656c339bf874` both runs) and identical
  to the committed copies (`git status` showed no `.receipt.json` modification). All 16 files
  exist and are non-empty.
- Determinism: `cargo test -p praxis-graphlaw --test chatman_e2e_pipeline` run 5 consecutive
  times → `ok. 1 passed` each run; the test itself
  (`s1_through_s6_pipeline_is_byte_identical_across_five_independent_runs`) asserts
  receipt_root byte-identity across five independent engine runs internally.
- Finding (non-gating): the `*.ocel.json` log bodies are NOT byte-deterministic across runs —
  `event_id` fields are fresh UUIDs from chicago-tdd-tools' sealer and event numbering varies
  with test interleaving. The sealed digest in `<suite>.receipt.json` (computed by `seal_run`
  over canonical event content) is what DoD Gate D item 2 gates, and it is deterministic.
  Recommend either canonicalizing `event_id` in the tooling or noting the exclusion in the DoD.

## Gate E — Quality (Chatman-scoped): PASS (items 4–5 advisory, UNVERIFIED)

- `cargo fmt -p praxis-graphlaw --check` → exit 0, no diff anywhere (Chatman surface clean).
- `cargo clippy -p praxis-graphlaw --all-targets -- -D warnings > /tmp/clippy_full.txt` →
  exit 101 crate-wide (68 error/warning lines), but
  `grep -cE "src/chatman|tests/chatman" /tmp/clippy_full.txt` → `0`: zero findings touch
  `src/chatman/` or `tests/chatman_*`. All findings are in tripleindex.rs (9),
  hooks/ (17), reasoner/ (7), sparql/ (4), owlrl/ (4), shacl/ (7), queryengine/ (2),
  bindings.rs (2), lib.rs (3), parser_edge_cases_test.rs (2), validate.rs (1) — the documented
  preexisting-debt exclusion (named exclusion 6; note lib.rs and parser_edge_cases_test.rs are
  additional out-of-surface locations beyond the exclusion's listed spans).
- Forbidden production tokens:
  `grep -rnE "\.unwrap\(\)|\.expect\(|panic!\(|todo!\(|unimplemented!\(|unwrap_or_default|
  SystemTime|Instant::now|unsafe " crates/praxis-graphlaw/src/chatman/` → 0 hits (even in
  test modules); `grep -rn "\.ok()" crates/praxis-graphlaw/src/chatman/` → 0 hits.
- Duplicate canonical types:
  `grep -rhoE "pub (struct|enum) [A-Za-z0-9_]+" crates/praxis-graphlaw/src/chatman/*.rs |
  sort | uniq -d` → empty; corroborated by the passing
  `gate_no_duplicate_canonical_types_in_crate` static gate.
- Mutation score (item 4) and line coverage (item 5): UNVERIFIED — advisory for this closure
  per the audit instructions; not run this session.

## Snapshot locality: PASS

- Baseline lives in praxis:
  `crates/praxis-graphlaw/tests/snapshots/chicago_tdd_tools__testing__snapshot__chatman_s1_receipt_shape.snap`.
- `cargo test -p praxis-graphlaw --test chatman_snapshot_semantics` → `ok. 3 passed; 0 failed`.
- Finding (outside this repo, non-gating): a stale pending file
  `/Users/sac/chicago-tdd-tools/src/testing/snapshots/...chatman_s1_receipt_shape.snap.new`
  exists in the chicago-tdd-tools repo — a leftover insta `.snap.new`, not an accepted
  baseline; recommend deleting it there.

## Verdict

All Gate A–E binding criteria pass with the command output cited above; the only unverified
items (mutation, coverage) are explicitly advisory for this closure. No nonlocal blocker
exists. Verdict: ADMITTED_DRY_RUN_PUBLISHABLE, scoped exactly to the S1–S6 surface with the
six named exclusions above.

Signed: independent Gate F auditor (non-authoring session), 2026-07-10T01:23:35Z
