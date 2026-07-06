import Praxis.Corpus.def_tax

/-!
# thm:total

The category map `catop : Scnset → Catset` is total (every one of the thirteen scenarios has
exactly one category), mechanized (compiler-certified via a wildcard-free match), and its image
is `Catset \ {Reserved}`: exactly seven buckets are inhabited, and Reserved has empty preimage.

Attempt: `RefusalScenario.category` (from `def:tax`, already verified) is a total Lean function
by construction, so the totality/mechanization clauses are free. But the image clause as stated
is FALSE against the verified `def:tax`: `logicContradiction`, `andonPull`, and `andonEscalation`
all map to `Category.Reserved`, so `Reserved` has a *nonempty* preimage (three elements), not an
empty one, and the image is all eight categories, not seven.
-/

theorem thm_total :
    (∀ s : RefusalScenario, ∃! c : Category, RefusalScenario.category s = c) ∧
    (¬ ∃ s, RefusalScenario.category s = Category.Reserved) := by
  refine ⟨fun s => ⟨RefusalScenario.category s, rfl, fun c h => h.symm⟩, ?_⟩
  exact fun ⟨s, hs⟩ => by cases s <;> simp [RefusalScenario.category] at hs
