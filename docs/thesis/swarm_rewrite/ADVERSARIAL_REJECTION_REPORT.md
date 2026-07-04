# Adversarial Rejection Report: Pass 7 Rejection Test

This report evaluates the rewritten foundations paper located at `/Users/sac/praxis/docs/thesis/swarm_rewrite/00_foundations_rewritten.tex` against the 14 Pass 7 Rejection Criteria, the Prime Directive, internal tag correctness, and compilation stability.

## Review Summary

**Verdict**: **REQUEST_CHANGES** (Rejection)

The rewritten paper is structurally sound and compiles cleanly to PDF, and all associated codebase tests pass. However, it fails several critical Pass 7 Rejection Criteria—specifically, a duplicate proof of Rice's Theorem (Criterion 12), multiple cases of unintroduced mathematical and technical notation (Criterion 13), and theorems depending on prose or containing undefined terms (Criterion 14). Additionally, there are sentence-level tagging gaps violating the Prime Directive.

---

## Detailed Pass 7 Rejection Criteria Audit

### 1. "Only way" claims without a formal theorem
* **Status**: **PASS**
* **Verification**: Grep search for "only way" returned no results. The exposition frames the proposed system as "one architecture in which scalable action standing is obtained" (Line 172-173), avoiding absolute exclusivity claims.

### 2. "Mathematically guarantee correctness" without defining correctness
* **Status**: **PASS**
* **Verification**: The paper explicitly clarifies that it does "not claim absolute semantic correctness of model outputs, but rather verify that actuation conforms strictly to the specified syntactic and lifecycle constraints" (Line 91-92). Further, Line 567 states: "A receipt proves committed trace structure, not moral or semantic correctness."

### 3. "Published" without a public publication receipt
* **Status**: **PASS**
* **Verification**: The term "published" does not appear in the text. Prior work is referenced strictly via standard citation syntax (e.g., `\cite{chatman2025}`).

### 4. "Planetary scale" without a bounded model
* **Status**: **PASS**
* **Verification**: The term "planetary" is not used. Scalability claims are modeled and bounded using the human cognitive capacity constant $\kappa$ (where $\kappa \approx 4$).

### 5. "Physics" without a formally defined invariant
* **Status**: **PASS**
* **Verification**: Neither "physics" nor "physical" is referenced in the document.

### 6. "Trust" without specifying the receipt claim
* **Status**: **PASS**
* **Verification**: The word "trust" is only used in context (e.g., "Trust Without Comprehension" title and "cannot be trusted via direct runtime comprehension" in Line 109), followed by the receipt boundary mapping. Line 769 bounds the scope: "We do not analyze multi-agent game-theoretic trust dynamics; our scope is limited to single-system receipt boundaries."

### 7. "Agent" without a bounded capability definition
* **Status**: **PASS**
* **Verification**: The agent's actuation capability is formally modeled and bounded by the manufacturing morphism $\muop$ (Definition 6.1: "terminating with a priori bounded cost") and register-level constraints (the standing byte $b \in \mathbb{B}^8$).

### 8. "Receipt proves virtue" confusion
* **Status**: **PASS**
* **Verification**: Line 567 explicitly separates trace commitment from virtue/correctness: "A receipt proves committed trace structure, not moral or semantic correctness."

### 9. Rice theorem used too broadly
* **Status**: **PASS**
* **Verification**: Theorem 3.2 is correctly restricted: it instantiates Rice's Theorem on the observation space where observations range over finite encodings denoting programs, proving the undecidability of non-trivial semantic properties of denoted functions.

### 10. Code claim without code correspondence
* **Status**: **PASS**
* **Verification**: All code files listed in Appendix B (Table 2) exist in the `/Users/sac/praxis/` repository. Key structures (such as `LawObject`, `DenialPolarity`, sealed `Stage` trait, `PowlReplayVerifier`, and BLAKE3 hash frame) align with the implementations. Project tests were run and pass successfully.

### 11. Benchmark claim without benchmark receipt
* **Status**: **PASS**
* **Verification**: No quantitative benchmark performance numbers (e.g., execution times or throughput) are claimed in the paper. The benchmark suite `crates/agent8/benches/fleet_sweep.rs` is listed only as a verifying test code file in Appendix C.

### 12. Duplicate Rice exposition
* **Status**: **FAIL (Critical)**
* **Verification**: Two distinct `proof` environments are written back-to-back under `Theorem 3.2` (Rice, specialized to observations):
  - Proof 1 (Lines 247-253) halts abruptly at: `"Then $o_{M,x}$ denotes a function with property $P$ iff $M(x)$ halts."`
  - Proof 2 (Lines 255-262) repeats the same construction but completes the contradiction: `"Thus, $D(o_{M,x})$ decides the halting problem, which is a contradiction. Hence, no such decider $D$ exists."`
  - **Correction required**: Remove the duplicate/incomplete first proof block.

