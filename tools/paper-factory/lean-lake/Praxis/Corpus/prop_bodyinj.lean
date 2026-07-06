import Mathlib.Tactic
import Praxis.Corpus.def_body

/-!
`prop:bodyinj`: `body` is injective on the semantic field tuple: if `body(fr) = body(fr')`
then `fr` and `fr'` agree on every field of `Frame` (`def:frame`).

`def:body` already axiomatizes `body_injective : Function.Injective body`, i.e.
`body fr = body fr' → fr = fr'`. Since `Frame` equality *is* agreement on every field of
`Frame` (there is no further hidden state), this proposition is exactly that axiom's
statement rephrased -- a direct corollary, not a new fact, so no additional axiom is
introduced here.
-/

/-- `body` is injective: equal hash bodies imply equal frames, i.e. the source frames
agree on every semantic field. Proved directly from the axiomatized `body_injective`
in `def:body`; no new axiom needed. -/
theorem body_injective_on_fields : Function.Injective body :=
  body_injective
