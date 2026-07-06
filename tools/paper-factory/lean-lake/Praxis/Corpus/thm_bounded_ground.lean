import Mathlib.Data.Fintype.Vector
import Mathlib.Data.Fintype.BigOperators
import Mathlib.Data.Fintype.Card
import Mathlib.Tactic.Ring
import Praxis.Corpus.def_ground

/-!
# thm:bounded-ground — The number of ground actions is finite and a priori bounded

> Let `D` have schemas of maximum parameter arity `k` over `|Ob| = N` objects. The
> number of ground actions is at most `|S| · N^k`, a finite constant independent of
> the goal; consequently a forward search over grounded actions has a branching
> factor bounded a priori, and manufacture terminates within a fixed depth bound.

We formalize the counting claim: given a `Fintype` of schemas `D.Schema` and a
`Fintype` object universe `Ob`, if every schema's parameter list has length at most
`k`, then `Fintype.card (GroundAction D Ob) ≤ Fintype.card D.Schema * Fintype.card Ob ^ k`.

The proof reuses Mathlib wholesale: `GroundAction D Ob` is put in bijection with the
sigma type `Σ s : D.Schema, List.Vector Ob (arity s)` (a ground action *is* exactly a
schema paired with a length-indexed argument vector — `List.Vector` already carries
this length invariant, so no arity bookkeeping is reproved by hand); cardinality of
that sigma type is computed via Mathlib's `Fintype.card_sigma` and `card_vector`
(`Fintype.card (List.Vector α n) = Fintype.card α ^ n`), and the final bound is a
one-line monotonicity argument (`Finset.sum_le_card_nsmul` composed with
`Nat.pow_le_pow_right`) — no combinatorics is hand-rolled.

The "search terminates within a fixed depth bound" clause of the source sentence is
the informal algorithmic corollary of this finiteness/boundedness fact and is not
separately formalized (it is not a further mathematical statement about grounding,
but a remark about the search procedure that consumes it).
-/

namespace Praxis.Corpus.ThmBoundedGround

open Praxis.Corpus.DefDomain
open Praxis.Corpus.DefGround

variable {D : LiftedDomain} {Ob : Type} [Fintype Ob] [Fintype D.Schema]

/-- A ground action is exactly a schema together with a length-indexed argument
vector, the length being that schema's parameter count: `GroundAction` (a schema plus
a `List Ob` of the right length, carried as a separate proof field) is in canonical
bijection with the sigma type of `List.Vector`s indexed by arity. -/
def groundActionEquivSigma :
    GroundAction D Ob ≃ Σ s : D.Schema, List.Vector Ob (D.schemaData s).params.length where
  toFun a := ⟨a.schema, ⟨a.args, a.arity_eq⟩⟩
  invFun p := ⟨p.1, p.2.1, p.2.2⟩
  left_inv _ := rfl
  right_inv _ := rfl

instance : Fintype (GroundAction D Ob) :=
  Fintype.ofEquiv _ groundActionEquivSigma.symm

/-- **thm:bounded-ground.** If every schema of `D` has parameter arity at most `k`,
the number of ground actions is at most `|D.Schema| · N^k` where `N = |Ob|` — a finite
bound depending only on the domain and object universe, not on any goal. -/
theorem card_groundAction_le (k : ℕ)
    (hk : ∀ s : D.Schema, (D.schemaData s).params.length ≤ k) :
    Fintype.card (GroundAction D Ob) ≤ Fintype.card D.Schema * Fintype.card Ob ^ k := by
  rw [Fintype.card_congr groundActionEquivSigma, Fintype.card_sigma]
  calc
    ∑ s : D.Schema, Fintype.card (List.Vector Ob (D.schemaData s).params.length)
        = ∑ _s : D.Schema, Fintype.card Ob ^ (D.schemaData _s).params.length := by
          simp [card_vector]
    _ ≤ ∑ _s : D.Schema, Fintype.card Ob ^ k := by
          apply Finset.sum_le_sum
          intro s _
          exact Nat.pow_le_pow_right (Nat.zero_le _) (hk s)
    _ = Fintype.card D.Schema * Fintype.card Ob ^ k := by
          rw [Finset.sum_const, Finset.card_univ, smul_eq_mul]

end Praxis.Corpus.ThmBoundedGround
