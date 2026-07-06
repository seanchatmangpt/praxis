import Mathlib.Tactic

/-!
`def:classes`: the semantic failure-class axis, capped at eight constructors by doctrine.

  FailureClass = { LogicFault, BudgetBreach, AuthorityVacuum, TransientFault,
                    Stall, StarvedInput, CertifiedUnsat, GeometryGap }

A finite closed enumeration is exactly what Lean's `inductive` gives natively (with
`DecidableEq`, `Fintype`, `Repr` derived for free) -- no Mathlib-level composition is
needed beyond the `Fintype`/`DecidableEq` instances, which come from `deriving`.
-/

inductive FailureClass : Type where
  | LogicFault
  | BudgetBreach
  | AuthorityVacuum
  | TransientFault
  | Stall
  | StarvedInput
  | CertifiedUnsat
  | GeometryGap
  deriving DecidableEq, Repr, Fintype

/-- The doctrine caps the semantic axis at eight classes. -/
theorem FailureClass.card_eq_eight : Fintype.card FailureClass = 8 := by decide
