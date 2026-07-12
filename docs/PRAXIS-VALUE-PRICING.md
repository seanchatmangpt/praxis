# Praxis Value & Prestige Pricing

*Last Updated: 2026-07-11*

Audience: whoever is about to price Praxis in a real conversation with a real prospect, or
whoever has to defend a Praxis number to their own leadership afterward. This is a strategy
brief for how to think about the number before you say it, and how to hold it once you have.
It is not a rate card, a cost worksheet, or an ROI model, and no figure in it should be handed
to a customer, auditor, or internal finance function as an audited historical cost.

## What Kind of Pricing This Is

Praxis is priced as a Veblen good — a thing whose value is perceived partly *through* the
price, not despite it — rather than priced cost-plus (materials plus margin) or ROI-justified
(a spreadsheet proving payback period). This is a deliberate choice, not a rationalization for
charging more than the work "costs."

It is also not a novel or fringe choice. Luxury goods have used it for a century. Elite
consulting brands (the ones that bill a partner's day at a number no junior associate could
individually justify) use it every engagement. Enterprise software vendors use it whenever the
number on the contract signals that the buyer chose the category leader, not merely a vendor.
None of these buyers are naive; they are buying the signal along with the substance, because
the signal is itself functional — it's what the buyer takes back to their own stakeholders.

The discipline this implies: a good number doesn't need to survive an audit, it needs to
survive a retelling. The customer's champion will repeat your number to their CFO, their board,
their own client. What makes the number "good" is whether it still sounds coherent on the third
retelling — not whether it reconciles to a spreadsheet you'd hand over in discovery. Optimize
for narrative coherence, not defensibility.

## The Hook: Five Specialties, One Continuous Context

The pricing posture above only works if the breadth behind it is real, not asserted. Direct
inspection of one Praxis codebase shows working systems across five separate technical
specialties, reasoned about inside a single continuous context rather than handed off between
teams:

1. **Rust systems engineering** — BLAKE3 cryptographic receipt chains, with a
   zero-unsafe-code-outside-cryptographic-verification discipline enforced throughout.
2. **RDF/SPARQL semantic-web engineering** — an oxigraph-backed SPARQL execution layer, SHACL
   shape validation, and roughly seventy real Turtle ontology files carrying stable external
   namespaces.
3. **Formal verification** — Lean 4 / Mathlib theorem proving, backed by an automated
   axiom-audit tool that has already found seventy-one unauthorized axioms sitting in a
   pre-existing proof corpus.
4. **Distributed systems** — Erlang/OTP supervision trees, including a working Raft-style
   leader-election implementation for a worker pool.
5. **Automated planning and applied mathematics** — PDDL classical and temporal planning, a
   POWL v2 process-mining formalism whose implementation cites specific published academic
   algorithms by name and definition number, and a multifractal partition-function estimator
   computing Z(q, ε), τ(q), the generalized dimension D(q), and the singularity spectrum f(α)
   via Legendre transform.

Any one of these is a legitimate hire. A Rust systems engineer trusted near a cryptographic
receipt path is not common. A Lean 4 engineer who can audit a proof corpus for unauthorized
axioms is rarer still. What actually anchors a prestige number is finding all five inside one
continuous line of reasoning — the same discipline that caught a hazard in the Erlang layer
also caught a vacuous proof in the Lean layer. That continuity, not any single specialty, is
what the price is for.

## The Three Catches: What the Price Actually Buys

Lead every pricing conversation with what almost went wrong, not with what Praxis is built
from. Three specific things were caught before they could reach a customer's stakeholders. Each
is precise, not a vague "we found some bugs" gesture — precision is what makes a story
retellable.

## The Theorem That Proved Nothing

A collaborator delivered a Lean 4 theorem meant to certify a caching-safety mechanism — a
"local modulation freezing" claim, the kind of formal result meant to be the load-bearing proof
behind a production caching decision. The theorem compiled. It typechecked. Its conclusion was:

```lean
∃ frozen_state, frozen_state = True
```

That statement is true unconditionally, for any input, regardless of any hypothesis in the
theorem — witnessed trivially by `frozen_state := True`. It says nothing about caching,
freezing, or the system it was supposed to certify. It would have shipped as a proven
multifractal-freezing result, sitting in the codebase as formal backing for a safety mechanism
that had never actually been proven safe.

