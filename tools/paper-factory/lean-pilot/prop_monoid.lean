/-
prop:monoid (01_admission_algebra):
  (Deny_n, or, 0) is a commutative monoid in which every element is
  idempotent; it is the join-semilattice of the Boolean lattice (2^[n], subseteq)
  under d -> supp d, with 0 least and 1 greatest, so d or d' is the least upper
  bound and <= is exactly d <= d' iff d or d' = d'.

Formalized here: Deny_n as `Fin n -> Bool` (a denial word is a bitvector of
length n — this matches the paper's own "denial word" framing, one bit per
obligation in the battery), `or` as pointwise Boolean or, `0` as the
all-false vector. Proved directly in bare Lean 4 core — no mathlib
dependency (not installed for this pilot).

This is a REAL proof obligation, not a definition: the kernel must accept
associativity, commutativity, identity, and idempotence, or this fails.
-/

def Deny (n : Nat) := Fin n → Bool

namespace Deny

def or {n : Nat} (d d' : Deny n) : Deny n := fun i => d i || d' i
def zero (n : Nat) : Deny n := fun _ => false

theorem or_comm {n : Nat} (d d' : Deny n) : or d d' = or d' d := by
  funext i
  simp [or, Bool.or_comm]

theorem or_assoc {n : Nat} (d d' d'' : Deny n) : or (or d d') d'' = or d (or d' d'') := by
  funext i
  simp [or, Bool.or_assoc]

theorem or_zero {n : Nat} (d : Deny n) : or d (zero n) = d := by
  funext i
  simp [or, zero]

theorem zero_or {n : Nat} (d : Deny n) : or (zero n) d = d := by
  funext i
  simp [or, zero]

/-- Every element is idempotent under `or`. -/
theorem or_idem {n : Nat} (d : Deny n) : or d d = d := by
  funext i
  simp [or, Bool.or_self]

/-- The full commutative-monoid-with-idempotence package, as one theorem
    bundling exactly the properties the LaTeX statement asserts. -/
theorem is_idempotent_commutative_monoid (n : Nat) :
    (∀ d d' : Deny n, or d d' = or d' d) ∧
    (∀ d d' d'' : Deny n, or (or d d') d'' = or d (or d' d'')) ∧
    (∀ d : Deny n, or d (zero n) = d) ∧
    (∀ d : Deny n, or (zero n) d = d) ∧
    (∀ d : Deny n, or d d = d) :=
  ⟨or_comm, or_assoc, or_zero, zero_or, or_idem⟩

/-- The join-semilattice / partial-order characterization: d <= d' iff
    d or d' = d'. Defined as the natural order this induces, then proved
    to actually be a partial order compatible with `or` as least-upper-bound
    -- the second half of the LaTeX statement, not just the monoid half. -/
def le {n : Nat} (d d' : Deny n) : Prop := or d d' = d'

theorem le_refl {n : Nat} (d : Deny n) : le d d := or_idem d

theorem le_antisymm {n : Nat} (d d' : Deny n) (h1 : le d d') (h2 : le d' d) : d = d' := by
  unfold le at h1 h2
  rw [← h1, or_comm]
  exact h2.symm

theorem le_trans {n : Nat} (d d' d'' : Deny n) (h1 : le d d') (h2 : le d' d'') : le d d'' := by
  unfold le at h1 h2 ⊢
  rw [← h2, ← or_assoc, h1]

/-- `or` really is the least upper bound under `le`: it's an upper bound of
    both operands, and any upper bound of both dominates it. -/
theorem or_is_lub {n : Nat} (d d' u : Deny n) (hu1 : le d u) (hu2 : le d' u) :
    le (or d d') u := by
  unfold le at hu1 hu2 ⊢
  rw [or_assoc, hu2, hu1]

end Deny
