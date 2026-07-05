/-
prop:semilattice — ({0,1}^n, ∨, 0) is a bounded join-semilattice;
consequently adding obligations can only enlarge the denial, never
shrink it.

We reuse `con:denial`'s `Word`, `wOr`, `wZero` (rebuilt here on the same
axiomatic skeleton, since Lean files don't `import` each other in this
pilot) and prove:
  1. the semilattice laws: `wOr` is commutative, associative, idempotent,
     and `wZero` is the bottom/identity element — i.e. `(Word, wOr, wZero)`
     is a bounded join-semilattice;
  2. the pointwise order `x ≤ y := ∀ i, x i = true → y i = true` is a
     genuine partial order for which `wOr` is the least-upper-bound
     operation, so joining in any extra lane's contribution (`wOr x y`)
     can never make the result smaller than what was already there:
     `y ≤ wOr x y`. This is the formal content of "adding obligations can
     only enlarge the denial, never shrink it": each new obligation lane
     contributes via `wOr`, and `wOr` only ever grows (in this order) the
     accumulated denial word.
-/

axiom m : Nat

def Word : Type := Fin m → Bool

def wOr (x y : Word) : Word := fun i => x i || y i

def wZero : Word := fun _ => false

def wLe (x y : Word) : Prop := ∀ i, x i = true → y i = true

-- ---------------------------------------------------------------------
-- Semilattice laws
-- ---------------------------------------------------------------------

theorem wOr_comm : ∀ x y : Word, wOr x y = wOr y x := by
  intro x y
  funext i
  simp [wOr, Bool.or_comm]

theorem wOr_assoc : ∀ x y z : Word, wOr (wOr x y) z = wOr x (wOr y z) := by
  intro x y z
  funext i
  simp [wOr, Bool.or_assoc]

theorem wOr_idem : ∀ x : Word, wOr x x = x := by
  intro x
  funext i
  simp [wOr]

theorem wOr_zero_right : ∀ x : Word, wOr x wZero = x := by
  intro x
  funext i
  simp [wOr, wZero]

theorem wOr_zero_left : ∀ x : Word, wOr wZero x = x := by
  intro x
  funext i
  simp [wOr, wZero]

-- ---------------------------------------------------------------------
-- Order structure: `wLe` is a partial order with bottom `wZero`, and
-- `wOr` is its join (least upper bound).
-- ---------------------------------------------------------------------

theorem wLe_refl : ∀ x : Word, wLe x x := by
  intro x i h
  exact h

theorem wLe_trans : ∀ x y z : Word, wLe x y → wLe y z → wLe x z := by
  intro x y z hxy hyz i hi
  exact hyz i (hxy i hi)

theorem wLe_antisymm : ∀ x y : Word, wLe x y → wLe y x → x = y := by
  intro x y hxy hyx
  funext i
  cases hx : x i with
  | false =>
    cases hy : y i with
    | false => simp [hx, hy]
    | true =>
      have hcontra := hyx i hy
      simp [hx] at hcontra
  | true =>
    have hy := hxy i hx
    simp [hx, hy]

theorem wZero_bot : ∀ x : Word, wLe wZero x := by
  intro x i h
  simp [wZero] at h

/-- Adding `x`'s contribution via `wOr` can only enlarge `y`, never
shrink it: every lane already set in `y` stays set in `wOr x y`. This is
the formal statement of "adding obligations can only enlarge the denial,
never shrink it". -/
theorem le_wOr_right : ∀ x y : Word, wLe y (wOr x y) := by
  intro x y i hi
  simp [wOr, hi]

theorem le_wOr_left : ∀ x y : Word, wLe x (wOr x y) := by
  intro x y i hi
  simp [wOr, hi]

/-- `wOr x y` is an upper bound of `x` and `y`, and it is the *least*
such upper bound: any `z` above both `x` and `y` is above `wOr x y`. -/
theorem wOr_least_upper_bound :
    ∀ x y z : Word, wLe x z → wLe y z → wLe (wOr x y) z := by
  intro x y z hxz hyz i hi
  simp [wOr] at hi
  cases hi with
  | inl h => exact hxz i h
  | inr h => exact hyz i h
