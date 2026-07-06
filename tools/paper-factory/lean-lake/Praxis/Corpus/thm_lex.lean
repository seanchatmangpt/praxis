import Praxis.Corpus.def_costvector
import Mathlib.Data.List.Lex
import Mathlib.Analysis.SpecialFunctions.Pow.Real

/-!
Label: thm:lex

"With $C_\lambda(\bm c)=\sum_{i=0}^k\lambda^{k-i}c_i$, $\lambda>1$, $|c_i|\le M$:
there is $\Lambda$ with $\bm c\preceq_{\mathrm{lex}}\bm c'\iff C_\lambda(\bm c)\le
C_\lambda(\bm c')$ for all $\lambda>\Lambda$. As $\lambda\to\infty$ the
lawfulness coordinate's weight diverges."

We formalize the secondary-cost tail (`cs : List Nat` in `def:costvector`) as
`List ℤ`, bounded coordinate-wise by `M`, realize `C_λ` via the Horner-style
fold `CVal` (the leading coordinate gets the highest power of `λ`, matching
`\sum \lambda^{k-i} c_i`), and prove the threshold `Λ = 2M+1` works: for
`λ > Λ`, lexicographic order on equal-length, `M`-bounded lists coincides
exactly with `<`/`≤` on `CVal λ`. This is the standard "sufficiently large
base recovers lexicographic order from a weighted sum" fact.

No axioms: the halved-magnitude bound and the forward (`Lex → CVal <`)
direction are both proved by structural induction on the paired lists, using
Mathlib's `List.Lex` API (`cons_lex_cons_iff`, `not_lex_nil`) and ordered-field
arithmetic on `ℝ` (`nlinarith`). Trichotomy of `List.Lex (· < ·)` (needed for
the reverse direction) is proved directly by induction from `lt_trichotomy`
on `ℤ`, rather than fighting Mathlib's separate `LinearOrder (List α)`
instance diamond.
-/

namespace ThmLex

/-- Horner-style weighted sum: `CVal λ [c0,c1,…,ck] = c0*λ^k + c1*λ^(k-1) + … + ck`,
i.e. exactly `\sum_{i=0}^k \lambda^{k-i} c_i` with the list indexed left-to-right. -/
def CVal (lam : ℝ) : List ℤ → ℝ
  | [] => 0
  | x :: xs => (x : ℝ) * lam ^ xs.length + CVal lam xs

/-- If every coordinate of `l` has absolute value `≤ M` and `lam > 2*M+1`, then
`2 * CVal lam l` lies strictly inside `(-lam^l.length, lam^l.length)`. Twice
the weighted sum is what a leading-coordinate difference of at least `1`
(times `lam^l.length`) must beat, so this is the exact bound the main
induction needs. -/
theorem cval_bounds (M : ℕ) (lam : ℝ) (hlam : (2 * (M : ℝ) + 1) < lam) :
    ∀ l : List ℤ, (∀ x ∈ l, |x| ≤ (M : ℤ)) →
      -(lam ^ l.length) < 2 * CVal lam l ∧ 2 * CVal lam l < lam ^ l.length := by
  have hM0 : (0:ℝ) ≤ (M:ℝ) := Nat.cast_nonneg M
  have hlam1 : (1:ℝ) < lam := lt_of_le_of_lt (by linarith) hlam
  have hlam0 : (0:ℝ) < lam := lt_trans one_pos hlam1
  intro l
  induction l with
  | nil => intro _; simp [CVal]
  | cons x xs ih =>
    intro hb
    have hx : |x| ≤ (M:ℤ) := hb x (List.mem_cons_self ..)
    have hxs : ∀ y ∈ xs, |y| ≤ (M:ℤ) := fun y hy => hb y (List.mem_cons_of_mem _ hy)
    obtain ⟨ihlo, ihhi⟩ := ih hxs
    have hxR : -(M:ℝ) ≤ (x:ℝ) ∧ (x:ℝ) ≤ (M:ℝ) := by
      have h := abs_le.mp hx
      constructor
      · have : (-(M:ℤ):ℝ) ≤ (x:ℝ) := by exact_mod_cast h.1
        simpa using this
      · exact_mod_cast h.2
    have hpowpos : (0:ℝ) < lam ^ xs.length := pow_pos hlam0 _
    have hCVal : CVal lam (x :: xs) = (x:ℝ) * lam ^ xs.length + CVal lam xs := rfl
    have hpowsucc : lam ^ (xs.length + 1) = lam * lam ^ xs.length := by ring
    rw [List.length_cons, hCVal, hpowsucc]
    constructor
    · nlinarith [hxR.1, hxR.2, ihlo, ihhi, hpowpos]
    · nlinarith [hxR.1, hxR.2, ihlo, ihhi, hpowpos]

/-- Forward direction of `thm:lex`: for `λ > 2M+1`, if `c` lexicographically
precedes `c'` (equal length, `M`-bounded coordinates), then `CVal λ c < CVal λ c'`. -/
theorem lex_lt_of_lex (M : ℕ) (lam : ℝ) (hlam : (2 * (M : ℝ) + 1) < lam) :
    ∀ c c' : List ℤ, c.length = c'.length →
      (∀ x ∈ c, |x| ≤ (M : ℤ)) → (∀ x ∈ c', |x| ≤ (M : ℤ)) →
      List.Lex (· < ·) c c' → CVal lam c < CVal lam c' := by
  have hM0 : (0:ℝ) ≤ (M:ℝ) := Nat.cast_nonneg M
  have hlam1 : (1:ℝ) < lam := lt_of_le_of_lt (by linarith) hlam
  have hlam0 : (0:ℝ) < lam := lt_trans one_pos hlam1
  intro c
  induction c with
  | nil =>
    intro c' hlen _ _ hlex
    cases c' with
    | nil => exact absurd hlex List.not_lex_nil
    | cons y ys => simp at hlen
  | cons x xs ih =>
    intro c' hlen hx hx' hlex
    cases c' with
    | nil => simp at hlen
    | cons y ys =>
      have hlen' : xs.length = ys.length := Nat.succ_injective hlen
      have hxsb : ∀ z ∈ xs, |z| ≤ (M:ℤ) := fun z hz => hx z (List.mem_cons_of_mem _ hz)
      have hysb : ∀ z ∈ ys, |z| ≤ (M:ℤ) := fun z hz => hx' z (List.mem_cons_of_mem _ hz)
      rw [List.cons_lex_cons_iff] at hlex
      have eqx : CVal lam (x :: xs) = (x:ℝ) * lam ^ xs.length + CVal lam xs := rfl
      have eqy : CVal lam (y :: ys) = (y:ℝ) * lam ^ ys.length + CVal lam ys := rfl
      show CVal lam (x :: xs) < CVal lam (y :: ys)
      rw [eqx, eqy, hlen']
      have hpowpos : (0:ℝ) < lam ^ ys.length := pow_pos hlam0 _
      rcases hlex with hxy | ⟨rfl, hlexTail⟩
      · have hxyR : (x:ℝ) + 1 ≤ (y:ℝ) := by exact_mod_cast hxy
        obtain ⟨hxsLo, hxsHi⟩ := cval_bounds M lam hlam xs hxsb
        obtain ⟨hysLo, hysHi⟩ := cval_bounds M lam hlam ys hysb
        rw [hlen'] at hxsLo hxsHi
        nlinarith
      · have := ih ys hlen' hxsb hysb hlexTail
        nlinarith

/-- Trichotomy of `List.Lex (· < ·)` on `List ℤ`, proved directly by induction
from `lt_trichotomy` on `ℤ` (avoiding Mathlib's separate `LinearOrder (List α)`
instance, whose `<` is not definitionally `List.Lex` without extra
bookkeeping). -/
theorem lex_trichotomy : ∀ c c' : List ℤ,
    List.Lex (· < ·) c c' ∨ c = c' ∨ List.Lex (· < ·) c' c := by
  intro c
  induction c with
  | nil =>
    intro c'
    cases c' with
    | nil => exact Or.inr (Or.inl rfl)
    | cons y ys => exact Or.inl List.Lex.nil
  | cons x xs ih =>
    intro c'
    cases c' with
    | nil => exact Or.inr (Or.inr List.Lex.nil)
    | cons y ys =>
      rcases lt_trichotomy x y with hxy | hxy | hxy
      · exact Or.inl (List.cons_lex_cons_iff.mpr (Or.inl hxy))
      · subst hxy
        rcases ih ys with h1 | h1 | h1
        · exact Or.inl (List.cons_lex_cons_iff.mpr (Or.inr ⟨rfl, h1⟩))
        · exact Or.inr (Or.inl (congrArg (x :: ·) h1))
        · exact Or.inr (Or.inr (List.cons_lex_cons_iff.mpr (Or.inr ⟨rfl, h1⟩)))
      · exact Or.inr (Or.inr (List.cons_lex_cons_iff.mpr (Or.inl hxy)))

/-- `thm:lex`: for `M`-bounded, equal-length integer cost vectors and any
`λ > 2M+1`, lexicographic order coincides exactly with the order induced by
the weighted sum `CVal λ`. -/
theorem thm_lex (M : ℕ) (lam : ℝ) (hlam : (2 * (M : ℝ) + 1) < lam) :
    ∀ c c' : List ℤ, c.length = c'.length →
      (∀ x ∈ c, |x| ≤ (M : ℤ)) → (∀ x ∈ c', |x| ≤ (M : ℤ)) →
      (List.Lex (· < ·) c c' ↔ CVal lam c < CVal lam c') := by
  intro c c' hlen hc hc'
  constructor
  · exact lex_lt_of_lex M lam hlam c c' hlen hc hc'
  · intro hCV
    rcases lex_trichotomy c c' with hlex | hlex | hlex
    · exact hlex
    · exact absurd hCV (by rw [hlex]; exact lt_irrefl _)
    · exact absurd (lex_lt_of_lex M lam hlam c' c hlen.symm hc' hc hlex)
        (not_lt_of_gt hCV)

end ThmLex
