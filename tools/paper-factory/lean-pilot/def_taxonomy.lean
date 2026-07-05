/-
def:taxonomy — Refusal taxonomy.

"A refusal taxonomy is a pair (S, cat) where S is a finite set of concrete
refusal scenarios and cat : S → C is a total map onto the eight-element
category set C."

This is a *definition*: the only proof obligation is that the file
type-checks. We model:
  * `C` as a concrete eight-element inductive (the eight refusal
    categories), so "eight-element category set" is witnessed literally
    rather than assumed via an opaque cardinality axiom;
  * `S` as an abstract `Fintype`-style finite carrier (a `Type` together
    with a `List` enumerating it and a proof every element occurs), kept
    abstract since the concrete scenario set is corpus-external;
  * `cat` as a plain total function `S → C` (totality is definitional in
    Lean: every `f : S → C` is total by construction);
  * `RefusalTaxonomy` bundles `(S, cat)` as a structure, matching the
    pair `(S, cat)` in the statement.
-/

/-- The eight-element category set `C`. -/
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

/-- `Category` has exactly the eight constructors above. -/
def Category.all : List Category :=
  [ .scopeViolation, .missingObligation, .staleReceipt, .clockDependence
  , .vocabViolation, .unauthorizedActor, .malformedGraph, .policyConflict ]

theorem Category.all_length : Category.all.length = 8 := by decide

theorem Category.all_nodup : Category.all.Nodup := by decide

theorem Category.all_complete : ∀ c : Category, c ∈ Category.all := by
  intro c; cases c <;> decide

/-- A finite carrier: a type `S` together with an explicit enumeration
witnessing finiteness (`S` is the finite set of concrete refusal
scenarios). -/
structure FiniteCarrier where
  S : Type
  elems : List S
  complete : ∀ s : S, s ∈ elems

/-- A refusal taxonomy: a pair `(S, cat)` where `S` is a finite set of
concrete refusal scenarios and `cat : S → C` is a total map onto the
eight-element category set `C`. -/
structure RefusalTaxonomy where
  carrier : FiniteCarrier
  cat : carrier.S → Category
