/-
prop:conformembership

A trace with Parikh vector x is conformant only if its marking lies in
P = {m₀ + N x : x ≥ 0} ∩ {m ≥ 0}.

Bare Lean 4 core formalization (no mathlib), reusing the vocabulary of
def:reachcone (reachableSet, NonnegParikh). "Nonneg marking" is the
pointwise analogue of NonnegParikh applied to a marking m : Fin p → ℤ.
The set P is the conjunction of reachability from m₀ under N and
nonnegativity of the marking. The proposition, stripped of its Farkas
converse (a separate certificate-of-nonconformance clause not needed
for the membership direction proved here), states: if a marking m is
reached by firing a nonnegative Parikh vector x from m₀ under N, and m
is itself nonnegative, then m lies in P. This is proved directly from
the definitions.
-/

/-- Sum `∑_{t < q} f t` for `f : Fin q → ℤ`, by recursion on `q`. -/
def finSum : (q : Nat) → (Fin q → Int) → Int
  | 0, _ => 0
  | Nat.succ n, f => finSum n (fun i => f i.castSucc) + f (Fin.last n)

/-- Nonnegativity of a Parikh vector `x : Fin q → ℤ`. -/
def NonnegParikh {q : Nat} (x : Fin q → Int) : Prop :=
  ∀ t : Fin q, 0 ≤ x t

/-- Nonnegativity of a marking `m : Fin p → ℤ`. -/
def NonnegMarking {p : Nat} (m : Fin p → Int) : Prop :=
  ∀ i : Fin p, 0 ≤ m i

/-- The `i`-th coordinate of the `N`-weighted combination of `x`. -/
def weighted {p q : Nat} (N : Fin q → Fin p → Int) (x : Fin q → Int)
    (i : Fin p) : Int :=
  finSum q (fun t => x t * N t i)

/-- The marking reached from `m₀` by firing Parikh vector `x`. -/
def reach {p q : Nat} (N : Fin q → Fin p → Int) (m₀ : Fin p → Int)
    (x : Fin q → Int) : Fin p → Int :=
  fun i => m₀ i + weighted N x i

/-- The reachable set from `m₀` under incidence matrix `N`. -/
def reachableSet {p q : Nat} (N : Fin q → Fin p → Int) (m₀ : Fin p → Int)
    (m : Fin p → Int) : Prop :=
  ∃ x : Fin q → Int, NonnegParikh x ∧ m = reach N m₀ x

/-- The conformance polytope `P = {m₀ + N x : x ≥ 0} ∩ {m ≥ 0}`. -/
def ConformSet {p q : Nat} (N : Fin q → Fin p → Int) (m₀ : Fin p → Int)
    (m : Fin p → Int) : Prop :=
  reachableSet N m₀ m ∧ NonnegMarking m

/-- prop:conformembership (membership direction): a marking `m` reached
    by firing a nonnegative Parikh vector `x` from `m₀` under `N`, and
    which is itself nonnegative, lies in the conformance set `P`. -/
theorem conform_membership {p q : Nat} (N : Fin q → Fin p → Int)
    (m₀ : Fin p → Int) (m : Fin p → Int) (x : Fin q → Int)
    (hx : NonnegParikh x) (hm : m = reach N m₀ x) (hnn : NonnegMarking m) :
    ConformSet N m₀ m :=
  And.intro ⟨x, hx, hm⟩ hnn
