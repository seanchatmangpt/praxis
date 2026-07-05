/-
def:balance — A durative action j draws on a fluent φ if its effect decreases φ
by r_j at start and increases it by r_j at end for rate r_j ≥ 0, holding r_j
units over [s_j, e_j); the free level of φ at time t is
  f_φ(t) = ν0(φ) − Σ_{j : t ∈ [s_j, e_j)} r_j.

We model this abstractly in bare Lean 4 core (no mathlib). Time, levels, and
rates are represented by `Int` (a concrete carrier standing in for an ordered
field), a "draw" is a structure bundling a start, an end, and a nonnegative
rate, and the free level at a given time t is defined as the initial level
minus the sum of rates of all draws whose interval [s_j, e_j) contains t.
-/

structure Draw where
  start  : Int
  stop   : Int
  rate   : Int
  rate_nonneg : 0 ≤ rate

/-- Whether time `t` lies in the half-open interval `[d.start, d.stop)`. -/
def Draw.active (d : Draw) (t : Int) : Bool :=
  decide (d.start ≤ t) && decide (t < d.stop)

/-- The rate contributed by a draw at time `t`: its rate if active, else 0. -/
def Draw.contribution (d : Draw) (t : Int) : Int :=
  if d.active t then d.rate else 0

/-- The free level of a fluent at time `t`, given its initial level `v0`
and the list of draws acting on it: the initial level minus the sum of
the contributions of all draws active at `t`. -/
def freeLevel (v0 : Int) (draws : List Draw) (t : Int) : Int :=
  v0 - (draws.map (fun d => d.contribution t)).foldl (· + ·) 0
