# Bootstrap, Cold-Start, and First/Last-Mile Limitations of Multifractal Workflow

Version: v26.7.13. Companion to `docs/releases/v26.7.13/THESIS.md` and its Chapter 28
("Limitations and Open Theorems"). Where this document names a limitation Chapter 28 already
states formally, it cites the section instead of restating it. Where it names a limitation not
yet in Chapter 28, that is disclosed explicitly, not implied to be covered.

## Why this document exists

MFW's guarantees (deterministic replay, receipted completion, admission-gated mutation) all
presuppose something is already admitted, shaped, and governed. Every one of those presuppositions
was itself produced by an act that the guarantees do not cover: a human or an LLM chose a
vocabulary, wrote a shape, authored a planning domain, or interpreted raw text into RDF, and none
of those acts happened inside the deterministic, receipted spine they made possible. This is not a
missing feature to be implemented later — bootstrapping a system that enforces invariants on
admitted state always requires at least one ungoverned step to produce the first admitted state.
The question worth tracking is not "can this be eliminated" but "is each such step disclosed,
bounded, and checked after the fact, or silently assumed."

## Quick reference

1. Ontology/vocabulary selection — who picks the right public term, and how is the pick checked
2. SHACL shape authorship — the single highest-leverage undeclared judgment call in the pipeline
3. PDDL8 domain authorship — planning models of a new problem class start unverified
4. Raw observation to candidate RDF — the literal first mile, interpretation with no precedent
5. External actuator trust on first use — no baseline for differential verification yet
6. Receipt-chain genesis — the chain proves tamper evidence, not truth, starting from an assumed header
7. Permission root-of-trust — who authorizes the first authority; ODRL policy artifact still empty
8. Namespace/identity allocation — collision is discovered, not prevented, today
9. Last mile: receipted output re-entering non-deterministic interpretation
10. No ambient correspondence across systems, including the human reading the output
11. Residue synthesis for a genuinely novel failure — no programmatic ResidueState synthesizer
12. The measurement rail's own cold start — empirical multifractality needs history that a new
    deployment does not have
13. Self-referential limit — the process that builds MFW is not itself governed by MFW

## I. First mile: getting fresh reality into the system

### 1. Ontology/vocabulary selection

Choosing FIBO over inventing a private M&A predicate, or ODRL over a bespoke permission vocabulary,
is a semantic-fit judgment — not a syntactic one, so no SHACL shape can validate it a priori.
Chapter 28.8 ("Ontology quality") states this precisely: "Public vocabulary reduces private
lock-in but does not guarantee correct modeling. Terms can be misused, source ontologies can
overlap, and institutional distinctions can exceed existing public terms... it does not eliminate
semantic review." Today that review is a human or an LLM doing ad hoc research (WebSearch, reading
spec documents) — unreceipted, unadmitted, and outside the plan-digest-bound permission apparatus
that governs every other mutation. Vendoring the resulting ontology file into the repository is a
substrate change with no admission gate of its own; it happens by ordinary commit, not by the
actuation discipline the substrate then enforces on everything else.

### 2. SHACL shape authorship

Every domain pack (`dogfood-lifecycle-pack`, `soc2-audit-pack`, `dry-run-publish-pack`,
`togaf-adm-pack`) required a hand- or LLM-authored shape file before any admission could happen in
that domain at all. A wrong or overly permissive shape does not fail loudly — it lets a bad
actuation through deterministically and reproducibly, every time, which is a worse failure mode
than an obviously broken one because replay will faithfully reconstruct the same mistake. There is
no meta-shape validating whether a shape is correct for its domain; correctness is established only
empirically, after the fact, by adversarial fixtures (the pattern `soc2-audit-pack` uses) — and for
a first-of-its-kind domain, no such fixtures exist yet either.

### 3. PDDL8 domain authorship

Chapter 28.2 ("Planning complexity") and 28.3 ("Residual-goal extraction") both bear on this.
Authoring the actions, preconditions, and effects for a new problem class cannot be verified against
the real world it claims to model except empirically — whether the resulting plans are sensible,
whether execution matches intent. Chapter 28.3 names the sharpest instance directly: "An LLM can
propose the goal; the system still needs bounded validation that the proposal corresponds to real
residue and does not widen scope" — an explicit admission that the LLM's proposal is not
self-validating, and that the validating step is a separate, currently under-specified obligation.

### 4. Raw observation to candidate RDF

The literal first mile: turning an arbitrary email, document, or utterance into candidate RDF
(`THESIS.md` Appendix J, §J.2 → §J.3) is an interpretive act. For an established domain this
becomes routine once a pattern exists; for a domain encountering its first case, no such pattern
exists yet, and inventing the mapping is unreceipted judgment by construction — the deterministic
spine cannot begin until this step has already happened once, off the books.

## II. Bootstrap and root-of-trust

### 5. External actuator trust on first use

