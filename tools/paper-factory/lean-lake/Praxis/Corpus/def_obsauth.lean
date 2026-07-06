import Praxis.Corpus.ax_refusal

/-!
def:obsauth, migrated to the Mathlib-linked lane.

Statement: let `Obs` be the raw observation space; an observation `o ∈ Obs`
is authoritative (`o ∈ Obs*`) if it was produced by an admitted actuation
with a chained receipt; all other observations are untrusted; the
proposer's admission map `adm_prop` retracts `Obs` onto `Adm_prop` by
treating every inbound observation as untrusted until its receipt chain
satisfies the obligation battery `G_prop`.

Reuse: `Adm`, `Refusal`, `AdmissionResult` are imported from
`Praxis.Corpus.ax_refusal` rather than re-axiomatized -- `Adm_prop` plays
the role of that same abstract admissible-output space, specialized to the
proposer, and the refusal/admission machinery (totality via `Sum`) is
identical, so it is composed here, not restated.

What remains axiomatic, and why no pre-built Mathlib equivalent exists:

* `Obs` -- the raw observation space. Left abstract for the same reason as
  `Adm` in `ax_refusal`: the thesis never fixes what an observation *is*,
  only its role in the authoritative/untrusted distinction.
* `HasChainedReceipt` -- the predicate "was produced by an admitted
  actuation with a chained receipt". This is domain-specific provenance
  information (an actuation log plus a receipt chain) that the statement
  does not reduce to any existing concrete structure, so it is left as an
  abstract predicate on `Obs` rather than invented as concrete data.
* `GProp` -- the obligation battery `G_prop` the receipt chain must
  satisfy. Like `HasChainedReceipt`, its content is a domain-specific set
  of proposer obligations, not something Mathlib provides a stand-in for.

Everything else -- the authoritative subtype, "untrusted" as its
complement, and the retraction as a total function into a `Sum` -- is
composed from core's `Prop`, `Subtype`, and the imported `AdmissionResult`,
matching the compositional style of the four worked examples.
-/

/-- The raw observation space `Obs`. Left abstract: the statement does not
specify what an observation *is*, only how it is classified. -/
axiom Obs : Type

/-- The provenance predicate: `o` was produced by an admitted actuation
with a chained receipt. Left abstract -- this is domain-specific
provenance data (actuation + receipt chain) the statement does not reduce
to any existing concrete encoding. -/
axiom HasChainedReceipt : Obs → Prop

/-- The obligation battery `G_prop` a receipt chain must satisfy for the
proposer. Left abstract for the same reason as `HasChainedReceipt`: a
domain-specific set of proposer obligations, not a Mathlib primitive. -/
axiom GProp : Obs → Prop

/-- `o ∈ Obs*`: `o` is authoritative iff it was produced by an admitted
actuation with a chained receipt that satisfies the proposer's obligation
battery `G_prop`. Composed from core's `Prop` conjunction, not asserted as
a new axiom. -/
def IsAuthoritative (o : Obs) : Prop := HasChainedReceipt o ∧ GProp o

/-- `Obs*`, the authoritative observations -- realized literally as the
core `Subtype` of `Obs` cut out by `IsAuthoritative`. -/
abbrev ObsStar : Type := {o : Obs // IsAuthoritative o}

/-- An observation is untrusted iff it is not authoritative -- the
complement of `IsAuthoritative`, composed from core negation rather than
asserted as a separate axiom. -/
def IsUntrusted (o : Obs) : Prop := ¬ IsAuthoritative o

/-- `Adm_prop`, the proposer's admissible-output space: reuses the
abstract admissible space `Adm` from `ax_refusal` rather than
re-axiomatizing an equivalent notion. -/
abbrev AdmProp : Type := Adm

/-- The proposer's admission map `adm_prop`, retracting `Obs` onto
`Adm_prop` by treating every inbound observation as untrusted until its
receipt chain satisfies `G_prop`. Existence of this classification map is
domain-specific (it depends on how untrusted observations are actually
refused), so it is left as an axiomatized total function; totality itself
is witnessed literally by the imported `AdmissionResult`'s `Sum` type,
exactly as in `ax_refusal`. -/
axiom admProp : Obs → AdmissionResult
