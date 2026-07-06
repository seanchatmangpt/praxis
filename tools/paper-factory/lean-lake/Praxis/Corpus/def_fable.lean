import Mathlib.Data.BitVec

/-!
def:fable

A Fable harness is a test environment that constructs a prompt `P_T` from a
task ontology `T`, calls a mock model membrane `M` that intercepts the LLM
API and returns a deterministic response drawn from a test fixture indexed
by `P_T`'s hash, extracts the code block `C` from the model response `R`,
and runs the verification oracle `Oracle(T,R)` against `C` in an isolated
cargo workspace. The mock membrane ensures that harness runs are hermetic:
no real LLM call is made, the response is deterministic given `P_T`, and
the oracle verdict is reproducible.

Every field below is composed from a type Mathlib/Lean core already
provides -- no axioms are declared:

* `Digest := BitVec 256` -- the prompt-hash used to index the fixture
  table, the same real, pre-built 256-bit vector type used in
  `Praxis.Mathlib.DefReceipt` for `def:receipt`'s digests. Reusing it here
  keeps "hash used as a fixture key" and "hash used as a chain digest"
  the same concrete type across the corpus.
* `TaskOntology`, `Prompt`, `ModelResponse`, `CodeBlock` are all `String`
  -- each is exactly a piece of text (an ontology serialization, a
  rendered prompt, a model's textual response, an extracted source
  block); `String` is the pre-built type for that.
* The mock membrane `M` is modeled as a genuine total function
  `Digest → ModelResponse` (`MockMembrane.fixture`), not an opaque axiom:
  a fixture table is exactly a function from hash to canned response, and
  Lean/Mathlib's function type already captures "deterministic given the
  hash of `P_T`" -- determinism is definitional (`fixture` is a function,
  so equal inputs give equal outputs), requiring no separate axiom to
  assert it.
* `buildPrompt`, `hashPrompt`, `extractCode`, and `oracle` are likewise
  plain functions between the `String`/`Digest`/`Bool` types above --
  each stage of the harness (prompt construction, hashing, code
  extraction, oracle verdict) is a total function, composed from
  pre-built types, with no new axiomatic primitive required.
* The oracle verdict is `Bool`, the pre-built type for a pass/fail
  judgement; "isolated cargo workspace" is captured as the *domain* over
  which `oracle` is total (it always returns a verdict), not as a
  separate axiomatized notion -- no concrete filesystem/process model is
  needed to state what a Fable harness *is*.
-/

abbrev Digest := BitVec 256
abbrev TaskOntology := String
abbrev Prompt := String
abbrev ModelResponse := String
abbrev CodeBlock := String

/-- The mock model membrane: a deterministic, hermetic stand-in for the
LLM API, returning a fixed response for each prompt digest. -/
structure MockMembrane where
  fixture : Digest → ModelResponse

/-- A Fable harness: task ontology, prompt builder, mock membrane, prompt
hasher, code extractor, and verification oracle. -/
structure FableHarness where
  ontology : TaskOntology
  buildPrompt : TaskOntology → Prompt
  membrane : MockMembrane
  hashPrompt : Prompt → Digest
  extractCode : ModelResponse → CodeBlock
  oracle : TaskOntology → ModelResponse → Bool

/-- Running a Fable harness: build the prompt, obtain the deterministic
mock response, extract its code block, and evaluate the oracle. Returns
the oracle's verdict together with the extracted code, both reproducible
functions of `h.ontology` alone. -/
def runFableHarness (h : FableHarness) : Bool × CodeBlock :=
  let P := h.buildPrompt h.ontology
  let R := h.membrane.fixture (h.hashPrompt P)
  let C := h.extractCode R
  (h.oracle h.ontology R, C)