It was caught, and named precisely: not "wrong," but vacuous — the hypothesis is never used.
Naming the failure mode exactly is what let it get fixed instead of argued about. The
replacement is a real, non-vacuous bound, derived from `Real.add_one_le_exp` and
`Real.exp_le_exp`: `|exp(wx) - exp(wb)| <= exp(wb) * (exp(eps) - 1)`, with a proven vanishing
limit as the perturbation shrinks. That theorem says something, because its hypotheses are
actually load-bearing.

## The Pathway That Would Have Hot-Loaded Code into Production

An Erlang workflow-orchestration module (`apps/arazzo_runner`) contained a pathway that would
take LLM-generated code and hot-load it directly into a running production node for execution.
Sitting next to it was a checked-in document asserting fabricated AGI capabilities as
already-completed, already-shipped work.

Both were found, both were verified real — not a false alarm, not a misreading of intent — and
both were removed. The legitimate infrastructure underneath was left untouched: a working
Raft-style leader-election implementation for the worker pool, which had nothing to do with
either hazard. That distinction matters: the fix was surgical, not a purge, because the
discipline that finds the hazard is the same discipline that knows what isn't one.

## The Self-Caught Overclaim

Five days into a release, a project status document claimed "8/8 publishable crates published"
to crates.io. A later, more careful verification pass found that the flagship crate was
actually unregistered — the registry API returned a 404 for it. The claim was stale, and it was
generated by the same discipline now being priced, about its own prior work.

That third story does the most work in a pricing conversation, because it pre-empts the
objection before it's raised. The natural pushback to "we catch things before they ship" is
"sure, but who catches you?" The answer is already on record: the discipline caught its own
five-day-old overclaim and flagged it at the source, rather than letting it stand until a
customer found it first.

## Why This Required Judgment, Not Automation

None of these three is a linter finding. A vacuous theorem typechecks — the compiler has no
opinion about whether a proof's hypotheses are used. A hot-load pathway runs without complaint
until someone asks whether it should exist. A "published" claim reads as true right up until
someone queries the registry. Catching each one required someone who understood the specific
domain well enough to ask the question the tooling doesn't ask — which is why the five
specialties above are part of what's being priced, not a separate feature from it.

## Applying the Narrative Techniques

Once a catch has landed — before any number is said out loud — apply these four techniques.
For Veblen-good pricing, they aren't tricks layered on top of a "real" number; they are how the
number gets its meaning.

**Anchor to a vivid, specific catch, not a generic multiplier.** Don't say "we add 30%
risk-mitigation value" or "we improve quality by a measurable margin." Nobody retells a
percentage. Say "the price is what it costs to have someone in the room who would have caught
the `frozen_state` theorem before it shipped as a proof." A specific, named catch invites a
story response — and a story is what the buyer takes back to their own stakeholders.

**Use round numbers, not computed ones.** A number like $500,000 reads as a decision. A number
like $487,300 reads as a spreadsheet output that leaked into the room, and it invites exactly
that treatment: "walk me through the $487,300." The first signals "we decided this is what it's
worth"; the second signals "we backed into this from costs," which is the cost-plus framing
this document argues against. This is an illustration of the *type* of number to use, not a
recommended figure — set the actual number to the deal in front of you, and round it
deliberately rather than backing into a round number from a cost estimate.

**Attach the number to something irreplaceable, never to hours or a rate.** The moment a number
is traceable to a day-rate, the customer's next move is arithmetic: divide by the rate,
multiply by the days they think it should take, negotiate to that figure. "$400/hour" invites
"what if it only takes 200 hours." Attach the number instead to what has no day-rate: the
standing capability to catch a vacuous safety proof, a live production code-injection pathway,
or a five-day-old overclaim, before any of the three reaches a customer's stakeholders. That
capability isn't purchased by the hour; it's either present in the system or it isn't.

**Frame the big number as the cost to replicate the alternative, not the cost of the work.**
This is the strongest version of the claim, and it happens to also be the honest one. "What we
spent building this" is a historical assertion someone can ask to see receipts for. "What it
would cost to independently staff and verify this yourselves" is a claim about a hypothetical
staffing plan, and hypotheticals aren't audited — they're evaluated on plausibility.

## What the Alternative Costs to Replicate

The staffing profile behind this frame is not invented; it falls directly out of the five
specialties above, and each catch above shows why the specialty is load-bearing rather than
decorative. Reproducing this breadth means fielding, at minimum:

- a **Rust systems engineer** trusted near cryptographic receipt code, who may also cover the
  oxigraph-based RDF/SPARQL layer or be joined by a dedicated semantic-web/knowledge engineer,
  depending on how the buyer's own org is shaped;
