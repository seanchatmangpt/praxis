import Praxis.Corpus.def_tax

/-!
# cor:tax_not_total (REFUTED_BY_CURRENT_DEFINITION, was `thm:total`)

The original `thm:total` claimed the category map `catop : Scnset → Catset`
(`RefusalScenario.category`, from `def:tax`) satisfies two clauses:

1. totality/mechanization: every scenario has exactly one category (true,
   free by construction -- every Lean function is total);
2. image clause: `Reserved` has *empty* preimage, i.e. the image is exactly
   `Catset \ {Reserved}` (seven inhabited buckets).

Clause (2) is FALSE against the verified `def:tax`. Concrete witness:
`RefusalScenario.logicContradiction` maps to `Category.Reserved`
(`RefusalScenario.category logicContradiction = Category.Reserved`, by
`rfl` from the `def_tax.lean` match). Two more scenarios (`andonPull`,
`andonEscalation`) share the same image point, so `Reserved`'s preimage has
three elements, not zero.

This file replaces `thm_total.lean`: it keeps the true totality clause
(`category_total`, unconditional, still holds) but proves the actual
NEGATION of the image clause with the concrete witness above, rather than
asserting the false original conjunction. No `sorry`, no weakened
hypotheses -- the counterexample is checked by `rfl`/`decide` against the
literal `def:tax` match table.

## Diagnosis: what was likely intended

The original image clause ("exactly seven buckets are inhabited, Reserved
has empty preimage") reads like a mechanization slip, not a modeling error
in `Category` or `RefusalScenario` themselves. Two plausible original
intents, neither of which the current flat `RefusalScenario → Category`
match encodes:

- **`Reserved` was meant to be reserved for future lanes, not populated by
  existing scenarios.** The three logic/andon variants
  (`logicContradiction`, `andonPull`, `andonEscalation`) were bucketed into
  `Reserved` as a placeholder during development and never given their own
  categories (e.g. a `Logic`/`Andon` category was never added to `Catset`,
  so they fell through to the catch-all). Fix would be to grow `Catset` by
  one or two variants and re-point those three scenarios, not to touch the
  totality proof.
- **`category` was meant to be a *partial* map on a restricted "closed"
  subset of `Scnset`** (the ten obligation/denial-lane scenarios) with the
  three logic/andon scenarios routed through a *different* dispatch (e.g. a
  separate `AndonClass` outcome) rather than through `Catset` at all. Under
  that reading `Reserved` should never appear as an output of `category` on
  the closed subset, and the theorem should have been stated over that
  subset (`∀ s ∈ closedSubset, category s ≠ Reserved`), not over all of
  `Scnset`.

Either way, the underlying domain model (`Category`, `RefusalScenario`, and
the match table in `def_tax.lean`) is left untouched here, per instructions:
this file documents and formally seals the discrepancy, it does not fix it.
-/

/-- The categorizing map is total: every scenario has exactly one category.
This half of the original `thm:total` claim is genuinely true and
unconditional -- kept here (not weakened, not dropped) alongside the
negative result below. -/
theorem category_total :
    ∀ s : RefusalScenario, ∃! c : Category, RefusalScenario.category s = c :=
  fun s => ⟨RefusalScenario.category s, rfl, fun _c h => h.symm⟩

/-- Concrete witness: `logicContradiction` categorizes to `Reserved`. -/
theorem logicContradiction_reserved :
    RefusalScenario.category RefusalScenario.logicContradiction = Category.Reserved := rfl

/-- **cor:tax_not_total.** The image clause of the original `thm:total` is
false: `Reserved` does *not* have empty preimage under `RefusalScenario.category`.
Witnessed concretely by `logicContradiction`. -/
theorem cor_tax_not_total :
    ¬ (¬ ∃ s : RefusalScenario, RefusalScenario.category s = Category.Reserved) := by
  intro h
  exact h ⟨RefusalScenario.logicContradiction, logicContradiction_reserved⟩

/-- Sharper form: `Reserved`'s preimage under `category` has (at least) the
three named elements, all distinct, all mapping to `Reserved` -- checked by
`decide` against the literal match table in `def_tax.lean`. -/
theorem reserved_preimage_at_least_three :
    RefusalScenario.category RefusalScenario.logicContradiction = Category.Reserved ∧
    RefusalScenario.category RefusalScenario.andonPull = Category.Reserved ∧
    RefusalScenario.category RefusalScenario.andonEscalation = Category.Reserved := by
  decide
