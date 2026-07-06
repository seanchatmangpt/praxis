/-!
refusal:v1scope

"v1 scope: beat scheduling, full R1/W1/C1 SLO matrix, quarantine parole,
supra-cell supervision. Each receipted with salvage: the enum and one
tier transition ship; epoch boundaries already provide the re-admission
cadence beats would refine; quarantined classes await a parole
mechanism; cells supervising cells is the composition step after this
one."

This is a scope-refusal note, not a mathematical claim: it enumerates
four candidate v1 features and records, for each, whether it ships in
v1 or is deferred with a stated reason (its "salvage"). The honest
formalization is exactly that -- a finite enum of the candidate
features, and a decision function recording ship/defer + salvage
reason, matching the prose one-for-one. No numeric SLO matrix, no
concrete scheduler, and no supervision protocol is invented here: this
statement is only about what is in vs. out of v1, not how the shipped
pieces work internally (those get their own `def:`/`construction:`
formalizations elsewhere in the corpus).

Everything below is composed from pre-built Lean/Mathlib pieces
(`Bool`, `String`, `Prod`) -- no bespoke axiom is declared, because a
scope decision is exactly data, not a claim requiring a hash, an
observation space, or any other irreducible primitive.
-/

/-- The four v1-scope candidates named in the statement. -/
inductive V1ScopeCandidate where
  | beatScheduling
  | fullSlOMatrix
  | quarantineParole
  | supraCellSupervision
  deriving DecidableEq, Repr

open V1ScopeCandidate

/-- Whether a candidate ships in v1. Only `beatScheduling` ships in
this formalization, per "the enum and one tier transition ship" --
i.e. exactly one piece (a single tier transition) of the beat-
scheduling candidate is in scope, not the full SLO matrix. -/
def shipsInV1 : V1ScopeCandidate → Bool
  | beatScheduling => true
  | fullSlOMatrix => false
  | quarantineParole => false
  | supraCellSupervision => false

/-- The salvage reason recorded for each candidate: why a deferred
piece is not simply dropped, but has an existing (or future) path to
the same effect. -/
def salvage : V1ScopeCandidate → String
  | beatScheduling =>
    "the enum and one tier transition ship"
  | fullSlOMatrix =>
    "epoch boundaries already provide the re-admission cadence beats would refine"
  | quarantineParole =>
    "quarantined classes await a parole mechanism"
  | supraCellSupervision =>
    "cells supervising cells is the composition step after this one"

/-- The full v1-scope decision: each candidate paired with its
ship/defer status and its salvage reason, as one record per candidate,
matching the statement's structure exactly. -/
def v1ScopeReceipt : V1ScopeCandidate → Bool × String :=
  fun c => (shipsInV1 c, salvage c)

/-- Sanity check: exactly one candidate ships in v1 (`beatScheduling`,
via its single tier transition), the other three are deferred with
salvage. This is the "each receipted with salvage" claim, made
checkable. -/
theorem v1_scope_exactly_one_ships :
    (shipsInV1 beatScheduling = true) ∧
    (shipsInV1 fullSlOMatrix = false) ∧
    (shipsInV1 quarantineParole = false) ∧
    (shipsInV1 supraCellSupervision = false) := by
  decide

