/-
cor:dominate — An admitted plan sorts strictly before any refused plan
regardless of its secondary costs: for any finite risk, makespan, token,
latency, and switch values, an admitted CostVector is ⪯_lex (in fact
strictly `<`) the refused top element; the price of unlawfulness is
infinite in the limit that defines the order.

Instantiated at thm:lex's k=1 setting: `c0` is the lawfulness coordinate
(`0` = admitted, `1` = refused), `c1`/`b1` the bounded secondary cost.
This is a corollary of `lex`: whenever the lawfulness coordinates differ
(`0 < 1`), the `lex` relation holds unconditionally, independent of the
secondary coordinates `a1`, `b1`.
-/

/-- The lexicographic order `⪯_lex` on `(c0,c1)` pairs, as in thm:lex. -/
def lex (a0 a1 b0 b1 : Nat) : Prop := a0 < b0 ∨ (a0 = b0 ∧ a1 ≤ b1)

/-- **cor:dominate**: an admitted plan (lawfulness coordinate `0`) sorts
strictly before any refused plan (lawfulness coordinate `1`) under
`⪯_lex`, for arbitrary finite secondary costs `a1` (admitted plan's) and
`b1` (refused plan's) — the secondary coordinates play no role once the
lawfulness coordinates differ. -/
theorem cor_dominate (a1 b1 : Nat) : lex 0 a1 1 b1 := by
  unfold lex
  left
  omega
