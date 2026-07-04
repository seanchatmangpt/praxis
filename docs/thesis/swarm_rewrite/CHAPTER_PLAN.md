# Table of Contents and Chapter Plan — The Chatman Equation Thesis Program

This document outlines the 12-chapter plan and 3 appendices for the rewrite of the Chatman Equation Thesis Program (Paper 00 Foundations). It establishes a rigorous logical dependency order, maps mathematical objects to code artifacts in the `praxis` codebase, and resolves the four major inconsistencies identified in the dependency graph critique.

---

## Dependency Order & Logical Flow

The chapter plan adheres strictly to the following dependency constraint chain:
`Problem` (Ch 1) $\to$ `Notation/Equation` (Ch 2) $\to$ `Boundary` (Ch 3) $\to$ `Admission Algebra` (Ch 4) $\to$ `Lifecycle/Typestate` (Ch 5) $\to$ `Manufacture` (Ch 6) $\to$ `Receipt` (Ch 7) $\to$ `BRCE` (Ch 8) $\to$ `Conservation` (Ch 9) $\to$ `Worked Example` (Ch 10) $\to$ `Scale/Map` (Ch 11) $\to$ `Conclusion` (Ch 12).

---

## Chapter Plan

### Chapter 0: Abstract
*   **Role & Objective**: Summarize the thesis contribution, introducing the factored Chatman Equation, the five objects, the computability boundary, typestate lifecycles, cryptographic receipt chains, and the conservation of consequence.
*   **Mathematical Objects**: $\mathcal{O}$, $\mathcal{O}^*$, $\alpha$, $\bot$, $\mu$, $\mathcal{A}$, $\mathcal{R}$, $\mathcal{H}$, $\kappa$, $\mathcal{Life}$, $\mathsf{BRCE}$.
*   **Logical Dependencies**: Summarizes Chapters 1–12.

---

### Chapter 1: The Problem: Trust Without Comprehension
*   **Role & Objective**: Motivate the research. Address the cognitive mismatch between human working-memory capacity ($\kappa \approx 4$) and the astronomical state spaces of agentic AI systems. Define the comprehension-verification gap.
*   **Mathematical Objects**: Cognitive capacity limit ($\kappa$), Comprehension-verification gap ($\mathrm{Comp}(\sigma) / \mathrm{Ver}_{\partial}(\sigma) \to \infty$).
*   **Logical Dependencies**: None (fundamental problem statement).
*   **Code Correspondence**: N/A (theoretical framing).

---

### Chapter 2: The Equation: Five Objects and One Route
*   **Role & Objective**: Introduce the governing factored Chatman Equation $\mathcal{A} = \mu(\mathcal{O}^*)$ with $\mathcal{R} = \mathrm{receipt}(\mathcal{A})$, and define the five core algebraic spaces.
*   **Mathematical Objects**:
    *   $\mathcal{O}$: Raw Observation Space
    *   $\mathcal{O}^*$: Admitted Observation Space
    *   $\alpha$: Admission Map
    *   $\mu$: Manufacturing Morphism
    *   $\mathcal{A}$: Artifact Space
    *   $\mathcal{R}$: Receipt Space
*   **Logical Dependencies**: Chapter 1 (problem motivation).
*   **Code Correspondence**: `crates/praxis-core/src/law.rs` (defining `LawObject` and stage pipeline).
*   **Resolution of Inconsistencies**:
    *   *Domain/Codomain of $\mu$*: Pre-emptively defines the signature of $\mu$ to include the refusal element: $\mu: \mathcal{O}^* \cup \{\bot\} \to \mathcal{A} \cup \{\bot\}$, establishing that the equation evaluates successfully on the admitted subspace and propagates refusal cleanly.

---

### Chapter 3: The Computability Boundary: Why Admission Exists
*   **Role & Objective**: Formulate the boundary separating raw input $\mathcal{O}$ from admitted inputs $\mathcal{O}^*$ using computability theory. Specialize Rice's Theorem to show that arbitrary semantic properties of user-submitted observations are undecidable by a total program, necessitating a syntactic quarantine.
*   **Mathematical Objects**: Universal computer model $\mathcal{U}$, Observation space $\mathcal{O}$, Semantic properties, Decidable boundary.
*   **Logical Dependencies**: Chapter 2 (introduces raw and admitted spaces).
*   **Code Correspondence**: Raw payload parsing of `serde_json::Value` in `crates/praxis-core/src/law.rs`.

---

### Chapter 4: Refusal as Data: The Admission Algebra
*   **Role & Objective**: Formally define the admission map $\alpha$ as a computable partial retraction. Establish the algebraic properties of the refusal constant $\bot$ as a first-class value, refusal composition via the denial monoid, and the exhaustive taxonomy of refusal categories.
*   **Mathematical Objects**:
    *   $\mathcal{O}_\bot$: Extended Observation Space ($\mathcal{O} \cup \{\bot\}$)
    *   $\alpha$: Admission Map ($\alpha: \mathcal{O}_\bot \to \mathcal{O}^* \cup \{\bot\}$)
    *   $\bot$: Refusal Constant
    *   $D$: Denial Monoid (commutative idempotent join-semilattice $(\{0, 1\}^n, \lor, \mathbf{0})$)
    *   $C$: Refusal Taxonomy Categories (Identity, Capacity, Topology, Temporal, Lifecycle, Authorization, Prerequisites, Reserved)
