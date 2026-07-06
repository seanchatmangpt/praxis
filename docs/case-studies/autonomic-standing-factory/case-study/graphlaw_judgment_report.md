# GraphLaw judgment report (real run)

Verdict: **NotReadyWithReasons**

Scope: local-first autonomic release-governance for the seanchatmangpt fleet

Case-study subjects found in the merged graph: 1 (must be exactly 1)

Derived triples across all materialize() passes: 22

Unsatisfied-dependency count (aggregate rule): 10

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
| Criterion06 | false | true | PDDL produces/records a lawful repair/action plan |
| Criterion07 | false | true | POWL models the execution process |
| Criterion08 | false | true | OCEL records the case-study run |
| Criterion09 | false | true | wasm4pm validates process conformance |
| Criterion10 | false | false | receipts verify where applicable |
| Criterion11 | false | false | benchmark evidence exists where performance claims are made |
| Criterion12 | false | true | Autonomic Platform displays case-study state with provenance |
| Criterion13 | false | false | Claude Code policy consumes or points to standing |
| Criterion14 | false | false | unsupported claims are diagnosable |
| Criterion15 | false | true | external operator side effects are separated from release blockers |

## Graph hash

`blake3:b4405bf64afff70c68007d1cd9c0002a1c898200e1feda0c4d510edf4c1d5555`

generated_at_utc (sourced from standing envelope): 2026-07-06T22:27:34.851Z
