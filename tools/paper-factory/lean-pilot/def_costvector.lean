/-
def:costvector — The cost vector.

Assign a plan `c = (c_0, c_1, ..., c_k)`, with `c_0 ∈ {0,1}` the unadmitted
indicator (`0` = admitted) and `c_1, ..., c_k` bounded secondary costs.
Plans are ordered by the lexicographic order `⪯_lex` on these vectors.

We reuse the `Obs`/`Adm`/`adm` vocabulary from `def:adm` verbatim (a plan's
unadmitted indicator is read off `adm`), and model:
  * a cost vector as `c_0 : Bool` (the unadmitted indicator, `false` =
    admitted, matching "0 = admitted") together with a length-`k` vector of
    natural-number secondary costs `Fin k → Nat`;
  * the unadmitted indicator derived from a plan's underlying observation
    via `adm`, `unadmitted o := ¬ Adm (adm o)`, i.e. `c_0 = true` exactly
    when `adm` sent `o` to `Rfsl` (outside `Adm`);
  * the lexicographic order `⪯_lex` on cost vectors, first comparing `c_0`
    (admitted `<` unadmitted, i.e. `false < true`), then, when the
    indicators agree, comparing the secondary-cost vectors lexicographically
    by increasing index.

This is a *definition*: the only proof obligation is that the file
type-checks.
-/

axiom Obs : Type
axiom Adm : Obs → Prop
axiom Adm_decidable : ∀ o, Decidable (Adm o)
axiom Rfsl : Obs
axiom Rfsl_not_Adm : ¬ Adm Rfsl
axiom rho : Obs → Obs
axiom allOk : Obs → Bool
noncomputable def adm (o : Obs) : Obs :=
  if allOk o then rho o else Rfsl

-- ---------------------------------------------------------------------
-- def:costvector proper
-- ---------------------------------------------------------------------

/-- Number of secondary cost components, `k`. -/
axiom k : Nat

/-- A cost vector `c = (c_0, c_1, ..., c_k)`: the unadmitted indicator
`c_0 : Bool` (`false` = admitted) paired with `k` bounded secondary
costs, represented as a total function `Fin k → Nat`. -/
structure CostVector where
  c0 : Bool
  csec : Fin k → Nat

/-- The unadmitted indicator read off a plan's underlying observation via
`adm`: `true` exactly when `adm` refused `o` (sent it outside `Adm`,
i.e. to `Rfsl`), `false` (admitted) otherwise. -/
noncomputable def unadmitted (o : Obs) : Bool :=
  open Classical in
  if Adm (adm o) then false else true

/-- The cost vector attached to a plan via its underlying observation. -/
noncomputable def costVectorOf (o : Obs) (csec : Fin k → Nat) : CostVector :=
  { c0 := unadmitted o, csec := csec }

/-- Lexicographic order on the secondary-cost tails, comparing indices
`0, 1, ..., k-1` in increasing order: `v ⪯ w` iff `v` and `w` agree on
every earlier index and, at the first index where they may differ, `v`'s
value is `≤` (this is the standard "first difference decides" order,
phrased directly as a `≤`-at-first-difference relation). -/
def secLex (v w : Fin k → Nat) : Prop :=
  ∀ i : Fin k, (∀ j : Fin k, j < i → v j = w j) → v i ≤ w i

/-- The lexicographic order `⪯_lex` on cost vectors: admitted plans
(`c0 = false`) precede unadmitted ones (`c0 = true`); among plans with
equal unadmitted indicator, compare secondary costs via `secLex`. -/
def costLex (a b : CostVector) : Prop :=
  (a.c0 = false ∧ b.c0 = true) ∨ (a.c0 = b.c0 ∧ secLex a.csec b.csec)
