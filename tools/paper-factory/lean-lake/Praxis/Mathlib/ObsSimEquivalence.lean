/-!
Shared `Obs`/`Sim` layer (used by `thm:rice`, `def:adm`, `def:mu`),
reformalized in the Mathlib lane -- specifically the `Sim` equivalence
axioms, not the full Rice's-theorem reduction (which stays as future
work; see Chapter 7 of the thesis for why a bounded pilot, not a full
migration, is what this pass claims).

Bare-core version (`tools/paper-factory/lean-pilot/thm_rice.lean` and
`def_adm.lean`/`def_mu.lean`, which reuse it verbatim) states three
separate hand-written axioms:

    axiom Sim_refl  : ∀ o, Sim o o
    axiom Sim_symm  : ∀ {o1 o2}, Sim o1 o2 → Sim o2 o1
    axiom Sim_trans : ∀ {o1 o2 o3}, Sim o1 o2 → Sim o2 o3 → Sim o1 o3

Here, those three properties are packaged into ONE axiom using Lean's
pre-built `Equivalence` structure (`Init.Core`, part of Lean 4's own
core library, no Mathlib import even required for this specific
structure -- it predates and is more fundamental than Mathlib itself):

    axiom Sim_equiv : Equivalence Sim

`Sim_refl`/`Sim_symm`/`Sim_trans` below are then not axioms at all, but
theorems trivially projected out of `Sim_equiv` -- the reflexivity,
symmetry, and transitivity obligations are certified once, together, as
a single structure instance, rather than declared as three independent,
individually-unchecked axioms whose mutual consistency as "actually an
equivalence relation" is never itself verified by the kernel in the
bare-core version (nothing stops three unrelated axioms named
`Sim_refl`/`Sim_symm`/`Sim_trans` from being individually true but
jointly nonsensical; bundling them into one `Equivalence` value is
exactly as strong a set of hypotheses, but it is one auditable unit
instead of three, and it is the same `Equivalence` structure every other
equivalence relation in Mathlib is built from -- reusing the pre-built
vocabulary instead of re-deriving the three-axiom pattern from scratch.
-/

axiom Obs : Type
axiom Sim : Obs → Obs → Prop

axiom Sim_equiv : Equivalence Sim

theorem Sim_refl (o : Obs) : Sim o o := Sim_equiv.refl o
theorem Sim_symm {o1 o2 : Obs} (h : Sim o1 o2) : Sim o2 o1 := Sim_equiv.symm h
theorem Sim_trans {o1 o2 o3 : Obs} (h1 : Sim o1 o2) (h2 : Sim o2 o3) : Sim o1 o3 :=
  Sim_equiv.trans h1 h2

-- `Sim` is also directly usable as a `Setoid` -- Mathlib's standard
-- interface for "a type with a distinguished equivalence relation" --
-- for free, since a `Setoid` is exactly a carrier type plus an
-- `Equivalence` proof.
instance : Setoid Obs where
  r := Sim
  iseqv := Sim_equiv
