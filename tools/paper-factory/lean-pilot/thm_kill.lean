-- thm:kill
-- For a staged validator V=(V_1,...,V_k) in which every stage is sound and
-- complete, and a mutation operator m with correct stage stg(m):
-- Kill(m)=1 iff V_{stg(m)} rejects m(s*), and the least rejecting stage
-- reported by V equals stg(m).

-- Vocabulary reused verbatim from def:mut and def:staged
-- (tools/paper-factory/lean-pilot/def_mut.lean, def_staged.lean), inlined
-- here since this pilot's files are checked standalone (no import graph).

inductive StageResult where
  | pass
  | reject
  deriving DecidableEq, Repr

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

structure StagedValidator (S : Type) (k : Nat) where
  stages : Fin k → Stage S

def StagedValidator.accepts {S : Type} {k : Nat} (V : StagedValidator S k) (s : S) : Prop :=
  ∀ i : Fin k, (V.stages i).invariant s

structure MutationOperator {S : Type} {k : Nat} (V : StagedValidator S k) where
  apply : (s : S) → V.accepts s → S
  violates : ∀ (s : S) (h : V.accepts s), ¬ ∀ i : Fin k, (V.stages i).invariant (apply s h)

-- i witnesses stg(m) at s iff m(s) violates stage i's invariant and passes
-- every earlier stage, i.e. i = min{i : m(s*) ∉ I_i}.
def MutationOperator.stg {S : Type} {k : Nat} {V : StagedValidator S k}
    (m : MutationOperator V) (s : S) (h : V.accepts s) : Fin k → Prop :=
  fun i => ¬ (V.stages i).invariant (m.apply s h) ∧
    ∀ j : Fin k, j < i → (V.stages j).invariant (m.apply s h)

-- Kill(m) = 1 iff running the pipeline on the mutant yields a reject at
-- some stage.
def MutationOperator.killed {S : Type} {k : Nat} {V : StagedValidator S k}
    (m : MutationOperator V) (s : S) (h : V.accepts s) : Prop :=
  ∃ i : Fin k, (V.stages i).run (m.apply s h) = StageResult.reject

-- thm:kill.
-- Given every stage of V sound and complete, and i a witness of stg(m) at
-- s: Kill(m) = 1 iff V_i rejects m(s*), and any other witness of stg(m)
-- (i.e. any least rejecting stage reported by V) equals i.
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
