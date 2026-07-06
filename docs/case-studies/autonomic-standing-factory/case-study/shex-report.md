# ShEx report

Status: FINAL for this lane's run. Real numbers, from
`final_graphlaw_verdict.json` / `shex-report.json`:

```
conforms: true
failure_count: 0
```

The shape map validated by the real run covers the case-study subject
(`CaseStudyShape`), the real standing envelope (`StandingEnvelopeShape`,
found via `?s a praxis:StandingEnvelope`), the judge role-holder
(`GraphLawJudgmentShape`, found via `?s a praxis:Judge`), the
not-yet-produced process-validation placeholder
(`ProcessValidationShape`, found via `?s a praxis:ProcessValidationRef`),
and the one promoted claim (`PromotedClaimShape`, found via
`?s a praxis:PromotedClaim`) — every entry built from a live SPARQL query
over the merged graph, not a fixed list.

---

## Preliminary stage (superseded by the numbers above)

Status: PRELIMINARY — see `shacl-report.md`'s header note; the same
applies here. Superseded by `graphlaw_judgment_report.md` and
`shex-report.json` once `src/bin/case_study_judge.rs` runs.

## What was checked at this stage

The temporary sanity test confirmed `case-study/shex/case-study.shex`
parses via `shexc_parser::parse_shexc`, and that the real
`praxis:AutonomicStandingFactoryCaseStudy` subject conforms to
`praxis:CaseStudyShape` when validated via `TripleStore::validate_shex_c`
against the seed graph alone (`graphlaw_judgment.ttl`, before merging
Lane 1's live `standing.ttl`):

```
report.conforms == true
```

## Scope notes

- `CaseStudyShape`'s `praxis:hasEvidence @praxis:StandingEnvelopeShape +`
  requires >=1 standing envelope per the control spec; `recordsExecution`,
  `validatesProcess`, `plansRepair`, `modelsProcess`, `displaysStanding`,
  and `hasExternalSideEffect` are all `* ` (>=0) since Lanes 3-7 have not
  produced real evidence yet.
- `PromotedClaimShape` requires `>=1 praxis:hasEvidence` — the seed
  graph's one promoted claim (`praxis:ClaimCanonicalStandingEvidenceExists`)
  satisfies this with 3 real evidence references.
- "Exactly one CaseStudy subject in the whole graph" is NOT expressible in
  this crate's ShExC subset (no global cross-graph cardinality construct —
  shape conformance is always per focus node) and is instead checked by
  `src/bin/case_study_judge.rs` via an explicit SPARQL `COUNT` query,
  recorded in `graphlaw_judgment_report.md`.

The full shape map (built from real subjects in the merged graph, covering
`StandingEnvelopeShape`, `EvidenceBundleShape`, `GraphLawJudgmentShape`,
`ProcessValidationShape`, and `PromotedClaimShape` in addition to
`CaseStudyShape`) is validated once inside `case_study_judge.rs` — see
`shex-report.json` for the real per-node conformance results.
