# The algebra of refusal

**Source:** `docs/thesis/01_admission_algebra.tex`. Three theorems:
`thm:mono` (line 318), `thm:total` (line 419), `thm:freehom` (line 557).

## The plain-English version

If a gatekeeper can only check surface shape (see
[00-why-nothing-can-understand-you.md](00-why-nothing-can-understand-you.md)), the next
question is: what happens when you have a *list* of gates, not just one? Does the order
you check them in matter? Can adding a new safety rule ever accidentally let something
through that used to be blocked? Can you be sure every possible case gets *some* answer,
with nothing falling through the cracks?

This paper's answer is to treat "reasons for denial" as members of an algebraic structure
— the same kind of structure that ordinary addition or set-union belong to — so that
these questions have provable, not just hoped-for, answers.

### 1. Adding more rules can only deny more, never admit more (`thm:mono`)

Picture the set of "denial reasons so far" as something that only ever grows as you check
more rules — never shrinks. If observation `o` was already denied under a small rule set
`G`, then checking it against a *bigger* rule set `G'` (with `G` inside `G'`) can never
suddenly clear it. Denial is "antitone" in the rule set: more rules, less (or equal)
admission, always. This is exactly the property you want from a safety system — you
never want tightening security to accidentally open a door.

### 2. Every case gets exactly one classification, and the compiler proves it (`thm:total`)

There are thirteen distinct "kinds of denial" scenario in the system. The theorem says:
every one of the thirteen maps to exactly one of seven named categories (like `Identity`,
`Capacity`, `Temporal`) — no scenario is left unclassified, and none maps to two
categories at once. What makes this interesting isn't the math — it's *how* it's
guaranteed: the classification is written as a Rust `match` expression with no wildcard
catch-all arm. If someone later adds a fourteenth scenario and forgets to classify it, the
code simply **fails to compile**. The proof of totality isn't a separate check running
somewhere — it's the same guarantee the Rust compiler already gives you for free, put to
use as a mathematical property.

### 3. A pipeline's overall denial doesn't care what order its stages ran in (`thm:freehom`)

If a request passes through several pipeline stages, each of which can independently deny
it for its own reason, you might worry that *the order* of the stages changes the outcome,
or that running the same stage twice changes something. The theorem says neither is true:
the combined denial reason for a whole pipeline is completely determined by the *set* of
distinct stages that denied it — not their order, not how many times each ran. This is
what lets you reason about "did stage X ever object to this request?" as a simple
yes/no question, independent of the pipeline's plumbing.

## Verification status

- `thm:mono`: verified in the Mathlib lane; **blocked** in the bare-core lane (one of its
  dependency statements didn't verify there — the Mathlib-lane result is the one to trust).
- `thm:total`: verified in the bare-core lane; **unformalized** in the Mathlib lane (a real,
  specific Lean error on file — the bare-core proof is the one that's machine-checked).
- `thm:freehom`: verified in *both* lanes.

This mixed picture is itself an honest data point, not swept under the rug: which lane
successfully mechanizes a given theorem varies statement by statement, and both lanes are
kept as independent, parallel verification attempts rather than one being silently
preferred. See
[../reference/00-biggest-theorems-table.md](../reference/00-biggest-theorems-table.md)
for exact citations and the real kernel error text where a lane failed.
