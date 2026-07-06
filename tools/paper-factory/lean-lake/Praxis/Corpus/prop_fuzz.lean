import Praxis.Corpus.def_fuzz

/-!
# prop:fuzz

If the total admission retraction `adm : Obs → Option (Adm ⊕ Rfsl)` never
returns `none` (i.e. `adm` is total/terminating with codomain
`Adm ∪ {Rfsl}`, in the `Option`-encoding of `def:fuzz`) and every admitted
`x : Adm` is well-formed (i.e. `Adm`'s elements are well-formed by
construction — refusals are already unconditionally categorised via the
total `category` map, per `def:fuzz`), then the oracle verdict `Ω_∂(o) = 1`
for every `o ∈ Obs`.

Conversely (with no extra hypotheses), `Ω_∂(o) = 0` happens exactly when
`adm o = none`: an observed `0`-verdict is therefore always a witness that
the retraction crashed/diverged/returned an unlabelled outcome on that
input — a defect in `adm` itself, not a property that depends on which `o`
was supplied.
-/

namespace Praxis.Corpus

variable {Obs Adm Rfsl : Type*}

/-- Forward direction: if `adm` is total (never `none`) and every admitted
artifact is well-formed, the oracle verdict is `true` everywhere. -/
theorem FuzzOracle.Ω_eq_true_of_total (F : FuzzOracle Obs Adm Rfsl)
    (htotal : ∀ o, F.adm o ≠ none) (hwf : ∀ x, F.wellFormed x) :
    ∀ o, F.Ω o = true := by
  intro o
  rcases hadm : F.adm o with _ | y
  · exact absurd hadm (htotal o)
  · rcases y with x | r
    · simp [FuzzOracle.Ω, hadm, hwf x]
    · simp [FuzzOracle.Ω, hadm]

/-- A `false` verdict is exactly a witness that `adm` returned `none` on
that input (given that admitted artifacts are well-formed by construction):
any observed `Ω_∂(o) = 0` is a genuine defect in the retraction
(`adm o = none`), not a property of `o` itself. -/
theorem FuzzOracle.Ω_eq_false_iff (F : FuzzOracle Obs Adm Rfsl)
    (hwf : ∀ x, F.wellFormed x) (o : Obs) :
    F.Ω o = false ↔ F.adm o = none := by
  unfold FuzzOracle.Ω
  rcases F.adm o with _ | y
  · simp
  · rcases y with x | r <;> simp [hwf]

end Praxis.Corpus