The `ExternalCutCompiler` trait lets a new actuator implementation be plugged in, and Chapter
12.11 ("Differential verification") is the mechanism that detects drift between an actuator's
claimed and actual behavior. Differential verification requires a baseline of prior good behavior
to differ against. A brand-new actuator's first invocation has no baseline — trust in it, at that
moment, rests on code review and test coverage, the same unreceipted category of evidence as
everything else in this document, not on the differential mechanism the architecture otherwise
relies on.

### 6. Receipt-chain genesis

Chapter 13.3 ("Hash chaining") defines $h_0 = \text{Hash}(\text{header})$ and states plainly: "A
chain proves tamper evidence, not event truth." The chain can prove nothing changed after $h_0$
was fixed; it cannot prove $h_0$ itself, or the admission pipeline that produced the events feeding
the chain, were trustworthy at genesis. This is the same structural limit every hash-chained
system has (the blockchain genesis-block problem in different clothing) — worth naming here
specifically because "receipted" is otherwise used in this project as if it settles the question
of trust rather than relocating it to one earlier point.

### 7. Permission root-of-trust

Plan-bound permission (Chapter 11) requires an authority to grant it. Nothing in the current
architecture answers who authorizes the first authority. This is not abstract: the ODRL policy
artifact is, per this session's own direct check, still an empty, quarantined vocabulary slot (`grep
-rl odrl: crates/ packs/` returns zero files) — the permission system's own bootstrap is the
concrete, currently-open instance of this limitation, not a hypothetical one.

### 8. Namespace/identity allocation

`THESIS.md` §26.3 reports 33 remaining orphaned files from the mfact formalization rail, 28 of
which are namespace collisions, and states plainly that "namespace collisions are load-bearing
because ambiguous identity corrupts claim and artifact ledgers." There is no enforced allocation
mechanism preventing a new domain pack from colliding with an existing namespace choice — collision
is discovered after the fact, the same pattern as shape and ontology correctness above.

## III. Last mile: getting output back out into a non-deterministic world

### 9. Receipted output re-entering non-deterministic interpretation

A receipt proves what MFW did. It proves nothing about how the human, counterparty, or regulator
who receives the output will read, act on, or respond to it — the same asymmetry as the "first
mile," mirrored on exit. This is the direct consequence of the OTP/Little's-Law framing discussed
earlier in this session: MFW governs up to the admission boundary in both directions; the moment
output crosses back out to an external, unpredictable party, MFW's guarantees stop applying to what
happens next, and there is no receipt for the recipient's own behavior.

### 10. No ambient correspondence

Chapter 12.3 states the general principle this instantiates: "Two artifacts do not correspond
merely because both were generated from RDF... Correspondence is always parameterized. There is no
global relation same source ⇒ same semantics." Stated for cross-engine correspondence
(Rust/Erlang/WASM/AtomVM) in the thesis, but the same limit applies to the human reading a
compliance report or the counterparty receiving a contract: shared provenance does not imply shared
interpretation, and no bridge in this architecture claims to close that particular gap.

## IV. Recursive and self-referential bootstrap

### 11. Residue synthesis for a genuinely novel failure

No command-failure-to-`ResidueState` synthesizer exists; residues used by the recursive-repair
machinery today are hand-authored Turtle fixtures, not derived programmatically from a real
failure. A failure mode not anticipated by an existing fixture is, structurally, a first-mile
problem recurring inside the repair loop: encountering it for the first time requires the same
unreceipted interpretation step as encountering a new domain does.

### 12. The measurement rail's own cold start

Chapter 28.11 ("Empirical multifractality") names this directly: "Finite workflow logs,
nonstationarity, hierarchy construction, heavy tails, and short scale ranges can create false
spectra. The multifractal claim remains empirical until robust replicated evidence exists." A new
deployment has no execution history at all — the statistical rail (Chapters 17-20) that is
supposed to characterize the non-deterministic residue empirically cannot say anything meaningful
until the system has already been running long enough to generate the history it needs, which is
itself a maturity/bootstrap condition, not a capability available at cold start.

### 13. Self-referential limit

The most concrete instance in this document. The process that built MFW's own admission, SHACL, and
planning machinery — this repository's 8-hour autonomous self-improvement loop and the sessions
around it — was not itself running under MFW's own governance: no plan-digest-bound permission, no
admitted-RDF-observation trail, no receipted actuation for the code changes that built the trust
apparatus, beyond ordinary git commits. This is the same self-report/self-verification-sharing-an-
author problem named earlier in this session's conversation, applied to MFW's own construction
rather than to a conversational claim: the tool that will eventually certify other systems'
determinism was not, in its own construction, certified by anything but the humans and LLMs writing
it. Nothing in this document resolves that; naming it precisely is the point.

## V. Gaps found by adversarial review

A dedicated completeness pass against this document (not by its original author) found seven
further structural gaps not covered by items 1-13. They are listed here rather than folded into
the numbering above so the provenance stays visible: these were found by checking what a thorough
architect would ask that the first draft didn't, the same discipline this document asks of the
rest of the project.

### 14. Multi-tenant / multi-org first-merge bootstrap

When two organizations' independently-admitted RDF graphs meet for the first time — a shared
supply-chain workflow, an M&A due-diligence corpus, cross-org SOC2 evidence exchange — each graph
was validated under its own shapes, its own permission root-of-trust (item 7), and its own receipt
chain with its own $h_0$ (item 6). The merge event itself has no admission gate: whose shape wins on
conflict, whose authority is superior, how the two chains relate, are all decided once, informally,
at first contact. Distinct from item 8 (collision within one deployment) and item 7 (a single
authority's own bootstrap) — this is reconciling two already-governed graphs, a joining problem
neither covers.

### 15. Clock / time source-of-truth for the first OWL-Time literal

No wall clock in hash/receipt paths (a hard invariant) forces every in-graph timestamp to come from
an admitted OWL-Time literal — but something outside the deterministic spine had to read a real
clock once to produce that literal's value. Nothing names which clock source is authoritative,
bounds skew between actuators, or detects a forged or stale timestamp being admitted as fact.
Distinct from item 6, which is about the hash header's trustworthiness, not the semantic content of
time literals entering the graph before they reach that header.

### 16. Cryptographic key / signer identity bootstrap

Appendix K.5 ("Receipt without occurrence truth") states plainly: "A compromised broker signs a
receipt for a command it never sent. The receipt authenticates the broker's assertion, not physical
truth" — which presupposes a signing key exists. No section addresses how a verifier first obtains
and trusts an actuator's or broker's key, how rotation is handled without breaking chain
verifiability, or what follows if the first key is compromised at issuance. Distinct from item 5
(behavioral drift with no differential baseline) and item 6 (unkeyed content-hash integrity) —
authenticity (who signed it) is a third trust primitive neither covers.

### 17. Shape/ontology version migration after real data already conforms

Chapter 28.7 states algorithm or profile changes "must name the exact profile and preserve
migration proofs," implying shapes and ontologies are expected to change over a deployment's life —
but not what happens at the concrete moment facts admitted under shape v1 violate a corrected shape
v2. Re-validate the whole historical graph, grandfather old facts under a dual-shape window, or
accept that receipted history and current lawfulness silently diverge — no answer exists, and
whichever is chosen first sets an unreviewed precedent. Distinct from item 2, which is silent on
what happens after a shape is later found wrong.

### 18. Cross-engine toolchain / reproducible-build trust bootstrap

Rust, Erlang/BEAM, WASM, and AtomVM each need their own compiler or runtime to produce the binaries
that execute admitted workflows. Nothing attests to, pins, or verifies the provenance of the
toolchains themselves — the classic Thompson "trusting trust" problem. A compromised or merely
unverified toolchain undermines determinism and receipts regardless of how correct the source is.
Distinct from item 10 (whether two engines' outputs mean the same thing) — this is upstream of that:
whether the binary either engine runs was built by a trustworthy compiler at all.

### 19. Receipt-chain re-genesis / hash-algorithm migration bootstrap

Chapter 28.7's anticipated algorithm or canonicalization changes mean a live deployment with real
history may need to rekey or abandon an existing chain mid-lifecycle, not only at cold start.
Establishing a new $h_0'$ and a link — cryptographic, documentary, or a deliberate absence of one —
back to the old chain is itself an ungoverned, one-time act. Distinct from item 6, which covers only
the original genesis; a re-genesis after real receipted history exists is a different moment with a
different failure mode.

### 20. Retroactive deletion / legal-erasure precedent vs. immutable hash chain

A regulatory or contractual deletion obligation (GDPR erasure, a litigation hold's release) can
require removing or redacting a fact already baked into the hash chain, where Chapter 13.3 states
"if an event changes, is deleted... subsequent hashes change." The first time this tension is
resolved — tombstone-and-rehash, cryptographic redaction, or refusal — sets a structural precedent
with no prior governed pattern to check it against. Distinct from items 9 and 10 (last mile: output
exiting to non-deterministic interpretation) — this is the inverse, a legal mandate to remove
something already inside the immutable structure, a first-mile problem running in reverse.

## What these have in common

Every item above is the same shape: a judgment call (semantic fit, shape correctness, domain
completeness, interpretation, trust, genesis, authority, identity, correspondence, novelty) that
must happen before or after the deterministic spine can apply, and that the spine's own guarantees
do not reach. None of this is an argument against the architecture — Chapter 28.8's "does not
eliminate semantic review" is the right posture, not a concession. It is a argument against ever
describing the system as closing these gaps rather than bounding and disclosing them.

## See Also

- `docs/releases/v26.7.13/THESIS.md` — Chapter 28 (Limitations and Open Theorems), §12.3, §12.11,
  §13.3, §26.3, Appendix J
- `docs/releases/v26.7.13/THESIS_GROUNDING.md` — adoption-time re-checks of the thesis's standing
  claims
- `docs/jira/v26.7.12/CROWN_STATUS.md` — the crown-witness edge ledger these bootstrap gaps sit
  upstream and downstream of
- `docs/standing/REALITY_INDEX.md`, `docs/standing/CLAUDE_CODE_POLICY.md` — standing verification
  conventions this document follows
