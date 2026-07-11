# Chapter 3. The Decomposition Functor: Compiling PDDL to POWL v2

## 3.1 Introduction
In the Praxis architecture, the transition from declarative intent to executable consequence is not merely a compilation step; it is formalized as a structure-preserving transformation between two distinct semantic categories. We define the **Decomposition Functor**, denoted mathematically as $\mu_{\text{cng}}$, which physically instantiates this mapping via the `cng` (Chatman Engine) crate. 

This functor resolves human and AI intent—expressed as fluid, declarative Planning Domain Definition Language (PDDL) models encoded in RDF Turtle—into a rigid, partially ordered executable state machine defined by POWL v2. In doing so, it collapses the combinatorial search space of intelligent intent into a deterministic, coinductive stream of lawful execution.

## 3.2 The Categorical Formulation of Intent and Execution

Let $\mathcal{C}_{Int}$ be the category of declarative intent, where objects are state configurations derived from the admitted observation graph $O^*$, and morphisms are allowable actions constrained by domain rules ($P, C$). In this category, intent is fluid; a path to a goal is guaranteed to exist, but not strictly ordered.

Let $\mathcal{C}_{Exec}$ be the category of executable processes (POWL v2), where objects are strictly bounded execution states (`powl2:ActivityLeaf`) and morphisms represent irrefutable partial-order precedence bindings (`powl2:precedes`).

The Chatman Equation establishes the macro-manufacturing function $A = \mu(O^*)$. The `cng` crate physicalizes a specific functorial projection within this equation:
$$ \mu_{\text{cng}}: \mathcal{C}_{Int} \to \mathcal{C}_{Exec} $$

This functor is strictly structure-preserving, total, and deterministic. It guarantees that for any admitted set of RDF PDDL artifacts, $\mu_{\text{cng}}$ maps to exactly one valid RDF POWL v2 workflow artifact. It preserves provenance via `prov:wasDerivedFrom` and maintains semantic authority in the public ontology without introducing any private, in-memory representations.

## 3.3 The Functorial Mapping (The `cng` Projection)

The `cng` implementation enacts a constructive mapping that collapses the abstract space of PDDL into the concrete execution tape of POWL v2. The functorial mapping is explicitly defined by the public ontology transformation:

| Domain Category ($\mathcal{C}_{Int}$: PDDL RDF) | Codomain Category ($\mathcal{C}_{Exec}$: POWL v2 RDF) | Morphism / Constraints |
|-------------------------------------------------|-------------------------------------------------------|------------------------|
| Admitted Artifact Set                           | `powl2:Model` root (`<base>/n0`)                        | Identity preservation  |
| Combined Plan (Total/Partial Order)             | `powl2:PartialOrder`                                  | Coinductive unfolding  |
| Plan Operation $i$ (Ground Action)              | `powl2:ActivityLeaf` at `<base>/n0/c<i>`              | Action localization    |
| Plan Position $i$                               | `powl2:ChildBinding` (`childIndex`, `childModel`)     | Structural binding     |
| Plan Order, transitively closed ($i < j$)       | `powl2:precedes` between two bindings                 | Temporal rigidity      |
| Composed Plan Source IRI                        | `powl2:derivedFrom` on the root                       | Provenance closure     |
| Source Artifacts (Content-Addressed)            | `prov:wasDerivedFrom <urn:blake3:...>`                | Cryptographic lineage  |

Through this mapping, semantic authority stays entirely in the imported and exported RDF artifacts. The $\mu_{\text{cng}}$ functor generates stable hashes (BLAKE3) ensuring that every generated POWL artifact is deterministically reproducible under a fixed seed.

## 3.4 Collapsing Intent into a Coinductive Stream

The combinatorial space of PDDL must be safely collapsed into a bounded execution tape (strictly $\leq 64$ operations within the `bcinr-powl` runtime constraint) without semantic loss. The functor achieves this by projecting the solution path into a non-branching, coinductive stream.

In the execution phase, the generated POWL AST is lowered to `bcinr_powl::compiler::PowlAstNode`, admitted via Kahn acyclicity and reachability checks, and processed by a branchless `scheduler_tick` loop. The execution can be viewed mathematically as a coinductive stream:
$$ \text{unfold} : State \to (Event \times State) $$

The runtime enforces, per tick, that no activity fires before its strictly projected predecessors. The artifact is not merely executed; it serves as a strict *conformance artifact*. If the coinductive execution violates the partial order topology established by $\mu_{\text{cng}}$, execution mathematically halts.

## 3.5 The Refusal Algebra as Functorial Adjoints

Because $\mu_{\text{cng}}$ must maintain absolute systemic rigor, any mapping failure must be lifted into the codomain as a typed refusal rather than a systemic panic. This Refusal Algebra strictly adheres to the Rust AGI Core Team invariants: every error is a typed `Refusal` variant, with zero panics or silent defaults. 

The functor maps unresolvable elements of $\mathcal{C}_{Int}$ to explicit refusal states:
- $\bot_{Shape} \mapsto \text{CNG\_R06: InvalidPowl}$ (Shape violation)
- $\bot_{Reach} \mapsto \text{CNG\_R04: PlanUnsolvable}$ (Unreachable goal)
- $\bot_{Bounds} \mapsto \text{CNG\_R05: UnsupportedConstruct}$ (Branching intent or tape bounds exceeded)

By handling failures within the algebraic structure itself, $\mu_{\text{cng}}$ guarantees that no silent fallbacks or hand-authored placeholders bypass the transformation. The categorical structure strictly refuses what it cannot map.

## 3.6 Enriched Associativity via Commuting Squares
The decomposition of concurrent POWL sequences (e.g., `helper ∥ main`) is mathematically bound by **Commuting Squares** rather than the traditional Mac Lane Pentagon. The `cng` compiler enforces enriched associativity, ensuring that parallel task execution paths mathematically commute to the exact same BLAKE3 state digest regardless of thread interleaving.

## 3.7 The Universal Final Coalgebra ($M$-Types)
The branchless `scheduler_tick` loop executing the POWL graph is the literal Rust instantiation of the **Universal Final Coalgebra** ($M$-Type). Because the enterprise workflow is a non-terminating reactive system, it cannot be modeled by strictly well-founded inductive W-Types. The `bcinr-powl` runtime unfolds execution states coinductively, establishing rigorous bisimulation proofs to guarantee systemic liveness.

## 3.8 Conclusion

The `cng` crate establishes the vital mathematical boundary $\mu$ that translates declarative AI intent (the unstructured "what") into an executable, immutable machine state (the rigid "how"). Through rigorous categorical alignment, cryptographic content addressing, and coinductive runtime strategies, the Decomposition Functor ensures that every executed workflow inherits the irrefutable standing of its native reality.
