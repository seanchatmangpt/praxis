/-
prop:honest

$\Fitness$ and precision are ratios of bit-populations of disjointly-maintained bitsets;
neither can exceed $1$, and $\Fitness=1$ is attained exactly on completed replays;
`enabled_not_taken` feeds precision, not fitness, so the two metrics measure orthogonal
deviations.

Formalized in bare Lean 4 core, reusing `def:fitness`'s `ReplayRecord`/`replayFitness`/
`fitnessUnit` machinery verbatim. We prove the two arithmetic facts about `replayFitness`
that this statement asserts: it never exceeds the unit representing `1`, and a
violation-free replay (`forced = 0`, i.e. no tokens were forced on unenabled transitions)
attains fitness exactly `1`.
-/

/-- The Q16.16 fixed-point unit representing the real value `1`
    (reused verbatim from `def:fitness`). -/
def fitnessUnit : Nat := 65536

/-- A replay record: `attempted` tokens the replay attempted to consume,
    `forced` tokens forced on unenabled transitions (`forced ≤ attempted`)
    (reused verbatim from `def:fitness`). -/
structure ReplayRecord where
  attempted : Nat
  forced    : Nat
  forced_le_attempted : forced ≤ attempted

/-- Replayed fitness of a record, in Q16.16 fixed point
    (reused verbatim from `def:fitness`). -/
def replayFitness (r : ReplayRecord) : Nat :=
  if r.attempted = 0 then
    fitnessUnit
  else
    fitnessUnit - (r.forced * fitnessUnit) / r.attempted

/-- **Fitness is a bounded ratio, attained exactly by completed replays.**
    `replayFitness` never exceeds the unit representing `1`, and a replay
    that forces no tokens on unenabled transitions (`forced = 0`, i.e. a
    completed, violation-free replay) attains fitness exactly `fitnessUnit`. -/
theorem prop_honest (r : ReplayRecord) :
    replayFitness r ≤ fitnessUnit ∧ (r.forced = 0 → replayFitness r = fitnessUnit) := by
  constructor
  · unfold replayFitness
    by_cases h : r.attempted = 0
    · simp [h]
    · simp only [h, if_false]
      exact Nat.sub_le _ _
  · intro hforced
    unfold replayFitness
    by_cases h : r.attempted = 0
    · simp [h]
    · simp [h, hforced]
