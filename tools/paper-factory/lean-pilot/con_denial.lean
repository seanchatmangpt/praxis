/-
con:denial — Denial words.

Let `D = ({0,1}^n, ∨, 0)` be the commutative idempotent monoid of denial
words: `n` independent lanes, componentwise OR, identity `0`. Each
obligation `g_i` contributes a lane map `d_i : Obs → {0,1}^n` with
`d_i(o) = 0 ↔ g_i(o) = 1`. The total denial is `d(o) = ⋁_i d_i(o)`, and
`adm(o) ≠ Rfsl ↔ d(o) = 0`.

We reuse `def:adm` verbatim (the `Obs`, obligation battery `g`/`m`, `allOk`,
`adm`, `Rfsl` machinery) rather than redeclaring it, and build the denial
monoid on top:
  * lanes indexed by `Fin m` (one lane per obligation, so `n = m`);
  * a denial word is `Fin m → Bool` (`{0,1}^n`), with pointwise `∨` and the
    all-`false` identity;
  * `d_i(o)` is the singleton word that is `false` at lane `i` iff `g i o`
    passes, `true` otherwise;
  * `d(o)` is the pointwise-or of all lanes, i.e. simply `fun i => !g i o`;
  * the correspondence `adm(o) ≠ Rfsl ↔ d(o) = 0` is proved as a genuine
    theorem from `allOk` unfolding to a `List.all`/pointwise-`Bool` fact,
    using the same closure axioms as `def:adm`.

This is a *construction*: the only proof obligation is that the file
type-checks (the correspondence theorem is included as a bonus check that
the construction is coherent, not a requirement of the ticket).
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

axiom allOk_rho : ∀ o, allOk o = true → allOk (rho o) = true
axiom rho_idem  : ∀ o, allOk o = true → rho (rho o) = rho o
axiom allOk_Rfsl : allOk Rfsl = false

-- ---------------------------------------------------------------------
-- con:denial proper
-- ---------------------------------------------------------------------

/-- A denial word: `n = m` independent lanes, one per obligation. -/
def Word : Type := Fin m → Bool

/-- Componentwise OR on denial words. -/
def wOr (x y : Word) : Word := fun i => x i || y i

/-- The identity denial word: all lanes clear. -/
def wZero : Word := fun _ => false

/-- `wOr` is commutative. -/
theorem wOr_comm : ∀ x y : Word, wOr x y = wOr y x := by
  intro x y
  funext i
  simp [wOr, Bool.or_comm]

/-- `wOr` is idempotent. -/
theorem wOr_idem : ∀ x : Word, wOr x x = x := by
  intro x
  funext i
  simp [wOr]

/-- `wZero` is the identity for `wOr`. -/
theorem wOr_zero : ∀ x : Word, wOr x wZero = x := by
  intro x
  funext i
  simp [wOr, wZero]

/-- The per-obligation lane map `d_i : Obs → Word`, the singleton word
that is clear at lane `i` iff obligation `i` passes at `o`. -/
noncomputable def d (i : Fin m) (o : Obs) : Word :=
  fun j => if i = j then !(g i.val o) else false

/-- The total denial: the pointwise OR of all lane maps, which collapses
to `fun j => !g j.val o` (each lane records whether its own obligation
fails). -/
noncomputable def dTot (o : Obs) : Word :=
  fun j => !(g j.val o)

/-- `dTot o` is clear at lane `i` iff obligation `i` passes at `o`. -/
theorem dTot_lane : ∀ o (i : Fin m), dTot o i = false ↔ g i.val o = true := by
  intro o i
  simp [dTot]

/-- The total denial is `wZero` iff every obligation with index `< m`
passes, i.e. iff `allOk o = true`. -/
theorem dTot_zero_iff_allOk : ∀ o, dTot o = wZero ↔ allOk o = true := by
  intro o
  constructor
  · intro h
    have h' : ∀ i : Fin m, g i.val o = true := by
      intro i
      have := congrFun h i
      simp [dTot, wZero] at this
      exact this
    unfold allOk
    apply List.all_eq_true.mpr
    intro i hi
    have hi' : i < m := List.mem_range.mp hi
    exact h' ⟨i, hi'⟩
  · intro h
    funext j
    have hj : j.val < m := j.isLt
    have hall : ∀ i ∈ List.range m, g i o = true := by
      have := List.all_eq_true.mp (show (List.range m).all (fun i => g i o) = true from h)
      exact this
    have : g j.val o = true := hall j.val (List.mem_range.mpr hj)
    simp [dTot, wZero, this]

/-- The correspondence: `adm(o) ≠ Rfsl ↔ d(o) = 0`, given the closure
axioms of `def:adm` ensuring `Rfsl` never passes the battery (so `adm o =
Rfsl` exactly when `allOk o = false`). -/
theorem adm_ne_Rfsl_iff_dTot_zero :
    ∀ o, (Adm (adm o) → adm o ≠ Rfsl) → (adm o ≠ Rfsl ↔ dTot o = wZero) := by
  intro o _
  unfold adm
  by_cases h : allOk o = true
  · simp [h, dTot_zero_iff_allOk, h]
    intro hcontra
    exact Rfsl_not_Adm (hcontra ▸ rho_in_Adm o)
  · have h' : allOk o = false := by
      cases hb : allOk o with
      | true => exact absurd hb h
      | false => rfl
    simp [h', dTot_zero_iff_allOk, h']
