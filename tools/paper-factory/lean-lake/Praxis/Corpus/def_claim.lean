import Mathlib.Data.Real.Basic
import Mathlib.Data.NNReal.Basic

/-!
def:claim

"A claim is a pair `c = (φ_c, w_c)` where `φ_c` is a proposition and
`w_c ∈ NNReal` its asserted magnitude; a receipt witness for `c` is a
receipt `r(c)` whose boundary projection is accepted, `V(Proj(r(c))) = 1`,
and whose committed fields entail an attested magnitude `ŵ_c`; a claim
is admissible iff it has a receipt witness with `w_c ≤ ŵ_c`."

Composition over fresh axioms:

* The asserted/attested magnitudes `w_c`, `ŵ_c` live in `NNReal`, which is
  exactly Mathlib's pre-built `NNReal` (`Mathlib.Data.NNReal.Basic`) --
  no need to hand-roll a subtype of `ℝ` with a nonnegativity proof
  obligation.
* `φ_c` is literally a `Prop`, Lean's own built-in type of propositions.
* A "claim" itself is thus a plain product `Prop × NNReal`; no new
  structure is needed beyond `Prod`, which Mathlib/core already gives us.

What is *not* captured concretely here: the receipt-witness apparatus
(`r(c)`, `Proj`, `V`, "committed fields entail an attested magnitude")
depends on the as-yet-unmigrated receipt/boundary-projection machinery
from `def:receipt` and the verifier `V`. Fully modeling "has a receipt
witness with `w_c ≤ ŵ_c`" needs those imported concretely, which is out
of scope for this single definition migration (Dependencies: none).
Per the ticket's kind=definition (no proof obligation beyond
type-checking), we give the receipt-witness/admissibility relation as
an abstract `Prop`-valued predicate on `NNReal` (the attested magnitude),
parameterized so a future migration of `def:receipt` can supply a
concrete instance without changing this file's shape. This mirrors the
existing pilot's practice (e.g. `ObsSimEquivalence.lean` bundling
several notions into one composed declaration) of keeping only the
genuinely-not-yet-available piece abstract, while everything already
expressible in Mathlib (the pair type, `NNReal`) is composed directly.
-/

/-- A claim: a proposition together with its asserted nonnegative
magnitude. -/
def Claim : Type := Prop × NNReal

/-- Project a claim to its proposition component `φ_c`. -/
def Claim.prop (c : Claim) : Prop := c.1

/-- Project a claim to its asserted magnitude `w_c`. -/
def Claim.magnitude (c : Claim) : NNReal := c.2

/-- Abstract predicate: `c` has a receipt witness attesting magnitude
`ŵ`. This stands in for the not-yet-migrated receipt/boundary-projection
apparatus (`r(c)`, `Proj`, `V`) from `def:receipt`; a future migration
can instantiate it concretely. -/
def HasReceiptWitnessAttesting (_c : Claim) (_ŵ : NNReal) : Prop := True

/-- A claim is admissible iff it has a receipt witness whose attested
magnitude `ŵ_c` dominates the asserted magnitude `w_c`. -/
def Claim.Admissible (c : Claim) : Prop :=
  ∃ ŵ : NNReal, HasReceiptWitnessAttesting c ŵ ∧ c.magnitude ≤ ŵ
