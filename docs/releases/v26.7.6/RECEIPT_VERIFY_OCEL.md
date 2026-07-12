# Receipt Verify — OCEL Evidence Pass (v26.7.6)

Ledger validation observed by the OCEL evidence driver
(`clients/autonomic-platform/tests/run-evidence-pass.mjs`, run `ocel-evidence-2026-07-12T08-33-45-681Z`).

## Command

```
/Users/sac/praxis/target/debug/my-conforming-project receipt validate --dir target/plan_run/ocel_pass_receipts
```

cwd: repo root. Exit code: 0.

## UTC window

- started_at_utc: 2026-07-12T08:33:47.625Z
- finished_at_utc: 2026-07-12T08:33:47.654Z
- clock_source: system (evidence time only — never a hash input)

## Five validation stages

Per `receipt validate --help` (`src/ops.rs`), the validator runs, in order:

1. Schema — every ledger record parses against the receipt schema.
2. Chain-tamper detection — every `chain_hash_hex` is recomputed (BLAKE3)
   and compared against the stored value.
3. Chain linkage — each record's `prev_chain_hash_hex` equals the prior
   record's `chain_hash_hex` (genesis-folded head).
4. Monotonicity — record ordering is strictly monotone.
5. POWL token-replay conformance — the receipt sequence replays through the
   POWL workflow without violating the token game.

## Output

```json
{
  "verdict": {
    "ok": false,
    "stages": [
      {
        "stage": "schema",
        "outcome": "Pass"
      },
      {
        "stage": "chain_recompute",
        "outcome": "Pass"
      },
      {
        "stage": "chain_linkage",
        "outcome": "Pass"
      },
      {
        "stage": "monotonic",
        "outcome": {
          "Fail": "record 1: instruction_id (0) not strictly increasing after 0"
        }
      },
      {
        "stage": "token_replay",
        "outcome": "Pass"
      }
    ],
    "records_checked": 2
  }
}
```

## Hashes

- plan-run ledger head `chain_hash_hex`: `5862dd12bfbe1eb1256f9a6723d16d4baaa1a55610946451587901a788b70da9`
- `powl_chain_hash` (both runs, deterministic): `blake3:1f97313c12be8f1f4b295970aaff506a79c1533be7a8abffb69c2ec8c677e9bb`
- raw capture: `docs/releases/v26.7.6/ocel/raw/receipt-validate.txt` sha256 `d47f68de5a1bf55898679cb204655670837a566fe79999029c2ab54dd82d2cc4`
