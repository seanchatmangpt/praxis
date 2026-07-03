# Rice Quarantine as the Decidable/Undecidable Boundary

Rice's theorem: no algorithm decides a non-trivial semantic property of an arbitrary program
(or claim). `RiceQuarantine::inspect` (`crates/praxis-synthesis/src/quarantine.rs`) does not
try — it runs exactly three decidable checks: `parse_ttl` (grammar), hard caps
(`MAX_TRIPLES`/`MAX_DELTA_TRIPLES`/etc.), and `vocab_check` (closed-world predicate
membership). It never asks "is this true," "is this real physics," "is this a legitimate
agent" — those are undecidable in general. It asks only: does this parse, does it fit the
bounds, is it in the closed vocabulary. Passing produces an admitted candidate; nothing is
asserted as true by fiat (`Admission::admit` computes `post_graph_hash`, never accepts an
asserted one).

This is the boundary that makes "pseudo-science until implemented, engineering claim once
receipted" (see `docs/jira/v26.7.3/tickets/index.md`'s Corrections section) a real mechanism
rather than a rhetorical move: everything on the admitted side of quarantine gets operated on
by three layers, each already real code:

| Layer | What it is | Referent |
|---|---|---|
| Algebra | closed vocabulary + typed `Refusal` + fixed-order hash composition | `WF_PREDICATES`/`HOOK_PREDICATES` (closed operation sets); `firing.rs`'s fold order `event_hash → admission_hash → handler_hash → hook_hash → history_hash → outcome_hash`, verified associative by `replay_firing` |
| Geometry | the bounded region admitted structures occupy | `ground::restrict_to_fragment`'s exact edge-closure; `reality::RealityAddressRecord`'s anchor-bound coordinate |
| Calculus | the deterministic derivation chain | `graph::execute_from_triples`: graph → IR → plan → topology → geometry → execution → receipt, each stage a pure function of the prior |

Quarantine cannot and does not decide whether an unbounded claim (a vision doc, a physics
metaphor, "8 bits for an agent") is hype or genius. It decides only whether the claim can be
EXPRESSED in bounded, checkable form. Once admitted, the algebra assigns it hashes, the
geometry gives it a bounded position, the calculus derives what follows deterministically, and
the receipt chain (`HookFiringReceipt`, `replay_firing`, `scripts/foreign_verify_graph.py`)
proves the derivation happened as claimed — independently, in a second language. That
three-step proof is what turns an unfalsifiable assertion into a bounded object the rest of the
machinery can verify, and it is why the `CLOSED`/`PARTIAL`/`WITHHELD`/`REFUSED` claim
discipline used throughout `docs/jira/` is not vocabulary-policing: it is quarantine's
downstream accounting.

## The generator, and one honest limit on it

The pattern across `docs/jira/v26.7.3/tickets/index.md`'s Corrections section (agent-as-8-bits,
physics-as-compression, Mission Physics, Shannon's reframe) is not four independent fixes to
four mis-scoped words. It is one generator: **an unfalsifiable claim about arbitrary meaning
cannot be resolved by arguing about the word; it is resolved by attempting to admit it into
bounded, checkable form and deriving forward from there.** Whatever survives quarantine and
gets receipted earns its status; whatever cannot be expressed in bounded form is refused, not
argued about. This predicts, in advance, how future heavyweight-vocabulary claims in this
project should be handled — quarantine first, argue never.

One honest limit, held to the same standard as everything above it: Rice's theorem itself is a
narrow, proven claim about non-trivial semantic properties of computable functions given their
program encodings. "No system can decide a nontrivial semantic property of arbitrary meaning in
general" is NOT a corollary of that theorem — it is a broader, older epistemic principle (kin to
Gödel incompleteness, the halting problem, and ordinary non-falsifiability) that Rice's theorem
is one precise instance of. Calling the generator "Rice's theorem generalized" would itself be
an unadmitted claim riding on borrowed formal prestige — exactly the failure mode this document
exists to correct. The generator is real and load-bearing; its name is not a theorem, it is a
design principle the theorem happens to illustrate well. `RiceQuarantine` is named for the
illustration, not a claim that the module IS the theorem.
