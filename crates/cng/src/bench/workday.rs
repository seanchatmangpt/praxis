//! Single-operator workday benchmark (PROJ-608/610/611): a roster of ONE
//! operator executes a deterministic, logical-tick day through the real cng
//! manufacture chain. Each tick: select/admit one category workload
//! artifact set → derive the standing role (Mycin + Datalog) → manufacture
//! via `manufacture_set` → execute/validate → receipt. Every tick the
//! standing-next-action.rq SELECT over the observation store must derive
//! EXACTLY one lawful next action while work remains, or the run refuses
//! `CNG_R12 StandingAmbiguous`.
//!
//! Bounded admission → resume (PROJ-611, workday mode only): when a tick's
//! manufacture refuses (the seeded `refusal_per_mille` injection withholds
//! the final problem fragment), the loop manufactures an
//! `ex:AdmissionRequest` artifact naming exactly the minimal missing
//! admission, then deterministically synthesizes that minimal admission
//! fragment (seed-derived — MOCKED-HUMAN: the admission MECHANISM is real
//! and receipted, the granting human is simulated by the benchmark), admits
//! it, and resumes the interrupted workflow at tick+1. The Fortune-5 path
//! (`run()`) keeps its terminal refusals unchanged.
//!
//! Hook actuation (PROJ-612) and Dialect Registry gate (PROJ-613): at
//! workday start the `WorkdayHookBroker` validates the Dialect Registry
//! against its closed shape (`CNG_R14 DialectRegistryRefused` BEFORE any
//! tick) and admits `hooks/workday-pack.ttl`; each hook's HookStanding
//! ladder (REGISTERED→ADMITTED→AUTHORIZED→READY at admission,
//! EXECUTED→RECEIPTED at its first firing) is emitted as `hook_standing`
//! observations. Every executed transition is actuated through its
//! category hook and must yield a `HookReceipt` (`CNG_R13
//! UnreceiptedActuation` otherwise); the receipt lands as a `hook_receipt`
//! observation carrying `ex:hookDeltaHash`. REPLAYABLE (order 7) is
//! emitted only after the end-of-day producer replay verification
//! re-manufactures every tick set byte-identically (PROJ-614/622); the
//! independent auditor pass (`workday_verify`, PROJ-616) re-checks the
//! whole bundle from files alone.
//!
//! Determinism: ticks are logic counters (splitmix64-seeded), never wall
//! clock; nothing time-based enters any digest; two same-seed runs are
//! byte-identical across observations and evidence digests (workflow ids
//! are tick-derived, never path-derived).
//!
//! External dispatch (PROJ-618/619/620/621): the broker choke point
//! (`actuate_transitions`) additionally routes external-class categories
//! (see `dispatch::route_category` for the deterministic rule) through the
//! `DispatchAdapter` loopback surface — contract out to `dispatch/outbox/`,
//! consequence back through `dispatch/inbox/` and the staged lawful
//! re-entry pipeline; `api-orchestration` ticks run the admitted Arazzo
//! description through the same adapter (one dispatch per `arz:Step`, in
//! dependsOn order). Both outbound dispatch and inbound consequence are
//! receipted as observations and their digests fold into the chain.
//!
//! Evidence chain composition (`evidence_chain_digest`): one BLAKE3 fold
//! over (1) the per-tick POWL digests in tick-id (BTreeMap) order, then
//! (2) the per-dispatch (contract digest, consequence digest) pairs in
//! dispatch-id (BTreeMap) order, then (3) the run-level graphlaw
//! `hook_hash` over every HookVerdictRecord of the day in actuation order.
//! All inputs are content-derived; no wall clock, no path-derived values.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;

use crate::powl::CngRefusal;

use super::arazzo::{default_description_path, run_arazzo_projection};
use super::dispatch::{
    route_category, workday_contract, DispatchAdapter, DispatchOutcome, ExecutionClass,
    SynthesisMode,
};
use super::generate::write_set;
use super::hooks::WorkdayHookBroker;
use super::manufacture::{manufacture_set, SetOutcome};
use super::roles::{derive_roles_datalog, roster_workers, run_construct, select_rows, ObsWriter};
use super::run::{emit_record_observations, evidence_digest, obs_dir_digest, OCEL_CONSTRUCT_STEMS};
use super::templates::{load_templates, QuerySet, Templates};
use super::{fill_template, splitmix64, CATEGORIES, RWAI_PREFIX};

/// The dreg: `DialectEntry` local name of the dialect the workday hook
/// packs execute — every pack hook is kind "delta" (see
/// `hooks/workday-pack.ttl`). Emitted as `ex:dialect dreg:delta` on every
/// hook observation (PROJ-622 vocab gap (e)); the GRAPHLAW_DIALECT_CLOSURE
/// marker checks that every observed dialect is a registered entry.
const WORKDAY_HOOK_DIALECT: &str = "delta";

/// Marker query file stem → the v26.7.10 success markers it proves
/// (PROJ-622/727). Each `queries/markers/<stem>.rq` returns one row whose
/// `?value` is 0 iff the marker holds; `V26_7_10_PRODUCTION_READY` is the
/// conjunction of all sixteen named markers and carries no query of its
/// own. Every stem here is a UNIVERSAL (=0) law that holds — vacuously
/// where its obs kinds are absent — on a lawful single-operator workday;
/// the INVERTED existence markers live in [`DISTRIBUTED_MARKER_MAP`]
/// instead (adding them here would refuse every single-engine run).
const MARKER_MAP: [(&str, &[&str]); 10] = [
    (
        "marker-autonomic-loop",
        &[
            "AUTONOMIC_LOOP_CLOSED",
            "ONE_PERSON_RECURSIVE_WORKFLOW_PROVEN",
        ],
    ),
    ("marker-child-closure", &["RECURSIVE_CHILD_CLOSURE_PROVEN"]),
    ("marker-dialect-closure", &["GRAPHLAW_DIALECT_CLOSURE"]),
    (
        "marker-external-dispatch",
        &[
            "EXTERNAL_WORKFLOW_DISPATCH_PROVEN",
            "EXTERNAL_RESULT_READMISSION_PROVEN",
        ],
    ),
    (
        "marker-hook-actuation",
        &["HOOK_ACTUATION_PROVEN", "ZERO_UNRECEIPTED_ACTUATION"],
    ),
    (
        "marker-timeout-escalation",
        &["TIMEOUT_ESCALATION_PROVEN", "COMPENSATION_WORKFLOW_PROVEN"],
    ),
    // PROJ-727 distributed-evidence universal markers. On a single-operator
    // workday the isolation/remote/divergence laws hold VACUOUSLY (no such
    // obs kinds); the arazzo pair law is exercised by every dispatch.
    (
        "marker-engine-isolation",
        &[
            "SHARED_MEMORY_CROSSINGS_ZERO",
            "DIRECT_ENGINE_BYPASSES_ZERO",
        ],
    ),
    (
        "marker-remote-execution",
        &[
            "REMOTE_WORKFLOWS_ACKNOWLEDGED",
            "REMOTE_WORKFLOWS_COMPLETED",
        ],
    ),
    ("marker-replay-divergence", &["REPLAY_DIVERGENCES_ZERO"]),
    ("marker-arazzo-dispatch", &["ARAZZO_WORKFLOWS_DISPATCHED"]),
];

