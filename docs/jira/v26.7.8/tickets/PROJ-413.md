# PROJ-413: Implement `ChatmanEngine` Core Pipeline Orchestration
**Title:** Implement the Direct-Operation Pipeline (`RDFState` to `Receipt`)
**Type:** Feature
**Target:** `/Users/sac/praxis` (module: `src/chatman/`)
**Status:** IN PROGRESS (workflow wf_255e0807)

> Note: path details in this ticket are superseded by
> `docs/chatman-engine/DEFINITION_OF_DONE.md`.

## Description
Implement the core `ChatmanEngine` structural pipeline. The execution must strictly follow the bottom-up stack defined in the thesis: `RDFState -> OWLClosure -> PDDLPlan -> POWLAdmission -> KnowledgeHook -> Receipt`.

## Implementation Spec
1. **New File:** `/Users/sac/praxis/src/chatman/engine.rs` (approx. 120 LOC)
   * Define `pub struct ChatmanEngine { store: oxigraph::store::Store }`.
   * Define `pub fn admit_transition(&mut self, invocation: InvocationEnvelope) -> Result<AdmittedTransition, Refusal>`.
   * **fetch_snapshot:** Map `invocation.snapshot_id` to an `oxigraph::model::NamedNode` and verify `store.contains_named_graph` (returns `Refusal::SnapshotNotFound`).
   * **apply_owl_closure:** Query the graph for SHACL/ShEx violations using `store.query`.
   * **generate_pddl_plan:** Query the Oxigraph store for `pddl:hasDomainText` and `pddl:hasProblemText` literals. Parse them via `bcinr_pddl::parse::domain_from_pddl` and `problem_from_pddl`. Validate groundability via `bcinr_pddl::ground::GroundProblem::build` (returns `Refusal::PlanInfeasible`).
   * **admit_powl_trace:** Query the store for `powl:hasOcelLogText`. Deserialize using `bcinr_powl::ocel::OCEL`. (returns `Refusal::TraceUnlawful`).
   * **trigger_knowledge_hooks:** Query the store for instances of `hook:KnowledgeHook` where `hook:triggered` is true. Verify `hook:permitted` is also true (returns `Refusal::HookUnpermitted`).

2. **File Mod:** register `pub mod engine;` in the `src/chatman/` module root, below
   `pub mod abi;`.

## Acceptance Criteria
- [ ] No stubs or placeholders. All `bcinr_pddl`, `bcinr_powl`, and `oxigraph` boundaries are invoked with real Rust types.
- [ ] Engine compilation cleanly links the `oxigraph` SPARQL queries to the `bcinr` parsers.
