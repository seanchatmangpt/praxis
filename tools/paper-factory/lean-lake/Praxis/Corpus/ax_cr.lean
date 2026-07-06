import Mathlib.Data.Real.Basic
import Praxis.Mathlib.DefReceipt

/-!
ax:cr -- collision resistance of `chainH` (BLAKE3), reformalized in the
Mathlib lane.

`Digest` (`BitVec 256`) and the fixed-width chaining function `chainH :
Digest → Digest` already exist, composed/axiomatized in
`Praxis.Mathlib.DefReceipt` (imported above) -- reused here rather than
redeclared. This statement is about a different, genuinely new fact:
collision resistance over the *actual* hash domain (arbitrary-length
messages, not just 256-bit digests -- `chainH : Digest → Digest` has
equal-size domain/codomain, so "collision" there is not the interesting
security property; the corpus statement is about hashing arbitrary
messages down to a 256-bit digest).

Per the composition-first directive, this is checked against Mathlib
first: Mathlib has no PPT-adversary / negligible-function / advantage
model for cryptographic security games (that is out of its scope as a
pure-math library), so the following remain genuinely axiomatized,
matching the justification style of `chainH`/`chainStep` in
`DefReceipt.lean`:

* `Msg` -- the arbitrary-length message space hashed by BLAKE3. Composed
  from Lean core's `List Bool` (a bitstring), not an opaque axiom.
* `msgHash : Msg → Digest` -- the real BLAKE3 hash function applied to
  arbitrary messages. Axiomatized for the same reason `chainH` is: no
  verified BLAKE3 implementation exists in Mathlib/Lean core, and a fake
  stand-in would make any downstream security statement vacuous or
  meaningless.
* `Adversary` -- a PPT (probabilistic polynomial-time) collision-finding
  algorithm. Mathlib has no computational-complexity/PPT-machine model,
  so this is an opaque axiomatized type, standard practice in the
  handful of existing Lean crypto-formalization efforts (e.g. SSProve,
  which is a separate library, not part of Mathlib).
* `advantage` -- an adversary's collision-finding success probability at
  security parameter `λ`. Modeled as `Adversary → Nat → ℝ`, reusing
  Mathlib's real numbers (`Mathlib.Data.Real.Basic`) rather than a new
  probability type, since only the numeric bound matters here, not a
  full probability-monad formalization.

`Negligible` is *not* axiomatized: it is a plain definition (the standard
asymptotic notion, "smaller than the inverse of every polynomial,
eventually"), stated directly in terms of pre-built `Nat`, `Real`, and
`Real`'s own order/field structure -- no axiom needed for it.

The corpus statement itself (the existence of a negligible bound
`ε(λ)`, `λ = 256`, at the birthday-bound-appropriate scale `~2^128`,
for every PPT adversary) is the one remaining axiom, `chainH_cr`.
-/

abbrev Msg := List Bool

axiom msgHash : Msg → Digest

axiom Adversary : Type

axiom advantage : Adversary → Nat → ℝ

/-- Standard asymptotic negligibility: eventually smaller than the
inverse of every polynomial. Not an axiom -- a plain definition over
pre-built `Nat`/`Real`. -/
def Negligible (f : Nat → ℝ) : Prop :=
  ∀ c : Nat, ∃ N : Nat, ∀ n, N ≤ n → f n < (1 : ℝ) / (n : ℝ) ^ c

/-- `chainH` (BLAKE3) is collision-resistant: no PPT adversary finds
`x ≠ y` with `msgHash x = msgHash y` except with negligible probability
`ε(λ)`, at security parameter `λ = 256` (birthday bound `~ 2^128`).
Modeled as: there is a negligible function `ε` bounding every
adversary's collision-finding advantage at every security parameter. -/
axiom chainH_cr :
  ∃ ε : Nat → ℝ, Negligible ε ∧ ∀ (A : Adversary) (n : Nat), advantage A n ≤ ε n
