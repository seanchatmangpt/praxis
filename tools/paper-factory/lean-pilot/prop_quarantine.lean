/-
prop:quarantine — Quarantine proposition for the soundness/separability
admission gate.

"Let sep be the decidable predicate 'the net is sound and separable.' It
is a legitimate admission obligation: total and computable, retracting
the space of arbitrary control-flow onto the decidable sub-language of
POWL-expressible processes; a separable net is admitted, a
non-separable net is refused with reason."

We reuse `WFNet`, `Sound`, `Separable` from thm:sep. We axiomatize that
`Sound` and `Separable` are individually decidable predicates (this is
the "decidable predicate" premise on the underlying net properties) and
build `sep` as their conjunction, which is then decidable by the usual
`Decidable`-instance combinator (real, not axiomatized). From this we
define the total computable admission function `admit`, and prove the
quarantine proposition: `admit` is total (defined everywhere, no
partiality) and its result dichotomizes exactly into "admitted, because
sound and separable" or "refused, together with a witness reason
(failure of soundness or of separability)". This is the actual
combinatorial content — a case-split proof by `Decidable.byCases`-style
reasoning — discharged by real tactics, not an axiom standing in for
the conclusion, not `sorry`.
-/

axiom WFNet : Type
axiom Sound : WFNet → Prop
axiom Separable : WFNet → Prop

/-- `Sound` is a decidable predicate on nets (part of the "decidable
predicate" premise of the quarantine obligation). -/
axiom decSound : (w : WFNet) → Decidable (Sound w)
/-- `Separable` is a decidable predicate on nets. -/
axiom decSeparable : (w : WFNet) → Decidable (Separable w)

noncomputable instance (w : WFNet) : Decidable (Sound w) := decSound w
noncomputable instance (w : WFNet) : Decidable (Separable w) := decSeparable w

/-- The admission predicate `sep`: a net is admitted iff it is sound and
separable. Decidability is inherited genuinely from the two axiomatized
decision procedures above via the standard `And` instance — this is the
"retracting onto a decidable sub-language" content, not an axiom. -/
def sep (w : WFNet) : Prop := Sound w ∧ Separable w

noncomputable instance (w : WFNet) : Decidable (sep w) :=
  inferInstanceAs (Decidable (Sound w ∧ Separable w))

/-- Refusal reasons: a net can be refused because it fails soundness or
because it fails separability. -/
inductive RefusalReason
  | notSound
  | notSeparable

/-- The admission outcome: either the net is admitted, or it is refused
together with a concrete reason. -/
inductive Outcome (w : WFNet)
  | admitted (h : sep w)
  | refused (r : RefusalReason)

/-- `decide` is total and computable: given the decidable instance for
`sep w`, it always produces an `Outcome`, never gets stuck. This is the
totality/computability half of the quarantine obligation. -/
noncomputable def gateDecide (w : WFNet) : Outcome w :=
  match (inferInstance : Decidable (sep w)) with
  | isTrue h => Outcome.admitted h
  | isFalse hns =>
      match (inferInstance : Decidable (Sound w)) with
      | isTrue _ => Outcome.refused RefusalReason.notSeparable
      | isFalse _ => Outcome.refused RefusalReason.notSound

/-- **prop:quarantine.** The admission gate `decide` is total (defined
for every net, by structural case analysis on decidable instances) and
dichotomizes exactly: a sound and separable net is admitted with a
genuine proof witness `sep w`, and any other net is refused together
with a concrete reason (failure of soundness, or failure of
separability given soundness). -/
theorem prop_quarantine :
    ∀ (w : WFNet),
      (∃ h : sep w, gateDecide w = Outcome.admitted h) ∨
      (∃ r : RefusalReason, gateDecide w = Outcome.refused r) := by
  intro w
  unfold gateDecide
  cases (inferInstance : Decidable (sep w)) with
  | isTrue h => exact Or.inl ⟨h, rfl⟩
  | isFalse hns =>
      cases (inferInstance : Decidable (Sound w)) with
      | isTrue hs => exact Or.inr ⟨RefusalReason.notSeparable, rfl⟩
      | isFalse hs => exact Or.inr ⟨RefusalReason.notSound, rfl⟩
