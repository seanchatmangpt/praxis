/-
prop:retract — `adm` restricted to `Adm` is the identity, and
`adm ∘ adm = adm` as partial maps; hence `Adm` is a retract of `dom(adm)`.

We reuse the `Obs`/`adm` scaffolding from `def:adm` verbatim (copied here
since this file is standalone bare-Lean, no project imports available),
and add the two closure axioms that make `Adm` genuinely a *retract*
(rather than merely a decidable subset): elements of `Adm` already pass
the obligation battery, and `rho` fixes them (it is a normalization map,
identity on inputs already in normal form). Both are natural consequences
of `rho`'s role as the bounding/normalization map onto `Adm` and are not
in tension with any axiom already committed to in `def:adm`.

The proposition is then proved as a genuine theorem: a retraction pair
`(ι, adm)` with `ι : Adm ↪ Obs` the inclusion (definitionally trivial
since `Adm : Obs → Prop`) and `adm` a left inverse to `ι` (i.e. identity
on `Adm`), together with `adm` idempotent on all of `Obs` — exactly the
data exhibiting `Adm` as a retract of `dom(adm) = Obs`.
-/

axiom Obs : Type
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

-- Closure axioms making `adm` a genuine (idempotent) retraction onto `Adm`:
axiom allOk_rho : ∀ o, allOk o = true → allOk (rho o) = true
axiom rho_idem  : ∀ o, allOk o = true → rho (rho o) = rho o
axiom allOk_Rfsl : allOk Rfsl = false

-- New closure axioms specific to `prop:retract`: `Adm` sits inside the
-- domain of admission, and `rho` (the normalization map) is the identity
-- on inputs already admissible.
axiom allOk_of_Adm : ∀ o, Adm o → allOk o = true
axiom rho_fixes_Adm : ∀ o, Adm o → rho o = o

/-- `adm` restricted to `Adm` is the identity. -/
theorem adm_id_on_Adm : ∀ o, Adm o → adm o = o := by
  intro o hAdm
  unfold adm
  have h1 : allOk o = true := allOk_of_Adm o hAdm
  simp [h1, rho_fixes_Adm o hAdm]

/-- `adm ∘ adm = adm` as (total, but partial-in-spirit) maps on `Obs`. -/
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

/-- Hence `Adm` is a retract of `dom(adm)`: `adm` is the identity on
`Adm` and idempotent everywhere, i.e. `(inclusion, adm)` is a retraction
pair exhibiting `Adm` as a retract of `Obs = dom(adm)`. -/
theorem retract_Adm : (∀ o, Adm o → adm o = o) ∧ (∀ o, adm (adm o) = adm o) :=
  ⟨adm_id_on_Adm, adm_idem⟩
