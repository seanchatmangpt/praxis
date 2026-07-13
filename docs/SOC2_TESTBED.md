# SOC2 Audit-Engagement Testbed

This case study demonstrates that the `multifractal-workflow`/`cng` PDDL→POWL pipeline
can model a real, standard SOC2 Type II audit engagement lifecycle — 10 phases from
Scoping through Report Handoff, evidence-gathered continuously via F09 growth, with every
control point that cannot be genuinely evidenced refusing typed rather than fabricating a
pass. It explicitly is **not** a SOC2 compliance claim, attestation, or opinion — about
this repo, `crates/cng`, or any other system. SOC2 "compliance" is an auditor's
professional attestation under AICPA's Trust Services Criteria (TSC), issued by a licensed
CPA firm; nothing a PDDL planner, a POWL decomposition, or a Rust test suite produces can
ever *be* that attestation. This document reports what was built and what actually passed,
using the vocabulary this repo requires: evidenced, exception identified, remediation
applied, evidence bundle assembled — never compliant, passed the audit, or SOC2-ready.

## The fence, made structural

The fence is enforced in the PDDL action space itself, not just in prose. The
`soc2-audit-pack` domain (`packs/soc2-audit-pack/`) has no action anywhere whose effect
predicate is `(compliant ?x)` or `(opinion-issued ?x)`; the only terminal goal atom across
all 10 phases is `(evidence-bundle-complete ?x)`. Stage 1's
`verify_no_compliance_or_opinion_effects()` (`crates/cng/src/bench/soc2.rs`) makes this a
mechanical check: it parses every action in the real 30-action domain and greps effect
predicates for `compliant`/`opinion`, rather than trusting a doc comment. Two adversarial
mutants — renaming an effect to `audit-compliant` and to `auditor-opinion-issued` — were
run against this check and both refused typed (`CNG_R05 UnsupportedConstruct`, naming the
fence) instead of silently passing.

Building this domain against the real AICPA TSC structure caught three framing corrections
against a sibling effort (`mfact`, which formalizes TSC correspondence in Lean) that this
session applied: **CC8** is Change Management, not Processing Integrity — Processing
Integrity is its own top-level category (`PI`), not a Common Criteria subpoint. **PI1.1–1.5**
concerns the completeness/accuracy/timeliness/authorization of *business data processing*,
not audit receipts or evidence artifacts. **CC9** ("Risk Mitigation") has two distinct
points of focus — business-disruption risk and vendor/business-partner risk — which is why
the ontology's SKOS concept scheme splits it into `CC9-1`/`CC9-2` rather than treating it
as one control point. These are cited here as exact facts about the TSC taxonomy, not as
a claim about mfact's own state — this repo does not have visibility into whether mfact
independently caught or corrected the same framing.

## What was built

- **Ontology and phases** (Stage 1, commit `756a2584`): `packs/soc2-audit-pack/` — a
  public-ontology-first domain (SKOS/PROV/DCTERMS/ORG) with 5 concept schemes: 10 audit
  phases, 5 TSC categories, and the CC1–CC9/A1-1..A1-3/C1-1/C1-2 control-point hierarchy
  (with the CC9 split above applied). 13 templates render to 13 on-disk PDDL/SHACL
  fixtures under `crates/cng/tests/fixtures/soc2/`, verified byte-identical to their
  template bodies before commit.
- **Category and roles wiring** (Stage 2, commit `3909f89d`): `soc2-audit` added as
  `crates/cng`'s 16th bench category; `role_of` gained an `auditor` role; a Mycin
  sub-table (`soc2_role_rules`/`infer_soc2_standing_role`) and a matching Datalog rule set
  (`rules/bench-roles.dl`) cover 5 named standing roles (control-owner,
  internal-audit-lead, compliance-program-manager, remediation-engineer,
  evidence-custodian) at 0.95/0.9 certainty factors, with a test
  (`soc2_standing_roles_mycin_and_datalog_agree`) asserting the two representations agree.
  Three COUNT-shaped SPARQL metrics (evidenced-controls, exceptions, remediation-status)
  and `Soc2EvidenceMetrics`/`compute_evidence_metrics` were added, tagging derived ratios
  `DERIVED_ARITHMETIC` rather than presenting them as directly measured.
