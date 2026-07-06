import Mathlib.Tactic

/-!
`prop:totalaccounting`: Every node of every supervised run carries exactly one
disposition (`Completed(r)`, `Parked`, `SkippedBy`, `GaveUp`); no silent rows
exist (test-pinned: `|dispositions| = |V|`).

Design notes on reuse vs. axiomatization:
- The four disposition alternatives form a finite closed sum type, one
  payload-carrying constructor (`Completed`) and three plain ones -- exactly
  what Lean's native `inductive` gives, as in `def:branch`'s `Response`.
- "Every node carries exactly one disposition" is modeled the same way the
  thesis's own runs work: as a *total function* `V → Disposition R`. A total
  function already gives, by construction, exactly one output per input --
  no separate existence/uniqueness axiom is needed for that half of the
  statement.
- The remaining, test-pinned half -- "no silent rows exist,
  `|dispositions| = |V|`" -- is the actual mathematical content: the graph of
  the assignment (the finite set of `(node, disposition)` rows) has the same
  cardinality as the node set `V`. This is proved, not axiomatized, as a
  corollary of Mathlib's `Finset.card_image_of_injective` (the pairing
  `v ↦ (v, disposition v)` is injective because its first projection recovers
  `v`), composed with the existing `Fintype`/`Finset.univ` machinery -- no new
  axioms of any kind.
-/

/-- The four lawful dispositions a supervised run can leave on a node,
    parameterized by the payload type `R` carried by `Completed`. -/
inductive Disposition (R : Type) : Type where
  | Completed (r : R)
  | Parked
  | SkippedBy
  | GaveUp
  deriving DecidableEq

/-- `prop:totalaccounting`: for any finite node set `V` and any total
    disposition assignment `disposition : V → Disposition R` (which, being a
    function, already gives every node exactly one disposition), the finite
    set of disposition rows `{(v, disposition v) | v ∈ V}` has exactly `|V|`
    elements -- no node is silently dropped or duplicated. -/
theorem totalaccounting {V R : Type} [Fintype V] [DecidableEq V] [DecidableEq R]
    (disposition : V → Disposition R) :
    (Finset.univ.image (fun v => (v, disposition v))).card = Fintype.card V := by
  rw [Finset.card_image_of_injective _ (fun a b hab => (congrArg Prod.fst hab))]
  simp
