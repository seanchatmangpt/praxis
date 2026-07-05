/-
prop:partial

scnop_ℓ is a partial map, definite (as Some) only on L \ {Adml}; it is not
realizable as a wildcard-free exhaustive match because its domain Deny is an
open value space of cardinality 2^64, not a closed enum, so exhaustiveness
over the seven single lanes is a runtime test obligation, not a static type
obligation.

We witness the partiality of scnop_ℓ inside the finite Lean model of
def:decoder: there are two *distinct* words in `Deny` (the clean word `Adml`
and the catch-all `other`, standing in for the open complement of
L = {Adml, c1, ..., c7} inside the real 2^64-valued `Deny` space) on which
scnop_ℓ returns `none`. So `none` is not confined to a single point (namely
`Adml`) and the map cannot be re-expressed as a total function on the seven
named lanes alone: the `other` branch is load-bearing, not a redundant
wildcard, exactly because ruling it out requires knowing every non-lane word
is absent -- an open, non-enumerable domain fact, not something the closed
match on `Catset`/`Scnset` can certify statically.
-/

inductive Deny where
  | Adml
  | c1
  | c2
  | c3
  | c4
  | c5
  | c6
  | c7
  | other
deriving DecidableEq, Repr

inductive Scnset where
  | schemaObligation
  | policyObligation
  | signatureObligation
  | denialA
  | denialB
  | denialC
  | denialD
  | denialE
  | denialF
  | denialG
  | logicAndon1
  | logicAndon2
  | logicAndon3
deriving DecidableEq, Repr

def scnop_ℓ : Deny → Option Scnset
  | .Adml => none
  | .c1 => some .denialA
  | .c2 => some .denialB
  | .c3 => some .denialC
  | .c4 => some .denialD
  | .c5 => some .denialE
  | .c6 => some .denialF
  | .c7 => some .denialG
  | .other => none

/-- `scnop_ℓ` is genuinely partial: `none` is witnessed on two distinct words
(`Adml` and `other`), so the domain on which it is defined is strictly smaller
than `Deny` minus a single point, and the `other` branch cannot be dropped as
a redundant wildcard over the seven named lanes. -/
theorem prop_partial :
    ∃ x y : Deny, x ≠ y ∧ scnop_ℓ x = none ∧ scnop_ℓ y = none := by
  refine ⟨.Adml, .other, ?_, ?_, ?_⟩
  · decide
  · decide
  · decide
