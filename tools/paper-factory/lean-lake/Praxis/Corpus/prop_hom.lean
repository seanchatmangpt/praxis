import Praxis.Corpus.def_lifecat

/-!
# prop:hom

For stages `X, Y` of `Life`, the hom-set `Life(X, Y)` has exactly one element if `Y` is
reachable from `X` along the quiver, and is empty otherwise.

`Life` is the free path category on the linear quiver
`raw --judge--> val --admit--> admd --receipt--> rcpt` (see `Praxis.Corpus.def_lifecat`).
Reachability is witnessed by a rank function `raw < val < admd < rcpt` along which every
generating arrow increases rank by exactly one; uniqueness of a path between two stages
follows because the quiver is linear, i.e. every stage has at most one incoming generating
arrow, so a path into `Y` is forced to factor through the unique predecessor of `Y`.
-/

namespace Praxis.Corpus

open CategoryTheory LifeObj

/-- Numeric rank of each stage along the linear quiver: `raw < val < admd < rcpt`. -/
def rank : LifeObj → Nat
  | raw => 0
  | val => 1
  | admd => 2
  | rcpt => 3

/-- Each generating arrow (an edge of the underlying quiver, not a path in `Life`) strictly
increases rank by exactly one. -/
theorem rank_hom_succ {X Y : LifeObj} (f : LifeHom X Y) : rank Y = rank X + 1 := by
  cases f <;> rfl

/-- Every morphism of `Life` (i.e. every path in the underlying quiver) is
weakly rank-increasing. -/
theorem rank_le_of_path {X Y : Life} (p : X ⟶ Y) : rank X ≤ rank Y := by
  induction p with
  | nil => exact le_refl _
  | cons _ f ih => have := rank_hom_succ f; omega

/-- The quiver is linear: any two morphisms between the same two stages coincide, since a
non-trivial path into `Y` is forced to factor through `Y`'s unique predecessor. -/
theorem path_unique {X Y : Life} (p q : X ⟶ Y) : p = q := by
  induction p with
  | nil =>
    cases q with
    | nil => rfl
    | cons q' g => exact absurd (rank_le_of_path q') (by have := rank_hom_succ g; omega)
  | cons p' f ih =>
    cases q with
    | nil => exact absurd (rank_le_of_path p') (by have := rank_hom_succ f; omega)
    | cons q' g => cases f <;> cases g <;> exact ih q' ▸ rfl

/-- The hom-set of `Life` between any two stages has exactly one element if the target is
reachable from the source (`rank X ≤ rank Y`), and is empty otherwise. -/
theorem hom_card (X Y : Life) :
    (rank X ≤ rank Y → ∃ f : X ⟶ Y, ∀ g : X ⟶ Y, g = f) ∧
      (¬ rank X ≤ rank Y → IsEmpty (X ⟶ Y)) := by
  refine ⟨fun h => ?_, fun h => ⟨fun p => h (rank_le_of_path p)⟩⟩
  cases X <;> cases Y <;>
    first
      | exact absurd h (by decide)
      | exact ⟨Quiver.Path.nil, fun g => path_unique g _⟩
      | exact ⟨Quiver.Path.nil.cons LifeHom.judge, fun g => path_unique g _⟩
      | exact ⟨(Quiver.Path.nil.cons LifeHom.judge).cons LifeHom.admit, fun g => path_unique g _⟩
      | exact ⟨((Quiver.Path.nil.cons LifeHom.judge).cons LifeHom.admit).cons LifeHom.receipt,
          fun g => path_unique g _⟩
      | exact ⟨Quiver.Path.nil.cons LifeHom.admit, fun g => path_unique g _⟩
      | exact ⟨(Quiver.Path.nil.cons LifeHom.admit).cons LifeHom.receipt, fun g => path_unique g _⟩
      | exact ⟨Quiver.Path.nil.cons LifeHom.receipt, fun g => path_unique g _⟩

end Praxis.Corpus
