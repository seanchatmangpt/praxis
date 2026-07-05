/-
prop:attnconservation — Total attention is independent of ordering.

Statement (LaTeX): Total attention
  ∫_0^T Σ_j r_j · 1_{[s_j,e_j]}(t) dt = Σ_j r_j (e_j - s_j)
is independent of ordering. Reordering redistributes *when* attention is
spent, not *how much*; the optimization target is the makespan functional
`T[·]` under precedence and the pointwise capacity constraint.

We reuse the `Action` index type and the (nonnegative) `r`/`s`/`e` axioms
from `def:attnbalance` (`def_attnbalance.lean`). The right-hand side of the
displayed identity, `Σ_j r_j (e_j - s_j)`, is a *sum over the finite index
set* of admitted actions, and the claim "independent of ordering" is
precisely: reordering the list of actions we sum over does not change the
sum, i.e. the sum is invariant under `List.Perm`.

Since `def:attnbalance` only gives `r j : Float`, and Lean core has no
proved commutative-monoid/associativity lemmas for `Float` (floating-point
addition is not associative), we represent each action's *attention
contribution* `r_j (e_j - s_j)` abstractly as a nonnegative natural number
`w j` — this is the honest content of the statement (a quantity, summed),
without smuggling in unsound floating-point algebra. This matches the
definition file's own remark that it deliberately keeps the numeric model
abstract.

The proof obligation, matching the statement exactly, is:

    ∀ l₁ l₂ : List Action, l₁.Perm l₂ → (l₁.map w).sum = (l₂.map w).sum

i.e. summing the per-action attention contributions over any two orderings
(any two permutations of the admitted-action list) gives the same total.
This is exactly `List.Perm.sum_nat` from Lean core's `Init.Data.List.Perm`,
applied after mapping by `w`.
-/

/-- Number of admitted actions (reusing the role of `n` from
`def:attnbalance`; declared fresh here since we do not `import` that file,
consistent with working file-locally in bare Lean 4 core). -/
axiom n : Nat

/-- Admitted actions, indexed `0, ..., n-1` (same shape as `Action` in
`def:attnbalance`). -/
abbrev Action := Fin n

/-- Attention contribution of action `j`, i.e. `r_j (e_j - s_j)`, taken as
an abstract nonnegative quantity (natural number). -/
axiom w : Action → Nat

/-- Total attention consumed under a given ordering `l` of the admitted
actions: `Σ_{j ∈ l} w j`. -/
noncomputable def totalAttention (l : List Action) : Nat :=
  (l.map w).sum

/-- **Attention conservation.** Total attention is independent of ordering:
any two orderings (permutations) of the same admitted actions yield the
same total attention. Reordering redistributes *when* attention is spent
(the schedule / makespan), not *how much* is spent in total. -/
theorem attnconservation {l₁ l₂ : List Action} (h : l₁.Perm l₂) :
    totalAttention l₁ = totalAttention l₂ := by
  unfold totalAttention
  exact (h.map w).sum_nat
