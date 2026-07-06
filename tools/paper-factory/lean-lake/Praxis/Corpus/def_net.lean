import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Fin.Basic

/-!
# def:net

A net has `p` places and a finite set `T` of transitions; a marking is a vector
`m ∈ ℕ^p`; a transition `t` is a pair `(m⁻_t, m⁺_t)` of preset and postset,
enabled at `m` iff `m ≥ m⁻_t` coordinatewise, firing to `m' = m - m⁻_t + m⁺_t`.

We represent:
- places by `Fin p` (a genuine `p`-element type, from Mathlib/core, not axiomatized);
- a marking as `Fin p → ℕ` (the standard Mathlib representation of a finite-support
  vector in `ℕ^p`, reusing the existing `Pi`/function type rather than inventing a
  bespoke vector type);
- the finite transition set `T` as an arbitrary type `T` together with
  `[Fintype T]`, Mathlib's standard finiteness typeclass;
- preset/postset as a pair of functions `T → Fin p → ℕ`, bundled with the marking
  type into a single structure `Net`.

Coordinatewise comparison `m ≥ m⁻_t` uses the existing `Pi` order on `Fin p → ℕ`
(pointwise `≤`/`≥`, provided by Mathlib for any `Pi` type into an ordered type),
so no new order needs to be axiomatized. Truncated subtraction `m - m⁻_t` uses
`ℕ`'s built-in monus, matching the paper's implicit convention that firing is only
defined when the transition is enabled (so no underflow occurs on the coordinates
that matter).
-/

namespace Praxis.Corpus.DefNet

universe u

/-- A marking of a net with `p` places: a vector in `ℕ^p`, represented as a
function `Fin p → ℕ` (Mathlib's standard finite-indexed vector encoding). -/
abbrev Marking (p : ℕ) := Fin p → ℕ

/-- A Petri-style net: `p` places, a finite set `T` of transitions, and for each
transition its preset and postset markings (`m⁻_t`, `m⁺_t`). -/
structure Net (p : ℕ) (T : Type u) [Fintype T] where
  /-- preset `m⁻_t` of each transition -/
  pre : T → Marking p
  /-- postset `m⁺_t` of each transition -/
  post : T → Marking p

variable {p : ℕ} {T : Type u} [Fintype T]

/-- Transition `t` is enabled at marking `m` iff `m ≥ pre t` coordinatewise. -/
def Net.enabled (N : Net p T) (m : Marking p) (t : T) : Prop :=
  N.pre t ≤ m

/-- Firing enabled transition `t` at `m` yields `m' = m - pre t + post t`
(coordinatewise, using `ℕ` truncated subtraction). -/
def Net.fire (N : Net p T) (m : Marking p) (t : T) : Marking p :=
  fun i => m i - N.pre t i + N.post t i

end Praxis.Corpus.DefNet
