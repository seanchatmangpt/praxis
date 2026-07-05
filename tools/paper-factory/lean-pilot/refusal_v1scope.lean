/- refusal:v1scope
   v1 scope: beat scheduling, full R1/W1/C1 SLO matrix, quarantine parole,
   supra-cell supervision. Each receipted with salvage: the enum and one
   tier transition ship; epoch boundaries already provide the re-admission
   cadence beats would refine; quarantined classes await a parole mechanism;
   cells supervising cells is the composition step after this one.

   This is a `refusal` kind: an explicit out-of-scope declaration for v1,
   not a theorem. It is encoded as an enumerated inductive type of the
   deferred features, each paired with the salvage rationale that
   justifies deferring it, so the refusal is a checked (type-checking-only)
   piece of vocabulary rather than free-text commentary. -/

/-- The features explicitly out of scope for v1. -/
inductive OutOfScopeV1 where
  | beatScheduling
  | fullSlOMatrix
  | quarantineParole
  | supraCellSupervision
  deriving DecidableEq, Repr

/-- The salvage rationale recorded against each deferred feature. -/
def salvage : OutOfScopeV1 → String
  | .beatScheduling => "epoch boundaries already provide the re-admission cadence beats would refine"
  | .fullSlOMatrix => "the enum and one tier transition ship"
  | .quarantineParole => "quarantined classes await a parole mechanism"
  | .supraCellSupervision => "cells supervising cells is the composition step after this one"

/-- Every deferred feature carries a nonempty salvage rationale. -/
theorem salvage_nonempty (f : OutOfScopeV1) : (salvage f).length > 0 := by
  cases f <;> decide
