This case study uses the fleet itself as the subject. cargo-cicd emits standing evidence; praxis-graphlaw adjudicates that evidence; PDDL/POWL plan and model repair/validation flow; OCEL records execution; wasm4pm validates conformance; Autonomic Platform displays the verdict.

## Scope

Local-first autonomic release-governance for the seanchatmangpt fleet. See
`PRODUCTION_READINESS.md` for the declared scope statement, the 15
acceptance criteria, and the 8 non-goals this case study does not claim.

## Roles

| Role | Holder | Function |
|---|---|---|
| Emitter | `cargo-cicd` | Compiles fleet artifact state into a standing evidence document (JSON + Turtle + OCEL) |
| Judge | `praxis-graphlaw` | Validates that evidence's shape (SHACL/ShEx) and structure (N3/Datalog), derives readiness facts |
| Validator | `wasm4pm` | Validates process conformance of the recorded execution (OCEL) against a POWL model |
| Display | Autonomic Platform | Renders the judged, sourced standing — never computes its own verdict |

## Lane 1 findings (cargo-cicd front door + standing compiler)

Full detail: `lane-reports/lane-1-cargo-cicd.md`. Summary, verified against
the real files in `/Users/sac/praxis/target/praxis-standing/` and
`/Users/sac/cargo-cicd/target/praxis-standing/` as they exist on disk at the
time this lane started (this lane did not regenerate them):

- `standing.json` — schema_id `cicd-standing.v1`, `release_id: "v26.6.30"`,
  28 artifacts (12 `rust_crate`, plus `Doc`/`Workflow`/`Bench`/`Client`
  kinds), each carrying a `standing` array (e.g. `["DISCOVERED"]`),
  `ladder_level`, `evidence` array (artifact-hash or command evidence), and
  an (empty, in this run) `external_operator_side_effects` array.
- `standing.ttl` — `@prefix praxis: <https://praxis.dev/ontology/standing#>`,
  one `praxis:StandingDocument` root
  (`praxis:StandingDocument-v26_6_30`) with `praxis:releaseId` and
  `praxis:generator`, and one `praxis:StandingArtifact` individual per
  artifact (`praxis:id`, `praxis:kind`, `praxis:path`, `praxis:standing`,
  `praxis:ladderLevel`, `praxis:evidence` — the evidence literal is a
  JSON-encoded string, not a nested graph). This is the exact namespace
  this lane's `graphlaw_judgment.ttl` and shapes reuse — no second standing
  vocabulary was invented.
- `standing.ocel.json` — Shape-A OCEL (`eventTypes`/`events`/`objectTypes`/
  `objects`), one `standing_compiled` event per artifact, each with an
  `artifact` relationship to a matching object. Parses with the real
  `wasm4pm_compat::ocel::OCEL` type (Lane 1's regression test
  `standing_ocel_shape_a_parses_as_wasm4pm_compat_ocel` proves this).
- Determinism: two consecutive `standing refresh` runs under praxis's real
  `doctor_command` config produced byte-identical `standing.ttl`
  (`sha256 4127bda9...` both times) after Lane 1's Command-evidence TTL fix
  — the only field that legitimately changes between runs is
  `standing.json`'s `generated_at_utc` sidecar timestamp, which this lane's
  graph seed treats as a sourced literal, never as a computed "now".

## Lane 2 (this lane): GraphLaw judgment model

Builds the RDF seed graph, SHACL shapes, ShEx schema, and N3/Datalog rules
that let `praxis-graphlaw` adjudicate the Lane 1 evidence, plus the judge
binary (`src/bin/case_study_judge.rs`) that merges the seed graph with the
live `target/praxis-standing/standing.ttl`, materializes, checks denials,
validates SHACL/ShEx, and reads the resulting verdict fact via SPARQL. See
`lane-reports/lane-2-graphlaw.md` for the full account, including the real
SHACL/ShEx conformance numbers and denial results from that run, and
`case-study/final_graphlaw_verdict.json` for the machine-readable verdict
this document must never contradict.

The verdict is derived, not asserted: `case_study_judge.rs` reads whichever
of `praxis:ProductionReadyForDeclaredScope`,
`praxis:PilotReadyWithExternalSideEffects`, or
`praxis:NotReadyWithReasons` SPARQL finds attached to the case-study subject
after `materialize()` reaches fixpoint. No lane, including this one, hand-picks
the verdict string.

## Lanes 3-7 (not yet run)

PDDL repair planning, POWL process modeling, OCEL v2 capture, wasm4pm
process validation, Autonomic Platform display, evidence manifest/claim
promotion, and the independent Integration Gate Audit are out of this
lane's scope. `case-study/graphlaw_judgment.ttl` seeds honest
not-yet-produced placeholder nodes for each of their outputs (see that
file's `## Not-yet-produced` section) so this lane's SHACL/ShEx/N3 model
does not silently assume evidence that does not exist yet.
