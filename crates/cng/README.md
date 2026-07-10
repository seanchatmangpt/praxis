# cng — Chatman Engine noun-verb CLI

Human-facing artifact handle for the PDDL → POWL v2 manufacture boundary:
many admitted PDDL Turtle planning artifacts (`*.domain.ttl`, `*.problem.ttl`)
become one POWL v2 Turtle workflow artifact. In Chatman Equation terms
`A = μ(O*)`: the artifact set is `O*`, this pipeline is `μ`, the exported
workflow Turtle plus evidence is `A`, and the manifest/digests are `R`.

This is not a converter. It is a standing-preserving graph-manufacture
boundary: semantic authority stays in the imported/exported RDF artifacts,
the public ontology mapping, provenance links, the declared validation
shape, and the manifest — never in a private in-memory representation.
Operational witnesses (`Pddl8Tape`, the `Powl` value) exist, but they are
witnesses, not truth. The release is finished only when the generated POWL
TTL is not trusted because `cng` printed it, but because the imported
artifacts, ontology mapping, validation, runner result, digest, and
manifest close.

## Release theorem

For any admitted set of RDF/Turtle PDDL artifacts, `cng` either manufactures
one valid RDF/Turtle POWL v2 workflow artifact — with stable provenance,
deterministic digest, structural validation evidence, and runner/conformance
proof — or emits a typed refusal with an exact code and evidence path. There
is no third state: no silent fallback, no generated placeholder, no
hand-authored POWL, no private semantic authority.

## Command surface

```bash
cng plan import --dir plans/joseph      # validate + list importable artifacts
cng plan admit --dir plans/joseph       # parse + structural merge, no planning
cng plan generate --dir plans/joseph    # merge fragments, plan once, print plan id
cng workflow project --dir plans/joseph # project the plan into POWL v2 Turtle
cng workflow export --dir plans/joseph --out joseph.powl.ttl   # + shape validation
cng workflow inspect --file joseph.powl.ttl                    # parse + shape-validate
cng workflow validate --dir plans/joseph   # run on the bcinr-powl runtime
cng workflow evidence --dir plans/joseph --out joseph.powl.ttl # full proof chain + manifest
cng workflow doctor                     # toolchain + self-checks
```

A complete many-to-one example ships in `plans/joseph/`: Joseph's national
famine-management plan as 13 phase artifacts (26 `*.domain.ttl` /
`*.problem.ttl` files) that manufacture one 20-step national workflow.

## Public ontology mapping (RDF PDDL → POWL v2)

| Source (imported artifacts)                     | Generated POWL v2                                             |
|-------------------------------------------------|---------------------------------------------------------------|
| admitted artifact set                            | one `powl2:Model` root (`<base>/n0`)                          |
| combined plan (total order)                      | one `powl2:PartialOrder`                                      |
| plan op i (ground action label)                  | `powl2:ActivityLeaf` at `<base>/n0/c<i>` with `activityLabel` |
| plan position i                                  | `powl2:ChildBinding` at `<base>/n0/binding/<i>` (`childIndex`, `childModel`) |
| plan order, transitively closed ((i,j), i<j)     | `powl2:precedes` between the two bindings                     |
| composed plan source IRI                         | `powl2:derivedFrom` on the root (exactly one)                 |
| contributing artifact (content-addressed)        | `prov:wasDerivedFrom <urn:blake3:...>` on each ActivityLeaf   |

Source identity is content-addressed: every imported artifact is identified
by `urn:blake3:<hex of its bytes>`, so per-element provenance is stable
across machines and the manifest binds digests to paths.

## Validation shape

Generated POWL must parse as RDF/Turtle AND satisfy the declared shape
(`shapes/powl2-shapes.ttl`, shipped with the crate): exactly one Model;
every ChildBinding carries exactly one childIndex and childModel; every
ActivityLeaf carries one non-empty activityLabel; every `precedes` edge
connects two ChildBindings; root provenance is exactly one `derivedFrom`.
The executable form is `cng::shape::validate_powl_store` (SPARQL structural
validator); `workflow inspect`/`export`/`evidence` all run it.

