import Praxis.Corpus.def_mu

/-!
Label: prop:mudet

"Under (M1), $x\mapsto\muop(x)$ is single-valued, so $o\mapsto\muop(\adm(o))$ is
a partial function on $\Obs$: two runs on the same admitted input agree
bit-for-bit."

`mu f hc : Option Code → Option Code` is a plain Lean function (M1 is
automatic for Lean functions, per `def_mu`). Single-valuedness / determinism
is exactly the statement that equal inputs produce equal outputs, i.e. that
`mu f hc` respects `Eq` -- which is `congrArg`. Composing with `adm` (a plain
Lean function, `def:adm`) preserves this: two runs on the same admitted input
`o` (`adm o = adm o'` whenever `o = o'`) agree bit-for-bit on
`mu f hc (adm o)`.

No axioms: `congrArg`/`congrFun` are core Lean, not Mathlib additions.
-/

open Nat.Partrec (Code)

/-- `prop:mudet`: `mu f hc` is single-valued -- equal admitted inputs produce
identical outputs ("two runs on the same admitted input agree bit-for-bit").
This is definitional determinism of Lean functions (`congrArg`), applied to
the manufacturing morphism `mu f hc` from `def:mu`. -/
theorem mu_deterministic (f : Code → Code) (hc : Computable f)
    (x y : Option Code) (h : x = y) :
    mu f hc x = mu f hc y :=
  congrArg (mu f hc) h

/-- Corollary form matching the statement's phrasing: composed with `adm`,
two runs on the same admitted input `o = o'` agree bit-for-bit. -/
theorem mu_adm_deterministic {α : Type*} (f : Code → Code) (hc : Computable f)
    (adm : α → Option Code) (o o' : α) (h : o = o') :
    mu f hc (adm o) = mu f hc (adm o') :=
  congrArg (fun x => mu f hc (adm x)) h
