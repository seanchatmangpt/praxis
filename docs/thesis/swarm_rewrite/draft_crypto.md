# Cryptography Agent - Receipt Integrity (Role 2.8)

## Structured Notes

1. **The Frame (`OcelCausalFrame`)**
   - **Definition:** A fixed-width, 128-byte, cache-line-aligned `repr(C, align(64))` structure that encodes a single manufacturing step.
   - **Serialization:** The hash body $\body(\fr)$ is a 99-byte canonical serialization that commits specific fields without padding: `instruction_id`, `fired_mask`, `denial` (admission verdict), `obj_refs`, `ts_ns`, `activity_idx`, `node_kind`, and `prior_hash`.
   - **Importance:** The fixed-width and unambiguous length-determined serialization ensure that any two distinct histories map to distinct bytes before hashing.

2. **Payload Hash (Commitment via Object References)**
   - **Mechanism:** The frame lacks a dedicated "payload" field. Instead, it repurposes the eight `obj_refs[0:8]` 32-bit words to carry the 256-bit BLAKE3 digest ($\dg(p)$) of the canonical JSON bytes of the admitted payload.
   - **Property:** The raw tuple constructor `PackedObjRef(w)` preserves all 32 bits intact, ensuring the full 256-bit digest is committed into bytes [24, 56) of the hash body. The frame thus binds exactly to the payload's content, not an opaque handle.

3. **The Chain Step**
   - **Chain Rule:** $h_{+} = \chainH(h_{-} \concat \body(\fr))$
   - **Definition:** A rolling BLAKE3 commitment that folds each frame's hash body onto the preceding chain commitment ($h_{-}$), initialized from a genesis value $\Genesis = \chainH(\bm 0_{32})$.
   - **Determinism:** The chain is a pure left fold computed at a single construction site (`build_admission_frame` / `chain_from_frame`), guaranteeing no drift between emission (minting) and replay (recomputation).

4. **Collision-Resistance Assumptions**
   - **Axiom:** $\chainH$ (BLAKE3) is collision-resistant. A probabilistic polynomial-time adversary can find a collision only with negligible probability $\varepsilon(256)$ (security bound $\sim 2^{128}$).
   - **Faithful Chain Theorem:** Any change to any committed field of any frame in a ledger $\Ledger$ that yields the same terminal commitment $h_n$ requires exhibiting a hash collision.

5. **What a Receipt Does NOT Prove (Integrity-Virtue Scope Theorem)**
   - A receipt proves integrity, authenticity (if signed), and conformance-as-checked, but explicitly does NOT prove:
     - **Obligation Adequacy:** It does not prove the chosen obligation battery ($G$) was the *right* set of checks, only that they were evaluated.
     - **World-Fitness:** It does not prove the artifact is safe, correct, useful, or ethical in the world. Integrity is not virtue.
     - **Semantic Truth:** It does not prove the admitted observation actually *meant* what it purported to mean; it only proves that it passed the admission gate (a retraction, not a decision on truth).

---

## Chapter Draft: Receipt Integrity and Cryptographic Consequences

### 1. The Frame and Payload Hash
The foundational unit of receipt cryptography is the **frame**. In our implementation, the `OcelCausalFrame` is constructed as a fixed-width, 128-byte, cache-line-aligned struct. This mechanistic constraint is vital because unambiguous, length-determined serialization prevents ambiguities during hashing. The fixed layout includes fields capturing the admission verdict (`denial`), timestamp (`ts_ns`), step sequence (`instruction_id`), and process structure indicators. 

Crucially, the frame commits the payload through a purposeful repurposing of its object-reference slots. Rather than allocating a new field, the eight 32-bit words of `obj_refs` capture the full 256-bit BLAKE3 digest of the payload. When serialized to its 99-byte canonical hash body $\body(\fr)$ (excluding memory padding), the frame indelibly binds the step's consequence directly to the exact byte-content of the payload.

### 2. The Chain Step
A receipt is not a copy of an execution but a **commitment** to it. The timeline of an execution is folded into a ledger via the chain rule: 
$$h_{+} = \chainH(h_{-} \concat \body(\fr))$$
This rolling commitment mixes the $32$-byte predecessor hash with the $99$-byte frame body. Starting from a genesis state, every appended frame mathematically anchors the entire prior history into its successor. Because the fold is deterministic and implemented at a singular construction site, any discrepancy during re-verification isolably indicates a tamper rather than logic drift. 

### 3. Collision-Resistance Assumptions and Faithfulness
This architecture rests on the axiom of collision resistance for the underlying hash function (BLAKE3). Under this assumption, an adversary cannot find two distinct inputs yielding the same digest without performing an infeasible amount of computation. 

This axiom supports the **Faithful Chain Theorem**: if a bounded verifier computes the terminal commitment $h_n$ and it matches the stored or signed terminal hash, the history has not been altered. Mutating any committed field—such as swapping a payload, altering a timestamp, or faking an admission verdict—would inevitably perturb $h_n$ unless the adversary breaks the collision resistance. The cost for the verifier remains bounded at $O(1)$ per recomputed link.

### 4. The Boundary of Cryptography: What a Receipt Does NOT Prove
A cryptographic apparatus enforces the integrity of a ledger, but it is a critical fallacy to conflate cryptographic integrity with inherent goodness. The Integrity-Virtue Scope Theorem mandates that we acknowledge what a receipt cannot prove:
1. **It does not prove obligation adequacy:** The receipt records which checks passed, but it does not warrant that those checks were sufficient or correct for the task.
2. **It does not prove world-fitness:** The fact that a deterministic process ran unaltered does not make the resulting artifact benign, safe, or ethical.
3. **It does not prove semantic truth:** A receipt proves an observation passed the admission gate; it does not prove the observation was semantically truthful.

In sum, a receipt commits to a history. It does not absolve the consequences of that history.
