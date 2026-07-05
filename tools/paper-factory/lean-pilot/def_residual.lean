/- def:residual
   For environmental observations `O` and target artifact `A`, the measurement `μop(O)`
   returns a residual vector `R ∈ ℝ^k` with `r_i = measured_i - midpoint(target_i)`;
   reconciliation selects the dominant dimension `argmax_i |r_i|` and applies a repair
   operator subject to `RepairBand` limits.

   Bare Lean 4 core (no mathlib): reals are modeled by `Float`, and an `R^k` vector by
   `Fin k → Float`. -/

/-- A residual vector over `k` dimensions. -/
def Residual (k : Nat) := Fin k → Float

/-- A target band for a single dimension, given by its low/high bounds. -/
structure TargetBand where
  low  : Float
  high : Float

/-- The midpoint of a target band. -/
def TargetBand.midpoint (t : TargetBand) : Float :=
  (t.low + t.high) / 2

/-- Compute the residual vector: `r_i = measured_i - midpoint(target_i)`. -/
def residual {k : Nat} (measured : Fin k → Float) (target : Fin k → TargetBand) :
    Residual k :=
  fun i => measured i - (target i).midpoint

/-- Absolute value on `Float`. -/
def Residual.absAt {k : Nat} (r : Residual k) (i : Fin k) : Float :=
  if r i < 0 then -(r i) else r i

/-- A repair band bounding how large a repair may be. -/
structure RepairBand where
  limit : Float

/-- Whether a proposed repair magnitude stays within the repair band. -/
def RepairBand.admits (b : RepairBand) (magnitude : Float) : Prop :=
  magnitude ≤ b.limit

/-- Fold over `Fin (n+1)` picking, at each step, whichever of the current best index
    and the next index has larger `|r_i|` (ties keep the earlier index). This underlies
    the dominant-dimension selection `argmax_i |r_i|`. -/
def dominantDimAux {k : Nat} (r : Residual k) :
    (n : Nat) → n < k → Fin k → Fin k
  | 0, _, best => best
  | Nat.succ m, h, best =>
      let i : Fin k := ⟨Nat.succ m, h⟩
      let best' := if r.absAt best < r.absAt i then i else best
      dominantDimAux r m (Nat.lt_of_succ_lt h) best'

/-- The dominant dimension `argmax_i |r_i|` of a nonempty residual vector. -/
def dominantDim {k : Nat} (h : 0 < k) (r : Residual k) : Fin k :=
  let last : Nat := k - 1
  have hlast : last < k := Nat.sub_lt h Nat.one_pos
  dominantDimAux r last hlast ⟨last, hlast⟩
