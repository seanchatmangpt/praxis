/-
def:mrr

Let `A` be a set of client accounts; for each `a ∈ A` let `lawful_targets(a)` be
the evidence-gated target stages and `realized(a,s)` the revenue function under
stage `s`; the Maximum Reachable Revenue is
`MRR = max_{plan} Σ_{a ∈ A} realized(a)`.

We model:
- `A` as an abstract type of accounts, given as a `List A` (finite set of
  accounts under consideration).
- `Stage` as an abstract type of target stages.
- `realized : A → Stage → Nat` as the revenue function under a given stage.
- a `plan` as a choice of stage per account, i.e. a function `A → Stage`
  drawn from a finite list of candidate plans (the evidence-gated
  `lawful_targets` restriction is captured by restricting to this list of
  admissible plans).

`sumRevenue accounts realized plan` sums the realized revenue of a single
plan over all accounts, and `MRR` is the maximum of that sum over all
admissible plans.
-/

def sumRevenue {A Stage : Type} (accounts : List A) (realized : A → Stage → Nat)
    (plan : A → Stage) : Nat :=
  (accounts.map (fun a => realized a (plan a))).foldr (· + ·) 0

def MRR {A Stage : Type} (accounts : List A) (realized : A → Stage → Nat)
    (plans : List (A → Stage)) : Nat :=
  (plans.map (sumRevenue accounts realized)).foldr max 0
