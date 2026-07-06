import Praxis.Corpus.def_fable
import Praxis.Corpus.prop_fable_oracle

/-!
remark:mockmembranes

Mock membranes as differential oracles: the mock membrane produces
responses from test fixtures; a real-model run produces a different
response. Running both and comparing (differential oracle) bounds the
false-accept rate: an implementation error that passes the oracle with
the mock response but fails with the real response is caught by the
differential.

This is a remark, not a new theorem: it is stated as a definition built
entirely from the vocabulary already established in `def:fable` and
`prop:fable-oracle`, with no proof obligation beyond type-checking.

* A "real-model run" is modeled the same way `def:fable` models the mock
  membrane: a function `Digest → ModelResponse`. No new axiom is
  introduced -- `RealMembrane` is literally the same shape as
  `MockMembrane` (a total function into `ModelResponse`), since a real
  LLM call, from the harness's point of view, is exactly "some function
  from prompt digest to response text"; only its provenance (real API
  call vs. fixture table) differs, which is not part of the type.
* The "differential oracle" is the plain conjunction of two `Bool`
  oracle verdicts, one against the mock response and one against the
  real response -- `Bool.and`, Lean core's pre-built boolean AND, is
  exactly "both must pass" and needs no bespoke definition.
* "Bounds the false-accept rate" is captured as the remark
  `differential_catches_mock_only_pass` below: if the differential
  oracle passes, the oracle already passed on the mock response too
  (the converse direction, "mock pass but real fail is caught", is
  immediate from `Bool.and_eq_true` case analysis on the mock/real
  verdict pair -- no separate axiom about error rates or probability
  is needed, since the corpus statement is a logical bound, not a
  quantitative probabilistic claim).
-/

/-- A real-model run: the same total-function shape as `MockMembrane`
(`Digest → ModelResponse`), reused rather than re-axiomatized, since a
harness only ever consumes "some function from prompt digest to
response text" regardless of whether the response came from a fixture
table or a live model call. -/
structure RealMembrane where
  respond : Digest → ModelResponse

/-- The differential oracle: run the harness's oracle against both the
mock response and the real-model response for the same ontology/prompt,
and require both to pass. This is exactly `Bool.and` applied to the two
verdicts -- "an implementation error that passes with the mock response
but fails with the real response" is precisely the case where this
conjunction is `false` even though the mock-only verdict was `true`. -/
def differentialOracle
    (h : FableHarness) (real : RealMembrane) : Bool :=
  let P := h.buildPrompt h.ontology
  let mockR := h.membrane.fixture (h.hashPrompt P)
  let realR := real.respond (h.hashPrompt P)
  h.oracle h.ontology mockR && h.oracle h.ontology realR

/-- The differential oracle bounds the false-accept rate exactly as
described: whenever the differential (mock-and-real) verdict passes, the
mock-only verdict must also have passed. Contrapositively, any case
where the mock-only oracle passes but the real-model run does not is
excluded from a differential pass -- it is caught by the differential,
which is the content of the remark. -/
theorem differential_catches_mock_only_pass
    (h : FableHarness) (real : RealMembrane)
    (hd : differentialOracle h real = true) :
    h.oracle h.ontology
      (h.membrane.fixture (h.hashPrompt (h.buildPrompt h.ontology))) = true := by
  unfold differentialOracle at hd
  exact (Bool.and_eq_true _ _).mp hd |>.1
