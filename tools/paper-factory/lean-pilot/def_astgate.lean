/-
def:astgate

An AST gate is a syntactic predicate g : T → {0,1} on an abstract syntax
tree T, computable in O(|T|) time; the retrofit applier gates every
proposed change through an ordered battery (g_1,...,g_m): the change is
admitted iff all gates pass, ⋀_i g_i(T) = 1; a failing gate returns a
denial with the gate index and offending AST node.
-/

-- Abstract syntax tree, left abstract.
opaque AST : Type

-- An AST gate is a decidable syntactic predicate on ASTs.
def ASTGate := AST → Bool

-- The result of running a battery of gates over an AST: either every
-- gate passed, or the first failing gate is identified together with
-- the offending AST node.
inductive GateResult where
  | admitted : GateResult
  | denied : (index : Nat) → (node : AST) → GateResult

-- An ordered battery of AST gates (g_1, ..., g_m).
abbrev GateBattery := Array ASTGate

-- Run a battery of gates against an AST, admitting the change iff all
-- gates pass (⋀_i g_i(T) = 1), otherwise reporting the first failing
-- gate's index and the offending node.
def runBattery (battery : GateBattery) (t : AST) : GateResult :=
  let rec go (i : Nat) : GateResult :=
    if h : i < battery.size then
      if battery[i] t then
        go (i + 1)
      else
        GateResult.denied i t
    else
      GateResult.admitted
  termination_by battery.size - i
  go 0
