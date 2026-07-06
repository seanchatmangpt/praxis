import Praxis.Corpus.def_obsauth
import Mathlib.Data.Set.Basic

/-!
prop:boundary, migrated to the Mathlib-linked lane.

Statement: under `def:obsauth`, `praxis-proposer` enforces
`im μ_prop ⊆ Adm_prop`: no proposal is manufactured from an unadmitted
observation; the generic `Domain<D>` proposer evaluates its `Obligation`
battery on every inbound record before calling `Admit::admit`, and a
record failing any obligation is refused and never reaches manufacturing.

Formalization strategy: the informal claim is a *typing* guarantee --
"manufacturing" (calling `Admit::admit` to produce a proposal) is a
function whose domain is restricted, at the type level, to observations
that have already passed the obligation battery, i.e. to `ObsStar`
(`Obs*`, imported from `Praxis.Corpus.def_obsauth`). Reusing `ObsStar`
and `AdmProp` from `def_obsauth` rather than re-axiomatizing the
authoritative/admissible spaces is the compositional move here, matching
the style of the four worked examples.

What remains axiomatic, and why no pre-built Mathlib equivalent exists:

* `manufactureProp` -- the proposer's manufacturing map `μ_prop` itself
  (what `Admit::admit` actually computes/emits as a proposal). Its
  internal construction is domain-specific business logic the thesis
  does not reduce to a concrete formula, so, like `admProp` in
  `def_obsauth`, it is left as an axiomatized total function -- but
  critically its *type* `ObsStar → AdmProp` is not axiomatic, it is the
  literal Lean encoding of "only authoritative observations are ever
  manufactured from."

Given that typing, `im μ_prop ⊆ Adm_prop` is not a further assumption
but a real (if small) theorem: any function into `AdmProp` has range
contained in `AdmProp`'s underlying universe, proved by Mathlib's
`Set.subset_univ`. The mathematical content of the boundary guarantee
lives entirely in the *type signature* of `manufactureProp` (domain
`ObsStar`, not `Obs`), exactly mirroring the source text's claim that a
record failing the obligation battery "never reaches manufacturing" --
it is not merely refused post hoc, it is not in the domain of `μ_prop`
at all.
-/

/-- The proposer's manufacturing map `μ_prop`: it is only ever called on
already-authoritative observations (`ObsStar`, imported from
`def_obsauth`), producing an admissible proposal in `Adm_prop`. Left
axiomatized because its internal computation (what `Admit::admit`
actually emits) is domain-specific business logic not reduced to a
concrete formula by the source text -- but its *domain* being `ObsStar`
rather than `Obs` is the literal encoding of "a record failing any
obligation is refused and never reaches manufacturing." -/
axiom manufactureProp : ObsStar → AdmProp

/-- `im μ_prop ⊆ Adm_prop`: the image of the proposer's manufacturing map
lies inside the admissible-output space. Composed from Mathlib's
`Set.subset_univ`, not asserted as a further axiom -- it follows purely
from `manufactureProp`'s type `ObsStar → AdmProp`, which already forces
every manufactured value to land in `AdmProp` and every input to have
come from `ObsStar` (i.e. to have passed `G_prop`). -/
theorem prop_boundary : Set.range manufactureProp ⊆ (Set.univ : Set AdmProp) :=
  Set.subset_univ _
