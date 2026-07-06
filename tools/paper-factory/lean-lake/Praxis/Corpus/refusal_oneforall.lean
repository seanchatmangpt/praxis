import Praxis.Corpus.def_earned

/-!
# `refusal:oneforall` — No `OneForAll`

OTP justifies `one_for_all` by *shared mutable fate* between siblings: if one sibling's
failure invalidates another's invariants, restarting only the first is unsound, so the
whole cohort must restart together. In an acyclic data-flow plan (`def:earned`'s `Strategy`
is built over a `WellFounded`, hence acyclic, `edge` relation) same-depth siblings are
independent by construction — neither is upstream of the other, so there is no shared
mutable state for a `OneForAll` coupling to encode. The coupling is therefore
inexpressible in this model, not merely unused.

We do not encode this as a disprovable mathematical proposition (there is no ambient
"space of all possible strategies" inside which to prove `OneForAll` absent — it is a
modeling choice, not a theorem). Instead, following the ticket's own diagnostic — "the
variant does not appear in the Strategy enum at all; an exhaustive-match test makes its
future addition a compilation event, forcing the doctrine conversation" — we encode the
refusal as an exhaustive `match` over `Strategy` (imported unchanged from `def:earned`,
not redefined). `Strategy` has exactly two constructors, `restForOne` and `oneForOne`.
This `def` has no wildcard (`_`) arm: it only compiles because those are the *only* two
constructors that exist. Should a future author add a third `Strategy.oneForAll`
constructor, this match becomes non-exhaustive and the file fails to compile, which is
exactly the "compilation event" the doctrine calls for — forcing a human conversation
about the doctrine rather than silently accepting the new variant.

No `axiom` is declared: the refusal is witnessed by the Lean elaborator's own
exhaustiveness check on a closed inductive type, which is a stronger and more honest
encoding than an asserted axiom would be.
-/

namespace Praxis.Corpus.RefusalOneForAll

open Praxis.Corpus.DefEarned

/-- Exhaustive, wildcard-free witness that `Strategy` has exactly the two doctrine-approved
constructors `restForOne` and `oneForOne`. There is no third `oneForAll` constructor to
match: adding one is a compile-time event, not a silent addition, which is the guarantee
`refusal:oneforall` demands. -/
def strategyIsExhaustivelyTwoValued (s : Strategy) : Unit :=
  match s with
  | Strategy.restForOne => ()
  | Strategy.oneForOne => ()

end Praxis.Corpus.RefusalOneForAll
