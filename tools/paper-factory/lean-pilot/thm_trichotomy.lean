/- thm:trichotomy
   For a query q evaluated against an admitted kernel, exactly one of
   Answered (a positive proof DAG with fan-in ≤ 8 plus receipt),
   Denied (a negative proof plus receipt), or Invalid (a RejectionCode,
   no proof) holds. -/

/-- Reasons a prolog8 construct can be rejected from the bounded Horn fragment. -/
inductive RejectionCode where
  | arityExceeded
  | bodyTooLong
  | tooManyVariables
  | fanInExceeded
  | cutUsed
  | dynamicMutation
  | unstratifiedNegation
  | unboundedRecursion
  | runtimeTextParsing
  | sideEffect
  | nonInternedTerm
  deriving Repr, DecidableEq

/-- A single node in a proof DAG, positive or negative. -/
inductive ProofNode where
  | positive (label : Nat)
  | negative (label : Nat)
  deriving Repr, DecidableEq

/-- Opaque receipt sufficient for deterministic replay of a decision. -/
structure Receipt where
  hash : Nat
  deriving Repr, DecidableEq

/-- A decision carries a proof (DAG of proof nodes) and a replay receipt. -/
structure Decision where
  proof   : List ProofNode
  receipt : Receipt
  deriving Repr, DecidableEq

/-- The result of `Kernel::query`: a set of accepted decisions, a single
    denial witnessed by a decision, or an outright rejection code. -/
inductive QueryResult where
  | answered (decisions : List Decision)
  | denied (decision : Decision)
  | invalid (code : RejectionCode)
  deriving Repr, DecidableEq

open QueryResult

/-- Predicate: the result is an `answered` outcome. -/
def isAnswered : QueryResult → Prop
  | answered _ => True
  | _ => False

/-- Predicate: the result is a `denied` outcome. -/
def isDenied : QueryResult → Prop
  | denied _ => True
  | _ => False

/-- Predicate: the result is an `invalid` outcome. -/
def isInvalid : QueryResult → Prop
  | invalid _ => True
  | _ => False

/-- **Trichotomy**: for any query result `r`, exactly one of `Answered`,
    `Denied`, `Invalid` holds — i.e. at least one holds, and no two hold
    simultaneously. -/
theorem trichotomy (r : QueryResult) :
    (isAnswered r ∨ isDenied r ∨ isInvalid r) ∧
    ¬ (isAnswered r ∧ isDenied r) ∧
    ¬ (isAnswered r ∧ isInvalid r) ∧
    ¬ (isDenied r ∧ isInvalid r) := by
  cases r with
  | answered decisions =>
      refine ⟨Or.inl trivial, ?_, ?_, ?_⟩
      · rintro ⟨_, hd⟩; exact hd
      · rintro ⟨_, hi⟩; exact hi
      · rintro ⟨hd, _⟩; exact hd
  | denied decision =>
      refine ⟨Or.inr (Or.inl trivial), ?_, ?_, ?_⟩
      · rintro ⟨ha, _⟩; exact ha
      · rintro ⟨ha, _⟩; exact ha
      · rintro ⟨_, hi⟩; exact hi
  | invalid code =>
      refine ⟨Or.inr (Or.inr trivial), ?_, ?_, ?_⟩
      · rintro ⟨ha, _⟩; exact ha
      · rintro ⟨ha, _⟩; exact ha
      · rintro ⟨hd, _⟩; exact hd
