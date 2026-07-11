# PROJ-726 — arazzo-pack — graph to arazzo/openapi/asyncapi YAML

Status: ALIVE — evidenced this session (uncommitted; HEAD `1f3f9bc`, Phase 6 commit not run)

Track: E (multi-engine execution).
Milestone: v26.7.10-revised (No-LLM Multi-Actor Planning + Multi-Engine Execution).
Governing doctrine: `docs/releases/v26.7.10/DEFINITION_OF_DONE.md` (PROJ-730);
plan of record: the approved v26.7.10-revised plan. Control surface:
`docs/releases/v26.7.10/RELEASE_CONTROL.md` (v26.7.10-revised scope section).

New `packs/arazzo-pack/` (ggen Tera, otel-weaver-pack precedent) renders `arazzo.yaml` plus
per-engine `openapi.yaml`/`asyncapi.yaml` from the graph; cng re-verifies
`digest(render(graph))`, never re-admits YAML. The OpenAPI/AsyncAPI documents are the
generated, digest-recorded declared contract for the (UNVERIFIED) HTTP binding — DoD §20
item 1. Gate: G12.

## Evidence (this session)

`packs/arazzo-pack/{pack.toml,ontology.ttl,templates/arazzo.yaml.tmpl,
templates/engine-openapi.yaml.tmpl,templates/engine-asyncapi.yaml.tmpl}` on disk (authored
before this session). Registered in `ggen.toml [packs]` this session (PROJ-744) and verified
via an isolated scratch ggen project: `ggen sync run` twice, byte-identical output
(`arazzo.yaml`, `engine-openapi.yaml`, `engine-asyncapi.yaml`), receipt digests matched
recomputed BLAKE3 hashes exactly — this was NOT run against the live repo's own `ggen.toml`/
receipt state (see PROJ-745 for the `digest(render(graph))` Rust seam and its own honest
gap).
