import Praxis.MFW.Operators

/-!
# Praxis/MFW/Transition.lean — the MFW transition `Φ`

```
Φ(G,W,g,H,R) = (G,W,CLOSED,H,R)                                          if C(G) ⊨ g
             = REFUSE                                                     if ¬Admissible(C(G),g)
             = (K(G'), W ⊙_a M(Π(C(G),g,H)), g, H, R')                    otherwise
```

`MFWResult` is a genuine closed three-way outcome tag, modeled after the corpus's own
tri- or n-state result pattern (`Praxis.Corpus.DefQueryResult.QueryResult`'s
`Answered`/`Denied`/`Invalid`, and `Disposition`'s four-way
`Completed`/`Parked`/`SkippedBy`/`GaveUp` in `prop_totalaccounting.lean`): a plain
closed `inductive`, no Mathlib composition needed beyond what `inductive` already
gives natively.

`Φ` itself is a genuine `def`: case analysis over `Entails`/`Admissible` (both real
defs from `Operators.lean`) that, in the `Continue` branch, *calls* the axiomatized
external operators (`piPlan`, `M`, `E`, `α`) but does no reasoning about their
results — it is pure structural dispatch, so it remains a `def`, not an axiom, even
though some of the values it composes are axiomatized.
-/

namespace Praxis.MFW

open Praxis.Corpus.DefDomain
open Praxis.Corpus.DefGround
open Praxis.Corpus.DefSaturation
open Praxis.Corpus

variable {D : LiftedDomain} {Ob : Type} [DecidableEq (GroundAtom D Ob)]

/-- The three-way MFW transition outcome, modeled after the corpus's existing
tri- or n-state result pattern (`QueryResult`, `Disposition`). -/
inductive MFWResult : Type where
  /-- The closure already entails the goal: the configuration is done. -/
  | Closed
  /-- The goal is not admissible for this domain at all (its residue can never be
  resolved), so the configuration is refused outright. -/
  | Refused
  /-- Neither closed nor refused: one more plan/manufacture/attach/execute/admit/
  capitalize round is required. -/
  | Continue
  deriving DecidableEq, Repr

open Classical in
/-- `Φ(G,W,g,H,R)`: the MFW transition. Given the fixed immediate-consequence
operator `T` used to compute closures (`Praxis.Corpus.DefSaturation.ConsequenceOp`,
threaded through exactly as `C`/`Admissible` already require it), and a socket `a` at
which manufactured children are attached, `Φ` dispatches:

* `Closed`, unchanged configuration, when `C(G) ⊨ g` (`Entails`);
* `Refused`, unchanged configuration, when the goal is not even `Admissible` for this
  domain (its residue is structurally unreachable, independent of how much further
  planning/execution runs);
* otherwise `Continue`, with a new configuration built by: planning `π = piPlan (C(G))
  g H` over the closed graph state; manufacturing `M π`; attaching it to `W` at socket
  `a` (`⊙`); executing the resulting workflow against `G` (`E`); admitting the
  returned observation (`α`); and capitalizing the newly-admitted atoms into `G` via
  `K`. `R'` — the receipt-chain update from executing and admitting this round — is
  left as a caller-supplied parameter (`newFrames`), since `Praxis.Receipt`
  construction from an `α`-admitted observation is itself an external receipting
  step this purely definitional pass does not further decompose.

  Non-constructive: `Entails`/`Admissible` are plain `Set`-membership `Prop`s with no
  general `Decidable` instance, so the case split below uses classical (excluded
  middle) decidability, matching how other non-constructive `Set`-level predicates in
  this corpus (e.g. `Praxis.Corpus.DefLogicAdm`'s abstract satisfaction predicates)
  are already treated. -/
noncomputable def Φ (T : ConsequenceOp D Ob) (a : Nat) (X : MFWConfig D Ob)
    (newFrames : List Praxis.Receipt) : MFWConfig D Ob × MFWResult :=
  let closure := C T X.G
  if Entails closure X.g then
    (X, MFWResult.Closed)
  else if ¬ Admissible T X.g then
    (X, MFWResult.Refused)
  else
    let π := piPlan X.G X.g X.H
    let manufactured := M π
    let W' := attach X.W a manufactured
    let observed := E W' X.G
    let admitted := α (D := D) (Ob := Ob) observed
    let G' := K X.G admitted
    ({ X with G := G', W := W', R := List.append X.R newFrames }, MFWResult.Continue)

end Praxis.MFW
