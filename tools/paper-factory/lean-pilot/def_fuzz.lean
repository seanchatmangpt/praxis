/-
Label: def:fuzz
Kind: definition

For the total admission retraction `adm : Obs ⇀ Adm ∪ {Rfsl}`, the fuzzing
oracle `Ω_∂` maps generated `o ∈ Obs` to `1` iff `adm(o)` terminates and
yields either an admitted `x ∈ Adm` with well-formed artifact `μ(x)`, or a
refusal `Rfsl` carrying a category in the eight-bucket taxonomy; `Ω_∂(o) = 0`
iff `adm` crashes, diverges, or returns an unlabelled outcome.

We model:
- `Obs`, `Adm` as arbitrary types.
- the eight-bucket refusal taxonomy as an inductive type `RefusalCategory`
  with exactly eight constructors.
- a refusal outcome as `Rfsl` carrying one such category.
- the well-formedness of an artifact `μ x` as a predicate `WellFormed`.
- the partial admission retraction `adm` on a finite domain (given as a
  `List Obs`) as a total function into the outcome type
  `Adm ⊕ RefusalCategory`, together with an explicit "defined on X" witness
  list, so that "terminates and yields an admitted/refusal outcome" is
  represented by `adm` actually being total (an ordinary Lean function),
  while divergence/crash/unlabelled outcomes on inputs outside the modelled
  domain are represented by absence from that witness list.
- the fuzzing oracle `Ω_∂` as a function `Obs → Bool` built from `adm`,
  `μ`, `WellFormed`, and the domain witness list `X`: it returns `true`
  exactly when `o ∈ X` (i.e. `adm` is known to terminate on `o`) and the
  outcome is either a well-formed admitted artifact or *some* refusal
  category (any of the eight buckets counts as labelled), and `false`
  otherwise.
-/

section Fuzz

variable {Obs Adm : Type}

/-- The eight-bucket refusal taxonomy. -/
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

/-- The outcome of the admission retraction: either an admitted value with
its artifact, or a refusal carrying a taxonomy category. -/
inductive AdmOutcome (Adm : Type) where
  | admitted (x : Adm)
  | refusal (c : RefusalCategory)

/-- Well-formedness of the artifact produced for an admitted value. -/
def WellFormedArtifact (Adm : Type) := Adm → Bool

/-- The fuzzing oracle `Ω_∂`. Given:
- `adm : Obs → AdmOutcome Adm`, the total admission retraction (modelled as
  an ordinary total Lean function, since totality/termination on the
  observed domain is exactly the hypothesis being fuzzed for),
- `μWF : WellFormedArtifact Adm`, the well-formedness check on artifacts,
- `X : List Obs`, the finite set of observations on which `adm` is known
  (witnessed) to actually terminate with a labelled outcome,
`Ω_∂` returns `true` on `o` iff `o ∈ X` and `adm o` is either a
well-formed admitted artifact or any refusal category (all eight buckets
count as "labelled"), and `false` otherwise -- including for every
`o ∉ X`, which stands for the crash/divergence/unlabelled case. -/
def fuzzOracle [DecidableEq Obs]
    (adm : Obs → AdmOutcome Adm) (μWF : WellFormedArtifact Adm)
    (X : List Obs) : Obs → Bool :=
  fun o =>
    decide (o ∈ X) &&
      (match adm o with
        | AdmOutcome.admitted x => μWF x
        | AdmOutcome.refusal _ => true)

end Fuzz
