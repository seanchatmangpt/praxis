namespace Praxis

/-- Raw observation. Filled by generated modules. -/
structure Observation where
  label : String
deriving Repr, DecidableEq

/-- Admitted observation. -/
structure AdmittedObservation where
  label : String
deriving Repr, DecidableEq

/-- Receipt marker. -/
structure Receipt where
  label : String
deriving Repr, DecidableEq

end Praxis
