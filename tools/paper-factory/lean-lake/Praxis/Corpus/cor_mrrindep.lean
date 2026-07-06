import Praxis.Corpus.thm_mono
import Praxis.Corpus.thm_mrrsep

/-!
# cor:mrrindep

`thm:mrrsep` holds exactly when no obligation in `G_prop` couples distinct accounts; if
such coupling exists, the proposer's obligation battery must be extended with a
`BlockingConstraint` on the shared resource, which by `thm:mono` only tightens the
admitted set and preserves the denial algebra.

This corollary packages both halves of that claim from already-verified corpus lemmas,
with no new axioms:

1. The "no coupling" branch is exactly `MRR_separable` (`thm:mrrsep`): when the joint
   plan space for account set `Aset` factors as independent per-account target choices
   `A → T`, MRR equals the sum of per-account maxima.
2. The "coupling exists" branch is the direct specialisation of `thm_mono` to the
   one-obligation extension `G' = c :: G` (adding a single extra `BlockingConstraint`
   obligation `c` to the battery `G`): `G.Sublist (c :: G)` is the tautological
   `List.sublist_cons_self`, so `thm_mono` fires unconditionally and gives both the
   denial-algebra compose identity and the admission-tightening inclusion
   `Adm_{c::G} ⊆ Adm_G`, i.e. adding the blocking obligation can only refuse observations
   the smaller battery already admitted, never admit new ones -- "preserves the denial
   algebra" in exactly `thm:mono`'s sense.
-/

open DenialPolarity Obligation

namespace Praxis.Corpus.CorMrrindep

/-- The coupling-exists branch: extending an obligation battery `G` by one extra
`BlockingConstraint` obligation `c` (`G' := c :: G`) only tightens the admitted set and
preserves the denial-algebra compose identity, as a direct corollary of `thm:mono` --
`G.Sublist (c :: G)` holds unconditionally (`List.sublist_cons_self`), so no side
hypothesis on `c` (e.g. being an actual `BlockingConstraint` lane rather than some other
obligation) is needed for this half of the claim. -/
theorem blocking_extension_tightens {Obs : Type} (G : List (Obligation Obs))
    (c : Obligation Obs) :
    (∀ o : Obs,
      DenialPolarity.compose (totalDenial G o) (totalDenial (c :: G) o)
        = totalDenial (c :: G) o) ∧
    (∀ o : Obs, DenialPolarity.is_admitted (totalDenial (c :: G) o) →
      DenialPolarity.is_admitted (totalDenial G o)) :=
  Praxis.Corpus.ThmMono.thm_mono (List.sublist_cons_self c G)

/-- The no-coupling branch: when the joint plan space for `Aset` is the independent
product `A → T` (no obligation couples distinct accounts' target choices), MRR is the
sum of per-account maxima -- exactly `thm:mrrsep`, re-exported here under `cor:mrrindep`'s
name for direct use alongside `blocking_extension_tightens`. -/
theorem no_coupling_separable {A T : Type*} [Fintype A] [DecidableEq A] [Fintype T]
    [Nonempty T] (Aset : Finset A) (f : A → T → ℝ) :
    MRR Aset (fun (p : A → T) (a : A) => f a (p a))
      = Aset.sum (fun a => Finset.univ.sup' Finset.univ_nonempty (f a)) :=
  MRR_separable Aset f

/-- `cor:mrrindep`, both branches bundled: independence (no coupling obligation) gives
MRR separability (`thm:mrrsep`), while coupling -- modelled as extending the battery by
one `BlockingConstraint` obligation -- only tightens admission and preserves the
denial-algebra compose identity (`thm:mono`). -/
theorem cor_mrrindep
    {A T : Type*} [Fintype A] [DecidableEq A] [Fintype T] [Nonempty T]
    (Aset : Finset A) (f : A → T → ℝ)
    {Obs : Type} (G : List (Obligation Obs)) (c : Obligation Obs) :
    (MRR Aset (fun (p : A → T) (a : A) => f a (p a))
        = Aset.sum (fun a => Finset.univ.sup' Finset.univ_nonempty (f a))) ∧
    ((∀ o : Obs,
        DenialPolarity.compose (totalDenial G o) (totalDenial (c :: G) o)
          = totalDenial (c :: G) o) ∧
     (∀ o : Obs, DenialPolarity.is_admitted (totalDenial (c :: G) o) →
       DenialPolarity.is_admitted (totalDenial G o))) :=
  ⟨no_coupling_separable Aset f, blocking_extension_tightens G c⟩

end Praxis.Corpus.CorMrrindep
