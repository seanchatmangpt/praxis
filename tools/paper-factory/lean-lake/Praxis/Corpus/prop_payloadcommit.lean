import Mathlib.Tactic
import Praxis.Corpus.con_commit
import Praxis.Corpus.prop_bodyinj

/-!
`prop:payloadcommit`: under Construction~`con:commit`, `body(fr)` determines `dg(p)`, and
by Proposition~`prop:bodyinj` any change to `dg(p)` changes `body(fr)`: a frame commits to
the content of the artifact, not an opaque handle.

`con:commit` packs `dg(p) = chainH(p)` into `fr.obj_refs` (the `Fin 8 → PackedObjRef`
field). So "a change to `dg(p)`" is exactly a change to `fr.obj_refs`, and "changes
`body(fr)`" is exactly `body fr ≠ body fr'`. This is a direct corollary of the
already-proved `body_injective_on_fields` (`prop:bodyinj`, itself a corollary of the
`body_injective` axiom from `def:body`): contrapose it against the `Frame.obj_refs`
projection, no new axiom needed.
-/

/-- `prop:payloadcommit`: if two frames disagree on `obj_refs` (i.e. on the packed digest
`dg(p)` that `con:commit` writes there), their hash bodies disagree too -- a frame's body
commits to the artifact's content (via `obj_refs`), not to an opaque handle. -/
theorem payload_commit_injective (fr fr' : Frame) (h : fr.obj_refs ≠ fr'.obj_refs) :
    body fr ≠ body fr' :=
  fun heq => h (congrArg Frame.obj_refs (body_injective_on_fields heq))
