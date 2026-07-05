/-
def:adm — The admission map.

Fix a decidable set `Adm ⊆ Obs` on which a finite battery of obligations
`g_1,...,g_m : Obs → {0,1}` is computable. The admission map is the partial
retraction

    adm : Obs → Adm ∪ {Rfsl},   adm(o) = ρ(o) ∈ Adm  if all g_i(o) = 1,
                                          Rfsl        otherwise,

with ρ a computable bounding/normalization and `Rfsl` the distinguished
refusal. Admission is idempotent on its image: `adm ∘ adm = adm`.

We build on the `Obs` computability layer already axiomatized for `thm:rice`
(reusing it verbatim rather than redeclaring it) and model:
  * the finite battery of `m` obligations as `g : Nat → Obs → Bool`, only
    the first `m` of which are consulted (`allOk`);
  * `Adm` as a subset of `Obs` (rather than a fresh type), so that the
    codomain `Adm ∪ {Rfsl}` is literally a subset of `Obs` and the
    retraction equation `adm ∘ adm = adm` type-checks as stated;
  * `rho` as the computable bounding/normalization map into `Adm`;
  * `Rfsl` as a distinguished observation outside `Adm`.

This is a *definition*: the only proof obligation is that the file
type-checks. We additionally prove the stated idempotence as a genuine
theorem from the natural closure axioms a real admission map satisfies
(the battery re-passes on normalized output, and `rho` is idempotent on
its own image, and `Rfsl` never passes the battery) — this is not required
for the definition to type-check, but it makes the "idempotent on its
image" clause of the definition concrete and checked rather than asserted.
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

-- ---------------------------------------------------------------------
-- def:adm proper
-- ---------------------------------------------------------------------

/-- The admissible subset `Adm ⊆ Obs`, decidable by hypothesis. -/
axiom Adm : Obs → Prop
axiom Adm_decidable : ∀ o, Decidable (Adm o)

/-- Size of the finite obligation battery. -/
axiom m : Nat

/-- The obligation battery `g_1, ..., g_m : Obs → {0,1}`, each computable
(represented as `Obs → Bool`); only indices `< m` are consulted. -/
axiom g : Nat → Obs → Bool

/-- All obligations pass at `o`. -/
noncomputable def allOk (o : Obs) : Bool :=
  (List.range m).all (fun i => g i o)

/-- The computable bounding/normalization map into `Adm`. -/
axiom rho : Obs → Obs
axiom rho_in_Adm : ∀ o, Adm (rho o)

/-- The distinguished refusal, outside `Adm`. -/
axiom Rfsl : Obs
axiom Rfsl_not_Adm : ¬ Adm Rfsl

/-- The admission map: `adm(o) = ρ(o)` if all obligations pass at `o`,
else `Rfsl`. Its image is exactly `Adm ∪ {Rfsl}`. -/
noncomputable def adm (o : Obs) : Obs :=
  if allOk o then rho o else Rfsl

-- Closure axioms making `adm` a genuine (idempotent) retraction:
-- normalized output re-passes the battery, `rho` is idempotent on its own
-- image, and `Rfsl` never passes the battery.
axiom allOk_rho : ∀ o, allOk o = true → allOk (rho o) = true
axiom rho_idem  : ∀ o, allOk o = true → rho (rho o) = rho o
axiom allOk_Rfsl : allOk Rfsl = false

/-- Admission is idempotent on its image: `adm ∘ adm = adm`. -/
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
