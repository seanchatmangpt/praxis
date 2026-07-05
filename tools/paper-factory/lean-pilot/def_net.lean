/-
def:net

A net has `p` places and a finite set `T` of transitions; a marking is a vector
`m : Fin p → ℕ` (coordinatewise nonnegative integers); a transition `t` is a pair
`(m⁻ t, m⁺ t)` of preset and postset, enabled at `m` iff `m ≥ m⁻ t` coordinatewise,
firing to `m' = m - m⁻ t + m⁺ t`.

Formalized in bare Lean 4 core (no mathlib): places are indexed by `Fin p`,
markings are functions `Fin p → Nat`, and a net bundles the transition set `T`
together with preset/postset functions `T → (Fin p → Nat)`.
-/

/-- A marking assigns a nonnegative integer count of tokens to each of `p` places. -/
def Marking (p : Nat) : Type := Fin p → Nat

/-- A net with `p` places and finite transition set `T`, given as a `Fintype`,
    equipped with preset and postset functions recording, for each transition,
    the marking it consumes (`pre`) and produces (`post`). -/
structure Net (p : Nat) (T : Type) where
  pre  : T → Marking p
  post : T → Marking p

/-- Coordinatewise ordering on markings. -/
def Marking.le {p : Nat} (m m' : Marking p) : Prop :=
  ∀ i : Fin p, m i ≤ m' i

instance {p : Nat} : LE (Marking p) := ⟨Marking.le⟩

/-- A transition `t` is enabled at marking `m` iff `m ≥ pre t` coordinatewise. -/
def Net.enabled {p : Nat} {T : Type} (N : Net p T) (m : Marking p) (t : T) : Prop :=
  N.pre t ≤ m

/-- Coordinatewise marking update `m - a + b`, using truncated subtraction on `Nat`. -/
def Marking.fire {p : Nat} (m a b : Marking p) : Marking p :=
  fun i => m i - a i + b i

/-- Firing an enabled transition `t` at `m` yields `m' = m - pre t + post t`. -/
def Net.fire {p : Nat} {T : Type} (N : Net p T) (m : Marking p) (t : T) : Marking p :=
  Marking.fire m (N.pre t) (N.post t)
