# arazzo-pack

ggen pack (PROJ-726, v26.7.10-revised) that projects the Arazzo-as-RDF graph into an
Arazzo 1.1.0 YAML document and the per-engine OpenAPI 3.1 / AsyncAPI 3.0 capability
documents for multi-engine execution. RDF stays authoritative; every YAML file here is a
projection artifact, never a source of truth and never re-admitted.

## What renders, from what graph

| Template | Output (`to:`) | SPARQL queries |
|----------|----------------|----------------|
| `templates/arazzo.yaml.tmpl` | `generated/arazzo.yaml` | `a_info`, `b_sources`, `c_workflows`, `d_wf_depends`, `e_steps`, `f_parameters`, `g_criteria`, `h_on_success`, `i_on_failure`, `j_action_criteria`, `k_step_outputs`, `l_wf_outputs` |
| `templates/engine-openapi.yaml.tmpl` | `generated/engine-openapi.yaml` | `a_engines` |
| `templates/engine-asyncapi.yaml.tmpl` | `generated/engine-asyncapi.yaml` | `a_engines` |

`ontology.ttl` is this pack's own content: the `arzeng:` engine capability contract
(coordinator C, helper H, main M) — a documented bridge vocabulary pending real
engine-description triples from PROJ-722/723. Two external sources are unioned into the
sync graph via this pack's `ggen.toml` `extra_ontologies` entry (`[packs].arazzo-pack`),
not committed into this pack:

1. `crates/cng/ontologies/arazzo.ttl` — the `arz:` Arazzo 1.1.0 80/20 vocabulary.
2. `crates/cng/examples/arazzo-api-orchestration.ttl` — workflow instance data.

This replaces the pack's former `make-ontology.sh` committed-union convention (deleted):
ggen now unions declared `extra_ontologies` after the pack's own `ontology.ttl` at sync
time, so the pack graph can no longer drift from its external sources.

## Design choices

1. Single OpenAPI/AsyncAPI documents enumerating all engines (as `servers` entries),
   because a template's `to:` is one fixed output path — the simpler deterministic option
   over per-engine output files.
2. Request/message bodies are described by reference to
   `crates/cng/shapes/dispatch-shapes.ttl` (`text/turtle`, `type: string`), not re-authored
   as JSON Schema.
3. Optional graph values are normalized with SPARQL `COALESCE` to `""` so template row
   access never hits an unbound variable; empty string means "omit the YAML key".

## Determinism claim

Every template sets `determinism: true` (ggen double-renders and byte-compares), and every
SPARQL SELECT carries `ORDER BY` over all projected variables. No wall clock, no
randomness, no filesystem-order dependence. Verified this session by running
`ggen sync run` twice in a scratch project and diffing `shasum -a 256` of all three
outputs — byte-identical (see PROJ-726 ticket evidence for the exact digests).

## Honest boundary

The OpenAPI/AsyncAPI documents are the DECLARED capability/event contract of the Chatman
Engine processes (`submitWorkflow`, `getExecutionEvidence`, `quiesce`;
`workflowAcknowledged`, `workflowResultProduced`). The transport binding implemented this
increment is filesystem (`engines/<id>/{inbox,outbox}` Turtle exchange, plan decision 6):
declared-contract mechanism ALIVE (rendered from the graph, digest-recorded); HTTP/broker
binding UNVERIFIED and intentionally absent. Engine instances in `ontology.ttl` are
hand-authored declarations, not runtime-derived facts.

## Downstream verification seam (Rust wiring is a separate ticket)

cng must verify `digest(render(graph))`: recompute BLAKE3 over the rendered YAML bytes and
compare against the digest recorded in the ggen receipt (`.ggen-v2/receipt.json`
per-output digests) before dispatching — never re-admit or re-parse the YAML as truth.
That Rust wiring (digest check + `ARAZZO_RENDERED` state advance) is named here as the
seam only; it is not part of this pack.

## See Also

- `packs/otel-weaver-pack/` — frontmatter/Tera precedent this pack follows.
- `packs/togaf-adm-pack/` — `extra_ontologies` convention precedent this pack now follows
  (own content in `ontology.ttl`; external sources declared in `ggen.toml`, not unioned in).
- `crates/cng/ontologies/arazzo.ttl` and `crates/cng/shapes/arazzo-shapes.ttl` —
  authoritative vocabulary and closed shapes (refused 1.1 features listed there).
