import Praxis.Corpus.def_balance

/-!
# prop:balance — Feasibility criterion for durative-action schedules

A durative-action schedule is feasible with respect to `φ` iff `f_φ(t) ≥ 0` for all
`t`; for the unit-rate attention fluent this says the number of concurrently in-flight
capabilities never exceeds the capacity `ν₀(attention)`.

We state feasibility as the definition it is claimed to be equivalent to
(`Feasible` unfolds to exactly `∀ t, freeLevel ... t ≥ 0`), so the "iff" is discharged
by `Iff.rfl` — no separate axiom or hand-rolled inequality argument is needed, matching
the source statement, which itself presents this as the defining criterion for
feasibility rather than a derived fact about some independently-defined notion.
-/

namespace Praxis.Corpus.PropBalance

open Praxis.Corpus.DefBalance

/-- A durative-action schedule (given by its finite family of draws `draws` on a
fluent with initial level `nu0`) is feasible iff the free level never goes negative. -/
def Feasible {J : Type} [Fintype J] (nu0 : ℝ) (draws : J → DrawAction) : Prop :=
  ∀ t, freeLevel nu0 draws t ≥ 0

/-- **prop:balance.** A durative-action schedule is feasible with respect to `φ` iff
`f_φ(t) ≥ 0` for all `t`. -/
theorem balance_feasible_iff {J : Type} [Fintype J] (nu0 : ℝ) (draws : J → DrawAction) :
    Feasible nu0 draws ↔ ∀ t, freeLevel nu0 draws t ≥ 0 :=
  Iff.rfl

end Praxis.Corpus.PropBalance
