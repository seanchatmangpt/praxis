/-
con:strips

Take the places to be the ground atoms; a ground action `t` with delete-effects
`m⁻ t` and add-effects `m⁺ t` is a transition, and `GroundProblem::find_plan`
searches marking space by BFS with the successor relation exactly transition
firing.

Formalized in bare Lean 4 core (no mathlib), reusing `def:net`'s `Net`/`Marking`
machinery: a STRIPS ground problem over `p` ground atoms and action set `T` is
exactly a `Net p T`, whose `pre`/`post` play the role of delete effects and
add effects, and whose reachable markings under `Net.fire` are the states
searched by BFS.
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

/-- A STRIPS ground problem over `p` ground atoms (places) and ground action set
    `T` (transitions): places are the ground atoms, a ground action `t` is a
    transition whose `pre` records its delete-effects `m⁻ t` (the atoms it
    requires/removes) and whose `post` records its add-effects `m⁺ t`. This is
    definitionally `def:net`'s `Net p T` — the construction is the reuse. -/
def Strips (p : Nat) (T : Type) : Type := Net p T

/-- View a STRIPS problem as its underlying net, exposing `enabled`/`fire`. -/
def Strips.toNet {p : Nat} {T : Type} (S : Strips p T) : Net p T := S

/-- A ground action `t` is applicable at marking (state) `m` iff `m` satisfies
    its delete-effects precondition `m⁻ t`, i.e. `t` is enabled in the net. -/
def Strips.applicable {p : Nat} {T : Type} (S : Strips p T) (m : Marking p) (t : T) : Prop :=
  Net.enabled S.toNet m t

/-- Applying an applicable ground action `t` at state `m` yields the successor
    state `m - m⁻ t + m⁺ t` — exactly transition firing, which is the successor
    relation `find_plan`'s BFS searches over. -/
def Strips.apply {p : Nat} {T : Type} (S : Strips p T) (m : Marking p) (t : T) : Marking p :=
  Net.fire S.toNet m t
