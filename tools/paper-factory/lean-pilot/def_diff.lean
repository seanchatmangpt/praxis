/-
Label: def:diff
Kind: definition

For f,g : D → R targeting the same S, the differential oracle is
Ω_{f,g}(x) = [f(x) = g(x)]; a test passes at x iff Ω_{f,g}(x) = 1.
-/

section Oracle

variable {D R : Type}

/-- A specification relating inputs to outputs. -/
def Spec (D R : Type) := D → R → Prop

/-- The correctness question `Q_S(f)`. -/
def CorrectnessQuestion (S : Spec D R) (f : D → R) : Prop :=
  ∀ x : D, S x (f x)

/-- A decidable oracle for `f` on the finite domain `X` (given as a `List D`). -/
structure Oracle (S : Spec D R) (f : D → R) (X : List D) where
  Ω : D → Bool
  witnesses : ∀ x ∈ X, Ω x = true → S x (f x)

end Oracle

section Diff

variable {D R : Type} [DecidableEq R]

/-- The differential oracle `Ω_{f,g}(x) = [f(x) = g(x)]`, comparing two
implementations `f g : D → R` targeting the same specification. -/
def diffOracle (f g : D → R) : D → Bool :=
  fun x => decide (f x = g x)

/-- A test at `x` passes iff `Ω_{f,g}(x) = true`, i.e. `f x = g x`. -/
def diffTestPasses (f g : D → R) (x : D) : Prop :=
  diffOracle f g x = true

end Diff
