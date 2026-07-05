/-
prop:section

The lane encoder laneop : Scnset -> Deny implemented by denial_lane is total
via a wildcard-free match over all thirteen scenarios; restricted to the
seven denial-lane scenarios Scnset_ℓ, it and scnop_ℓ form a section-retraction
pair, so the seven single lanes and the seven denial-lane scenarios are in
bijection; the remaining six scenarios map onto closest-fit lanes and are not
in that bijection's range.

This file reuses `Deny` and `scnop_ℓ` from def_decoder.lean verbatim, and
`Scnset` from thm_total.lean verbatim (both already kernel-verified). It adds
the lane encoder `laneop : Scnset → Deny` (total, wildcard-free on the seven
denial scenarios, closest-fit `other` elsewhere) and proves the
section-retraction pair on the restricted seven-element subsets, which
delivers the bijection between `{c1,...,c7}` and
`{denialA,...,denialG}`.
-/

/-- `Deny` (reproduced from `def:decoder`). -/
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

/-- `Scnset` (reproduced from `def:tax` / `thm:total`). -/
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

/-- `scnop_ℓ`, reproduced verbatim from def_decoder.lean. -/
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

/-- `laneop`, i.e. `denial_lane`: the lane encoder `Scnset → Deny`. It is
total via a wildcard-free match over all thirteen scenarios: each of the
seven denial-lane scenarios maps to its named single-lane constant, and every
other scenario maps to the catch-all `other` (its closest-fit lane, since
none of them belongs to a named single lane). -/
def laneop : Scnset → Deny
  | .schemaObligation => .other
  | .policyObligation => .other
  | .signatureObligation => .other
  | .denialA => .c1
  | .denialB => .c2
  | .denialC => .c3
  | .denialD => .c4
  | .denialE => .c5
  | .denialF => .c6
  | .denialG => .c7
  | .logicAndon1 => .other
  | .logicAndon2 => .other
  | .logicAndon3 => .other

/-- `laneop` is total: an ordinary Lean function already assigns exactly one
`Deny` value to every `Scnset` value, witnessed by the wildcard-free match. -/
theorem laneop_total :
    ∀ s : Scnset, ∃ d : Deny, laneop s = d ∧ ∀ d' : Deny, laneop s = d' → d = d' := by
  intro s
  exact ⟨laneop s, rfl, fun d' hd' => hd'⟩

/-- The seven denial-lane scenarios `Scnset_ℓ`. -/
def isDenialScn : Scnset → Prop
  | .denialA | .denialB | .denialC | .denialD | .denialE | .denialF | .denialG => True
  | _ => False

/-- The seven single-lane words. -/
def isSingleLane : Deny → Prop
  | .c1 | .c2 | .c3 | .c4 | .c5 | .c6 | .c7 => True
  | _ => False

/-- Section: restricted to the seven denial-lane scenarios, `laneop` followed
by `scnop_ℓ` returns the original scenario. -/
theorem laneop_scnop_section :
    ∀ s : Scnset, isDenialScn s → scnop_ℓ (laneop s) = some s := by
  intro s hs
  cases s <;> simp [isDenialScn] at hs <;> rfl

/-- Retraction: restricted to the seven single-lane words, `scnop_ℓ` followed
by `laneop` returns the original word (after unwrapping the `some`). -/
theorem scnop_laneop_retraction :
    ∀ d : Deny, isSingleLane d → ∃ s : Scnset, scnop_ℓ d = some s ∧ laneop s = d := by
  intro d hd
  cases d <;> simp [isSingleLane] at hd
  · exact ⟨.denialA, rfl, rfl⟩
  · exact ⟨.denialB, rfl, rfl⟩
  · exact ⟨.denialC, rfl, rfl⟩
  · exact ⟨.denialD, rfl, rfl⟩
  · exact ⟨.denialE, rfl, rfl⟩
  · exact ⟨.denialF, rfl, rfl⟩
  · exact ⟨.denialG, rfl, rfl⟩

/-- Bijection: the seven single lanes and the seven denial-lane scenarios are
in bijection, via the section-retraction pair `(laneop, scnop_ℓ)`. -/
theorem lane_denial_bijection :
    (∀ s : Scnset, isDenialScn s → scnop_ℓ (laneop s) = some s) ∧
    (∀ d : Deny, isSingleLane d → ∃ s : Scnset, scnop_ℓ d = some s ∧ laneop s = d) :=
  ⟨laneop_scnop_section, scnop_laneop_retraction⟩

/-- The remaining six scenarios (the three obligation scenarios and the three
`logicAndon` scenarios) map to the closest-fit lane `other`, which is not in
the range of the `{c1,...,c7}` ↔ `{denialA,...,denialG}` bijection witnessed
above (i.e. they are not denial-lane scenarios, so they are outside the
bijection's domain, and their image `other` is outside `{c1,...,c7}`). -/
theorem remaining_scenarios_outside_bijection :
    ∀ s : Scnset, ¬ isDenialScn s → laneop s = .other ∧ ¬ isSingleLane (laneop s) := by
  intro s hs
  cases s <;> simp_all [isDenialScn, laneop, isSingleLane]

theorem prop_section :
    (∀ s : Scnset, ∃ d : Deny, laneop s = d ∧ ∀ d' : Deny, laneop s = d' → d = d') ∧
    ((∀ s : Scnset, isDenialScn s → scnop_ℓ (laneop s) = some s) ∧
     (∀ d : Deny, isSingleLane d → ∃ s : Scnset, scnop_ℓ d = some s ∧ laneop s = d)) ∧
    (∀ s : Scnset, ¬ isDenialScn s → laneop s = .other ∧ ¬ isSingleLane (laneop s)) :=
  ⟨laneop_total, lane_denial_bijection, remaining_scenarios_outside_bijection⟩
