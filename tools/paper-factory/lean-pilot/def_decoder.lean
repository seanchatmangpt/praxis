/-
def:decoder

Let L = {Adml, c1, ..., c7} ⊆ Deny be the clean word together with the seven
named single-lane constants; the lane decoder scnop_ℓ : Deny ⇀ Scnset is
scenario_for_denial_lane: it returns None on Adml, returns the matching
denial-lane scenario on each c_i, and returns None on every word not in L.
-/

/-- `Deny`: the word alphabet containing the clean word `Adml`, the seven
named single-lane constants `c1..c7`, and a catch-all `other` standing for
every word outside `L = {Adml, c1, ..., c7}`. -/
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

/-- `Catset`: the eight-element category enum (reproduced from `def:tax`). -/
inductive Catset where
  | Identity
  | Capacity
  | Topology
  | Temporal
  | Lifecycle
  | Authorization
  | Prerequisites
  | Reserved
deriving DecidableEq, Repr

/-- `Scnset`: the thirteen-variant `RefusalScenario` enum (reproduced from
`def:tax`). -/
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

/-- `scnop_ℓ`, i.e. `scenario_for_denial_lane`: the lane decoder `Deny ⇀ Scnset`.
It returns `none` on the clean word `Adml`, the matching denial-lane scenario
on each single-lane constant `c1..c7`, and `none` on every word not in
`L = {Adml, c1, ..., c7}` (i.e. on `other`). -/
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
