import Praxis.Corpus.def_denialcode
import Praxis.Corpus.prop_obtotal

/-!
# def:tax

The category set `Catset` is the eight-element enum
Identity/Capacity/Topology/Temporal/Lifecycle/Authorization/Prerequisites/Reserved;
the scenario set `Scnset` is the thirteen-variant `RefusalScenario` enum partitioned into
three obligation-driven, seven denial-lane, and three logic/andon variants; the category map
`catop : Scnset → Catset` is `RefusalScenario::category`.

Both `Category` and `RefusalScenario` are small, closed, finite tag sets with no numeric or
algebraic content -- exactly what Lean's `inductive` enums already are (with `DecidableEq`
and `Repr` derived automatically, mirroring Rust's `#[derive(PartialEq, Debug)]` enums), so
there is no Mathlib type to compose here beyond the inductive-enum machinery itself.
-/

/-- The eight-element category set `Catset`. -/
inductive Category where
  | Identity
  | Capacity
  | Topology
  | Temporal
  | Lifecycle
  | Authorization
  | Prerequisites
  | Reserved
deriving DecidableEq, Repr

/-- The thirteen-variant scenario set `Scnset`: three obligation-driven variants (tagging the
`ObligationKind` cases from `def:ob`/`prop:obtotal`), seven denial-lane variants (one per
nonzero `DenialPolarity` lane from `def:denialcode`), and three logic/andon variants. -/
inductive RefusalScenario where
  -- three obligation-driven variants
  | schemaViolation
  | policyViolation
  | temporalViolation
  -- seven denial-lane variants
  | lane1Denial
  | lane2Denial
  | lane3Denial
  | lane4Denial
  | lane5Denial
  | lane6Denial
  | lane7Denial
  -- three logic/andon variants
  | logicContradiction
  | andonPull
  | andonEscalation
deriving DecidableEq, Repr

namespace RefusalScenario

/-- The category map `catop : Scnset → Catset`, i.e. `RefusalScenario::category`. -/
def category : RefusalScenario → Category
  | schemaViolation => Category.Prerequisites
  | policyViolation => Category.Authorization
  | temporalViolation => Category.Temporal
  | lane1Denial => Category.Identity
  | lane2Denial => Category.Identity
  | lane3Denial => Category.Capacity
  | lane4Denial => Category.Capacity
  | lane5Denial => Category.Topology
  | lane6Denial => Category.Topology
  | lane7Denial => Category.Lifecycle
  | logicContradiction => Category.Reserved
  | andonPull => Category.Reserved
  | andonEscalation => Category.Reserved

end RefusalScenario
