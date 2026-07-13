//! Fortune-5 scale benchmark for Autonomic Recursive Workflow on the REAL
//! cng manufacture chain. Nothing here mocks or bypasses the product path:
//! every workflow goes through `pipeline::import_artifacts` (oxigraph Turtle
//! admission) → `pipeline::generate_plan` (bcinr-pddl grounding + bounded
//! BFS) → `pipeline::hierarchical_projection` → provenance serialization →
//! `shape::validate_powl_store` → `runner::validate_run_hierarchical`
//! (bcinr-powl compile + branchless scheduler + conformance) → BLAKE3
//! receipts.
//!
//! Evidence discipline (Phases 1–4 of the Recursive Workflow benchmark
//! plan):
//! - Every fact the benchmark asserts about itself is first emitted as an
//!   observation (`obs:` vocabulary,
//!   `crates/praxis-graphlaw/ontologies/core/bench-obs.ttl`) rendered from
//!   the `.template.ttl` files — never inline `format!` Turtle.
//! - The OCEL evidence graph is materialized by running the on-disk
//!   `queries/ocel-*.construct.rq` CONSTRUCTs over the observation store;
//!   `ocel_graph_digest` hashes its sorted N-Triples serialization.
//! - Headline `RunReport` numbers are ASSIGNED from the on-disk
//!   `queries/metric-*.rq` SELECTs over the evidence graph; the in-process
//!   Rust counters live in `RunReport.telemetry` and a mismatch with the
//!   graph-derived numbers is a typed refusal (the graph is the authority).
//! - Zero inline SPARQL: classification and attachment discovery are
//!   oxigraph pattern scans over admitted graphs; every SPARQL string is
//!   loaded from the queries directory.
//!
//! Wall-clock timing lives here (benchmark instrumentation), never in the
//! manufacture path itself; digests, receipts, and observation facts
//! contain no time — `obs:obsSeq` is a logical counter.
//!
//! Submodules: [`templates`] (Turtle observation templates + on-disk
//! `.rq` query set), [`generate`] (corpus generation), [`report`]
//! (metrics/report types), [`manufacture`] (per-set real chain execution),
//! [`roles`] (SPARQL exec helpers, attachment derivation, Datalog role
//! layer), [`run`] (top-level orchestration), [`audit_replay`]
//! (independent bundle auditor), [`verify`] (replay/export verification),
//! `multifractal` (Rail G Track 2b: `Z(q,epsilon)` -> `tau(q)` -> `D(q)`
//! -> `alpha(q)`/`f(alpha)` over real per-tick `tape_ops` mass, see that
//! module's doc comment and `docs/jira/v26.7.11/
//! RAIL_G_MEASUREMENT_DESIGN.md`).

mod api_docs;
mod arazzo;
mod audit_replay;
pub mod decomp;
mod dispatch;
pub mod dispatch_diagram;
mod engine;
mod generate;
mod hooks;
pub mod ipc;
mod manufacture;
mod multifractal;
pub mod refusal_sarif;
mod report;
pub mod report_pretty;
mod roles;
mod run;
mod soc2;
mod templates;
mod togaf;
mod verify;
mod workday;
pub mod workday_verify;

pub use audit_replay::{audit_replay, AuditReplayReport};
pub use engine::{
    engine_collect_remote, engine_dispatch_remote, engine_resume, engine_serve,
    EngineCoordinateReport, EngineIdentity, EngineServeReport, ENGINE_VERSION,
};
pub use generate::generate;
pub use report::{EvidenceManifest, RunReport};
pub use report_pretty::{render_engine_serve_report_human, render_workday_report_human};
pub use run::run;
pub use templates::{BenchConfig, GenerateReport, QuerySet};
pub use verify::{verify, VerifyReport};
pub use workday::{
    build_decomp_marker_store, evaluate_planning_markers, full_production_ready, workday,
    WorkdayConfig, WorkdayReport,
};

pub(crate) const WORKERS_PER_ROSTER_PARTITION: usize = 5_000;
pub(crate) const OBS_PER_PARTITION: usize = 4_000;
pub(crate) const RWAI_PREFIX: &str = "http://example.org/rwai#";
pub(crate) const CATEGORIES: [&str; 15] = [
    "email-routing",
    "calendar-change",
    "invoice-matching",
    "purchase-order-approval",
    "expense-review",
    "hr-notice",
    "customer-request",
    "logistics-event",
    "compliance-check",
    "document-request",
    "software-delivery",
    "admission-request",
    // PROJ-609: content-bearing categories. `interruption` artifacts carry
    // ex:interrupts → the in-flight workflow instance IRI; `planning`
    // artifacts carry ex:plansFor → the next-tick standing IRI. Rendered
    // from the bench-category-*.template.ttl files, never inline.
    "interruption",
    "planning",
    // PROJ-621: external-API orchestration category. Its workday ticks
    // additionally run the admitted Arazzo description (examples/
    // arazzo-api-orchestration.ttl) through the dispatch broker: each
    // arz:Step is projected into a DispatchContract and executed
    // EXTERNAL_MACHINE_DISPATCH through the loopback adapter.
    "api-orchestration",
];
pub(crate) const STEP_VERBS: [&str; 8] = [
    "classify",
    "extract",
    "match",
    "verify",
    "check",
    "authorize",
    "execute",
    "record",
];

