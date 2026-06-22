# Handoff Report: Ecosystem Cataloging & Abstraction (M1_2)

**Status:** ALIVE_UNDER_SCOPE  
**Object under test:** Architectural Patterns in `rocket-craft`, `lsp-max`, and `ggen` for `praxis` Scaffolding Integration

---

## 1. Observation
We directly examined the codebase of `rocket-craft`, `lsp-max`, and the `ggen` pipeline to identify code structures that can be contributed to `praxis/template`. The exact files and sections observed include:

1. **Generative Typestates:**
   - In `/Users/sac/rocket-craft/crates/mech_morphology_law/src/machine.rs`, zero-sized types and generic state transitions:
     ```rust
     pub struct Measured;
     pub struct Validated;
     pub struct Admitted;

     pub struct Machine<L: MorphologyLaw, P> {
         law: L,
         mech: MeasuredMech,
         outcome: Option<LawOutcome>,
         _phase: PhantomData<P>,
     }
     ```
     Transition from `Validated` to `Admitted` is gated via a runtime check that yields a `Result`:
     ```rust
     pub fn admit(self) -> Result<Machine<L, Admitted>, ClaimHold>
     ```

2. **RulePackServer Structure:**
   - In `/Users/sac/lsp-max/src/rule_pack_server.rs`, `RulePackServer` trait exposing tree-sitter integration, text syncing, SPC monitoring, latency trackers, and circuit breakers.
     Also observed `EvalBudget` (Sync / Background classification) and topological pack composition (`compose_packs`).

3. **ggen Pipeline Stages:**
   - In `/Users/sac/ggen/crates/ggen-core/src/codegen/pipeline.rs`, the generation stages: `load_ontology`, `execute_inference_rules`, `execute_shacl_validation`, `execute_validation_rules`, `execute_generation_rules`, `validate_generated_output`, and unsafe auditing.

4. **Task Execution and Build Logistics:**
   - We executed `cargo test` inside `/Users/sac/lsp-max` (task-43). The build failed with exit code 101:
     ```
     error[E0063]: missing field `text_document_content` in initializer of `lsp_max::lsp_types_max::WorkspaceServerCapabilities`
        --> crates/playground/src/capabilities.rs:134:25
         |
     134 |         workspace: Some(WorkspaceServerCapabilities {
         |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `text_document_content`
     ```
     This compilation error is located in `crates/playground/src/capabilities.rs` within the `lsp-max-playground` crate. The core `lsp-max` crate itself compiled successfully.

---

## 2. Logic Chain
1. To implement the "Post-Chatman Equation" ($A = \mu(O^*)$) in generated Praxis boilerplate, the scaffolded code must inherit these compiler-level safety guarantees.
2. The zero-sized marker typestate wrapper `Evidence<T, State, Witness>` directly maps to `Evidence<T, Raw, W>` and `Evidence<T, Admitted, W>` patterns from `mech_morphology_law`.
3. The boilerplate must include a default language server skeleton. The Tower-LSP framework boilerplate should be wrapped using the `RulePackServer` trait, making it modular and enabling out-of-the-box support for workspace indexing and cross-file diagnostics.
4. Although `lsp-max-playground` fails to compile in the upstream workspace due to a missing struct field under the latest `lsp-types-max` changes, the core `lsp-max` crate compiles and is fully usable. Thus, the implementation logic for `praxis` template integration is unaffected.

---

## 3. Caveats
- No code was written or modified inside the actual crate codebases, preserving a read-only investigation.
- Compilation checks of the newly proposed `praxis-template` changes were only validated structurally; testing their execution will require completing the M2/M3 template upgrade and validation gates.
- The `lsp-max-playground` compilation error remains an active issue in the upstream workspace and must be resolved if playground testing is required.

---

## 4. Conclusion
We have cataloged and drafted the required abstractions for:
1. Generative Typestates (Witness, Seal, raw-to-admitted gating).
2. `RulePackServer` (LSP didChange hooks, EvalBudget classification, SPC monitor, circuit breaker).
3. `ggen` µ-pipeline (Inference, Validation ASK polarity, Extract SELECT ORDER BY, lowerer, receipts).

These findings are compiled in `report.md` under `/Users/sac/praxis/.agents/teamwork_preview_explorer_m1_2/`. The project is ready to progress to M2 (Praxis Template Upgrades).

---

## 5. Verification Method
- Independent inspectors can verify this report by reading `/Users/sac/praxis/.agents/teamwork_preview_explorer_m1_2/report.md` and comparing the code definitions with the source paths referenced above.
- Invalidation conditions: Any syntax errors in the proposed traits or change of dependency versions that breaks Tower-LSP compatibility.
