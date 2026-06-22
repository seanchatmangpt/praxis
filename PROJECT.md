# Project: Praxis Boilerplate Generator Upgrade

## Architecture
The upgraded `praxis` boilerplate generator compiles template projects using `cargo generate`. It is being upgraded to natively produce Rust code that implements the "Post-Chatman Equation" ($A = \mu(O^*)$), featuring:
1. **Generative Typestates**: Zero-Sized Types representing compilation-gated phase transitions (e.g. `Raw` -> `Admitted`) using `Evidence<T, State, Witness>`, `Admit` traits, and the Witness and Seal patterns.
2. **`RulePackServer`**: Language server structures wrapping `tower-lsp` to automate text syncing, workspace indexing, incremental AST parsing (via tree-sitter), and latency reclassification (`EvalBudget`).
3. **`ggen` Micro-Pipeline**: Ontological code-generation workflows using TTL ontology definitions, inference, SHACL/SPARQL validation, SELECT extraction, and Tera rendering with cryptographic receipts.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Ecosystem Cataloging & Abstraction | Catalog the `rocket-craft` and `lsp-max` workspaces for patterns, ZSTs, `RulePackServer` traits, and the `ggen` pipeline. | none | IN_PROGRESS |
| M2 | Praxis Template Upgrades | Upgrade `praxis/template` to natively include `lsp` and `ggen` features, dependencies, and code skeleton (`src/lsp.rs`, etc.). | M1 | PLANNED |
| M3 | Programmatic Verification Harness | Create `tools/hollow-gate` verification script to verify that generated projects structurally conform and compile. | M2 | PLANNED |
| M4 | E2E Generation & Verification | Run the generator, compile the output, and run the verification script. Produce BLAKE3 receipts. | M3 | PLANNED |
| M5 | Quality Audit & Review | Reviewer and Forensic Auditor verification checks. | M4 | PLANNED |

## Interface Contracts
- **`Evidence` Transition**: `Evidence<T, Raw, W>` -> `Evidence<T, Admitted, W>` via `Admit` trait.
- **`RulePackServer` Trait**: Implemented by language servers to inherit default protocol handlers.

## Code Layout
- `template/` - Skeleton project for `cargo generate`
- `crates/chatman-common` - Shared house crate
- `tools/hollow-gate` - Verification script (planned)
- `survey/` - Ecosystem catalogs and reports
