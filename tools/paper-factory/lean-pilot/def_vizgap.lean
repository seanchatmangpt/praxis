/- def:vizgap
   A visual gap report is the output of `measure_gap`: for each reconcilable dimension
   `i ∈ {1,...,k}`, the residual `r_i = measured_i - midpoint(target_i)` and the dominant
   dimension `i* = argmax_i |r_i|` together with a rendered diff block; the report has
   `k` residual values and one dominant index, size `O(k)` independent of the interior.

   Bare Lean 4 core (no mathlib). Builds on def:residual (`Residual`, `dominantDim`). -/

def Residual (k : Nat) := Fin k → Float

structure TargetBand where
  low  : Float
  high : Float

def TargetBand.midpoint (t : TargetBand) : Float :=
  (t.low + t.high) / 2

def residual {k : Nat} (measured : Fin k → Float) (target : Fin k → TargetBand) :
    Residual k :=
  fun i => measured i - (target i).midpoint

def Residual.absAt {k : Nat} (r : Residual k) (i : Fin k) : Float :=
  if r i < 0 then -(r i) else r i

def dominantDimAux {k : Nat} (r : Residual k) :
    (n : Nat) → n < k → Fin k → Fin k
  | 0, _, best => best
  | Nat.succ m, h, best =>
      let i : Fin k := ⟨Nat.succ m, h⟩
      let best' := if r.absAt best < r.absAt i then i else best
      dominantDimAux r m (Nat.lt_of_succ_lt h) best'

def dominantDim {k : Nat} (h : 0 < k) (r : Residual k) : Fin k :=
  let last : Nat := k - 1
  have hlast : last < k := Nat.sub_lt h Nat.one_pos
  dominantDimAux r last hlast ⟨last, hlast⟩

/-- A rendered diff block accompanying the report (opaque payload, e.g. text/markup). -/
def DiffBlock := String

/-- A visual gap report: `k` residual values, one dominant index, and a rendered
    diff block. Size `O(k)` independent of the interior (a `Residual k` plus a
    single `Fin k` plus one `DiffBlock`). -/
structure VizGapReport (k : Nat) where
  residuals   : Residual k
  dominant    : Fin k
  diff        : DiffBlock

/-- `measure_gap`: given measurements and target bands for a nonempty set of `k`
    reconcilable dimensions, together with a way to render the diff, produce the
    visual gap report. -/
def measureGap {k : Nat} (h : 0 < k) (measured : Fin k → Float)
    (target : Fin k → TargetBand) (render : Residual k → Fin k → DiffBlock) :
    VizGapReport k :=
  let r := residual measured target
  let d := dominantDim h r
  { residuals := r, dominant := d, diff := render r d }
