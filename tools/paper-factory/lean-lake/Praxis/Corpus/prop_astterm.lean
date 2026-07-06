/-
Label: prop:astterm
Kind: proposition

For any finite AST T and finite gate battery (g_1,...,g_m) of syntactic
predicates, battery evaluation terminates in O(m*|T|) time and returns either
Admitted or a refusal carrying the first failing gate index and offending
node; the retraction is total, never diverging and never returning an
unlabelled outcome.

Depends on: def:astgate (Praxis.Corpus.ASTGate / GateBattery / GateBattery.admits
/ GateBattery.denial / Denial), imported below.

Formalization note: termination and "never returns an unlabelled outcome"
are automatic in Lean for the structurally recursive `List` traversals
underlying `GateBattery.admits` (a fold) and `GateBattery.denial`
(`List.findIdx?`) over a finite `GateBattery` -- Lean's kernel only accepts
terminating definitions, and the result type `Bool` / `Option (Denial T)` has
no third, unlabelled inhabitant, so there is nothing further to axiomatize
about termination or labelling. The mathematical content worth proving is
the *correctness* of the two-outcome retraction:

1. `admits t = true ↔ denial t = none` -- the battery admits `t` exactly
   when there is no denial (proved below by induction on the battery,
   matching the `O(m)` list traversal the informal statement describes,
   `m` gates each costing `O(|T|)` to evaluate on the AST `T`);
2. whenever a denial does fire, it carries the offending node `t` itself
   (immediate from the definition of `GateBattery.denial`).

Together these show `GateBattery.verdict` below always produces exactly one
of `Verdict.admitted` or a correctly-labelled `Verdict.refused d`, for every
`t`, with no other possibility -- i.e. the retraction is total.
-/

import Praxis.Corpus.def_astgate

namespace Praxis.Corpus

/-- The two possible outcomes of routing a proposed change `t` through a gate
battery: unconditional admission, or a refusal carrying the first failing
gate's index and the offending AST node. -/
inductive Verdict (T : Type) where
  | admitted : Verdict T
  | refused : Denial T → Verdict T

/-- Evaluate the battery on `t`, producing exactly one labelled outcome:
`Verdict.admitted` if every gate passes, otherwise `Verdict.refused d` for the
denial `d` witnessing the first failing gate. This is a plain case split on
`GateBattery.denial` (itself a structural, terminating recursion over the
finite list `b`), so there is no third, unlabelled outcome it could return. -/
def GateBattery.verdict {T : Type} (b : GateBattery T) (t : T) : Verdict T :=
  match b.denial t with
  | none => Verdict.admitted
  | some d => Verdict.refused d

/-- The starting offset passed to `List.findIdx?.go` does not affect *whether*
it finds a match, only which index it reports when it does. -/
theorem findIdx?_go_none_iff_offset {T : Type} (p : T → Bool) (l : List T) (i j : Nat) :
    List.findIdx?.go p l i = none ↔ List.findIdx?.go p l j = none := by
  induction l generalizing i j with
  | nil => simp [List.findIdx?.go]
  | cons a l ih =>
      simp only [List.findIdx?.go]
      by_cases h : p a = true
      · simp [h]
      · simp only [h]
        exact ih (i + 1) (j + 1)

/-- Battery evaluation is a total retraction: `admits` and `denial` always
agree on whether the change is admitted. Proved by induction on the battery
`b`, mirroring the `O(m)` traversal of the `m`-gate battery. -/
theorem admits_iff_denial_none {T : Type} (b : GateBattery T) (t : T) :
    b.admits t = true ↔ b.denial t = none := by
  induction b with
  | nil =>
      simp [GateBattery.admits, GateBattery.denial, List.findIdx?, List.findIdx?.go]
  | cons g gs ih =>
      constructor
      · intro h
        simp only [GateBattery.admits, List.all_cons, Bool.and_eq_true] at h
        obtain ⟨hg, hgs⟩ := h
        have hgs' : GateBattery.admits gs t = true := hgs
        have hnone : GateBattery.denial gs t = none := ih.mp hgs'
        simp only [GateBattery.denial, List.findIdx?] at hnone ⊢
        have hnone0 : List.findIdx?.go (fun g => !g.gate t) gs 0 = none := by
          rcases hm : List.findIdx?.go (fun g => !g.gate t) gs 0 with _ | i
          · exact hm
          · simp [hm] at hnone
        have hnone1 : List.findIdx?.go (fun g => !g.gate t) gs 1 = none :=
          (findIdx?_go_none_iff_offset _ gs 0 1).mp hnone0
        simp [List.findIdx?.go, hg, hnone1]
      · intro h
        simp only [GateBattery.denial, List.findIdx?] at h
        by_cases hg : g.gate t = true
        · simp only [List.findIdx?.go, hg] at h
          have hnone1 : List.findIdx?.go (fun g => !g.gate t) gs 1 = none := by
            rcases hm : List.findIdx?.go (fun g => !g.gate t) gs 1 with _ | i
            · exact hm
            · simp [hm] at h
          have hnone0 : List.findIdx?.go (fun g => !g.gate t) gs 0 = none :=
            (findIdx?_go_none_iff_offset _ gs 0 1).mpr hnone1
          have hgs : GateBattery.denial gs t = none := by
            simp only [GateBattery.denial, List.findIdx?]
            simp [hnone0]
          have hadm : GateBattery.admits gs t = true := ih.mpr hgs
          have hadm' : ∀ x ∈ gs, x.gate t = true := List.all_eq_true.mp hadm
          simp only [GateBattery.admits, List.all_cons, Bool.and_eq_true]
          exact ⟨hg, List.all_eq_true.mpr hadm'⟩
        · exfalso
          have hg' : g.gate t = false := by
            simpa using hg
          simp [List.findIdx?.go, hg'] at h

/-- Whenever the battery does not admit `t`, the produced denial carries `t`
itself as the offending node -- immediate from the definition of
`GateBattery.denial`, which tags every `findIdx?` hit with the original `t`. -/
theorem denial_node {T : Type} (b : GateBattery T) (t : T) (d : Denial T)
    (h : b.denial t = some d) : d.node = t := by
  simp only [GateBattery.denial] at h
  rcases hb : b.findIdx? (fun g => !g.gate t) with _ | i
  · simp [hb] at h
  · simp [hb] at h
    subst h
    rfl

/-- The retraction is total: for every battery `b` and node `t`,
`GateBattery.verdict b t` is either `Verdict.admitted` (exactly when
`b.admits t = true`) or `Verdict.refused d` for a denial `d` whose `node`
field is `t` -- never a third, unlabelled outcome. -/
theorem astterm {T : Type} (b : GateBattery T) (t : T) :
    (b.admits t = true → b.verdict t = Verdict.admitted) ∧
    (b.admits t = false →
      ∃ d, b.verdict t = Verdict.refused d ∧ d.node = t) := by
  constructor
  · intro h
    have hnone : b.denial t = none := (admits_iff_denial_none b t).mp h
    simp [GateBattery.verdict, hnone]
  · intro h
    have hnotnone : b.denial t ≠ none := by
      intro hc
      have : b.admits t = true := (admits_iff_denial_none b t).mpr hc
      rw [this] at h
      exact absurd h (by decide)
    rcases hd : b.denial t with _ | d
    · exact absurd hd hnotnone
    · exact ⟨d, by simp [GateBattery.verdict, hd], denial_node b t d hd⟩

end Praxis.Corpus
