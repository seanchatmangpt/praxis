import Mathlib.Order.WellFounded

/-!
# PROJ-769 / PRD v26.7.11 §6.6 — Bounded Descent

Target 2 of the 9 declared Lean/Lake formalization targets at `PRD.md:1035-1043`:
"bounded descent under lexicographic cost vectors."

PRD §6.6 (`docs/jira/v26.7.11/PRD.md:226-249`), verbatim:

> Each recursive substitution SHALL consume budget or decrease a declared
> well-founded measure. The initial cost vector SHALL support:
> `C(W) = ⟨d,a,u,r⟩`
> where d: remaining decomposition depth; a: unresolved activities;
> u: unresolved uncertainty; r: unresolved resource dependencies.
> Required relation: `C(W_child) <_lex C(W_parent)`.
> A child that cannot establish descent SHALL be refused.

## Formalization strategy

`C(W)` is realized as `CostVec4`, a 4-field `Nat` record matching the PRD's own
`⟨d,a,u,r⟩` order exactly. This is a genuinely different (and simpler) model than
`Praxis/Corpus/def_costvector.lean`'s `Bool × List Nat` (that earlier paper's cost
vector carries a variable-length secondary-cost tail plus an admitted/unadmitted
indicator bit; PRD §6.6's is a fixed 4-tuple of plain natural counts with no
indicator bit), so this file does not import or extend it.

The PRD's "well-founded measure" claim is the actual content-bearing part: this
file proves `CostVec4.lt` — realized as the pullback along `toProd` of Lean core's
own nested `Prod.Lex` relation (`Init/WF.lean`'s `Prod.lex` combinator, the exact
mechanism Lean's `termination_by` elaborator uses to justify lexicographic
termination measures) — is a well-founded relation, unconditionally, with no
boundedness hypothesis needed (`Nat`'s own `<` is well-founded, and `Prod.lex`
preserves well-foundedness through nested products). This is a different, simpler
technique than `Praxis/Corpus/thm_lex.lean`'s (which recovers lex order from a
real-valued weighted sum over `M`-bounded lists, aimed at order-*equivalence*); here
the goal is well-foundedness directly, over plain `Nat` coordinates, matching PRD
§6.6's coordinates being plain natural counts (not real-valued).

Well-foundedness is exactly the reason "a child that cannot establish descent SHALL
be refused" is a sound termination guard: requiring every recursive substitution
step to strictly lex-decrease `C(W)` under this relation forbids an infinite
refusal-free descent, proved below as `no_infinite_descent`.

No axioms: `CostVec4`, `toProd`, and `lt` are plain data/`InvImage`; well-foundedness
is derived from Lean core's `Nat.lt_wfRel.wf` through `Prod.lex`/`InvImage.wf`, not
assumed.
-/

/-- `C(W) = ⟨d,a,u,r⟩` (PRD §6.6): remaining decomposition depth, unresolved
activities, unresolved uncertainty, unresolved resource dependencies, in the PRD's
own declared order. -/
structure CostVec4 where
  d : Nat
  a : Nat
  u : Nat
  r : Nat
deriving DecidableEq, Repr

namespace CostVec4

/-- `C(W)` as the nested pair Lean core's `Prod.Lex`/`Prod.lex` combinator expects,
preserving PRD §6.6's declared coordinate order `d, a, u, r`. -/
def toProd (x : CostVec4) : Nat × Nat × Nat × Nat := (x.d, x.a, x.u, x.r)

/-- The well-founded lexicographic relation on nested `Nat` quadruples, built from
Lean core's `Prod.lex` combinator (`Init/WF.lean`) applied three times over
`Nat.lt_wfRel`. -/
@[reducible] def nestedWfRel : WellFoundedRelation (Nat × Nat × Nat × Nat) :=
  Prod.lex Nat.lt_wfRel (Prod.lex Nat.lt_wfRel (Prod.lex Nat.lt_wfRel Nat.lt_wfRel))

/-- Strict lexicographic order on `CostVec4` (PRD §6.6's `<_lex`): compare `d`, then
`a`, then `u`, then `r`, realized as the pullback along `toProd` of `nestedWfRel`'s
underlying relation rather than a hand-rolled disjunction. -/
def lt (x y : CostVec4) : Prop :=
  InvImage nestedWfRel.rel toProd x y

instance : LT CostVec4 := ⟨lt⟩

/-- `thm:bounded_descent`'s well-foundedness core: `CostVec4.lt` is well-founded —
unconditionally, with no boundedness hypothesis needed, because `Nat`'s own `<` is
well-founded and Lean core's `Prod.lex` combinator preserves well-foundedness
through nested products. There is no infinite strictly `lt`-descending chain of
cost vectors, full stop. -/
theorem wellFounded_lt : WellFounded lt :=
  InvImage.wf toProd nestedWfRel.wf

/-- No infinite strictly-descending chain of cost vectors exists: if a sequence
`f : ℕ → CostVec4` strictly `lt`-descends at every step, that is a contradiction.
Formalizes "each recursive substitution SHALL consume budget or decrease a
declared well-founded measure" together with "a child that cannot establish
descent SHALL be refused": requiring descent at every admitted step is exactly
what a well-founded measure needs to guarantee termination, and refusal is what
enforces the requirement. -/
theorem no_infinite_descent (f : Nat → CostVec4) (hf : ∀ n, lt (f (n + 1)) (f n)) :
    False := by
  have hex := wellFounded_lt.has_min (Set.range f) ⟨f 0, ⟨0, rfl⟩⟩
  obtain ⟨m, ⟨n, hn⟩, hmin⟩ := hex
  exact hmin (f (n + 1)) ⟨n + 1, rfl⟩ (hn ▸ hf n)

end CostVec4
