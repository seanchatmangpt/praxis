import Praxis.Corpus.def_net
import Mathlib.Tactic.IntervalCases

/-!
# prop:safe

On a safe (1-bounded) net, the integer firing rule
`m' = m - m⁻_t + m⁺_t` coincides with the branchless bitset update
`enabled_tokens ← (enabled_tokens & ¬m⁻_t) | m⁺_t`, and the enabling test
coincides with the branchless subset check `(enabled & m⁻_t) ⊕ m⁻_t = 0`.

We reuse `Praxis.Corpus.DefNet.Net` (places/transitions/pre/post as already
defined) and `Praxis.Corpus.DefNet.Net.fire`/`Net.enabled` directly — no new
net structure is introduced (invariant 6).

"Safe (1-bounded)" is formalized as the hypothesis that every coordinate of
the marking, of each transition's preset, and of each transition's postset
is `≤ 1` (a `Fin p → ℕ` vector whose entries are all in `{0, 1}`, i.e. a
`ℕ`-encoded bitset). The "branchless bitset update"/"subset check" from the
paper are formalized with Lean/core's `Bool` operations (`&&`, `||`, `!`,
`Bool.xor`) via `decide (x = 1)` to read a `{0,1}`-valued `ℕ` coordinate as a
bit, and `Bool.toNat` to write a bit back as a `{0,1}`-valued `ℕ` coordinate —
these are core Lean, not axiomatized. No bespoke bitset/`BitVec` type is
introduced: the paper's "bitset" *is* a `Fin p → ℕ` marking restricted to
`{0,1}` entries, matching the existing `Marking p := Fin p → ℕ` from
`def:net`, so composing on top of that representation is the smallest diff
(invariant 6).

Both directions are proved by case-splitting each of the (finitely many,
`{0,1}`-valued) coordinates via `interval_cases` and closing the resulting
arithmetic/boolean identity by `decide`/`omega` — a genuine proof obligation
discharged from the definitions, not an axiom standing in for the
conclusion.
-/

namespace Praxis.Corpus.PropSafe

open Praxis.Corpus.DefNet

universe u

variable {p : ℕ} {T : Type u} [Fintype T]

/-- One coordinate is a *safe bit*: a `ℕ` value that is either `0` or `1`,
i.e. a `{0,1}`-valued coordinate of a 1-bounded (safe) marking/preset/postset,
exactly the paper's notion of a bit in `enabled_tokens`/`m⁻_t`/`m⁺_t`. -/
def SafeBit (n : ℕ) : Prop := n ≤ 1

/-- The firing rule `m' = m - m⁻_t + m⁺_t`, restricted to a coordinate `i`
where `m i`, `pre t i`, `post t i` are all safe bits and the resulting
coordinate `fire m t i` is itself a safe bit (i.e. firing this transition at
this place does not overflow past `1`, exactly the "safe (1-bounded)"
precondition under which the paper's bitset identity is claimed), coincides
with the branchless bitset update
`(m_bit && !pre_bit) || post_bit` read back as a `ℕ` via `Bool.toNat`. -/
theorem fire_eq_bitset_update (N : Net p T) (m : Marking p) (t : T) (i : Fin p)
    (hm : SafeBit (m i)) (hpre : SafeBit (N.pre t i)) (hpost : SafeBit (N.post t i))
    (hsafe : SafeBit (N.fire m t i)) :
    N.fire m t i
      = ((decide (m i = 1) && !decide (N.pre t i = 1)) || decide (N.post t i = 1)).toNat := by
  unfold SafeBit at hm hpre hpost hsafe
  unfold Net.fire at hsafe ⊢
  interval_cases (m i) <;> interval_cases (N.pre t i) <;> interval_cases (N.post t i) <;>
    simp_all

/-- The enabling test `N.pre t ≤ m` (coordinatewise, `Net.enabled`),
restricted to a coordinate `i` where `m i` and `pre t i` are safe bits,
coincides with the branchless subset check
`(enabled & m⁻_t) ⊕ m⁻_t = 0`, i.e. `Bool.xor (m_bit && pre_bit) pre_bit = false`. -/
theorem enabled_coord_iff_subset_check (m_i pre_i : ℕ)
    (hm : SafeBit m_i) (hpre : SafeBit pre_i) :
    pre_i ≤ m_i ↔ Bool.xor (decide (m_i = 1) && decide (pre_i = 1)) (decide (pre_i = 1)) = false := by
  unfold SafeBit at hm hpre
  interval_cases m_i <;> interval_cases pre_i <;> simp_all

/-- Full-marking version: on a safe net (every coordinate of `m`, `pre t`,
`post t`, and the fired result is a bit), the arithmetic firing rule agrees
coordinatewise with the branchless bitset update at every place. -/
theorem fire_eq_bitset_update_all (N : Net p T) (m : Marking p) (t : T)
    (hm : ∀ i, SafeBit (m i)) (hpre : ∀ i, SafeBit (N.pre t i))
    (hpost : ∀ i, SafeBit (N.post t i)) (hsafe : ∀ i, SafeBit (N.fire m t i)) :
    ∀ i, N.fire m t i
      = ((decide (m i = 1) && !decide (N.pre t i = 1)) || decide (N.post t i = 1)).toNat :=
  fun i => fire_eq_bitset_update N m t i (hm i) (hpre i) (hpost i) (hsafe i)

/-- Full-marking version: on a safe net (every coordinate of `m` and `pre t`
is a bit), `Net.enabled` agrees at every place with the branchless subset
check. -/
theorem enabled_iff_subset_check_all (N : Net p T) (m : Marking p) (t : T)
    (hm : ∀ i, SafeBit (m i)) (hpre : ∀ i, SafeBit (N.pre t i)) :
    N.enabled m t ↔
      ∀ i, Bool.xor (decide (m i = 1) && decide (N.pre t i = 1)) (decide (N.pre t i = 1)) = false := by
  unfold Net.enabled
  constructor
  · intro h i
    exact (enabled_coord_iff_subset_check (m i) (N.pre t i) (hm i) (hpre i)).mp (h i)
  · intro h i
    exact (enabled_coord_iff_subset_check (m i) (N.pre t i) (hm i) (hpre i)).mpr (h i)

end Praxis.Corpus.PropSafe
