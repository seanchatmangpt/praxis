# Case Study — Solace Cloud SOC2 Type II Audit Engagement Pipeline

This case study uses a fictional first-time SOC2 Type II audit engagement (Solace Cloud, a B2B
SaaS company) as the subject. `packs/soc2-audit-pack` emits the PUBLIC-vocabulary ontology and
engagement fixtures; the `cng` manufacture chain admits, plans, and hierarchically projects the
10-phase engagement into a POWL evidence structure; SHACL validates the case-study instance
data; a Mycin certainty-factor layer and a real Datalog engine derive standing audit roles in
parity; and 3 on-disk SPARQL queries measure the resulting evidence bundle. See
`AUDIT_READINESS_ASSESSMENT.md` for the declared scope statement and criteria this case study
does and does not claim.

## COMPLIANCE-OVERCLAIM FENCE (governs every claim in this bundle)

SOC2 "compliance" is an auditor's professional attestation under the AICPA Trust Services
Criteria, issued by a licensed CPA firm. Nothing in this pipeline, this case study, or this
document IS SOC2 compliance or a SOC2 opinion. This bundle documents the AUDIT ENGAGEMENT
PROCESS — evidence gathering, control design/operating-effectiveness evaluation, exception
handling — and the EVIDENCE BUNDLE it produces for a human auditor's professional judgment.
Every artifact and every metric below is worded as evidenced/exception-identified/remediation-
applied/evidence-bundle-assembled — never as "compliant," "passed," or "SOC2-ready." The
structural enforcement point (mechanical, not just this prose) is
`crates/cng/src/bench/soc2.rs::verify_no_compliance_or_opinion_effects`, which greps the parsed,
merged PDDL domain for any action effect naming "compliant" or "opinion" and refuses typed if one
is ever found; the only terminal goal atom the domain admits is `evidence-bundle-complete`.

## Stages

- **Stage 1** (commit `756a258470e19bedc3d12b456d9df7b3030ec76b`): `packs/soc2-audit-pack`
  (ontology + 13 templates), the 10 chained phase fixtures + 1 problem fragment + 1 case-study
  instance file, `crates/cng/src/bench/soc2.rs`/`soc2_test.rs` (4 tests: full-cycle plan/project/
  validate/replay, the 8-constraint split law, SHACL validation with a negative mutant, and the
  compliance-overclaim fence's mechanical enforcement).
- **Stage 2** (this bundle): a 16th `CATEGORIES` bench entry (`soc2-audit`, content-bearing via
  `ex:evidencesControl`), a 5-role SOC2 standing-role Mycin/Datalog layer, 3 new evidence-metric
  SPARQL queries plus a `DERIVED_ARITHMETIC`-tagged ratio, and this case-study bundle.

## Pipeline roles

| Role | Holder | Function |
|---|---|---|
| Ontology | `packs/soc2-audit-pack/ontology.ttl` | PUBLIC-vocabulary (skos/prov/dcterms/org) SKOS concept schemes for the 10 phases, 5 TSC categories, and CC/A1/C1 criteria |
| Planner | `cng::pipeline` (`bcinr-pddl`) | Admits the 10 phase fragments + 1 problem fragment, merges structurally, plans the 30-step cycle |
| Projector | `cng::powl` | Groups the plan into a 10-child hierarchical POWL by artifact provenance, exports deterministic Turtle |
| Shape judge | `crates/cng/shapes/soc2-shapes.ttl` + the crate's 3 generic shape queries | Validates the case-study instance data (control points, phase activities, the audited enterprise, evidence artifacts) |
| Standing-role layer | `bench::roles` (Mycin + praxis-graphlaw Datalog) | Derives the 5 SOC2 standing roles (control-owner, internal-audit-lead, compliance-program-manager, remediation-engineer, evidence-custodian) with certainty factors, cross-checked against a real Datalog engine |
| Evidence metrics | `bench::soc2::compute_evidence_metrics` + 3 `metric-soc2-*.rq` queries | Measures evidenced-control count, exception-register-artifact count, remediation-log-artifact count, and one Rust-computed `DERIVED_ARITHMETIC` ratio |

## Lane findings

- `lane-reports/lane-1-ontology-and-pack.md` — ontology/pack structure, fixture inventory.
- `lane-reports/lane-2-pddl-powl-pipeline.md` — merge/plan/hierarchical-projection results.
- `lane-reports/lane-3-shacl-validation.md` — shape law and the negative-mutant proof.
- `lane-reports/lane-4-mycin-datalog-roles.md` — the 5-role standing layer and its parity test.
- `lane-reports/lane-5-sparql-evidence-metrics.md` — the 3 new queries and the derived ratio.

## Concurrent-edit disclosure

While Stage 2 was in progress, `git status` on `crates/cng/tests/fixtures/soc2/` and
`packs/soc2-audit-pack/` showed live, uncommitted changes from a separate concurrent session
(observed task title: "SOC2 Fortune-5 rescale: ontology/case-study/templates/hook-actuation") —
renaming the case-study company from Solace Cloud to Arclight across the pack templates and
rendered fixtures, deleting `solace-case-study.ttl` in favor of a new `arclight-case-study.ttl`,
mid-transaction (the Rust side, `soc2.rs`/`soc2_test.rs`, had not yet been updated to match, so
Stage 1's own committed tests were transiently failing against the live working tree). No
destructive action was taken against that work: none of its files were reverted, restored, or
edited. This case-study bundle's artifacts were instead generated from an ISOLATED scratchpad
copy of the Stage-1-COMMITTED fixture content (`git show 756a258470e19bedc3d12b456d9df7b3030ec76b:crates/cng/tests/fixtures/soc2/*` —
see each lane report's "source" note), so every number and digest in this bundle is real,
pipeline-produced output — just computed against a private copy rather than the contested shared
directory, to avoid racing a live session. `crates/cng/src/bench/soc2_test.rs` retains its
permanent tests unchanged in behavior (they read the canonical `tests/fixtures/soc2/` path and
will pass once that directory is back in a consistent state, whichever company name that
concurrent effort settles on).

## See Also

- `AUDIT_READINESS_ASSESSMENT.md` — declared scope, criteria claimed and not claimed
- `EVIDENCE_MANIFEST.md` — every artifact path, hash, and producing command
- `FINAL_VERDICT.md` — pipeline-verification status (never a compliance verdict)
- `crates/cng/src/bench/soc2.rs` — the fence disclosure this bundle inherits
