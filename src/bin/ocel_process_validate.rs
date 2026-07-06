//! src/bin/ocel_process_validate.rs — wasm4pm integrity + POWL conformance
//! over the v26.7.6 OCEL evidence log.
//!
//! Pipeline (library composition, because the wasm4pm CLI's conformance
//! command is stubbed — see `crates/wasm4pm-cli/src/commands/mining.rs:25`
//! in /Users/sac/wasm4pm):
//! 1. Parse the log as `wasm4pm_compat::ocel::OCEL` (OCEL 2.0 Shape A).
//! 2. Integrity gate: `wasm4pm_compat::ocel::validate::validate` with a
//!    permissive (empty) cardinality map — OCEDO/OCPQ Def. 2 invariants.
//! 3. UTC ordering: every `event.time` is an ISO-8601 UTC `Z` instant and
//!    times are non-decreasing in log order.
//! 4. Process conformance: the release-loop model is a
//!    `powl2_decompose::Powl::PartialOrder` whose children are leaves,
//!    single-activity loops, or a loop over a leaf sequence (the benchmark
//!    triple). Because child alphabets are disjoint, exact language
//!    membership of the projected + consecutively-deduped trace is decidable
//!    directly from the model (counts, loop-sequence pattern, and the
//!    all-of-i-before-all-of-j partial order); the decision procedure is
//!    validated differentially against `Powl::language_upto` in unit tests.
//! 5. Object participation: >= 1 each of browser_session, client_surface,
//!    receipt_chain, benchmark_result, screenshot.
//! 6. Emit the report JSON, then append the validator's own bookkeeping
//!    events to the log (idempotent, fixed `val_e*` ids). Closure rule: the
//!    validator validates the log as it existed BEFORE these bookkeeping
//!    events — their types are outside the model alphabet, so projection
//!    drops them on re-runs, and the report hash is taken before the append.
//!
//! Exit 0: conforming. Exit 1: validation findings. Exit 2: refusal
//! (io/parse/model-shape) — every failure is a typed [`Refusal`], no panics.

#![allow(clippy::print_stdout)]
// Recorded lint debt (v26.7.6 verification gate) — same header as
// src/bin/dod.rs; see docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::process::ExitCode;

use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use powl2_decompose::{ChoiceGraph, GNode, Powl};
use serde::Serialize;
use serde_json::{json, Value};
use wasm4pm_compat::ocel::process_conformance::{
    membership_violations, model_alphabet, project_dedupe, ChildKind, ModelView,
};
use wasm4pm_compat::ocel::validate::validate;
use wasm4pm_compat::ocel::{ObjectTypeCardinality, OCEL};

/// Default input: the ONE final OCEL 2.0 evidence log of the release pass.
const DEFAULT_LOG: &str = "docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json";
/// Output report path.
const REPORT_PATH: &str = "docs/releases/v26.7.6/ocel/wasm4pm-process-validation.json";
const RELEASE_ID: &str = "v26.7.6";

/// Event types appended by this validator itself. Outside the model
/// alphabet by construction (closure rule): the conformance projection
/// drops them, so re-running the validator over an already-annotated log
/// still validates the pre-bookkeeping process.
const BOOKKEEPING_TYPES: [&str; 6] = [
    "wasm4pm_process_model_generated",
    "wasm4pm_process_validation_started",
    "wasm4pm_process_validation_completed",
    "wasm4pm_conformance_passed",
    "ocel_log_validated",
    "validation_run_finished",
];

/// Object types that must each have >= 1 instance in the log.
const REQUIRED_OBJECT_TYPES: [&str; 5] = [
    "browser_session",
    "client_surface",
    "receipt_chain",
    "benchmark_result",
    "screenshot",
];

/// Typed refusal — every failure path, no panics, no silent defaults.
#[derive(Debug, thiserror::Error)]
enum Refusal {
    #[error("io on {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("OCEL parse refusal on {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("model shape refusal: {0}")]
    ModelShape(String),
    #[error("append refusal: {0}")]
    Append(String),
}

// ─── the release-loop POWL model ────────────────────────────────────────────

