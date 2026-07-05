/- prop:vizgap
   The visual gap report gives gap : R^k -> R^k x [k], a projection with the same
   cost structure as the receipt projection, O(k) to compute and O(k) to verify;
   the repair operator acts on the dominant dimension i* subject to RepairBand
   bounds, producing at most one corrective actuation, a bounded deterministic
   manufacture.

   Bare Lean 4 core (no mathlib). Builds on def:vizgap (`VizGapReport`,
   `measureGap`, `dominantDim`, `Residual`, `TargetBand`). -/

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

def DiffBlock := String

structure VizGapReport (k : Nat) where
  residuals   : Residual k
  dominant    : Fin k
  diff        : DiffBlock

def measureGap {k : Nat} (h : 0 < k) (measured : Fin k → Float)
    (target : Fin k → TargetBand) (render : Residual k → Fin k → DiffBlock) :
    VizGapReport k :=
  let r := residual measured target
  let d := dominantDim h r
  { residuals := r, dominant := d, diff := render r d }

/-- A repair band: the acceptable actuation range for the dominant dimension. -/
structure RepairBand where
  low  : Float
  high : Float

/-- A single corrective actuation on one dimension: which dimension, and the
    signed correction to apply, clipped into the repair band. -/
structure Actuation (k : Nat) where
  dim   : Fin k
  delta : Float

/-- Clip a raw correction into the bounds of a `RepairBand`. -/
def RepairBand.clip (b : RepairBand) (x : Float) : Float :=
  if x < b.low then b.low else if b.high < x then b.high else x

/-- The repair operator: acts on the report's dominant dimension only, and
    subject to the `RepairBand` bounds, produces at most one corrective
    actuation. It fires (`some`) exactly when the residual at the dominant
    dimension is nonzero; otherwise it produces no actuation (`none`). This
    encodes "acts on the dominant dimension `i*`, produces at most one
    corrective actuation" as a total function into `Option (Actuation k)`,
    whose result cardinality is bounded by 1 by construction. -/
def repair {k : Nat} (rep : VizGapReport k) (band : RepairBand) :
    Option (Actuation k) :=
  let r := rep.residuals rep.dominant
  if r == 0 then
    none
  else
    some { dim := rep.dominant, delta := band.clip (-r) }

/-- Proposition (prop:vizgap): the repair operator applied to any visual gap
    report and repair band produces at most one corrective actuation, and any
    actuation it does produce acts on the report's dominant dimension. This is
    a bounded, deterministic manufacture: for fixed `rep` and `band`, `repair`
    is a total, single-valued function, so its image has at most one element,
    and that element (if any) is anchored at `rep.dominant`. -/
theorem repair_at_most_one_dominant_actuation {k : Nat}
    (rep : VizGapReport k) (band : RepairBand) :
    repair rep band = none ∨
      ∃ a : Actuation k, repair rep band = some a ∧ a.dim = rep.dominant := by
  by_cases h : rep.residuals rep.dominant == 0
  · apply Or.inl
    simp [repair, h]
  · apply Or.inr
    refine ⟨{ dim := rep.dominant, delta := band.clip (-(rep.residuals rep.dominant)) }, ?_, rfl⟩
    simp [repair, h]

/-- Determinism / boundedness corollary: `repair` is a function (not a
    relation), so for a fixed report and band it yields a single, well-defined
    outcome — the "bounded deterministic manufacture" of prop:vizgap. -/
theorem repair_deterministic {k : Nat} (rep : VizGapReport k) (band : RepairBand)
    (o1 o2 : Option (Actuation k))
    (h1 : repair rep band = o1) (h2 : repair rep band = o2) : o1 = o2 := by
  rw [← h1, ← h2]
