# Evidence-Pipeline Verification Status — Solace Cloud SOC2 Case Study

Not a "final verdict" in the compliance sense — see the COMPLIANCE-OVERCLAIM FENCE in
`CASE_STUDY.md`. This document states PIPELINE VERIFICATION status only: which tests passed,
which artifacts were produced, and what they measured. It is never a statement about whether
Solace Cloud (fictional) is SOC2 compliant.

## Status

**Evidence pipeline verified: 10/10 phases evidenced, 5/5 standing roles derived in Mycin/
Datalog parity, 3/3 evidence-metric queries measured, 0 unexplained SHACL violations on the
shipped case-study instance data, 0 compliance/opinion effect atoms in the merged 30-action PDDL
domain — all verified against the Stage-1-committed fixture content. UNVERIFIED against the live
shared `tests/fixtures/soc2/` directory as of this writing, because a separate concurrent session
was mid-transaction renaming those exact fixtures (see the "in shared tree" column below and
`CASE_STUDY.md`'s concurrent-edit disclosure) — this is a disclosed environmental condition, not
a defect in this stage's code.**

## What was verified (Stage 1 + Stage 2 combined)

**Column note**: "In isolation" = verified against a private scratchpad copy of the
Stage-1-COMMITTED fixtures (`git show 756a258470e19bedc3d12b456d9df7b3030ec76b:...`), isolated
from the concurrent, uncommitted, in-progress rename described in `CASE_STUDY.md`. "In shared
tree" = the SAME test, same code, run against the live `crates/cng/tests/fixtures/soc2/`
directory at the moment this document was last updated — which a separate live session was
mid-transaction renaming (Solace Cloud → Arclight) at that moment.

| Check | In isolation | In shared tree (at time of writing) | Evidence |
|---|---|---|---|
| Full 30-step audit cycle plans, projects into 10 phase children, validates POWL structural shape, replays byte-identically | PASS | FAILS (`define-system-boundary(arclight)` vs expected `(solace)` — the OTHER session's renamed fixture content, not a defect in this test or the pipeline) | `soc2_test.rs::full_audit_cycle_plans_projects_validates_and_replays_byte_identically` |
| 8-constraint split law (PDDL8 bound) holds and its violation refuses typed | PASS | PASS (unaffected by the rename) | `soc2_test.rs::eight_constraint_split_violations_refuse_typed` |
| Case-study instance data satisfies the SHACL shapes; a stripped-notation mutant is caught | PASS | FAILS (`solace-case-study.ttl` deleted by the other session in favor of an unwired `arclight-case-study.ttl`) | `soc2_test.rs::case_study_instance_data_passes_the_soc2_shapes_and_a_mutant_fails` |
| No action effect in the merged domain ever asserts "compliant" or "opinion"; 2 adversarial mutants refuse typed | PASS | PASS (unaffected) | `soc2_test.rs::no_action_effect_ever_asserts_compliance_or_opinion` |
| `soc2-audit` bench category (16th `CATEGORIES` entry) is content-bearing, hook-actuated, and role-classified | PASS | PASS (unaffected — this check has no dependency on the case-study company name) | `hooks_test.rs::broker_covers_every_bench_category`, `hooks_test.rs::real_registry_passes_the_closed_shape_gate` |
| 5 SOC2 standing roles derive matching text between Mycin certainty-factor rules and a real Datalog engine | PASS | PASS (unaffected) | `roles_test.rs::soc2_standing_roles_mycin_and_datalog_agree` |
| Evidence-bundle metrics (2 measured counts + 1 `DERIVED_ARITHMETIC` ratio) compute correctly, including the divide-by-zero case | PASS | FAILS (same missing `solace-case-study.ttl` as above) | `soc2_test.rs::soc2_evidence_metrics_measure_the_shipped_case_study_instance_data` |
| Full `cng` lib regression | 179/179 (isolated per-module runs, see `case-study/raw/test-run-output.txt`) | 176/179 (3 of the above fail; 3 separately-disclosed sequence-shift fixes below already applied and passing) | this session's `just cng-test-lib-isolated soc2-2` runs |

The 3 "in shared tree" failures are entirely attributable to the other session's in-progress,
uncommitted rename (confirmed by reading the diff: only the company name and its derived object
constants changed; predicate/action structure is untouched) — not a defect this stage
introduced. They are expected to clear on their own once that session either finishes updating
`soc2.rs`/`soc2_test.rs` to match its new fixtures, or that work is reverted/committed
separately. This stage's own permanent tests were not weakened, skipped, or altered to
paper over them.

## Measured evidence-bundle numbers (Solace Cloud, 3 documented control points)

From `case-study/ocel/evidence-metrics.json`:

```json
{
  "measurement_class": "SOC2_EVIDENCE_BUNDLE_METRICS",
  "evidenced_controls": 3,
  "exception_register_artifacts": 1,
  "remediation_log_artifacts": 1,
  "derived_exception_register_ratio": 0.3333333333333333,
  "derived_exception_register_ratio_class": "DERIVED_ARITHMETIC"
}
```

Reading: 3 control points (CTRL-ACCESS-PROVISIONING, CTRL-DR-FAILOVER-TEST,
CTRL-DATA-CLASSIFICATION) carry a complete evidentiary record; the engagement produced 1
exception-register deliverable (evidence that exception identification was performed for this
engagement — not a count of exceptions within it) and 1 remediation-log deliverable (evidence
that a management response was recorded). The ratio is Rust arithmetic over the two measured
counts, explicitly tagged `DERIVED_ARITHMETIC` in the struct itself — never a SPARQL aggregate,
never an implied pass rate.

## Disclosed: 3 pre-existing tests required a fix-forward sequence-shift correction

Adding the 16th `CATEGORIES` entry shifts the deterministic `splitmix64` category-draw sequence
for every fixed-seed test that draws categories. Three pre-existing tests broke as a direct,
unavoidable consequence and were fixed forward (never reverted, never weakened):

1. `hooks_test.rs::broker_covers_all_fifteen_categories` (renamed
   `broker_covers_every_bench_category`) and `real_registry_passes_the_closed_shape_gate` — both
   hardcoded a literal category count; fixed to read `CATEGORIES.len()` dynamically, and
   `hooks/workday-pack-2.ttl` gained the `ex:hook-soc2-audit` entry every category requires.
2. `multifractal_test.rs::track2b_real_workday_tape_ops_measurement` — the shifted draw sequence
   surfaced `UnreceiptedActuation` before the hook fix above; resolved by the same hook addition.
3. `workday_test.rs::workday_bounded_admission_resumes_every_refusal` — the shifted sequence now
   draws `api-orchestration` within its 3 ticks; fixed by seeding the same `generated/
   arazzo.yaml` + `.ggen-v2/receipt.json` precondition `track2b_real_workday_tape_ops_measurement`
   already seeds for the identical, pre-existing reason.

None of these three fixes touch SOC2-specific logic; they are the necessary, disclosed cost of
widening a shared category enum, fixed forward per this repo's own policy.

## Concurrent-edit disclosure

See `CASE_STUDY.md`'s own section. This bundle's artifacts were computed against an isolated
scratchpad copy of the Stage-1-committed fixtures, not the live (mid-rename) shared directory —
no destructive action was taken against the concurrent session's work.

## Evidence references

- Full-cycle plan/POWL: `case-study/pddl-out/plan.json`, `case-study/powl/solace-soc2-powl.ttl`,
  `case-study/powl/powl-digest.txt`
- SHACL: `case-study/shapes/soc2-shapes.ttl`, `case-study/shapes/shape-violations.json`
- Evidence metrics: `case-study/ocel/evidence-metrics.json`
- Full evidence index: `EVIDENCE_MANIFEST.md`

## Commit references

- Stage 1: `756a258470e19bedc3d12b456d9df7b3030ec76b`
- Stage 2: this session's commit (see repository log; recorded in `EVIDENCE_MANIFEST.md`)
