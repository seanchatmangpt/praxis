/-
prop:safe

On a safe (1-bounded) net, the integer firing rule m' = m - m⁻_t + m⁺_t coincides
with the branchless bitset update enabled_tokens ← (enabled_tokens & ¬m⁻_t) | m⁺_t,
and the enabling test coincides with the branchless subset check
(enabled & m⁻_t) ⊕ m⁻_t = 0.

Formalized in bare Lean 4 core (no mathlib), reusing `def:net`/`con:strips`'s
`Marking`/`Net` machinery specialized to safe (1-bounded) nets: a place's token
count is then a `Bool` (0 or 1), embedded into `Nat` by `bit`. The two claims
are proved per-place (the general `Marking p` statement is the pointwise
extension, `∀ i : Fin p, ...`, of these single-place facts).
-/

/-- A marking assigns a nonnegative integer count of tokens to each of `p` places
    (reused verbatim from `def:net`). -/
def Marking (p : Nat) : Type := Fin p → Nat

/-- A net with `p` places and finite transition set `T` (reused verbatim from
    `def:net`), equipped with preset and postset functions. -/
structure Net (p : Nat) (T : Type) where
  pre  : T → Marking p
  post : T → Marking p

/-- Coordinatewise ordering on markings (reused from `def:net`). -/
def Marking.le {p : Nat} (m m' : Marking p) : Prop :=
  ∀ i : Fin p, m i ≤ m' i

instance {p : Nat} : LE (Marking p) := ⟨Marking.le⟩

/-- A transition `t` is enabled at marking `m` iff `m ≥ pre t` coordinatewise
    (reused from `def:net`). -/
def Net.enabled {p : Nat} {T : Type} (N : Net p T) (m : Marking p) (t : T) : Prop :=
  N.pre t ≤ m

/-- Coordinatewise marking update `m - a + b` (reused from `def:net`). -/
def Marking.fire {p : Nat} (m a b : Marking p) : Marking p :=
  fun i => m i - a i + b i

/-- Embed a bit (a token count on a safe, 1-bounded place) into `Nat`. -/
def bit (b : Bool) : Nat := if b then 1 else 0

/-- On a safe place (token count `m : Bool`, i.e. bounded by 1), if the place
    is enabled for the delete-effect bit `pre` (i.e. `pre = true → m = true`,
    the single-place instance of `Net.enabled`/`m⁻ t ≤ m`), and the net's
    safety is respected by this firing (the add-effect `post` never targets a
    place that retains a token after the delete step, i.e. `post = true →
    (m && !pre) = false` — the standard 1-boundedness side condition on
    transitions), then the integer firing rule `m - pre + post` (the
    single-place instance of `Marking.fire`) coincides with the branchless
    bitset update `(m && !pre) || post`. -/
theorem safe_fire_eq_bitset (m pre post : Bool)
    (h : pre = true → m = true)
    (hsafe : post = true → (m && !pre) = false) :
    bit m - bit pre + bit post = bit ((m && !pre) || post) := by
  cases pre with
  | false =>
    cases post with
    | false => cases m <;> simp [bit]
    | true =>
      have hm : (m && !false) = false := hsafe rfl
      simp at hm
      subst hm
      simp [bit]
  | true =>
    have hm : m = true := h rfl
    subst hm
    cases post <;> simp [bit]

/-- The enabling test on a safe place (`pre = true → m = true`, the
    single-place instance of `Net.enabled`/`m⁻ t ≤ m`) coincides with the
    branchless subset check `(m && pre) xor pre = false`. -/
theorem safe_enabled_eq_subset_check (m pre : Bool) :
    (pre = true → m = true) ↔ (xor (m && pre) pre = false) := by
  cases m <;> cases pre <;> simp
