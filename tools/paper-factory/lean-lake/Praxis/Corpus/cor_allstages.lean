import Praxis.Corpus.prop_monoid
import Praxis.Corpus.thm_freehom

/-!
# cor:allstages

`Φ_o(w) = Adml ↔ φ_o(s_i) = Adml` for every stage `s_i` in `w`; a single refusing
stage refuses the whole pipeline, and the aggregate word records every lane that
fired anywhere along `w`.

This is a direct corollary of `thm:freehom`'s homomorphism law
(`aggregateDenial_cons`, `aggregateDenial_nil`) composed with the fact -- already a
named Lean/core lemma, `BitVec.or_eq_zero_iff` -- that a bitwise-OR is zero iff both
operands are zero: no new algebra, no axioms. `DenialPolarity.val` unwraps to
`UInt64`, whose equality with `0` is decided via `UInt64.toBitVec_inj` against
`BitVec.or_eq_zero_iff` on the underlying `BitVec 64`.

The corollary is stated in two parts, exactly mirroring the prose: (1) the
iff-characterization of clean aggregation by clean stages, from which "a single
refusing stage refuses the whole pipeline" reads off as the contrapositive
(`not_admitted_of_exists_not_admitted`); (2) `Φ_o(w)` "records every lane that
fired anywhere along `w`" is the fact that a lane bit is set in `Φ_o(w)` iff it is
set in some `φ_o(s_i)`, via `DenialPolarity.compose`'s definitional bitwise-OR and
`BitVec.getLsbD_or` composed along the fold (`aggregateDenial_getLsbD`).
-/

namespace Pipeline

open DenialPolarity

variable {Stage : Type} (φ : Stage → DenialPolarity)

/-- The underlying bitwise-OR-is-zero fact, transported from `BitVec` through the
`DenialPolarity`/`UInt64` newtypes: `compose a b = Adml ↔ a = Adml ∧ b = Adml`. -/
theorem compose_eq_adml_iff (a b : DenialPolarity) :
    compose a b = Adml ↔ a = Adml ∧ b = Adml := by
  constructor
  · intro h
    cases a with | mk a =>
    cases b with | mk b =>
    have hv : (DenialPolarity.mk a).val ||| (DenialPolarity.mk b).val = (Adml).val :=
      congrArg DenialPolarity.val h
    simp only [compose] at h
    have hab : a ||| b = 0 := by
      have := congrArg DenialPolarity.val h
      simpa [Adml] using this
    have hbv : a.toBitVec ||| b.toBitVec = (0 : UInt64).toBitVec := by
      have := congrArg UInt64.toBitVec hab
      simpa using this
    have h0 : (0 : UInt64).toBitVec = (0 : BitVec 64) := by rfl
    rw [h0] at hbv
    obtain ⟨ha, hb⟩ := BitVec.or_eq_zero_iff.1 hbv
    refine ⟨?_, ?_⟩
    · congr 1
      exact UInt64.toBitVec_inj.1 (by simpa using ha)
    · congr 1
      exact UInt64.toBitVec_inj.1 (by simpa using hb)
  · rintro ⟨rfl, rfl⟩
    exact compose_adml_right Adml

/-- **cor:allstages**, part 1: `Φ_o(w) = Adml` iff `φ_o(s_i) = Adml` for every
stage `s_i` occurring in `w` -- the aggregate is clean exactly when every stage's
own denial word is clean. -/
theorem aggregateDenial_eq_adml_iff (w : Seq Stage) :
    aggregateDenial φ w = Adml ↔ ∀ s ∈ w, φ s = Adml := by
  induction w with
  | nil => simp [aggregateDenial_nil]
  | cons s w ih =>
    rw [aggregateDenial_cons, compose_eq_adml_iff, ih]
    constructor
    · rintro ⟨hs, hw⟩ s' hs'
      rcases List.mem_cons.1 hs' with h | h
      · rwa [h]
      · exact hw s' h
    · intro h
      exact ⟨h s (List.mem_cons_self ..), fun s' hs' => h s' (List.mem_cons_of_mem s hs')⟩

/-- **cor:allstages**, part 2 (contrapositive): a single refusing stage refuses
the whole pipeline -- if some stage `s_i ∈ w` has `φ_o(s_i) ≠ Adml`, the aggregate
`Φ_o(w) ≠ Adml` either. -/
theorem aggregateDenial_ne_adml_of_exists_ne_adml (w : Seq Stage)
    (h : ∃ s ∈ w, φ s ≠ Adml) : aggregateDenial φ w ≠ Adml := by
  intro hcontra
  obtain ⟨s, hs, hne⟩ := h
  exact hne ((aggregateDenial_eq_adml_iff φ w).1 hcontra s hs)

/-- **cor:allstages**, part 3: `Φ_o(w)` records every lane that fired anywhere
along `w` -- a byte-lane bit is set in the aggregate iff it is set in some
individual stage's denial word `φ_o(s_i)`. -/
theorem aggregateDenial_getLsbD (w : Seq Stage) (i : Fin 64) :
    (aggregateDenial φ w).val.toBitVec.getLsbD i.val
      ↔ ∃ s ∈ w, (φ s).val.toBitVec.getLsbD i.val := by
  induction w with
  | nil =>
    simp only [aggregateDenial_nil]
    constructor
    · intro h
      exact absurd h (by simp [Adml])
    · rintro ⟨s, hs, _⟩
      exact absurd hs (List.not_mem_nil)
  | cons s w ih =>
    rw [aggregateDenial_cons]
    show (compose (φ s) (aggregateDenial φ w)).val.toBitVec.getLsbD i.val ↔ _
    simp only [compose, UInt64.toBitVec_or, BitVec.getLsbD_or, Bool.or_eq_true]
    rw [ih]
    constructor
    · rintro (h | ⟨s', hs', h⟩)
      · exact ⟨s, List.mem_cons_self .., h⟩
      · exact ⟨s', List.mem_cons_of_mem s hs', h⟩
    · rintro ⟨s', hs', h⟩
      rcases List.mem_cons.1 hs' with rfl | hs'
      · exact Or.inl h
      · exact Or.inr ⟨s', hs', h⟩

end Pipeline
