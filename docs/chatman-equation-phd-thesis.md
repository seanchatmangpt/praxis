# The Chatman Equation
## A Dissertation Prospectus

**Title:** *Authority Must Be Computed: The Chatman Equation A = μ(O) as a Theory of Trust in Context-Bounded Software Construction*

**One-line thesis:** Memory is fallible everywhere, so authority must live in what can be recomputed — context is not what you hold, it's what you can reconstruct.

---

## Abstract

Large language model agents now write a substantial fraction of new software, yet an agent's context window is small, lossy, and non-persistent. This dissertation argues that the binding constraint of agentic software development is not model capability but **context**, and that the correct response is an architectural law rather than a bigger window: correctness must be externalized into artifacts that are *cheaper to verify than to trust*. The law is formalized as the **Chatman Equation**, `A = μ(O)` — an artifact is the deterministic projection of an admitted ontology — extended to its live-system form `Fₙ₊₁ = μ(O*, C*, P*, T*, Fₙ)` and its three-pole generalization `A ≅ O ≅ L` (artifact ≅ ontology ≅ event log). The dissertation gives the equation an algebra (closure operators, delta groups, hash homomorphisms), a geometry (metric drift on canonical graph space), and a calculus (deltas as differentiation, replay as integration), then validates it empirically: a working projection engine built *by* context-bounded agents *under* the law it implements, whose receipted history caught every one of its own construction defects at a context boundary. The claim defended is that five independent systems converged on the same three-law immune system — closed vocabulary, evidence-earned status, chained tamper-evident history — because it is the unique fixed point of software construction under context loss.

---

## Chapter 0 — The Equation and Its Forms

**0.1 Classical form.** `A = μ(O)`: an artifact A is the image of an ontology O under a deterministic projection μ. Software is not authored; it is *admitted and projected*.

**0.2 Live form.** Because the filesystem already exists, the honest equation is a recurrence over live state:

```
Fₙ₊₁ = μ(O*, C*, P*, T*, Fₙ)        Rₙ₊₁ = receipt(Fₙ₊₁ − Fₙ)
```

where O* is the admitted graph, C* configuration wiring, P* locked packs, T* templates carrying intent, F the live filesystem, and R the decision-binding receipt. The star denotes *admitted* (gated, hashed, refused-by-name on drift) rather than merely present.

**0.3 The main law.** Idempotence is not a quality attribute; it is the defining law: `sync(sync(F)) = sync(F)`. A projection that is not a no-op on unchanged inputs is not a projection — it is an author with opinions.

**0.4 Post-Chatman generalization.** `A ≅ O ≅ L`: the runtime event log L (OCEL) is a third pole, isomorphic to artifact and ontology. Coherence is triple: what was declared, what was built, and what actually ran must be mutually reconstructible. (Prior art: `research/post_chatman_research.md`; this dissertation supplies the missing formal substrate and the empirical engine.)

---

## Chapter 1 — Context Loss Is the Adversary

**Claim.** The failures that motivate the equation are not malice or incompetence but *confident synthesis across a context gap*. No participant — human or model — ever holds full context; correctness that lives in any participant's memory is already lost.

**Evidence (all from the construction record, receipted):**

| Defect | Context boundary that produced it | What caught it |
|---|---|---|
| SPARQL `GROUP_CONCAT` nondeterminism silently swapping `--dry-run`/`--watch` (both `bool`, compiled green) | No single context held both aggregate-ordering semantics and the handler signature | Regeneration diff against a fixed handler contract |
| `write_lock` rewriting identical bytes → infinite watch loop while "idempotent" | Idempotence claimed in one context, falsifiable only in another (a running watcher) | Live watch smoke test, not review |
| Root workspace manifest shipped *committed-clobbered* by a prior session's `Overwrite` rule | Prior session's context ended before disk state was compared to intent | Receipt outputs list exonerating the current sync; git archaeology |
| `force: true` breaking second-sync idempotence in a freshly authored pack | Pack author's context lacked the write-decision precedence table | 96-cell combinatorial matrix finding, re-encountered and refused by the idempotence gate |

**Corollary (convergent evolution).** Five systems built without citing one another — ggen, praxis, wasm4pm, wasm4pm-compat, chicago-tdd-tools — independently precipitated the same three laws: (1) closed vocabulary refusing unknowns by name, (2) status earned by evidence and never asserted, (3) chained tamper-evident history. Identical selection pressure (an LLM that will claim success without doing the work) produced identical immune systems. This is the empirical anchor that the laws are forced, not stylistic.

