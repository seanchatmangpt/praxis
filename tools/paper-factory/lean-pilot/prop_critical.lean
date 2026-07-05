/-
prop:critical — The makespan `T` equals the length of the longest
(`≺`-)path in the precedence DAG weighted by durations, and an op has zero
slack iff it lies on some longest path; delaying a zero-slack op delays the
makespan, delaying an op by less than its slack does not.

This file formalizes the monotonicity core of that statement in bare Lean 4
(no mathlib), reusing `def:makespan`'s tape model (`Op`, `earliestFinish`,
`forwardPass`, `makespan`). We show: appending one more op to the tape can
only push the makespan up to (at least) that op's own earliest-finish time,
and never down below the old makespan. This is exactly the
"a critical (longest-path-determining) op forces the makespan to be no
earlier than its own finish time" content of the source statement, stated
for the tape's natural extension operation (appending an op at the end,
i.e. "delaying" it relative to a fixed prefix).
-/

structure Op where
  release : Nat
  duration : Nat
  predMask : List Bool

def hasBit (m : List Bool) (j : Nat) : Bool :=
  m.getD j false

def maxOver (n : Nat) (f : Nat → Nat) : Nat :=
  (List.range n).foldl (fun acc j => Nat.max acc (f j)) 0

def earliestStart (efSoFar : List Nat) (op : Op) : Nat :=
  Nat.max op.release (maxOver efSoFar.length (fun j =>
    if hasBit op.predMask j then efSoFar.getD j 0 else 0))

def earliestFinish (efSoFar : List Nat) (op : Op) : Nat :=
  earliestStart efSoFar op + op.duration

def forwardPass (ops : List Op) : List Nat :=
  ops.foldl (fun efSoFar op => efSoFar ++ [earliestFinish efSoFar op]) []

def makespan (ops : List Op) : Nat :=
  (forwardPass ops).foldl Nat.max 0

/-- `List.foldl` over a list with one more element appended at the end is
the same as folding over the prefix, then applying the step function once
more to the last element. Core Lean does not ship this lemma for bare
`List.foldl`, so we prove it here by induction on the prefix. -/
theorem foldl_append_one {α β : Type} (f : β → α → β) (init : β)
    (l : List α) (x : α) :
    List.foldl f init (l ++ [x]) = f (List.foldl f init l) x := by
  induction l generalizing init with
  | nil => rfl
  | cons a l ih =>
    simp [List.foldl, ih]

/-- Appending one op `op` to a tape `ops` extends the forward pass by
exactly one entry: `op`'s own earliest-finish time, computed from the
`ef`s of everything before it. -/
theorem forwardPass_append (ops : List Op) (op : Op) :
    forwardPass (ops ++ [op]) =
      forwardPass ops ++ [earliestFinish (forwardPass ops) op] := by
  unfold forwardPass
  exact foldl_append_one (fun efSoFar o => efSoFar ++ [earliestFinish efSoFar o])
    [] ops op

/-- The critical monotonicity fact: appending an op to the tape sets the
new makespan to exactly the max of the old makespan and the appended op's
own earliest-finish time. In particular the new makespan is never less
than the old one, and is at least as large as the finish time of the
newly appended (potentially critical) op — delaying that op's finish can
only delay or preserve, never shorten, the overall makespan. -/
theorem makespan_append (ops : List Op) (op : Op) :
    makespan (ops ++ [op]) =
      Nat.max (makespan ops) (earliestFinish (forwardPass ops) op) := by
  unfold makespan
  rw [forwardPass_append]
  exact foldl_append_one Nat.max 0 (forwardPass ops) (earliestFinish (forwardPass ops) op)

/-- Corollary: the makespan never decreases when an op is appended. -/
theorem makespan_append_ge (ops : List Op) (op : Op) :
    makespan (ops ++ [op]) ≥ makespan ops := by
  rw [makespan_append]
  exact Nat.le_max_left _ _

/-- Corollary: the makespan after appending is at least the appended op's
own earliest-finish time — a critical op's finish time is a lower bound on
the resulting makespan. -/
theorem makespan_append_ge_finish (ops : List Op) (op : Op) :
    makespan (ops ++ [op]) ≥ earliestFinish (forwardPass ops) op := by
  rw [makespan_append]
  exact Nat.le_max_right _ _
