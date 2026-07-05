/-
thm:freehom

Φ_o : (Stage*, ·, ε) → (Deny, compose, Adml) is the unique monoid
homomorphism extending φ_o along Stage ↪ Stage*; because the target is
commutative and idempotent, Φ_o(w) depends only on the set of distinct
stage-denials occurring in w, invariant under reordering and repetition.

This is a REAL proof obligation (theorem), proved in bare Lean 4 core.

Setup reused verbatim (redeclared standalone, as `def_pipeline.lean` itself
does) from:
  - def:pipeline  (Stage, Pipeline = Stage*, Φ_o as a fold of φ_o along w)
  - prop:monoid   (Deny n as the commutative idempotent monoid target,
                   here instantiated as the abstract carrier with `or`/`zero`
                   satisfying exactly the laws proved there)
-/

/-- `Stage`, the set of admission stages, kept abstract, as in def:pipeline. -/
opaque Stage : Type

/-- A pipeline is a finite sequence `w ∈ Stage*`, the free monoid on `Stage`. -/
abbrev Pipeline := List Stage

/-- The target monoid `Deny n`, exactly as in prop:monoid: a denial word is a
bitvector of length `n`. -/
def Deny (n : Nat) := Fin n → Bool

namespace Deny

def or {n : Nat} (d d' : Deny n) : Deny n := fun i => d i || d' i
def zero (n : Nat) : Deny n := fun _ => false

theorem or_comm {n : Nat} (d d' : Deny n) : or d d' = or d' d := by
  funext i; simp [or, Bool.or_comm]

theorem or_assoc {n : Nat} (d d' d'' : Deny n) :
    or (or d d') d'' = or d (or d' d'') := by
  funext i; simp [or, Bool.or_assoc]

theorem or_idem {n : Nat} (d : Deny n) : or d d = d := by
  funext i; simp [or, Bool.or_self]

theorem zero_or {n : Nat} (d : Deny n) : or (zero n) d = d := by
  funext i; simp [or, zero]

end Deny

variable {n : Nat} (φ : Stage → Deny n)

/-- The aggregate denial `Φ_o(w) = ⋁ᵢ φ_o(s_i)`, `Φ_o(ε) = Adml = zero`,
built as a right fold so `w = ε` and `w = s :: w'` match the paper's
recursive presentation of the free-monoid extension directly. -/
def aggregateDenial : Pipeline → Deny n
  | [] => Deny.zero n
  | s :: w => Deny.or (φ s) (aggregateDenial w)

/-- `Φ_o(ε) = Adml`, definitionally, as in def:pipeline. -/
example : aggregateDenial φ [] = Deny.zero n := rfl

/-- `Φ_o` really is a monoid homomorphism out of the free monoid `Stage*`:
it sends concatenation to `compose` (`or`) and `ε` to `Adml`. -/
theorem aggregateDenial_append (w w' : Pipeline) :
    aggregateDenial φ (w ++ w') = Deny.or (aggregateDenial φ w) (aggregateDenial φ w') := by
  induction w with
  | nil => simp [aggregateDenial, Deny.zero_or]
  | cons s w ih =>
      show Deny.or (φ s) (aggregateDenial φ (w ++ w'))
        = Deny.or (Deny.or (φ s) (aggregateDenial φ w)) (aggregateDenial φ w')
      rw [ih, Deny.or_assoc]

/-- Uniqueness: `aggregateDenial φ` is the *only* function `Stage* → Deny n`
that sends `ε ↦ Adml` and extends `φ` compatibly with `s :: w ↦ φ(s) · F(w)`
(i.e. the unique monoid homomorphism extending `φ` along `Stage ↪ Stage*`). -/
theorem aggregateDenial_unique (F : Pipeline → Deny n)
    (hnil : F [] = Deny.zero n)
    (hcons : ∀ s w, F (s :: w) = Deny.or (φ s) (F w)) :
    ∀ w, F w = aggregateDenial φ w
  | [] => hnil
  | s :: w => by rw [hcons s w, aggregateDenial_unique F hnil hcons w]; rfl

/-- Invariance under reordering: swapping two adjacent stages in a pipeline
does not change the aggregate denial, since the target monoid is
commutative. -/
theorem aggregateDenial_swap (a b : Stage) (w : Pipeline) :
    aggregateDenial φ (a :: b :: w) = aggregateDenial φ (b :: a :: w) := by
  show Deny.or (φ a) (Deny.or (φ b) (aggregateDenial φ w))
     = Deny.or (φ b) (Deny.or (φ a) (aggregateDenial φ w))
  rw [← Deny.or_assoc, ← Deny.or_assoc, Deny.or_comm (φ a) (φ b)]

/-- Invariance under repetition: a repeated stage contributes nothing new,
since the target monoid is idempotent — so `Φ_o(w)` depends only on the
*set* of distinct stage-denials occurring in `w`. -/
theorem aggregateDenial_dup (a : Stage) (w : Pipeline) :
    aggregateDenial φ (a :: a :: w) = aggregateDenial φ (a :: w) := by
  show Deny.or (φ a) (Deny.or (φ a) (aggregateDenial φ w))
     = Deny.or (φ a) (aggregateDenial φ w)
  rw [← Deny.or_assoc, Deny.or_idem]
