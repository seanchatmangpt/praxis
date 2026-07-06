# Fortune-5 Readiness — Praxis v26.7.6 "After Neon"

Readiness categories used throughout: RESEARCH_READY / DEMO_READY /
PILOT_READY / ENTERPRISE_HARDENING_REQUIRED / PRODUCTION_READY. Assignments
below are made from test evidence in this repository, not from intent.

**Overall assignment: DEMO_READY, with the receipt/verification core
RESEARCH_READY-plus and everything enterprise-facing
ENTERPRISE_HARDENING_REQUIRED.** Nothing in this release is PRODUCTION_READY.

## 1. Executive summary

Praxis addresses one enterprise problem: AI-generated technical work (code,
plans, formal statements) arrives without standing — nobody can say, with
evidence, what was checked, by what authority, and whether the result can be
replayed. Praxis manufactures that standing: artifacts are admitted by a
law-state graph, planned, executed as workflow, gauged by the Lean 4 kernel
and by tests, and evidenced by computed BLAKE3 receipts. The demonstrated
result is a 219-statement mathematics corpus with 178/202 labels
kernel-verified against Mathlib, each with a per-statement receipt
(`tools/paper-factory/lean-lake/mathlib_migration_receipts.jsonl`).

## 2. The enterprise problem

- AI output volume exceeds review capacity; approval degrades to assertion.
- Audit teams cannot replay how an AI artifact was produced or checked.
- Provenance is bolted on (logs, tickets) rather than computed (hashes).

Praxis's counter-position: standing is a manufactured, receipted property.
Unknown inputs are refused by name (closed vocabularies — RELEASE_CONTROL.md
Sec. 4), errors are typed `Refusal` variants rather than panics or silent
defaults, and every receipt is computed (BLAKE3, genesis-folded, `ts_ns=0`).

## 3. Deployment model: local-first

- Entire system builds and runs on a single workstation:
  `cargo check --workspace` exit 0 on 2026-07-06 (RELEASE_CONTROL.md Sec. 8).
- No network services required at runtime; the synthesis pipeline is
  guaranteed LLM-free at runtime by test
  (`crates/praxis-synthesis/tests/no_llm_runtime.rs` — deps frozen to
  pddl-index, chatman-common, blake3, serde, serde_json, thiserror).
- Data never leaves the machine unless the operator publishes it. This is
  the primary security property and the primary pilot advantage: no
  data-processing agreement needed to evaluate.

Category: DEMO_READY (builds locally, e2e tests exist:
`tests/plan_run_e2e.rs`, `crates/ggen/tests/*_e2e.rs`). Not PILOT_READY: no
install/packaging story beyond `cargo` from a checked-out workspace, and the
one-command full-loop demo is not yet landed (RELEASE_CONTROL.md exit
criterion 3: NOT STARTED).

## 4. Security posture

| Property | Evidence | Category |
|---|---|---|
| No `unsafe` in factory | `crates/ggen/src/lib.rs` `#![deny(unsafe_code)]`; also `crates/praxis-core` | RESEARCH_READY |
| No stdout side channels in factory | `#![deny(clippy::print_stdout)]` in `crates/ggen/src/lib.rs` | RESEARCH_READY |
| No runtime LLM / no exfiltration path in synthesis | `crates/praxis-synthesis/tests/no_llm_runtime.rs` | RESEARCH_READY |
| Typed refusals, no panics | invariant 1; refusal tests per command NOT complete (RELEASE_CONTROL.md exit criterion 5: NOT STARTED) | ENTERPRISE_HARDENING_REQUIRED |
| Dependency audit / cargo-audit in CI | no evidence found | ENTERPRISE_HARDENING_REQUIRED |
| Secrets handling, signing keys | not applicable yet (no signing); receipts are hashes, not signatures | ENTERPRISE_HARDENING_REQUIRED |

## 5. Data boundaries

- Inputs: RDF/Turtle graphs (e.g. `examples/v26_7_6_after_neon/goal.ttl`),
  templates, Lean sources. All file-based, operator-supplied.
- Outputs: generated artifacts + receipts under `.ggen/receipts/`.
- Closed vocabularies (`wf:`, `hook:`, `prayer-kernel:`, `agent:`) mean
  foreign predicates cannot silently enter the law-state; they are refused
  by name (paired with `docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md`).
- No telemetry: `crates/ggen/src/telemetry.rs` was deleted in the current
  working tree (git status), consistent with the local-first posture.

## 6. Evidence, receipt, and replay models

- **Evidence**: every gate emission is a receipt record; the Lean lane
  produced one receipt per statement (202 records,
  `tools/paper-factory/lean-lake/mathlib_migration_receipts.jsonl`).
- **Receipts**: BLAKE3, genesis-folded, no wall clock in hash paths
  (`ts_ns=0`). Chain verification test: `crates/ggen/tests/receipt_chain_e2e.rs`.
  Published chain: `.ggen/receipts/latest.json` plus dated `sync-*.json`.
- **Replay**: `ggen sync` re-renders artifacts from graph source-of-truth
  (`crates/ggen/tests/sync_e2e.rs`, `multi_template_determinism.rs`); Lean
  verdicts replay via `lake env lean`. Full-loop byte-identical two-run
  replay is an exit criterion, not yet demonstrated (RELEASE_CONTROL.md
  criterion 3). Category: RESEARCH_READY per-lane, DEMO_READY end-to-end
  only after criterion 3 closes.

