//! cng library surface: the μ pipeline (import → merge → plan → project →
//! serialize) behind the noun-verb CLI, exposed so integration tests can
//! drive the exact code the binary runs.

#[cfg(feature = "bench")]
pub mod bench;
/// jira-tracking-pack: `jira` noun-verb command handlers over ticket data
/// parsed (by `packs/jira-tracking-pack/make-ontology.py`) from real
/// `docs/jira/*/tickets/index.md` markdown tables. Depends only on the
/// hard `oxigraph` dependency, so it is unconditional like
/// `otel_ocel`/`otel_rdf` themselves; `crates/cng/src/jira_routes.rs`
/// (`ggen sync`-generated) is its only caller.
pub mod jira;
/// PROJ-766: Rail G measurement-profile schema + per-workflow-family
/// execution measure `mu_x`, populating `G_RESULT`. Depends only on
/// `otel_ocel` and the hard `oxigraph` dependency, so it is unconditional
/// like `otel_ocel`/`otel_rdf` themselves.
pub mod measurement;
/// PROJ-764: derives `urn:graph:ocel` from `urn:graph:otel` via SPARQL
/// CONSTRUCT (closes the second half of gap G11, `docs/otel-rdf-handoff.md`)
/// and names the mission's 5-layer named-graph separation. Depends only on
/// `otel_rdf` and the hard `oxigraph` dependency, so it is unconditional
/// like `otel_rdf` itself.
pub mod otel_ocel;
/// PROJ-763: maps Weaver-admitted OTLP spans into the named RDF graph
/// `urn:graph:otel` (closes gap G11, `docs/otel-rdf-handoff.md`). Depends
/// only on `telemetry_gen` (five `ATTR_*` constants) and the hard `oxigraph`
/// dependency, so — unlike `bench`/`otel-live` — it is unconditional: no new
/// optional dependency is introduced by this module.
pub mod otel_rdf;
/// PROJ-765: PROV-O transformation ancestry + digest-chain receipt for the
/// `G_OTEL -> G_OCEL` CONSTRUCT projection, populating `G_RECEIPT`. Depends
/// only on `otel_ocel` and the hard `oxigraph`/`blake3` dependencies, so it
/// is unconditional like `otel_ocel`/`otel_rdf` themselves.
pub mod otel_receipt;
pub mod pipeline;
/// Increment 2 approval-seam backend: `plan present` / `plan check` /
/// `plan step` (`crates/cng/src/main.rs`). See the module's own doc for the
/// disclosed deviation from the original design (a locally reimplemented
/// plan digest, not a `praxis-graphlaw`/`bench`-gated call) and why the
/// module is therefore unconditional rather than `bench`-gated like
/// `plan decompose`.
pub mod plan_approval;
pub mod powl;
#[cfg(feature = "runner")]
pub mod runner;
pub mod shape;
/// Generated Weaver semantic-convention bindings (ggen; see file header).
///
/// Unconditional as of PROJ-763: the module itself has zero external
/// dependencies (plain structs and `&'static str` constants) even though
/// its original sole consumer (`src/bin/otel-live.rs`) is gated behind the
/// optional `otel-live` feature. `otel_rdf`'s in-process admission check
/// needs `ATTR_*`/`REGISTRY_GROUP_ID` as the single source of truth for the
/// same five-attribute contract the external Weaver live-check enforces, so
/// the module can no longer be feature-gated away from the default build.
pub mod telemetry_gen;
