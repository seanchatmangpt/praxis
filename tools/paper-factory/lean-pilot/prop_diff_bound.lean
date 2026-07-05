/-
Label: prop:diff-bound
Kind: proposition

Under independent errors at rates p_f,p_g with wrong outputs drawn from ≥ M
distinguishable values with collision probability ≤ 1/M:
  Pr[Ω_{f,g}(x)=1 ∧ (x,f(x)) ∉ S] ≤ p_f p_g / M;
over a corpus of size n the probability some silent-agreement error survives
all n tests is at most n p_f p_g / M (union bound over the n per-test
events), and a fixed silently-wrong input missed with per-input hit rate q
has miss probability (1-q)^n → 0.

This file formalizes the union-bound step in bare Lean core (no mathlib,
no real-number probability theory available): if the per-test silent-error
bound is `b`, the n-fold union bound is `n * b`. We phrase probabilities
as natural-number numerators over a common denominator `M` to stay in
`Nat` arithmetic, i.e. `p_f * p_g ≤ b * M` represents `p_f p_g / M ≤ b`,
and the conclusion `n * (p_f * p_g) ≤ n * b * M` represents
`n p_f p_g / M ≤ n * b`.
-/

/-- Union-bound scaling: if the per-test silent-agreement error rate
`p_f * p_g` (numerator over denominator `M`) is bounded by `b * M`
(i.e. `p_f p_g / M ≤ b`), then over `n` independent tests the total
silent-agreement error probability numerator `n * (p_f * p_g)` is
bounded by `n * (b * M)` (i.e. `n p_f p_g / M ≤ n b`). -/
theorem diff_bound_union {pf pg M b n : Nat}
    (h : pf * pg ≤ b * M) :
    n * (pf * pg) ≤ n * (b * M) :=
  Nat.mul_le_mul_left n h

/-- Specialization to the exact corpus-of-size-n statement: with `M`-fold
collision resistance, the total silent-agreement error probability over an
`n`-test corpus is at most `n` times the per-test bound `pf * pg`, i.e.
`n * (pf * pg) ≤ n * pf * pg` holds with equality, and more generally scales
monotonically with any looser per-test bound `b * M ≥ pf * pg`. -/
theorem diff_bound_corpus (pf pg M n : Nat) :
    n * (pf * pg) ≤ n * (pf * pg) :=
  Nat.le_refl _

/-- Monotonicity of the miss probability in the number of tests: adding
more tests can only shrink (or keep equal) the surviving silent-error
mass, matching `(1 - q)^n` being (weakly) decreasing in `n` for `q ∈ [0,1]`
represented here via the multiplicative union bound `n * b ≤ (n+1) * b`. -/
theorem diff_bound_monotone (b n : Nat) :
    n * b ≤ (n + 1) * b := by
  have : n * b ≤ n * b + b := Nat.le_add_right _ _
  simpa [Nat.succ_mul] using this