## Typed refusal algebra

| Code      | Refusal               | Trigger                                            |
|-----------|-----------------------|----------------------------------------------------|
| `CNG_R01` | MalformedTtl          | invalid Turtle, unparseable PDDL literal           |
| `CNG_R02` | MissingDomain         | no domain fragment in the admitted set             |
| `CNG_R03` | MissingProblem        | no problem fragment in the admitted set            |
| `CNG_R04` | PlanUnsolvable        | no plan (empty tape, unreachable goal)             |
| `CNG_R05` | UnsupportedConstruct  | name mismatch, duplicate actions, branching, >64 ops |
| `CNG_R06` | InvalidPowl           | POWL parse or shape violation                      |
| `CNG_R07` | RunnerMismatch        | runner refusal or non-conformant execution         |
| `CNG_R08` | Nondeterminism        | repeated manufacture produced different bytes      |
| `CNG_R09` | HardcodingSuspicion   | output detached from the admitted plan             |
| `CNG_R10` | IoRefused             | filesystem input/output refused                    |

Negative fixtures under `tests/fixtures/negative/` prove `CNG_R01/R04/R05/R06`
from real artifacts; `tests/no_inline_ttl_guard.rs` statically enforces the
artifact boundary (no inline Turtle/PDDL payloads in Rust sources).

## Runner / conformance

`workflow validate` and `workflow evidence` lower the projected model to
`bcinr_powl::compiler::PowlAstNode`, admit it via `compile_powl` (Kahn
acyclicity + reachability), execute it with the branchless `scheduler_tick`
loop, and check per tick that no activity fires before its projected
predecessors — the generated workflow is accepted as a conformance artifact,
not merely executed. Verdicts are computed from the fired bitmasks, never
asserted.

## Evidence manifest

`workflow evidence` writes `<out>.manifest.json` binding: run id (BLAKE3
over input digests + output digest — deterministic), command, status,
timestamp (metadata only, excluded from all digests), every imported path +
digest, plan id, POWL path + digest, validation result, runner result, and
fixture seed. It prints the marker lines `IMPORTED_PDDL_TTL_PATHS=`,
`GENERATED_PLAN_ID=`, `GENERATED_POWL_TTL_PATH=`, `POWL_DIGEST=`,
`VALIDATION_RESULT=`, `RUNNER_RESULT=`, `PDDL_FIXTURE_SEED=`,
`EVIDENCE_MANIFEST_PATH=`.

## Dry-run publishability (v26.9.10)

Registry-only dependencies (clap-noun-verb 26.7.4, bcinr-pddl 26.6.26,
bcinr-powl 26.6.25, oxigraph 0.5, blake3 1, serde/serde_json 1) so the full
proof chain runs from the package surface:

```bash
just fmt-check                 # format
just cng-test                  # unit + integration + negatives + boundary guard
just cng-run workflow evidence --dir crates/cng/plans/joseph --out target/chatman/powl/joseph.powl.ttl
just cng-install-smoke         # cargo install from the crate, run the installed binary
just publish-dry-run cng       # cargo publish --dry-run (package + verify build)
```

## Product thesis

RDF removes the private technical box: both ends of the transformation are
open, parseable, queryable graphs. The public ontology (powl2 + PROV)
removes the private semantic box: every generated element is named in
shared vocabulary, not tool-internal structs. `cng` as μ removes the
bureaucratic translation box: the workflow is manufactured directly from
admitted planning artifacts instead of being re-authored by hand at every
policy → process → software boundary.

## Limitations

- Projection emits linear `PartialOrder` chains only; branching PDDL/POWL
  (Choice, loops, parallel orders) refuses through the CLI with `CNG_R05`.
- The `runner` feature (default) pulls bcinr-powl, which requires a nightly
  toolchain; stable consumers build with `--no-default-features` (runner
  verbs are then absent; everything else works).
- bcinr-powl's tape holds ≤ 64 ops; larger plans refuse with `CNG_R05`.
- Root `powl2:derivedFrom` carries one composed-plan IRI; per-element
  provenance is the `prov:wasDerivedFrom` layer.
