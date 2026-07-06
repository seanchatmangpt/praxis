# Standing Schema v1 (`praxis-standing.v1`)

Shared schema for compiled standing documents (`standing.json`) that describe
the release-readiness of every artifact in a praxis release. This document is
the schema-of-record; `crates/cargo-cicd-core/src/standing/model.rs` in the
`cargo-cicd` repo carries the matching Rust types (`serde`-derived, same
field names, `SCREAMING_SNAKE_CASE` status tags).

## Relationship to existing praxis docs

- `docs/releases/v26.7.6/NO_TERMINAL_BLOCKERS.md` defines an 8-status
  resolution vocabulary (`TEMP_BLOCKED`, `RESOLVED_BY_REPAIR`,
  `RESOLVED_BY_EXISTING_SURFACE`, `RESOLVED_BY_LOCAL_EQUIVALENT`,
  `RESOLVED_BY_SCOPE_RECLASSIFICATION`,
  `RESOLVED_BY_EXTERNAL_OPERATOR_SIDE_EFFECT`, `OCEL_PROVEN`,
  `WASM4PM_PROVEN`) for *how a blocker on a single pass was closed*. This
  schema supersedes/extends that: `OCEL_PROVEN` and `WASM4PM_PROVEN` reappear
  here as standing statuses on individual artifacts (not just ledger rows),
  and the resolution-route statuses fold into
  `external_operator_side_effects` / evidence entries on the artifact rather
  than a separate ledger vocabulary.
- `docs/releases/v26.7.6/CLAIM_PROMOTION_TABLE.md` is the hand-written,
  per-release artifact of claim promotions (claim, prior status, evidence,
  OCEL event ids, final standing). The compiled `standing.json` produced
  under this schema is intended to eventually supersede that hand-written
  table: every row of the promotion table becomes a `StandingArtifact` with
  its `standing` list and `evidence` array populated from the same OCEL/
  wasm4pm event ids and artifact paths cited there today.

## The 20 statuses

| Status | Meaning |
|---|---|
| `UNSEEN` | Artifact has not been discovered/indexed by any tooling pass yet. |
| `DISCOVERED` | Artifact is indexed (path, kind known) but nothing has been run against it. |
| `BUILDS` | Artifact compiles/builds cleanly. |
| `TESTED` | Artifact's test suite passes. |
| `LINT_CLEAN` | Artifact passes lint/clippy with no warnings. |
| `BENCHMARKED` | Artifact has at least one attached benchmark result. |
| `RECEIPTED` | A receipt (BLAKE3, genesis-folded) has been computed for the artifact's build/test evidence. |
| `RECEIPT_VERIFIED` | The receipt chain recomputes and verifies (linkage + hash recompute pass). |
| `OCEL_PROVEN` | Claim is backed by events in a validated OCEL v2 log. |
| `WASM4PM_PROVEN` | Claim is backed by wasm4pm process validation (conformance/replay). |
| `CLIENT_VISIBLE` | Artifact is exercised end-to-end from a real client surface (e.g. Playwright-driven UI). |
| `PUBLICATION_READY` | Artifact is ready for a publication artifact (paper, arXiv package) to reference it. Requires `scope`. |
| `PUBLISH_READY` | Artifact is packaged and dry-run verified for publishing (e.g. `cargo publish --dry-run`). Requires `scope`. |
| `PILOT_READY` | Artifact is ready to run in a scoped pilot deployment. Requires `scope`. |
| `PRODUCTION_READY` | Artifact is ready for production use within a stated scope. Requires `scope`. |
| `EXTERNAL_OPERATOR_SIDE_EFFECT` | Remaining action requires a human operator with external credentials (publish, submit, change visibility); packaged locally, executed operator-side. |
| `NON_STANDING` | Artifact intentionally has no standing tracked (e.g. scratch/throwaway). |
| `QUARANTINED` | Artifact is known-bad and excluded from normal gates pending repair. |
| `RETIRED` | Artifact is no longer maintained/shipped. |
| `DUPLICATE` | Artifact is a duplicate of another tracked artifact; see its `evidence`/notes for the canonical one. |

## Readiness ladder (0-9)

A single artifact can carry multiple statuses (its `standing` list). The
ladder collapses that list to one integer, the artifact's furthest
achieved rung, so dashboards and gates can threshold on a single number.

| Level | Status |
|---|---|
| 0 | `DISCOVERED` |
| 1 | `BUILDS` |
| 2 | `TESTED` |
| 3 | `RECEIPTED` |
| 4 | `OCEL_PROVEN` |
| 5 | `WASM4PM_PROVEN` |
| 6 | `REPLAYABLE` |
| 7 | `PUBLISH_READY` |
| 8 | `PILOT_READY` |
| 9 | `PRODUCTION_READY_FOR_SCOPE` |

