# The Typed Refusal Algebra: Sealing Execution Entropy

## 1. Introduction

In the Praxis architecture, execution entropy—the proliferation of unhandled edge cases, silent failures, and non-deterministic states—is sealed at the boundary of the type system. Central to this discipline is the **Typed Refusal Algebra**, implemented via the `CngRefusal` enum in the `cng` crate. Rather than relying on unstructured error strings or silent fallbacks (such as `.unwrap_or_default()`), Praxis formalizes every failure mode as a distinct, typed variant. This chapter details the foundational refusal codes (`CNG_R01` through `CNG_R11`), explaining how Rust's strict compiler guarantees enforce structural determinism, prevent unreceipted actuation, and halt non-deterministic execution before it propagates.

## 2. Core Refusal Codes (`CNG_R01` to `CNG_R11`)

The refusal codes in the `cng` crate serve as an impenetrable set of gates. Each code corresponds to a specific, irreducible failure in data integrity, logical solvability, or determinism.

### 2.1 Data and Structural Integrity
* **`CNG_R01 MalformedTtl`**: Refuses input artifacts that are not valid RDF/Turtle, or contain unparseable PDDL literals.
* **`CNG_R02 MissingDomain` & `CNG_R03 MissingProblem`**: Halts execution when the required PDDL domain or problem fragments are absent from the admitted set.
* **`CNG_R06 InvalidPowl`**: Rejects POWL graphs that fail structural parsing or shape validation, ensuring only well-formed procedural artifacts enter the pipeline.

### 2.2 Execution and Logical Solvability
* **`CNG_R04 PlanUnsolvable`**: Emitted when the merged planning surface admits no valid plan (e.g., unreachable goals, empty tapes).
* **`CNG_R05 UnsupportedConstruct`**: Prevents the use of unsupported operations such as mismatched domain names or branching POWL constructs.
* **`CNG_R07 RunnerMismatch`**: Refuses execution if the runner’s output does not strictly conform to the projected procedural order.

### 2.3 Determinism and Evidence Integrity
* **`CNG_R08 Nondeterminism`**: A critical safeguard that halts the pipeline if repeated manufacture produces byte-divergent outputs. Under a fixed seed, execution must yield identical receipts; `CNG_R08` traps entropy at runtime.
* **`CNG_R09 HardcodingSuspicion`**: Detects and refuses outputs that do not reflect the admitted plan, preventing detached or canned execution.
* **`CNG_R10 IoRefused`**: Explicitly types filesystem IO refusals.
* **`CNG_R11 AuditMismatch`**: Triggers when an independent audit replay produces a digest diverging from the recorded evidence, exposing third-party integrity failures.

## 3. Rust's Compiler Guarantees and Pattern Matching

Praxis leverages Rust's algebraic data types (`enum`) and exhaustive pattern matching (`match`) to enforce these invariants at compile time. Infallible operations are guaranteed by the type system, while fallible operations return a `Result<T, CngRefusal>`. 

Because Rust requires exhaustive pattern matching, it is structurally impossible for a developer to forget an error state. Praxis strictly bans silent error swallows such as `.unwrap()`, `.expect()`, `.ok()`, or `.unwrap_or_default()`. If a function might encounter `CNG_R08 Nondeterminism`, the compiler forces the caller to explicitly handle this variant. Consequently, execution entropy cannot leak across module boundaries; it is sealed by the compiler.

## 4. Enforcing Zero Unreceipted Actuation

The Typed Refusal Algebra naturally extends to system-level laws, most notably the **Zero Unreceipted Actuation** law (`actuate(c) ⟹ ∃R (R⊢c)`). In Praxis, every executed workflow transition must produce exactly one graphlaw `HookReceipt`. 

If an actuation occurs without a valid receipt, the system emits a refusal (such as `CNG_R13 UnreceiptedActuation`). Through the type system, this state is never downgraded to a warning. Because the refusal algebra maps such violations to strict enum variants, the broker must return a typed error. The strict Rust compiler ensures that unreceipted actuations cannot be bypassed, ensuring every state transition is fully accounted for in the canonical N-Quads evidence graph.

## 5. Halting Non-Deterministic Execution

Praxis operates under the invariant that identical inputs must produce byte-identical receipts without algorithmic surprises. The typed refusal algebra, specifically codes like `CNG_R08` (Nondeterminism) and `CNG_R11` (AuditMismatch), actively enforces this at runtime.

By eliminating wall clocks from hash paths and forbidding randomness, the system design aims for pure determinism. However, if environmental entropy or logic bugs introduce non-determinism, the `cng` pipeline detects the byte-divergence during manufacture or audit replay. Instead of silently carrying the corrupted state forward, the pipeline maps the divergence to a `CngRefusal` variant. Combined with Rust's pattern matching, this guarantees that non-deterministic execution immediately halts the pipeline, preserving the integrity of the overall system and honoring the strict AGI-level Rust core-team discipline.
