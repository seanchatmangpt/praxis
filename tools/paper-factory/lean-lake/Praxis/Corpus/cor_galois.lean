import Praxis.Corpus.thm_mono

/-!
# cor:galois

The assignment $G\mapsto\Adm_G$ is an order-reversing map from $(2^{\mathcal{G}},\subseteq)$
to $(2^{\Obs},\subseteq)$: $G\subseteq G'\Rightarrow\Adm_{G'}\subseteq\Adm_G$; the empty
obligation set admits everything ($\Adm_\varnothing=\Obs$), and enlarging obligations
monotonically tightens the gate.

This is the direct corollary of `thm:mono` (`Praxis/Corpus/thm_mono.lean`), read as a Galois-
connection statement about the map `G ↦ Adm_G := {o | is_admitted (totalDenial G o)}`. No new
axioms or algebra are needed: `Adm_{G'} ⊆ Adm_G` is exactly `thm_mono`'s second conjunct
(already proved there via `compose_eq_Adml_iff`-style unfolding of `is_admitted`), and
`Adm_∅ = Obs` is the base case `slnil` of `thm_mono`'s own induction (`totalDenial [] o =
Adml`, i.e. the empty obligation set's denial code is always the all-admit code, so every
observation is admitted). We state the order-reversing property in `Set`-inclusion form
(`Set.Sublist`-indexed lists still realize `G ⊆ G'` as `List.Sublist`, matching `thm:mono`'s
own reading of set inclusion for obligation *sets*) and the `Adm_∅ = Obs` fact as a `Set.eq_univ_of_forall`.
-/

open DenialPolarity Obligation Praxis.Corpus.ThmMono

namespace Praxis.Corpus.CorGalois

/-- `Adm_G := {o | is_admitted (totalDenial G o)}`, the admission set of obligation list `G`. -/
def Adm {Obs : Type} (G : List (Obligation Obs)) : Set Obs :=
  {o | DenialPolarity.is_admitted (totalDenial G o)}

/-- `Adm_∅ = Obs`: the empty obligation set admits everything, since `totalDenial [] o`
unfolds (by `List.foldr_nil`) to the all-admit code `Adml`, which is admitted by definition
(`is_admitted`, `def:denialcode`). Base case of `thm_mono`'s own induction (`slnil`), reused
here directly rather than re-derived. -/
theorem adm_empty_eq_univ {Obs : Type} : (Adm ([] : List (Obligation Obs))) = Set.univ := by
  apply Set.eq_univ_of_forall
  intro o
  show DenialPolarity.is_admitted (totalDenial [] o)
  simp [totalDenial, DenialPolarity.is_admitted, DenialPolarity.Adml]

/-- `cor:galois`: `G ↦ Adm_G` is order-reversing, `G ⊆ G' ⇒ Adm_{G'} ⊆ Adm_G` (realized, as
in `thm:mono`, via `List.Sublist`), and `Adm_∅ = Obs`. The order-reversing half is exactly
`thm_mono`'s second conjunct, packaged here as `Set` inclusion between the `Adm` sets. -/
theorem cor_galois {Obs : Type} {G G' : List (Obligation Obs)} (hsub : G.Sublist G') :
    Adm G' ⊆ Adm G ∧ (Adm ([] : List (Obligation Obs))) = Set.univ := by
  refine ⟨?_, adm_empty_eq_univ⟩
  intro o ho
  exact (thm_mono hsub).2 o ho

end Praxis.Corpus.CorGalois
