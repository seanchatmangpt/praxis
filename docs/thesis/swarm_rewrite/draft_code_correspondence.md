# Appendix: Code Correspondence

This appendix anchors the abstract mathematical objects and theorems presented throughout the thesis to their concrete implementations in the `praxis` codebase. It serves as a verification guide, demonstrating how the theoretical constructs are strictly enforced by the Rust type system and cryptographic protocols.

## 1. The Bounded Receipted Chatman Equation (BRCE B1: $A = \mu(O^*)$)
**Reference:** Part 0, `def:brce`
**Implementation:** `crates/praxis-core/src/law.rs` and `crates/praxis-core/src/lifecycle.rs`

The foundational equation $A = \mu(O^*)$ separates raw observations ($O$) from admitted observations ($O^*$). This is enforced through the **Fused Law Object** (`LawObject<Payload, S: Stage, Law>`) and its typestate lifecycle:

- **Mathematical Object ($O \to O^*$):** The transition from raw observations to admitted ones is managed by the `Judge` and `Admit` traits. 
- **Enforcement:** The typestates defined in `lifecycle.rs` (`Raw`, `Validated`, `Admitted`, `Receipted`) seal the state transitions at compile-time. A payload cannot reach the `Admitted` state (becoming an element of $O^*$) without satisfying the required `Obligation` battery via the `Judge::judge` method. The type system ensures that manufacture ($\mu$) or receipt generation can only happen on `LawObject<Payload, Admitted, Law>`. Illegal transitions are uninhabited by definition.

## 2. Admission Monoid and Refusal Algebra
**Reference:** Part I
**Implementation:** `crates/praxis-core/src/refusal.rs`

The algebra of admission treats refusals as first-class outputs rather than exceptions, allowing the combination of multiple obligation checks into a single verdict.

- **Mathematical Object (Admission Monoid):** The set of refusal taxonomy categories (`RefusalCategory`) and concrete instances (`RefusalScenario`) forms the domain. 
- **Enforcement:** The `compose_denials` function implements the monoidal composition law, safely folding a set of `RefusalScenario` items into a single composed `DenialPolarity` word. The mapping is total and exhaustive; every `Obligation` failure is strictly mapped to its category and lane.

## 3. Faithful Projection Theorem and Receipt Cryptography
**Reference:** Part IV (`thm:faithful`) and Part II
**Implementation:** `crates/praxis-core/src/receipt_record.rs`, `crates/praxis-core/src/law.rs`, and `crates/praxis-core/src/signing.rs`

The requirement that every admitted observation produces a deterministic, tamper-evident cryptographic receipt (Receipt Totality B3).

- **Mathematical Object (Faithful Projection & Collision Resistance):** The mathematical commitment connecting an admitted payload to its causal chain.
- **Enforcement:** The `ReceiptRecord` struct persists a snapshot of the execution. The `recompute_chain_hash` method re-evaluates the hash using `build_admission_frame` and `chain_from_frame`, independently of the live object. Because the hash is computed as `BLAKE3(prev_chain_hash || ocel_frame_bytes)`, a single byte tamper is caught immediately. The system is fail-closed, ensuring that the receipt accurately projects the interior state without exposing the entire payload.

## 4. Marking Polytope and Conformance Geometry
**Reference:** Part III (`def:polytope`, `thm:farkas`)
**Implementation:** `crates/praxis-core/src/replay_adapter.rs`

The conformance invariant (B4) defines system states geometrically using the Petri net state equation $m = m_0 + N \cdot x$.

- **Mathematical Object (Token Geometry & Bounded Grounding):** The tracking of token states as execution proceeds, asserting that paths out of bounds are separated by a Farkas certificate.
- **Enforcement:** The `replay_adapter.rs` module maps the `judge -> admit -> receipt` lifecycle to a fixed 3-node SEQ POWL token model. It defines token transitions (`TOK_START`, `TOK_JUDGED`, `TOK_ADMITTED`, `TOK_DONE`). Using `PowlReplayVerifier`, `replay_lifecycle` enforces that token requirements are strictly met at each step. A lawful sequence guarantees a Q16.16 fitness of `1.0` (`0x0001_0000`), while any out-of-order execution emits a `ReplayViolation::TokenNotEnabled`.

## 5. Rice Quarantine (Boundary Schema)
**Reference:** Part 0 (`thm:rice`)
**Implementation:** `crates/praxis-core/src/quarantine.rs`

Undecidable observations must be refused at the boundary before they can enter the evaluation logic.

- **Mathematical Object (Decidable Boundary):** The projection of untrusted $O$ into a bounded, verifiable space.
- **Enforcement:** The `RiceQuarantine` wraps a `BoundarySchema` (e.g., `JsonBoundarySchema`). Before a `LawObject` can even be instantiated, raw strings must successfully pass `RiceQuarantine::admit`, which validates them against expected schemas and custom predicates. Any failure results in an immediate rejection (`QuarantineError`), preventing malformed or unbounded inputs from penetrating the core admission logic.

---
**Status:** Verification of these mappings in the `crates/praxis-core/src/` tree confirms that the type definitions, cryptographic operations, and protocol steps precisely enforce the theoretical bounds outlined in the main thesis parts.
