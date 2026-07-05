/-
def:tax

The category set Catset is the eight-element enum Identity/Capacity/Topology/
Temporal/Lifecycle/Authorization/Prerequisites/Reserved; the scenario set Scnset
is the thirteen-variant RefusalScenario enum partitioned into three
obligation-driven, seven denial-lane, and three logic/andon variants; the
category map catop : Scnset -> Catset is RefusalScenario::category.
-/

/-- `Catset`: the eight-element category enum. -/
inductive Catset where
  | Identity
  | Capacity
  | Topology
  | Temporal
  | Lifecycle
  | Authorization
  | Prerequisites
  | Reserved
deriving DecidableEq, Repr

/-- `Scnset`: the thirteen-variant `RefusalScenario` enum, partitioned into
three obligation-driven variants, seven denial-lane variants, and three
logic/andon variants. -/
inductive Scnset where
  -- three obligation-driven variants
  | schemaObligation
  | policyObligation
  | signatureObligation
  -- seven denial-lane variants
  | denialA
  | denialB
  | denialC
  | denialD
  | denialE
  | denialF
  | denialG
  -- three logic/andon variants
  | logicAndon1
  | logicAndon2
  | logicAndon3
deriving DecidableEq, Repr

/-- `catop`, i.e. `RefusalScenario::category`: the category map sending each
scenario variant to its owning category. -/
def catop : Scnset → Catset
  | .schemaObligation => .Identity
  | .policyObligation => .Authorization
  | .signatureObligation => .Authorization
  | .denialA => .Identity
  | .denialB => .Capacity
  | .denialC => .Topology
  | .denialD => .Temporal
  | .denialE => .Lifecycle
  | .denialF => .Authorization
  | .denialG => .Prerequisites
  | .logicAndon1 => .Reserved
  | .logicAndon2 => .Reserved
  | .logicAndon3 => .Reserved