## 7. Integration surfaces

- CLI verbs on `ggen` (sync/lint/validate/watch — `crates/ggen/src/verbs/handlers.rs`)
  and the planner verb (`src/verbs/plan.rs`, commit `8336f29`).
- Graph-law queries via `crates/praxis-graphlaw` (SPARQL/N3/Datalog/SHACL/ShEx).
- Client surfaces are inventoried but adapter-gapped:
  `docs/releases/v26.7.6/CLIENT_SURFACES.md` (commit `7138359` records the
  adapter gap as typed). Category: ENTERPRISE_HARDENING_REQUIRED — no REST,
  no SSO, no multi-user story.
- First adapter-wired client landed: `clients/autonomic-platform/` (vite build
  PASS 2026-07-06; DECK/OPS/HUD read receipts/plan/registry with provenance,
  GLOBE/ARENA remain mock-labeled NON-STANDING).

## 8. CI/CD

- Local gates exist: `just verify-all` (DoD) and `just test-changed`
  (justfile). No hosted CI evidence found in the repo for this release;
  criterion 1 (`just verify-all` green, output captured) is UNKNOWN in
  RELEASE_CONTROL.md. Category: ENTERPRISE_HARDENING_REQUIRED.

## 9. Compliance / SOC 2 considerations

Not audited; no controls documentation exists. Honest position: the receipt
architecture is *evidence-friendly* (immutable computed chains map naturally
onto SOC 2 change-management and processing-integrity evidence), but no
control mapping, access control, or retention policy exists. Category:
ENTERPRISE_HARDENING_REQUIRED.

## 10. SBOM path

`cargo metadata`/`cargo tree` can produce a complete dependency graph today;
CycloneDX generation via `cargo-cyclonedx` is the obvious path. The frozen
dependency set of `praxis-synthesis` (6 crates, test-enforced) makes that
crate's SBOM trivially auditable. No SBOM artifact is checked in yet.

## 11. Threat model (summary)

| Threat | Mitigation today | Gap |
|---|---|---|
| Forged receipt | receipts computed, chain-verified (`receipt_chain_e2e.rs`) | hashes are not signatures; no key-backed attestation |
| Poisoned graph input | closed vocabularies, refusal by name; SHACL/ShEx gates (`graphlaw_e2e.rs`) | shape coverage not measured |
| Nondeterminism smuggling wall-clock into hashes | `ts_ns=0` invariant | needs a lint/test that greps hash paths for clock calls |
| Malicious template in factory | `deny(unsafe_code)`, no stdout | templates can still write arbitrary paths — needs a write-boundary test |
| Supply chain | frozen deps in synthesis crate | rest of workspace unaudited |

## 12. Failure modes

By construction, failures are refusals: typed `Refusal` variants, never
panics or silent defaults (invariant 1). Known deviations to fix before any
pilot: refusal completeness per command is NOT STARTED (RELEASE_CONTROL.md
criterion 5); `praxis-reconciler` is orphaned/untested (INVENTORY.md);
`lsp-max` hardcoded-path patch (root `Cargo.toml:153`) ties the build to a
MISSING external lineage.

## 13. Buyer FAQ

- **Does it call an LLM in production?** The synthesis pipeline provably not
  (`no_llm_runtime.rs`). LLMs are used at authoring time (e.g., the Lean
  reformalization waves), gated by the kernel.
- **Can we verify a receipt independently?** Yes — BLAKE3 over published
  inputs; the chain test shows the procedure.
- **What happens on unknown input?** Named refusal, not best-effort.
- **Is it multi-tenant?** No. Single-operator, local-first.
- **What's actually been verified end to end?** A 202-label math corpus
  (178 verified vs Mathlib) and a planner→factory→receipt loop with an e2e
  test (`tests/plan_run_e2e.rs`). Nothing customer-shaped yet.

## 14. Production-ready now vs. not yet

**Ready now (RESEARCH_READY/DEMO_READY):** receipt computation and chain
verification; deterministic ggen factory; graph-law admission with typed
refusal; Lean admission gate; planner vertical loop with e2e test;
local-first build.

**Not yet (ENTERPRISE_HARDENING_REQUIRED):** packaging/install, hosted CI,
refusal-completeness across the command surface, signed receipts, SBOM
artifact, access control, compliance mapping, multi-user surfaces,
one-command deterministic full-loop demo.

## 15. 30-day pilot plan

Weeks 1–2: close RELEASE_CONTROL.md exit criteria 1, 3, 5, 7 (verify-all
green with captured output; deterministic two-run demo; refusal tests per
command; receipt-chain verification output). Weeks 2–3: package a
single-binary or single-checkout install; produce SBOM; run the full loop on
one pilot-partner artifact class (e.g., their internal spec corpus → graph →
generated doc + receipts). Week 4: pilot exit report — receipts delivered,
replay demonstrated on the partner's machine, gap list.

Entry gate for the pilot: all four criteria above green. Until then the
correct external posture is DEMO_READY.

## 16. 90-day path

Days 1–30: pilot above. Days 31–60: signed receipts (key-backed attestation
over the BLAKE3 chain), hosted CI with `just verify-all` as the gate,
write-boundary test for templates, cargo-audit + SBOM in CI. Days 61–90:
second pilot domain (code-generation standing, not just math), SOC 2 control
mapping draft, decision point on PILOT_READY→ENTERPRISE_HARDENING exit. F5
conversations before day 90 are premature by this plan's own evidence bar.
