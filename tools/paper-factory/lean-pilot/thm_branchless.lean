/-
thm:branchless — Branchless SoA admission word.

Under the SoA layout, the admissibility of 64 agents is computed by the
single branchless word expression `adm = P_1 & ... & P_8`; bit `j` of
`adm` is 1 iff agent `j` has all eight lanes OK.

We reuse `SoAWord` from `def:layouts` (each lane `ℓ : Fin 8` is a
`Bitset 64 := Fin 64 → Bool`), and reuse the branchless (pure `&&`-fold,
no `if`) shape from `con:agent8`'s `byteAdmitted`. The theorem is the
correctness statement: the branchless word-AND admission bit for agent
`j` is `true` iff every one of the 8 lanes is `true` for agent `j`.
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