---

## Chapter 2 — The Algebra, Geometry, and Calculus of μ

(Formal development; foundation in `docs/ggen-theory.md`.)

**2.1 Algebra.**
- Ontologies form a category **Ont** (objects: RDF graphs closed over a vocabulary V; morphisms: graph homomorphisms). μ's enrichment stage is a closure operator — extensive, monotone, idempotent (Kuratowski) — hence `μ₂(μ₂(O)) = μ₂(O)` is checkable, not aspirational.
- Deltas `Δ(O,O′) = (additions, removals)` form a group under composition with cancellation: `Δ ⊕ Δ⁻¹ = ∅`, inverse is an involution. (Implemented and property-tested: `Delta::{compose, inverse, is_empty}`.)
- The hash is a homomorphism from the quotient of graphs by blank-node relabeling and insertion order: `H = BLAKE3 ∘ canonicalize`. Well-definedness on the quotient is exactly the pair of tested laws *insertion-order invariance* and *blank-node-renaming invariance*.
- Chained receipts are a monoid homomorphism from histories to digests; tamper-detection soundness is injectivity-on-prefixes of that homomorphism.

**2.2 Geometry.** Canonical graph space carries the symmetric-difference metric `d(O,O′) = |O △ O′|`. "Drift" is distance from the last admitted point; the doctor's three checks (lock drift, orphaned artifacts, receipt-vs-disk staleness) are radius measurements in three projections of this space. Feasible regions are SHACL/ASK-restricted subspaces; admission is membership testing.

**2.3 Calculus.** The delta is discrete differentiation of ontology history; replay is integration: `Oₙ = O₀ ⊕ Σ Δᵢ`, verified term-by-term against the receipt chain (`record[i].chain_hash = record[i+1].prev_chain_hash`, genesis-rooted). Incremental regeneration is a directional derivative of μ; the chain rule across pipeline stages μ₁…μ₅ bounds what a change in O can touch in A.

**2.4 Why each engineering rule is a theorem, not a taste.** No wall-clock in hashed material (else H is not a function of its inputs); ORDER BY in every projection SELECT (else μ is a relation, not a function — the flag-swap incident is the counterexample that proves necessity); fail-closed refusal (else the feasible region is open and admission is vacuous).

---

## Chapter 3 — The Ontology as Compressed, Verifiable Context

**Claim.** `A = μ(O)` is a context-management theorem. The graph is the unique artifact that is simultaneously (a) small enough to fit in any context window, (b) canonical enough to hash, and (c) expressive enough to regenerate everything downstream. Externalizing authority into O converts "shared memory," which no agent has, into "shared fixed point," which every agent can reconstruct.

**3.1 The dogfood loop as context reconstruction.** The engine's own CLI routes are projections of `schema/praxis.ttl` `CliCommand` individuals; regenerating them from a cold start reproduces the binary's surface byte-identically. A session that loses all context can recover the system's intent from O + T + receipts alone. The construction of the engine survived a mid-session context compaction for precisely this reason: the state that mattered was outside the window.

**3.2 Packs as portable context.** A pack (ontology + templates, content-hashed, locked) is compressed integration context that any consumer project can admit. Empirical result: eight framework packs (clap-noun-verb, chicago-tdd-tools, star-toml, lsp-max, praxis-core, wasm4pm-compat, wasm4pm-algorithms, wasm4pm-cognition), all 28 pairwise compositions and the 8-pack union proven collision-free and idempotent; corruption of any one pack refused by name (FM-PACK-008) before any write. The 2027 distribution thesis is this chapter operationalized.

**3.3 The gated-narrator corollary.** The wasm4pm doctrine "Old AI is the factory, LLMs are the brochure" is enforced, not aspirational: `AuthorityKind ∈ {HumanProse, LlmProjection, MachineEvidence}` and only the third is admissible evidence. The dissertation's own construction is an instance — every agent claim in the build was gated by an independent cold verifier spawning real binaries.

---

## Chapter 4 — Empirical Validation: An Engine Built Under Its Own Law

**Method.** Build a complete projection engine (graph, template, write, sync, pack, lint, receipt-chain, doctor, watch — CalVer 26.7.4, `praxis/crates/ggen`) using only context-bounded agents, gating every phase with falsifier sets, and treat every construction defect as data.

