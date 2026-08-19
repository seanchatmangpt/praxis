# PROJ-813: Create an "Excluded packs" section in `docs/GGEN_PARITY.md`

**Status**: DONE — independent audit confirmed the `## Excluded packs` section diff matches
ticket scope exactly (5 rows, correct insertion point), and independently re-verified all cited
facts (pack directory contents, template lists, `to:` targets, `quadrature-pack` re-check,
directory absence) against real files.
**Dependencies**: PROJ-811

## Scope

`docs/GGEN_PARITY.md` (148 lines: Measured template usage, Where crates/ggen exceeds the
reference, Deferred by design, Blockers resolved this session, Disclosed limitations, See also)
has **no section documenting intentionally-excluded packs** — confirmed by a full read this
session, not inferred.

Five packs are confirmed intentionally outside the ggen-template pipeline, each for a distinct
reason:

- **`dogfood-lifecycle-pack`** — no `templates/` directory at all; `pack.toml` states by design
  it's ontology+shapes+fixtures for a session hook, not a ggen-template pack.
- **`ma-case-study-pack`** — no `templates/` directory; `pack.toml` states it's TBox-only
  ontology+shapes+fixtures, consumed directly by
  `crates/praxis-graphlaw/tests/ma_case_hook_actuation.rs` and as `azure-terraform-pack`'s
  `ma-case-study` row instance data — consumed as raw fixtures, not via template generation.
- **`lean-math-pack`** — has real templates, but they target `procint/**.lean`; no `procint/`
  directory exists anywhere under this repo (per project memory, this is the separate `~/mfact`
  Lean package, out-of-repo).
- **`post-release-pack`** — has real templates (14), but they target `paper/`, `release/`,
  `research/wfnet/`, `procint/...` — none of which exist under this repo.
- **`quadrature-pack`** — not independently re-verified this session; carried from the prior
  survey's exclusion list. Re-confirm its exclusion reason before writing the doc entry (the
  prior survey's own summary line said "4 DOCUMENT-EXCLUDE" while its table listed 5 rows
  including this one — the mismatch was flagged, not resolved. Re-check `quadrature-pack`'s
  `pack.toml`/templates directly before writing its entry).

## Why this needs a decision before it needs an edit

The instruction to "match the existing section's format" (following the pattern already used
for `dogfood-lifecycle-pack`/`self-monitoring-pack`/`ma-case-study-pack` elsewhere) assumed such
a section already existed in `GGEN_PARITY.md`. It doesn't — the closest related content is
`docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md:54` (a SHACL-authorship limitations
discussion that happens to name 4 of these packs in a different context) and two release PRD/ARD
docs that discuss `dogfood-lifecycle-pack`/`self-monitoring-pack` as milestone claims, not as a
ggen-wiring exclusion list. Inventing a new section format under agent judgment was explicitly
avoided this session per the "additive only, don't invent new fields/formats" discipline applied
to PROJ-812 — the same discipline extends here. This ticket exists to make that a deliberate
choice (where does the section go, what format) rather than a silent gap.

## Proposed format (for review, not yet applied)

A new `## Excluded packs` section in `GGEN_PARITY.md`, one row per pack:

| Pack | Reason |
|---|---|
| dogfood-lifecycle-pack | no `templates/` dir — ontology+shapes+fixtures for a session hook |
| ma-case-study-pack | no `templates/` dir — TBox fixtures consumed directly by tests, not via template rendering |
| lean-math-pack | templates target `procint/` — out-of-repo (`~/mfact` Lean package) |
| post-release-pack | templates target `paper/`/`release/`/`research/wfnet/`/`procint/` — none exist in this repo |
| quadrature-pack | UNVERIFIED this session — re-check before filling in |

## Verification plan

Docs-only change: `just fmt-check` is unaffected (markdown isn't a fmt target), but should still
be run as the cheapest available check since it's blocked by PROJ-811 regardless. No `cargo
check`/`test-changed` dependency for this ticket specifically — safe to land as soon as the
format/location decision above is confirmed, independent of PROJ-811's Rust-workspace blocker.
