/-
def:saturation — Let (D, Ob) be a grounded PDDL problem with initial atom set
m0 and ground action set T. The Nemo Datalog program Pi has one fact per atom
in m0 and one rule per ground action; the saturation sat(Pi) is the least
fixpoint lfp(T_Pi) of the immediate-consequence operator; an atom p is
reachable iff p in sat(Pi).

We model this abstractly in bare Lean 4 core (no mathlib), reusing the
`GroundAction` vocabulary from def:ground. Ground atoms are left as an
opaque payload type `Atom`. The immediate-consequence operator `Tstep` maps
a current atom set to the atoms derivable by firing one ground action (via
an abstract `fires` predicate telling us which atom a ground action
produces from a given atom set) union the initial facts. Since we have no
ambient lattice/fixpoint theory (no mathlib), we define the saturation
constructively as the result of iterating `Tstep` a given number of times
from the empty set — `sat` at iteration count `n` — and package "the"
saturation as the state reached after iterating over all ground actions
once per action (a finite, computable stand-in for the least fixpoint,
since `T` is itself finite). Reachability of an atom `p` is then defined
as membership of `p` in this iterated set.
-/

/-- One step of the immediate-consequence operator: given the current atom
    set `m`, the initial facts `m0`, the ground actions `T`, and an abstract
    firing relation `fires` (which ground action derives which atom from a
    given atom set), return the next atom set: the initial facts together
    with every atom derivable by firing some ground action against `m`. -/
def Tstep
    (Atom GroundAction : Type) [DecidableEq Atom]
    (m0 : List Atom) (T : List GroundAction)
    (fires : GroundAction → List Atom → Option Atom)
    (m : List Atom) : List Atom :=
  (m0 ++ (T.filterMap (fun a => fires a m))).eraseDups

/-- Iterate the immediate-consequence operator `n` times starting from the
    empty atom set, producing the atom set reachable after `n` rounds. -/
def iterate
    (Atom GroundAction : Type) [DecidableEq Atom]
    (m0 : List Atom) (T : List GroundAction)
    (fires : GroundAction → List Atom → Option Atom)
    (n : Nat) : List Atom :=
  match n with
  | 0 => []
  | k + 1 => Tstep Atom GroundAction m0 T fires (iterate Atom GroundAction m0 T fires k)

/-- The saturation sat(Pi): since `T` is finite (a `List`), `Tstep` reaches
    its fixpoint after at most `T.length + 1` rounds (each round either adds
    at least one new atom derivable from a not-yet-fired action, or the
    process has already stabilized); we take that many iterations as the
    saturation, standing in for `lfp(T_Pi)`. -/
def saturation
    (Atom GroundAction : Type) [DecidableEq Atom]
    (m0 : List Atom) (T : List GroundAction)
    (fires : GroundAction → List Atom → Option Atom) : List Atom :=
  iterate Atom GroundAction m0 T fires (T.length + 1)

/-- An atom `p` is reachable iff it lies in the saturation `sat(Pi)`. -/
def reachable
    (Atom GroundAction : Type) [DecidableEq Atom]
    (m0 : List Atom) (T : List GroundAction)
    (fires : GroundAction → List Atom → Option Atom)
    (p : Atom) : Prop :=
  p ∈ saturation Atom GroundAction m0 T fires
