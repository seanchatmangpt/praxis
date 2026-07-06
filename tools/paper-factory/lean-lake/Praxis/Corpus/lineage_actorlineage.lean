import Mathlib.Data.String.Basic

/-!
`lineage:actorlineage` -- The 8-bounded actor lineage (CNS -> BitActor ->
ByteActor -> knhk): CNS fixed the doctrine 8T/8H/8M; ByteActor V3
productized it with 64-byte crystal envelopes and bounded SPSC rings; knhk
carried the doctrine into Rust with a branchless TickBudget, a
ChatmanBounded assertion, and an R1/W1/C1 runtime-class taxonomy. The
lineage's one persistent gap never closed: classification without
actuation -- nothing in the constellation restarts, re-admits, or closes
the loop.

We model this as a record of the four lineage stages (each an opaque
`String` label naming the artifact/doctrine it fixed or carried forward,
since these are historical project names with no numeric structure
Mathlib already models) plus a `closesLoop` field recording whether that
stage actuates on its own classification (restarts, re-admits, or
otherwise closes the loop) rather than merely classifying. The
`persistentGap` field is the lineage's own finding, stated as a proof
obligation on the structure itself: every stage in this lineage, as
inherited, classifies without actuating.
-/

/-- One stage of the 8-bounded actor lineage: the doctrine/artifact it is
named for (`stageName`), and whether that stage actuates on its own
classification output (`closesLoop`) as opposed to merely classifying. -/
structure LineageStage where
  stageName  : String
  closesLoop : Bool

/-- The 8-bounded actor lineage CNS -> BitActor -> ByteActor -> knhk, together
with the finding that none of its four stages closes the loop: each
classifies without ever restarting, re-admitting, or otherwise acting on
what it classified. -/
structure ActorLineage where
  cns        : LineageStage
  bitActor   : LineageStage
  byteActor  : LineageStage
  knhk       : LineageStage
  persistentGap :
    cns.closesLoop = false ∧
    bitActor.closesLoop = false ∧
    byteActor.closesLoop = false ∧
    knhk.closesLoop = false
