# What We Refuse to Claim

Source: `docs/claims/WITHHELD_CLAIMS.md`

A project that makes strong claims about determinism, bounded computation, and
receipted proof needs an equally explicit record of what it does *not* claim.
Without one, the boundary between "proven" and "aspirational" erodes silently
as code grows. praxis keeps that boundary as an append-only register: claims
are added when a capability is scoped, and removed only by a receipted
retraction, never edited away.

The register below is reproduced faithfully from the source file.

---

## v26.7.3 (praxis-synthesis hook/firing/kernel work)

1. **Production trillion-agent control.** Scale tests exercise bounded
   synthetic cells; no claim is made about operating real fleets at any
   production scale.
2. **Complete moral interpretation.** The prayer kernel is a typed index
   from clause to problem class to a bounded support action. It does not
   interpret scripture, adjudicate morality, or claim theological
   completeness.
3. **God's hidden will.** The system models only the human-side workflow.
   `god-receives-unbounded` is a surrender marker, not a model; nothing is
   inferred about what is surrendered to.
4. **Unbounded future computation.** Everything is 8-bounded and
   epoch-clocked. Any problem that cannot be bounded is surrendered,
   quarantined, or refused — never "computed eventually."
5. **Full scripture-to-action automation.** Raw scripture is quarantined
   data (`tests/kernel_coverage.rs :: raw_scripture_is_quarantined_data_not_law`).
   Only pre-declared workflow fragments ground; no text-to-action pipeline
   exists or is claimed.
6. **LLM planning authority.** No LLM exists anywhere in the runtime
   (`tests/no_llm_runtime.rs`). An LLM is at most a quarantined proposer
   whose output faces the same admission gate as any other bytes.
7. **Solver optimality.** Solver8 derives a bounded plan satisfying the
   declared constraints; no optimality (shortest, cheapest, or otherwise)
   is claimed or tested.
8. **Foreign verification beyond the implemented stages.** The Python
   verifier re-derives exactly the stages listed in
   `docs/v26.7.3/RECEIPTS_REPLAY_VERIFY.md`; hook evaluation and
   plan/topology/geometry re-derivation are Rust-side only (named
   limitations, refolded-from-payload).
9. **Per-member external-event runtime bridge.** No file watcher, message
   consumer, or event daemon exists in praxis; deltas enter only when a
   caller presents a `MeaningSource` at the quarantine door. The bridge is
   roadmap, not code.
10. **Foreign re-derivation of the window-history commitment.** The firing
    receipt's `history_hash` commits to the first 7 history deltas' computed
    event hashes and is folded into the outer chain; the Rust replayer
    (`replay_firing`) refuses a mismatched history. The Python `firing`
    verifier folds `history_hash` as claimed (it takes no history input), so
    no claim is made that the foreign verifier authenticates history bytes.
