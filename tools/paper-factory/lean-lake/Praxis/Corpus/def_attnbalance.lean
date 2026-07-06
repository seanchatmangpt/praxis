import Praxis.Corpus.def_adm
import Mathlib

/-!
Label: def:attnbalance

"Let $c(t)\ge0$ be capacity and admitted action $j$ draw rate $r_j\ge0$ on
$[s_j,e_j]$. Free capacity is $f(t)=c(t)-\sum_{j:\,t\in[s_j,e_j]}r_j$. A
schedule is feasible iff $f(t)\ge0$ for all $t$."

`t` ranges over `ℝ` (an arbitrary time parameter). Capacity `c : ℝ → ℝ` and
each action's draw rate `r : ι → ℝ` are plain real-valued functions -- the
nonnegativity side conditions ("$c(t)\ge0$", "$r_j\ge0$") are carried as
explicit hypotheses on the constructor rather than baked into the type, since
the statement's substance is the balance equation and the feasibility
predicate, not a refinement type for nonnegative reals (Mathlib's `NNReal`
would obscure the arithmetic here for no gain). The finite index set of
admitted actions `j` is `ι` with a `Fintype` instance -- exactly the "finite
battery" pattern already used for the obligation list in `def:adm`, here
applied to the finite collection of currently-admitted actions rather than
obligations. Each action's active window $[s_j,e_j]$ is realized as
`Set.Icc (s j) (e j)`, Mathlib's closed-interval, and the summation
"$\sum_{j:\,t\in[s_j,e_j]} r_j$" is `Finset.sum` over the sub-`Finset` of
`Finset.univ` selected by the decidable membership predicate
`t ∈ Set.Icc (s j) (e j)` (decidable since `ℝ` has a linear order).

No axioms: `f` and `feasible` are plain data/prop-level definitions composed
from `Set.Icc`, `Finset.filter`, `Finset.sum`, and real number arithmetic/order,
all already in Mathlib.
-/

open Finset

variable {ι : Type*} [Fintype ι] [DecidableEq ι]

/-- `def:attnbalance`: free capacity at time `t`, given total capacity `c`,
per-action draw rates `r`, and each admitted action `j`'s active window
`[s j, e j]`. Subtracts the draw rates of exactly the actions currently active
at `t` from the raw capacity `c t`. -/
noncomputable def freeCapacity (c : ℝ → ℝ) (r : ι → ℝ) (s e : ι → ℝ) (t : ℝ) : ℝ :=
  c t - ∑ j in univ.filter (fun j => t ∈ Set.Icc (s j) (e j)), r j

/-- `def:attnbalance`: a schedule (capacity `c`, draw rates `r`, windows
`s`/`e`) is feasible iff the free capacity `freeCapacity c r s e t` is
nonnegative at every time `t`. -/
def scheduleFeasible (c : ℝ → ℝ) (r : ι → ℝ) (s e : ι → ℝ) : Prop :=
  ∀ t : ℝ, freeCapacity c r s e t ≥ 0
