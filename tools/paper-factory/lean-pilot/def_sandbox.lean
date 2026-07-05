/-
def:sandbox

Let `P_T` be a prompt compiled from task ontology `T`, and let `C` be the code
block extracted from model response `R`. The sandbox verification oracle is
the composite:

  Oracle(T, R) = cargo_build(C) ∧ cargo_test(C) ∧ cargo_clippy(C) ∧ safety_audit(C)

The outcome is receipted and chained to the ledger using rolling BLAKE3 hashes.

We formalize the oracle abstractly: given a code artifact `C` (opaque carrier
type), four independent boolean check predicates, and a task/response pair,
the oracle is the conjunction of the four checks.
-/

/-- Opaque carrier for a task ontology `T`. -/
axiom TaskOntology : Type

/-- Opaque carrier for a model response `R`. -/
axiom ModelResponse : Type

/-- Opaque carrier for a code block `C` extracted from a response. -/
axiom CodeBlock : Type

/-- Extraction of the code block from a model response. -/
axiom extractCode : ModelResponse → CodeBlock

/-- The four independent sandbox checks, each a decidable predicate on code. -/
axiom cargoBuild   : CodeBlock → Bool
axiom cargoTest    : CodeBlock → Bool
axiom cargoClippy  : CodeBlock → Bool
axiom safetyAudit  : CodeBlock → Bool

/-- The sandbox verification oracle: conjunction of the four checks on the
code extracted from the response `R` (the ontology `T` is carried as an
input to the compiled prompt, not used directly by the boolean composite). -/
noncomputable def Oracle (_T : TaskOntology) (R : ModelResponse) : Bool :=
  let C := extractCode R
  cargoBuild C && cargoTest C && cargoClippy C && safetyAudit C
