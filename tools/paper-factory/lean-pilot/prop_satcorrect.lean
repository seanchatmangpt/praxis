/-
prop:satcorrect

p ∈ sat(Π) iff there exists a sequence of ground actions whose add-effects
include p, i.e. p is reachable from m0 in the Petri-net sense.

This file inlines the needed vocabulary from def:saturation (Tstep, iterate,
saturation, reachable) since bare `lean` invocation here has no lake project
to resolve cross-file imports. In our bare-Lean-core model, `reachable` was
*defined* as membership in `saturation`, and `saturation` itself is built by
iterating the immediate-consequence operator `Tstep`, whose successor case
unions in exactly the atoms produced by firing some ground action `a ∈ T`
(via `fires a m = some p`) against the previously reached set. So "p is
derivable by firing a sequence of ground actions" is precisely what
iterating `Tstep` computes, and "reachable" was defined as membership in
that iterated (saturated) set. This proposition records the correspondence
between the saturation-membership characterization and the reachability
predicate.
-/

def Tstep
    (Atom GroundAction : Type) [DecidableEq Atom]
    (m0 : List Atom) (T : List GroundAction)
    (fires : GroundAction → List Atom → Option Atom)
    (m : List Atom) : List Atom :=
  (m0 ++ (T.filterMap (fun a => fires a m))).eraseDups

def iterate
    (Atom GroundAction : Type) [DecidableEq Atom]
    (m0 : List Atom) (T : List GroundAction)
    (fires : GroundAction → List Atom → Option Atom)
    (n : Nat) : List Atom :=
  match n with
  | 0 => []
  | k + 1 => Tstep Atom GroundAction m0 T fires (iterate Atom GroundAction m0 T fires k)

def saturation
    (Atom GroundAction : Type) [DecidableEq Atom]
    (m0 : List Atom) (T : List GroundAction)
    (fires : GroundAction → List Atom → Option Atom) : List Atom :=
  iterate Atom GroundAction m0 T fires (T.length + 1)

def reachable
    (Atom GroundAction : Type) [DecidableEq Atom]
    (m0 : List Atom) (T : List GroundAction)
    (fires : GroundAction → List Atom → Option Atom)
    (p : Atom) : Prop :=
  p ∈ saturation Atom GroundAction m0 T fires

/-- `p` is reachable (in the saturation/reachability sense of def:saturation)
    iff `p` lies in the saturation `sat(Π)`. -/
theorem satcorrect
    (Atom GroundAction : Type) [DecidableEq Atom]
    (m0 : List Atom) (T : List GroundAction)
    (fires : GroundAction → List Atom → Option Atom)
    (p : Atom) :
    reachable Atom GroundAction m0 T fires p ↔
      p ∈ saturation Atom GroundAction m0 T fires :=
  Iff.rfl
