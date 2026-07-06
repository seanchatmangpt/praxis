# Claim Rules

Every prose claim of readiness in this repo (a doc, a commit message, a PR
description, an agent's status report) is checked against the compiled
`praxis-standing.v1` index (`target/praxis-standing/standing.json`) by
`anti-llm-cheat-lsp`'s `ANTI-LLM-STANDING-000..006` diagnostics
(`/Users/sac/anti-llm-cheat-lsp/src/rules/standing.rs`). This document states
each rule in plain language with an example violation and fix. The LSP is the
enforcement mechanism; this table is the human-readable mirror of it.

Invocation: `anti-llm-cheat-lsp server scan --dir .` (one-shot) or
`anti-llm-cheat-lsp server serve` (editor LSP mode) — see
`ANTI_LLM_CHEAT_LSP_POLICY.md` in this directory for exact commands and config.

## The 6 rules

### ANTI-LLM-STANDING-000 — index missing, unparseable, or internally invalid

**Rule**: the configured `index_path` (default
`target/praxis-standing/standing.json`) must exist, parse as
`praxis-standing.v1` JSON, and every entry must pass the scoped-readiness
validation rule (any of `PRODUCTION_READY`/`PILOT_READY`/`PUBLISH_READY`/
`PUBLICATION_READY` requires a non-empty `scope`).

- **Violation example**: `just standing` was never run in this checkout, so
  `target/praxis-standing/standing.json` does not exist, but a doc claims
  "praxis-graphlaw is PUBLISH_READY."
- **Fix**: run `just standing` (or `cargo-cicd standing refresh`) to produce
  a fresh, valid index before making the claim.

### ANTI-LLM-STANDING-001 — unscoped readiness claim

**Rule**: any claim using `production-ready`, `pilot-ready`, `publish-ready`,
or `publication-ready` language must carry an explicit scope phrase ("for
`<scope>`" / "scoped to `<scope>`") in the same sentence. This check runs
even when `[standing]` is not configured in `anti.toml` — it is purely
textual.

- **Violation example**: "praxis-graphlaw is production-ready." (no scope
  stated)
- **Fix**: "praxis-graphlaw is PRODUCTION_READY for local release validation
  and crates.io dry-run."

### ANTI-LLM-STANDING-002 — claimed status outruns the index

**Rule**: a claim naming a specific standing status (`PRODUCTION_READY`,
`PILOT_READY`, `PUBLISH_READY`, `PUBLICATION_READY`, `OCEL_PROVEN`,
`WASM4PM_PROVEN`) for a named artifact must find that exact status in the
artifact's `standing` list in the index.

- **Violation example**: "the autonomic-platform client is OCEL_PROVEN" when
  `standing.json` lists that artifact's `standing` as `["BUILDS",
  "TESTED"]` only.
- **Fix**: either run the OCEL process-validation pass and re-refresh the
  index so `OCEL_PROVEN` actually appears, or downgrade the claim to what the
  index supports (`TESTED`).

### ANTI-LLM-STANDING-003 — "published" claimed without an operator-side-effect record

**Rule**: `praxis-standing.v1` has no `PUBLISHED` status — only
`PUBLISH_READY` (dry-run verified) and `EXTERNAL_OPERATOR_SIDE_EFFECT`
(an operator actually completed the external action) exist. A claim that
something is "published" must find a non-empty
`external_operator_side_effects` entry on that artifact.

- **Violation example**: "praxis-graphlaw is published to crates.io" when the
  standing entry only carries `PUBLISH_READY` and the
  `external_operator_side_effects` list still reads `["real crates.io
  publish requires operator credentials"]` (i.e. it has not happened).
- **Fix**: say "praxis-graphlaw is PUBLISH_READY (dry-run verified); real
  publish is pending operator action" until an operator runs `cargo publish
  -p praxis-graphlaw` and the side-effect entry is updated to reflect
  completion.

### ANTI-LLM-STANDING-004 — "alive"/"verified" claimed without receipt or OCEL backing

**Rule**: a claim that an artifact is "alive" or "verified" must find
`RECEIPT_VERIFIED` or `OCEL_PROVEN` in that artifact's `standing` list.

- **Violation example**: "the release is ALIVE" with no receipt-verify or
  OCEL evidence attached to the release artifact in the index.
- **Fix**: attach the receipt-verify run or OCEL log as evidence, refresh the
  index, then make the claim — or state the actual status (`TESTED`,
  `BUILDS`) instead.

### ANTI-LLM-STANDING-005 — "benchmarked" claimed without evidence

**Rule**: a `benchmarked` claim must find either the `BENCHMARKED` status or
at least one evidence entry on that artifact.

- **Violation example**: "praxis-graphlaw is benchmarked" with an empty
  `evidence` array and no `BENCHMARKED` status.
- **Fix**: run the benchmark, attach the raw output path
  (`docs/releases/v26.7.6/ocel/raw/bench-graphlaw.txt`) as a `command` or
  `artifact` evidence entry, refresh, then claim it.

### ANTI-LLM-STANDING-006 — stale index

**Rule**: `generated_at_utc` on the loaded index must be no older than
`max_index_age_secs` (default 86,400s / 24h) relative to wall-clock `now`.

- **Violation example**: `standing.json` was generated three days ago; a
  claim is made today without re-running `just standing`.
- **Fix**: re-run `just standing` before relying on the index for any claim
  older than the configured freshness window.

## Non-blocking exemption

All rules except `ANTI-LLM-STANDING-006`'s bare index-property check honor
`[surface].non_blocking_path_prefixes` in `anti.toml` — claims inside
vision/roadmap docs at an exempted path prefix are downgraded to
non-blocking (still reported, not gate-failing). This is the same mechanism
`ANTI-LLM-SURFACE-001` uses; standing does not invent a parallel exemption
list.
