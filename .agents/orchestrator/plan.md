# Plan: Praxis Boilerplate Generator Upgrade

## Objective
Upgrade the `praxis` boilerplate generator by integrating architectural insights from the Chatman ecosystem (`rocket-craft` and `lsp-max`), specifically Generative Typestates, `RulePackServer`, and the `ggen` µ-pipeline.

## Orchestration Strategy
We are utilizing the Project Pattern under the POWL v2 operating doctrine. Work is decomposed into sequential milestones. For each milestone, we spawn dedicated specialist subagents.

## Milestones
- **M1: Ecosystem Cataloging & Abstraction Analysis**
  - Scope: Investigate `rocket-craft` and `lsp-max` to catalog architectural patterns, ZST typestates, `RulePackServer` structures, and the `ggen` µ-pipeline. Deliver a Markdown catalog report in `survey/`.
  - Verification: Reviewer check of the catalog.
  - Status: IN_PROGRESS

- **M2: Praxis Template Upgrades**
  - Scope: Modify the `praxis/template` (and workspace config `Cargo.workspace.toml`) to include `lsp` and `ggen` features, dependencies, and scaffolded Rust code (`src/lsp.rs`, ZSTs, witness, and seal patterns in `src/types.rs`).
  - Verification: Worker build and test verification.
  - Status: PLANNED

- **M3: Programmatic Verification Harness**
  - Scope: Implement a programmatic verification script/tool (e.g. in `tools/hollow-gate` or as a bin target) that executes the generator to produce a sample project, checks that it compiles, and parses/verifies the output codebase contains typestate and `RulePackServer` structures.
  - Verification: Run check of the script.
  - Status: PLANNED

- **M4: End-to-End Generation & Verification**
  - Scope: Execute the upgraded praxis generator, compile the emitted sample project, and verify conformance using the programmatic verification script. Produce cryptographic receipts (BLAKE3).
  - Verification: Complete E2E run output verification.
  - Status: PLANNED

- **M5: Quality Audit & Verification**
  - Scope: Perform forensic integrity audit and challenger validation checks on the upgraded generator and sample output.
  - Verification: Forensic Auditor clean report.
  - Status: PLANNED

## Team Roster & Roster Status
- **Explorer 1 (Ecosystem Cataloger)**: `teamwork_preview_explorer` (Conv ID: TBD) -> M1 investigation.
- **Worker 1 (Template Developer)**: `teamwork_preview_worker` (Conv ID: TBD) -> M2 development.
- **Worker 2 (Harness Developer)**: `teamwork_preview_worker` (Conv ID: TBD) -> M3 development.
- **Reviewer 1 (Quality Verifier)**: `teamwork_preview_reviewer` (Conv ID: TBD) -> Review and feedback.
- **Auditor 1 (Integrity Auditor)**: `teamwork_preview_auditor` (Conv ID: TBD) -> Verification check.
