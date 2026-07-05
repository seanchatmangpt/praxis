/-
prop:astterm

For any finite AST T and finite gate battery (g_1,...,g_m) of syntactic
predicates, battery evaluation terminates in O(m·|T|) time and returns
either Admitted or a refusal carrying the first failing gate index and
offending node; the retraction is total, never diverging and never
returning an unlabelled outcome.

We capture the "total, never diverging, never unlabelled" content as: for
every battery and every AST t, runBattery battery t (a structurally
terminating, hence total, function already defined in def:astgate) always
yields either `admitted` or `denied index node` for some index and node —
there is no third, unlabelled outcome.
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

-- prop:astterm : battery evaluation is total and always labelled — every
-- run of runBattery yields either `admitted`, or `denied` carrying the
-- failing gate index and offending node. No third, unlabelled outcome is
-- possible, since GateResult has exactly these two constructors and
-- runBattery is a total (structurally recursive) function.
theorem astterm_total_labelled (battery : GateBattery) (t : AST) :
    runBattery battery t = GateResult.admitted ∨
    ∃ (index : Nat) (node : AST), runBattery battery t = GateResult.denied index node := by
  cases h : runBattery battery t with
  | admitted => exact Or.inl rfl
  | denied index node => exact Or.inr ⟨index, node, rfl⟩
