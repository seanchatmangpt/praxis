import Praxis.Corpus.thm_lex

/-!
Label: cor:dominate

"An admitted plan sorts strictly before any refused plan regardless of its
secondary costs: for any finite risk, makespan, token, latency, and switch
values, an admitted `CostVector` is `≺_lex` the refused top element; the
price of unlawfulness is infinite in the limit that defines the order."

We model the lawfulness coordinate as the leading entry of the `List ℤ`
encoding from `thm:lex` (admitted = some value `a`, refused = a strictly
larger value `r`, e.g. the "top element" reached only by refusal), and the
remaining risk/makespan/token/latency/switch coordinates as an arbitrary
equal-length tail `cs`/`cs'` (no boundedness needed for this direction: a
strict lead-coordinate gap already forces lexicographic order by
`List.cons_lex_cons_iff`, independent of the tail). This is exactly `thm:lex`
specialized/simplified to the case the paper cites as "the price of
unlawfulness is infinite": no finite tail cost can compensate for a strictly
worse lawfulness coordinate, so the admitted vector beats the refused one in
`≺_lex` for *every* choice of secondary costs, and (via `thm_lex`) for every
sufficiently large weight `λ` in the `C_λ` realization as well.

No axioms: both parts are direct consequences of `List.cons_lex_cons_iff`
(for the `≺_lex` half) and `ThmLex.thm_lex` (for the `C_λ` half), matching
`ThmRiceViaMathlib.lean`'s style of deriving a corollary from an
already-proved theorem rather than restating it as a fresh axiom.
-/

namespace CorDominate

open ThmLex

/-- `cor:dominate`, `≺_lex` half: whatever the (equal-length) secondary-cost
tails `cs`, `cs'` are, an admitted lawfulness coordinate `a` strictly below a
refused one `r` already forces the admitted cost vector `a :: cs` to
lexicographically precede the refused one `r :: cs'`. The secondary costs
(risk, makespan, tokens, latency, switches) play no role: unlawfulness
dominates absolutely. -/
theorem admitted_lex_lt_refused (a r : ℤ) (cs cs' : List ℤ) (h : a < r) :
    List.Lex (· < ·) (a :: cs) (r :: cs') :=
  List.cons_lex_cons_iff.mpr (Or.inl h)

/-- `cor:dominate`, weighted-sum half: for any `M`-bounded, equal-length
secondary-cost tails and any `λ` past `thm:lex`'s threshold `2M+1`, the same
strict lawfulness gap forces the admitted plan's weighted cost `C_λ` to be
strictly less than the refused plan's -- the price of unlawfulness is
infinite in the limit defining the order, since it wins for every
sufficiently large `λ` regardless of the (bounded) secondary costs. -/
theorem admitted_cval_lt_refused (M : ℕ) (lam : ℝ) (hlam : (2 * (M : ℝ) + 1) < lam)
    (a r : ℤ) (cs cs' : List ℤ) (hlen : cs.length = cs'.length) (h : a < r)
    (hcs : ∀ x ∈ cs, |x| ≤ (M : ℤ)) (hcs' : ∀ x ∈ cs', |x| ≤ (M : ℤ))
    (ha : |a| ≤ (M : ℤ)) (hr : |r| ≤ (M : ℤ)) :
    CVal lam (a :: cs) < CVal lam (r :: cs') := by
  refine thm_lex M lam hlam (a :: cs) (r :: cs') (by simp [hlen])
    ?_ ?_ |>.mp (admitted_lex_lt_refused a r cs cs' h)
  · intro x hx
    rcases List.mem_cons.mp hx with rfl | hx
    · exact ha
    · exact hcs x hx
  · intro x hx
    rcases List.mem_cons.mp hx with rfl | hx
    · exact hr
    · exact hcs' x hx

end CorDominate
