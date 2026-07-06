import Praxis.Corpus.cor_allstages
import Praxis.Corpus.thm_firedhom
import Praxis.Corpus.thm_freehom

/-!
# prop:fleet

Let a fleet of `N` manufactures produce aggregate words `Φ^(1),…,Φ^(N)`, packed as
`b^(r) = firedmap(Φ^(r)) ∈ {0,1}^8`; then `firedmap(Φ_o(w)) = ⋁_i firedmap(φ_o(s_i))`,
and a fleet admission sweep is one linear pass computing `⋁_r b^(r)`, returning `0`
iff every agent is admitted.

## Reuse discipline

* `Pipeline.aggregateDenial`, `Pipeline.aggregateDenial_nil`, `Pipeline.aggregateDenial_cons`
  are taken as-is from `Praxis.Corpus.thm_freehom` / `Praxis.Corpus.cor_allstages`
  (dependencies `thm:freehom`, `cor:allstages`).
* `DenialPolarity.firedmap`, `firedmap_adml`, `firedmap_compose` are taken as-is from
  `Praxis.Corpus.thm_firedhom` (dependency `thm:firedhom`): `firedmap` is already
  established there as a monoid homomorphism `(Deny, compose, Adml) → (BitVec 8, |||, 0)`.

The statement has two parts, proved in order:

1. `firedmap_aggregateDenial`: for a single manufacturer's pipeline `w`, packing the
   aggregate word commutes with folding `firedmap` over the individual stages via `|||`
   -- i.e. `firedmap(Φ_o(w)) = ⋁_i firedmap(φ_o(s_i))`. This is immediate by induction
   on `w` from `firedmap`'s homomorphism laws (`firedmap_adml`, `firedmap_compose`), no
   new axioms.
2. `fleetSweep_eq_zero_iff`: for a fleet of packed words `b^(1),…,b^(N) : BitVec 8`, one
   linear OR-fold ("the fleet admission sweep") is `0` iff every `b^(r) = 0` -- i.e. iff
   every manufacturer's pipeline is clean. Proved by induction on the list of packed
   words using the fixed-width bitwise fact `x ||| y = 0 ↔ x = 0 ∧ y = 0`, discharged
   for `BitVec 8` by `bv_decide` (bit-blasting to SAT over the 8-bit finite domain),
   exactly the same discipline `thm:firedhom` already used for the analogous `UInt64`
   fact (`DenialPolarity.or_eq_zero_iff`). No axiom is introduced anywhere in this file.
-/

namespace Pipeline

open DenialPolarity

variable {Stage : Type} (φ : Stage → DenialPolarity)

/-- **prop:fleet**, part 1: packing the aggregate denial word commutes with folding
`firedmap` over the individual stage-denials via bitwise-OR -- `firedmap(Φ_o(w))
= ⋁_i firedmap(φ_o(s_i))`, expressed as a right fold over the pipeline `w`. -/
theorem firedmap_aggregateDenial (w : Seq Stage) :
    firedmap (aggregateDenial φ w)
      = w.foldr (fun s acc => firedmap (φ s) ||| acc) (0#8) := by
  induction w with
  | nil => simp [aggregateDenial_nil, firedmap_adml]
  | cons s w ih => rw [aggregateDenial_cons, firedmap_compose, ih]; rfl

/-- The fixed-width bitwise fact underlying the fleet sweep: an 8-bit OR is zero iff
both operands are zero, discharged by `bv_decide` over the finite `BitVec 8` domain
(the same discipline `thm:firedhom` uses for `DenialPolarity.or_eq_zero_iff`). -/
theorem bitvec8_or_eq_zero_iff (x y : BitVec 8) : x ||| y = 0#8 ↔ x = 0#8 ∧ y = 0#8 := by
  bv_decide

/-- **prop:fleet**, part 2: a fleet admission sweep -- one linear OR-fold `⋁_r b^(r)`
over the packed words `b^(1),…,b^(N)` of a fleet of `N` manufactures -- returns `0`
iff every manufacturer's packed word `b^(r)` is `0`, i.e. iff every agent is admitted. -/
theorem fleetSweep_eq_zero_iff (bs : List (BitVec 8)) :
    bs.foldr (· ||| ·) (0#8) = 0#8 ↔ ∀ b ∈ bs, b = 0#8 := by
  induction bs with
  | nil => simp
  | cons b bs ih =>
    simp only [List.foldr_cons, bitvec8_or_eq_zero_iff, ih, List.mem_cons, forall_eq_or_imp]

/-- **prop:fleet**, combined: for a fleet of `N` manufactures with pipelines
`w^(1),…,w^(N) : Seq Stage`, packing each aggregate word as `b^(r) = firedmap(Φ_o(w^(r)))`
-- itself the OR-fold of each pipeline's own stage-denials by part 1 -- the fleet
admission sweep `⋁_r b^(r)` is `0` iff every manufacturer's packed word `b^(r)` is `0`,
i.e. iff every agent is admitted. Combines `firedmap_aggregateDenial` (part 1, packing
commutes with the OR-fold) with `fleetSweep_eq_zero_iff` (part 2, the sweep is zero iff
every packed word is). -/
theorem fleetSweep_eq_zero_iff_all_admitted (ws : List (Seq Stage)) :
    (ws.map (fun w => firedmap (aggregateDenial φ w))).foldr (· ||| ·) (0#8) = 0#8
      ↔ ∀ w ∈ ws, firedmap (aggregateDenial φ w) = 0#8 := by
  exact fleetSweep_eq_zero_iff (ws.map (fun w => firedmap (aggregateDenial φ w))) |>.trans
    ⟨fun h w hw => h _ (List.mem_map.2 ⟨w, hw, rfl⟩),
     fun h b hb => by obtain ⟨w, hw, rfl⟩ := List.mem_map.1 hb; exact h w hw⟩

end Pipeline
