import Praxis.Corpus.def_ob

/-!
# prop:obtotal

The map `δ_g : Obligation.Obs → Deny` (realized here as `Obligation.deltaG`, corresponding to
the thesis's `From<&Obligation> for RefusalScenario`) is implemented by a match over the three
`ObligationKind` constructors with no wildcard arm. Because `ObligationKind` has exactly three
constructors (`schema`, `policy`, `temporal`) and any Lean `match`/`cases` over it must cover
all of them to type-check, totality of such a map is a *static* property enforced by the kernel
at elaboration time, not a runtime hope: every `k : ObligationKind` is definitionally one of the
three named cases, so no value can "fall through" a wildcard-free match.

We formalize this exhaustiveness-is-totality property directly: every `ObligationKind` value is
one of the three named constructors. This is exactly the fact the kernel already uses to accept
`deltaG`'s definition (via `DenialPolarity` case analysis inherited from `def:denialcode`) without
a wildcard arm, so no further axiom is needed -- it is a direct consequence of `ObligationKind`
being an inductive type with exactly these three constructors, proved by `cases`.
-/

/-- The three-constructor match discipline is total: every `ObligationKind` is one of the
named cases `schema`, `policy`, or `temporal`. This is precisely the static exhaustiveness
property that lets `Obligation.deltaG` (and any `RefusalScenario`-producing match on
`ObligationKind`) be accepted by the kernel with no wildcard arm. -/
theorem ObligationKind.totality (k : ObligationKind) :
    k = ObligationKind.schema ∨ k = ObligationKind.policy ∨ k = ObligationKind.temporal := by
  cases k <;> simp
