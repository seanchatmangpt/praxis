/-
Label: prop:necessity
Kind: proposition

For $S$ a non-trivial semantic property, $Q_S(f)$ is undecidable; hence no
procedure decides $Q_S(f)$ for arbitrary $f$, and any mechanical trust in $f$
must be trust in some $\Omega$ evaluated on finite $X$, plus an argument
bounding residual risk on $D\setminus X$.

We do not have a computability library available (bare Lean core, no
mathlib), so the undecidability half of the statement cannot be formalized
directly as a statement about decision procedures. What *is* formalizable
and load-bearing from `def:oracle` is the consequence the proposition draws:
an `Oracle` (a decidable predicate `Ω` witnessing correctness only on a
finite `X`) does not, by itself, entail the full `CorrectnessQuestion` on
all of `D`. That is: trust restricted to `X` is strictly weaker than trust
on `D`, so an argument bounding residual risk on `D \ X` is *necessary* to
close the gap. We prove this as a genuine existence theorem: there is an
instance where a witnessing `Oracle` exists on `X` yet `CorrectnessQuestion`
fails on `D`.
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
meaning that `Ω x = true` witnesses `(x, f x) ∈ S`. -/
structure Oracle (S : Spec D R) (f : D → R) (X : List D) where
  Ω : D → Bool
  witnesses : ∀ x ∈ X, Ω x = true → S x (f x)

end Oracle

/-- Necessity: a witnessing oracle restricted to a finite `X` does not, in
general, entail the correctness question on all of `D`. Concretely, there is
an implementation `f`, a nontrivial spec `S` (the identity relation on
`Bool`), and a finite `X` strictly smaller than `D`, such that an `Oracle`
witnessing `f` on `X` exists, yet `CorrectnessQuestion S f` fails — some
`x ∈ D \ X` violates the spec. Hence mechanical trust confined to `Ω` on `X`
must be supplemented by a separate argument bounding residual risk on
`D \ X`. -/
theorem prop_necessity :
    ∃ (D R : Type) (S : Spec D R) (f : D → R) (X : List D),
      Nonempty (Oracle S f X) ∧ ¬ CorrectnessQuestion S f := by
  refine ⟨Bool, Bool, (fun x y => y = x), (fun _ => true), [true], ?_, ?_⟩
  · refine ⟨⟨fun _ => true, ?_⟩⟩
    intro x hx _
    -- the only element of X is `true`, and `f true = true = true`
    simp at hx
    subst hx
    rfl
  · -- CorrectnessQuestion fails at `x = false`, since `f false = true ≠ false`
    intro h
    have := h false
    simp at this
