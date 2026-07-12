import Mathlib.Order.Basic
import Praxis.Corpus.def_powl

/-!
# PROJ-THERMO-3: Entropy Integrator
-/

/-- The `F(S,G)` functional, returning evidence that execution pressure (entropy) S
exceeds the capability gradient G. -/
def F {R : Type} [LT R] (S G : R) : Prop :=
  G < S

/-- A POWL state `W_n` packaging the POWL model alongside its current thermodynamic
execution pressure (entropy) S and capability gradient G. -/
structure PowlState (A : Type) (R : Type) [LT R] where
  model : POWL A
  S : R
  G : R

/-- `W_n` autonomically manufactures a child state `W_{n+1}` only if the
execution pressure exceeds the mathematically defined capability gradient,
as evaluated by the `F(S,G)` functional. -/
inductive Manufactures {A : Type} {R : Type} [LT R] : PowlState A R → PowlState A R → Prop where
  | step (W_n W_n_1 : PowlState A R)
         (h_thermo : F W_n.S W_n.G) : Manufactures W_n W_n_1

/-- Validation that autonomic manufacture STRICTLY requires the pressure constraint. -/
theorem manufacture_requires_pressure {A : Type} {R : Type} [LT R]
    {W_n W_n_1 : PowlState A R}
    (h_manuf : Manufactures W_n W_n_1) :
    F W_n.S W_n.G := by
  cases h_manuf with
  | step h_thermo => exact h_thermo