/// Marker map for DISTRIBUTED (multi-engine) runs only (PROJ-727):
/// the universal =0 laws from [`MARKER_MAP`]'s distributed rows plus the
/// INVERTED existence markers, whose queries return 0 when the existential
/// holds (e.g. `IF(?engines >= 2, 0, 1)`) — see each query header for the
/// inversion note. Evaluated by the multi-engine coordinator
/// (`engine_collect_remote`) over the coordinator ∪ engine-bundle
/// observation union; never by the single-operator workday (a lawful
/// single-engine run would refuse the existence markers).
pub(super) const DISTRIBUTED_MARKER_MAP: [(&str, &[&str]); 6] = [
    (
        "marker-engine-isolation",
        &[
            "SHARED_MEMORY_CROSSINGS_ZERO",
            "DIRECT_ENGINE_BYPASSES_ZERO",
        ],
    ),
    (
        "marker-remote-execution",
        &[
            "REMOTE_WORKFLOWS_ACKNOWLEDGED",
            "REMOTE_WORKFLOWS_COMPLETED",
        ],
    ),
    ("marker-replay-divergence", &["REPLAY_DIVERGENCES_ZERO"]),
    ("marker-arazzo-dispatch", &["ARAZZO_WORKFLOWS_DISPATCHED"]),
    // Inverted existence markers (0 = the existential holds).
    (
        "marker-multi-engine-execution",
        &["MULTI_ENGINE_EXECUTION_PROVEN", "ENGINE_INSTANCES_PROVEN"],
    ),
    (
        "marker-arazzo-inter-engine",
        &["ARAZZO_INTER_ENGINE_WORKFLOW_PROVEN"],
    ),
];

/// Marker map for the PLANNING surface (PROJ-739/740): the six no-LLM
/// decomposition proof markers plus the three structural/absence markers,
/// evaluated over a dedicated `decomposition-result.ttl` evidence graph
/// (`build_decomp_marker_store`) from a real `cng plan decompose` run
/// (`bench::decomp::decompose`/`decompose_with`, PROJ-741) — never the
/// obs/evidence union `build_marker_store` loads, since a plain
/// single-operator workday never produces `decomp:` facts and a plain
/// `cng plan decompose` run never produces `obs:` facts. This mirrors the
/// [`DISTRIBUTED_MARKER_MAP`] precedent: a marker family that owns its own
/// store construction rather than being folded into `MARKER_MAP`.
pub(super) const PLANNING_MARKER_MAP: [(&str, &[&str]); 7] = [
    (
        "marker-decomposition-derived",
        &["DECOMPOSITION_DERIVED_PROVEN"],
    ),
    (
        "marker-decomposition-receipted",
        &["DECOMPOSITION_CANDIDATES_RECEIPTED"],
    ),
    (
        "marker-decomposition-interface-state",
        &["INTERFACE_STATE_PROVEN"],
    ),
    (
        "marker-decomposition-non-interference",
        &["NON_INTERFERENCE_PROVEN"],
    ),
    (
        "marker-decomposition-release-closure",
        &["RESOURCE_RELEASE_CLOSED"],
    ),
    (
        "marker-decomposition-single-actor-typed",
        &["SINGLE_ACTOR_TYPED_RESULT"],
    ),
    (
        "marker-no-llm-authoring",
        &[
            "LLM_CALLS_ZERO",
            "ENGLISH_SUBGOALS_ZERO",
            "CANNED_SUBGOALS_ZERO",
        ],
    ),
];

/// Configuration of one single-operator workday.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkdayConfig {
    /// splitmix64 seed driving category selection and refusal injection.
    pub seed: u64,
    /// Number of logical ticks in the day (one workload set per tick).
    pub ticks: usize,
    /// Injected bounded-admission refusal rate (per mille of ticks),
    /// reusing the `generate` machinery's withheld-final-problem case.
    pub refusal_per_mille: usize,
}

/// Report of one workday run. Headline numbers are ASSIGNED from the
/// on-disk metric SELECTs over the OCEL evidence graph (the graph is the
/// authority); the `telemetry_*` fields are in-process counters, reconciled
/// against the graph by a typed refusal gate before this struct exists.
#[derive(Debug, serde::Serialize)]
pub struct WorkdayReport {
    /// Always "MEASURED_CNG_RESULT".
    pub measurement_class: &'static str,
    pub out_dir: String,
    pub seed: u64,
    pub ticks: usize,
    // --- Graph-derived headline numbers (metric-*.rq SELECT authority).
    pub workers_represented: u64,
    pub workflow_instances: u64,
    pub executed_transitions: u64,
    pub receipts: u64,
    pub refusals: u64,
    pub admission_requests: u64,
    pub admissions_granted: u64,
    pub resumes: u64,
    /// replay_verified events in the evidence graph (metric-replay.rq
    /// SELECT authority; PROJ-614). Every receipted tick set was replayed
    /// through the real manufacture chain byte-identically.
    pub replay_verified: u64,
    /// hook_receipt events carrying ex:hookDeltaHash in the evidence graph
    /// (metric-hook-actuations.rq `?receipted` SELECT authority; PROJ-612/
    /// 614 — metric-hook-receipts.rq was folded into that query).
    pub hook_receipts: u64,
    /// dispatch_sent events in the evidence graph (PROJ-619; the graph is
    /// the authority, reconciled against the adapter telemetry).
    pub dispatches_sent: u64,
    /// consequence_admitted events in the evidence graph (PROJ-619).
    pub consequences_admitted: u64,
    /// consequence_refused events in the evidence graph (PROJ-619/620).
    pub consequences_refused: u64,
    /// dispatch_timed_out events in the evidence graph (PROJ-620).
    pub dispatch_timeouts: u64,
    /// remediation_manufactured events in the evidence graph (PROJ-620).
    pub remediations: u64,
    /// DISTINCT engine identities with an engine_started event in the
    /// evidence graph (metric-engine-instances.rq authority; PROJ-727).
    /// 0 on a single-operator workday — the serve loops that emit
    /// engine_started run in their own processes/bundles. No Rust twin
    /// exists on this path, so this number is graph-only (not gated).
    pub engine_instances: u64,
    /// remote_dispatch_sent events in the evidence graph (PROJ-727;
    /// 0 on the loopback-only workday, reconciled against the adapter).
    pub remote_dispatches: u64,
    /// remote_consequence_received events in the evidence graph (PROJ-727;
    /// 0 on the loopback-only workday, reconciled against the adapter).
    pub remote_consequences_received: u64,
    /// arazzo_workflow_generated events in the evidence graph (PROJ-727;
    /// one per broker dispatch lifecycle, reconciled).
    pub arazzo_workflows_generated: u64,
    /// arazzo_workflow_dispatched events in the evidence graph (PROJ-727;
    /// the dispatched twin of every generated projection, reconciled).
    pub arazzo_workflows_dispatched: u64,
    /// metric-dispatch-closure.rq facet counts over the observation graph
    /// (PROJ-614): open_external / unacknowledged / returned_unadmitted /
    /// refused_consequences / compensating / completed_trees. The
    /// unacknowledged and returned_unadmitted facets are CNG_R19-gated to 0
    /// before this struct exists.
    pub dispatch_closure: BTreeMap<String, u64>,
    /// The seventeen v26.7.10 success markers (PROJ-622/727: sixteen named
    /// markers + the conjunction), each derived from a
    /// `queries/markers/*.rq` SELECT over the obs ∪ evidence ∪ registry
    /// union store. All `true` by construction: a false marker refused
    /// (`CNG_R20 MarkerFalse`) before this struct exists; carried so JSON
    /// consumers see which markers were checked.
    pub markers: BTreeMap<String, bool>,
    // --- Telemetry (Rust counters; reconciled, never authoritative).
    pub telemetry_refusals: usize,
    pub telemetry_transitions: usize,
    pub telemetry_next_action_answers: usize,
    /// Broker-counted successful actuations (telemetry; reconciled against
    /// the graph-derived `hook_receipts`).
    pub telemetry_hook_actuations: usize,
    /// Adapter-counted outbound dispatches (telemetry; reconciled against
    /// the graph-derived `dispatches_sent`).
    pub telemetry_dispatches_sent: usize,
    /// Adapter-counted admitted consequences (telemetry; reconciled).
    pub telemetry_consequences_admitted: usize,
    /// Adapter-counted rendered Arazzo projections (telemetry; reconciled
    /// against the graph-derived `arazzo_workflows_generated`).
    pub telemetry_arazzo_generated: usize,
    /// Adapter-counted dispatched Arazzo projections (telemetry;
    /// reconciled against `arazzo_workflows_dispatched`).
    pub telemetry_arazzo_dispatched: usize,
    /// Adapter-counted remote dispatches (telemetry; reconciled against
    /// `remote_dispatches` — 0 on the loopback-only workday).
    pub telemetry_remote_dispatches: usize,
    /// Adapter-counted remote consequences received (telemetry; reconciled
    /// against `remote_consequences_received`).
    pub telemetry_remote_consequences_received: usize,
    // --- Digests (no wall clock anywhere in their inputs).
    pub evidence_chain_digest: String,
    pub ocel_graph_digest: String,
    pub obs_digest: String,
    /// Run-level graphlaw hook digest over every HookVerdictRecord of the
    /// day, in actuation order; folded into `evidence_chain_digest` (see
    /// module docs, "Evidence chain composition").
    pub run_hook_hash: String,
}