### 13. Unintroduced notation
* **Status**: **FAIL (Major)**
* **Verification**: The following mathematical and structural notation is used before being defined:
  - $\Rfsl$ and $\Obsbot$ are used extensively in Chapter 2 (e.g., Definition 2.1, Construction 2.1) and Chapter 3, but are formally defined only in Chapter 4 (Definition 4.1).
  - The domain $\mathbb{B}^8$ (Chapter 4, Construction 4.2) is never defined in the document.
  - The register operation alphabet $\mathcal{I}_8 = \{ \wedge_8, \vee_8, \oplus_8, \neg_8, \text{z}, \text{nz}, \text{pop}_8 \}$ is introduced in Chapter 4 (Construction 4.2) without mathematical definitions for the operators.
  - Concatenation operator $\parallel$ and causal frame bracket notation $\langle \cdot, \cdot \rangle$ in Construction 7.1 are not defined.
  - "Q16.16 format" in Chapter 5 (Definition 5.4) is a representation format but is not mathematically defined.
  - **Correction required**: Define $\Obsbot$ and $\Rfsl$ in the notation section or in Chapter 2. Define operations in $\mathcal{I}_8$ and formatting conventions.

### 14. Theorem depending on prose
* **Status**: **FAIL (Major)**
* **Verification**:
  - **Proposition 4.2 (Monotonicity of Denial)**: Statement uses $d_G(o)$ and $d_{G'}(o)$ without defining them in the statement or in a prior definition; the definition is introduced only in the proof.
  - **Theorem 4.3 (Constant-Depth Eligibility)**: Statement references "The gate, mask, and select operations", but "mask" is defined only in the proof, and "select" is never defined.
  - **Theorem 5.2 (Illegal Transitions are Uninhabited)**: Statement references Rust compiler-specific concepts ("stages be represented as Rust types", "fails to compile") rather than purely mathematical terms.
  - **Correction required**: Ensure all terms and predicates are defined in the construction/definition before the theorem is stated. Remove compiler/host-language dependence from mathematical theorem statements.

---

## Prime Directive Tagging Audit

The document enforces `% [TAG]` sentence-level comments on almost all lines. However, the following gaps violate the strict requirement that every sentence must have an associated category:

1. **Chapter Summaries**: The summary itemize lines in all chapters (e.g., lines 151-153, 215-217, etc.) contain complete sentences detailing "What was admitted/refused" but are missing comment tags. These represent reader orientation and should end with `% [ORIENT]`.
2. **List Introductions**: Lines introducing structural lists (e.g., Line 184: `The Chatman equation relates five spaces and maps:`) are untagged sentence clauses.

---

## Technical Verification Results

### 1. LaTeX Compilation Check
* **Command**: `pdflatex -interaction=nonstopmode 00_foundations_rewritten.tex`
* **Status**: **PASS** (33 pages, 315503 bytes, compiled successfully without fatal errors).

### 2. Codebase Unit & Integration Tests
* **Command**: `cargo test`
* **Status**: **PASS** (all tests passed, including `snapshots_verbs`, `config_admission`, `differential`, `indexed_grounding`, and `frontier_matrix`).

---

## Recommendations & Action Items

1. **Remove Duplicate Proof**: Delete the first `\begin{proof} ... \end{proof}` block (Lines 247-253) under Theorem 3.2.
2. **Re-order Definition of $\Rfsl$ and $\Obsbot$**: Move the definitions of the extended observation space $\Obsbot$ and the refusal constant $\Rfsl$ to Chapter 2 (prior to Definition 2.1) or Appendix A (Notation) to prevent unintroduced notation.
3. **Define $\mathcal{I}_8$ and $\mathbb{B}^8$**: Add explicit definitions for $\mathbb{B}^8$ (8-bit binary words) and the register operators (bitwise AND, OR, XOR, NOT, zero-test, non-zero-test, population count) in Chapter 4.
4. **Define $d_G(o)$ and $P_m(b)$ before theorems**: Define the denial word projection $d_G(o)$ in Construction 4.1, and define the mask predicate $P_m(b)$ in Construction 4.2.
5. **Clean up compiler concepts in Theorem 5.2**: State the theorem in terms of the path category $\Life$ (where out-of-order transitions correspond to empty hom-sets), and move the Rust compilation mapping to the Operational Correspondence section.
6. **Apply Prime Directive tags to Chapter Summaries**: Add `% [ORIENT]` comments to the end of all summary items inside the `itemize` environments.
