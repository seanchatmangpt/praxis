# ProcInt Naming Policy Doctrine

## 1. Zero Arbitrary Decisions
All arbitrary, subjective, or aesthetically-driven naming decisions are strictly forbidden. The naming conventions across the entire `ProcInt` Lean 4 codebase and `ontology.ttl` must directly follow objective characteristics of the mathematical object, structure, or theorem being described.

## 2. Objective Naming Characteristics
Every module, definition, theorem, and type name must be derived from one of the following objective sources:

### A. Original Mathematicians / Authors
Names may be constructed using the concatenations of the original mathematicians or researchers who formalized the concept. 
- *Format:* Hyphenated or concatenated names of the originators.
- *Examples:* `Alexandrov` (topology), `Forman` (discrete Morse theory), `Adriansyah_MunozGama` (temporal profiles), `VanDerAalst` (cubes).

### B. Explicit Topological / Algebraic Properties
Names for types, structures, and definitions must describe the exact explicit topological, causal, or algebraic properties that define them. No semantic shortcuts are permitted.
- *Format:* Verb/noun phrases that reflect the exact set-theoretic, causal, or algebraic invariant.
- *Examples:* `TemporalOrder`, `CausalConsistency`, `ProcessCube`, `cell_subset_slice`, `push_bounded`, `wellPosed_prefixLen_pos`.

### C. Exact Theorem Names
Theorems and proofs must be named for the exact proposition they prove, constructed by linking the subject and the predicate or property being asserted, without any filler words.
- *Format:* `[subject]_[property]` or `[subject]_[action]_[conclusion]`
- *Examples:* `happensBefore_irrefl` (happens-before is irreflexive), `happensBefore_trans` (happens-before is transitive), `archived_only_deletes`, `deleted_terminal`.

## 3. Structural Formatting Rules
- **Types / Structures / Inductives:** Strict `PascalCase` matching the explicit mathematical concept (e.g., `CausalChain`, `LifecycledObject`).
- **Theorems / Definitions:** Strict `camelCase` or `snake_case` reflecting the property mapped (e.g., `cell_dim`, `linked_singleton`).
- **Namespaces:** Must map 1:1 to the explicit ontology domains (e.g., `Analytics.Cube`). The prefix `ProcInt.` is mandated for the global namespace.

## 4. Enforcement and Compliance
Any new definitions or proofs introduced to the codebase or `ontology.ttl` that introduce a name not explicitly backed by mathematical literature, an explicit structural property, or the exact theorem signature will be treated as a bug and refused.
