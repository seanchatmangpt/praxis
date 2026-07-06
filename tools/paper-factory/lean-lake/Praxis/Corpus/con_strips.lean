import Praxis.Corpus.def_net
import Mathlib.Logic.Relation

/-!
# con:strips

Take the places to be the ground atoms; a ground action `t` with delete-effects
`m⁻_t` and add-effects `m⁺_t` is a transition, and `GroundProblem::find_plan`
searches marking space by BFS with the successor relation exactly transition
firing.

We reuse `Praxis.Corpus.DefNet.Net` directly: a `GroundProblem` with `p` ground
atoms and `T` ground actions *is* a `Net p T` (places = atoms, transitions =
actions, `pre`/`post` = delete/add effects). No new structure is introduced —
`GroundProblem` is an `abbrev` for `Net`, matching invariant 6 (smallest diff,
reuse first; no new subsystem where a const table/alias suffices).

The one-step successor relation used by `find_plan`'s search ("marking `m'` is
reached from `m` by firing some enabled transition") is exactly `Net.fire`
restricted to enabled transitions. `find_plan`'s BFS explores the transitive
closure of this relation; the *existence* of a plan from `m₀` to `mg` is
exactly reachability in that relation, which Mathlib already provides as
`Relation.ReflTransGen` (the reflexive-transitive closure of a relation) —
so no bespoke "BFS reachability" predicate is axiomatized either.

We do not model the BFS *algorithm* itself (queue, visited-set, frontier
order) as that is an implementation/search-strategy detail, not a
mathematical structure the paper's statement asks us to construct; the
statement is about the search *space* (transition firing as successor
relation) and what `find_plan` searches over, which `step`/`Reachable`
capture exactly.
-/

namespace Praxis.Corpus.ConStrips

open Praxis.Corpus.DefNet

universe u

/-- A ground STRIPS problem with `p` ground atoms as places and `T` ground
actions as transitions: literally a `Net p T`, since a ground action `t` with
delete-effects `m⁻_t` and add-effects `m⁺_t` *is* a transition. -/
abbrev GroundProblem (p : ℕ) (T : Type u) [Fintype T] := Net p T

variable {p : ℕ} {T : Type u} [Fintype T]

/-- One step of the search: marking `m'` is a successor of `m` iff some
transition `t` is enabled at `m` and firing it yields `m'`. This is exactly
the successor relation `find_plan`'s BFS explores. -/
def GroundProblem.step (P : GroundProblem p T) (m m' : Marking p) : Prop :=
  ∃ t : T, P.enabled m t ∧ P.fire m t = m'

/-- A plan exists from `m₀` to `mg` iff `mg` is reachable from `m₀` by
finitely many successor steps — the reflexive-transitive closure of `step`,
i.e. exactly what a BFS over the successor relation searches for. -/
def GroundProblem.reachable (P : GroundProblem p T) (m₀ mg : Marking p) : Prop :=
  Relation.ReflTransGen P.step m₀ mg

end Praxis.Corpus.ConStrips