- **Continuous re-evidencing / F09 growth** (Stage 3, commit `fd504676`):
  `crates/cng/src/bench/soc2_growth.rs` bridges cng's `Powl` and F09's
  `powl2_decompose::Powl` and runs a 6-descent quarterly re-testing cycle (Q1–Q4 retests,
  one remediation continuation, one annual closure; `RE_TEST_BUDGET = 6`) grafted onto a
  real, provenance-located socket in the OE-Testing phase. One genuine exception
  (`CTRL-ACCESS-PROVISIONING` evidence gap) is surfaced in Q2 and remediated in a separate
  goal continuation; a 7th descent past budget refuses typed
  (`MFWGrowthRefused::DescentBudgetExhausted { budget: 6, depth: 6 }`) rather than
  silently extending. Because `multifractal-workflow` already depends on `cng`, the
  reverse dependency needed for this bridge is only permitted by Cargo as a
  `[dev-dependencies]` edge — so `bench::soc2_growth` is `#[cfg(test)]`-scoped end to end,
  disclosed in both files' module docs.

## Verification

Numbers below are taken directly from each stage's own session report; nothing here is
recomputed or rounded up.

- **Stage 1**: `soc2` filter — 4 passed, 0 failed. Full `crates/cng` lib suite —
  177 passed, 0 failed (173 pre-existing + 4 new, no regression). Fixture-integrity check
  (`sha256sum -c` across all 13 fixtures) confirmed byte-identical after an adversarial
  precondition break was applied and reverted.
- **Stage 2**: `bench::soc2` — 5 passed; `bench::roles` — 2 passed; `bench::hooks` —
  5 passed. Full lib suite: 179 passed in an isolated scratchpad copy of the
  Stage-1-committed fixtures, but **176 passed / 3 failed against the live shared tree**
  at the time, due to a disclosed concurrent-edit collision (a separate process renaming
  the case study's fixture data mid-session, uncommitted). Three unrelated pre-existing
  tests broke from widening the bench category count 15→16 (a fixed-seed `splitmix64`
  draw-sequence shift) and were fixed forward in this same commit.
- **Stage 3**: `soc2_growth` filter — 5 passed, 0 failed. Full lib suite: **181 passed,
  3 failed** — the 3 failures are reported as pre-existing, attributed to a separate
  Solace→Arclight rescale commit (`f8c9e3dd`) landing on `soc2.rs`/`soc2_test.rs`'s
  still-Solace-named constants, tracked as a follow-up rather than fixed in Stage 3. This
  document does not independently re-run that suite to confirm the attribution — treat the
  "pre-existing, not caused by Stage 3" framing as **UNVERIFIED** by this document, only as
  reported.
- Across all three stages: `just fmt-check-pkg cng` is reported clean on every file each
  stage touched. Clippy findings reported in Stage 1 were pre-existing dead-code false
  positives shared with `togaf.rs`, not a regression; one real finding (an unused `pub use`)
  was fixed before that stage's commit.
- None of the three stage commits (`756a2584`, `3909f89d`, `fd504676`) are marked
  crown-frontier commits.

## Relationship to the parallel mfact effort

`mfact` (a sibling repo) proves Lean 4 theorems about TSC correspondence and explicitly
stops before making any runtime-correspondence claim — a theorem there strengthens
evidence for a specific control point, never becomes SOC2 compliance itself. This praxis
case study is a structurally **separate** effort: a workflow-engine domain-modeling
exercise over PDDL/POWL, not a discharge of any mfact-defined `StepCorrespondence`
obligation. The two are not currently wired together, and nothing in this document, in
`crates/cng/src/bench/soc2*.rs`, or in `packs/soc2-audit-pack/` claims otherwise.

## See also

- `docs/case-studies/solace-cloud-soc2/` — the full case-study bundle (CASE_STUDY.md,
  AUDIT_READINESS_ASSESSMENT.md, FINAL_VERDICT.md, EVIDENCE_MANIFEST.md, lane reports).
- `packs/soc2-audit-pack/` — ontology, phase templates, SHACL shapes.
- `crates/cng/src/bench/soc2.rs`, `soc2_growth.rs` — fence check, phase order, growth
  orchestrator.
- `docs/GGEN_PARITY.md` — the honest-disclosure style this document mirrors.
