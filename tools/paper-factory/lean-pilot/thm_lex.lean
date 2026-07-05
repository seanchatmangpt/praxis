/-
thm:lex — Lexicographic order as the λ→∞ limit of a weighted linear
combination.

Statement: with C_λ(c) = Σ_{i=0}^k λ^{k-i} c_i, λ>1, |c_i| ≤ M, there is
Λ such that c ⪯_lex c' ↔ C_λ(c) ≤ C_λ(c') for all λ>Λ; as λ→∞ the
lawfulness (i.e. most-significant, index-0) coordinate's weight diverges.

This file formalizes the k=1 instantiation: a cost vector is a pair
`(c0, c1) : Nat × Nat` — `c0` the most-significant ("lawfulness") coordinate
matching `def:costvector`'s `c0`, `c1` a single bounded secondary cost,
matching `def:costvector`'s `csec`. Then `C_λ(c0,c1) = λ*c0 + c1`
(the k=1 case of `Σ λ^{k-i} c_i`), and `⪯_lex` is: compare `c0` first,
then `c1` on ties. The theorem is proved in full for this case, with a
real (non-`sorry`) proof; the argument is the same threshold-`Λ`
mechanism used at any fixed `k` (the general statement's proof for
`k > 1` proceeds by the same induction on coordinates, peeling off one
coordinate at a time exactly as the `c0`/`c1` split does here).
-/

/-- The weighted linear combination `C_λ(c0,c1) = λ*c0 + c1`, the `k=1`
case of `Σ_{i=0}^k λ^{k-i} c_i` (here `c0` carries weight `λ^1`, `c1`
weight `λ^0 = 1`). -/
def C (lam c0 c1 : Nat) : Nat := lam * c0 + c1

/-- The lexicographic order `⪯_lex` on `(c0,c1)` pairs: compare `c0`
first (most significant / "lawfulness" coordinate), and on ties compare
`c1`. -/
def lex (a0 a1 b0 b1 : Nat) : Prop := a0 < b0 ∨ (a0 = b0 ∧ a1 ≤ b1)

/-- **thm:lex** (k=1 instantiation): with every coordinate bounded by
`M`, the threshold `Λ = M` already suffices — for every `λ > M`,
`⪯_lex` and `C_λ(·) ≤ C_λ(·)` agree. As `λ → ∞` the weight `λ` on the
lawfulness coordinate `c0` diverges, so any fixed bound `M` on the
secondary coordinate is eventually dominated: this is exactly why a
threshold `Λ` (here `Λ = M`) exists. -/
theorem thm_lex (M a0 a1 b0 b1 : Nat)
    (ha1 : a1 ≤ M) (hb1 : b1 ≤ M) :
    ∃ Lambda : Nat, ∀ lam : Nat, Lambda < lam →
      (lex a0 a1 b0 b1 ↔ C lam a0 a1 ≤ C lam b0 b1) := by
  refine ⟨M, fun lam hlam => ?_⟩
  unfold C lex
  rcases Nat.lt_trichotomy a0 b0 with h | h | h
  · -- a0 < b0: both sides true.
    have hdiff : a0 + 1 ≤ b0 := h
    have hmul : lam * (a0 + 1) ≤ lam * b0 := Nat.mul_le_mul_left lam hdiff
    have : lam * a0 + lam ≤ lam * b0 := by
      have := hmul
      rw [Nat.mul_add, Nat.mul_one] at this
      exact this
    have hlt : lam * a0 + a1 ≤ lam * b0 + b1 := by
      have hM : a1 ≤ lam := Nat.le_trans ha1 (Nat.le_of_lt hlam)
      omega
    constructor
    · intro _; exact hlt
    · intro _; exact Or.inl h
  · -- a0 = b0: reduces to comparing a1, b1 directly.
    subst h
    constructor
    · intro hcase
      rcases hcase with hlt0 | ⟨_, hle1⟩
      · exact absurd hlt0 (Nat.lt_irrefl a0)
      · omega
    · intro hle
      have : a1 ≤ b1 := by omega
      exact Or.inr ⟨rfl, this⟩
  · -- a0 > b0: both sides false.
    have hdiff : b0 + 1 ≤ a0 := h
    have hmul : lam * (b0 + 1) ≤ lam * a0 := Nat.mul_le_mul_left lam hdiff
    have hge : lam * b0 + lam ≤ lam * a0 := by
      rw [Nat.mul_add, Nat.mul_one] at hmul
      exact hmul
    have hgt : lam * b0 + b1 < lam * a0 + a1 := by
      have hM : b1 ≤ lam := Nat.le_trans hb1 (Nat.le_of_lt hlam)
      omega
    constructor
    · intro hcase
      rcases hcase with hlt0 | ⟨heq0, _⟩
      · exact absurd hlt0 (by omega)
      · exact absurd heq0 (by omega)
    · intro hle
      exact absurd hle (by omega)
