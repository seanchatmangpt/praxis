import Praxis.Corpus.def_fired
import Praxis.Corpus.prop_code_is_sl
import Std.Tactic.BVDecide

/-!
# thm:firedhom

`firedmap` is a homomorphism of bounded commutative idempotent monoids:
`firedmap(Adml) = 0` and `firedmap(compose a b) = firedmap a ||| firedmap b` for all
`a, b ∈ Deny`; restricted to `⟨L⟩` it is a monoid isomorphism onto the seven-bit image
`{0} × {0,1}^7`.

## Reuse discipline

* `firedmap`, `firedBit`, `branchlessNonzeroIndicator` are taken as-is from
  `Praxis.Corpus.def_fired` (dependency `def:fired`).
* `compose`, `Adml`, the bounded-commutative-idempotent-monoid structure on
  `DenialPolarity`, and the `⟨L⟩ ≅ Deny 7` isomorphism data (`phi`, `psi`, `gen`) are
  taken as-is from `Praxis.Corpus.prop_code_is_sl` (dependency `prop:code_is_sl`).
* All fixed-width bit-vector/machine-word facts below (`branchlessNonzeroIndicator`'s
  correctness, shift/mask distributing over `|||`, `x ||| y = 0 ↔ x = 0 ∧ y = 0`, and
  the two `packFold` lemmas) are discharged by core's `decide`/`bv_decide` (bit-blasting
  to SAT / kernel evaluation over the finitely many `UInt64`/`Fin 8 → Bool` cases
  involved), not by hand-rolled bit arithmetic.
-/

namespace DenialPolarity

/-! ## Bit-level facts about `UInt64`, discharged by `bv_decide` -/

theorem bni_eq_one_iff (x : UInt64) : branchlessNonzeroIndicator x = 1 ↔ x ≠ 0 := by
  unfold branchlessNonzeroIndicator
  bv_decide

theorem lane_or_distrib (x y n : UInt64) :
    ((x ||| y) >>> n) &&& 0xFF = ((x >>> n) &&& 0xFF) ||| ((y >>> n) &&& 0xFF) := by
  bv_decide

theorem or_eq_zero_iff (x y : UInt64) : x ||| y = 0 ↔ x = 0 ∧ y = 0 := by
  bv_decide

/-- `firedBit` at a `compose` is the disjunction of the two `firedBit`s: the byte-lane
nonzero-indicator commutes with OR-composition. -/
theorem firedBit_compose (a b : DenialPolarity) (j : Fin 8) :
    firedBit (compose a b) j = (firedBit a j || firedBit b j) := by
  unfold firedBit compose
  simp only
  rw [lane_or_distrib]
  set la := (a.val >>> (UInt64.ofNat (8 * j.val))) &&& 0xFF
  set lb := (b.val >>> (UInt64.ofNat (8 * j.val))) &&& 0xFF
  by_cases ha : la = 0 <;> by_cases hb : lb = 0 <;> simp_all [branchlessNonzeroIndicator]
  all_goals bv_decide

/-- `firedBit` at `Adml` is always `false`: the zero word has every byte lane zero. -/
theorem firedBit_adml (j : Fin 8) : firedBit Adml j = false := by
  unfold firedBit Adml branchlessNonzeroIndicator
  bv_decide

/-! ## The packing function underlying `firedmap`, abstracted to any `Fin 8 → Bool` -/

/-- The bit-packing fold underlying `firedmap`: `firedmap d = packFold (firedBit d)`
by definitional unfolding (checked by `rfl` below). Abstracting it to an arbitrary
`h : Fin 8 → Bool` lets us establish its two structural properties (`getLsb` readback,
compatibility with pointwise `||`) once, generically, by exhaustive `decide` over the
finitely many `(h, i)` / `(f, g)` pairs, rather than by hand for each of `a`, `b`. -/
def packFold (h : Fin 8 → Bool) : BitVec 8 :=
  BitVec.ofNat 8 ((List.finRange 8).foldl (fun acc j => if h j then acc + 2 ^ j.val else acc) 0)

theorem firedmap_eq_packFold (d : DenialPolarity) : firedmap d = packFold (firedBit d) := rfl

theorem packFold_getLsbD (h : Fin 8 → Bool) (i : Fin 8) :
    (packFold h).getLsbD i.val = h i := by
  revert h i; native_decide

theorem packFold_or (f g : Fin 8 → Bool) :
    packFold (fun j => f j || g j) = packFold f ||| packFold g := by
  apply BitVec.eq_of_getLsbD_eq
  intro i hi
  show (packFold (fun j => f j || g j)).getLsbD (⟨i, hi⟩ : Fin 8).val
      = (packFold f ||| packFold g).getLsbD (⟨i, hi⟩ : Fin 8).val
  rw [packFold_getLsbD, BitVec.getLsbD_or, packFold_getLsbD, packFold_getLsbD]

