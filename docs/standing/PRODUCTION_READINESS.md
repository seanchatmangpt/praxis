# Production Readiness Ladder

The 0-9 readiness ladder collapses an artifact's `standing` list (which can
hold any combination of the 20 statuses) to one integer: the highest rung
justified by a ladder status currently present. `ladder_level` is always
computed (`cargo_cicd_core::standing::model::compute_ladder_level`), never
asserted — see `docs/standing/STANDING_SCHEMA.md` for the full schema.

## The ladder

| Level | Status | What it takes |
|---|---|---|
| 0 | `DISCOVERED` | Artifact is indexed (path, kind known); nothing run against it yet. |
| 1 | `BUILDS` | Compiles/builds cleanly. |
| 2 | `TESTED` | Test suite passes. |
| 3 | `RECEIPTED` | A BLAKE3 genesis-folded receipt has been computed for the build/test evidence. |
| 4 | `OCEL_PROVEN` | Backed by events in a validated OCEL v2 log. |
| 5 | `WASM4PM_PROVEN` | Backed by wasm4pm process validation (conformance/replay). |
| 6 | `REPLAYABLE` | Verified receipts + OCEL/wasm4pm proof combine to a demonstrated clean replay (no dedicated status; an upstream policy computes this, not a single field). |
| 7 | `PUBLISH_READY` | Packaged and dry-run verified for publishing. **Requires scope.** |
| 8 | `PILOT_READY` | Ready for a scoped pilot deployment. **Requires scope.** |
| 9 | `PRODUCTION_READY_FOR_SCOPE` | `PRODUCTION_READY` present *and* carries a non-empty `scope`. |

### The scoped-readiness rule

Rungs 7, 8, and 9 (and the off-ladder `PUBLICATION_READY`) are gated: none of
`PUBLISH_READY` / `PILOT_READY` / `PRODUCTION_READY` / `PUBLICATION_READY`
count unless the artifact also carries a non-empty `scope` string.
`PRODUCTION_READY` with an empty or missing scope computes to rung 8, not 9
(`ladder_rung` in `model.rs` falls back explicitly) — an unscoped
"production-ready" claim is capped one rung below where it would otherwise
land, it never silently reaches 9.

## Worked walk-through: three real praxis artifacts

### 1. `crates/praxis-graphlaw` (rust_crate) — rung 7, `PUBLISH_READY`

Standing: `BUILDS, TESTED, RECEIPT_VERIFIED, OCEL_PROVEN, WASM4PM_PROVEN,
PUBLISH_READY`. Scope: `"local release validation and crates.io dry-run"`.

- Builds and passes tests in every `just verify-all` run.
- Receipt-verified: `docs/releases/v26.7.6/RECEIPT_VERIFY_OCEL.md`.
- OCEL/wasm4pm proven: `docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json`,
  `docs/releases/v26.7.6/WASM4PM_PROCESS_VALIDATION.md`.
- Dry-run packaged: `cargo publish --dry-run --allow-dirty -p praxis-graphlaw`
  → exit 0 (`docs/releases/v26.7.6/ocel/raw/cargo-publish-dry-run.txt`).
- **Why not rung 9**: the real `cargo publish -p praxis-graphlaw` has not been
  run — that is an `EXTERNAL_OPERATOR_SIDE_EFFECT` (operator credentials),
  not a `PRODUCTION_READY` claim. See `EXTERNAL_OPERATOR_SIDE_EFFECTS.md`.

### 2. `clients/autonomic-platform` (client) — rung 2, `TESTED`, moving toward `OCEL_PROVEN`

Standing (current, per `CLIENT_SURFACES.md`/`OCEL_PLAYWRIGHT_WASM4PM_COMPLETION.md`):
`BUILDS, TESTED`. No scope set (none of the gated statuses are present, so
none is required).

- Builds: `cd clients/autonomic-platform && npm run build` (`vite build`) —
  passes.
- Is the Playwright target for the release's OCEL evidence pass
  (`clients/autonomic-platform/tests/playwright/ocel-wasm4pm-validation.spec.ts`),
  which is how it would earn `OCEL_PROVEN` (rung 4) once that evidence is
  attached to *this artifact's* standing entry specifically, not just cited
  in the release-level document.
- **Why rung 2, not higher**: standing is per-artifact. The release-level
  OCEL log exists and is real, but until the standing compiler's client
  ingestor (`ingest_client_builds` in `cargo-cicd-core`) attaches an
  `ocel_event` evidence entry to the `autonomic-platform` artifact
  specifically and its `standing` list gains `OCEL_PROVEN`, the client's own
  ladder position stays at `TESTED`. Claiming otherwise for this artifact
  would trip `ANTI-LLM-STANDING-002`.

### 3. The v26.7.6 release as a whole — release-level rung 7 (`PUBLISH_READY`/`PUBLICATION_READY` lanes), external operator action pending for both

Per `docs/releases/v26.7.6/FINAL_STATUS.md`: all seven exit criteria are met
with evidence, `just verify-all` is green, the OCEL v2 log passes wasm4pm
integrity validation (0 errors) and POWL conformance (fitness 1.0). Two
publication lanes are dry-run-verified:

- **crates.io lane**: `PUBLISH_READY`, scope `"crates.io dry-run for
  praxis-graphlaw"`. Real publish needs `cargo login` + `cargo publish -p
  praxis-graphlaw` — operator-side, typed as
  `EXTERNAL_OPERATOR_SIDE_EFFECT`, not a blocker on the release's own
  standing.
- **arXiv lane**: `PUBLICATION_READY`, scope `"arXiv cs.SE primary / cs.LO
  cross-list submission"`. Real submission needs the artifact bundle made
  public and the upload at arxiv.org — same treatment.
- **Why not rung 9 / unscoped "production-ready"**: a release is not a
  single `rust_crate` with a single `PRODUCTION_READY` scope; it is scored
  per-artifact. Calling the whole release "production-ready" without naming
  which artifact, in which scope, trips `ANTI-LLM-STANDING-001` (unscoped
  claim) even though the underlying evidence is strong. The correct claim
  is the one `FINAL_STATUS.md` actually makes: "ALIVE" (backed by
  `RECEIPT_VERIFIED`/`OCEL_PROVEN`, satisfying `ANTI-LLM-STANDING-004`) with
  two named lanes at `PUBLISH_READY`/`PUBLICATION_READY` and external
  operator action as the only remainder.

## Reading the ladder correctly

- A high `ladder_level` on one artifact says nothing about a different
  artifact's standing — the client example above shows a release-level OCEL
  pass that has not yet propagated to a specific client's own entry.
- Rung 6 (`REPLAYABLE`) has no dedicated status; do not expect to find it as
  a literal string in `standing`. It is inferred by policy once verified
  receipts and OCEL/wasm4pm proof are both present.
- Never round an unscoped `PRODUCTION_READY`/`PILOT_READY`/`PUBLISH_READY`/
  `PUBLICATION_READY` up to its full rung — the scoring code itself refuses
  to (rung 8 fallback for unscoped `PRODUCTION_READY`), and
  `ANTI-LLM-STANDING-001` refuses it in prose.
