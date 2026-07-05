/-
def:fitness

Replay fitness is `Fitness = 1 - (tokens forced on unenabled transitions) / (tokens the replay attempted) ∈ [0,1]`,
represented as Q16.16 with unit `0x0001_0000 = 65536`; the `token_replay` stage accepts a record iff its
replayed fitness equals that unit.

We model a replay record by the number of tokens attempted and the number of tokens forced on
unenabled transitions (forced ≤ attempted, and attempted convention: 0 attempted tokens is a
vacuous replay of full fitness). Fitness is represented in Q16.16 fixed point, i.e. as a natural
number where `unit = 65536` denotes the value `1`.
-/

/-- The Q16.16 fixed-point unit representing the real value `1`. -/
def fitnessUnit : Nat := 65536

/-- A replay record: `attempted` tokens the replay attempted to consume,
`forced` tokens forced on unenabled transitions (`forced ≤ attempted`). -/
structure ReplayRecord where
  attempted : Nat
  forced    : Nat
  forced_le_attempted : forced ≤ attempted

/-- Replayed fitness of a record, in Q16.16 fixed point.
    `Fitness = 1 - forced / attempted`, scaled by `fitnessUnit`.
    When `attempted = 0` the replay is vacuous and fitness is defined to be the unit (full fitness). -/
def replayFitness (r : ReplayRecord) : Nat :=
  if r.attempted = 0 then
    fitnessUnit
  else
    fitnessUnit - (r.forced * fitnessUnit) / r.attempted

/-- The `token_replay` stage accepts a record iff its replayed fitness equals the fitness unit
    (i.e. fitness `= 1`, no tokens were forced). -/
def acceptsReplay (r : ReplayRecord) : Prop :=
  replayFitness r = fitnessUnit
