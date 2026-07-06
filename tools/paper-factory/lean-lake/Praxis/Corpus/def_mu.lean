import Praxis.Corpus.def_adm

/-!
Label: def:mu

"A manufacturing morphism is a deterministic computable map
$\muop:\Adm\to\Act$ satisfying determinism/reproducibility (M1) and
boundedness (M2); $\muop$ is undefined off $\Adm$ and $\muop(\Rfsl)=\Rfsl$."

Continuing the encoding of `def_adm`: `Adm` is realized as `Option Code`, the
codomain of `adm` (`some o` for admitted observations, `none` for `Rfsl`).
`Act` is likewise instantiated as `Option Code`: manufacturing actions are
observations-shaped data (`Code`), with the same `Rfsl` sentinel.

- Determinism/reproducibility (M1): `f : Code → Code` is a plain Lean
  function -- every Lean function is deterministic and total by construction,
  so M1 is automatic and needs no separate hypothesis.
- Boundedness (M2): realized as computability of `f`, i.e. `f` arises from a
  `Code` value (`Nat.Partrec.Code` already restricts to Turing-computable
  partial maps); we bundle this as `hc : Computable f`, using Mathlib/core's
  `Computable` predicate rather than inventing a new boundedness notion.
- "`μ` is undefined off `Adm`": `μ` is only ever applied to the `Option Code`
  produced by `adm`, so any observation not admitted (mapped to `none` by
  `adm`) simply never reaches the `some` branch of `μ`; enforced by the
  `Option.map`-style case split below rather than a separate domain
  restriction, since `Option` already encodes the partial-map carrier.
- "`μ(Rfsl) = Rfsl`": the `none` branch of the match returns `none`.

No axioms: composed entirely from `Option`, `Nat.Partrec.Code`, and the
existing core `Computable` predicate.
-/

open Nat.Partrec (Code)

/-- `def:mu`: a manufacturing morphism. Given a deterministic computable map
`f : Code → Code` (witnessed computable via `hc`, satisfying M1/M2), `mu f hc`
lifts `f` to the admitted-observation carrier `Option Code` (the codomain of
`adm`), acting as `f` on admitted observations (`some o ↦ some (f o)`) and
sending `Rfsl` to `Rfsl` (`none ↦ none`), so `μ` is undefined off `Adm` and
`μ(Rfsl) = Rfsl`. -/
def mu (f : Code → Code) (_hc : Computable f) : Option Code → Option Code
  | some o => some (f o)
  | none => none
