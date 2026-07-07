# wasm4pm Validation Report

Real results from Lane 4's `case-study/wasm4pm_validation.json`, re-run and
re-confirmed by Lane 6 (`cargo run --bin ocel_process_validate --
docs/case-studies/autonomic-standing-factory/case-study/ocel_case_study.json
--model case-study`, exit 0). Lane 6's re-run reproduced the identical
`is_conforming`/`fitness`/`violations`/`event_count`/`object_count` values
below (only the `validated_at_utc` sidecar timestamp changed, per Lane 4's
own documented allowed-volatility contract — the checked-in file's other
content, and therefore its sha256, was restored to Lane 4's original after
Lane 6's confirmation run).

## `case-study/wasm4pm_validation.json` (Criterion09)

```json
{
  "is_conforming": true,
  "fitness": 1.0,
  "violations": [],
  "integrity_report_summary": { "valid": true, "error_count": 0, "error_codes": [] },
  "event_count": 20,
  "object_count": 11,
  "ocel_sha256": "5260a884bd70bb0c598843f9cfa650b67100cc4d057c352ef8adde43ebb8c8cb",
  "ocel_blake3": "bdee3ea0a8de49dfd1f8dd113be60fff5acf227ce3820106a39946d10bffacca",
  "model_ref": "case-study",
  "ocel_ref": "docs/case-studies/autonomic-standing-factory/case-study/ocel_case_study.json"
}
```

sha256 of the file (Lane 4's original, restored): `62403c522bb610694529451d9ad6d31e328ed7c8e28b242c62505f37181c092a`.

Method: "library composition: `wasm4pm_compat::ocel::validate` (OCEDO/OCPQ
integrity) + `powl2_decompose::Powl` case-study model with the same direct
membership decision procedure used for the release model." The wasm4pm CLI
conformance command is stubbed
(`/Users/sac/wasm4pm/crates/wasm4pm-cli/src/commands/mining.rs:25`), hence
this in-process composition — same architecture Lane 4 documented for the
v26.7.6 release model's own validation.

## `case-study/standing_ocel_validation.json` (Lane 1's standing OCEL, structural proof)

```json
{
  "valid": true,
  "event_count": 28,
  "object_count": 28,
  "parse_errors": [],
  "log_ref": "target/praxis-standing/standing.ocel.json"
}
```

Via the new `--model standing-integrity` mode (Lane 4) — structural-only
OCEDO/OCPQ `validate` (no process model applies to a standing snapshot, it
is not a process log).

## Regression: release-v26.7.6 model unaffected

`cargo run --bin ocel_process_validate` (default, no `--model` flag —
byte-for-byte the pre-Lane-3 release path) re-run by Lane 6:
`[ocel_process_validate] conforming; bookkeeping already present, skipped`,
exit 0. `cargo test --bin ocel_process_validate`: **8/8 passed**
(`canonical_trace_is_a_member`, `missing_required_event_is_rejected`,
`order_violation_is_rejected`, `repeated_once_event_is_rejected`,
`broken_benchmark_pattern_is_rejected`,
`project_dedupe_drops_foreign_and_collapses_repeats`,
`utc_parser_accepts_z_and_rejects_offsets_and_regressions`,
`membership_agrees_with_language_upto`) — unchanged after Lane 6's
verification re-runs.

Note: re-running the release-model pass touches
`docs/releases/v26.7.6/ocel/wasm4pm-process-validation.json`'s
`validated_at_utc` sidecar field only (evidence-time, not a hash input, per
the validator's own documented closure rule — the same non-hash-affecting
field Lane 3 already found and reverted). Lane 6 reverted this file to its
committed state after confirming the regression pass, per Lane 3's
precedent, so this release evidence file carries no incidental diff from
this lane's verification pass.
