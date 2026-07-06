# Receipts — Praxis v26.7.6 "After Neon"

Date: 2026-07-06. Companion to TEST_REPORT.md. Every hash below was
recomputed by a verifier this session, not copied from an earlier claim.

## 1. Chain state

| Chain | Head / hash | Records | Verifier | Verdict |
|---|---|---|---|---|
| Demo ledger (`target/plan_run/after_neon_det_receipts/receipts.jsonl`) | `chain_hash_hex fcf49d5688f9c32d8e62fe522342e2be2bbf1c0440cda8aec06ac3bb14bdcffb` | 1 (fresh sandbox ledger, genesis-anchored) | `my-conforming-project receipt validate --dir …` | ok: true — schema, chain_recompute, chain_linkage, monotonic, token_replay all Pass |
| POWL execution chain (in the demo output) | `powl_chain_hash blake3:1f97313c12be8f1f4b295970aaff506a79c1533be7a8abffb69c2ec8c677e9bb` | 4 fired slots | two independent runs + `tests/plan_run_e2e.rs::two_runs_identical_chain_hashes` | identical across runs; test green in `just verify-all` round 6 |
| Factory sync ledger (`.ggen/receipts/`) | `35bc4ab0c984ed5198e2609ec771f17a24d020d6e6882c2bb82ea6feab04765a` | 8 | `ggen receipt verify` + `ggen receipt history` | valid: true (both) |
| Lean lane (`tools/paper-factory/lean-lake/mathlib_migration_receipts.jsonl`) | 202 per-label records (commit `1ea2385`) | 202 | `lake build` replay, 2026-07-06 | Build completed successfully (826 jobs), exit 0 |

## 2. Receipt schema (demo ledger record, observed)

The single record in the deterministic demo ledger, field by field:

| Field | Value | Invariant it proves |
|---|---|---|
| `version` | 1 | schema versioning |
| `instruction_id`, `activity_idx`, `node_kind` | 0, 0, 0 | POWL binding slots |
| `ts_ns` | **0** | invariant 3 — no wall clock in any hash/receipt path |
| `payload_hash_hex` | `95d4cd79a7ebc5f9e41fa3b1253cc7780896068e0774fcf9ef339b7bb980fcb5` | payload is content-addressed (BLAKE3) |
| `prev_chain_hash_hex` | `00…00` (32 zero bytes) | genesis anchor — invariant 2, chains fold from genesis |
| `chain_hash_hex` | `fcf49d5688f9c32d8e62fe522342e2be2bbf1c0440cda8aec06ac3bb14bdcffb` | computed, then *recomputed* by `receipt validate` (chain_recompute stage Pass) — never asserted-in |
| `andon` | `Green` | lifecycle state bound into the admission frame |
| `obligation_count` | 0 | bound into the frame hash since commit `4190e71` |
| `object_ids` | `["law:95d4cd79a7ebc5f9"]` | object binding derived from the payload hash |

## 3. Verification result (criterion 7)

`receipt validate` output (exit 0):

- schema: Pass
- chain_recompute: Pass — the verifier recomputed every `chain_hash_hex`
  from `payload_hash_hex` + meta + `prev_chain_hash_hex`; a tampered record
  fails here (pinned by `tests/` chain-tamper suites).
- chain_linkage: Pass — each `prev` equals the prior record's chain hash.
- monotonic: Pass.
- token_replay: Pass — POWL token replay conforms to the recorded firing.

Determinism cross-check: deleting the ledger and re-running the identical
demo command reproduced the ledger **byte-identically** (`diff -r` clean),
and the two full run-output captures hash to the same SHA-256
(`ebab9f63f6214c830075064f016227f2bb13c2960cd1337c027cd2789d194d8e`).

## 4. Evidence artifact digests

| Artifact | SHA-256 |
|---|---|
| `just verify-all` green log (round 6) | `bdb7063dc981c32823ff882e694f35dd7090346e5b9a701e8a2239e592cea806` |
| demo run 1 output JSON | `d65b5befaf4082c3b2e9c7aab0f8a518154a9e700732dab5baa3ad2225f49fe3` |
| demo run 2 output JSON | `e8dacf1304c21bd99d4a730a38b7138b234bdceaa4fc56bd237d690cfd21659d` |
| demo same-path runs 3 and 4 (identical) | `ebab9f63f6214c830075064f016227f2bb13c2960cd1337c027cd2789d194d8e` |
| `receipt validate` output | `01531056f4436f56207e262888f28b5e301e1d38103a4f7cdf4b5ae8c961f22d` |
| `domain.pddl` / `problem.pddl` / `plan.json` | `f569919b…7f246967` / `9330bbf2…2e54f315` / `52b999be…75105ff3a` |
| `lake build` log | `8f757aece326bd1aac56bfdd81003f3745bba434b74c4fccdd15a6e1d69312af` |
| `no-sorry` findings JSON | `58bfd757f926f48f0404eb5e1fbcc06a39b0b09c40382906e3657f723101344b` |
| publish dry-run log | `a9feb88e2c68e0851ff4f463f5c4c6ae89a907e635a09620b1349de41ee1de7b` |
| thesis PDF | `c46c43be320850093e755f495464adf5b6c41ccbf734cd368bbb745c8d82f0b7` |
| `arxiv-submission.tar.gz` | `67e0725f3c1299ca1b0a66f7b1e64fc4816612e1ece941fdb0185727875ad767` |
