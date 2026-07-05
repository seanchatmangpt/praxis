/-
Label: lineage:actorlineage
Kind: lineage

The 8-bounded actor lineage (CNS -> BitActor -> ByteActor -> knhk): CNS fixed the
doctrine 8T/8H/8M; ByteActor V3 productized it with 64-byte crystal envelopes and
bounded SPSC rings; knhk carried the doctrine into Rust with a branchless
TickBudget, a ChatmanBounded assertion, and an R1/W1/C1 runtime-class taxonomy.
The lineage's one persistent gap never closed: classification without actuation --
nothing in the constellation restarts, re-admits, or closes the loop.

This is a documentary/lineage statement, not a theorem: we encode it as a
structured record of the stages in the lineage and the standing gap, so it
type-checks as data rather than carrying a proof obligation.
-/

/-- A single stage in the 8-bounded actor lineage, named and given a one-line
    characterization of what doctrine/artifact it fixed or carried forward. -/
structure LineageStage where
  name : String
  contribution : String

/-- The persistent, never-closed gap in the lineage: classification without
    actuation. Nothing in the constellation restarts, re-admits, or closes the
    loop. -/
structure PersistentGap where
  description : String

/-- The full 8-bounded actor lineage: CNS -> BitActor -> ByteActor -> knhk,
    together with the one gap that persists across every stage. -/
structure ActorLineage where
  stages : List LineageStage
  gap : PersistentGap

/-- The concrete instance of the lineage described by this statement. -/
def actorLineage : ActorLineage :=
  { stages :=
      [ { name := "CNS"
        , contribution := "fixed the doctrine 8T/8H/8M" }
      , { name := "BitActor"
        , contribution := "intermediate stage carrying the 8T/8H/8M doctrine forward" }
      , { name := "ByteActor V3"
        , contribution :=
            "productized the doctrine with 64-byte crystal envelopes and bounded SPSC rings" }
      , { name := "knhk"
        , contribution :=
            "carried the doctrine into Rust with a branchless TickBudget, a " ++
            "ChatmanBounded assertion, and an R1/W1/C1 runtime-class taxonomy" }
      ]
  , gap :=
      { description :=
          "classification without actuation -- nothing in the constellation " ++
          "restarts, re-admits, or closes the loop" }
  }

#check actorLineage
