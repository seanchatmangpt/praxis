/- def:fable

A Fable harness is a test environment that constructs a prompt `P_T` from a task
ontology `T`, calls a mock model membrane `M` that intercepts the LLM API and
returns a deterministic response drawn from a test fixture indexed by `P_T`'s
hash, extracts the code block `C` from the model response `R`, and runs the
verification oracle `Oracle(T,R)` against `C` in an isolated cargo workspace.
The mock membrane ensures that harness runs are hermetic: no real LLM call is
made, the response is deterministic given `P_T`, and the oracle verdict is
reproducible.

We model this abstractly in bare Lean 4 core:
- `TaskOntology` is the space of task ontologies `T`.
- `Prompt` is the space of constructed prompts `P_T`.
- `Hash` is the fixture index space (the hash of a prompt).
- `Response` is the space of model responses `R`.
- `Code` is the space of extracted code blocks `C`.
- `Verdict` is the outcome space of the verification oracle.

A `FableHarness` packages:
- `buildPrompt : TaskOntology → Prompt`, constructing `P_T` from `T`;
- `hash : Prompt → Hash`, indexing a prompt into the fixture space;
- `mockMembrane : Hash → Response`, the deterministic mock model membrane `M`
  (a pure function of the prompt's hash — no real LLM call, hence hermetic);
- `extractCode : Response → Code`, extracting the code block `C` from `R`;
- `oracle : TaskOntology → Response → Verdict`, the verification oracle
  `Oracle(T,R)` run against the extracted code in an isolated workspace.

Determinism and reproducibility are structural here: `mockMembrane` and `oracle`
are total functions, so identical inputs (`P_T`'s hash; `(T,R)`) always yield
identical outputs — the Lean-level encoding of "hermetic". -/

structure FableHarness
    (TaskOntology Prompt Hash Response Code Verdict : Type) where
  buildPrompt   : TaskOntology → Prompt
  hash          : Prompt → Hash
  mockMembrane  : Hash → Response
  extractCode   : Response → Code
  oracle        : TaskOntology → Response → Verdict

/-- The end-to-end run of a Fable harness on a task ontology `T`: build the
prompt, hash it, obtain the deterministic mock response, extract the code
block, and compute the oracle verdict. Returns the extracted code together
with the verdict. -/
def FableHarness.run
    {TaskOntology Prompt Hash Response Code Verdict : Type}
    (H : FableHarness TaskOntology Prompt Hash Response Code Verdict)
    (T : TaskOntology) : Code × Verdict :=
  let P := H.buildPrompt T
  let h := H.hash P
  let R := H.mockMembrane h
  let C := H.extractCode R
  (C, H.oracle T R)

/-- Hermeticity / reproducibility: since `mockMembrane` and `oracle` are plain
(total) functions of their arguments, running the harness twice on the same
task ontology yields identical results — no hidden nondeterminism, no real
LLM call. -/
theorem FableHarness.run_deterministic
    {TaskOntology Prompt Hash Response Code Verdict : Type}
    (H : FableHarness TaskOntology Prompt Hash Response Code Verdict)
    (T : TaskOntology) :
    H.run T = H.run T := rfl
