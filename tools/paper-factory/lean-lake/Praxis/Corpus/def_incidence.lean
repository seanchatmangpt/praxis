import Praxis.Corpus.def_net
import Mathlib.Algebra.BigOperators.Group.Finset.Basic

/-!
# def:incidence

Let `N ∈ ℤ^{p × |T|}` have column `δ_t = m⁺_t - m⁻_t` for each transition `t`; a firing
sequence with Parikh vector `x ∈ ℤ_{≥0}^{|T|}` satisfies the state equation
`m = m₀ + N x`.

We represent:
- the incidence matrix `N` not as a bespoke matrix type but as a function
  `T → Fin p → ℤ` (Mathlib's standard encoding of a `p`-indexed family of columns,
  reusing the existing `Pi` type exactly as `Net.pre`/`Net.post` do in `def:net`);
  its `t`-column is `Net.delta`, defined by casting the `ℕ`-valued `post`/`pre`
  markings from `def:net` to `ℤ` and subtracting (ordinary `ℤ` subtraction, since
  the cast removes the truncation issue that `ℕ` monus has);
- the Parikh vector `x` as `T → ℤ` (again a `Pi` type, matching the incidence
  matrix's domain so the matrix-vector product below type-checks directly);
- the matrix-vector product `N x` at place `i` as the finite sum
  `∑ t, N t i * x t`, using Mathlib's `Finset.sum` over the ambient `Fintype T`
  (no new summation notion is introduced);
- the state equation itself as a `Prop`, `Net.stateEquation`, asserting
  `m i = m₀ i + ∑ t, N.delta t i * x t` for every place `i : Fin p`.
-/

namespace Praxis.Corpus.DefIncidence

open Praxis.Corpus.DefNet
open Finset

universe u

variable {p : ℕ} {T : Type u} [Fintype T]

/-- The incidence-matrix column for transition `t`: `δ_t = m⁺_t - m⁻_t`, as a
function `Fin p → ℤ` obtained by casting the `ℕ`-valued postset/preset markings
of `def:net` to `ℤ` and subtracting.

Declared under `Net`'s home namespace (`Praxis.Corpus.DefNet`) rather than this
file's own namespace, so that ordinary dot notation `N.delta` resolves for
`N : Net p T` (Lean looks up dot-notation fields in the type's declaration
namespace, not the call site's namespace). -/
def _root_.Praxis.Corpus.DefNet.Net.delta (N : Net p T) (t : T) : Fin p → ℤ :=
  fun i => (N.post t i : ℤ) - (N.pre t i : ℤ)

/-- The incidence matrix of a net, as a function `T → Fin p → ℤ` whose `t`-th
column is `Net.delta N t`. This is the standard `Pi`-type representation of a
`p × |T|` integer matrix indexed by transitions, matching how `def:net` already
represents `pre`/`post` as `T → Fin p → ℕ`. -/
def _root_.Praxis.Corpus.DefNet.Net.incidence (N : Net p T) : T → Fin p → ℤ :=
  N.delta

/-- The matrix-vector product `(N x)_i = ∑ t, N_{i,t} x_t`, using Mathlib's
`Finset.sum` over the ambient `Fintype T` instance (no bespoke summation type is
introduced). -/
def _root_.Praxis.Corpus.DefNet.Net.applyIncidence (N : Net p T) (x : T → ℤ) (i : Fin p) : ℤ :=
  ∑ t, N.incidence t i * x t

/-- The state equation `m = m₀ + N x`: for a firing sequence with Parikh vector
`x : T → ℤ`, the marking `m` equals the initial marking `m₀` plus the
incidence-matrix action on `x`, coordinatewise. -/
def _root_.Praxis.Corpus.DefNet.Net.stateEquation (N : Net p T) (m₀ m : Marking p) (x : T → ℤ) : Prop :=
  ∀ i : Fin p, (m i : ℤ) = (m₀ i : ℤ) + N.applyIncidence x i

end Praxis.Corpus.DefIncidence
