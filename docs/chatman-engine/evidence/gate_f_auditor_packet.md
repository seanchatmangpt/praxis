# Gate F Independent Auditor Packet

Hand this file (and nothing else from chat history) to a fresh agent/session that did NOT
author any of this run's fixes. The auditor must re-derive every conclusion from repo state
and command output, not from this packet's prose — this packet only points at where to look.

## Scope
Chatman Engine v26.7.9 S1–S6 core admission, planning, workflow-check, receipt, replay, and
evidence pipeline.

## Commit
Run `git -C /Users/sac/praxis log -1 --oneline` and `git -C /Users/sac/praxis status
--porcelain` yourself — do not trust a pasted value.

## Explicit exclusions (do not grade against these)
- N3 cubic-scaling work (commit 7765777) — untouched, out of scope by user instruction.
- Deferred S3→S4 OrchestratedPlan/TapeBridge projection — reverted, not part of this
  artifact (the committed `bridge.rs` types are in scope; the engine-side projection
  wiring is not — verify `grep -c orchestrated_plan crates/praxis-graphlaw/src/chatman/engine.rs`
  returns 0).
- PROJ-415 (SHACL CompiledShape population), PROJ-416 (Pattern-4 canonical-render receipt
  consumers), PROJ-417 (WASM full-pipeline HashMismatch replay) — separate OPEN tickets,
  outside the S1–S6 surface.
- Crate-wide non-Chatman clippy debt (predates commit 2dd4f04) — Gate E is Chatman-scoped
  per the corrected DoD; out-of-surface clippy findings are documented preexisting debt,
  not gate failures.

## Required independent re-derivation
- Gate A: `cargo tree -e features -p praxis-graphlaw | grep rdf-12` (expect non-empty).
- Gate B: per `docs/chatman-engine/DEFINITION_OF_DONE.md` Gate B section, run its literal
  commands yourself.
- Gate C: run the full chatman suite; read `docs/chatman-engine/evidence/gate_c_adjudication.md`
  for the DoD/repo-layout mismatch and judge independently whether the adjudication is sound —
  do not accept it on the builder's word.
- Gate D: re-run `docs/chatman-engine/evidence/gate_d_determinism_plan.md`'s commands yourself.
- Gate E (Chatman-scoped): `cargo fmt -p praxis-graphlaw --check` (no diff in
  Chatman-surface files) and full (not tail-truncated)
  `cargo clippy -p praxis-graphlaw --all-targets -- -D warnings` captured to a file; the
  gate passes when zero findings touch `crates/praxis-graphlaw/src/chatman/` or
  `crates/praxis-graphlaw/tests/chatman_*`; out-of-surface findings are the documented
  preexisting-debt exclusion.
- Snapshot locality: verify the `chatman_s1_receipt_shape` baseline lives at
  `crates/praxis-graphlaw/tests/snapshots/` inside praxis (not in chicago-tdd-tools) and
  `cargo test -p praxis-graphlaw --test chatman_snapshot_semantics` passes 3/3.
- OCEL: verify `.cargo-cicd/ocel/chatman/*.json` exist, are non-empty, and are deterministic
  across two independent runs (rm -rf the dir between runs, diff digests).
- Duplicate canonical types: independently grep for duplicate `pub struct`/`pub enum` names
  across `crates/praxis-graphlaw/src/chatman/*.rs`.
- Forbidden production tokens: independently grep `crates/praxis-graphlaw/src/chatman/` for
  `.unwrap()`, `.expect(`, `panic!(`, `todo!(`, `unimplemented!(`, `unwrap_or_default`,
  bare `.ok()`, `SystemTime`, `Instant::now`, `unsafe` (test modules exempt).
- Standing index: do NOT refresh it yourself — that happens only after you write the verdict.

## Verdict vocabulary (this run's override of the DoD's own broader vocabulary)
Write exactly one of:
- `ADMITTED_DRY_RUN_PUBLISHABLE` — all gates pass with cited command output, no self-reports.
- `REFUSED_WITH_NONLOCAL_BLOCKER` — cite the exact blocker and prove it requires external
  credentials, unavailable infrastructure, destructive user approval, or a policy decision
  genuinely outside this repository. Missing tests/snapshots/flags/OCEL wiring/stale docs are
  NOT nonlocal blockers by this run's own rules — if that's all that's wrong, it must be fixed,
  not refused around.

Write the verdict into `docs/chatman-engine/chicago_tdd_final_report.md`, sign it as the
independent auditor (not the builder session), and stop — do not proceed to standing refresh.
