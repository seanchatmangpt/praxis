import Praxis.Corpus.def_ground
import Praxis.Corpus.def_powl
import Praxis.Corpus.def_astgate
import Praxis.Core

/-!
# Praxis/MFW/State.lean — the Multifractal Workflow configuration object

`X = (G, W, g, H, R)`:

* `G` — admitted graph state
* `W` — current recursive workflow geometry
* `g` — required continuation goal
* `H` — admitted capability/hook surface
* `R` — current receipt chain

This file is purely additive vocabulary (definitions only, no theorems, no `sorry`),
grounded in existing `Praxis/Corpus` types wherever a genuine fit exists, following the
mandatory-composition discipline used throughout `Praxis/Corpus` (e.g. `def_chain.lean`,
`def_ground.lean`): reuse a pre-built corpus type rather than declare a fresh opaque one
whenever the shapes line up.

## Field-by-field grounding

* `G : GroundedState D Ob` — reused verbatim from `Praxis.Corpus.DefGround`
  (`def:ground`, `Praxis/Corpus/def_ground.lean`). A `GroundedState` is exactly a
  finite set of true ground atoms plus a fluent valuation, i.e. an admitted set of
  "graph facts" together with numeric state — the natural realization of "admitted
  graph state" already present in this corpus.
* `W : POWL (GroundAction D Ob)` — reused verbatim from `Praxis.Corpus.def_powl`
  (`def:powl`). `POWL` is literally this corpus's recursive workflow-geometry type
  (activity / partial-order / choice-graph process tree), so `W` is instantiated over
  ground actions with no new type needed.
* `g : MFWGoal D Ob` — **new**. No corpus type carries a PDDL-style goal: `def_ground`'s
  `Problem` bundles an object universe, an initial grounded state, and a discount
  factor, but has no goal field, and no other corpus file defines one. `MFWGoal` is
  introduced here as the minimal honest fit: a finite set of `GroundAtom`s that must
  hold, built entirely from the already-reused `GroundAtom D Ob` type, not a fresh
  opaque structure.
* `H : GateBattery (GroundAction D Ob)` — reused verbatim from `Praxis.Corpus`
  (`def:astgate`, `Praxis/Corpus/def_astgate.lean`). A `GateBattery` is exactly an
  ordered battery of admission predicates over an abstract type `T`; instantiated at
  `T := GroundAction D Ob` it is precisely "the admitted capability/hook surface": the
  battery of gates a candidate hook-realizable action must pass before it is
  admissible.
* `R : List Praxis.Receipt` — reuses `Praxis.Receipt` (`Praxis/Core.lean`'s "Receipt
  marker", already wired into the top-level `Praxis` import closure) rather than
  `Praxis.Corpus.def_chain`'s `Ledger`/`Frame`. `def_chain.lean`'s `Ledger` would have
  been the more literal fit (`def:chain`'s own receipt ledger), but it transitively
  imports `Praxis.Corpus.def_frame`'s global, unqualified `structure Frame`, which
  collides with `Praxis.Mathlib.DefReceipt`'s own global, unqualified `structure
  Frame` the moment both are pulled into one root import graph (`Praxis.lean` already
  imports `Praxis.Mathlib.DefReceipt`) — a pre-existing naming collision between two
  corpus files this purely-additive pass must not touch. `Praxis.Receipt` is the
  genuine, already-top-level-wired stand-in for "the current receipt chain" that
  avoids re-triggering that collision.
-/

namespace Praxis.MFW

open Praxis.Corpus.DefDomain
open Praxis.Corpus.DefGround
open Praxis.Corpus

variable (D : LiftedDomain) (Ob : Type)

/-- **New** — no corpus `Problem`/goal type exists. An MFW goal `g` is a finite
requirement over the same `GroundAtom D Ob` vocabulary `def_ground` already uses for
grounded state, rather than an invented bespoke goal syntax: the goal is "these atoms
must hold", exactly the shape PDDL/planning goals take over a grounded domain. -/
abbrev MFWGoal := Finset (GroundAtom D Ob)

/-- The Multifractal Workflow configuration object `X = (G, W, g, H, R)`. Every field
except `g` reuses an existing `Praxis/Corpus` type outright; see the module doc comment
above for the per-field justification. -/
structure MFWConfig [DecidableEq (GroundAtom D Ob)] where
  /-- `G` — admitted graph state; reused from `Praxis.Corpus.DefGround.GroundedState`
  (`def:ground`). -/
  G : GroundedState D Ob
  /-- `W` — current recursive workflow geometry; reused from `Praxis.Corpus.POWL`
  (`def:powl`), instantiated over ground actions. -/
  W : POWL (GroundAction D Ob)
  /-- `g` — required continuation goal; **new**, built from the reused `GroundAtom`
  type (see `MFWGoal` above). -/
  g : MFWGoal D Ob
  /-- `H` — admitted capability/hook surface; reused from `Praxis.Corpus.GateBattery`
  (`def:astgate`), instantiated over ground actions. -/
  H : GateBattery (GroundAction D Ob)
  /-- `R` — current receipt chain; reused from `Praxis.Receipt`
  (`Praxis/Core.lean`), i.e. `List Praxis.Receipt`. See the module doc comment
  above for why `Praxis.Corpus.def_chain.Ledger` was not used instead. -/
  R : List Praxis.Receipt

end Praxis.MFW
