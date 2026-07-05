/-
thm:total

The category map catop : Scnset -> Catset is total (every one of the thirteen
scenarios has exactly one category, via a wildcard-free match) and its image
covers Catset.

Note: the source LaTeX additionally claims the image excludes `Reserved`
(exactly seven buckets inhabited, `Reserved` empty). That does not hold for
the actual verified `catop` in def_tax.lean, whose match sends
`logicAndon1/2/3 ↦ .Reserved` — so `Reserved` is in fact inhabited. This file
formalizes the part that is true of the real definition: `catop` is total
(a plain total Lean function, witnessed by the exhaustive wildcard-free
match) and surjective onto all eight elements of `Catset`, `Reserved`
included.
-/

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

def catop : Scnset → Catset
  | .schemaObligation => .Identity
  | .policyObligation => .Authorization
  | .signatureObligation => .Authorization
  | .denialA => .Identity
  | .denialB => .Capacity
  | .denialC => .Topology
  | .denialD => .Temporal
  | .denialE => .Lifecycle
  | .denialF => .Authorization
  | .denialG => .Prerequisites
  | .logicAndon1 => .Reserved
  | .logicAndon2 => .Reserved
  | .logicAndon3 => .Reserved

instance : DecidableEq Catset := inferInstance
instance : DecidableEq Scnset := inferInstance

/-- `catop` is total: being an ordinary Lean function `Scnset → Catset` it
already assigns exactly one `Catset` value to every `Scnset` value; the
wildcard-free match above is the mechanized witness of this. -/
theorem catop_total : ∀ s : Scnset, ∃ c : Catset, catop s = c ∧ ∀ c' : Catset, catop s = c' → c = c' := by
  intro s
  exact ⟨catop s, rfl, fun c' hc' => hc'⟩

/-- The image of `catop` is all of `Catset`: every category, including
`Reserved`, has a nonempty preimage. -/
theorem catop_surjective : ∀ c : Catset, ∃ s : Scnset, catop s = c := by
  intro c
  cases c with
  | Identity => exact ⟨.schemaObligation, rfl⟩
  | Capacity => exact ⟨.denialB, rfl⟩
  | Topology => exact ⟨.denialC, rfl⟩
  | Temporal => exact ⟨.denialD, rfl⟩
  | Lifecycle => exact ⟨.denialE, rfl⟩
  | Authorization => exact ⟨.policyObligation, rfl⟩
  | Prerequisites => exact ⟨.denialG, rfl⟩
  | Reserved => exact ⟨.logicAndon1, rfl⟩

theorem thm_total :
    (∀ s : Scnset, ∃ c : Catset, catop s = c ∧ ∀ c' : Catset, catop s = c' → c = c') ∧
    (∀ c : Catset, ∃ s : Scnset, catop s = c) :=
  ⟨catop_total, catop_surjective⟩
