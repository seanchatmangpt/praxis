# Changelog

All notable changes to **praxis** are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions use the
constellation's CalVer scheme (`YY.M.PATCH`).

This file is generated from the repository's conventional-commit history. Where
a line describes work landed in the working tree during the Genesis week but not
yet in a commit at the time of writing, it is marked `(working tree)` — a
changelog written honestly inherits the same discipline as the receipts.

## [26.7.2] — 2026-07-02 — Genesis release

The seven-day Genesis program's release. Version bumped from `26.6.30`;
monotonic and distinct from `wasm4pm 26.7.1`. Tag `v26.7.2` is applied at chain
close, over a committed, quiescent, build-green tree (see the Day 7 receipt).

### Added
- **Capability frontier matrix** (`frontier matrix` verb, `src/frontier.rs`,
  `tests/frontier_matrix.rs`): a DfCM matrix built on `wasm4pm_compat::dfcm`
  over every external capability source explored this session. Each source is
  one cell — Admitted (with the socket it landed in) or Impossible (refused,
  with reason + salvage). `coverage == 1.0`, `pass_rate == 1.0`, zero silent
  rows. Serialized to `target/frontier-report.json`.
- **Proposer + `propose` verbs** (`crates/praxis-proposer`, PR-14): revenue
  observation → ranked proposals → goal, sitting *outside* the admission
  boundary (proposals are untrusted `O`, not `O*`). MRR (Maximum Reachable
  Revenue) algebra. `(working tree)`
- **RevTAC v0** mission vocabulary: revenue operators author missions in an
  ontology, never raw PDDL. `(working tree)`
- **`verify` verb** (Lane 7): affidavit-style receipt certification pipeline
  (`decode → check_format → chain_integrity → continuity → verify_commitments
  → evaluate_profile`), timing-instrumented, over `praxis_core::verify`.
- **Andon second gate** (Lane 5): `refusal.rs` taxonomy, `Andon::Halted`
  breaking change, `AndonRing`, and a `prolog8 Kernel::query` proof-carrying
  admission path in `ops.rs`; honest `denial` word on `ReceiptMeta`.
- **MCP+ law-object server** (Lane 8b): de-stubbed tool surface, `ToolResultCache`
  ported into `src/mcp_cache.rs`, `.mcp.json`.
- **ReceiptRecord → SHACL** bridge (`src/receipt_shacl.rs`): validates against
  the real `open-ontologies` `sr:SharedReceiptV1` shapes. `(working tree)`
- **Revenue pipe** orchestration (`src/revenue.rs`) + `revenue_demo` binary.
  `(working tree)`
- **`dod` binary**: Definition-of-Done gate (fmt + clippy + test hard gate;
  receipts/evidence soft checks).

### Changed
- Repointed to `wasm4pm 26.7.1`; `praxis-proposer` promoted into the workspace.
- Vision 2030 PRD reconciled against the real verb surface; release-criteria
  table verified PASS/PARTIAL/FAIL against exercised reality, not inference.

### Fixed
- `fix(tests)`: gate unsigned-receipt snapshots out of `law-signed` builds.
- `fix(mcp)`: lawobject-server tests green under `--all-features`.

### Docs
- Genesis program (`docs/GENESIS.md`), PDDL capability model, CPhy roadmap,
  concepts catalog, OCEL 2.0 research report, walkthrough fixed against the
  real surface.

## [26.6.30] — 2026-06-29

### Added
- `star-toml` `TrustedLoader`, `cicd.toml`, signed-receipts feature
  (ed25519 + `KeyPair`), `praxis-retrofit` fleet-wide standardization platform,
  `template-{wasm,integration,mcp}` variants, phase-typed AAA testkit.
- `GitLock` + `GitAuditLedger` (gitvan patterns), three architectural traits.

### Fixed
- lsp-max pinned to `26.6.24`; doc comments for chain/error/lib modules;
  agent-generated module compilation errors; praxis-retrofit build issues.

### Docs
- Onboarding & API docs, kit-level `CONTRIBUTING.md`, implementation report,
  wasm4pm retrofit case study.

[26.7.2]: https://github.com/seanchatmangpt/praxis/releases/tag/v26.7.2
[26.6.30]: https://github.com/seanchatmangpt/praxis/releases/tag/v26.6.30
