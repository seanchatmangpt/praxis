-- def:staged
-- A staged validator is a pipeline V = (V_1,...,V_k) where each stage
-- V_i : S -> {pass} ∪ {reject_i} tests a decidable invariant I_i ⊆ S;
-- the pipeline accepts s iff s ∈ ⋂_i I_i, reporting on rejection the
-- least i with s ∉ I_i. Stage V_i is sound if V_i(s) = reject_i ⇒ s ∉ I_i
-- and complete if s ∉ I_i ⇒ V_i(s) = reject_i.

-- Outcome of a single validation stage on a state space S.
inductive StageResult where
  | pass
  | reject
  deriving DecidableEq, Repr

-- A single stage: a decision procedure paired with the invariant it tests.
structure Stage (S : Type) where
  invariant : S → Prop
  decInvariant : DecidablePred invariant
  run : S → StageResult

-- Soundness: rejecting implies the invariant genuinely fails.
def Stage.sound {S : Type} (V : Stage S) : Prop :=
  ∀ s, V.run s = StageResult.reject → ¬ V.invariant s

-- Completeness: failing the invariant implies the stage rejects.
def Stage.complete {S : Type} (V : Stage S) : Prop :=
  ∀ s, ¬ V.invariant s → V.run s = StageResult.reject

-- A staged validator is a finite pipeline of k stages, indexed by Fin k.
structure StagedValidator (S : Type) (k : Nat) where
  stages : Fin k → Stage S

-- The pipeline accepts s iff s satisfies every stage's invariant,
-- i.e. s ∈ ⋂_i I_i.
def StagedValidator.accepts {S : Type} {k : Nat} (V : StagedValidator S k) (s : S) : Prop :=
  ∀ i : Fin k, (V.stages i).invariant s
