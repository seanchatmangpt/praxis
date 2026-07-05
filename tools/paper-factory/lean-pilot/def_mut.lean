-- def:mut
-- A mutation operator m maps a valid subject s* ∈ ⋂_i I_i to a mutant m(s*)
-- violating a non-empty set of invariants; its correct stage is
-- stg(m) = min{i : m(s*) ∉ I_i}; m is killed by V, written Kill(m) = 1,
-- iff V rejects m(s*).

-- Vocabulary reused verbatim from def:staged (tools/paper-factory/lean-pilot/def_staged.lean),
-- inlined here since this pilot's files are checked standalone (no import graph).

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

-- A staged validator is a finite pipeline of k stages, indexed by Fin k.
structure StagedValidator (S : Type) (k : Nat) where
  stages : Fin k → Stage S

-- The pipeline accepts s iff s satisfies every stage's invariant,
-- i.e. s ∈ ⋂_i I_i.
def StagedValidator.accepts {S : Type} {k : Nat} (V : StagedValidator S k) (s : S) : Prop :=
  ∀ i : Fin k, (V.stages i).invariant s

-- A mutation operator on a staged validator: given a subject known to be
-- accepted by V (i.e. s ∈ ⋂_i I_i), produces a mutant together with a proof
-- that it violates at least one stage's invariant (a non-empty set of
-- invariants is violated).
structure MutationOperator {S : Type} {k : Nat} (V : StagedValidator S k) where
  apply : (s : S) → V.accepts s → S
  violates : ∀ (s : S) (h : V.accepts s), ¬ ∀ i : Fin k, (V.stages i).invariant (apply s h)

-- The correct stage of a mutant application: i is a witness of stg(m) iff
-- the mutant violates stage i's invariant and passes every earlier stage,
-- i.e. i = min{i : m(s*) ∉ I_i}.
def MutationOperator.stg {S : Type} {k : Nat} {V : StagedValidator S k}
    (m : MutationOperator V) (s : S) (h : V.accepts s) : Fin k → Prop :=
  fun i => ¬ (V.stages i).invariant (m.apply s h) ∧
    ∀ j : Fin k, j < i → (V.stages j).invariant (m.apply s h)

-- A mutant is killed by V, i.e. Kill(m) = 1, iff running the pipeline on the
-- mutant yields a reject at some stage.
def MutationOperator.killed {S : Type} {k : Nat} {V : StagedValidator S k}
    (m : MutationOperator V) (s : S) (h : V.accepts s) : Prop :=
  ∃ i : Fin k, (V.stages i).run (m.apply s h) = StageResult.reject
