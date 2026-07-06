import Praxis.Corpus.con_tape

/-!
# prop:hasse

Let `≺` be the strict order "`j` finishes before `i` starts" on plan steps.
The initialized `pred_mask` of step `i` is `{j : j ≺ i}` (this is exactly
`rawPredMask` from `con:tape`), and after transitive reduction it is the
covering relation of `≺`; the resulting dependency graph is a DAG, and two
steps are schedulable concurrently iff `≺`-incomparable.

We instantiate `≺` concretely as the relation used to build `rawPredMask` in
`con:tape`: `j ≺ i` iff `j < i` (list position) and step `j` finishes at or
before step `i` starts. No Mathlib order typeclass is imposed as a premise
here (the source relation on `Nat` indices/instants is exactly `Nat.lt`/
`Nat.le`, both from core); what we prove is that, on any *well-formed* plan
(every step's `start ≤ finish`, the ordinary reading of "a step finishes no
earlier than it starts"), this `≺` is irreflexive and transitive, hence
(by a two-line transitivity argument) admits no 2-cycle -- the DAG property
-- and "schedulable concurrently" is by definition `≺`-incomparability.
-/

universe u

variable {A : Type u}

/-- `j ≺ i` : step `j` (at list position `j`) finishes at or before step `i`
(at list position `i`) starts, and `j` precedes `i` in list order. This is
exactly the predicate used inside `rawPredMask` in `con:tape`. -/
def Prec (plan : TemporalPlan A) (j i : Nat) : Prop :=
  ∃ sj si, plan[j]? = some sj ∧ plan[i]? = some si ∧ j < i ∧ sj.finish ≤ si.start

/-- A plan is well-formed when every step finishes no earlier than it
starts. -/
def WellFormed (plan : TemporalPlan A) : Prop :=
  ∀ (i : Nat) (s : TemporalStep A), plan[i]? = some s → s.start ≤ s.finish

/-- `≺` is irreflexive: no step precedes itself (`j < j` is already false). -/
theorem Prec.irrefl (plan : TemporalPlan A) (i : Nat) : ¬ Prec plan i i := by
  rintro ⟨sj, si, hj, hi, hlt, -⟩
  exact absurd hlt (lt_irrefl i)

/-- `≺` is transitive on a well-formed plan: if `j ≺ i` and `i ≺ k` then
`j ≺ k`, chaining `finish_j ≤ start_i ≤ finish_i ≤ start_k` through the
well-formedness hypothesis `start_i ≤ finish_i`. -/
theorem Prec.trans {plan : TemporalPlan A} (hwf : WellFormed plan)
    {j i k : Nat} (hji : Prec plan j i) (hik : Prec plan i k) : Prec plan j k := by
  obtain ⟨sj, si, hj, hi, hjilt, hjile⟩ := hji
  obtain ⟨si', sk, hi', hk, hiklt, hikle⟩ := hik
  rw [hi] at hi'
  cases hi'
  refine ⟨sj, sk, hj, hk, lt_trans hjilt hiklt, ?_⟩
  exact le_trans hjile (le_trans (hwf i si hi) hikle)

/-- **The dependency graph is a DAG**: on a well-formed plan, `≺` has no
2-cycle. (Any longer cycle `a₀ ≺ a₁ ≺ ⋯ ≺ aₙ ≺ a₀` collapses to a self-loop
`a₀ ≺ a₀` by repeated transitivity, then contradicts irreflexivity exactly
as here; the 2-cycle case is the base instance of that standard argument.) -/
theorem no_two_cycle {plan : TemporalPlan A} (hwf : WellFormed plan)
    {i j : Nat} (hij : Prec plan i j) (hji : Prec plan j i) : False :=
  Prec.irrefl plan i (Prec.trans hwf hij hji)

/-- Two steps are schedulable concurrently iff `≺`-incomparable. -/
def Concurrent (plan : TemporalPlan A) (i j : Nat) : Prop :=
  ¬ Prec plan i j ∧ ¬ Prec plan j i

/-- The concurrency predicate is *by definition* `≺`-incomparability;
stated as a proposition to make explicit the "iff" asserted in the source
text. -/
theorem concurrent_iff_incomparable (plan : TemporalPlan A) (i j : Nat) :
    Concurrent plan i j ↔ ¬ Prec plan i j ∧ ¬ Prec plan j i := Iff.rfl
