/-
  ax:cr — Collision resistance of chainH (BLAKE3).

  Statement: chainH is collision-resistant: no PPT adversary finds x ≠ y with
  chainH x = chainH y except with negligible probability ε(λ), λ = 256,
  birthday bound ~ 2^128.

  We model this abstractly in bare Lean 4 (no mathlib):
  - `Bytes` : the domain/codomain of the hash (abstract carrier).
  - `chainH` : the hash function itself.
  - `SecurityParam` : the security parameter λ (fixed at 256 here).
  - `PPTAdversary` : an abstract type of adversary strategies (a PPT adversary
    is any inhabitant of this type — the polynomial-time restriction is a
    computational-complexity notion outside bare Lean's scope, so it is
    folded into the type itself as an assumption).
  - `AdvSucceeds A x y` : adversary `A` outputs the pair `(x, y)` as its
    attempted collision.
  - `negligible` : an abstract predicate on `SecurityParam → ℝ`-like bound
    functions, standing in for "negligible in λ" (again abstracted since
    bare Lean has no analysis library to define limits/asymptotics).
  - `AdvSuccessProb` : the success probability of an adversary at a given
    security parameter, abstracted as a value in a totally ordered carrier
    `Prob` bounded above by `one`.

  The axiom asserts: for every PPT adversary `A`, the probability that `A`
  finds a nontrivial collision (x ≠ y, chainH x = chainH y) at security
  parameter λ = 256 is bounded by some negligible function ε.
-/

axiom Bytes : Type

axiom chainH : Bytes → Bytes

/-- The security parameter λ. -/
abbrev SecurityParam : Type := Nat

/-- Fixed instantiation λ = 256, as named in the statement. -/
def lambda256 : SecurityParam := 256

/-- Abstract carrier for probabilities / negligible bounds. -/
axiom Prob : Type
axiom Prob.le : Prob → Prob → Prop
axiom Prob.one : Prob

/-- A PPT adversary is an abstract strategy that, given the security
    parameter, attempts to output a colliding pair. -/
axiom PPTAdversary : Type

/-- `AdvOutput A λ` : the pair `(x, y)` that adversary `A` outputs when run
    at security parameter `λ`. -/
axiom AdvOutput : PPTAdversary → SecurityParam → Bytes × Bytes

/-- `AdvSuccessProb A λ` : the probability, over the adversary's random
    coins, that `AdvOutput A λ` is a nontrivial collision of `chainH`. -/
axiom AdvSuccessProb : PPTAdversary → SecurityParam → Prob

/-- `negligible ε` : `ε` (a function of the security parameter) is
    negligible in `λ`, standing in for the birthday-bound asymptotic
    `~ 2^{-128}` at `λ = 256`. -/
axiom negligible : (SecurityParam → Prob) → Prop

/-- A collision witnessed by an adversary output: the two components differ
    yet hash to the same value under `chainH`. -/
def IsCollision (p : Bytes × Bytes) : Prop :=
  p.1 ≠ p.2 ∧ chainH p.1 = chainH p.2

/-- `ax:cr` — `chainH` (BLAKE3) is collision-resistant: there exists a
    negligible bound `ε` such that every PPT adversary `A`, run at security
    parameter `λ = 256`, succeeds in producing a nontrivial collision with
    probability at most `ε λ`. -/
axiom chainH_collision_resistant :
  ∃ ε : SecurityParam → Prob,
    negligible ε ∧
    ∀ A : PPTAdversary,
      IsCollision (AdvOutput A lambda256) →
      Prob.le (AdvSuccessProb A lambda256) (ε lambda256)
