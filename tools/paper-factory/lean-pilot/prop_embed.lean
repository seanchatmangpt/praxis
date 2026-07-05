/-
prop:embed

The refusing branches of thm:trichotomy embed into $(\Deny,\text{compose},\Adml)$
via the taxonomy: run_kernel_query maps Denied to scenario KernelDenied
(category Authorization) and Invalid to KernelInvalid (category Identity); by
denial_lane both compose into lane CONFORMANCE_GATE_FAILED, so a logical
refusal is subject to monotonicity, classification, pipeline composition, and
fired-mask projection exactly as any obligation refusal.

This file reuses `QueryResult`/`isDenied`/`isInvalid` from thm_trichotomy.lean,
`Scnset`/`catop`/`Catset` from def_tax.lean, and `Deny`/`laneop`/`isSingleLane`
from prop_section.lean, all verbatim (bare Lean core has no import mechanism
across these standalone pilot files, so the shared inductives are reproduced
here exactly as already kernel-verified elsewhere). It adds the embedding map
`run_kernel_query : QueryResult → Scnset` sending `denied` to the scenario
`denialF` (whose `catop` category is `Authorization`, matching "KernelDenied")
and `invalid` to the scenario `denialA` (whose `catop` category is `Identity`,
matching "KernelInvalid"), and proves that both refusing branches land, via
`catop` and then `laneop`, in a single well-defined lane that is one of the
seven named single lanes (the shared "CONFORMANCE_GATE_FAILED" lane family),
never the admitted/other lanes.
-/

/-- `RejectionCode` (reproduced from `thm_trichotomy.lean`). -/
inductive RejectionCode where
  | arityExceeded
  | bodyTooLong
  | tooManyVariables
  | fanInExceeded
  | cutUsed
  | dynamicMutation
  | unstratifiedNegation
  | unboundedRecursion
  | runtimeTextParsing
  | sideEffect
  | nonInternedTerm
  deriving Repr, DecidableEq

/-- `ProofNode` (reproduced from `thm_trichotomy.lean`). -/
inductive ProofNode where
  | positive (label : Nat)
  | negative (label : Nat)
  deriving Repr, DecidableEq

/-- `Receipt` (reproduced from `thm_trichotomy.lean`). -/
structure Receipt where
  hash : Nat
  deriving Repr, DecidableEq

/-- `Decision` (reproduced from `thm_trichotomy.lean`). -/
structure Decision where
  proof   : List ProofNode
  receipt : Receipt
  deriving Repr, DecidableEq

/-- `QueryResult` (reproduced from `thm_trichotomy.lean`). -/
inductive QueryResult where
  | answered (decisions : List Decision)
  | denied (decision : Decision)
  | invalid (code : RejectionCode)
  deriving Repr, DecidableEq

open QueryResult

/-- `isAnswered` (reproduced from `thm_trichotomy.lean`). -/
def isAnswered : QueryResult → Prop
  | answered _ => True
  | _ => False

/-- `isDenied` (reproduced from `thm_trichotomy.lean`). -/
def isDenied : QueryResult → Prop
  | denied _ => True
  | _ => False

/-- `isInvalid` (reproduced from `thm_trichotomy.lean`). -/
def isInvalid : QueryResult → Prop
  | invalid _ => True
  | _ => False

/-- `Catset` (reproduced from `def_tax.lean`). -/
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

/-- `Scnset` (reproduced from `def_tax.lean` / `thm_total.lean`). -/
inductive Scnset where
  | schemaObligation
  | policyObligation
  | signatureObligation
  | denialA
  | denialB
  | denialC
  | denialD
  | denialE
  | denialF
  | denialG
  | logicAndon1
  | logicAndon2
  | logicAndon3
deriving DecidableEq, Repr

/-- `catop` (reproduced verbatim from `def_tax.lean`). -/
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

/-- `Deny` (reproduced from `prop_section.lean`). -/
inductive Deny where
  | Adml
  | c1
  | c2
  | c3
  | c4
  | c5
  | c6
  | c7
  | other
