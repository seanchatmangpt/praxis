# Deployment Plan — Praxis v26.7.6 "After Neon"

Target: a pilot deployment of the local-first standing-manufacturing loop on
a machine other than the development workstation, with receipts proving each
stage. Readiness baseline: `docs/releases/v26.7.6/FORTUNE5_READINESS.md`
(overall DEMO_READY; nothing PRODUCTION_READY). This plan is the path from
DEMO_READY to PILOT_READY. Every step has a gate; gates are receipts or
recorded command outputs, not assertions.

## Phase 0 — Close release exit criteria (blocker for everything)

From RELEASE_CONTROL.md Sec. 5:

1. `just verify-all` green, output captured (criterion 1 — currently UNKNOWN).
2. One-command full-loop demo, byte-identical receipts across 2 runs
   (criterion 3 — NOT STARTED). Build on `src/plan_run.rs` +
   `examples/v26_7_6_after_neon/` (commit `8336f29`).
3. Refusal tests per command in the Sec. 3 command table (criterion 5 —
   NOT STARTED). Unknown input → named `Refusal`, never panic/default.
4. Receipt-chain verification output recorded (criterion 7 — UNKNOWN;
   procedure per `crates/ggen/tests/receipt_chain_e2e.rs`).

Gate: RELEASE_CONTROL.md Sec. 5 shows all four green with evidence rows.

## Phase 1 — Packaging (single-checkout install)

- Deliverable: a pinned-toolchain checkout that builds with
  `cargo build --release` on a clean macOS/Linux machine (rust-version 1.82
  pinned in root `Cargo.toml:5`).
- Resolve the two build-coupling risks first (both in INVENTORY.md "Known
  couplings"): the optional `ggen-graph` path dep on the frozen `~/ggen`
  repo (root `Cargo.toml:52,80` — replaced in function by
  `crates/praxis-graphlaw`, commit `564543d`; remove or fence the feature),
  and the `lsp-max` hardcoded-path patch (`Cargo.toml:153`, MISSING
  tower-lsp-max lineage — make the feature optional-off by default).
- Decide `praxis-reconciler`: adopt into workspace members or delete
  (INVENTORY.md — currently orphaned/untested). No orphan code ships.
- Emit an SBOM (`cargo cyclonedx` or `cargo tree` snapshot) into the
  release directory.

Gate: clean-machine build receipt (command output + toolchain versions)
recorded in RELEASE_CONTROL.md Sec. 8; SBOM file checked in.

## Phase 2 — Cold-start verification lane

- Script the Lean lane cold start: fetch pinned Mathlib prebuilt cache
  (approach from commit `dab70b7`), `lake env lean` over
  `tools/paper-factory/lean-lake/Praxis/Corpus/`, diff verdicts against
  `mathlib_migration_receipts.jsonl`.
- Gate: second-machine verdict diff is empty (or diffs are explained and
  recorded — no silent tolerance).

## Phase 3 — Pilot execution (30 days, per FORTUNE5_READINESS.md Sec. 15)

- Week 1: install on the pilot machine (Phase 1 artifact); run the
  one-command demo; hand the operator the receipt chain and the replay
  command.
- Weeks 2–3: pilot workload — admit one partner artifact class as graph
  facts (closed-vocabulary mapping doc first, pattern:
  `docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md`), plan → workflow → generate →
  receipt. All data stays on the pilot machine (local-first boundary,
  FORTUNE5_READINESS.md Sec. 3/5).
- Week 4: pilot exit report — receipts delivered, replay demonstrated by
  the operator without the developer present, refusal behavior demonstrated
  on a malformed input, gap list filed as tickets.

Gate: operator-run replay receipt matches developer-run receipt
byte-for-byte.

## Phase 4 — Hardening backlog (post-pilot, 90-day path)

Ordered from FORTUNE5_READINESS.md Sec. 11/16 threat gaps:

1. Signed receipts (key-backed attestation over the BLAKE3 chain — hashes
   alone don't prove authorship).
2. Hosted CI running `just verify-all` + cargo-audit + SBOM on every commit.
3. Template write-boundary test in the ggen factory (templates must not
   write outside the sync root).
4. Wall-clock lint: a test that scans hash/receipt paths for clock calls,
   mechanizing invariant 3 (`ts_ns=0`).
5. Differential test over one bcinr hot path (ADVERSARIAL_REVIEW.md
   challenge 6 — currently OPEN).

## Non-goals for v26.7.6 deployment

No hosted/multi-tenant service, no REST API, no SSO, no production SLA, no
Fortune-5 production commitment. Each of these is
ENTERPRISE_HARDENING_REQUIRED per FORTUNE5_READINESS.md and out of scope
until the pilot exit report exists.

## Rollback and hygiene

FIX FORWARD ONLY: no destructive git operations; deployment fixes land as
new commits. Pilot machines get tagged release checkouts, never the dirty
development workspace (RELEASE_CONTROL.md Sec. 9 records the commit-hygiene
risk).
