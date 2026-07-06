import Praxis.Corpus.def_adm

/-!
Label: prop:retract

"$\adm$ restricted to $\Adm$ is the identity, and $\adm\circ\adm=\adm$ as
partial maps; hence $\Adm$ is a retract of $\dom(\adm)$."

`adm` is the partial map `Code → Option Code` defined in `def:adm`. Restricted
to `Adm` it is the identity: if `o ∈ Adm` and all obligations hold, `adm Adm gs
o = some o`. Composition as partial maps is Kleisli composition on `Option`
(`Option.bind`); idempotence `adm ∘ adm = adm` means
`(adm Adm gs o).bind (adm Adm gs) = adm Adm gs o` for every `o`, i.e. once `o`
has been admitted, re-running the admission test on the same `o` never fails
and never changes the result. Both facts are proved directly from the `if`
definition of `adm` by case analysis on the guard `o ∈ Adm ∧ gs.all (· o)`;
no axioms needed, this is a direct computation.
-/

open Nat.Partrec (Code)

/-- `adm` restricted to `Adm` (i.e. whenever the admission guard holds) is the
identity, realized as `some o`. -/
theorem adm_restrict_id (Adm : Set Code) [DecidablePred (· ∈ Adm)]
    (gs : List (Code → Bool)) (o : Code)
    (h : o ∈ Adm ∧ gs.all (fun g => g o)) :
    adm Adm gs o = some o := by
  unfold adm
  rw [if_pos h]

/-- `adm ∘ adm = adm` as partial maps (Kleisli composition on `Option`): once
`o` passes the admission test, re-admitting the result changes nothing and
never fails. Hence `Adm` is a retract of `dom (adm Adm gs)`. -/
theorem adm_comp_adm (Adm : Set Code) [DecidablePred (· ∈ Adm)]
    (gs : List (Code → Bool)) (o : Code) :
    (adm Adm gs o).bind (adm Adm gs) = adm Adm gs o := by
  unfold adm
  by_cases h : o ∈ Adm ∧ gs.all (fun g => g o)
  · rw [if_pos h, Option.bind_some, if_pos h]
  · rw [if_neg h]
    rfl