/// Observation-template kinds, keyed by the `bench-observation-<kind>`
/// template file suffix. `roster` is the generate-time roster variant.
/// The `workday-tick`/`next-action`/`admission-requested`/
/// `admission-granted`/`resumed` kinds belong to the single-operator
/// workday loop (PROJ-608/610/611); `hook-receipt` (per-transition
/// actuation evidence) and `hook-standing` (HookStanding lifecycle) belong
/// to the workday hook broker (PROJ-612/613). The `dispatch-*` /
/// `consequence-*` / `remediation-manufactured` kinds belong to the external
/// dispatch broker surface (PROJ-619/620): both outbound dispatch and inbound
/// consequence are receipted as observations, every bounded poll is an
/// observation, and timeout/refused-conformance remediation (escalation or
/// compensation workflow manufacture) is an observation. `replay-verified`
/// (PROJ-614/616) records a replay re-manufacture that reproduced the
/// recorded POWL digest byte-identically; it is projected into the OCEL
/// evidence graph by `ocel-replays.construct.rq` and counted by
/// `metric-replay.rq`. The `engine-*` and `resume-verified` kinds belong to
/// the multi-engine serve/resume surface (PROJ-722/723/724): every serve
/// observation carries `obs:producedByEngine` (the deterministic
/// EngineIdentity), poll counts are logical, and `resume-verified` records
/// a chain-prefix-verified ledger reload. The `remote-dispatch-sent`/
/// `remote-consequence-received` and `arazzo-workflow-*` kinds belong to
/// the distributed dispatch surface (PROJ-727): remote kinds are emitted
/// only when a contract targets a non-loopback engine, arazzo kinds on
/// every broker lifecycle (rendered projection + its dispatched twin).
/// `direct-engine-bypass` and `shared-memory-crossing` are FORBIDDEN
/// kinds: no production path emits them — they exist so the isolation
/// markers have defined referents and negative fixtures can falsify them.
pub(crate) const OBS_KINDS: [&str; 38] = [
    "imported",
    "planned",
    "projected",
    "shape-validated",
    "transition-fired",
    "receipted",
    "refused",
    "roster-admitted",
    "socket-attached",
    "role-derived",
    "roster",
    "workday-tick",
    "next-action",
    "admission-requested",
    "admission-granted",
    "resumed",
    "hook-receipt",
    "hook-standing",
    "dispatch-sent",
    "dispatch-acknowledged",
    "dispatch-poll",
    "consequence-returned",
    "consequence-admitted",
    "consequence-refused",
    "dispatch-timed-out",
    "remediation-manufactured",
    "replay-verified",
    "engine-started",
    "engine-poll",
    "engine-executed",
    "engine-quiesced",
    "resume-verified",
    "remote-dispatch-sent",
    "remote-consequence-received",
    "direct-engine-bypass",
    "shared-memory-crossing",
    "arazzo-workflow-generated",
    "arazzo-workflow-dispatched",
];

/// splitmix64: deterministic, seedable, dependency-free.
pub(crate) fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub(crate) fn short_hex(v: u64) -> String {
    format!("{v:012x}")[..8].to_string()
}

/// Fills `{{KEY}}` placeholders in a template. Same mechanism as the PDDL
/// domain/problem template substitution in `generate::write_set`.
///
/// # Complexity
/// O(|template| * |pairs|).
pub(crate) fn fill_template(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut body = template.to_string();
    for (key, value) in pairs {
        body = body.replace(&format!("{{{{{key}}}}}"), value);
    }
    body
}

/// Strips the rwai `ex:` prefix from a full IRI, yielding the local name
/// the observation templates re-prefix as `ex:{{...}}`.
pub(crate) fn rwai_local(iri: &str) -> &str {
    iri.strip_prefix(RWAI_PREFIX).unwrap_or(iri)
}

/// Runs `work` over `items` on up to `threads` OS threads, chunked
/// contiguously. No work stealing; deterministic partitioning.
pub(crate) fn parallel_chunks<T: Sync>(items: &[T], threads: usize, work: impl Fn(&T) + Sync) {
    let threads = usize::max(1, threads);
    let chunk = items.len().div_ceil(threads).max(1);
    let work_ref = &work;
    std::thread::scope(|scope| {
        for slice in items.chunks(chunk) {
            scope.spawn(move || {
                for item in slice {
                    work_ref(item);
                }
            });
        }
    });
}