`ladder_level` is computed, never asserted: it is the max ladder position
among the statuses present in `standing`. Statuses outside this ladder
(`UNSEEN`, `LINT_CLEAN`, `BENCHMARKED`, `RECEIPT_VERIFIED`,
`CLIENT_VISIBLE`, `PUBLICATION_READY`, `EXTERNAL_OPERATOR_SIDE_EFFECT`,
`NON_STANDING`, `QUARANTINED`, `RETIRED`, `DUPLICATE`) do not move the
ladder position on their own; they are recorded in `standing` but do not
raise `ladder_level` beyond what the ladder statuses in the same list
justify. `RECEIPT_VERIFIED` implies but is distinct from ladder rung 3
(`RECEIPTED`); `PRODUCTION_READY_FOR_SCOPE` (rung 9) is only reached when
`PRODUCTION_READY` is present *and* carries a non-empty `scope`.

## Artifact JSON shape

```json
{
  "id": "string, stable identifier",
  "kind": "rust_crate | client | doc | paper | bench | workflow | ontology | binary",
  "path": "string, repo-relative path",
  "standing": ["STATUS", "..."],
  "scope": "string, required iff PRODUCTION_READY/PILOT_READY/PUBLISH_READY/PUBLICATION_READY present, else omitted",
  "ladder_level": 0,
  "evidence": [
    {"kind": "command", "command": "string", "exit_code": 0, "utc": "RFC3339", "artifact": "path or null"},
    {"kind": "ocel_event", "event_id": "string", "path": "path to OCEL log"},
    {"kind": "receipt", "chain_hash": "blake3:...", "path": "path to receipt chain"},
    {"kind": "artifact", "path": "string", "hash": "blake3:..."}
  ],
  "external_operator_side_effects": ["string, e.g. 'crates.io publish requires operator credentials'"]
}
```

### Worked example — `crates/praxis-graphlaw`

```json
{
  "id": "praxis-graphlaw",
  "kind": "rust_crate",
  "path": "crates/praxis-graphlaw",
  "standing": [
    "BUILDS",
    "TESTED",
    "RECEIPT_VERIFIED",
    "OCEL_PROVEN",
    "WASM4PM_PROVEN",
    "PUBLISH_READY"
  ],
  "scope": "local release validation and crates.io dry-run",
  "ladder_level": 7,
  "evidence": [
    {
      "kind": "command",
      "command": "cargo test -p praxis-graphlaw",
      "exit_code": 0,
      "utc": "2026-07-06T19:00:00Z",
      "artifact": "docs/releases/v26.7.6/GRAPHLAW_FEATURES.md"
    },
    {
      "kind": "ocel_event",
      "event_id": "drv_e18",
      "path": "docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json"
    },
    {
      "kind": "receipt",
      "chain_hash": "blake3:9f8e1e18…5f1d91",
      "path": "docs/releases/v26.7.6/RECEIPT_VERIFY_OCEL.md"
    },
    {
      "kind": "command",
      "command": "cargo publish --dry-run --allow-dirty -p praxis-graphlaw",
      "exit_code": 0,
      "utc": "2026-07-06T19:44:59Z",
      "artifact": "docs/releases/v26.7.6/ocel/raw/cargo-publish-dry-run.txt"
    }
  ],
  "external_operator_side_effects": [
    "real crates.io publish requires operator credentials"
  ]
}
```

`ladder_level` is 7 (`PUBLISH_READY`) because `PRODUCTION_READY`/`PILOT_READY`
are absent; `RECEIPT_VERIFIED` and `WASM4PM_PROVEN` are both present but the
ladder position is governed by the highest ladder-listed status
(`PUBLISH_READY`, rung 7), not by the count of statuses held.

## Top-level document shape

```json
{
  "release_id": "string, e.g. v26.7.6",
  "generated_at_utc": "RFC3339",
  "generator": "string, tool/command that produced this document",
  "standing_version": "1",
  "artifacts": [/* StandingArtifact, ... */]
}
```

## Validation rule (scoped-readiness)

Any artifact whose `standing` list contains one or more of
`PRODUCTION_READY`, `PILOT_READY`, `PUBLISH_READY`, `PUBLICATION_READY`
**must** carry a non-empty `scope` string. An artifact with any of these
four statuses and a missing or empty `scope` is **invalid**: reject at
construction time (typed error, not a panic or silent default — see
praxis invariant 1) or flag it in the validation report if constructed
from external/untrusted data. This mirrors the existing praxis rule that a
readiness claim without a stated scope is not a claim at all — see the
`scope 'local release validation and crates.io dry-run'` field on the
worked example above, and the operator-side-effect framing already used
for `crates.io publish` in `CLAIM_PROMOTION_TABLE.md` row 10 and
`NO_TERMINAL_BLOCKERS.md`'s `RESOLVED_BY_EXTERNAL_OPERATOR_SIDE_EFFECT` row.
