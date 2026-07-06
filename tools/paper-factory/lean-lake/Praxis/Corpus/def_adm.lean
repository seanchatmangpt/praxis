import Praxis.Corpus.cor_noadmit

/-!
Label: def:adm

"Fix a decidable sub-collection $\Adm\subseteq\Obs$ and a finite battery of
total, computable obligations $g_1,\dots,g_m$; the admission map $\adm$ is the
partial retraction sending $o$ to $\rho(o)\in\Adm$ if all $g_i(o)=1$, else to
$\Rfsl$."

`Obs` is instantiated as `Nat.Partrec.Code` (`Code`), matching the encoding
used throughout `cor_noadmit`/`thm_rice`. A "decidable sub-collection"
`Adm ⊆ Obs` is a `Set Code` with a `DecidablePred` instance on membership. The
"finite battery of total, computable obligations" is a `List (Code → Bool)` --
`List` is finite by construction and every `Code → Bool` function is total by
Lean's totality; `gs.all` folds them into the single "all `g_i(o) = 1`" test.
The retraction `ρ` is realized as the identity on the underlying carrier
(returning `o` itself once membership in `Adm` and the obligation battery are
confirmed): since `Adm : Set Code` is a *sub*-collection of the same carrier
`Code`, the retraction onto it is literally the inclusion map, requiring no
separate `ρ` function. Failure (either `o ∉ Adm` or some `g_i(o) ≠ 1`) maps to
`Rfsl`, realized as `none` in `Option Code`.

No axioms: this is a plain data-level definition composed from `Set`,
`DecidablePred`, `List`, `Bool`, and `Option`, all from core/Mathlib.
-/

open Nat.Partrec (Code)

/-- `def:adm`: the admission map. Given a decidable admissible sub-collection
`Adm ⊆ Code` and a finite battery `gs` of total computable obligations
`Code → Bool`, `adm Adm gs o` returns `some o` (the retraction `ρ o`, realized
as the inclusion since `Adm ⊆ Code`) when `o ∈ Adm` and every obligation in
`gs` evaluates to `true` on `o`, and `none` (`Rfsl`) otherwise. -/
def adm (Adm : Set Code) [DecidablePred (· ∈ Adm)] (gs : List (Code → Bool))
    (o : Code) : Option Code :=
  if o ∈ Adm ∧ gs.all (fun g => g o) then some o else none