*   **Logical Dependencies**: Chapter 3 (justifies syntactic quarantine boundary).
*   **Code Correspondence**:
    *   `Andon::Halted` in `crates/praxis-core/src/law.rs` representing $\bot$.
    *   `DenialPolarity` and `compose_denials` in `crates/praxis-core/src/refusal.rs` representing the denial monoid.
    *   `RefusalCategory` enum and exhaustiveness match checks in `crates/praxis-core/src/refusal.rs`.
*   **Resolution of Inconsistencies**:
    *   *$\bot$'s Status in $\mathcal{O}$*: Resolves the circularity of $\alpha(\bot) = \bot$ by introducing the extended observation space $\mathcal{O}_\bot = \mathcal{O} \cup \{\bot\}$. The admission map is defined over this extended domain, ensuring the idempotence theorem $\alpha \circ \alpha = \alpha$ holds rigorously across all inputs.

---

### Chapter 5: Lifecycle Order: Typestate and Process
*   **Role & Objective**: Formulate lifecycle transitions using category theory and Petri nets. Interpret stages as types and show that illegal lifecycle transitions are uninhabited hom-sets, resulting in compiler type errors in the host language.
*   **Mathematical Objects**:
    *   Category $\mathcal{Life}$: Objects $\{\mathsf{Raw}, \mathsf{Validated}, \mathsf{Admitted}, \mathsf{Receipted}\}$, arrows $\{j, a, r\}$
    *   Morphisms, Hom-sets
    *   Petri Net Replay Game, Conformance Fitness $\varphi \in [0, 1]$
*   **Logical Dependencies**: Chapter 4 (defines admission and refusal values).
*   **Code Correspondence**:
    *   Sealed trait `Stage` and type parameters on `LawObject` in `crates/praxis-core/src/lifecycle.rs`.
    *   Type signatures of `Judge::judge`, `Admit::admit`, and `Receipt` transitions in `crates/praxis-core/src/law.rs`.
    *   `PowlReplayVerifier` in `crates/praxis-core/src/replay_adapter.rs`.
*   **Resolution of Inconsistencies**:
    *   *Lifecycle Stages vs. Retraction Map*: Reconciles the single-step algebraic retraction $\alpha$ with the category transitions $j$ (judgment) and $a$ (admission). Formally factors the retraction as $\alpha = a \circ j$, where $j: \mathcal{O}_\bot \to \mathcal{O}_\bot \cup \{\bot\}$ evaluates the obligations, and $a: \mathcal{O}_\bot \cup \{\bot\} \to \mathcal{O}^* \cup \{\bot\}$ normalizes the payload. The intermediate stage $\mathsf{Validated}$ corresponds to the output of $j$ before normalization, providing a direct category-theoretic mapping.

---

### Chapter 6: Manufacture: Deterministic Bounded Production
*   **Role & Objective**: Formulate the manufacturing morphism $\mu$. Impose the axioms of determinism/reproducibility (M1) and size/complexity boundedness (M2) on artifact generation.
*   **Mathematical Objects**:
    *   Manufacturing morphism $\mu: \mathcal{O}^* \cup \{\bot\} \to \mathcal{A} \cup \{\bot\}$
    *   Axioms (M1) and (M2)
*   **Logical Dependencies**: Chapter 5 (advances from admitted stage to manufacture).
*   **Code Correspondence**: Ontology-to-PDDL compilation in `src/verbs/mfg.rs` utilizing the `bcinr-pddl` compiler.
*   **Resolution of Inconsistencies**:
    *   *Domain and Codomain Mismatch of $\mu$*: Formally adopts the signature $\mu: \mathcal{O}^* \cup \{\bot\} \to \mathcal{A} \cup \{\bot\}$ with $\mu(\bot) = \bot$. This resolves the domain error of evaluating $\mu$ on non-admitted or failed inputs and establishes refusal propagation.

---

### Chapter 7: Receipts: Commitment, Chain, and Replay
*   **Role & Objective**: Detail the structure of validation receipts. Formulate the cryptographic causal chain using hash commitment, proving that the latest receipt hash commits to the entire history of execution.
*   **Mathematical Objects**:
    *   Receipt tuple $r = (\mathrm{verdict}, h_+, \varphi, \mathrm{reason}) \in \mathcal{R}$
    *   Collision-resistant hash function $\mathcal{H}: \{0, 1\}^* \to \{0, 1\}^{256}$
    *   Causal frame $\mathsf{fr} = \langle\theta, \mathcal{H}(b)\rangle$ and chain relation $h_+ = \mathcal{H}(h_- \parallel \mathsf{fr})$
