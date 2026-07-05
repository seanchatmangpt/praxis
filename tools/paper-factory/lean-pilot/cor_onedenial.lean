/-
cor:onedenial — In denial polarity, with D_ℓ = ¬P_ℓ the 64-agent word of
denial lane ℓ, the denied set is ⋁_ℓ D_ℓ and the admitted set its
complement; fleet admission is the word-parallel join of denial lanes
followed by one complement, and adding a ninth lane costs one more AND
(resp. OR) per 64 agents.

We reuse `SoAWord`, `Bitset`, and `admWord`/`admWord_iff_all_lanes` from
`thm:branchless` verbatim, and formalize the corollary as: the
branchless OR-fold of the negated lanes (the denied word) is, bit for
bit, exactly the complement of the admission word.
-/

def Agent := Fin 8 → Bool

def Bitset (N : Nat) := Fin N → Bool

structure SoAWord where
  lane : Fin 8 → Bitset 64

/-- The branchless admission word: bit `j` is the AND of all 8 lane bits
at position `j`, computed as a pure `&&`-fold (no branch). -/
def admWord (w : SoAWord) : Bitset 64 :=
  fun j =>
    w.lane 0 j && w.lane 1 j && w.lane 2 j && w.lane 3 j &&
    w.lane 4 j && w.lane 5 j && w.lane 6 j && w.lane 7 j

/-- Correctness of the branchless SoA admission sweep: bit `j` of `adm`
is `true` iff agent `j` has all eight lanes OK. -/
theorem admWord_iff_all_lanes (w : SoAWord) (j : Fin 64) :
    admWord w j = true ↔ ∀ ℓ : Fin 8, w.lane ℓ j = true := by
  constructor
  · intro h ℓ
    simp only [admWord, Bool.and_eq_true] at h
    obtain ⟨⟨⟨⟨⟨⟨⟨h0, h1⟩, h2⟩, h3⟩, h4⟩, h5⟩, h6⟩, h7⟩ := h
    match ℓ with
    | 0 => exact h0
    | 1 => exact h1
    | 2 => exact h2
    | 3 => exact h3
    | 4 => exact h4
    | 5 => exact h5
    | 6 => exact h6
    | 7 => exact h7
  · intro h
    simp only [admWord]
    have h0 := h 0
    have h1 := h 1
    have h2 := h 2
    have h3 := h 3
    have h4 := h 4
    have h5 := h 5
    have h6 := h 6
    have h7 := h 7
    simp [h0, h1, h2, h3, h4, h5, h6, h7]

/-- A single denial lane: `D ℓ = ¬ P ℓ`, the negation of admission lane
`ℓ`, at every bit position. -/
def denLane (w : SoAWord) (ℓ : Fin 8) : Bitset 64 :=
  fun j => ! w.lane ℓ j

/-- The denied word: the branchless OR-fold of the eight denial lanes,
`⋁_ℓ D_ℓ`, computed as a pure `||`-fold (no branch). -/
def denWord (w : SoAWord) : Bitset 64 :=
  fun j =>
    denLane w 0 j || denLane w 1 j || denLane w 2 j || denLane w 3 j ||
    denLane w 4 j || denLane w 5 j || denLane w 6 j || denLane w 7 j

/-- Corollary (one denial): the denied word is exactly the bitwise
complement of the admission word — `⋁_ℓ D_ℓ = ¬(⋀_ℓ P_ℓ)`. Fleet
admission is thus the word-parallel join of denial lanes followed by
one complement. -/
theorem denWord_eq_not_admWord (w : SoAWord) (j : Fin 64) :
    denWord w j = !(admWord w j) := by
  simp only [denWord, admWord, denLane]
  cases w.lane 0 j <;> cases w.lane 1 j <;> cases w.lane 2 j <;> cases w.lane 3 j <;>
    cases w.lane 4 j <;> cases w.lane 5 j <;> cases w.lane 6 j <;> cases w.lane 7 j <;> rfl
