# Milestone Overview: v26.8.16 — Chatman/ggen Ecosystem Unification (Session Backlog)

Continuation of the chatman-engine unification review and ggen-ecosystem reconstitution review
run this session (2026-08-13 → 2026-08-16). Scope: close the workspace-build blocker that
currently fails every `just` gate, land the additive fixes both reviews classified
autonomous-safe, and record the items that need explicit human sign-off rather than silently
deferring them.

Source material: chatman cross-analysis + unification plan (`crates/chatman-common`,
`crates/praxis-graphlaw/src/chatman`, `crates/multifractal-workflow`, `crates/praxis-synthesis`,
`crates/cng`, `crates/powl2-decompose`); ggen cross-analysis + reconstitution plan
(`crates/ggen`, `packs/*`, `docs/GGEN_PARITY.md`, `docs/ggen-port-evaluation.md`); the
autonomous-iteration-loop design produced in this session.

## Ticket index

| # | Name | Scope | Dependencies | Status |
|---|------|-------|--------------|--------|
| 811 | [Fix broken `crates/cng` workspace dependency chain](PROJ-811.md) | `cng` → `multifractal-workflow` → `my-conforming-project` → `rust-fable-testbed` → `ggen-core` path dependency resolves to `/Users/sac/ggen/crates/ggen-core/Cargo.toml`, absent on this machine; breaks `cargo metadata` for the whole workspace, so `just fmt-check`/`check`/`clippy`/`test`/`verify-all` all fail before running | — | DONE — vendored ggen-core's `prompt_mfg` module into `rust-fable-testbed` (pinned to ggen commit `68d3c2560`, the last commit before `ggen-core` was retired upstream); dead path dependency dropped |
| 812 | [Wire orphaned-but-consumed ggen packs into `ggen.toml`](PROJ-812.md) | Additive 14-entry `[packs]` edit (11 f-numbered packs + azure-terraform/dry-run-publish/soc2-audit) verified to have real templates, real ontology, and real committed consumer output; edit is drafted and on disk, uncommitted, blocked on PROJ-811's gate | PROJ-811 | DONE |
| 813 | [Create an "Excluded packs" section in `docs/GGEN_PARITY.md`](PROJ-813.md) | No such section currently exists anywhere in the docs; 5 packs (`dogfood-lifecycle-pack`, `lean-math-pack`, `ma-case-study-pack`, `post-release-pack`, `quadrature-pack`) need a documented one-sentence exclusion reason each, reconciling a stale "4" count found in the prior survey's own summary line | PROJ-811 | DONE |
| 814 | [Extract shared receipt-root fold helper into `chatman-common`](PROJ-814.md) | `chatman::engine.rs:242,263` (`EngineProcessReceipt`, 9-digest fold) is independently reimplemented at `cng::otel_receipt.rs:34-36,234`; extract a shared `chatman_common::provenance::fold_digest_root` helper both wrap | PROJ-811 | OPEN — see audit note (byte-layout discrepancy blocks extraction) |
| 815 | [Document the POWL 3-way fork and cross-reference the two quarantine doctrines](PROJ-815.md) | Docs-only: name `powl2_decompose::Powl` as canonical-for-chatman vs. `cng::powl::Powl`'s disclosed clean-room duplicate vs. the unreconciled `wasm4pm-compat`/Lean-4 `~/mfact` variants; add cross-reference comments between `praxis-synthesis::quarantine.rs` and `chatman::quarantine.rs` | — | DONE |
| 816 | [Bootstrap the autonomous iteration loop](PROJ-816.md) | Seed `TaskList` with this milestone's tickets plus dependency edges per the session's loop-design decision (`/loop` self-paced wrapping a `Workflow` script per tick, tiered `fmt-check`/`test-changed`/`verify-all` gates, `TaskList` as live pointer + per-ticket `docs/jira` file as durable record) | PROJ-811 | BLOCKED (TaskList/TaskCreate tooling unavailable this session) |

---

## Notes

**PROJ-811 is the hard blocker for everything else in this milestone.** Every `just` gate
(`fmt-check`, `check`, `clippy`, `test`, `verify-all`, `test-changed`) fails at the
`cargo metadata` step before running any check-specific logic, because
`crates/cng/Cargo.toml`'s path-dependency chain resolves to `/Users/sac/ggen/crates/ggen-core/Cargo.toml`,
which does not exist on this machine. This is a machine-state/repo-topology question (is
`~/ggen` a missing sibling checkout that should exist, or is the path reference stale and should
point somewhere in-repo instead) — reserved for human sign-off per
`.claude/rules/autonomous-escalation-policy.md`'s "genuinely underdetermined product law" class,
not decided here.

**PROJ-812 and PROJ-813** are the additive, autonomous-safe items from the ggen reconstitution
review, both drafted/scoped and waiting only on PROJ-811 to actually run their gate.

**PROJ-814 and PROJ-815** are the lowest-risk items from the chatman reconstitution review
(receipt-fold extraction is a mechanical, reversible refactor with no crate-graph risk per that
review's own risk ranking; the documentation items are zero-code-risk).

**Deliberately excluded from this milestone** (per both reviews' own risk ranking, not
overlooked): converging `cng::powl::Powl` onto `powl2_decompose::Powl` (blocked on an unverified
`oxigraph` transitive-dependency question), deprecating any of the three parallel `Refusal`
taxonomies (API-breaking, needs a product decision), merging the four Receipt shapes (may be
correctly separate, not consolidatable), the `ggen` binary reinstall item (real machine-state
mutation, escalation-reserved), and `shape:`/SHACL wiring (L-sized, no current template demand).

## See Also

- `docs/CHATMAN_EQUATION.md` — canonical reference for `praxis-graphlaw::chatman::engine.rs`
- `docs/GGEN_PARITY.md`, `docs/ggen-port-evaluation.md`, `docs/ggen-theory.md` — ggen's own
  design/parity docs this milestone's tickets reconcile against
- `.claude/rules/autonomous-escalation-policy.md` — governs which items above are
  decide-and-proceed vs. reserved for sign-off
- `.claude/rules/rust-agi-core-team.md` invariant #9 ("API Stability Is a Promise") — the reason
  the Refusal-taxonomy unification is excluded from this milestone
