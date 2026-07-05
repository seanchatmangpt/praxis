/-
prop:obtotal

The map scnop : Obligation -> Scnset implemented by From<&Obligation> for
RefusalScenario is a total function realized by a wildcard-free match over
the three Obligation kinds; because the match has no wildcard arm, totality
is a static property of the program, not a runtime hope.
-/

/-- The Obligation enum has exactly three kinds (reused from def:ob). -/
inductive Obligation where
  | schema
  | policy
  | signature
deriving DecidableEq, Repr

/-- `Scnset`, the target of the `scnop` map: one refusal scenario tag per
obligation kind. -/
inductive Scnset where
  | schemaScenario
  | policyScenario
  | signatureScenario
deriving DecidableEq, Repr

/-- `scnop`, i.e. `From<&Obligation> for RefusalScenario`: a wildcard-free
match over the three Obligation kinds. -/
def scnop : Obligation → Scnset
  | .schema => .schemaScenario
  | .policy => .policyScenario
  | .signature => .signatureScenario

/-- Totality of `scnop`: every obligation is mapped to a definite scenario
by the wildcard-free match, exhibited by exhaustive case analysis. This is
a static property of the program (the match compiles because all three
constructors are covered), not a runtime hope. -/
theorem scnop_total (g : Obligation) :
    scnop g = Scnset.schemaScenario ∨
    scnop g = Scnset.policyScenario ∨
    scnop g = Scnset.signatureScenario := by
  cases g with
  | schema => exact Or.inl rfl
  | policy => exact Or.inr (Or.inl rfl)
  | signature => exact Or.inr (Or.inr rfl)
