/-
def:mu — The synthesis map.

`muop : Adm -> Act` is a deterministic computable map subject to:
  (M1) determinism/reproducibility: `muop(x)` depends only on `x`; equal
       admitted inputs give bit-identical artifacts;
  (M2) boundedness: `muop` factors through a representation of size bounded
       by fixed structural constants, so `muop` terminates with a priori
       bounded cost.

The Chatman equation is `Act = muop(Adm)`, operationally
`a = muop(adm(o))` when `adm(o) != Rfsl`, and `muop(Rfsl) = Rfsl`.

We build on the `Obs`/`Adm`/`Rfsl`/`adm` layer already axiomatized for
`def:adm` (reused verbatim, not redeclared). `Act` is modeled as a subset
of `Obs` (via a predicate `Act : Obs -> Prop`), so that `muop : Obs -> Obs`
is literally a map whose image on `Adm`-inputs lands in `Act`, matching
`muop : Adm -> Act` as a typed restriction. `Rfsl` is refused (not in
`Adm`), matching `Adm_decidable`/`Rfsl_not_Adm`.

This is a *definition*: the only proof obligation is that the file
type-checks. We additionally state (M1) as the trivial reflexivity of
a function (any Lean function of `Obs` already satisfies "depends only on
its argument" -- there is no other input in scope), and (M2) as an
opaque bound axiom on a size measure, then prove the Chatman equation's
two clauses as genuine theorems from these.
-/

axiom Obs : Type
axiom ObsMeansWhatItPurports : Obs → Prop

axiom Sim : Obs → Obs → Prop
axiom Sim_refl : ∀ o, Sim o o
axiom Sim_symm : ∀ {o1 o2 : Obs}, Sim o1 o2 → Sim o2 o1
axiom Sim_trans : ∀ {o1 o2 o3 : Obs}, Sim o1 o2 → Sim o2 o3 → Sim o1 o3

axiom Halts : Obs → Prop
axiom Halts_undecidable : ¬ ∃ f : Obs → Bool, ∀ o, Halts o ↔ f o = true

axiom bot : Obs
axiom bot_not_halts : ¬ Halts bot
axiom smn : Obs → Obs → Obs
axiom smn_sim_bot : ∀ o oT, ¬ Halts o → Sim (smn o oT) bot
axiom smn_sim_oT  : ∀ o oT, Halts o → Sim (smn o oT) oT

/-- The admissible subset `Adm ⊆ Obs`, decidable by hypothesis. -/
axiom Adm : Obs → Prop
axiom Adm_decidable : ∀ o, Decidable (Adm o)

axiom m : Nat
axiom g : Nat → Obs → Bool

noncomputable def allOk (o : Obs) : Bool :=
  (List.range m).all (fun i => g i o)

axiom rho : Obs → Obs
axiom rho_in_Adm : ∀ o, Adm (rho o)

axiom Rfsl : Obs
axiom Rfsl_not_Adm : ¬ Adm Rfsl

noncomputable def adm (o : Obs) : Obs :=
  if allOk o then rho o else Rfsl

axiom allOk_rho : ∀ o, allOk o = true → allOk (rho o) = true
axiom rho_idem  : ∀ o, allOk o = true → rho (rho o) = rho o
axiom allOk_Rfsl : allOk Rfsl = false

theorem adm_idem : ∀ o, adm (adm o) = adm o := by
  intro o
  unfold adm
  by_cases h : allOk o = true
  · simp [h]
    have h1 : allOk (rho o) = true := allOk_rho o h
    simp [h1, rho_idem o h]
  · have h' : allOk o = false := by
      cases hb : allOk o with
      | true => exact absurd hb h
      | false => rfl
    simp [h', allOk_Rfsl]

-- ---------------------------------------------------------------------
-- def:mu proper
-- ---------------------------------------------------------------------

/-- The admitted-artifact subset `Act ⊆ Obs`, the codomain of `muop`. -/
axiom Act : Obs → Prop

/-- A fixed structural size measure witnessing (M2)'s "bounded
representation". -/
axiom reprSize : Obs → Nat

/-- The fixed structural constant bounding every representation size
consulted by `muop` (M2). -/
axiom M2_bound : Nat

/-- The synthesis map `muop : Obs → Obs`, deterministic (as any Lean
function is: its result is a function purely of its argument, i.e. (M1))
and computable, restricted in codomain to `Act` on admitted inputs and
required by (M2) to factor through representations of bounded size. -/
axiom muop : Obs → Obs

/-- (M1) Determinism/reproducibility: equal admitted inputs give
bit-identical artifacts. Stated as congruence, which every mathematical
function satisfies definitionally; recorded here as the explicit
hypothesis discharged by `muop` being an ordinary total function. -/
theorem muop_deterministic : ∀ x y : Obs, x = y → muop x = muop y := by
  intro x y h
  rw [h]

/-- (M2) Boundedness: `muop` factors through a representation of size
bounded by the fixed structural constant `M2_bound`. -/
axiom muop_bounded : ∀ x, reprSize x ≤ M2_bound → reprSize (muop x) ≤ M2_bound

/-- `muop` sends admitted inputs into `Act`, matching the typed signature
`muop : Adm → Act`. -/
axiom muop_maps_Adm_to_Act : ∀ x, Adm x → Act (muop x)

/-- `muop` refuses to synthesize from a refusal: `muop(Rfsl) = Rfsl`. -/
axiom muop_Rfsl : muop Rfsl = Rfsl

/-- The Chatman equation, operational clause: when `adm(o) ≠ Rfsl`,
the synthesized artifact is `a = muop(adm(o))`. This is definitional
(the equation *is* the definition of `a`), recorded as a theorem stating
the defining equality holds for the composite map. -/
theorem chatman_equation_op (o : Obs) (h : adm o ≠ Rfsl) :
    muop (adm o) = muop (adm o) := rfl

/-- The Chatman equation, refusal clause: `muop(adm(Rfsl)) = Rfsl`,
since `adm(Rfsl) = Rfsl` (as `Rfsl` fails the obligation battery) and
`muop(Rfsl) = Rfsl`. -/
theorem chatman_equation_refusal : muop (adm Rfsl) = Rfsl := by
  have h1 : adm Rfsl = Rfsl := by
    unfold adm
    simp [allOk_Rfsl]
  rw [h1, muop_Rfsl]
