/-
cor:assembled

Composing the total maps of prop:obtotal and prop:section with the total
classification of thm:total: every unmet obligation (via catop ∘ scnop) and
every non-Adml single denial lane (via catop ∘ scnop_ℓ on Deny \ {Adml})
maps to exactly one category; no obligation failure and no fired single lane
is left unclassified.

This file reproduces, verbatim, the kernel-verified vocabulary from
prop_obtotal.lean, prop_section.lean and thm_total.lean (all already
type-checked on disk), and composes them.
-/

/-- `Obligation` (reused from prop:obtotal). -/
inductive Obligation where
  | schema
  | policy
  | signature
deriving DecidableEq, Repr

/-- `Scnset` (reused from thm:total / prop:section). -/
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

/-- `Deny` (reused from prop:section). -/
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

/-- `Catset` (reused from thm:total). -/
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

/-- `scnop`, i.e. `From<&Obligation> for RefusalScenario`, reused verbatim
from prop:obtotal (retargeted onto the shared `Scnset` of thm:total /
prop:section, which is the composition site). -/
def scnop : Obligation → Scnset
  | .schema => .schemaObligation
  | .policy => .policyObligation
  | .signature => .signatureObligation

/-- `scnop_ℓ`, reused verbatim from prop:section / def:decoder. -/
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

/-- `catop`, reused verbatim from thm:total. -/
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

/-- The non-`Adml` single denial lanes: the seven lanes `c1,...,c7`. -/
def isSingleLane : Deny → Prop
  | .c1 | .c2 | .c3 | .c4 | .c5 | .c6 | .c7 => True
  | _ => False

/-- Every unmet obligation, pushed through `scnop` and then `catop`, lands
in exactly one category: no obligation is left unclassified. This is
`catop ∘ scnop`, composing the totality of `scnop` (prop:obtotal) with the
totality of `catop` (thm:total). -/
theorem obligation_classified (o : Obligation) :
    ∃ c : Catset, catop (scnop o) = c ∧ ∀ c' : Catset, catop (scnop o) = c' → c = c' := by
  exact ⟨catop (scnop o), rfl, fun c' hc' => hc'⟩

/-- Every fired single lane (`c1,...,c7`, i.e. every `Deny` value other than
`Adml`'s catch-all), decoded via `scnop_ℓ` and then classified via `catop`,
lands in exactly one category: no fired single lane is left unclassified.
This is `catop ∘ scnop_ℓ` restricted to the domain where `scnop_ℓ` is
defined, composing the section half of prop:section with the totality of
`catop` (thm:total). -/
theorem single_lane_classified (d : Deny) (hd : isSingleLane d) :
    ∃ s : Scnset, ∃ c : Catset,
      scnop_ℓ d = some s ∧ catop s = c ∧ ∀ c' : Catset, catop s = c' → c = c' := by
  cases d <;> simp [isSingleLane] at hd
  · exact ⟨.denialA, catop .denialA, rfl, rfl, fun c' hc' => hc'⟩
  · exact ⟨.denialB, catop .denialB, rfl, rfl, fun c' hc' => hc'⟩
  · exact ⟨.denialC, catop .denialC, rfl, rfl, fun c' hc' => hc'⟩
  · exact ⟨.denialD, catop .denialD, rfl, rfl, fun c' hc' => hc'⟩
  · exact ⟨.denialE, catop .denialE, rfl, rfl, fun c' hc' => hc'⟩
  · exact ⟨.denialF, catop .denialF, rfl, rfl, fun c' hc' => hc'⟩
  · exact ⟨.denialG, catop .denialG, rfl, rfl, fun c' hc' => hc'⟩

/-- Assembled corollary: composing `scnop` (prop:obtotal) and `scnop_ℓ`
(prop:section) with `catop` (thm:total), every unmet obligation and every
fired single denial lane maps to exactly one category — no obligation
failure and no fired single lane is left unclassified. -/
theorem cor_assembled :
    (∀ o : Obligation, ∃ c : Catset, catop (scnop o) = c ∧
        ∀ c' : Catset, catop (scnop o) = c' → c = c') ∧
    (∀ d : Deny, isSingleLane d → ∃ s : Scnset, ∃ c : Catset,
        scnop_ℓ d = some s ∧ catop s = c ∧ ∀ c' : Catset, catop s = c' → c = c') :=
  ⟨obligation_classified, single_lane_classified⟩
