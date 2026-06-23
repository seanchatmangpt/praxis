# Current State of ~/praxis Directory

## 1. Directory Contents and Purpose
`~/praxis` serves as the centralized Rust "house style" and boilerplate generation kit for `seanchatmangpt`. Its goal is to provide a single place that defines scaffolding, linting, building, and releasing for all Rust repositories in the ecosystem. It applies a strong structural verification methodology known as the "Post-Chatman Equation" ($A = \mu(O^*)$).

**Key Contents:**
- `template/`: The `cargo generate` template that includes scaffolding. It's pre-configured with zero-sized types for compilation-gated phase transitions (`Raw`, `Validated`, `Admitted`), `RulePackServer` structures from `lsp-max`, and `ggen` micro-pipeline skeletons.
- `crates/chatman-common/`: The shared "house crate" providing unified error handling, telemetry, and BLAKE3 provenance helpers.
- `tools/`: Contains programmatic verification harnesses like `verify_conformance.sh` and the `hollow-gate` Rust tool to enforce structural conformance and prevent stubs/mocks.
- `survey/`: Contains an ecosystem catalog and synthesis reports from 10 agents that surveyed 18 repositories to extract these "house style" rulings.
- `apply.sh`: A shell script used to backfill these standard hygiene files into existing repositories.
- `PROJECT.md`, `README.md`, `WHAT_IT_DOES.md`: Detailed documentation and architecture constraints.

## 2. Architecture and Constraints Enforced
Applying `praxis` ensures all projects adhere strictly to these architectural constraints:
- **Generative Typestates & The Seal Pattern:** Uses `Evidence<T, State, Witness>`, `PhantomData`, ZSTs (`Raw`, `Validated`, `Admitted`), and an `Admit` trait to strictly control and verify state transitions at compile time.
- **`RulePackServer` Integration:** Integrates language server structures wrapping `tower-lsp` to handle workspace indexing, text syncing, and incremental AST parsing.
- **Micro-Pipeline Integration (`ggen`):** Projects incorporate `ggen` workflows driven by ontological definitions, SHACL/SPARQL validations, and BLAKE3 cryptographic receipts for execution provenance.
- **CLI Architecture:** Commands must follow a noun-verb architecture via `clap-noun-verb`, separating argument parsing from business logic execution.
- **Link-Time Plugin Discovery:** Employs `linkme::distributed_slice` to allow downstream crates to register handlers without runtime scanning.
- **Unified Hygiene:** Consistent CalVer versioning, CI/CD GitHub Actions workflows, toolchain pins, and rigorous lint rules (e.g., forbidding `unsafe_code`, `todo!`, `unimplemented!`, and `dbg!`).

## 3. Current Project State
According to `PROJECT.md`, all core milestones (M1 through M5) are marked as **DONE**:
1. Ecosystem cataloging and abstraction of `rocket-craft`/`lsp-max` patterns into the `survey/` directory.
2. `praxis/template` upgrades to natively include `lsp` and `ggen` features.
3. Creation of `tools/hollow-gate` for structural conformance verification.
4. E2E Generation and programmatic verification (`tools/verify_conformance.sh`).
5. Quality audit and review.

The directory is fully functional and successfully defines the laws and infrastructure required for structurally conforming Rust applications under the "Post-Chatman" architecture.
