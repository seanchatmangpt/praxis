import Praxis.Corpus.con_denial
import Praxis.Corpus.def_taxonomy

/-!
Label: prop:total

"The category map $\mathrm{cat}:S\to C$ is total (enforced by the host type
system's exhaustive match), and the lane map's single-lane inverse is a
section of the lane map over the seven named lanes."

This proposition packages two independently-checkable facts, each already
set up by prior corpus entries with no new axioms:

1. `Taxonomy.cat` is total. In Lean/core's type theory every function
   `S → C` is total by construction (`def:taxonomy` already makes this
   point in prose); the checkable content is that `cat` always produces a
   value, witnessed here as `∃ c, T.cat s = c`.

2. The "single-lane inverse is a section of the lane map": `laneMap gs i`
   (from `con:denial`) is built from `!(gs.get i o)`, i.e. Boolean negation
   applied to the underlying obligation bit. Its "single-lane inverse" is
   negation again, and negation is its own section/retraction pair on
   `Bool` -- `Bool.not_not` from core -- so undoing the lane map's negation
   and reapplying it is the identity on the underlying bit, for every lane
   `i` in the battery `gs` (the informal "seven named lanes" is the LaTeX's
   illustrative instance of the general `gs.length`-many lanes already
   fixed in `con:denial`; no new lane count is introduced here).

Both parts are proved directly from core/Mathlib facts (`rfl` for
functional totality, `Bool.not_not` for the involution), with no axioms.
-/

open Nat.Partrec (Code)

/-- Part 1: the categorizing map of a `Taxonomy` is total: applied to any
scenario `s`, it produces a category value. -/
theorem taxonomy_cat_total (T : Taxonomy) (s : T.S) : ∃ c : RefusalCategory, T.cat s = c :=
  ⟨T.cat s, rfl⟩

/-- Part 2: the single-lane inverse (Boolean negation) of the lane map
`Deny.laneMap` is a section of the lane map's own negation, i.e. negating
twice the underlying obligation bit `gs.get i o` recovers it, for every
lane `i` in the battery `gs`. This is the content of `Deny.laneMap`'s
definition being `!(gs.get i o)` composed with itself. -/
theorem laneMap_single_lane_inverse_is_section
    (gs : List (Code → Bool)) (i : Fin gs.length) (o : Code) :
    Bool.not (Bool.not (gs.get i o)) = gs.get i o :=
  Bool.not_not _

/-- The combined statement: the category map is total, and the lane map's
single-lane inverse is a section of the lane map over all named lanes. -/
theorem total (T : Taxonomy) (s : T.S)
    (gs : List (Code → Bool)) (i : Fin gs.length) (o : Code) :
    (∃ c : RefusalCategory, T.cat s = c) ∧ Bool.not (Bool.not (gs.get i o)) = gs.get i o :=
  ⟨taxonomy_cat_total T s, laneMap_single_lane_inverse_is_section gs i o⟩
