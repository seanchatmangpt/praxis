# Ticket: Bound the Admitted-Graph-to-Capability Projection (HDIT-as-Field-Layer Claim)

## Title
Scope and document the HDIT "semantic field compression" claim against `ground.rs` (PROJ-201)

## Description — CORRECTED 2026-07-03 (see index.md's "Corrections" section)
The original version of this ticket refused HDIT's compression claim on a false dichotomy:
"praxis uses exact decidable closure, HDIT claims approximate compression, therefore they are
unrelated and HDIT is withheld." That dichotomy does not survive scrutiny. Compression, in the
general sense that matters here, is any transform reducing an unbounded search/reference space
to a bounded one under some preserved guarantee. Lossy embeddings (PCA, hashing tricks) are ONE
species of that genus, trading exactness for size. `ground::restrict_to_fragment`'s exact
edge-closure over a workflow fragment's own `wf:capability`/`wf:constraint` membership edges
(`crates/praxis-synthesis/src/ground.rs`) is a DIFFERENT species of the same genus: it
compresses an unbounded admitted graph down to a bounded, addressable fragment, while
preserving exact reachability instead of trading it away. The original ticket dismissed HDIT
by conflating "not the same species" with "not the same genus, therefore inapplicable" — the
same vocabulary-triggered dismissal pattern later identified in this session's "agent"
discussion, here applied to "compression" and "physics."

The corrected framing: HDIT's claim (bounded, admissible capability coordinates emerge from a
large semantic space) is STRUCTURALLY TRUE of `ground.rs` already — it just uses the
exact-closure species of compression rather than the lossy-embedding species. No lossy
embedding scheme is being proposed or needed; the existing exact closure already satisfies the
genus-level claim with a strictly stronger guarantee (provable reachability vs. approximate
similarity).

## Acceptance Criteria — CORRECTED
- A doc entry (not a withheld-claims entry — this is now a CLOSED claim, not a withheld one)
  states plainly: `ground::restrict_to_fragment` implements the exact-closure species of
  bounded-graph compression; the lossy-embedding species (PCA/hashing/dimensionality reduction)
  is explicitly not needed and not planned, because the exact species already delivers the
  genus-level guarantee (bounded, addressable capability coordinates) with a strictly stronger
  correctness property.
- No new code, no new dependency, no new vocabulary (e.g. no "capability field" identifiers)
  introduced into `crates/praxis-synthesis/src/` — the correction is entirely about how this
  ticket CHARACTERIZES existing code, not about writing new code.
- Existing `restrict_to_fragment` tests continue to pass unchanged.
- If, during the audit, a genuine case is found where the existing exact closure scales poorly
  (e.g. pathological triple counts near `MAX_TRIPLES`), that becomes its own follow-up ticket
  with a concrete benchmark, at which point an approximate (lossy) compression species could
  legitimately be considered as a fallback tier — not folded into this one.

## Dependencies
None.

## Verification Mechanism
1. `cargo test -p praxis-synthesis` — full suite green, no behavior change.
2. `grep -n "HDIT-style" docs/claims/WITHHELD_CLAIMS.md` — new entry present.
3. Manual read of `crates/praxis-synthesis/src/ground.rs::restrict_to_fragment` confirming the
   closure remains exact-edge-based (no approximate/embedding-based logic introduced).
