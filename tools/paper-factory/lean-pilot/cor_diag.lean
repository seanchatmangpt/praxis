-- cor:diag
-- Under the Mutant Kill Theorem, the index of the killing stage certifies
-- which law-bearing invariant a mutation broke; a non-empty surviving set
-- that is not equivalent-mutants is a proof that the frame under-commits,
-- the Faithful Projection converse.
--
-- Formalized corollary of thm:kill: the killing-stage witness is unique
-- (the "diagonal" index stg(m) is a genuine function, not merely a
-- relation) — any two witnessing stages for the same mutation coincide.
-- This is exactly the uniqueness half of kill_correct, restated as its
-- own corollary so the certifying index can be treated as a well-defined
-- diagnostic (i.e. "which invariant broke") rather than an ambiguous set.

-- Vocabulary reused verbatim from thm:kill
-- (tools/paper-factory/lean-pilot/thm_kill.lean), inlined here since this
-- pilot's files are checked standalone (no import graph).

inductive StageResult where
  | pass
  | reject
  deriving DecidableEq, Repr

structure Stage (S : Type) where
  invariant : S → Prop
  decInvariant : DecidablePred invariant
  run : S → StageResult

def Stage.sound {S : Type} (V : Stage S) : Prop :=
  ∀ s, V.run s = StageResult.reject → ¬ V.invariant s

def Stage.complete {S : Type} (V : Stage S) : Prop :=
  ∀ s, ¬ V.invariant s → V.run s = StageResult.reject

structure StagedValidator (S : Type) (k : Nat) where
  stages : Fin k → Stage S

def StagedValidator.accepts {S : Type} {k : Nat} (V : StagedValidator S k) (s : S) : Prop :=
  ∀ i : Fin k, (V.stages i).invariant s

structure MutationOperator {S : Type} {k : Nat} (V : StagedValidator S k) where
  apply : (s : S) → V.accepts s → S
  violates : ∀ (s : S) (h : V.accepts s), ¬ ∀ i : Fin k, (V.stages i).invariant (apply s h)

def MutationOperator.stg {S : Type} {k : Nat} {V : StagedValidator S k}
    (m : MutationOperator V) (s : S) (h : V.accepts s) : Fin k → Prop :=
  fun i => ¬ (V.stages i).invariant (m.apply s h) ∧
    ∀ j : Fin k, j < i → (V.stages j).invariant (m.apply s h)

def MutationOperator.killed {S : Type} {k : Nat} {V : StagedValidator S k}
    (m : MutationOperator V) (s : S) (h : V.accepts s) : Prop :=
  ∃ i : Fin k, (V.stages i).run (m.apply s h) = StageResult.reject

theorem kill_correct {S : Type} {k : Nat} {V : StagedValidator S k}
    (hsc : ∀ i, (V.stages i).sound ∧ (V.stages i).complete)
    (m : MutationOperator V) (s : S) (h : V.accepts s) (i : Fin k)
    (hstg : m.stg s h i) :
    (m.killed s h ↔ (V.stages i).run (m.apply s h) = StageResult.reject) ∧
    (∀ j : Fin k, m.stg s h j → j = i) := by
  obtain ⟨hni, hprior⟩ := hstg
  have hrun : (V.stages i).run (m.apply s h) = StageResult.reject := (hsc i).2 _ hni
  refine ⟨⟨fun _ => hrun, fun _ => ⟨i, hrun⟩⟩, ?_⟩
  intro j hstgj
  obtain ⟨hnj, hpriorj⟩ := hstgj
  rcases Nat.lt_trichotomy j.val i.val with hji | hji | hji
  · exact absurd (hprior j hji) hnj
  · exact Fin.ext hji
  · exact absurd (hpriorj i hji) hni

-- cor:diag: the killing-stage witness of a mutation, when it exists, is
-- unique (the diagonal index that certifies which invariant broke is
-- well-defined).
theorem diag_witness_unique {S : Type} {k : Nat} {V : StagedValidator S k}
    (hsc : ∀ i, (V.stages i).sound ∧ (V.stages i).complete)
    (m : MutationOperator V) (s : S) (h : V.accepts s)
    (i j : Fin k) (hi : m.stg s h i) (hj : m.stg s h j) :
    i = j :=
  ((kill_correct hsc m s h i hi).2 j hj).symm
