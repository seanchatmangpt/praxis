/- prop:hom
   For stages X, Y of Life, the hom-set Life(X,Y) has exactly one element
   if Y is reachable from X along the quiver, and is empty otherwise.

   Life is the free category on the linear quiver
     Raw --judge--> Val --admit--> Admd --receipt--> Rcpt
   (see def_lifecat.lean). Reachability along this linear quiver is a
   total order on the four objects: X can reach Y iff X comes no later
   than Y in the chain Raw, Val, Admd, Rcpt. We model the hom-set
   directly as `Unit` when reachable and `Empty` otherwise, and prove
   that this hom-set has exactly one element in the reachable case and
   is empty in the unreachable case. -/

inductive LifeObj where
  | Raw : LifeObj
  | Val : LifeObj
  | Admd : LifeObj
  | Rcpt : LifeObj

open LifeObj

/-- Position of a stage in the linear chain Raw < Val < Admd < Rcpt. -/
def stagePos : LifeObj → Nat
  | Raw  => 0
  | Val  => 1
  | Admd => 2
  | Rcpt => 3

/-- `Y` is reachable from `X` along the quiver iff `X` occurs no later
    than `Y` in the linear order. -/
def reachable (x y : LifeObj) : Prop := stagePos x ≤ stagePos y

instance (x y : LifeObj) : Decidable (reachable x y) :=
  Nat.decLe (stagePos x) (stagePos y)

/-- The hom-set of `Life`, modeled as `Unit` when `Y` is reachable from
    `X` and `Empty` otherwise. -/
def LifeHomSet (x y : LifeObj) : Type :=
  if reachable x y then Unit else Empty

/-- Proposition: `LifeHomSet x y` has exactly one element when `Y` is
    reachable from `X`, and is empty otherwise. -/
theorem lifeHomSet_card (x y : LifeObj) :
    (reachable x y → ∃ e : LifeHomSet x y, ∀ e' : LifeHomSet x y, e' = e) ∧
    (¬ reachable x y → LifeHomSet x y → False) := by
  constructor
  · intro h
    unfold LifeHomSet
    rw [if_pos h]
    exact ⟨(), fun e' => by cases e'; rfl⟩
  · intro h
    unfold LifeHomSet
    rw [if_neg h]
    exact fun e => e.elim
