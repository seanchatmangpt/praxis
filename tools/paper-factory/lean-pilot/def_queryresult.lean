/- def:queryresult
   Kernel::query returns QueryResult, one of Answered(Vec<Decision>),
   Denied(Box<Decision>), or Invalid(RejectionCode); a Decision carries a
   proof (a Vec<ProofNode> DAG, positive or negative) and a receipt
   sufficient for deterministic replay. -/

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
