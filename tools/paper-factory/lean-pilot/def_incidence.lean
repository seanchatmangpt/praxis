/-
def:incidence

Let N ∈ Z^{p×|T|} have column δ_t = m⁺_t − m⁻_t for each transition t; a firing
sequence with Parikh vector x ∈ Z_{≥0}^{|T|} satisfies the state equation
m = m₀ + N x.

Formalized in bare Lean 4 core (no mathlib), building on `Net`/`Marking` from
def:net (repeated below verbatim since this file is checked standalone with no
import mechanism). Since bare core has no `Fintype`/`Finset.sum`, the finite
transition set `T` is taken concretely as `Fin n` and the sum over transitions
is a hand-rolled recursive fold over `Fin n`.

The incidence "matrix" is represented as its column function `Fin n → (Fin p →
Int)`, i.e. for each transition `t` the integer vector `post t − pre t` (lifted
from `Nat` to `Int` so the subtraction is exact rather than truncated). A
Parikh vector is a function `Fin n → Nat` counting firings of each transition.
The state equation is the proposition that a given marking `m` equals `m₀`
shifted by the incidence matrix applied to the Parikh vector `x`, i.e.
`m i = m₀ i + Σ_t x t * N t i` for every place `i`.
-/

/-- A marking assigns a nonnegative integer count of tokens to each of `p` places. -/
def Marking (p : Nat) : Type := Fin p → Nat

/-- A net with `p` places and `n` transitions (indexed by `Fin n`), equipped
    with preset and postset functions recording, for each transition, the
    marking it consumes (`pre`) and produces (`post`). -/
structure Net (p n : Nat) where
  pre  : Fin n → Marking p
  post : Fin n → Marking p

/-- A marking lifted to integers (needed since incidence columns can be negative). -/
def IMarking (p : Nat) : Type := Fin p → Int

/-- The incidence matrix of a net `N`, given as its column function: for each
    transition `t`, the vector `δ_t = post t − pre t` (as integers). -/
def Net.incidence {p n : Nat} (N : Net p n) : Fin n → IMarking p :=
  fun t i => (Int.ofNat (N.post t i)) - (Int.ofNat (N.pre t i))

/-- A Parikh vector counts, for each transition, how many times it fires. -/
def Parikh (n : Nat) : Type := Fin n → Nat

/-- Recursive sum, over the first `k` transitions (`k ≤ n`), of `x t * N.incidence t i`. -/
def incidenceSumAux {p n : Nat} (N : Net p n) (x : Parikh n) (i : Fin p) : Nat → Int
  | 0 => 0
  | k + 1 =>
    if h : k < n then
      let t : Fin n := ⟨k, h⟩
      incidenceSumAux N x i k + (Int.ofNat (x t)) * (N.incidence t i)
    else
      incidenceSumAux N x i k

/-- The full sum `Σ_t x t * N.incidence t i` over all `n` transitions. -/
def Net.incidenceSum {p n : Nat} (N : Net p n) (x : Parikh n) (i : Fin p) : Int :=
  incidenceSumAux N x i n

/-- The state-equation predicate: marking `m` is reachable from `m₀` via Parikh
    vector `x` under the incidence matrix of `N`, i.e. `m = m₀ + N x`
    coordinatewise over `Int`. -/
def Net.stateEquation {p n : Nat} (N : Net p n) (m₀ m : Marking p) (x : Parikh n) : Prop :=
  ∀ i : Fin p, (Int.ofNat (m i)) = (Int.ofNat (m₀ i)) + N.incidenceSum x i
