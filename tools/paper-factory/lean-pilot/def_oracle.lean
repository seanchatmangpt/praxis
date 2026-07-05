/-
Label: def:oracle
Kind: definition

For implementation `f : D → R` and specification `S ⊆ D × R`, the correctness
question is `Q_S(f) ≡ ∀ x, (x, f x) ∈ S`; a decidable oracle for `f` on finite
`X ⊆ D` is a total computable predicate `Ω : X → {0,1}` intended to witness
`(x, f x) ∈ S`, terminating on every `x ∈ X`.

We model:
- `D`, `R` as arbitrary types.
- a specification `S` as a predicate on `D × R`.
- the correctness question `Q_S f` as a Prop.
- an oracle for `f` restricted to a finite subset (given as a `List D` acting
  as the finite domain `X ⊆ D`) as a function `Ω : D → Bool`, total and
  computable by construction since it is an ordinary Lean function into
  `Bool` (the two-element type `{0,1}`), together with the intended-witness
  condition relating `Ω` to membership in `S`.
-/

section Oracle

variable {D R : Type}

/-- A specification relating inputs to outputs. -/
def Spec (D R : Type) := D → R → Prop

/-- The correctness question `Q_S(f)`. -/
def CorrectnessQuestion (S : Spec D R) (f : D → R) : Prop :=
  ∀ x : D, S x (f x)

/-- A decidable oracle for `f` on the finite domain `X` (given as a `List D`):
a total computable predicate `Ω : D → Bool`, together with the intended
meaning that `Ω x = true` witnesses `(x, f x) ∈ S`. Since `Ω` is an ordinary
Lean function `D → Bool`, it is total (defined on every `x`) and computable
by construction; termination on every `x ∈ X` is automatic. -/
structure Oracle (S : Spec D R) (f : D → R) (X : List D) where
  Ω : D → Bool
  witnesses : ∀ x ∈ X, Ω x = true → S x (f x)

end Oracle
