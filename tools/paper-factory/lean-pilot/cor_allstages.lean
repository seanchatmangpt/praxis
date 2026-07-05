/-
cor:allstages

Φ_o(w) = Adml iff φ_o(s_i) = Adml for every stage s_i in w; a single
refusing stage refuses the whole pipeline, and the aggregate word records
every lane that fired anywhere along w.

This is a REAL proof obligation (corollary), proved in bare Lean 4 core,
as a direct consequence of prop:monoid and thm:freehom (`aggregateDenial`
as a right fold of `or`/`zero` over the pipeline).

Setup reused verbatim from thm_freehom.lean / prop_monoid.lean: `Stage`
abstract, `Pipeline := List Stage`, `Deny n := Fin n → Bool` with pointwise
`or` and all-false `zero`, and `aggregateDenial` as the right fold computing
Φ_o.
-/

opaque Stage : Type

abbrev Pipeline := List Stage

def Deny (n : Nat) := Fin n → Bool

namespace Deny

def or {n : Nat} (d d' : Deny n) : Deny n := fun i => d i || d' i
def zero (n : Nat) : Deny n := fun _ => false

theorem or_comm {n : Nat} (d d' : Deny n) : or d d' = or d' d := by
  funext i; simp [or, Bool.or_comm]

theorem or_assoc {n : Nat} (d d' d'' : Deny n) :
    or (or d d') d'' = or d (or d' d'') := by
  funext i; simp [or, Bool.or_assoc]

theorem zero_or {n : Nat} (d : Deny n) : or (zero n) d = d := by
  funext i; simp [or, zero]

theorem or_zero {n : Nat} (d : Deny n) : or d (zero n) = d := by
  funext i; simp [or, zero]

end Deny

variable {n : Nat} (φ : Stage → Deny n)

def aggregateDenial : Pipeline → Deny n
  | [] => Deny.zero n
  | s :: w => Deny.or (φ s) (aggregateDenial w)

/-- **cor:allstages, part 1**: `Φ_o(w) = Adml` iff every stage occurring in
`w` is itself admissible (`φ_o(s_i) = Adml`). A single refusing stage
therefore refuses the whole pipeline (the contrapositive of the forward
direction). -/
theorem aggregateDenial_eq_zero_iff (w : Pipeline) :
    aggregateDenial φ w = Deny.zero n ↔ ∀ s ∈ w, φ s = Deny.zero n := by
  induction w with
  | nil => simp [aggregateDenial]
  | cons s w ih =>
      have hunfold : aggregateDenial φ (s :: w) = Deny.or (φ s) (aggregateDenial φ w) := by
        simp [aggregateDenial]
      constructor
      · intro h t ht
        rw [hunfold] at h
        have hs0 : φ s = Deny.zero n := by
          funext i
          have hpair : φ s i = false ∧ aggregateDenial φ w i = false := by
            have := congrFun h i; simpa [Deny.or, Deny.zero] using this
          simpa using hpair.1
        have hw0 : aggregateDenial φ w = Deny.zero n := by
          funext i
          have hpair : φ s i = false ∧ aggregateDenial φ w i = false := by
            have := congrFun h i; simpa [Deny.or, Deny.zero] using this
          simpa using hpair.2
        rcases List.mem_cons.mp ht with rfl | ht'
        · exact hs0
        · exact (ih.mp hw0) t ht'
      · intro h
        have hs0 : φ s = Deny.zero n := h s (List.mem_cons.mpr (Or.inl rfl))
        have hw0 : aggregateDenial φ w = Deny.zero n :=
          ih.mpr (fun t ht => h t (List.mem_cons.mpr (Or.inr ht)))
        rw [hunfold, hs0, hw0, Deny.zero_or]

/-- **cor:allstages, part 2**: the aggregate denial records every lane
`i` that fired anywhere along `w` — `Φ_o(w)` refuses lane `i` iff some
stage `s ∈ w` refuses lane `i` under `φ_o`. -/
theorem aggregateDenial_bit_iff (w : Pipeline) (i : Fin n) :
    aggregateDenial φ w i = true ↔ ∃ s ∈ w, φ s i = true := by
  induction w with
  | nil => simp [aggregateDenial, Deny.zero]
  | cons s w ih =>
      have hunfold : aggregateDenial φ (s :: w) i = (φ s i || aggregateDenial φ w i) := by
        simp [aggregateDenial, Deny.or]
      rw [hunfold]
      constructor
      · intro h
        rcases Bool.or_eq_true_iff.mp h with h1 | h2
        · exact ⟨s, List.mem_cons.mpr (Or.inl rfl), h1⟩
        · obtain ⟨t, ht, hti⟩ := ih.mp h2
          exact ⟨t, List.mem_cons.mpr (Or.inr ht), hti⟩
      · rintro ⟨t, ht, hti⟩
        rcases List.mem_cons.mp ht with rfl | ht'
        · simp [hti]
        · have : aggregateDenial φ w i = true := ih.mpr ⟨t, ht', hti⟩
          simp [this]
