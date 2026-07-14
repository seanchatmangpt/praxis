# Receipt Verify — OCEL Evidence Pass (v26.7.6)

Ledger validation observed by the OCEL evidence driver
(`clients/autonomic-platform/tests/run-evidence-pass.mjs`, run `ocel-evidence-2026-07-14T03-38-08-500Z`).

## Command

```
/Users/sac/praxis/target/debug/my-conforming-project receipt validate --dir target/plan_run/ocel_pass_receipts
```

cwd: repo root. Exit code: 0.

## UTC window

- started_at_utc: 2026-07-14T03:38:22.398Z
- finished_at_utc: 2026-07-14T03:38:22.427Z
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
    "records_checked": 4
  }
}
```

## Hashes

- plan-run ledger head `chain_hash_hex`: `a419aa299c8a67e877888a27e05ffd8be9cd7e4096356181e537986a44e22eb0`
- `powl_chain_hash` (both runs, deterministic): `blake3:1f97313c12be8f1f4b295970aaff506a79c1533be7a8abffb69c2ec8c677e9bb`
- raw capture: `docs/releases/v26.7.6/ocel/raw/receipt-validate.txt` sha256 `009bcad81ce8cc0cfccbeb9ac67c1d8ee454fee888c2c09b918800a69b1f5491`