**Results (all receipted in commits f011738 → 71699f8 → 314db73):**
- 182 tests, zero mocks (Chicago discipline: real filesystems, real oxigraph, real subprocesses).
- 96-cell exhaustive write-decision matrix: implementation matches the documented decision table in every cell; the one spec-vs-expectation tension (force-before-identical precedence) was surfaced as a finding, then independently re-encountered as a live defect in a freshly authored pack — the matrix predicted the wild bug.
- Property-tested laws: sync idempotence at byte and receipt-payload level; hash invariance under insertion order and blank-node renaming; delta group cancellation.
- Adversarial results: tampering any receipt field or any middle history record fails verification at the named index; corrupting any pack post-lock refuses by name; a template consuming an unprojected variable is refused before render (FM-TPL-003).
- The falsifier discipline mattered: of the four real bugs found, zero were found by reading code and four by refusing to let a report stand in for disk state.

**Interpretation.** The engine is simultaneously the theory's implementation and its experiment: a system whose construction process exhibits the failure mode the theory names, and whose mechanisms catch exactly those failures, is evidence that the law is load-bearing.

---

## Chapter 5 — The Residue Defines the Frontier

What cannot yet be externalized into verifiable form marks the open problems:

**5.1 Ontology governance (2028).** Everything downstream of O is receipted; assertion *into* O is still ungated. Program: closed predicate vocabularies enforced at validate time; asserted-vs-derived SHACL split (status properties only via CONSTRUCT from evidence); ontology deltas receipted with the existing Delta algebra — who changed the graph, from-hash to-hash, chained.

**5.2 Cross-pack receipt federation (2029).** `ggen.lock` binds content hashes, not receipt-chain heads; a producer can regenerate a pack from a corrupted upstream ontology and ship a valid content hash. Program: transitive attestation — consumer receipts bind producer chain heads, chains of chains down the supply graph.

**5.3 The intent residue (open).** The handlers layer and the human intent behind a triple resist compression into verifiable form. The generated/hand byte ratio is proposed as a first-class receipt field — a longitudinal measurement of how far the frontier moves. Where context resists externalization, trust remains social; the research program is the controlled shrinking of that region, never the pretense that it is empty.

---

## Chapter 6 — Related Work and the Defense Question

*"Isn't this formal methods rebranded?"* No. Formal methods assume a verifier with full context and pay proof-cost for total guarantees. The Chatman Equation assumes **no participant ever has full context — including the humans and the verifier** — and asks what artifacts make correctness survivable anyway. The answer (canonical graphs, closed vocabularies, decision-binding chained receipts, projection idempotence) costs O(hash) to check, was arrived at by convergent exhaustion across five codebases rather than derivation, and degrades gracefully: an unverified region is named and fenced (the residue), not assumed away.

Adjacent literatures to position against: reproducible builds and supply-chain attestation (in-toto, SLSA) — which receipt *builds* but not *decisions*; model-driven engineering — which projects from models but without admission gates or tamper-evident history, and which historically drowned in exactly the drift this equation refuses; process mining (van der Aalst) — which supplies the L pole and the conformance mathematics the three-pole form requires.

---

## Contributions

1. **Formal:** the Chatman Equation with its algebra/geometry/calculus, including proofs-by-construction that each engineering invariant (no wall-clock in hashes, ordered projections, fail-closed admission) is necessary for μ to be a function.
2. **Systems:** a complete, self-hosting projection engine satisfying the laws, with an 8-pack composition proof and a falsifier-driven verification methodology.
3. **Empirical:** the convergent-evolution observation across five independent systems, and the defect corpus tracing every construction failure to a context boundary.
4. **Programmatic:** the residue map — ontology governance, receipt federation, intent ratio — as the ordered research frontier for 2028–2030.

---

*Grounding artifacts:* `praxis/crates/ggen` (engine, 182 tests), commits `f011738`/`71699f8`/`314db73`, `docs/ggen-theory.md` (formal substrate), `docs/ggen-port-evaluation.md` (the audit that motivated the rebuild), `docs/vision-2030-prd-adr.md` (product form of the claim), `research/post_chatman_research.md` (three-pole prior art), `.ggen-v2` receipt chains and `.ggen/receipts/` dogfood chains (the evidence itself).