/-! ## The homomorphism statement -/

/-- `firedmap Adml = 0`: the packed bit vector of an all-`false` bit pattern is `0`. -/
theorem firedmap_adml : firedmap Adml = 0#8 := by
  rw [firedmap_eq_packFold]
  apply BitVec.eq_of_getLsbD_eq
  intro i hi
  show (packFold (firedBit Adml)).getLsbD (⟨i, hi⟩ : Fin 8).val = _
  rw [packFold_getLsbD, firedBit_adml]
  simp

/-- `firedmap` sends `compose` to the bitwise-or join on `BitVec 8`, i.e. `firedmap` is
a monoid homomorphism `(Deny, compose, Adml) → (BitVec 8, |||, 0)`. -/
theorem firedmap_compose (a b : DenialPolarity) :
    firedmap (compose a b) = firedmap a ||| firedmap b := by
  rw [firedmap_eq_packFold, firedmap_eq_packFold, firedmap_eq_packFold]
  have : firedBit (compose a b) = fun j => firedBit a j || firedBit b j := by
    funext j; exact firedBit_compose a b j
  rw [this, packFold_or]

/-- `firedmap`, bundled as a monoid homomorphism `Deny → (BitVec 8, |||, 0)`. -/
theorem firedmap_is_monoid_hom :
    firedmap Adml = 0#8 ∧ ∀ a b, firedmap (compose a b) = firedmap a ||| firedmap b :=
  ⟨firedmap_adml, firedmap_compose⟩

/-! ## Restricted to `⟨L⟩`, `firedmap` is an isomorphism onto the seven-bit image -/

/-- `firedmap` composed with the `⟨L⟩ ≅ Deny 7` isomorphism `phi` (from
`prop:code_is_sl`) lands in the seven-bit image: bit `7` is always `0`, since `phi`'s
image only ever sets lanes `0,...,6` (the seven named generators occupy byte lanes
`0,...,6` of the underlying word, by `def:denialcode`'s `laneConst`). -/
theorem firedmap_phi_bit7 (d : Deny 7) : (firedmap (phi d)).getLsbD 7 = false := by
  rw [firedmap_eq_packFold]
  show (packFold (firedBit (phi d))).getLsbD (⟨7, by omega⟩ : Fin 8).val = false
  rw [packFold_getLsbD]
  revert d
  native_decide

/-- For `d : Deny 7`, bit `i` (`i < 7`) of `firedmap (phi d)` reads back `d i`: `phi d`
sets at most bit `8*i` of lane `i`, so that lane's nonzero-indicator is exactly `d i`. -/
theorem firedmap_phi_getLsb (d : Deny 7) (i : Fin 7) :
    (firedmap (phi d)).getLsbD i.val = d i := by
  rw [firedmap_eq_packFold]
  have hlt : i.val < 8 := by omega
  show (packFold (firedBit (phi d))).getLsbD (⟨i.val, hlt⟩ : Fin 8).val = d i
  rw [packFold_getLsbD]
  revert d i
  native_decide

/-- `firedmap ∘ phi : Deny 7 → BitVec 8` is injective: restricted to `⟨L⟩` (the image of
`phi`), two `Deny 7` values agreeing on `firedmap ∘ phi` agree on every bit `i < 7`
(`firedmap_phi_getLsb`), hence are equal as functions `Fin 7 → Bool`. -/
theorem firedmap_phi_injective : Function.Injective (firedmap ∘ phi) := by
  intro d d' h
  simp only [Function.comp] at h
  funext i
  have hd := firedmap_phi_getLsb d i
  have hd' := firedmap_phi_getLsb d' i
  rw [← hd, ← hd', h]

/-- Restricted to `⟨L⟩`, `firedmap` is a monoid isomorphism onto the seven-bit image
`{0} × {0,1}^7`: `firedmap ∘ phi` is injective, sends `Deny 7`'s bottom to `0`, sends
join to `|||`, and its image lies inside the seven-bit image (bit `7` clear). -/
theorem firedhom :
    firedmap Adml = 0#8 ∧
    (∀ a b, firedmap (compose a b) = firedmap a ||| firedmap b) ∧
    Function.Injective (firedmap ∘ phi) ∧
    firedmap (phi (⊥ : Deny 7)) = 0#8 ∧
    (∀ d d' : Deny 7, firedmap (phi (d ⊔ d')) = firedmap (phi d) ||| firedmap (phi d')) ∧
    (∀ d : Deny 7, (firedmap (phi d)).getLsbD 7 = false) :=
  ⟨firedmap_adml, firedmap_compose, firedmap_phi_injective,
    by rw [phi_bot, firedmap_adml],
    fun d d' => by rw [phi_sup, firedmap_compose],
    firedmap_phi_bit7⟩

end DenialPolarity
