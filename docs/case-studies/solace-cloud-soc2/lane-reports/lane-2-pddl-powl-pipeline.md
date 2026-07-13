# Lane 2 — PDDL Merge/Plan and Hierarchical POWL Projection

Status: DONE.

## Commands run this session

```
just cng-test-lib-isolated soc2-2 bench::soc2 -- --nocapture
```

Result: `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 174 filtered out`.

## What the real chain produced (from `case-study/pddl-out/`)

- `import_artifacts` admits the 10 phase fragments + 1 problem fragment (11 `.ttl` artifacts;
  `solace-case-study.ttl` contributes no fragments — it carries no `ceng:pddlDomain`/
  `#pddlProblem` literal).
- `generate_plan` merges structurally and finds a 30-step plan (3 actions × 10 phases), forced
  linear by the precondition chain:
  - `tape.ops[0].label == "define-system-boundary(solace)"`
  - `tape.ops[29].label == "confirm-evidence-bundle-complete(solace)"`
  (`case-study/pddl-out/plan.json`)
- `hierarchical_projection` groups the plan into exactly 10 phase children — one per contributing
  fixture artifact, in engagement order (`case-study/powl/phase_sources.json`, 10 entries).
- `powl_to_turtle` exports deterministic Turtle
  (`case-study/powl/solace-soc2-powl.ttl`); its BLAKE3 digest
  (`case-study/powl/powl-digest.txt`) was re-derived from a second independent run this session
  and matched byte-for-byte, confirming replay determinism.
- `validate_powl_store` accepts the exported Turtle against the POWL structural shape.

## 8-constraint split law

`crate::bench::togaf::verify_eight_constraint_split` — reused verbatim from the TOGAF case study
(it is generic over `AdmittedSurface`, never TOGAF-specific) — holds over the real merged
surface, and two adversarial mutations (9 precondition conjuncts on one action; 9 goal conjuncts
on the merged problem) both refuse `CNG_R05 UnsupportedConstruct` naming the violation.
`soc2_test.rs::eight_constraint_split_violations_refuse_typed`.

## Evidence paths

- `case-study/pddl-out/domain.json`, `problem.json`, `plan.json`
- `case-study/powl/solace-soc2-powl.ttl`, `phase_sources.json`, `powl-digest.txt`
