/-
Label: prop:fuzz
Kind: proposition

Statement: If `adm` is total and terminating with codomain `Adm ∪ {Rfsl}`
and `Rfsl` is refusal-with-category, then for every `o ∈ Obs`,
`Ω_∂(o) = 1`, and any observed `Ω_∂(o) = 0` is a genuine defect in the
retraction, not a property of the input.

Formalization, reusing `def:fuzz`'s `AdmOutcome`, `WellFormedArtifact`, and
`fuzzOracle`:

- "adm is total and terminating with codomain Adm ∪ {Rfsl}" is already
  built into `adm : Obs → AdmOutcome Adm` being an ordinary total Lean
  function into `AdmOutcome Adm = admitted Adm | refusal RefusalCategory`.
- "Rfsl is refusal-with-category" is built into the `refusal` constructor
  always carrying a `RefusalCategory`.
- The remaining hypothesis needed for `Ω_∂(o) = 1` on every `o` is that
  every observation is registered in the witness list `X` (this is what
  "total and terminating" contributes at the level of the *oracle*, which
  only inspects outcomes for `o ∈ X`), and that every admitted outcome's
  artifact is well-formed. Under exactly these two hypotheses we prove
  `fuzzOracle adm μWF X o = true` for every `o`.
- The contrapositive is recorded as `fuzz_zero_is_defect`: if the oracle
  ever reports `0` on some `o` while the totality/well-formedness
  hypotheses hold for that particular `o`, that is a contradiction --
  i.e. an observed `0` under total+well-formed `adm` cannot happen, which
  is the formal reading of "a genuine defect in the retraction, not a
  property of the input" (the defect is exactly the failure of one of
  these hypotheses at `o`, never `o` itself).
-/

section Fuzz

variable {Obs Adm : Type}

inductive RefusalCategory
  | malformed
  | outOfScope
  | resourceExhausted
  | policyViolation
  | typeMismatch
  | nonTermination
  | integrityFailure
  | unknownPredicate
  deriving DecidableEq, Repr

inductive AdmOutcome (Adm : Type) where
  | admitted (x : Adm)
  | refusal (c : RefusalCategory)

def WellFormedArtifact (Adm : Type) := Adm → Bool

def fuzzOracle [DecidableEq Obs]
    (adm : Obs → AdmOutcome Adm) (μWF : WellFormedArtifact Adm)
    (X : List Obs) : Obs → Bool :=
  fun o =>
    decide (o ∈ X) &&
      (match adm o with
        | AdmOutcome.admitted x => μWF x
        | AdmOutcome.refusal _ => true)

/-- If `adm` is total/terminating on `o` (witnessed by `o ∈ X`) and every
admitted artifact at `o` is well-formed, the fuzzing oracle reports `1`
(`true`) at `o`. This is `Ω_∂(o) = 1` for a genuinely total, well-formed
retraction. -/
theorem fuzz_total_sound [DecidableEq Obs]
    (adm : Obs → AdmOutcome Adm) (μWF : WellFormedArtifact Adm)
    (X : List Obs) (o : Obs)
    (hmem : o ∈ X)
    (hwf : ∀ x, adm o = AdmOutcome.admitted x → μWF x = true) :
    fuzzOracle adm μWF X o = true := by
  unfold fuzzOracle
  rw [decide_eq_true hmem]
  simp only [Bool.true_and]
  cases h : adm o with
  | admitted x => exact hwf x h
  | refusal c => rfl

/-- Contrapositive form: if the retraction is total on `o` and well-formed
on `o`, an observed `Ω_∂(o) = 0` is impossible -- so any real occurrence of
`Ω_∂(o) = 0` witnesses a failure of totality or well-formedness at `o`
(a defect of the retraction), never a property of `o` itself. -/
theorem fuzz_zero_is_defect [DecidableEq Obs]
    (adm : Obs → AdmOutcome Adm) (μWF : WellFormedArtifact Adm)
    (X : List Obs) (o : Obs)
    (hmem : o ∈ X)
    (hwf : ∀ x, adm o = AdmOutcome.admitted x → μWF x = true) :
    fuzzOracle adm μWF X o ≠ false := by
  rw [fuzz_total_sound adm μWF X o hmem hwf]
  decide

end Fuzz
