/- def:fable (reused verbatim, needed since bare Lean has no cross-file import
   mechanism in this single-file setup) -/

structure FableHarness
    (TaskOntology Prompt Hash Response Code Verdict : Type) where
  buildPrompt   : TaskOntology → Prompt
  hash          : Prompt → Hash
  mockMembrane  : Hash → Response
  extractCode   : Response → Code
  oracle        : TaskOntology → Response → Verdict

/- prop:fable-oracle vocabulary (reused verbatim) -/

inductive ErrorStage where
  | prompt
  | mockMembrane
  | extractCode
  | verify
  deriving DecidableEq, Repr

inductive Verdict where
  | pass
  | fail (stage : ErrorStage)
  deriving DecidableEq, Repr

/- remark:mockmembranes

Mock membranes as differential oracles: the mock membrane produces responses
from test fixtures; a real-model run produces a different response. Running
both and comparing (differential oracle) bounds the false-accept rate: an
implementation error that passes the oracle with the mock response but fails
with the real response is caught by the differential.

We model this abstractly, reusing `FableHarness` and `Verdict` from def:fable
and prop:fable-oracle. A `DifferentialRun` packages a harness together with a
second, "real-model" response-producing function `realMembrane : Hash →
Response` standing in for an actual LLM call (as opposed to `mockMembrane`,
which draws from fixtures). Given a task ontology `T`, the differential oracle
compares the mock verdict and the real verdict; we call an implementation
"differentially caught" when it passes on the mock response but fails on the
real response. This is exactly the discrepancy the differential is designed to
detect, and it is a decidable proposition for any concrete harness. -/

/-- A differential run: a Fable harness `H` together with a real-model
membrane `realMembrane`, standing in for an actual (non-fixture) LLM call. -/
structure DifferentialRun
    (TaskOntology Prompt Hash Response Code : Type) where
  H            : FableHarness TaskOntology Prompt Hash Response Code Verdict
  realMembrane : Hash → Response

/-- The mock-side verdict of a differential run on task ontology `T`: build
the prompt, hash it, run the harness's `mockMembrane`, and take the oracle
verdict on that mock response. -/
def DifferentialRun.mockVerdict
    {TaskOntology Prompt Hash Response Code : Type}
    (D : DifferentialRun TaskOntology Prompt Hash Response Code)
    (T : TaskOntology) : Verdict :=
  let P := D.H.buildPrompt T
  let h := D.H.hash P
  D.H.oracle T (D.H.mockMembrane h)

/-- The real-side verdict of a differential run on task ontology `T`: build
the prompt, hash it, run the *real* membrane instead of the mock one, and take
the oracle verdict on that real response. -/
def DifferentialRun.realVerdict
    {TaskOntology Prompt Hash Response Code : Type}
    (D : DifferentialRun TaskOntology Prompt Hash Response Code)
    (T : TaskOntology) : Verdict :=
  let P := D.H.buildPrompt T
  let h := D.H.hash P
  D.H.oracle T (D.realMembrane h)

/-- An implementation is "differentially caught" on `T` when the mock-side
verdict is `pass` but the real-side verdict is not `pass` (i.e. `fail stage`
for some stage). This is the Lean-level content of "an implementation error
that passes the oracle with the mock response but fails with the real
response is caught by the differential" — and it is decidable for any
concrete `D` and `T`, since `Verdict` has decidable equality. -/
def DifferentialRun.differentiallyCaught
    {TaskOntology Prompt Hash Response Code : Type}
    (D : DifferentialRun TaskOntology Prompt Hash Response Code)
    (T : TaskOntology) : Prop :=
  D.mockVerdict T = Verdict.pass ∧ D.mockVerdict T ≠ D.realVerdict T

instance DifferentialRun.differentiallyCaught.decidable
    {TaskOntology Prompt Hash Response Code : Type}
    (D : DifferentialRun TaskOntology Prompt Hash Response Code)
    (T : TaskOntology) : Decidable (D.differentiallyCaught T) :=
  match decEq (D.mockVerdict T) Verdict.pass, decEq (D.mockVerdict T) (D.realVerdict T) with
  | isTrue hp, isTrue he => isFalse (fun h => h.2 he)
  | isTrue hp, isFalse hne => isTrue ⟨hp, hne⟩
  | isFalse hnp, _ => isFalse (fun h => hnp h.1)
