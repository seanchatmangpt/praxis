# Why nothing can understand you (and why that's the whole design)

**Source:** `docs/thesis/00_foundations.tex`, Theorem `thm:rice` (line 183) and Corollary
`cor:noadmit` (line 205).

## The plain-English version

Imagine you want to write a gatekeeper program that only lets through requests that are
"actually safe" — where "safe" means something about what the request *would do* if you
ran it, not just what it *looks like*. It seems like a reasonable thing to want. It is
provably impossible in general.

This isn't a Praxis-specific limitation, and it isn't a limitation of how clever the
gatekeeper's programmer is. It's a 1953 result called **Rice's theorem**: for *any*
non-trivial question about what a program's behavior actually *means* (does it ever do X?
does it always terminate? does it compute Y?) — where "non-trivial" just means "the
answer is 'yes' for some programs and 'no' for others" — there is no algorithm that can
answer that question correctly for every possible program you hand it. Not "we haven't
found one yet." Not "with a big enough computer." *No such algorithm can exist*, for the
same reason no algorithm can solve the halting problem — because if one did, you could
use it to build a halting-problem solver, and we already know that's impossible.

## Why this forces the entire architecture

The thesis's Corollary (`cor:noadmit`) draws the consequence directly: **any admission
procedure that actually terminates and gives an answer cannot be deciding what an
observation *means***. It can only be deciding something about its *surface form* — its
syntax, its shape, its size, whether it matches a pattern — because that's the only kind
of question a terminating algorithm is allowed to answer for every input.

This is why Praxis's admission layer is built the way it is: not as an "understanding"
step that inspects intent, but as a **quarantine** — a mechanical, syntactic check that
retracts an arbitrary input onto a smaller, decidable sub-language. It doesn't claim to
know what you meant. It claims something much smaller and much more defensible: that your
request's *shape* passed a specific, nameable, checkable set of gates.

## Why this is good news, not bad news

It would be tempting to read "no program can understand meaning" as a limitation to work
around. The thesis's point is the opposite: once you accept it as a hard boundary, you
stop pretending your admission check verifies something it structurally cannot verify,
and you get to make honest, specific claims about what it *does* verify instead. Every
other result in this doc set is really a version of that same move — trade an
unfalsifiable claim about "understanding" for a narrower, falsifiable, mechanically
checked one.

## Verification status

`thm:rice` has been machine-checked by the real Lean 4 kernel, in *both* verification
lanes:

- **Bare-core lane** (`tools/paper-factory/lean-pilot/thm_rice.lean`): the halting-problem
  reduction is proved by hand, from 9 axioms describing an abstract computability layer.
- **Mathlib lane** (`tools/paper-factory/lean-lake/Praxis/Corpus/thm_rice.lean`): the
  corpus's exact theorem shape is derived as a direct corollary of Mathlib's own
  already-published formalization of Rice's theorem
  (`ComputablePred.rice₂`, from `Mathlib.Computability.Halting`) — using *zero* new axioms,
  because Mathlib already has real program codes and a real undecidability proof to cite.

Both are listed `verified` in their respective receipts files. See
[../reference/00-biggest-theorems-table.md](../reference/00-biggest-theorems-table.md) for
the full citation table.
