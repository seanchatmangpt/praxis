# SHACL report

Status: FINAL for this lane's run — `src/bin/case_study_judge.rs` has run
against the real merged graph (`graphlaw_judgment.ttl` +
`target/praxis-standing/standing.ttl`). Real numbers, from
`final_graphlaw_verdict.json` / `shacl-report.json`:

| shape file | conforms | violations |
|---|---|---|
| standing-envelope.shacl.ttl | true | 0 |
| case-study.shacl.ttl | true | 0 |
| evidence-ref.shacl.ttl | true | 0 |
| final-verdict.shacl.ttl | true | 0 |

All four shapes conform. `final-verdict.shacl.ttl` is validated AFTER the
judge bin asserts the real `praxis:FinalVerdict-v26_6_30` node (status,
scope, evidence, generatedAtUtc all populated from real data — see that
file's module doc for the exact ordering), so this is a real, non-vacuous
pass, not the trivially-conforming zero-target case described below for
the pre-bin sanity stage.

---

## Preliminary stage (superseded by the table above)

Status: PRELIMINARY — written before `src/bin/case_study_judge.rs` exists,
to record the sanity-check evidence for item 8 of Lane 2's task list. The
full run (once the judge bin exists) supersedes this file — see
`graphlaw_judgment_report.md` and `shacl-report.json` for the authoritative
numbers, and this file's own final update at the end of this lane.

## What was checked at this stage

A temporary test (`crates/praxis-graphlaw/tests/zz_case_study_sanity_check_temp.rs`,
deleted once this lane's judge bin lands — see item 11's permanent tests
for its replacement) confirmed:

- All four shape files under `case-study/shapes/` parse via
  `ShapesGraph::parse` without error:
  `standing-envelope.shacl.ttl`, `case-study.shacl.ttl`,
  `evidence-ref.shacl.ttl`, `final-verdict.shacl.ttl`.

No `validate_shacl` conformance run against the merged (seed +
`target/praxis-standing/standing.ttl`) graph has happened yet at this
stage — that happens once, per shape file, inside `case_study_judge.rs`.
See `graphlaw_judgment_report.md` for the real `conforms`/violation-count
numbers per shape file once that binary runs.

## Design notes carried into the shapes

- `standing-envelope.shacl.ttl` requires exactly one sourced
  `praxis:generatedAtUtc` `xsd:dateTime` literal, `>=1 praxis:hasEvidence`,
  and `>=1 praxis:hasVerdict` (a gate-verdict marker) per
  `praxis:StandingEnvelope`.
- `case-study.shacl.ttl` requires exactly one `praxis:hasScope` string and
  leaves `hasEvidence`/`recordsExecution`/`validatesProcess` at `>=0` for
  now (tightened once Lanes 3-7 land real evidence) — deliberately does
  NOT attempt a global "exactly one CaseStudy subject in the graph" check
  in SHACL (SHACL shapes validate per-focus-node property cardinality, not
  cross-graph node counts); that check is done via an explicit SPARQL
  `COUNT` query in the judge bin instead (see that file's module doc).
- `evidence-ref.shacl.ttl` requires exactly one `praxis:path` string and
  exactly one `praxis:hash` string matching `^sha256:[0-9a-f]{64}$`.
- `final-verdict.shacl.ttl` requires status/scope/evidence/generatedAtUtc
  on `praxis:FinalVerdict`, plus a companion
  `praxis:ExternalOperatorSideEffectShape` requiring
  `praxis:nonBlocking true` — no `praxis:FinalVerdict` node exists in the
  pre-verdict graph, so this shape's first `validate_shacl` pass is
  vacuously conformant (0 matching focus nodes); the judge bin asserts a
  real `praxis:FinalVerdict` node immediately after computing the verdict
  and validates this shape a second time against it.