/// A refused tick awaiting its bounded admission: the exact minimal
/// admission fragment (seed-derived, withheld at generation time) and where
/// to admit it.
struct PendingAdmission {
    set_id: String,
    dir: PathBuf,
    withheld_final_problem: String,
    /// Workload category of the interrupted set (the resume path must
    /// actuate its transitions through the same category hook).
    category: &'static str,
}

/// Evaluates the standing-next-action SELECT and refuses
/// `CNG_R12 StandingAmbiguous` unless it returns exactly `expected` rows.
/// `expected` is 1 while work remains and 0 at a clean boundary; any other
/// observed cardinality means standing does not determine one lawful next
/// action.
///
/// # Complexity
/// One SELECT over O(obs facts); row materialization O(rows).
pub(super) fn expect_standing_rows(
    obs_store: &Store,
    standing_query: &str,
    tick: usize,
    expected: usize,
) -> Result<Vec<BTreeMap<String, String>>, CngRefusal> {
    let rows = select_rows(obs_store, standing_query)?;
    if rows.len() != expected {
        return Err(CngRefusal::StandingAmbiguous {
            tick,
            candidate_count: rows.len(),
        });
    }
    Ok(rows)
}

/// Graph-derived hook-actuation gate (PROJ-614): runs the on-disk
/// `metric-hook-actuations.rq` (the DEFINITION_OF_DONE-named authority,
/// superseding metric-hook-receipts.rq) over the OCEL evidence graph and
/// refuses on any transitions/receipts mismatch. Returns
/// `(transitions, receipted)` — the graph-side numbers the reconcile gate
/// compares against Rust telemetry.
///
/// # Errors
/// `CNG_R19 EvidenceGateFailed { gate: "unreceipted-actuations" }` when
/// `?mismatches != 0` (zero-unreceipted-actuation law refuted BY the graph).
///
/// # Complexity
/// One SELECT over O(evidence facts).
pub(super) fn hook_actuation_gate(
    evidence_store: &Store,
    queries: &QuerySet,
) -> Result<(u64, u64), CngRefusal> {
    let rows = select_rows(evidence_store, queries.get("metric-hook-actuations")?)?;
    let row = rows.first().ok_or_else(|| {
        CngRefusal::MalformedTtl("metric-hook-actuations.rq yielded no rows".to_string())
    })?;
    let field = |name: &str| -> Result<i64, CngRefusal> {
        row.get(name)
            .ok_or_else(|| {
                CngRefusal::MalformedTtl(format!("metric-hook-actuations.rq row missing ?{name}"))
            })?
            .parse::<i64>()
            .map_err(|e| CngRefusal::MalformedTtl(format!("metric-hook-actuations {name}: {e}")))
    };
    let transitions = field("transitions")?;
    let receipted = field("receipted")?;
    let mismatches = field("mismatches")?;
    if mismatches != 0 {
        return Err(CngRefusal::EvidenceGateFailed {
            gate: "unreceipted-actuations".to_string(),
            count: mismatches,
        });
    }
    Ok((transitions as u64, receipted as u64))
}

/// Graph-derived dispatch-closure gate (PROJ-614): runs the on-disk
/// `metric-dispatch-closure.rq` (the DEFINITION_OF_DONE-named authority;
/// the operational dispatch-closure.rq is the broker's per-parent law
/// evaluator, not a metrics source) over the OBSERVATION graph — the
/// closure/remediation predicates live only there — and returns every
/// facet count. Refuses when the graph shows unreceipted dispatches
/// (`unacknowledged`) or unadmitted-accepted consequences
/// (`returned_unadmitted`).
///
/// # Errors
/// `CNG_R19 EvidenceGateFailed` naming the failing gate; `CNG_R01` when a
/// facet row is malformed or missing.
///
/// # Complexity
/// One SELECT over O(obs facts) + O(facets) row scans.
pub(super) fn dispatch_closure_gate(
    obs_store: &Store,
    queries: &QuerySet,
) -> Result<BTreeMap<String, u64>, CngRefusal> {
    let rows = select_rows(obs_store, queries.get("metric-dispatch-closure")?)?;
    let mut facets: BTreeMap<String, u64> = BTreeMap::new();
    for row in rows {
        let facet = row.get("facet").cloned().ok_or_else(|| {
            CngRefusal::MalformedTtl("metric-dispatch-closure row missing ?facet".to_string())
        })?;
        let count = row
            .get("count")
            .ok_or_else(|| {
                CngRefusal::MalformedTtl("metric-dispatch-closure row missing ?count".to_string())
            })?
            .parse::<u64>()
            .map_err(|e| CngRefusal::MalformedTtl(format!("metric-dispatch-closure count: {e}")))?;
        facets.insert(facet, count);
    }
    for (facet, gate) in [
        ("unacknowledged", "unreceipted-dispatches"),
        ("returned_unadmitted", "unadmitted-consequences"),
    ] {
        let count = facets.get(facet).copied().ok_or_else(|| {
            CngRefusal::MalformedTtl(format!(
                "metric-dispatch-closure yielded no {facet} facet row"
            ))
        })?;
        if count > 0 {
            return Err(CngRefusal::EvidenceGateFailed {
                gate: gate.to_string(),
                count: count as i64,
            });
        }
    }
    Ok(facets)
}

/// Builds the marker evaluation store (PROJ-622): the OBSERVATION graph ∪
/// the OCEL evidence graph ∪ the Dialect Registry TTL. Each marker query's
/// header names which of the three vocabularies it reads; loading the
/// union keeps evaluation one-store simple while the disjoint namespaces
/// (obs:, ocel:, dreg:) keep the sources separable.
///
/// # Complexity
/// O(obs + evidence triples) inserts + O(registry bytes) parse.
pub(super) fn build_marker_store(
    obs_store: &Store,
    evidence_store: &Store,
    registry_path: &Path,
) -> Result<Store, CngRefusal> {
    let store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("marker store construction: {e}")))?;
    for source in [obs_store, evidence_store] {
        for quad in source.iter() {
            let quad =
                quad.map_err(|e| CngRefusal::IoRefused(format!("marker store iteration: {e}")))?;
            store
                .insert(&quad)
                .map_err(|e| CngRefusal::IoRefused(format!("marker store insert: {e}")))?;
        }
    }
    let registry = fs::read_to_string(registry_path)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", registry_path.display())))?;
    store
        .load_from_slice(
            RdfParser::from_format(RdfFormat::Turtle),
            registry.as_bytes(),
        )
        .map_err(|e| {
            CngRefusal::MalformedTtl(format!("registry load {}: {e}", registry_path.display()))
        })?;
    Ok(store)
}

/// Evaluates every v26.7.10 success marker (PROJ-622) over the marker
/// store. Markers are SPARQL-derived ONLY: each `queries/markers/*.rq`
/// returns one `?value` row where 0 = proven; any other value is a typed
/// refusal (nonzero process exit), never a warning.
/// `V26_7_10_PRODUCTION_READY` is the conjunction of the other sixteen.
///
/// # Errors
/// `CNG_R20 MarkerFalse` naming the first false marker and its value;
/// `CNG_R01/R05` for missing/malformed marker queries.
///
/// # Complexity
/// O(markers) SELECTs, each over O(union-store facts).
pub(super) fn evaluate_markers(
    marker_store: &Store,
    marker_queries: &QuerySet,
) -> Result<BTreeMap<String, bool>, CngRefusal> {
    let mut markers = evaluate_marker_map(marker_store, marker_queries, &MARKER_MAP)?;
    // Conjunction marker: reachable only when all sixteen above are true (a
    // false marker refused above), so this is `true` by construction — it
    // is still computed as the fold, not asserted.
    let all = markers.values().all(|v| *v);
    markers.insert("V26_7_10_PRODUCTION_READY".to_string(), all);
    Ok(markers)
}

