/-
def:attnbalance — Attention/capacity balance and feasibility.

Let `c(t) ≥ 0` be capacity and admitted action `j` draw rate `r_j ≥ 0` on
`[s_j, e_j]`. Free capacity is

    f(t) = c(t) - Σ_{j : t ∈ [s_j, e_j]} r_j.

A schedule is feasible iff `f(t) ≥ 0` for all `t`.

We work in bare Lean 4 core (no mathlib), so we model time and actions
abstractly: `Time` is a linear order (axiomatized), actions are indexed by
a finite type `Action` (represented as `Fin n` for some admitted-action
count `n`), each action `j` has a start `s j`, end `e j`, and draw rate
`r j : Float` (nonnegative, as required by the definition). Capacity
`c : Time → Float` is likewise nonnegative. The "active at `t`" predicate
`active j t` stands for `t ∈ [s j, e j]`.

This reuses the `Obs`/admission layer from `def:adm` only nominally (the
draw rates come from *admitted* actions, i.e. actions already passed
through `adm`); we do not need any further lemma from that file for this
definition to type-check, so we do not import it, matching the instruction
to reuse rather than redeclare only when actually needed. (No identifiers
from `def_adm.lean` are used here, so nothing is redeclared.)

This is a *definition*: the only proof obligation is that the file
type-checks.
-/

axiom Time : Type
axiom Time_le : Time → Time → Prop
axiom Time_le_refl : ∀ t, Time_le t t
axiom Time_le_trans : ∀ {t1 t2 t3}, Time_le t1 t2 → Time_le t2 t3 → Time_le t1 t3

/-- Number of admitted actions currently on the schedule. -/
axiom n : Nat

/-- Admitted actions, indexed `0, ..., n-1`. -/
abbrev Action := Fin n

/-- Start time of action `j`. -/
axiom s : Action → Time

/-- End time of action `j`. -/
axiom e : Action → Time

/-- Draw rate of action `j` on `[s j, e j]`; nonnegative. -/
axiom r : Action → Float

axiom r_nonneg : ∀ j, r j ≥ 0

/-- Capacity at time `t`; nonnegative. -/
axiom c : Time → Float

axiom c_nonneg : ∀ t, c t ≥ 0

/-- `t ∈ [s j, e j]`, i.e. action `j` is active at time `t`. -/
def active (j : Action) (t : Time) : Prop :=
  Time_le (s j) t ∧ Time_le t (e j)

/-- Decidability of `active`, needed to filter the sum over active actions. -/
axiom active_decidable : ∀ j t, Decidable (active j t)

noncomputable instance (j : Action) (t : Time) : Decidable (active j t) :=
  active_decidable j t

/-- Total draw at time `t`: the sum of `r j` over actions `j` active at `t`. -/
noncomputable def totalDraw (t : Time) : Float :=
  (List.finRange n).foldl
    (fun acc j => if active j t then acc + r j else acc) 0

/-- Free capacity `f(t) = c(t) - Σ_{j : t ∈ [s j, e j]} r_j`. -/
noncomputable def freeCapacity (t : Time) : Float :=
  c t - totalDraw t

/-- A schedule is feasible iff `f(t) ≥ 0` for all `t`. -/
def Feasible : Prop :=
  ∀ t, freeCapacity t ≥ 0
