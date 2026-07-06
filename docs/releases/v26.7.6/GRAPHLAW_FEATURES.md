# GraphLaw feature matrix — v26.7.6

`praxis-graphlaw` (the roxi fork) is the live default `GraphEngine` inside
`crates/ggen` as of this release. The seam is
`crates/ggen/src/graph.rs::GraphEngine` — engine-neutral owned results only
(`EngineQueryResults`, `ShaclOutcome`, `MaterializeOutcome`); no oxrdf/oxigraph
model type crosses it. `GraphLawStore` layers the GraphLaw reasoner (law state:
rules, SHACL/ShEx, denials) over a `DeterministicGraph` mirror (queryable
state: SPARQL 1.1, canonical BLAKE3 hashing). Every derived fact enters the
mirror exclusively through `GraphLawStore::materialize` — i.e. through the
GraphLaw reasoner. `state_hash` is BLAKE3 over canonically sorted quads,
computed ggen-side identically for both engines
(`graph::tests::graphlaw_state_hash_matches_oxigraph_for_same_facts`).

Every sync gate is optional-when-unconfigured: without a `[law]` table in
`ggen.toml`, no law stage runs and the two engines agree byte-for-byte
(`tests/graphlaw_e2e.rs::engines_agree_when_no_law_configured`).

| Capability | Status | Evidence (test / file) | Next action |
|---|---|---|---|
| RDF fact store (Turtle in, canonical N-Quads out) | Implemented | `crates/ggen/tests/sync_e2e.rs` (whole suite runs on GraphLaw default); `graph.rs::GraphLawStore::insert_turtle` | — |
| SPARQL SELECT / ASK / CONSTRUCT in templates | Implemented (via mirror) | `crates/ggen/src/template.rs::tests` (sparql/ask/construct); `tests/graphlaw_e2e.rs::engines_agree_when_no_law_configured` | Route SELECT/ASK through GraphLaw's native `sparql.rs` once it covers SPARQL 1.1 (`FILTER`/`OPTIONAL` subset today) |
| N3 rule materialization (forward chaining, `ggen.toml [law].rules`) | Implemented | `tests/graphlaw_e2e.rs::when_guard_passes_only_after_n3_materialization` (refuses on oxigraph, renders on GraphLaw — the reasoner-in-the-loop proof); `graph::tests::graphlaw_materialize_derives_facts_visible_to_sparql` | Iterate Enrich `construct:` + materialize to a joint fixed point (today: constructs, then one materialize pass) |
| Datalog (stratified negation, aggregates) | Partial | Engine support in `crates/praxis-graphlaw/src/datalog.rs` (`validate_rules` strata run in `TripleStore::add_rules`); no ggen fixture exercises negation/aggregates yet | Add a stratified-negation e2e fixture under `crates/ggen/tests` |
| SHACL gate (`[law].shapes`, pre-render, refusal names focus nodes) | Implemented | `tests/graphlaw_e2e.rs::shacl_violation_refuses_sync_naming_focus_node` (`FM-LAW-013`); `graph::tests::graphlaw_validate_shacl_flags_focus_node` | — |
| ShEx validation (ShExC) | Partial | `GraphEngine::validate_shex` wired (`graph.rs`, delegates to `validate_shex_c`); no ggen.toml surface or sync gate calls it yet | Add `[law].shex` config + gate once a consumer needs shape-map validation |
| Denial rules (`{ body } => false.`, post-materialization consistency) | Implemented | `tests/graphlaw_e2e.rs::denial_violation_refuses_sync` (`FM-LAW-011`); `graph::tests::graphlaw_check_denials_reports_violation` | — |
| Standing derivation (who may act, derived from rules) | Partial | Mechanically expressible today as N3 rules + `when:` ASK guards (exactly the materialization e2e pattern); no standing-specific vocabulary or fixture | Author the standing rule pack against `docs/v26.7.4/PUBLIC_ONTOLOGY_MAPPING.md` vocabularies |
| Explanation (which rules fired / what was derived) | Partial (counts + derived-triple diff) | `ggen law explain` (`verbs/handlers.rs::handle_law_explain`): rules-loaded count, per-file rule counts, full derived-triple diff | Per-derivation provenance (rule → derived triple traces) requires reasoner instrumentation in `praxis-graphlaw/src/reasoner` |
| Planner export (materialized graph → downstream planner) | Partial | `ggen law export` dumps canonical N-Triples + BLAKE3 state hash (`handle_law_export`); no PDDL projection | Consume from praxis-planner (Phase 3) via the N-Triples dump |
| Receipt ingest (law stage bound into the sync receipt) | Implemented | Rule/shape files are hashed into the receipt input closure (`sync.rs` law stage); `tests/graphlaw_e2e.rs::two_runs_same_fixture_same_graph_hash_and_valid_chain` (identical payload, verified chain) | — |
| CLI surface (`ggen law load/validate/derive/explain/export`) | Implemented | Generated route `crates/ggen/src/verbs/law.rs` from `schema/praxis.ttl` `praxis:CmdGgenLaw*` instances; handlers in `verbs/handlers.rs` | — |
| Backward chaining (`prove`/`solve`) | Missing (not surfaced) | Engine support exists (`praxis-graphlaw/src/backwardchaining.rs`); no `GraphEngine` method or verb exposes it | Surface as `ggen law prove` when a goal-directed consumer exists |

Refusal codes introduced: `FM-LAW-001..003` (law op on non-law engine),
`FM-LAW-004` (poisoned law lock), `FM-LAW-005/006` (fact/rule load),
`FM-LAW-007` (underivable derived-fact export), `FM-LAW-008/009`
(SHACL/ShEx schema parse), `FM-LAW-010/012` (unreadable rule/shape file),
`FM-LAW-011` (denial violated), `FM-LAW-013` (SHACL non-conformance).
All are typed `AppError` variants via `AppError::fm_law`
(`crates/ggen/src/error.rs`); none default silently.