/// Children of the top-level partial order, in canonical order. `Once` is a
/// plain leaf; `AtLeastOnce` a single-activity loop (`a+`); `SeqLoop` a loop
/// over a strict leaf sequence (`(a b c)+`). Mined from the actual event
/// sequence of the v26.7.6 log — see WASM4PM_PROCESS_VALIDATION.md.
const CHILD_SPECS: &[ChildSpec] = &[
    ChildSpec::Once("verifier_gate_invoked"),
    ChildSpec::Once("pddl_plan_requested"),
    ChildSpec::Once("pddl_plan_loaded"),
    ChildSpec::Once("powl_workflow_compiled"),
    ChildSpec::AtLeastOnce("powl_workflow_executed"),
    ChildSpec::Once("bcinr_transition_executed"),
    ChildSpec::AtLeastOnce("ggen_artifact_generated"),
    ChildSpec::AtLeastOnce("verifier_gate_completed"),
    ChildSpec::Once("claim_promoted_to_standing"),
    ChildSpec::Once("receipt_chain_verified"),
    ChildSpec::Once("graphlaw_state_loaded"),
    ChildSpec::Once("graphlaw_export_requested"),
    ChildSpec::Once("validation_run_started"),
    ChildSpec::Once("utc_clock_captured"),
    ChildSpec::Once("playwright_browser_launched"),
    ChildSpec::Once("route_loaded"),
    ChildSpec::Once("api_request_observed"),
    ChildSpec::AtLeastOnce("screenshot_captured"),
    ChildSpec::AtLeastOnce("ui_action_triggered"),
    ChildSpec::Once("trace_captured"),
    ChildSpec::Once("ocel_log_written"),
    ChildSpec::SeqLoop(&[
        "benchmark_run_started",
        "benchmark_run_completed",
        "benchmark_result_attached",
    ]),
];

/// Honest strict order pairs (`all of a before all of b`) mined from the
/// log: a pair is asserted only where the actual trace satisfies it and the
/// dependency is semantically required. Events with overlapping occurrence
/// spans (e.g. `ggen_artifact_generated` vs `verifier_gate_completed`, the
/// screenshot/ui-action interleaving) stay genuinely unordered.
const ORDER_LABEL_PAIRS: &[(&str, &str)] = &[
    ("verifier_gate_invoked", "pddl_plan_loaded"),
    ("pddl_plan_requested", "pddl_plan_loaded"),
    ("pddl_plan_loaded", "powl_workflow_compiled"),
    ("powl_workflow_compiled", "powl_workflow_executed"),
    ("powl_workflow_compiled", "bcinr_transition_executed"),
    ("bcinr_transition_executed", "ggen_artifact_generated"),
    ("bcinr_transition_executed", "verifier_gate_completed"),
    ("powl_workflow_executed", "claim_promoted_to_standing"),
    ("claim_promoted_to_standing", "receipt_chain_verified"),
    ("receipt_chain_verified", "graphlaw_state_loaded"),
    ("graphlaw_state_loaded", "graphlaw_export_requested"),
    ("graphlaw_export_requested", "validation_run_started"),
    ("ggen_artifact_generated", "validation_run_started"),
    ("verifier_gate_completed", "validation_run_started"),
    ("validation_run_started", "utc_clock_captured"),
    ("utc_clock_captured", "playwright_browser_launched"),
    ("playwright_browser_launched", "route_loaded"),
    ("route_loaded", "api_request_observed"),
    ("route_loaded", "screenshot_captured"),
    ("route_loaded", "ui_action_triggered"),
    ("api_request_observed", "trace_captured"),
    ("screenshot_captured", "trace_captured"),
    ("ui_action_triggered", "trace_captured"),
    ("trace_captured", "ocel_log_written"),
    ("ocel_log_written", "benchmark_run_started"),
];

