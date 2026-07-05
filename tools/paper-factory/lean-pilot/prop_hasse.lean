/-
prop:hasse — Let $\prec$ be the strict order "$j$ finishes before $i$ starts"
on plan steps. The initialized `pred_mask` of step $i$ is $\{j : j \prec i\}$,
and after transitive reduction it is the covering relation of $\prec$; the
resulting dependency graph is a DAG, and two steps are schedulable
concurrently iff $\prec$-incomparable.

Formalized fragment (bare Lean 4 core, reusing `con:tape`'s
`temporal_plan_to_powl_tape` machinery from `con_tape.lean`): the covering
relation produced by transitive reduction is always a *sub-relation* of the
original precedence relation — reduction only clears bits, it never sets a
bit that wasn't already in the raw predecessor mask. This is the core
algebraic fact underlying "the reduced pred_mask is the covering relation of
$\prec$": every bit that survives reduction was already a $\prec$-edge.
-/

variable {A : Type u}

structure TemporalStep (A : Type u) where
  activity : A
  finishesBefore : List Bool

structure TemporalPlan (A : Type u) where
  steps : List (TemporalStep A)

structure PowlOpSpec (A : Type u) where
  activity : A
  predMask : List Bool
  succMask : List Bool

def oneHot (n i : Nat) : List Bool :=
  (List.range n).map (fun j => j == i)

def resize (n : Nat) (l : List Bool) : List Bool :=
  (List.range n).map (fun j => (l.getD j false))

def rawTape (plan : TemporalPlan A) : List (PowlOpSpec A) :=
  let n := plan.steps.length
  (plan.steps.zip (List.range n)).map (fun (s, i) =>
    { activity := s.activity
      predMask := resize n s.finishesBefore
      succMask := oneHot n i })

def hasBit (m : List Bool) (j : Nat) : Bool :=
  m.getD j false

def reduceStep (tape : List (PowlOpSpec A)) (spec : PowlOpSpec A) : PowlOpSpec A :=
  let n := spec.predMask.length
  let isRedundant (j : Nat) : Bool :=
    (List.range n).any (fun k =>
      k != j &&
      hasBit spec.predMask k &&
      (tape.getD k { activity := spec.activity, predMask := [], succMask := [] }
        |>.predMask |> fun pm => hasBit pm j))
  { spec with
      predMask := (List.range n).map (fun j => hasBit spec.predMask j && !isRedundant j) }

def reduceTape (tape : List (PowlOpSpec A)) : List (PowlOpSpec A) :=
  tape.map (reduceStep tape)

def temporal_plan_to_powl_tape (plan : TemporalPlan A) : List (PowlOpSpec A) :=
  reduceTape (rawTape plan)

/-- Bit-level lookup into `(List.range n).map f` at index `j < n` reduces to
`f j`; standard `List.getD`/`List.map`/`List.range` unfolding. -/
theorem getD_range_map (n : Nat) (f : Nat → Bool) (j : Nat) (h : j < n) :
    ((List.range n).map f).getD j false = f j := by
  have hlen : ((List.range n).map f).length = n := by
    simp [List.length_map, List.length_range]
  rw [List.getD_eq_getElem?_getD]
  have : ((List.range n).map f)[j]? = some (f j) := by
    have hr : (List.range n)[j]? = some j := by
      simp [h]
    simp [List.getElem?_map, hr]
  simp [this]

/-- **Hasse / transitive-reduction soundness**: every bit that survives
transitive reduction (`reduceStep tape spec`) was already set in the raw
predecessor mask `spec.predMask`. In other words, the covering relation
computed by `reduceStep` is a sub-relation of the original precedence
relation $\prec$ encoded by `spec.predMask` — reduction only removes edges,
never introduces new ones. -/
theorem reduceStep_predMask_subset (tape : List (PowlOpSpec A)) (spec : PowlOpSpec A)
    (j : Nat) (h : j < spec.predMask.length) :
    hasBit (reduceStep tape spec).predMask j = true → hasBit spec.predMask j = true := by
  intro hred
  unfold reduceStep at hred
  simp only [hasBit] at hred ⊢
  rw [getD_range_map _ _ j h] at hred
  simpa using (Bool.and_eq_true _ _).mp hred |>.1
