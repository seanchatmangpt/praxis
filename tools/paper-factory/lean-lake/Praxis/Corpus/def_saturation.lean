import Mathlib.Order.FixedPoints
import Praxis.Corpus.def_ground

/-!
# def:saturation — Least fixpoint of the immediate-consequence operator

Given a grounded PDDL problem `(D, Ob)` with initial atom set `m₀` and ground action
set `T`, the (Nemo Datalog) program `Π` has one fact per atom in `m₀` and one rule per
ground action. Its saturation `sat(Π)` is the least fixpoint `lfp(T_Π)` of the
immediate-consequence operator `T_Π`; an atom `p` is reachable iff `p ∈ sat(Π)`.

We model ground atoms for a domain/object-universe pair `(D, Ob)` (reusing
`Praxis.Corpus.DefGround.GroundAtom`) as living in `Set (GroundAtom D Ob)`, which is a
`CompleteLattice` (Mathlib, `Order.SetLattice`), and take the immediate-consequence
operator itself as a monotone self-map `T_Π : Set (GroundAtom D Ob) →o Set (GroundAtom
D Ob)` — monotone because adding derived facts can only add antecedents, never remove
justified conclusions. The saturation is then literally Mathlib's prebuilt
`OrderHom.lfp` of that operator, i.e. `⨅ {S | T_Π S ≤ S}`, matching the thesis text's
`lfp(T_Π)` on the nose. No hand-rolled fixpoint construction is introduced: Mathlib's
`OrderHom.lfp` (`Mathlib/Order/FixedPoints.lean`) already provides the least-fixpoint
operator on any `CompleteLattice`, and `Set α` already has a `CompleteLattice`
instance regardless of finiteness of `α`. "Reachable iff `p ∈ sat(Π)`" is then simply
membership in `Saturation T_Π`.
-/

namespace Praxis.Corpus.DefSaturation

open Praxis.Corpus.DefGround
open Praxis.Corpus.DefDomain

/-- The immediate-consequence operator for a Datalog-style program built from a
grounded problem: a monotone self-map on sets of ground atoms. Monotone because
deriving new facts from a larger fact set never invalidates previously-derivable
facts (the antecedents needed are still present). -/
abbrev ConsequenceOp (D : LiftedDomain) (Ob : Type) :=
  Set (GroundAtom D Ob) →o Set (GroundAtom D Ob)

/-- `sat(Π) = lfp(T_Π)`: the saturation of the program is the least fixpoint of its
immediate-consequence operator, reusing Mathlib's `OrderHom.lfp` on the
`CompleteLattice` structure of `Set (GroundAtom D Ob)` rather than constructing a
bespoke fixpoint. -/
def Saturation {D : LiftedDomain} {Ob : Type} (T : ConsequenceOp D Ob) :
    Set (GroundAtom D Ob) :=
  OrderHom.lfp T

/-- An atom `p` is reachable (w.r.t. program `Π` with consequence operator `T`) iff
it lies in the saturation `sat(Π)`. -/
def Reachable {D : LiftedDomain} {Ob : Type} (T : ConsequenceOp D Ob)
    (p : GroundAtom D Ob) : Prop :=
  p ∈ Saturation T

end Praxis.Corpus.DefSaturation
