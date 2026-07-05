-- def:branch
-- A branch is (c, Σ, r): a class, a conjunction of signal predicates over a
-- crash snapshot, and a lawful response r ∈ {Restart, Park(ρ), Refuse(core), Escalate}.
-- The geometry maps each node to an ordered branch list; classification is
-- first-match-wins; geometry_hash = ca(Geo ‖ topology_hash).

inductive FailureClass where
  | LogicFault
  | BudgetBreach
  | AuthorityVacuum
  | TransientFault
  | Stall
  | StarvedInput
  | CertifiedUnsat
  | GeometryGap

-- Abstract carriers for the signal-predicate conjunction and the park/core payloads.
opaque CrashSnapshot : Type
opaque SignalPredicate : Type
opaque ParkPayload : Type
opaque CoreDump : Type

-- A lawful response.
inductive Response where
  | Restart
  | Park (ρ : ParkPayload)
  | Refuse (core : CoreDump)
  | Escalate

-- A branch: a class, a conjunction (list) of signal predicates, and a lawful response.
structure Branch where
  cls : FailureClass
  sigma : List SignalPredicate
  r : Response
