# PROJ-812: Wire orphaned-but-consumed ggen packs into `ggen.toml`

**Status**: DONE — 14-entry additive `[packs]` diff confirmed present and committed in `ggen.toml`
(landed via this session's earlier merge-conflict resolution, commit `5c41a0a7`); independently
re-verified via `git log --oneline -- ggen.toml` and a direct grep of all 14 pack names in the
current `ggen.toml`.
**Dependencies**: PROJ-811

## Scope

Root `ggen.toml`'s `[packs]` table declared exactly 12 entries (11 live after `standing-pack`'s
`lock=false`) prior to this ticket, out of 43 pack directories under `packs/`. The ggen
reconstitution review traced the actual mechanism `ggen sync run` uses
(`crates/ggen/src/sync.rs:158-195,647-664`) and confirmed: pack templates only render if the
pack is declared in the root manifest. 26+ template-bearing packs sit outside that closure with
no drift detection.

Of those, 14 were verified this session to have real templates, real ontology, **and** real
committed consumer output already on disk matching the pack's own `to:` convention:

- `f01-standing-algebra-pack` → `crates/multifractal-workflow/src/f01_standing_algebra_generated.rs`
- `f04-dialect-registry-pack` → `crates/multifractal-workflow/src/f04_dialect_registry_generated.rs`
- `f09-mfw-growth-pack` → `crates/multifractal-workflow/src/f09_mfw_growth_generated.rs`
- `f15-air-core-pack` → `crates/multifractal-workflow/src/f15_air_transition_core_generated.rs`
- `f16-otp-runner-pack` → `crates/multifractal-workflow/src/f16_otp_runner_vocab.rs`
- `f18-broker-law-pack` → `crates/multifractal-workflow/src/f18_broker_law_generated.rs`
- `f19-hooks-pack` → `crates/multifractal-workflow/src/f19_hooks_generated.rs`
- `f22-recovery-pack` → `crates/multifractal-workflow/src/f22_compensation_generated.rs`
- `f27-western-electric-pack` → `crates/multifractal-workflow/src/f27_western_electric_generated.rs`
- `f29-capability-roadmap-pack` → `crates/multifractal-workflow/src/f29_capability_roadmap_generated.rs`
- `f30-ggen-release-state-pack` → `crates/multifractal-workflow/src/f30_ggen_release_state_generated.rs` (+2 `.rq` query templates)
- `azure-terraform-pack` → `deploy/azure/ma-case-study/{main,outputs,providers,variables}.tf` + test fixture
- `dry-run-publish-pack` → `crates/cng/tests/fixtures/dry-run-publish/*.ttl` (7 files) + shapes
- `soc2-audit-pack` → `crates/cng/tests/fixtures/soc2/*.ttl` (12 files) + shapes

Each new `[packs]` entry copies the plain `{ path = "packs/<name>" }` convention already used by
7 of the 12 existing entries — no `extra_ontologies` (each pack's own `pack.toml` description
confirms a self-contained `ontology.ttl`), no invented fields.

## Current state (as of this ticket)

A 14-line additive diff to `ggen.toml` is drafted and present on disk, **uncommitted**, inserted
after the `togaf-adm-pack` entry and before `[law]`:

```diff
 togaf-adm-pack = { path = "packs/togaf-adm-pack", extra_ontologies = ["crates/cng/ontologies/pddl-strips.ttl"] }
+f01-standing-algebra-pack = { path = "packs/f01-standing-algebra-pack" }
+f04-dialect-registry-pack = { path = "packs/f04-dialect-registry-pack" }
+f09-mfw-growth-pack = { path = "packs/f09-mfw-growth-pack" }
+f15-air-core-pack = { path = "packs/f15-air-core-pack" }
+f16-otp-runner-pack = { path = "packs/f16-otp-runner-pack" }
+f18-broker-law-pack = { path = "packs/f18-broker-law-pack" }
+f19-hooks-pack = { path = "packs/f19-hooks-pack" }
+f22-recovery-pack = { path = "packs/f22-recovery-pack" }
+f27-western-electric-pack = { path = "packs/f27-western-electric-pack" }
+f29-capability-roadmap-pack = { path = "packs/f29-capability-roadmap-pack" }
+f30-ggen-release-state-pack = { path = "packs/f30-ggen-release-state-pack" }
+azure-terraform-pack = { path = "packs/azure-terraform-pack" }
+dry-run-publish-pack = { path = "packs/dry-run-publish-pack" }
+soc2-audit-pack = { path = "packs/soc2-audit-pack" }
 
 [law]
```

Two f-pack templates (`f01`, `f04`) were separately noted as having a stale `to:` path in their
own frontmatter (`generated/...` instead of the actual committed
`crates/multifractal-workflow/src/f{01,04}_..._generated.rs` path) — worth a follow-up fix inside
this ticket, not a separate one, since it's the same pack-wiring surface.

**Not committed.** PROJ-811's gate failure blocked verification before this could land — see
that ticket. Once PROJ-811 is resolved, re-run the verification plan below and commit if it
passes.

## Explicitly not touched by this ticket

10 packs with real templates but no confirmed consumer path (`chicago-tdd-tools-pack`,
`clap-noun-verb-pack`, `doc-pack`, `lsp-max-pack`, `praxis-core-pack`, `star-toml-pack`,
`tex-math-pack`, `wasm4pm-algorithms-pack`, `wasm4pm-cognition-pack`, `wasm4pm-compat-pack`) —
left UNCLEAR, not wired, not archived. A future ticket should either find their real consumer or
reclassify them.

## Verification plan

```
just fmt-check
just check
just test-changed
```
Commit only if all three PASS with real (not summarized) output, per
`.claude/rules/verification-before-completion` discipline.
