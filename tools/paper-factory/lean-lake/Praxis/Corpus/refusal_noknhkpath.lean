import Mathlib.Data.Finset.Basic

/-!
`refusal:noknhkpath` -- No path dependencies on `knhk`.

Thesis statement (verbatim, informal): the leanest candidate `knhk` crate
drags `tokio`/`reqwest`/`opentelemetry` in transitively; the mu-kernel ships
property-testing libraries as regular dependencies and uses unsafe
transmutes; the workspace does not build as a whole. Ports of 50--200-line
designs, tagged `PORT(knhk)` with per-item deltas, cost less than the
supply chain. Drift is greppable.

This is a dependency-policy refusal recorded from a live probe, not an
abstract mathematical claim, so the formalization stays as concrete as the
statement itself: dependency sets are genuine Mathlib `Finset String`
values (crate names), not a bespoke axiomatized "DependencySet" type --
matching `DefReceipt`'s discipline of composing from Nat/String/Prod/Sigma
rather than axiomatizing a container Mathlib already has.

The one thing that *is* axiomatized is the actual result of the live probe
(which crates a real `cargo tree` run reported) -- an external empirical
fact about a specific crate at a specific point in time, not something
derivable inside Lean, exactly as `DefReceipt` axiomatizes a real
cryptographic hash function rather than proving one exists.
-/

/-- Crate/package names, reusing `String` directly rather than a
bespoke identifier type. -/
abbrev CrateName := String

/-- The named, concrete set of forbidden transitive dependencies this
refusal is based on. -/
def disallowedTransitiveDeps : Finset CrateName :=
  {"tokio", "reqwest", "opentelemetry"}

/-- The transitive dependency closure of the leanest `knhk` candidate
crate, as recorded by the live dependency probe. External empirical
fact, hence an axiom. -/
axiom knhkTransitiveDeps : Finset CrateName

/-- The probe's finding: the candidate's transitive closure actually
overlaps the disallowed set (this is the concrete evidence backing the
refusal, e.g. `tokio` showing up under `cargo tree` for the candidate). -/
axiom knhk_drags_disallowed_deps :
    (knhkTransitiveDeps ∩ disallowedTransitiveDeps).Nonempty

/-- "No path dependency on `knhk`" as a decidable proposition: the
crate's transitive closure would have to be disjoint from the
disallowed set. -/
def NoPathDependencyOnKnhk : Prop :=
  Disjoint knhkTransitiveDeps disallowedTransitiveDeps

/-- The refusal itself: standing on the probe evidence, `knhk` is *not*
a no-path dependency, i.e. a forbidden transitive path always exists.
A genuine theorem derived from the axiomatized evidence, not an axiom
pretending to be the conclusion. -/
theorem refusal_noknhkpath : ¬ NoPathDependencyOnKnhk :=
  Finset.Nonempty.not_disjoint knhk_drags_disallowed_deps
