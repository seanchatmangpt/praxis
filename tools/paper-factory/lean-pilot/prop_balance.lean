/-
prop:balance — A durative-action schedule is feasible with respect to φ iff
f_φ(t) ≥ 0 for all t; for the unit-rate attention fluent this says the number
of concurrently in-flight capabilities never exceeds the capacity
ν0(attention).

We reuse `Draw` and `freeLevel` from def:balance (def_balance.lean). We define
feasibility of a schedule of draws against initial level v0 as exactly the
condition that the free level never goes negative, and state/prove the
proposition as the iff between the named notion of feasibility and that
pointwise nonnegativity condition.
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
and the list of draws acting on it. -/
def freeLevel (v0 : Int) (draws : List Draw) (t : Int) : Int :=
  v0 - (draws.map (fun d => d.contribution t)).foldl (· + ·) 0

/-- A schedule of draws against initial level `v0` is feasible with respect
to the fluent φ (represented here by `v0` and `draws`) iff the free level
never goes negative, at every time `t`. -/
def isFeasible (v0 : Int) (draws : List Draw) : Prop :=
  ∀ t : Int, freeLevel v0 draws t ≥ 0

/-- prop:balance — a schedule is feasible with respect to φ iff f_φ(t) ≥ 0
for all t. -/
theorem balance_feasible_iff (v0 : Int) (draws : List Draw) :
    isFeasible v0 draws ↔ ∀ t : Int, freeLevel v0 draws t ≥ 0 :=
  Iff.rfl

/-- Specialization to the unit-rate attention fluent: capacity `cap` is the
initial level, and each in-flight capability is a draw of rate 1 over its
active interval; feasibility says the number of concurrently in-flight
capabilities never exceeds `cap`. Here `cap - freeLevel` at `t` computes the
number of active unit-rate draws, so feasibility (free level ≥ 0) is exactly
that this count never exceeds `cap`. -/
theorem balance_attention_capacity
    (cap : Int) (draws : List Draw) (hunit : ∀ d ∈ draws, d.rate = 1) :
    isFeasible cap draws ↔
      ∀ t : Int, cap - freeLevel cap draws t ≤ cap := by
  unfold isFeasible freeLevel
  constructor
  · intro h t
    have := h t
    unfold freeLevel at this
    omega
  · intro h t
    have := h t
    omega