/// Evaluates one marker map (stem → marker names) over `marker_store`.
/// Shared by [`evaluate_markers`] (workday, [`MARKER_MAP`]) and the
/// multi-engine coordinator ([`DISTRIBUTED_MARKER_MAP`]). Each query
/// returns one `?value` row where 0 = proven (inverted existence queries
/// keep that convention — see their headers); any other value is a typed
/// refusal, never a warning.
///
/// # Errors
/// `CNG_R20 MarkerFalse` naming the first false marker and its value;
/// `CNG_R01/R05` for missing/malformed marker queries.
///
/// # Complexity
/// O(|map|) SELECTs, each over O(union-store facts).
pub(super) fn evaluate_marker_map(
    marker_store: &Store,
    marker_queries: &QuerySet,
    map: &[(&str, &[&str])],
) -> Result<BTreeMap<String, bool>, CngRefusal> {
    let mut markers: BTreeMap<String, bool> = BTreeMap::new();
    for (stem, names) in map.iter().copied() {
        let rows = select_rows(marker_store, marker_queries.get(stem)?)?;
        let value = rows
            .first()
            .and_then(|r| r.get("value"))
            .ok_or_else(|| CngRefusal::MalformedTtl(format!("{stem}.rq yielded no ?value row")))?
            .parse::<i64>()
            .map_err(|e| CngRefusal::MalformedTtl(format!("{stem} value parse: {e}")))?;
        for name in names {
            if value != 0 {
                return Err(CngRefusal::MarkerFalse {
                    marker: (*name).to_string(),
                    value,
                });
            }
            markers.insert((*name).to_string(), true);
        }
    }
    Ok(markers)
}

/// Builds the PLANNING marker evaluation store (PROJ-739/740/742) from a
/// `decomposition-result.ttl` evidence graph written by a real
/// `cng plan decompose` run (`bench::decomp::decompose`/`decompose_with`,
/// PROJ-741). Unlike [`build_marker_store`] (the obs ∪ evidence ∪ registry
/// union), this graph is self-contained (`decomp:`/`prov:`/`xsd:`/`powl2:`
/// vocabulary only, already parse-validated by
/// `decomp::mod::emit_result_graph` before it was written) — a dedicated
/// loader, not the obs/evidence union, matching the
/// [`DISTRIBUTED_MARKER_MAP`] precedent of a marker family owning its own
/// store construction.
///
/// # Errors
/// `CNG_R10 IoRefused` for an unreadable file; `CNG_R01 MalformedTtl` if
/// the graph does not parse (should not occur for a graph
/// `emit_result_graph` already parse-validated once).
///
/// # Complexity
/// O(bytes) read + O(triples) parse.
pub fn build_decomp_marker_store(result_graph_path: &Path) -> Result<Store, CngRefusal> {
    let turtle = fs::read_to_string(result_graph_path)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", result_graph_path.display())))?;
    let store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("decomp marker store construction: {e}")))?;
    store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .map_err(|e| {
            CngRefusal::MalformedTtl(format!(
                "decomp result graph {} does not parse: {e}",
                result_graph_path.display()
            ))
        })?;
    Ok(store)
}

/// Evaluates the nine v26.7.10 planning markers (PROJ-739/740,
/// [`PLANNING_MARKER_MAP`]) over a [`build_decomp_marker_store`] store.
/// SPARQL-derived only, same law as [`evaluate_markers`]: a false marker is
/// `CNG_R20 MarkerFalse`, never a warning.
///
/// # Errors
/// `CNG_R20 MarkerFalse` naming the first false planning marker; `CNG_R01/
/// R05` for missing/malformed marker queries.
///
/// # Complexity
/// O(|PLANNING_MARKER_MAP|) SELECTs, each over O(decomp graph facts).
pub fn evaluate_planning_markers(
    marker_store: &Store,
    marker_queries: &QuerySet,
) -> Result<BTreeMap<String, bool>, CngRefusal> {
    evaluate_marker_map(marker_store, marker_queries, &PLANNING_MARKER_MAP)
}

/// Full v26.7.10 production-readiness conjunction (PROJ-742). Folds the
/// interim single-operator [`evaluate_markers`] output (the [`MARKER_MAP`]
/// markers, computed by a `workday()` run that never invokes `decompose()`
/// and therefore cannot honestly claim the planning surface on its own)
/// with the nine planning markers ([`evaluate_planning_markers`],
/// PROJ-739/740, SPARQL-derived over a real `cng plan decompose` evidence
/// graph — PROJ-741) and, when a distributed bundle was also run, the six
/// [`DISTRIBUTED_MARKER_MAP`] markers. Returns the merged marker map with
/// `V26_7_10_PRODUCTION_READY` RECOMPUTED as the conjunction over every
/// entry supplied — the SAME marker name [`evaluate_markers`] emits, now
/// meaning what DEFINITION_OF_DONE §16 actually claims (the full planning
/// + distributed evidence surface), not merely the interim single-operator
/// gate. This is purely additive: [`evaluate_markers`]'s own signature and
/// the `workday()` call site are UNCHANGED (the interim-16 computation is
/// unmodified — "do not break the existing interim-16 computation," PROJ-
/// 742), as is `engine_collect_remote`'s own [`DISTRIBUTED_MARKER_MAP`]
/// evaluation; a release-verification step that has run a `workday()`
/// bundle AND a `cng plan decompose` bundle (and optionally a distributed
/// bundle) calls this function to get the DoD-accurate value.
///
/// # Complexity
/// O(|workday_markers| + |planning_markers| + |distributed_markers|) map
/// merges.
pub fn full_production_ready(
    workday_markers: &BTreeMap<String, bool>,
    planning_markers: &BTreeMap<String, bool>,
    distributed_markers: Option<&BTreeMap<String, bool>>,
) -> BTreeMap<String, bool> {
    let mut merged: BTreeMap<String, bool> = workday_markers
        .iter()
        .filter(|(name, _)| name.as_str() != "V26_7_10_PRODUCTION_READY")
        .map(|(name, value)| (name.clone(), *value))
        .collect();
    for (name, value) in planning_markers {
        merged.insert(name.clone(), *value);
    }
    if let Some(distributed) = distributed_markers {
        for (name, value) in distributed {
            merged.insert(name.clone(), *value);
        }
    }
    let all = merged.values().all(|v| *v);
    merged.insert("V26_7_10_PRODUCTION_READY".to_string(), all);
    merged
}

