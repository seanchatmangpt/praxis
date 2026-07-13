# Lane 3 — SHACL Validation

Status: DONE (Stage 1 shape law, re-verified this session with the same fixture content).

## Shapes

`crates/cng/shapes/soc2-shapes.ttl` (copied into `case-study/shapes/soc2-shapes.ttl`): 5 node
shapes — `Soc2ConceptShape` (every `skos:Concept` needs a notation handle), `Soc2AuditPhaseShape`
(closed; every phase execution needs notation + description + attribution + at least one
generated deliverable), `Soc2ControlPointShape` (closed; every control point needs notation +
description + attribution + exactly one closed-enumerated TSC-category subject),
`Soc2EnterpriseShape` (the audited organization needs notation + description + at least one
in-scope TSC category), `Soc2EvidenceArtifactShape` (every deliverable needs notation +
description).

## Result on the shipped Solace Cloud instance data

`case-study/shapes/shape-violations.json` — `[]` (0 violations), computed via the SAME 3 generic
shape queries every other shape law in this crate uses
(`registry-missing-fields.rq`/`shape-closed-violations.rq`/`shape-pattern-violations.rq`), never
a bespoke SPARQL-based SHACL target extension.

## Negative proof (the shapes actually bite)

`soc2_test.rs::case_study_instance_data_passes_the_soc2_shapes_and_a_mutant_fails` strips
`skos:notation "CTRL-DATA-CLASSIFICATION"` from the instance data and re-runs the same shape
check — the mutant produces ≥ 1 violation. Passed this session as part of
`just cng-test-lib-isolated soc2-2 bench::soc2`.

## Evidence paths

- `case-study/shapes/soc2-shapes.ttl`
- `case-study/shapes/shape-violations.json`
- `case-study/pddl/solace-case-study.ttl` (the validated instance data)
