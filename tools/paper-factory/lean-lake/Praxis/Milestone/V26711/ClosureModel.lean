/-!
# PROJ-769 / PRD v26.7.11 §9 — Parent-Child Closure: Shared Model

Shared infrastructure for targets 4, 5, and 6 of the 9 declared Lean/Lake
formalization targets at `PRD.md:1035-1043` (parent closure for `all_required`,
parent closure for `quorum`, idempotent duplicate-result transition) — not itself
one of the 9 targets.

## Real correspondence

Models `crates/praxis-graphlaw/src/chatman/closure.rs`'s `ChildCompletionState`
(`Open`/`Observed`/`Admitted`) and `RecursiveSocketClosure`'s per-child state map,
restricted to the three ingredients `ParentClosureAllRequired.lean`,
`ParentClosureQuorum.lean`, and `IdempotentDuplicateResult.lean` need: the
three-state completion lattice, a pointwise "admission never downgrades" order on
it, and a generic per-child state update.

No axioms: `ChildState` is a plain 3-constructor inductive type; `ChildState.le` and
`updateAt` are plain pattern-matching/`if`-based data operations.
-/

/-- `ChildCompletionState` (`closure.rs`): a child's completion state under a
declared closure law. -/
inductive ChildState where
  | Open
  | Observed
  | Admitted
deriving DecidableEq, Repr

/-- `TerminalAdmitted(c)` (PRD §9): true only for `Admitted` — an `Observed` child is
"observation until admitted" (PRD §9 line 525), never terminal. -/
def ChildState.terminalAdmitted : ChildState → Bool
  | .Admitted => true
  | _ => false

/-- The natural "admission never downgrades" order on `ChildState`:
`Open ≤ Observed ≤ Admitted`. Matches `closure.rs`'s `observe`/`admit`, which only
ever move a child forward along this chain, never backward. -/
def ChildState.le : ChildState → ChildState → Prop
  | .Open, _ => True
  | .Observed, .Open => False
  | .Observed, _ => True
  | .Admitted, .Admitted => True
  | .Admitted, _ => False

instance : LE ChildState := ⟨ChildState.le⟩

/-- Pointwise state upgrade over a child-indexed state function: `s ≤ s'` iff every
child's state only moves forward (or stays put) under `ChildState.le`. -/
def statePointwiseLe {ι : Type} (s s' : ι → ChildState) : Prop :=
  ∀ c, (s c).le (s' c)

/-- Updates the state at exactly one child index `c` by applying `f` to its current
state, leaving every other child's state unchanged — the generic shape of
`closure.rs`'s `observe`/`admit` (each is `updateAt` with a specific `f`). -/
def updateAt {ι : Type} [DecidableEq ι] (s : ι → ChildState) (c : ι)
    (f : ChildState → ChildState) : ι → ChildState :=
  fun x => if x = c then f (s x) else s x

@[simp] theorem updateAt_same {ι : Type} [DecidableEq ι] (s : ι → ChildState) (c : ι)
    (f : ChildState → ChildState) : updateAt s c f c = f (s c) := by
  simp [updateAt]

@[simp] theorem updateAt_other {ι : Type} [DecidableEq ι] (s : ι → ChildState)
    (c x : ι) (f : ChildState → ChildState) (h : x ≠ c) : updateAt s c f x = s x := by
  simp [updateAt, h]