deriving DecidableEq, Repr

/-- `laneop` / `denial_lane` (reproduced verbatim from `prop_section.lean`). -/
def laneop : Scnset → Deny
  | .schemaObligation => .other
  | .policyObligation => .other
  | .signatureObligation => .other
  | .denialA => .c1
  | .denialB => .c2
  | .denialC => .c3
  | .denialD => .c4
  | .denialE => .c5
  | .denialF => .c6
  | .denialG => .c7
  | .logicAndon1 => .other
  | .logicAndon2 => .other
  | .logicAndon3 => .other

/-- The seven single-lane words (reproduced from `prop_section.lean`); these
constitute the shared "CONFORMANCE_GATE_FAILED" denial-lane family that any
refusal — obligation-driven or logical — composes into. -/
def isSingleLane : Deny → Prop
  | .c1 | .c2 | .c3 | .c4 | .c5 | .c6 | .c7 => True
  | _ => False

/-- The embedding: `run_kernel_query` sends the two refusing branches of
`QueryResult` to taxonomy scenarios. `denied` embeds as `KernelDenied`
(scenario `denialF`, category `Authorization`); `invalid` embeds as
`KernelInvalid` (scenario `denialA`, category `Identity`). The `answered`
(non-refusing) branch has no embedding target here, since only refusals are
taxonomized. -/
def run_kernel_query : (r : QueryResult) → isDenied r ∨ isInvalid r → Scnset
  | denied _, _ => .denialF
  | invalid _, _ => .denialA
  | answered _, h => False.elim (by rcases h with h | h <;> exact h)

/-- **Embedding, denial branch**: `denied` embeds via `run_kernel_query` into
the scenario `denialF`, whose `catop` category is `Authorization`
("KernelDenied"), and whose `laneop` image is the single lane `c6`, a member
of the shared denial-lane family. -/
theorem embed_denied (d : Decision) :
    ∃ h : isDenied (denied d) ∨ isInvalid (denied d),
      catop (run_kernel_query (denied d) h) = Catset.Authorization ∧
      isSingleLane (laneop (run_kernel_query (denied d) h)) := by
  refine ⟨Or.inl trivial, ?_, ?_⟩
  · rfl
  · trivial

/-- **Embedding, invalid branch**: `invalid` embeds via `run_kernel_query` into
the scenario `denialA`, whose `catop` category is `Identity`
("KernelInvalid"), and whose `laneop` image is the single lane `c1`, a member
of the shared denial-lane family. -/
theorem embed_invalid (c : RejectionCode) :
    ∃ h : isDenied (invalid c) ∨ isInvalid (invalid c),
      catop (run_kernel_query (invalid c) h) = Catset.Identity ∧
      isSingleLane (laneop (run_kernel_query (invalid c) h)) := by
  refine ⟨Or.inr trivial, ?_, ?_⟩
  · rfl
  · trivial

/-- **`prop:embed`**: both refusing branches of `thm:trichotomy` (`Denied` and
`Invalid`) embed into the taxonomy via `run_kernel_query`, land in their
described categories (`Authorization` / `Identity`), and both compose — via
`laneop` — into a single named denial lane (the shared
"CONFORMANCE_GATE_FAILED" family, i.e. `isSingleLane`), exactly as any
obligation-driven refusal does. -/
theorem prop_embed :
    (∀ d : Decision, ∃ h : isDenied (denied d) ∨ isInvalid (denied d),
      catop (run_kernel_query (denied d) h) = Catset.Authorization ∧
      isSingleLane (laneop (run_kernel_query (denied d) h))) ∧
    (∀ c : RejectionCode, ∃ h : isDenied (invalid c) ∨ isInvalid (invalid c),
      catop (run_kernel_query (invalid c) h) = Catset.Identity ∧
      isSingleLane (laneop (run_kernel_query (invalid c) h))) :=
  ⟨embed_denied, embed_invalid⟩