/// Actuates every executed transition of a manufactured outcome through
/// its category hook (zero-unreceipted-actuation law) and emits the
/// `hook_receipt` observation per transition; on the hook's FIRST firing
/// of the day, also emits its EXECUTED and RECEIPTED `hook_standing`
/// observations. Refused outcomes actuate nothing (no transition executed).
///
/// This is the broker choke point (PROJ-619): after local hook actuation,
/// categories that route to an external execution class (deterministic
/// rule in `dispatch::route_category`) additionally dispatch one external
/// workflow through the loopback adapter under the same zero-unreceipted
/// law — `api-orchestration` runs the admitted Arazzo description (one
/// dispatch per step, PROJ-621), `software-delivery` dispatches with
/// recursive depth 1 (child closure, PROJ-620), `purchase-order-approval`
/// dispatches to the MOCKED-HUMAN surface.
///
/// # Errors
/// `CNG_R13 UnreceiptedActuation` from the broker when a transition's
/// category hook yields no receipt; dispatch refusals (`CNG_R15/R16/R17/
/// R18`) propagate; a loopback dispatch that does not admit is `CNG_R09`
/// (the deterministic mechanism is broken).
///
/// # Complexity
/// O(transitions) actuations (each one bounded pack materialization) +
/// at most one external dispatch lifecycle (bounded by deadline ticks and
/// `CHILD_FAN_OUT^depth`; Arazzo: O(steps) lifecycles).
#[allow(clippy::too_many_arguments)]
fn actuate_transitions(
    writer: &mut ObsWriter<'_>,
    broker: &mut WorkdayHookBroker,
    adapter: &mut DispatchAdapter<'_>,
    obs_store: &Store,
    fired_hooks: &mut BTreeSet<String>,
    set_id: &str,
    category: &str,
    tick: usize,
    outcome: &SetOutcome,
) -> Result<(), CngRefusal> {
    if outcome.refusal_code.is_some() {
        return Ok(());
    }
    let tick_text = tick.to_string();
    for (seq, _label) in outcome.activity_labels.iter().enumerate() {
        let receipt = broker.actuate(set_id, category, tick, seq)?;
        writer.emit(
            "hook-receipt",
            &[
                ("SET_ID", set_id),
                ("WORKFLOW_ID", set_id),
                ("TICK", tick_text.as_str()),
                ("HOOK_NAME", receipt.hook_name.as_str()),
                ("DIALECT", WORKDAY_HOOK_DIALECT),
                ("DELTA_HASH", receipt.delta_hash.as_str()),
                ("IDEMPOTENCY_KEY", receipt.idempotency_key.as_str()),
            ],
        )?;
    }
    if !outcome.activity_labels.is_empty() && fired_hooks.insert(category.to_string()) {
        for (state, order) in [("EXECUTED", "5"), ("RECEIPTED", "6")] {
            writer.emit(
                "hook-standing",
                &[
                    ("SET_ID", set_id),
                    ("HOOK_NAME", category),
                    ("STATE", state),
                    ("STANDING_ORDER", order),
                    ("DIALECT", WORKDAY_HOOK_DIALECT),
                ],
            )?;
        }
    }

    // --- External dispatch beside local actuation (PROJ-619/620/621).
    if !outcome.activity_labels.is_empty() {
        match route_category(category) {
            ExecutionClass::LocalActuation => {}
            class if category == "api-orchestration" => {
                debug_assert_eq!(class, ExecutionClass::ExternalMachineDispatch);
                run_arazzo_projection(
                    adapter,
                    writer,
                    obs_store,
                    &default_description_path(),
                    set_id,
                    category,
                    tick,
                )?;
            }
            class => {
                let contract = workday_contract(set_id, category, tick, class);
                let dispatch_id = contract.dispatch_id.clone();
                let dispatched = adapter.dispatch(
                    writer,
                    obs_store,
                    contract,
                    tick,
                    true,
                    SynthesisMode::LoopbackDeterministic,
                    1,
                )?;
                if dispatched != DispatchOutcome::Admitted {
                    return Err(CngRefusal::HardcodingSuspicion(format!(
                        "loopback dispatch {dispatch_id} did not admit \
                         ({dispatched:?}); the deterministic loopback mechanism \
                         is broken"
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Grants a pending bounded admission and resumes the interrupted workflow:
/// writes the withheld minimal admission fragment (MOCKED-HUMAN grant —
/// mechanism real, human simulated), emits admission_granted, re-runs the
/// real manufacture chain, emits resumed + the full record observation
/// sequence. Returns the successful outcome.
///
/// # Complexity
/// One full `manufacture_set` (pipeline-bounded) + O(transitions) obs.
#[allow(clippy::too_many_arguments)]
fn grant_and_resume(
    writer: &mut ObsWriter<'_>,
    broker: &mut WorkdayHookBroker,
    adapter: &mut DispatchAdapter<'_>,
    obs_store: &Store,
    fired_hooks: &mut BTreeSet<String>,
    pending: &PendingAdmission,
    tick: usize,
    export_dir: &Path,
) -> Result<SetOutcome, CngRefusal> {
    let tick_text = tick.to_string();
    let path = pending.dir.join("goal-final.problem.ttl");
    fs::write(&path, &pending.withheld_final_problem)
        .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
    writer.emit(
        "admission-granted",
        &[
            ("SET_ID", pending.set_id.as_str()),
            ("TICK", tick_text.as_str()),
            ("MISSING", "goal-final.problem.ttl"),
        ],
    )?;
    let outcome = manufacture_set(&pending.dir, Some(export_dir));
    if let Some(code) = outcome.refusal_code {
        // The granted admission was the exact minimal missing input; a
        // second refusal means the resume mechanism itself is broken.
        return Err(CngRefusal::HardcodingSuspicion(format!(
            "resume of {} after granted admission refused again with {code}",
            pending.set_id
        )));
    }
    writer.emit(
        "resumed",
        &[
            ("SET_ID", pending.set_id.as_str()),
            ("TICK", tick_text.as_str()),
        ],
    )?;
    emit_record_observations(writer, &pending.set_id, &outcome, &[], tick)?;
    actuate_transitions(
        writer,
        broker,
        adapter,
        obs_store,
        fired_hooks,
        &pending.set_id,
        pending.category,
        tick,
        &outcome,
    )?;
    Ok(outcome)
}

/// Runs a single-operator workday of `cfg.ticks` logical ticks into
/// `out_dir`. See the module docs for the per-tick sequence and the
/// bounded-admission → resume law.
///
/// # Errors
/// Typed `CngRefusal` only: `CNG_R12 StandingAmbiguous` when standing does
/// not determine exactly one lawful next action while work remains;
/// `CNG_R09 HardcodingSuspicion` when telemetry disagrees with the
/// graph-derived numbers or a granted admission fails to resume;
/// `CNG_R08 Nondeterminism` when the end-of-day replay re-manufacture does
/// not reproduce a recorded POWL digest; `CNG_R19 EvidenceGateFailed` when
/// the graph shows unreceipted actuations/dispatches or unadmitted
/// consequences; `CNG_R20 MarkerFalse` when any v26.7.10 success marker
/// evaluates false; I/O, parse, and pipeline refusals propagate unchanged.
///
/// # Complexity
/// O(ticks) manufactures (each pipeline-bounded) + one standing SELECT per
/// tick over O(obs facts) + O(t log t) evidence serialization.
pub fn workday(
    out_dir: &Path,
    cfg: &WorkdayConfig,
    queries_dir: Option<&Path>,
) -> Result<WorkdayReport, CngRefusal> {
    let templates: Templates = load_templates()?;
    let query_dir_owned;
    let query_dir = match queries_dir {
        Some(dir) => dir,
        None => {
            query_dir_owned = QuerySet::default_dir();
            &query_dir_owned
        }
    };
    let queries = QuerySet::load(query_dir)?;
    let standing_query = queries.get("standing-next-action")?.to_string();

    // --- Dialect Registry gate + hook pack admission (PROJ-612/613):
    // refuses CNG_R14 before any tick executes; the broker owns the
    // once-loaded pack and accumulates the day's hook verdict records.
    let hooks_dir = WorkdayHookBroker::default_hooks_dir();
    let mut broker = WorkdayHookBroker::new(
        &hooks_dir.join("dialect-registry.ttl"),
        &hooks_dir.join("dialect-registry.shape.ttl"),
        // Two packs in fixed order (graphlaw admits ≤ 12 hooks per pack).
        &[
            hooks_dir.join("workday-pack.ttl"),
            hooks_dir.join("workday-pack-2.ttl"),
        ],
        &queries,
    )?;

    fs::create_dir_all(out_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", out_dir.display())))?;

    // --- External dispatch adapter (PROJ-619): loopback outbox/inbox under
    // out_dir, dispatch templates + shape law from disk.
    let mut adapter = DispatchAdapter::new(out_dir, &queries)?;

    // --- Roster of ONE operator, rendered from the roster observation
    // template, written to disk, and consumed back through oxigraph —
    // identical admission path to the Fortune-5 run.
    let mut rng = cfg.seed;
    let departments = [
        "finance",
        "hr",
        "logistics",
        "sales",
        "engineering",
        "legal",
    ];
    let department = departments[(splitmix64(&mut rng) % departments.len() as u64) as usize];
    let roster_template = templates.obs.get("roster").ok_or_else(|| {
        CngRefusal::IoRefused("roster observation template missing from loaded set".to_string())
    })?;
    let roster_body = fill_template(
        roster_template,
        &[
            ("SUBJECT", "obs-roster-w0"),
            ("SEQ", "0"),
            ("SET_ID", "roster"),
            ("WORKER_ID", "w0"),
            ("ROLE", "operator"),
            ("DEPARTMENT", department),
            ("STANDING", "admitted"),
        ],
    );
    let roster_dir = out_dir.join("roster");
    fs::create_dir_all(&roster_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir roster: {e}")))?;
    let roster_path = roster_dir.join("partition-00000.ttl");
    fs::write(&roster_path, &roster_body)
        .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", roster_path.display())))?;
    let obs_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("observation store construction: {e}")))?;
    obs_store
        .load_from_slice(
            RdfParser::from_format(RdfFormat::Turtle),
            roster_body.as_bytes(),
        )
        .map_err(|e| CngRefusal::MalformedTtl(format!("roster load: {e}")))?;

    // --- Datalog role layer over the one-operator roster (same rules file
    // and derivation path as the Fortune-5 run).
    let workers = roster_workers(&obs_store)?;
    let rules_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("rules")
        .join("bench-roles.dl");
    let rules_text = fs::read_to_string(&rules_path)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", rules_path.display())))?;
    let datalog = derive_roles_datalog(&workers, &rules_text)?;

    let obs_dir = out_dir.join("obs");
    let export_dir = out_dir.join("generated");
    let admissions_dir = out_dir.join("admissions");
    let worker_iri = format!("{RWAI_PREFIX}w0");

    let mut writer = ObsWriter::new(&templates, &obs_store, &obs_dir, "workday")?;

    // --- HookStanding ladder at admission (PROJ-613): every pack hook
    // passed load(REGISTERED) → kh:HookShape SHACL (ADMITTED) → registry
    // gate (AUTHORIZED) → compile+Kahn schedule (READY) inside the broker
    // constructor; EXECUTED/RECEIPTED follow at first firing. REPLAYABLE
    // (order 7) is emitted only after the end-of-day producer replay
    // verification passes (PROJ-614/622; see the replay block below).
    //
    // # Complexity
    // O(hooks × 4) observation emissions.
    for hook_name in broker.hook_names().to_vec() {
        for (state, order) in [
            ("REGISTERED", "1"),
            ("ADMITTED", "2"),
            ("AUTHORIZED", "3"),
            ("READY", "4"),
        ] {
            writer.emit(
                "hook-standing",
                &[
                    ("SET_ID", "hooks"),
                    ("HOOK_NAME", hook_name.as_str()),
                    ("STATE", state),
                    ("STANDING_ORDER", order),
                    ("DIALECT", WORKDAY_HOOK_DIALECT),
                ],
            )?;
        }
    }

    for (worker_id, role) in &datalog.derived {
        writer.emit(
            "role-derived",
            &[
                ("SET_ID", "datalog"),
                ("WORKER_ID", worker_id.as_str()),
                ("ROLE", role.as_str()),
                // The Datalog layer runs before the first tick: tick 0.
                ("TICK", "0"),
            ],
        )?;
    }

    // --- The day. One workload set per logical tick; pending bounded
    // admissions are granted and resumed at the NEXT tick boundary.
    //
    // # Complexity
    // O(ticks) iterations; each is one manufacture + one or two standing
    // SELECTs over the growing obs graph (O(obs facts) each).
    let mut pending: Option<PendingAdmission> = None;
    let mut fired_hooks: BTreeSet<String> = BTreeSet::new();
    let mut telemetry_refusals = 0usize;
    let mut telemetry_transitions = 0usize;
    let mut telemetry_next_action_answers = 0usize;
    let mut admission_requests_telemetry = 0usize;
    let mut resumes_telemetry = 0usize;
    let mut receipt_digests: BTreeMap<String, String> = BTreeMap::new();
    let ticks_dir = out_dir.join("ticks");
    for tick in 0..=cfg.ticks {
        let tick_text = tick.to_string();
        // 1. Resolve standing from the previous tick: exactly one pending
        //    "admit" action, or a clean (zero-row) boundary.
        if let Some(p) = pending.take() {
            let rows = expect_standing_rows(&obs_store, &standing_query, tick, 1)?;
            let row = rows.first().ok_or_else(|| CngRefusal::StandingAmbiguous {
                tick,
                candidate_count: 0,
            })?;
            let action = row.get("action").cloned().unwrap_or_default();
            let set_id = row.get("setId").cloned().unwrap_or_default();
            if action != "admit" || set_id != p.set_id {
                return Err(CngRefusal::HardcodingSuspicion(format!(
                    "standing derived ({action}, {set_id}) but the pending admission is {}",
                    p.set_id
                )));
            }
            telemetry_next_action_answers += 1;
            writer.emit(
                "next-action",
                &[
                    ("SET_ID", set_id.as_str()),
                    ("TICK", tick_text.as_str()),
                    ("ACTION", action.as_str()),
                ],
            )?;
            let outcome = grant_and_resume(
                &mut writer,
                &mut broker,
                &mut adapter,
                &obs_store,
                &mut fired_hooks,
                &p,
                tick,
                &export_dir,
            )?;
            telemetry_transitions += outcome.transitions;
            resumes_telemetry += 1;
            receipt_digests.insert(p.set_id.clone(), outcome.powl_digest.clone());
        } else {
            expect_standing_rows(&obs_store, &standing_query, tick, 0)?;
        }
        if tick == cfg.ticks {
            // Epilogue boundary: the day is over once standing is clean.
            break;
        }

        // 2. Admit this tick's workload artifact set (seed-derived category,
        //    seeded bounded-admission injection).
        let set_id = format!("tick-{tick:04}");
        let category = CATEGORIES[(splitmix64(&mut rng) % CATEGORIES.len() as u64) as usize];
        let omit = (splitmix64(&mut rng) % 1000) < cfg.refusal_per_mille as u64;
        // PROJ-609 content targets: an interruption interrupts the CURRENT
        // tick's in-flight workflow instance; planning plans for the NEXT
        // tick's standing.
        let content_target = match category {
            "interruption" => Some(set_id.clone()),
            "planning" => Some(format!("standing-tick-{:04}", tick + 1)),
            _ => None,
        };
        let set_dir = ticks_dir.join(&set_id);
        let (_files, _bytes, withheld) = write_set(
            &templates,
            &set_dir,
            &mut rng,
            &format!("t{tick:04}"),
            &worker_iri,
            category,
            0,
            omit,
            content_target.as_deref(),
        )?;
        writer.emit(
            "workday-tick",
            &[
                ("SET_ID", set_id.as_str()),
                ("TICK", tick_text.as_str()),
                ("WORKER_ID", "w0"),
            ],
        )?;

        // 3. Standing must now derive exactly ONE lawful next action: the
        //    manufacture of this tick's set. Log the answer.
        let rows = expect_standing_rows(&obs_store, &standing_query, tick, 1)?;
        let row = rows.first().ok_or_else(|| CngRefusal::StandingAmbiguous {
            tick,
            candidate_count: 0,
        })?;
        let action = row.get("action").cloned().unwrap_or_default();
        telemetry_next_action_answers += 1;
        writer.emit(
            "next-action",
            &[
                ("SET_ID", set_id.as_str()),
                ("TICK", tick_text.as_str()),
                ("ACTION", action.as_str()),
            ],
        )?;

        // 4. Manufacture through the real chain; emit the record sequence.
        let outcome = manufacture_set(&set_dir, Some(&export_dir));
        emit_record_observations(&mut writer, &set_id, &outcome, &[], tick)?;
        actuate_transitions(
            &mut writer,
            &mut broker,
            &mut adapter,
            &obs_store,
            &mut fired_hooks,
            &set_id,
            category,
            tick,
            &outcome,
        )?;
        match outcome.refusal_code {
            None => {
                telemetry_transitions += outcome.transitions;
                receipt_digests.insert(set_id.clone(), outcome.powl_digest.clone());
            }
            Some(code) => {
                telemetry_refusals += 1;
                let withheld_body = withheld.ok_or_else(|| {
                    CngRefusal::UnsupportedConstruct(format!(
                        "tick {tick} refused {code} without an injected bounded-admission \
                         case; the workday has no lawful admission to request"
                    ))
                })?;
                // Manufacture the ex:AdmissionRequest artifact naming
                // exactly the minimal missing admission (template-rendered,
                // written OUTSIDE the artifact-set dir).
                fs::create_dir_all(&admissions_dir)
                    .map_err(|e| CngRefusal::IoRefused(format!("mkdir admissions: {e}")))?;
                let request_body = fill_template(
                    &templates.admission_request,
                    &[
                        ("SUBJECT", format!("admission-req-{set_id}").as_str()),
                        ("SET_ID", set_id.as_str()),
                        ("MISSING", "goal-final.problem.ttl"),
                        ("REFUSAL_CODE", code),
                        ("TICK", tick_text.as_str()),
                    ],
                );
                let request_path = admissions_dir.join(format!("{set_id}.admission-request.ttl"));
                fs::write(&request_path, &request_body).map_err(|e| {
                    CngRefusal::IoRefused(format!("write {}: {e}", request_path.display()))
                })?;
                writer.emit(
                    "admission-requested",
                    &[
                        ("SET_ID", set_id.as_str()),
                        ("TICK", tick_text.as_str()),
                        ("MISSING", "goal-final.problem.ttl"),
                        ("REFUSAL_CODE", code),
                    ],
                )?;
                admission_requests_telemetry += 1;
                pending = Some(PendingAdmission {
                    set_id,
                    dir: set_dir,
                    withheld_final_problem: withheld_body,
                    category,
                });
            }
        }
        // PROJ-721 eager per-tick obs flush: every tick's observations are
        // durable on disk before the next tick begins (crash-resume input;
        // partition layout stays deterministic — one partition per tick).
        writer.flush()?;
    }
    // The epilogue iteration (tick == cfg.ticks) resolved any final pending
    // admission before breaking; the day is complete.
    debug_assert!(pending.is_none());

    // --- Producer replay verification (PROJ-614/622): re-manufacture every
    // tick set through the real chain and require the byte-identical POWL
    // digest; each verification is receipted as a replay_verified
    // observation (metric-replay.rq counts them from the evidence graph),
    // and only after ALL replays pass does each fired hook earn its
    // REPLAYABLE HookStanding rung (order 7). Boundary: this is the
    // PRODUCER's re-manufacture verification, the same class as `run()`'s
    // replay sampling; the independent auditor pass is `workday_verify`,
    // which re-checks from bundle files alone.
    //
    // # Complexity
    // O(ticks) re-manufactures (each pipeline-bounded) + O(hooks) obs
    // emissions.
    for (set_id, expected) in &receipt_digests {
        let replay = manufacture_set(&ticks_dir.join(set_id), None);
        if replay.refusal_code.is_some() || &replay.powl_digest != expected {
            return Err(CngRefusal::Nondeterminism(format!(
                "workday replay of {set_id} did not reproduce its recorded POWL \
                 digest (expected {expected}, got {}, refusal {:?})",
                replay.powl_digest, replay.refusal_code
            )));
        }
    }
    for (set_id, expected) in &receipt_digests {
        writer.emit(
            "replay-verified",
            &[("SET_ID", set_id.as_str()), ("DIGEST", expected.as_str())],
        )?;
    }
    for hook_name in &fired_hooks {
        writer.emit(
            "hook-standing",
            &[
                ("SET_ID", "hooks"),
                ("HOOK_NAME", hook_name.as_str()),
                ("STATE", "REPLAYABLE"),
                ("STANDING_ORDER", "7"),
                ("DIALECT", WORKDAY_HOOK_DIALECT),
            ],
        )?;
    }
    writer.flush()?;

    // --- OCEL materialization + digests: identical construct order and
    // serialization to `run()`/`audit_replay()`.
    let evidence_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("evidence store construction: {e}")))?;
    for construct in OCEL_CONSTRUCT_STEMS {
        run_construct(&obs_store, queries.get(construct)?, &evidence_store)?;
    }
    let (evidence_nt, ocel_graph_digest) = evidence_digest(&evidence_store)?;
    let evidence_dir = out_dir.join("evidence");
    fs::create_dir_all(&evidence_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir evidence: {e}")))?;
    fs::write(evidence_dir.join("ocel.nt"), &evidence_nt)
        .map_err(|e| CngRefusal::IoRefused(format!("write ocel.nt: {e}")))?;
    let obs_digest = obs_dir_digest(out_dir)?;

    // --- Graph-derived headline numbers (the evidence graph is the
    // authority; Rust counters are telemetry, reconciled below).
    let count_of = |name: &str| -> Result<u64, CngRefusal> {
        super::roles::metric_count(&evidence_store, queries.get(name)?, name)
    };
    let workers_g = count_of("metric-workers")?;
    let instances_g = count_of("metric-workflow-instances")?;
    let refusals_g = count_of("metric-refusals")?;
    let receipts_g = count_of("metric-receipts")?;
    // Replay headline: metric-replay.rq over replay_verified events
    // (PROJ-614); reconciled against receipt_digests.len() below.
    let replay_verified_g = count_of("metric-replay")?;
    // Hook-actuation headline + CNG_R19 gate: metric-hook-actuations.rq is
    // the DEFINITION_OF_DONE-named authority (metric-hook-receipts.rq was
    // folded into its ?receipted column and removed).
    let (_transitions_evidence, hook_receipts_g) = hook_actuation_gate(&evidence_store, &queries)?;
    let transition_rows = select_rows(&evidence_store, queries.get("metric-transitions")?)?;
    let count_of_type = |wanted: &str| -> Result<u64, CngRefusal> {
        for row in &transition_rows {
            if row.get("type").map(String::as_str) == Some(wanted) {
                return row
                    .get("count")
                    .ok_or_else(|| {
                        CngRefusal::MalformedTtl(
                            "metric-transitions row missing ?count".to_string(),
                        )
                    })?
                    .parse::<u64>()
                    .map_err(|e| {
                        CngRefusal::MalformedTtl(format!("metric-transitions count: {e}"))
                    });
            }
        }
        Ok(0)
    };
    let fired_g = count_of_type("transition_fired")?;
    let admission_requests_g = count_of_type("admission_requested")?;
    let admissions_granted_g = count_of_type("admission_granted")?;
    let resumes_g = count_of_type("resumed")?;
    // Dispatch-surface headline numbers (PROJ-619/620): the dispatch events
    // flow through ocel-dispatches.construct.rq into the same evidence
    // graph, so metric-transitions' grouped SELECT is their authority too.
    let dispatches_sent_g = count_of_type("dispatch_sent")?;
    let dispatch_acked_g = count_of_type("dispatch_acknowledged")?;
    let dispatch_polls_g = count_of_type("dispatch_poll")?;
    let consequences_returned_g = count_of_type("consequence_returned")?;
    let consequences_admitted_g = count_of_type("consequence_admitted")?;
    let consequences_refused_g = count_of_type("consequence_refused")?;
    let dispatch_timeouts_g = count_of_type("dispatch_timed_out")?;
    let remediations_g = count_of_type("remediation_manufactured")?;
    // PROJ-727 distributed-evidence headline numbers: the arazzo pair flows
    // through ocel-remote-engine.construct.rq into the same evidence graph;
    // the remote kinds are 0 on this loopback-only path (the adapter's
    // remote counters agree, gated below). engine_instances is graph-only
    // (metric-engine-instances.rq; no Rust twin exists on this path).
    let arazzo_generated_g = count_of_type("arazzo_workflow_generated")?;
    let arazzo_dispatched_g = count_of_type("arazzo_workflow_dispatched")?;
    let remote_dispatches_g = count_of_type("remote_dispatch_sent")?;
    let remote_received_g = count_of_type("remote_consequence_received")?;
    let engine_instances_g = super::roles::metric_count(
        &evidence_store,
        queries.get("metric-engine-instances")?,
        "metric-engine-instances",
    )?;

    // --- Reconcile gate (CNG_R09 on disagreement): every headline number
    // must be independently derivable from the evidence graph.
    if workers_g != 1
        || instances_g as usize != cfg.ticks
        || refusals_g as usize != telemetry_refusals
        || receipts_g as usize != cfg.ticks
        || fired_g as usize != telemetry_transitions
        || admission_requests_g as usize != admission_requests_telemetry
        || admissions_granted_g as usize != resumes_telemetry
        || resumes_g as usize != resumes_telemetry
        || receipt_digests.len() != cfg.ticks
        // Replay verification (PROJ-614): every receipted tick set was
        // replayed and its verification is independently derivable from
        // the evidence graph.
        || replay_verified_g as usize != receipt_digests.len()
        // Zero-unreceipted-actuation (PROJ-612): every executed transition
        // has exactly one hook receipt in the evidence graph, and the
        // broker's telemetry agrees with the graph.
        || hook_receipts_g as usize != telemetry_transitions
        || broker.actuations() != telemetry_transitions
        // Zero-unreceipted-dispatch (PROJ-619): every outbound dispatch,
        // acknowledgement, poll, returned/admitted/refused consequence,
        // timeout, and remediation the adapter counted is independently
        // derivable from the evidence graph, and vice versa.
        || dispatches_sent_g as usize != adapter.telemetry.sent
        || dispatch_acked_g as usize != adapter.telemetry.acknowledged
        || dispatch_polls_g as usize != adapter.telemetry.polls
        || consequences_returned_g as usize != adapter.telemetry.returned
        || consequences_admitted_g as usize != adapter.telemetry.admitted
        || consequences_refused_g as usize != adapter.telemetry.refused
        || dispatch_timeouts_g as usize != adapter.telemetry.timeouts
        || remediations_g as usize != adapter.telemetry.remediations
        // PROJ-727: every rendered Arazzo projection and its dispatched
        // twin, and every remote boundary crossing (0 on loopback), is
        // independently derivable from the evidence graph.
        || arazzo_generated_g as usize != adapter.telemetry.arazzo_generated
        || arazzo_dispatched_g as usize != adapter.telemetry.arazzo_dispatched
        || remote_dispatches_g as usize != adapter.telemetry.remote_sent
        || remote_received_g as usize != adapter.telemetry.remote_received
    {
        return Err(CngRefusal::HardcodingSuspicion(format!(
            "workday telemetry/evidence mismatch — the SPARQL evidence graph is the \
             authority: graph workers={workers_g} instances={instances_g} \
             refusals={refusals_g} receipts={receipts_g} fired={fired_g} \
             admission_requests={admission_requests_g} granted={admissions_granted_g} \
             resumes={resumes_g} hook_receipts={hook_receipts_g} \
             replay_verified={replay_verified_g} vs telemetry ticks={} \
             refusals={telemetry_refusals} transitions={telemetry_transitions} \
             requests={admission_requests_telemetry} resumes={resumes_telemetry} \
             hook_actuations={} receipted_ticks={} — dispatch surface: graph \
             sent={dispatches_sent_g} acked={dispatch_acked_g} \
             polls={dispatch_polls_g} returned={consequences_returned_g} \
             admitted={consequences_admitted_g} refused={consequences_refused_g} \
             timeouts={dispatch_timeouts_g} remediations={remediations_g} vs \
             adapter telemetry {:?}",
            cfg.ticks,
            broker.actuations(),
            receipt_digests.len(),
            adapter.telemetry
        )));
    }

    // --- Graph-derived dispatch-closure facets + CNG_R19 gates (PROJ-614):
    // unreceipted dispatches and returned-but-unadmitted consequences
    // refuse; the facet counts land in the report.
    let dispatch_closure = dispatch_closure_gate(&obs_store, &queries)?;

    // --- Success markers (PROJ-622, CNG_R20): SPARQL-only, evaluated over
    // the obs ∪ evidence ∪ dialect-registry union store; any false marker
    // is a typed refusal (nonzero exit), never a warning.
    let marker_queries = QuerySet::load(&query_dir.join("markers"))?;
    let marker_store = build_marker_store(
        &obs_store,
        &evidence_store,
        &hooks_dir.join("dialect-registry.ttl"),
    )?;
    let markers = evaluate_markers(&marker_store, &marker_queries)?;

    // --- Receipt chain: BLAKE3 fold over the per-tick POWL digests in
    // tick-id order (BTreeMap iteration is the canonical order; all digests
    // are content-derived, never path- or time-derived), then the run-level
    // graphlaw hook_hash over every HookVerdictRecord of the day in
    // actuation order (PROJ-612 — see module docs, "Evidence chain
    // composition").
    //
    // # Complexity
    // O(ticks) hash updates + O(verdicts) for the hook hash.
    let run_hook_hash = broker.run_hook_hash()?;
    let mut receipt_chain = blake3::Hasher::new();
    for digest in receipt_digests.values() {
        receipt_chain.update(digest.as_bytes());
    }
    // Per-dispatch receipt pairs (contract digest, consequence digest) in
    // dispatch-id order (BTreeMap) — PROJ-619 chain composition.
    for (dispatch_id, (contract_digest, consequence_digest)) in &adapter.receipt_digests {
        receipt_chain.update(dispatch_id.as_bytes());
        receipt_chain.update(contract_digest.as_bytes());
        receipt_chain.update(consequence_digest.as_bytes());
    }
    receipt_chain.update(run_hook_hash.as_bytes());

    let report = WorkdayReport {
        measurement_class: "MEASURED_CNG_RESULT",
        out_dir: out_dir.display().to_string(),
        seed: cfg.seed,
        ticks: cfg.ticks,
        workers_represented: workers_g,
        workflow_instances: instances_g,
        executed_transitions: fired_g,
        receipts: receipts_g,
        refusals: refusals_g,
        admission_requests: admission_requests_g,
        admissions_granted: admissions_granted_g,
        resumes: resumes_g,
        replay_verified: replay_verified_g,
        hook_receipts: hook_receipts_g,
        dispatches_sent: dispatches_sent_g,
        consequences_admitted: consequences_admitted_g,
        consequences_refused: consequences_refused_g,
        dispatch_timeouts: dispatch_timeouts_g,
        remediations: remediations_g,
        engine_instances: engine_instances_g,
        remote_dispatches: remote_dispatches_g,
        remote_consequences_received: remote_received_g,
        arazzo_workflows_generated: arazzo_generated_g,
        arazzo_workflows_dispatched: arazzo_dispatched_g,
        dispatch_closure,
        markers,
        telemetry_refusals,
        telemetry_transitions,
        telemetry_next_action_answers,
        telemetry_hook_actuations: broker.actuations(),
        telemetry_dispatches_sent: adapter.telemetry.sent,
        telemetry_consequences_admitted: adapter.telemetry.admitted,
        telemetry_arazzo_generated: adapter.telemetry.arazzo_generated,
        telemetry_arazzo_dispatched: adapter.telemetry.arazzo_dispatched,
        telemetry_remote_dispatches: adapter.telemetry.remote_sent,
        telemetry_remote_consequences_received: adapter.telemetry.remote_received,
        evidence_chain_digest: format!("blake3:{}", receipt_chain.finalize().to_hex()),
        ocel_graph_digest,
        obs_digest,
        run_hook_hash,
    };
    let results_dir = out_dir.join("results");
    fs::create_dir_all(&results_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir results: {e}")))?;
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| CngRefusal::IoRefused(format!("workday report serialize: {e}")))?;
    fs::write(results_dir.join("workday-report.json"), &json)
        .map_err(|e| CngRefusal::IoRefused(format!("write workday-report.json: {e}")))?;
    Ok(report)
}

#[cfg(test)]
#[path = "workday_test.rs"]
mod workday_test;
