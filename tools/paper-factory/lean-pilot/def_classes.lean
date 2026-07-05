-- def:classes
-- FailureClass = {LogicFault, BudgetBreach, AuthorityVacuum, TransientFault,
--                 Stall, StarvedInput, CertifiedUnsat, GeometryGap}
-- the semantic axis, capped at eight by the doctrine.

inductive FailureClass where
  | LogicFault
  | BudgetBreach
  | AuthorityVacuum
  | TransientFault
  | Stall
  | StarvedInput
  | CertifiedUnsat
  | GeometryGap
