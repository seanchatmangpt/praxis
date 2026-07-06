import Praxis.Corpus.def_ob

/-!
# def:pipeline

Let `Stage` be the set of admission stages; fix `o` and let `φ_o : Stage → Deny` send a
stage to the denial word it emits on `o`; a pipeline is a finite sequence `w = s₁⋯sₖ ∈
Stage*`, an element of the free monoid on `Stage`; its aggregate denial is
`Φ_o(w) = ⋁_{i=1}^{k} φ_o(sᵢ)`, `Φ_o(ε) = Adml`.

`Stage` is left abstract (the thesis does not commit to a concrete encoding, matching the
treatment of `Obs` in `def:ob`), so it is a type parameter. The free monoid on `Stage` is
already provided by Mathlib/core as `List Stage` (with `[]` the empty word and `++` the
concatenation) -- there is no need to hand-roll a free-monoid construction. `Deny` is the
already-migrated `DenialPolarity` from `def:denialcode`.
-/

namespace Pipeline

variable {Stage : Type}

/-- A pipeline is a finite sequence of stages: an element of the free monoid `List Stage`
(`[]` is `ε`, `++` is concatenation). -/
abbrev Seq (Stage : Type) := List Stage

variable (φ : Stage → DenialPolarity)

/-- The aggregate denial `Φ_o(w) = ⋁_{i=1}^{k} φ_o(sᵢ)`, realized as the fold of
`compose` (bitwise OR) over the sequence, starting from the clean word `Adml`; on the
empty pipeline `Φ_o(ε) = Adml`. The per-stage denial map `φ_o : Stage → Deny`, for a
fixed observation `o`, is any function from stages to denial words -- it plays the same
role as `δ_g` in `def:ob`, so we take it as a hypothesis rather than re-deriving it. -/
def aggregateDenial (w : Seq Stage) : DenialPolarity :=
  w.foldr (fun s acc => DenialPolarity.compose (φ s) acc) DenialPolarity.Adml

end Pipeline
