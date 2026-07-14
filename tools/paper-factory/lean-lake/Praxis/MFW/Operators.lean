import Mathlib.Order.FixedPoints
import Praxis.MFW.State
import Praxis.Corpus.def_saturation
import Praxis.Core

/-!
# Praxis/MFW/Operators.lean — the eight MFW operators

`C` (semantic closure/contraction), `ρ` (irreducible unresolved residue), `Π` (PDDL
planning over hook-realizable actions), `M` (POWL manufacture of plan `π`), `⊙_a`
(attachment of a manufactured child at socket `a`), `E` (broker-controlled execution),
`α` (admission of a returned observation), `K` (SPARQL CONSTRUCT capitalization).

Per the task directive: operators genuinely grounded in existing corpus machinery are
given real, computable `def`s; operators that are inherently external processes (an
actual PDDL planner, an actual POWL manufacture pipeline, an actual broker-controlled
OTP execution, admission of an actually-returned observation) are declared `axiom`,
following the `ax_*.lean` doc-comment convention: one paragraph naming the concrete
real-world system/process the axiom stands for.

## Real defs vs. disclosed axioms

* `C`, `ρ`, `Entails`, `Admissible`, `attach` (`⊙`), `K` — real `def`s, composed from
  `Praxis.Corpus.DefSaturation` (`def:saturation`, itself Mathlib's `OrderHom.lfp`),
  `Set`/`Finset` operations, and structural `POWL` recursion.
* `Π`, `M`, `E`, `α` — `axiom`s. Each stands for a genuinely external system with no
  in-Lean model, exactly the same posture this corpus already takes toward `chainH`
  (`def_contentaddr.lean`) and `Verify` (`ax_verify.lean`): postulating the existence
  and type of the external primitive, not faking its computational content.
-/

namespace Praxis.MFW

open Praxis.Corpus.DefDomain
open Praxis.Corpus.DefGround
open Praxis.Corpus.DefSaturation
open Praxis.Corpus

variable {D : LiftedDomain} {Ob : Type} [DecidableEq (GroundAtom D Ob)]

/-! ## `C` — semantic closure/contraction (real def) -/

/-- `C(G)`: the semantic closure of the admitted graph state `G` under a fixed
immediate-consequence operator `T` (reused from `Praxis.Corpus.DefSaturation`,
`def:saturation`), realized as `G`'s own currently-true atoms unioned with the
program's saturation `lfp(T)`. `G`'s facts act as a floor: closure never loses a fact
`G` already admits, and adds everything the consequence operator can further derive.
No new fixpoint machinery is introduced — `Saturation T = OrderHom.lfp T` is Mathlib's
pre-built least fixpoint, exactly as `def_saturation.lean` already established. -/
def C (T : ConsequenceOp D Ob) (G : GroundedState D Ob) : Set (GroundAtom D Ob) :=
  Saturation T ∪ (G.trueAtoms : Set (GroundAtom D Ob))

/-! ## `ρ` — irreducible unresolved residue (real def) -/

/-- `ρ(C(G), g)`: the irreducible unresolved residue — the goal atoms not (yet)
present in the closure — realized as plain `Set` difference. -/
def rho (closure : Set (GroundAtom D Ob)) (goal : MFWGoal D Ob) :
    Set (GroundAtom D Ob) :=
  (goal : Set (GroundAtom D Ob)) \ closure

@[inherit_doc rho] notation "ρ" => rho

/-! ## Entailment / admissibility (real defs, used by `Transition.lean`) -/

/-- `C(G) ⊨ g`: the closure already entails the goal, i.e. every goal atom is already
present in the closure. Equivalent to `ρ(closure, goal) = ∅`. -/
def Entails (closure : Set (GroundAtom D Ob)) (goal : MFWGoal D Ob) : Prop :=
  (goal : Set (GroundAtom D Ob)) ⊆ closure

/-- `Admissible(C(G), g)`: the goal is admissible *in principle* for this domain — it
lies within the domain's full reachable saturation `lfp(T)`, independent of whatever
`G` currently happens to contain. This is deliberately weaker than `Entails`: a goal
can be `Admissible` (reachable in principle) without yet being `Entails`-ed by the
current closure, which is exactly the gap `Φ`'s `Continue` branch exists to close, and
a goal that fails `Admissible` can never become true no matter how much further
planning/execution runs, which is exactly `Φ`'s `Refused` branch. -/
def Admissible (T : ConsequenceOp D Ob) (goal : MFWGoal D Ob) : Prop :=
  (goal : Set (GroundAtom D Ob)) ⊆ Saturation T

/-! ## `Π` — PDDL planning over hook-realizable actions (axiom) -/

/-- A plan `π`: a finite sequence of ground actions, the standard PDDL/STRIPS plan
shape already used by this corpus's `TemporalPlan`/`GroundProblem` machinery
(`con_tape.lean`, `con_strips.lean`). -/
abbrev MFWPlan (D : LiftedDomain) (Ob : Type) := List (GroundAction D Ob)

/-- `piPlan(G, g, H)` (notation for the source's `Π(G, g, H)`; `Π` is reserved by
Lean's dependent-function-type notation, so the executable name is `piPlan`): PDDL
planning over hook-realizable actions. Stands for a real
external PDDL/STRIPS planner (e.g. an off-the-shelf heuristic-search planner such as
Fast Downward, or this repo's own `cng` PDDL-TTL pipeline) that, given the admitted
graph state, the required goal, and the admitted hook/capability surface `H`
restricting which ground actions are realizable, searches for and returns a
finite plan. No Mathlib/Lean-core term models heuristic search over a
combinatorial action space; genuinely postulating the existence of such a planner
(its type, not its search behavior) is the correct level of abstraction, matching
this corpus's treatment of `chainH`/`Verify` as opaque external primitives. -/
axiom piPlan (G : GroundedState D Ob) (g : MFWGoal D Ob)
    (H : GateBattery (GroundAction D Ob)) : MFWPlan D Ob

/-! ## `M` — POWL manufacture of plan `π` (axiom) -/

/-- `M(π)`: POWL manufacture of plan `π`. Stands for the real external POWL
manufacture process (this corpus's `temporal_plan_to_powl_tape`-adjacent pipeline
operating on *concrete* durations/costs/scheduling metadata that this purely
definitional pass does not carry) that turns a bare action sequence into a
scheduled, receipted process-tree fragment ready for attachment. Declared as an
axiom rather than a trivial structural wrapper (e.g. `POWL.partialOrder`-over-
`activity`) because a genuine manufacture step is expected to consult scheduling,
cost, and hook-realizability metadata this definitional pass deliberately leaves
out of `MFWPlan`; faking that content here would misrepresent a real
manufacturing process as free structural recursion. -/
axiom M (π : MFWPlan D Ob) : POWL (GroundAction D Ob)

/-! ## `⊙` — attachment of a manufactured child at socket `a` (real def) -/

/-- `W ⊙_a M(π)`: attach a manufactured child POWL fragment to the current workflow
geometry `W` at socket `a`. `POWL`'s two composite constructors (`partialOrder`,
`choiceGraph`) both carry a `children : List (POWL A)` field addressed by list
position (per `def_powl.lean`'s own workaround for the kernel's nested-inductive
positivity restriction); attachment is realized as appending the manufactured child
to that list, preserving the existing ordering/edge relation. A bare `activity` leaf
has no children list to extend, so it is first promoted to a two-child
`partialOrder` (the child socket and the manufactured attachment run
concurrently — no ordering constraint is imposed, matching `def_powl`'s own
`Prop`-valued, caller-supplied `prec`/`edges` relations). The socket index `a` is
accepted for interface fidelity with `W ⊙_a M(π)` but, since `def:powl` fixes no
socket-addressing scheme beyond list position, attachment is realized as append
(insertion at the end of the children list) regardless of `a`'s value — a documented
simplification, not a hidden one. -/
def attach {A : Type u} (W : POWL A) (_a : Nat) (child : POWL A) : POWL A :=
  match W with
  | POWL.activity act => POWL.partialOrder [POWL.activity act, child] (fun _ _ => False)
  | POWL.partialOrder children prec => POWL.partialOrder (children ++ [child]) prec
  | POWL.choiceGraph children edges => POWL.choiceGraph (children ++ [child]) edges

@[inherit_doc attach] notation:65 W " ⊙[" a "] " child => attach W a child

/-! ## `E` — broker-controlled execution (axiom) -/

/-- `E(W, G)`: broker-controlled execution of the workflow geometry `W` against the
admitted graph state `G`. Stands for the real external, side-effecting execution of
`W` by this corpus's broker (an actual OTP/actor-supervised process dispatching
hook-realizable actions and returning whatever the outside world actually reports),
reusing `Praxis.Observation` (`Praxis/Core.lean`) as the raw returned observation
type. Genuinely outside Lean's computational reach: execution has real-world side
effects (network calls, process spawns, hardware interaction) that no pure Lean term
can stand in for without faking them. -/
axiom E (W : POWL (GroundAction D Ob)) (G : GroundedState D Ob) : Praxis.Observation

/-! ## `α` — admission of a returned observation (axiom) -/

/-- `α(O)`: admission of a returned observation. Stands for the real external
admission/parsing pipeline (this corpus's `def:adm`/`def:obsauth` admission
discipline applied to an *actual* returned observation, not a symbolic one) that
turns a raw `Praxis.Observation` into a vetted, finite set of newly-admitted ground
atoms fit to be capitalized into the graph state. Declared as an axiom because the
admission decision depends on real external verification (signatures, receipts,
authority checks) this definitional pass does not carry, matching the posture this
corpus already takes toward `Verify` (`ax_verify.lean`). -/
axiom α (O : Praxis.Observation) : Finset (GroundAtom D Ob)

/-! ## `K` — SPARQL CONSTRUCT capitalization (real def) -/

/-- `K(G)`: SPARQL-CONSTRUCT-style capitalization of newly-admitted atoms into the
graph state — structurally exactly what a `CONSTRUCT` query does (insert the
constructed triples into the target graph), realized here as `Finset` union on
`G.trueAtoms`. This corpus has no SPARQL engine to invoke, but the *structural*
content of capitalization — persisting newly-derived facts into the admitted state —
is fully computable and is given as a real def rather than an axiom. -/
def K (G : GroundedState D Ob) (newAtoms : Finset (GroundAtom D Ob)) :
    GroundedState D Ob :=
  { G with trueAtoms := G.trueAtoms ∪ newAtoms }

end Praxis.MFW
