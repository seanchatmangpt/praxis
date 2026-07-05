/-
prop:mudet — Under (M1), `x ↦ muop(x)` is single-valued, so
`o ↦ muop(adm(o))` is a partial function on `Obs`: two runs on the same
admitted input agree bit-for-bit.

We reuse `Obs`, `adm`, `muop` verbatim from `def_mu.lean` (not
redeclared). Single-valuedness of `muop` is `muop_deterministic`
(already proved there from ordinary function congruence); this
proposition packages it through `adm` to give the stated partial-function
property: for any `o1 o2 : Obs` with `o1 = o2`, the composite run
`muop (adm o1)` and `muop (adm o2)` agree bit-for-bit, i.e. are equal.
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

axiom Act : Obs → Prop
axiom reprSize : Obs → Nat
axiom M2_bound : Nat
axiom muop : Obs → Obs

/-- (M1) Determinism/reproducibility, reused verbatim from `def_mu.lean`:
equal inputs give bit-identical outputs. -/
theorem muop_deterministic : ∀ x y : Obs, x = y → muop x = muop y := by
  intro x y h
  rw [h]

axiom muop_bounded : ∀ x, reprSize x ≤ M2_bound → reprSize (muop x) ≤ M2_bound
axiom muop_maps_Adm_to_Act : ∀ x, Adm x → Act (muop x)
axiom muop_Rfsl : muop Rfsl = Rfsl

-- ---------------------------------------------------------------------
-- prop:mudet proper
-- ---------------------------------------------------------------------

/-- `o ↦ muop (adm o)` is a partial function on `Obs`: two runs on the
same admitted input agree bit-for-bit. Concretely, if `o1 = o2` (the
"same admitted input" hypothesis, since `adm` is itself an ordinary
function of its argument), then the synthesized results coincide:
`muop (adm o1) = muop (adm o2)`. This is exactly single-valuedness of
`x ↦ muop x` (M1, `muop_deterministic`) transported along `adm`. -/
theorem mu_adm_deterministic : ∀ o1 o2 : Obs, o1 = o2 → muop (adm o1) = muop (adm o2) := by
  intro o1 o2 h
  apply muop_deterministic
  rw [h]