*   **Logical Dependencies**: Chapter 6 (manufacture outputs require receipts).
*   **Code Correspondence**:
    *   `OcelCausalFrame` hash calculation and BLAKE3 binding in `crates/praxis-core/src/law.rs`.
    *   `ReceiptRecord` and `ReceiptMeta` structs.

---

### Chapter 8: The Enforced Equation: BRCE
*   **Role & Objective**: Establish the Bounded Receipted Chatman Equation ($\mathsf{BRCE}$) by integrating all components under four system invariants: (B1) Gate, (B2) Bounded Manufacture, (B3) Receipt Totality, and (B4) Conformance.
*   **Mathematical Objects**:
    *   Invariants: B1, B2, B3, B4
    *   Verification predicates
*   **Logical Dependencies**: Chapter 7 (requires receipt structures).
*   **Code Correspondence**: Verification verification pipeline in `crates/praxis-core/src/verify.rs`.

---

### Chapter 9: Conservation of Consequence
*   **Role & Objective**: State and prove the primary conservation theorem under the $\mathsf{BRCE}$ invariants. Prove that receiptless action is excluded by construction, and that every actuation commits to an admitted observation digest.
*   **Mathematical Objects**:
    *   Conservation of Consequence theorem
    *   Actuation map, receipt positions, digest commitment map $\mathrm{dg}(x)$
*   **Logical Dependencies**: Chapter 8 (requires $\mathsf{BRCE}$ invariants).
*   **Code Correspondence**: Integration tests in `crates/praxis-core/tests/prop_law.rs`.
*   **Resolution of Inconsistencies**:
    *   *Conservation of Consequence Flaw (Injectivity vs. Commitment)*: Resolves the mathematical error of claiming $\mu$ is injective. The theorem is rewritten to separate:
        1.  *Causation*: Every actuated artifact $a$ is the image under $\mu$ of *at least one* admitted observation $x \in \mathcal{O}^*$ ($a \in \mathrm{im}(\mu)$).
        2.  *Commitment*: The cryptographic chain commits to the unique observation digest $\mathrm{dg}(x)$ via the causal frame $\mathsf{fr}$. Under collision resistance of $\mathcal{H}$, the receipt position is uniquely bound to that specific input $x$, preventing post-hoc history manipulation even if $\mu$ maps multiple distinct inputs to the same artifact.

---

### Chapter 10: Worked Example: A Contract-Claim Law Object
*   **Role & Objective**: Walk through a complete, concrete execution of the receipt calculus. Demonstrate the transition from a raw contract payload through obligation verification, PDDL compilation, and receipt generation.
*   **Mathematical Objects**: Concrete instances of $\mathcal{O}$, $\alpha$, $\mu$, $\mathcal{A}$, $\mathcal{R}$, and $\mathcal{Life}$.
*   **Logical Dependencies**: Chapters 1–9.
*   **Code Correspondence**: `crates/agent8/src/byte.rs` (representing the `agent8` status byte flags: `blocked` and `receipted`), and integration tests.

---

### Chapter 11: Map of the Thesis Program
*   **Role & Objective**: Situate Paper 00 within the broader Google Antigravity (AGY) research series. Relate the foundational concepts to downstream thesis papers (such as projection, scaling, and execution).
*   **Mathematical Objects**: Scale limits.
*   **Logical Dependencies**: Chapter 9 (conservation theorem).
*   **Code Correspondence**: `ggen.toml` (workspace project) and `crates/chatman-common` (provenance).

---

### Chapter 12: Conclusion
*   **Role & Objective**: Re-summarize findings, highlight the cognitive and mathematical guarantees of the receipt calculus, and outline future extensions.
*   **Mathematical Objects**: None.
*   **Logical Dependencies**: Chapters 1–11.

---

## Appendices

### Appendix A. Notation
*   **Overview**: A comprehensive dictionary defining all mathematical symbols, spaces, maps, and signatures.
*   **Inconsistency Resolution Mapping**:
    *   *Extended Observation Space*: Defines $\mathcal{O}_\bot = \mathcal{O} \cup \{\bot\}$ as the domain of $\alpha$.
    *   *Morphism Signature*: Formally records $\mu: \mathcal{O}^* \cup \{\bot\} \to \mathcal{A} \cup \{\bot\}$.
    *   *Factored Retraction*: Formally defines $\alpha = a \circ j$.

### Appendix B. Code Correspondence
*   **Overview**: A detailed lookup table mapping mathematical definitions, categories, and invariants to actual modules, files, and lines in the `praxis` codebase.

### Appendix C. Claim Standing Table
*   **Overview**: The finalized inventory of all claims extracted from the foundations LaTeX source, tracking their category (Definition, Theorem, Proof, etc.), recommended standing (ADMIT / CUT), and verifying test suites.
