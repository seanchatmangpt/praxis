/-
prop:total — Totality of the category map and section property of the
lane inverse.

"The category map cat : S -> C is total (enforced by the host type
system's exhaustive match), and the lane map's single-lane inverse is a
section of the lane map over the seven named lanes."

We reuse `RefusalTaxonomy` / `Category` from `def:taxonomy` and the
denial-word machinery (`Word`, lane indexing by `Fin m`) from
`con:denial` rather than redeclaring them.

Two genuine claims are proved:

1. `cat_total`: for any taxonomy `T` and scenario `s`, `T.cat s` produces
   an actual value of `Category` — totality of the map, witnessed
   directly (in Lean every function `S -> C` is total by construction;
   the exhaustive-match enforcement from the host type system is
   reflected here as the trivial existence of the image value).

2. `invLane_section`: the seven named lanes are embedded into the `m`
   available lanes via `sevenLanes : Fin 7 -> Fin m` (assuming the model
   has at least seven lanes, `hm7 : 7 <= m`), and `invLane : Fin m ->
   Option (Fin 7)` is a genuine one-sided inverse: `invLane` composed
   with `sevenLanes` is the identity (wrapped in `some`), i.e. `invLane`
   is a section of `sevenLanes` over the seven named lanes.
-/

-- ---------------------------------------------------------------------
-- def:taxonomy, reused verbatim
-- ---------------------------------------------------------------------

inductive Category : Type
  | scopeViolation
  | missingObligation
  | staleReceipt
  | clockDependence
  | vocabViolation
  | unauthorizedActor
  | malformedGraph
  | policyConflict
  deriving DecidableEq

structure FiniteCarrier where
  S : Type
  elems : List S
  complete : ∀ s : S, s ∈ elems

structure RefusalTaxonomy where
  carrier : FiniteCarrier
  cat : carrier.S → Category

-- ---------------------------------------------------------------------
-- con:denial, reused vocabulary (lane indexing by `Fin m`)
-- ---------------------------------------------------------------------

axiom m : Nat

/-- The model has at least the seven named lanes among its `m` lanes. -/
axiom hm7 : 7 ≤ m

-- ---------------------------------------------------------------------
-- prop:total
-- ---------------------------------------------------------------------

/-- The category map `cat : S → C` is total: every scenario is sent to
an actual `Category` value. -/
theorem cat_total (T : RefusalTaxonomy) (s : T.carrier.S) :
    ∃ c : Category, T.cat s = c :=
  ⟨T.cat s, rfl⟩

/-- The seven named lanes, embedded as the first seven of the `m`
available lanes. -/
def sevenLanes (i : Fin 7) : Fin m :=
  ⟨i.val, Nat.lt_of_lt_of_le i.isLt hm7⟩

/-- The single-lane inverse: recovers the named-lane index for any lane
`j < 7`, and `none` otherwise. -/
def invLane (j : Fin m) : Option (Fin 7) :=
  if h : j.val < 7 then some ⟨j.val, h⟩ else none

/-- `invLane` is a section of `sevenLanes` over the seven named lanes:
composing `sevenLanes` then `invLane` recovers the original index. -/
theorem invLane_section : ∀ i : Fin 7, invLane (sevenLanes i) = some i := by
  intro i
  unfold invLane sevenLanes
  simp
