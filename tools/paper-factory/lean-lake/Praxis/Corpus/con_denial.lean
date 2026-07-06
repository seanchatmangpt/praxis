import Praxis.Corpus.def_adm
import Praxis.Corpus.def_denialabstract

/-!
Label: con:denial

"Let $D=(\{0,1\}^n,\lor,\bm 0)$ be the commutative idempotent monoid of denial
words: $n$ independent lanes, componentwise OR, identity $\bm 0$. Each
obligation $g_i$ contributes a lane map $d_i:\Obs\to\{0,1\}^n$ with
$d_i(o)=\bm 0\iff g_i(o)=1$. The total denial is $d(o)=\bigvee_i d_i(o)$, and
$\adm(o)\ne\Rfsl\iff d(o)=\bm 0$."

`D = (\{0,1\}^n,\lor,\bm 0)` is exactly `Deny n` from `Praxis.Mathlib.PropMonoid`
(imported transitively via `Praxis.Corpus.def_denialabstract`): `Fin n → Bool`
with the pointwise `BooleanAlgebra` sup as `∨` and `⊥` as `\bm 0`. It is not
redefined here, per the corpus vocabulary-reuse rule.

`Obs` is `Nat.Partrec.Code`, matching `def:adm`. Given the same finite battery
`gs : List (Code → Bool)` of obligations used by `adm` in `def:adm`, we set
`n := gs.length` and construct:

* `laneMap gs i o : Deny gs.length`, the lane map `d_i` for obligation `i`,
  which is the all-zero word (`⊥`) exactly when `g_i(o) = 1` (`true`), and
  otherwise the single-hot word setting only lane `i` (so that `d_i(o) = \bm 0`
  really does capture "obligation `i` is denied by observation `o`" as a
  vector, matching the LaTeX's `d_i : \Obs \to \{0,1\}^n`).
* `denial gs o : Deny gs.length`, the total denial `d(o) = \bigvee_i d_i(o)`,
  built as the `Finset.univ.sup` of the lane maps -- Mathlib's finite-sup
  operation on a `BooleanAlgebra`/`Lattice`, not a hand-rolled fold.
* `denialAdmits gs o`, the proposition `d(o) = \bm 0`, restated directly in
  terms of the pre-built `⊥` from `Deny`'s `BooleanAlgebra` instance, giving
  the right-hand side of `\adm(o)\ne\Rfsl\iff d(o)=\bm 0` (the left-hand side
  is `adm Adm gs o ≠ none`, already available from `def:adm`).

This is a `construction`: it packages the LaTeX's data (the monoid, the lane
maps, the total denial, and the two sides of the stated correspondence) using
only pre-built Mathlib/core machinery (`Fin`, `Bool`, `Finset.sup`, the
`BooleanAlgebra` on `Deny n`), with no new axioms and no proof obligation
beyond this file type-checking.
-/

open Nat.Partrec (Code)

namespace Deny

/-- The lane map `d_i` for obligation `i` in a battery `gs`: the all-zero
word (`⊥`, i.e. clean/`Rfsl`-free) exactly when obligation `i` holds
(`g_i(o) = true`), otherwise the single-hot word marking lane `i` as denied. -/
def laneMap (gs : List (Code → Bool)) (i : Fin gs.length) (o : Code) : Deny gs.length :=
  fun j => if j = i then !(gs.get i o) else false

/-- The total denial `d(o) = \bigvee_i d_i(o)`, the finite `Lattice` sup
(Mathlib's `Finset.sup` over `Finset.univ : Finset (Fin gs.length)`) of the
per-obligation lane maps `laneMap`. -/
def denial (gs : List (Code → Bool)) (o : Code) : Deny gs.length :=
  Finset.univ.sup (fun i => laneMap gs i o)

/-- The right-hand side of the stated correspondence: `d(o) = \bm 0`, i.e.
the total denial word is the identity `⊥` of `Deny gs.length`'s
`BooleanAlgebra` instance. -/
def denialAdmits (gs : List (Code → Bool)) (o : Code) : Prop :=
  denial gs o = (⊥ : Deny gs.length)

end Deny
