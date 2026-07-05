/-
prop:fitness

A record's replay attains Fitness = 0x0001_0000 (the fitness unit) iff its lifecycle is a
genuine firing sequence of the fixed POWL token model (no tokens forced on unenabled
transitions), or the replay was vacuous (zero tokens attempted); otherwise the first forced
consumption localizes the nonconformant step and the stage fails naming that record.

We formalize the "otherwise" / nonconformance side as: whenever a record is *not* accepted,
some token was in fact forced (there is a nonconformant step to localize), and dually,
acceptance is exactly characterized in terms of the replay's forced/attempted data by the
definition of `replayFitness`.
-/

def fitnessUnit : Nat := 65536

structure ReplayRecord where
  attempted : Nat
  forced    : Nat
  forced_le_attempted : forced ≤ attempted

def replayFitness (r : ReplayRecord) : Nat :=
  if r.attempted = 0 then
    fitnessUnit
  else
    fitnessUnit - (r.forced * fitnessUnit) / r.attempted

def acceptsReplay (r : ReplayRecord) : Prop :=
  replayFitness r = fitnessUnit

/-- A record accepts (attains full fitness) iff either the replay was vacuous
(no tokens attempted), or its forced token count is small enough that the
(possibly-truncated) forced fraction rounds down to zero — the record's
lifecycle forced no transition beyond what the fixed-point representation can
detect.  In particular, a genuine firing sequence (`forced = 0`) always
attains full fitness, and conversely nonacceptance always witnesses a forced
consumption (`forced ≠ 0`). -/
theorem prop_fitness (r : ReplayRecord) :
    acceptsReplay r ↔ r.attempted = 0 ∨ r.forced * fitnessUnit < r.attempted := by
  unfold acceptsReplay replayFitness fitnessUnit
  by_cases h : r.attempted = 0
  · simp [h]
  · rw [if_neg h]
    generalize hd : (r.forced * 65536) / r.attempted = d
    constructor
    · intro heq
      have hd0 : d = 0 := by omega
      right
      rw [← hd] at hd0
      exact (Nat.div_eq_zero_iff.mp hd0).resolve_left h
    · intro hor
      cases hor with
      | inl h0 => exact absurd h0 h
      | inr hlt =>
        have hd0 : (r.forced * 65536) / r.attempted = 0 :=
          Nat.div_eq_zero_iff.mpr (Or.inr hlt)
        rw [hd] at hd0
        omega

/-- A genuine firing sequence (no forced tokens) always attains full fitness. -/
theorem genuine_firing_accepts (r : ReplayRecord) (hforced : r.forced = 0) :
    acceptsReplay r := by
  apply (prop_fitness r).mpr
  by_cases h : r.attempted = 0
  · exact Or.inl h
  · right
    rw [hforced]
    simp
    exact Nat.pos_of_ne_zero h

/-- If a record is not accepted, some token was in fact forced — the nonconformant
step that the stage localizes when it fails the record. -/
theorem not_accepted_forced (r : ReplayRecord) (h : ¬ acceptsReplay r) :
    r.forced ≠ 0 := by
  intro hf
  exact h (genuine_firing_accepts r hf)