/// Declarative child spec used to build the `Powl` value.
#[derive(Debug, Clone, Copy)]
enum ChildSpec {
    /// exactly one occurrence (plain leaf).
    Once(&'static str),
    /// one or more occurrences (single-activity choice loop).
    AtLeastOnce(&'static str),
    /// one or more repetitions of a strict leaf sequence.
    SeqLoop(&'static [&'static str]),
}

/// `▷ → child → □` with a `child → child` back edge: `L(child)+`.
fn loop_graph() -> ChoiceGraph {
    let mut edges = BTreeSet::new();
    edges.insert((GNode::Start, GNode::Child(0)));
    edges.insert((GNode::Child(0), GNode::Child(0)));
    edges.insert((GNode::Child(0), GNode::End));
    ChoiceGraph { n: 1, edges }
}

fn leaf(label: &str) -> Powl {
    Powl::Leaf(Some(label.to_string()))
}

fn one_or_more(label: &str) -> Powl {
    Powl::Choice {
        children: vec![leaf(label)],
        graph: loop_graph(),
    }
}

fn seq_loop(labels: &[&str]) -> Powl {
    let children: Vec<Powl> = labels.iter().map(|l| leaf(l)).collect();
    let mut order = BTreeSet::new();
    for i in 0..children.len() {
        for j in (i + 1)..children.len() {
            order.insert((i, j)); // transitively closed total order
        }
    }
    Powl::Choice {
        children: vec![Powl::PartialOrder { children, order }],
        graph: loop_graph(),
    }
}

/// Transitive closure; refuses on a cycle (a strict partial order is
/// irreflexive).
fn close_order(
    n: usize,
    base: &BTreeSet<(usize, usize)>,
) -> Result<BTreeSet<(usize, usize)>, Refusal> {
    let mut closed = base.clone();
    loop {
        let mut grew = false;
        let snapshot: Vec<(usize, usize)> = closed.iter().copied().collect();
        for &(a, b) in &snapshot {
            for &(c, d) in &snapshot {
                if b == c && closed.insert((a, d)) {
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }
    for &(a, b) in &closed {
        if a == b {
            return Err(Refusal::ModelShape(format!(
                "order pair cycle through child {a} of {n}"
            )));
        }
    }
    Ok(closed)
}

/// Build the release-loop model as an actual `powl2_decompose::Powl` value.
fn release_loop_model() -> Result<Powl, Refusal> {
    let mut label_to_child: BTreeMap<&str, usize> = BTreeMap::new();
    let children: Vec<Powl> = CHILD_SPECS
        .iter()
        .enumerate()
        .map(|(i, spec)| match spec {
            ChildSpec::Once(l) => {
                label_to_child.insert(l, i);
                leaf(l)
            }
            ChildSpec::AtLeastOnce(l) => {
                label_to_child.insert(l, i);
                one_or_more(l)
            }
            ChildSpec::SeqLoop(ls) => {
                for l in *ls {
                    label_to_child.insert(l, i);
                }
                seq_loop(ls)
            }
        })
        .collect();
    let mut base = BTreeSet::new();
    for (a, b) in ORDER_LABEL_PAIRS {
        let ia = label_to_child.get(a).copied().ok_or_else(|| {
            Refusal::ModelShape(format!("order pair label '{a}' not in any child"))
        })?;
        let ib = label_to_child.get(b).copied().ok_or_else(|| {
            Refusal::ModelShape(format!("order pair label '{b}' not in any child"))
        })?;
        base.insert((ia, ib));
    }
    let order = close_order(children.len(), &base)?;
    Ok(Powl::PartialOrder { children, order })
}

// ─── model view: derive the decision procedure from the Powl value ──────────
//
// `ChildKind`, `ModelView`, `model_alphabet`, `project_dedupe`, and
// `membership_violations` are the generic, POWL-AST-agnostic conformance
// decision procedure — they live in `wasm4pm_compat::ocel::process_conformance`
// (promoted there so other consumers don't have to re-derive the same
// disjoint-alphabet membership algorithm). Only the `Powl` classification
// glue below (`classify_child`/`model_view`, which is coupled to this
// crate's own `powl2_decompose::Powl` AST) and the release-specific model
// (`CHILD_SPECS`, `ORDER_LABEL_PAIRS`, `release_loop_model`) stay here, since
// they describe *this repo's own* release process, not a generic capability.

fn classify_child(child: &Powl) -> Result<ChildKind, Refusal> {
    match child {
        Powl::Leaf(Some(l)) => Ok(ChildKind::Once(l.clone())),
        Powl::Choice { children, graph } if *graph == loop_graph() && children.len() == 1 => {
            match &children[0] {
                Powl::Leaf(Some(l)) => Ok(ChildKind::AtLeastOnce(l.clone())),
                Powl::PartialOrder { children, order } => {
                    let mut labels = Vec::new();
                    for c in children {
                        match c {
                            Powl::Leaf(Some(l)) => labels.push(l.clone()),
                            other => {
                                return Err(Refusal::ModelShape(format!(
                                    "seq-loop inner child is not a labelled leaf: {other:?}"
                                )))
                            }
                        }
                    }
                    let mut want = BTreeSet::new();
                    for i in 0..labels.len() {
                        for j in (i + 1)..labels.len() {
                            want.insert((i, j));
                        }
                    }
                    if *order != want {
                        return Err(Refusal::ModelShape(
                            "seq-loop inner order is not a total chain".to_string(),
                        ));
                    }
                    Ok(ChildKind::SeqLoop(labels))
                }
                other => Err(Refusal::ModelShape(format!(
                    "loop child is neither leaf nor sequence: {other:?}"
                ))),
            }
        }
        other => Err(Refusal::ModelShape(format!(
            "unsupported top-level child shape: {other:?}"
        ))),
    }
}

fn model_view(model: &Powl) -> Result<ModelView, Refusal> {
    let Powl::PartialOrder { children, order } = model else {
        return Err(Refusal::ModelShape(
            "top level is not a PartialOrder".to_string(),
        ));
    };
    let kinds: Vec<ChildKind> = children
        .iter()
        .map(classify_child)
        .collect::<Result<_, _>>()?;
    ModelView::new(kinds, order.clone())
        .map_err(|e| Refusal::ModelShape(e.to_string()))
}

// ─── UTC ordering ───────────────────────────────────────────────────────────

/// Every time string must be RFC 3339 with the literal `Z` suffix, and the
/// parsed instants must be non-decreasing in log order (same-instant events
/// keep their stable log order — string comparison after parse breaks no
/// ties beyond that).
fn utc_violations(events: &[(String, String)]) -> Vec<String> {
    let mut violations = Vec::new();
    let mut prev: Option<(&str, DateTime<FixedOffset>)> = None;
    for (id, time) in events {
        if !time.ends_with('Z') {
            violations.push(format!(
                "event '{id}' time '{time}' is not an ISO-8601 UTC Z instant"
            ));
        }
        match DateTime::parse_from_rfc3339(time) {
            Ok(t) => {
                if let Some((pid, pt)) = prev {
                    if t < pt {
                        violations.push(format!(
                            "event '{id}' time '{time}' decreases below '{pid}'"
                        ));
                    }
                }
                prev = Some((id, t));
            }
            Err(e) => violations.push(format!(
                "event '{id}' time '{time}' failed RFC 3339 parse: {e}"
            )),
        }
    }
    violations
}

// ─── report ─────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct IntegritySummary {
    valid: bool,
    error_count: usize,
    error_codes: Vec<String>,
}

#[derive(Serialize)]
struct ModelReport {
    alphabet: Vec<String>,
    children: Vec<String>,
    order_pairs: Vec<[String; 2]>,
}

#[derive(Serialize)]
struct Report {
    is_conforming: bool,
    fitness: f64,
    violations: Vec<String>,
    integrity_report_summary: IntegritySummary,
    event_count: usize,
    object_count: usize,
    /// SHA-256 of the log bytes as validated (before the bookkeeping append).
    ocel_sha256: String,
    /// BLAKE3 of the same bytes (house receipt hash, invariant 2).
    ocel_blake3: String,
    model: ModelReport,
    method: String,
    closure_rule: String,
    validated_at_utc: String,
}

fn model_report(view: &ModelView) -> ModelReport {
    let child_desc = |k: &ChildKind| match k {
        ChildKind::Once(l) => l.clone(),
        ChildKind::AtLeastOnce(l) => format!("{l}+"),
        ChildKind::SeqLoop(ls) => format!("({})+", ls.join(" ")),
    };
    ModelReport {
        alphabet: model_alphabet(view).into_iter().collect(),
        children: view.children().iter().map(child_desc).collect(),
        order_pairs: view
            .order()
            .iter()
            .map(|&(i, j)| [child_desc(&view.children()[i]), child_desc(&view.children()[j])])
            .collect(),
    }
}

// ─── bookkeeping append (closure rule) ──────────────────────────────────────

/// Append the validator's own evidence events to the log. Idempotent: fixed
/// `val_e*` ids, skipped entirely when `val_e1` already exists. Only called
/// after a conforming validation, so `wasm4pm_conformance_passed` is never
/// asserted without the computation backing it.
fn append_bookkeeping(log_path: &str, report_sha256: &str, run_id: &str) -> Result<bool, Refusal> {
    let raw = std::fs::read_to_string(log_path).map_err(|source| Refusal::Io {
        path: log_path.to_string(),
        source,
    })?;
    let mut root: Value = serde_json::from_str(&raw).map_err(|source| Refusal::Parse {
        path: log_path.to_string(),
        source,
    })?;

    let already = root["events"]
        .as_array()
        .map(|evs| evs.iter().any(|e| e["id"] == "val_e1"))
        .ok_or_else(|| Refusal::Append("log has no events array".to_string()))?;
    if already {
        return Ok(false);
    }

    // Last event time; bookkeeping times are wall-clock UTC (evidence time,
    // not a hash input), clamped to stay non-decreasing.
    let last_time = root["events"]
        .as_array()
        .and_then(|evs| evs.last())
        .and_then(|e| e["time"].as_str())
        .and_then(|t| DateTime::parse_from_rfc3339(t).ok());
    let mut now = Utc::now().fixed_offset();
    if let Some(last) = last_time {
        if now < last {
            now = last;
        }
    }
    let stamp = now.to_rfc3339_opts(SecondsFormat::Millis, true);

    // Declare the bookkeeping event types (attribute decls match the
    // attributes emitted below).
    let decl_attrs = json!([
        { "name": "actor", "type": "string" },
        { "name": "run_id", "type": "string" },
        { "name": "release_id", "type": "string" },
        { "name": "event_source", "type": "string" },
        { "name": "standing_effect", "type": "string" },
        { "name": "evidence_refs", "type": "string" },
        { "name": "report_sha256", "type": "string" }
    ]);
    {
        let decls = root["eventTypes"]
            .as_array_mut()
            .ok_or_else(|| Refusal::Append("log has no eventTypes array".to_string()))?;
        for t in BOOKKEEPING_TYPES {
            if !decls.iter().any(|d| d["name"] == t) {
                decls.push(json!({ "name": t, "attributes": decl_attrs }));
            }
        }
    }

    // Objects the bookkeeping events reference (timestamped attributes per
    // the OCEL 2.0 wire shape).
    let obj_attr = |name: &str, value: &str| json!({ "name": name, "value": value, "time": stamp });
    let new_objects = [
        json!({
            "id": "powl_workflow:wasm4pm_release_loop_model",
            "type": "powl_workflow",
            "attributes": [
                obj_attr("object_label", "release-loop POWL 2.0 process model"),
                obj_attr("object_source", "src/bin/ocel_process_validate.rs"),
                obj_attr("standing", "evidence"),
                obj_attr("created_or_observed_by", "ocel_process_validate"),
                obj_attr("evidence_refs", REPORT_PATH),
            ],
            "relationships": []
        }),
        json!({
            "id": "report:wasm4pm_process_validation",
            "type": "report_artifact",
            "attributes": [
                obj_attr("object_label", "wasm4pm process-validation report"),
                obj_attr("object_source", "src/bin/ocel_process_validate.rs"),
                obj_attr("standing", "evidence"),
                obj_attr("created_or_observed_by", "ocel_process_validate"),
                obj_attr("path", REPORT_PATH),
                obj_attr("object_hash", report_sha256),
                obj_attr("evidence_refs", REPORT_PATH),
            ],
            "relationships": []
        }),
    ];
    {
        let objects = root["objects"]
            .as_array_mut()
            .ok_or_else(|| Refusal::Append("log has no objects array".to_string()))?;
        for o in new_objects {
            if !objects.iter().any(|x| x["id"] == o["id"]) {
                objects.push(o);
            }
        }
    }

    let mk_event = |seq: usize, ev_type: &str, object_id: &str, qualifier: &str| {
        json!({
            "id": format!("val_e{seq}"),
            "type": ev_type,
            "time": stamp,
            "attributes": [
                { "name": "actor", "value": "ocel_process_validate" },
                { "name": "run_id", "value": run_id },
                { "name": "release_id", "value": RELEASE_ID },
                { "name": "event_source", "value": "src/bin/ocel_process_validate.rs" },
                { "name": "standing_effect", "value": if ev_type == "wasm4pm_conformance_passed" { "claim_promoted" } else { "none" } },
                { "name": "evidence_refs", "value": REPORT_PATH },
                { "name": "report_sha256", "value": report_sha256 }
            ],
            "relationships": [
                { "objectId": object_id, "qualifier": qualifier }
            ]
        })
    };
    const MODEL_OBJ: &str = "powl_workflow:wasm4pm_release_loop_model";
    const REPORT_OBJ: &str = "report:wasm4pm_process_validation";
    let new_events = [
        mk_event(1, "wasm4pm_process_model_generated", MODEL_OBJ, "model"),
        mk_event(
            2,
            "wasm4pm_process_validation_started",
            REPORT_OBJ,
            "report",
        ),
        mk_event(
            3,
            "wasm4pm_process_validation_completed",
            REPORT_OBJ,
            "report",
        ),
        mk_event(4, "wasm4pm_conformance_passed", MODEL_OBJ, "model"),
        mk_event(5, "ocel_log_validated", REPORT_OBJ, "report"),
        mk_event(6, "validation_run_finished", REPORT_OBJ, "report"),
    ];
    {
        let events = root["events"]
            .as_array_mut()
            .ok_or_else(|| Refusal::Append("log has no events array".to_string()))?;
        events.extend(new_events);
    }

    let mut out = serde_json::to_string_pretty(&root).map_err(|source| Refusal::Parse {
        path: log_path.to_string(),
        source,
    })?;
    out.push('\n');
    std::fs::write(log_path, out).map_err(|source| Refusal::Io {
        path: log_path.to_string(),
        source,
    })?;
    Ok(true)
}

// ─── main ───────────────────────────────────────────────────────────────────

fn run() -> Result<bool, Refusal> {
    let log_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_LOG.to_string());

    let bytes = std::fs::read(&log_path).map_err(|source| Refusal::Io {
        path: log_path.clone(),
        source,
    })?;
    let ocel_sha256 = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(&bytes);
        hex_encode(&h.finalize())
    };
    let ocel_blake3 = blake3::hash(&bytes).to_hex().to_string();

    let text = String::from_utf8_lossy(&bytes);
    let ocel: OCEL = serde_json::from_str(&text).map_err(|source| Refusal::Parse {
        path: log_path.clone(),
        source,
    })?;
    // Raw time strings for the literal-Z check (chrono normalizes offsets).
    let raw: Value = serde_json::from_str(&text).map_err(|source| Refusal::Parse {
        path: log_path.clone(),
        source,
    })?;

    let mut violations: Vec<String> = Vec::new();

    // 1. integrity gate (permissive default cardinality: none declared)
    let cardinality: HashMap<String, ObjectTypeCardinality> = HashMap::new();
    let integrity = validate(&ocel, &cardinality);
    for e in &integrity.errors {
        violations.push(format!("integrity {}: {}", e.code, e.message));
    }

    // 2. UTC ordering over raw time strings, in log order
    let raw_times: Vec<(String, String)> = raw["events"]
        .as_array()
        .map(|evs| {
            evs.iter()
                .map(|e| {
                    (
                        e["id"].as_str().unwrap_or_default().to_string(),
                        e["time"].as_str().unwrap_or_default().to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let utc_viols = utc_violations(&raw_times);
    violations.extend(utc_viols.iter().cloned());

    // 3. process conformance
    let model = release_loop_model()?;
    let view = model_view(&model)?;
    let alphabet = model_alphabet(&view);
    let types: Vec<String> = ocel.events.iter().map(|e| e.event_type.clone()).collect();
    let projected = project_dedupe(&types, &alphabet);
    let member_viols = membership_violations(&projected, &view);
    let model_checks_total = view.children().len() + view.order().len();
    let fitness = if member_viols.is_empty() {
        1.0
    } else {
        let failed = member_viols.len().min(model_checks_total);
        (model_checks_total - failed) as f64 / model_checks_total as f64
    };
    violations.extend(member_viols.iter().map(|v| format!("conformance: {v}")));

    // 4. object participation
    for t in REQUIRED_OBJECT_TYPES {
        if ocel.count_objects_of_type(t) == 0 {
            violations.push(format!("participation: no object of type '{t}'"));
        }
    }

    let is_conforming = violations.is_empty();
    let report = Report {
        is_conforming,
        fitness,
        violations: violations.clone(),
        integrity_report_summary: IntegritySummary {
            valid: integrity.valid,
            error_count: integrity.errors.len(),
            error_codes: integrity.errors.iter().map(|e| e.code.clone()).collect(),
        },
        event_count: ocel.events.len(),
        object_count: ocel.objects.len(),
        ocel_sha256: ocel_sha256.clone(),
        ocel_blake3,
        model: model_report(&view),
        method: "library composition: wasm4pm_compat::ocel::validate (OCEDO/OCPQ \
                 integrity) + powl2_decompose::Powl release-loop model with a direct \
                 membership decision procedure (exact for disjoint child alphabets; \
                 differentially validated against Powl::language_upto in unit tests). \
                 The wasm4pm CLI conformance command is stubbed \
                 (crates/wasm4pm-cli/src/commands/mining.rs:25), hence in-process \
                 composition."
            .to_string(),
        closure_rule: "the validator validates the log as it existed BEFORE its own \
                       val_e* bookkeeping events: ocel_sha256/ocel_blake3 are taken \
                       before the append, and the bookkeeping event types are outside \
                       the model alphabet so conformance projection drops them on \
                       re-runs."
            .to_string(),
        validated_at_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
    };

    let mut report_json =
        serde_json::to_string_pretty(&report).map_err(|source| Refusal::Parse {
            path: REPORT_PATH.to_string(),
            source,
        })?;
    report_json.push('\n');
    std::fs::write(REPORT_PATH, &report_json).map_err(|source| Refusal::Io {
        path: REPORT_PATH.to_string(),
        source,
    })?;
    println!("{report_json}");

    if is_conforming {
        let report_sha256 = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(report_json.as_bytes());
            hex_encode(&h.finalize())
        };
        let run_id = ocel
            .events
            .first()
            .and_then(|e| {
                e.attributes
                    .iter()
                    .find(|a| a.name == "run_id")
                    .map(|a| a.value.to_string())
            })
            .unwrap_or_else(|| RELEASE_ID.to_string());
        let appended = append_bookkeeping(&log_path, &report_sha256, &run_id)?;
        eprintln!(
            "[ocel_process_validate] conforming; bookkeeping {}",
            if appended {
                "appended (val_e1..val_e6)"
            } else {
                "already present, skipped"
            }
        );
    } else {
        eprintln!(
            "[ocel_process_validate] NOT conforming: {} violation(s)",
            violations.len()
        );
    }
    Ok(is_conforming)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(refusal) => {
            eprintln!("[ocel_process_validate] refusal: {refusal}");
            ExitCode::from(2)
        }
    }
}

// ─── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    /// The canonical projected+deduped trace of the v26.7.6 log.
    fn canonical_trace() -> Vec<String> {
        s(&[
            "verifier_gate_invoked",
            "pddl_plan_requested",
            "pddl_plan_loaded",
            "powl_workflow_compiled",
            "powl_workflow_executed",
            "bcinr_transition_executed",
            "ggen_artifact_generated",
            "verifier_gate_completed",
            "powl_workflow_executed",
            "claim_promoted_to_standing",
            "receipt_chain_verified",
            "graphlaw_state_loaded",
            "verifier_gate_completed",
            "graphlaw_export_requested",
            "ggen_artifact_generated",
            "validation_run_started",
            "utc_clock_captured",
            "playwright_browser_launched",
            "route_loaded",
            "api_request_observed",
            "screenshot_captured",
            "ui_action_triggered",
            "screenshot_captured",
            "ui_action_triggered",
            "screenshot_captured",
            "ui_action_triggered",
            "trace_captured",
            "ocel_log_written",
            "benchmark_run_started",
            "benchmark_run_completed",
            "benchmark_result_attached",
            "benchmark_run_started",
            "benchmark_run_completed",
            "benchmark_result_attached",
            "benchmark_run_started",
            "benchmark_run_completed",
            "benchmark_result_attached",
        ])
    }

    #[test]
    fn canonical_trace_is_a_member() {
        let model = release_loop_model().expect("model builds");
        let view = model_view(&model).expect("model classifies");
        let v = membership_violations(&canonical_trace(), &view);
        assert!(v.is_empty(), "unexpected violations: {v:?}");
    }

    #[test]
    fn missing_required_event_is_rejected() {
        let model = release_loop_model().expect("model builds");
        let view = model_view(&model).expect("model classifies");
        let trace: Vec<String> = canonical_trace()
            .into_iter()
            .filter(|t| t != "ocel_log_written")
            .collect();
        let v = membership_violations(&trace, &view);
        assert!(
            v.iter().any(|m| m.contains("'ocel_log_written' occurs 0")),
            "expected missing-event violation, got {v:?}"
        );
    }

    #[test]
    fn order_violation_is_rejected() {
        let model = release_loop_model().expect("model builds");
        let view = model_view(&model).expect("model classifies");
        // Move validation_run_started to the very front: the driver phase
        // must precede it in the honest model.
        let mut trace = canonical_trace();
        let pos = trace
            .iter()
            .position(|t| t == "validation_run_started")
            .expect("present");
        let ev = trace.remove(pos);
        trace.insert(0, ev);
        let v = membership_violations(&trace, &view);
        assert!(!v.is_empty(), "expected order violation");
    }

    #[test]
    fn repeated_once_event_is_rejected() {
        let model = release_loop_model().expect("model builds");
        let view = model_view(&model).expect("model classifies");
        let mut trace = canonical_trace();
        trace.push("ocel_log_written".to_string()); // non-consecutive repeat survives dedupe
        let v = membership_violations(&trace, &view);
        assert!(
            v.iter().any(|m| m.contains("'ocel_log_written' occurs 2")),
            "expected exactly-once violation, got {v:?}"
        );
    }

    #[test]
    fn broken_benchmark_pattern_is_rejected() {
        let model = release_loop_model().expect("model builds");
        let view = model_view(&model).expect("model classifies");
        // Swap one completed/attached pair: (s a c) is not (s c a)+.
        let mut trace = canonical_trace();
        let n = trace.len();
        trace.swap(n - 1, n - 2);
        let v = membership_violations(&trace, &view);
        assert!(
            v.iter().any(|m| m.contains("loop-sequence")),
            "expected loop-sequence violation, got {v:?}"
        );
    }

    #[test]
    fn project_dedupe_drops_foreign_and_collapses_repeats() {
        let alphabet: BTreeSet<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        let trace = s(&["a", "a", "x", "a", "b", "b", "y", "b"]);
        // the two runs of 'a' are separated only by a non-alphabet symbol,
        // so they collapse; same for 'b'.
        assert_eq!(project_dedupe(&trace, &alphabet), s(&["a", "b"]));
    }

    #[test]
    fn utc_parser_accepts_z_and_rejects_offsets_and_regressions() {
        let ok = vec![
            ("e1".to_string(), "2026-07-06T19:10:43.285Z".to_string()),
            ("e2".to_string(), "2026-07-06T19:10:43.285Z".to_string()),
            ("e3".to_string(), "2026-07-06T19:13:25Z".to_string()),
        ];
        assert!(utc_violations(&ok).is_empty());

        let offset = vec![("e1".to_string(), "2026-07-06T19:10:43+02:00".to_string())];
        assert!(utc_violations(&offset)
            .iter()
            .any(|v| v.contains("not an ISO-8601 UTC Z")));

        let garbage = vec![("e1".to_string(), "yesterday".to_string())];
        assert!(!utc_violations(&garbage).is_empty());

        let regress = vec![
            ("e1".to_string(), "2026-07-06T19:14:00Z".to_string()),
            ("e2".to_string(), "2026-07-06T19:13:00Z".to_string()),
        ];
        assert!(utc_violations(&regress)
            .iter()
            .any(|v| v.contains("decreases")));
    }

    /// Differential grounding: on a small model of the same shape class, the
    /// direct membership procedure agrees with `Powl::language_upto` for
    /// every sequence over the alphabet up to length 5.
    #[test]
    fn membership_agrees_with_language_upto() {
        // a ; b+ ; (c d)+  with a ≺ b ≺ (c d)
        let children = vec![leaf("a"), one_or_more("b"), seq_loop(&["c", "d"])];
        let mut order = BTreeSet::new();
        order.insert((0, 1));
        order.insert((1, 2));
        order.insert((0, 2)); // transitive closure
        let model = Powl::PartialOrder { children, order };
        let view = model_view(&model).expect("small model classifies");

        let max_len = 5;
        let language = model.language_upto(max_len);

        let symbols = ["a", "b", "c", "d"];
        let mut stack: Vec<Vec<String>> = vec![vec![]];
        while let Some(t) = stack.pop() {
            let member = membership_violations(&t, &view).is_empty();
            let in_language = language.contains(&t);
            assert_eq!(
                member, in_language,
                "membership procedure disagrees with language_upto on {t:?}"
            );
            if t.len() < max_len {
                for sym in symbols {
                    let mut ext = t.clone();
                    ext.push(sym.to_string());
                    stack.push(ext);
                }
            }
        }
    }
}
