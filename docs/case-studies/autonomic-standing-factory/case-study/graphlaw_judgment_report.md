# GraphLaw judgment report (real run)

Verdict: **ProductionReadyForDeclaredScope**

Scope: local-first autonomic release-governance for the seanchatmangpt fleet

Case-study subjects found in the merged graph: 1 (must be exactly 1)

Derived triples across all materialize() passes: 15

Unsatisfied-dependency count (aggregate rule): 0

Denials: 0 — []

## SHACL

| shape file | conforms | violations |
|---|---|---|
| standing-envelope.shacl.ttl | true | 0 |
| case-study.shacl.ttl | true | 0 |
| evidence-ref.shacl.ttl | true | 0 |
| final-verdict.shacl.ttl | true | 0 |

## ShEx

conforms: true, failures: 0

## Acceptance criteria

| id | satisfied | critical | description |
|---|---|---|---|
| Criterion01 | true | true | canonical standing evidence exists |
| Criterion02 | true | true | GraphLaw validates shapes |
| Criterion03 | true | true | GraphLaw validates graph structure |
| Criterion04 | true | true | GraphLaw derives readiness facts |
| Criterion05 | true | true | GraphLaw computes closure over blockers/dependencies |
| Criterion06 | true | true | PDDL produces/records a lawful repair/action plan |
| Criterion07 | true | true | POWL models the execution process |
| Criterion08 | true | true | OCEL records the case-study run |
| Criterion09 | true | true | wasm4pm validates process conformance |
| Criterion10 | true | false | receipts verify where applicable |
| Criterion11 | true | false | benchmark evidence exists where performance claims are made |
| Criterion12 | true | true | Autonomic Platform displays case-study state with provenance |
| Criterion13 | true | false | Claude Code policy consumes or points to standing |
| Criterion14 | true | false | unsupported claims are diagnosable |
| Criterion15 | true | true | external operator side effects are separated from release blockers |

## Graph hash

`blake3:4e1843d2cf5dfc8b12e2ad30e72329ce58a77d1b8c6f7ac255101bec399a6efa`

generated_at_utc (sourced from standing envelope): 2026-07-06T22:27:34.851Z