- a **formal-methods/Lean engineer** capable of not just running a proof assistant but
  distinguishing a vacuous proof from a real one and constructing the real one in its place —
  the skill that caught the `frozen_state` theorem;
- a **distributed-systems/Erlang engineer** who can read a supervision tree closely enough to
  find a hot-load hazard before it reaches production, and who knows which parts of the module
  to leave alone; and
- a **process-mining/applied-math specialist** who can implement a published planning algorithm
  faithfully enough to cite it by definition number, and who is equally at home deriving a
  singularity spectrum.

That is a real, senior, four-to-five-person staffing plan, not a padded estimate — but state the
resulting figure the way it actually is: a strategic anchor describing the staffing and
coordination cost of the alternative, not a historical invoice or an audited internal cost
figure. Say that sentence, or one like it, before anyone else has to ask for it. It costs
nothing to say, and it's what keeps the number retellable rather than disputable — a claim that
collapses under a direct question isn't coherent, it's a liability.

## Delivering the Number in the Room

Order matters more than wording.

1. **Open with one catch, told precisely.** Technical buyers respond to the vacuous theorem,
   risk/compliance buyers respond to the hot-load pathway, and finance/leadership buyers
   respond to the self-caught overclaim, because it pre-empts "who checks you." Include the
   specific detail — the exact conclusion, the exact file, the exact stale claim. Vagueness
   reads as invention; precision is what makes the story theirs to retell.
2. **Name what almost happened, not what was fixed.** The value is in "this would have shipped
   as a proven result" or "this would have executed in production," not in the repair itself.
   The repair is expected; the catch is the point.
3. **State the figure once, plainly, and stop talking.** Do not preface it with hours,
   headcount, or a rate. Do not follow it with justification. A number stated once and left
   alone reads as confidence; a number followed by three sentences of defense reads as a number
   you don't quite believe.
4. **If asked how you arrived at it, answer with the alternative-cost frame, not an internal
   cost breakdown.** "It reflects what it would take to staff and independently verify this
   yourselves, across the specialties involved, with no guarantee you catch your own
   overclaiming the way this system caught its own" is a complete answer.
5. **If pressed toward an hourly or per-seat framing, redirect to the catch, not to a defense of
   the number.** "The `frozen_state` theorem would have needed the same review process that
   approved it in the first place to catch it a second time — that's what's being priced, not
   the hours" declines the hourly frame without conceding it.
6. **Never let the alternative-cost figure be treated as an audited historical spend.** The
   moment it's asked to survive an audit, restate it as illustrative, not walk it back.

Hold the same discipline in the room that the anecdotes themselves demonstrate. Don't say
"production-ready" without naming the scope it's ready for. Don't say a result "should work" —
say what was verified and how. The self-caught-overclaim anecdote is proof this discipline is
real; contradicting it with loose language in the sales conversation is the one thing that
would make the whole story incoherent on the retelling. The number is allowed to be aggressive.
The claims underneath it are not allowed to be soft.

## What This Document Is

This is a narrative-strategy document, not a cost-accounting one. It exists to organize how a
price is framed and told, not to certify what Praxis cost to build or what any customer should
be charged. The three catches are real and were verified against the repository; they are
evidence of a working discipline, not a certification of the whole system, and any readiness
claim used in a real conversation should be scoped the same way — to what was actually checked,
not to the system as a whole. The "cost to replicate" figures are explicitly narrative anchors:
useful for making a price legible and retellable, never to be represented as audited,
historical, or line-item accounting claims. Anyone using this document should be able to say
that plainly if asked directly — that's consistent with, not opposed to, the pricing strategy,
since a claim that collapses under a direct question was never narratively coherent to begin
with.

## See Also

- `docs/CORE_TEAM_DISCIPLINE.md` — the underlying engineering standards the three catches above
  were enforcing
- `docs/CHATMAN_EQUATION.md` — the formulation behind the systems referenced in the
  domain-breadth section
- `docs/standing/PRODUCTION_READINESS.md` — the scoped-readiness standard this project holds
  itself to internally
- `docs/standing/CLAIM_RULES.md` — the claim-vocabulary rules this document follows
- `docs/claims/WITHHELD_CLAIMS.md` — claims the project has deliberately declined to make;
  context for why this document restricts itself to the anecdotes sourced above
- `.claude/rules/no-overclaiming.md` — the vocabulary discipline behind the self-caught-
  overclaim anecdote and behind this document's own claims
