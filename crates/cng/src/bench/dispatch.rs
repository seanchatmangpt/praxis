//! External workflow dispatch with lawful re-entry (PROJ-618/619/620).
//!
//! The broker choke point (`workday::actuate_transitions` →
//! `WorkdayHookBroker::actuate`) gains a dispatch adapter beside local hook
//! actuation, behind the same zero-unreceipted law: every outbound dispatch
//! and every inbound consequence is receipted as an observation before it
//! has any effect, and both digests fold into the evidence chain.
//!
//! Routing rule (deterministic, category-mapped — documented per PROJ-619):
//! - `software-delivery` → `EXTERNAL_MACHINE_DISPATCH` (recursive depth 1,
//!   closure law `ALL_CHILDREN_REQUIRED` — exercises PROJ-620 child
//!   dispatch + closure),
//! - `api-orchestration` → `EXTERNAL_MACHINE_DISPATCH` via the Arazzo step
//!   projection (`bench::arazzo`, PROJ-621),
//! - `purchase-order-approval` → `EXTERNAL_HUMAN_DISPATCH` (depth 0; the
//!   loopback consequence is labeled MOCKED-HUMAN),
//! - every other category → `LOCAL_ACTUATION` (hook actuation only).
//!
//! Loopback adapter (decision 3 of the release plan): "external" targets are
//! a local filesystem surface — the outbound contract is rendered to
//! `<out_dir>/dispatch/outbox/<id>.ttl` and the consequence is
//! deterministically synthesized (content-derived delay, seed-free) into
//! `<out_dir>/dispatch/inbox/<id>.ttl`. `synthesize_consequence` is the
//! single seam where a real network adapter replaces synthesis: the
//! dispatch/re-entry MECHANISM is ALIVE; live third-party endpoints are out
//! of scope (UNVERIFIED) and human consequences are MOCKED-HUMAN.
//!
//! Return path, enforced IN ORDER (`collect_consequence`): provenance
//! verification → identity/correlation check → authority verification →
//! structural validation → semantic conformance → admission, else typed
//! `CNG_R17 ExternalConsequenceRefused { dispatch, stage }`. The external
//! result never touches standing before admission.
//!
//! Bounded polling: the poll loop is `for poll in 0..deadline_ticks` — the
//! loop bound IS the contract deadline (logical ticks), so unbounded polling
//! is structurally impossible; every poll is a receipted observation.
//!
//! Closure (PROJ-620): parents declare one `disp:ClosureLaw`; satisfaction
//! is read from the on-disk `queries/dispatch-closure.rq` SELECT over
//! admitted child consequences, never inferred in Rust control flow. A
//! parent whose law is unsatisfied stays open (BLOCKED), it never completes.
//!
//! Determinism: every id, delay, and digest is content-derived (BLAKE3 over
//! contract text/ids); no wall clock anywhere near a digest.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::NamedNodeRef;
use oxigraph::store::Store;

use crate::powl::CngRefusal;

use super::roles::{select_rows, ObsWriter};
use super::templates::QuerySet;
use super::workday::expect_standing_rows;
use super::{fill_template, RWAI_PREFIX};

/// Loopback synthesis delay modulus: the deterministic consequence arrives
/// at poll number `blake3(correlationId) % SYNTH_DELAY_MOD`. Workday
/// contracts use `deadline_ticks = SYNTH_DELAY_MOD * 2`, so a lawful
/// loopback consequence always arrives before the deadline; timeout is
/// exercised by contracts whose deadline is below the synthesized delay.
pub(super) const SYNTH_DELAY_MOD: u64 = 4;

/// Bounded child fan-out for recursive dispatch (PROJ-620). Together with
/// the contract's `recursive_depth` this bounds the child tree at
/// `CHILD_FAN_OUT^recursive_depth` dispatches.
pub(super) const CHILD_FAN_OUT: usize = 2;

/// The three execution classes a manufactured action routes to (PROJ-619).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExecutionClass {
    /// In-process hook actuation only (the PROJ-612 path).
    LocalActuation,
    /// Dispatch to an external machine surface (loopback adapter here).
    ExternalMachineDispatch,
    /// Dispatch to an external human surface (loopback, MOCKED-HUMAN).
    ExternalHumanDispatch,
}

impl ExecutionClass {
    /// Shape-vocabulary name (dispatch-shapes.ttl individual local name).
    pub(super) fn as_str(self) -> &'static str {
        match self {
            ExecutionClass::LocalActuation => "LOCAL_ACTUATION",
            ExecutionClass::ExternalMachineDispatch => "EXTERNAL_MACHINE_DISPATCH",
            ExecutionClass::ExternalHumanDispatch => "EXTERNAL_HUMAN_DISPATCH",
        }
    }
}

/// Deterministic per-category routing rule (see module docs). O(1).
pub(super) fn route_category(category: &str) -> ExecutionClass {
    match category {
        "software-delivery" | "api-orchestration" => ExecutionClass::ExternalMachineDispatch,
        "purchase-order-approval" => ExecutionClass::ExternalHumanDispatch,
        _ => ExecutionClass::LocalActuation,
    }
}

/// The 13-state dispatch machine (mirrors the `disp:DispatchState`
/// individuals in `shapes/dispatch-shapes.ttl`).
///
/// Lawful transition table (anything else is `CNG_R16`):
/// - MANUFACTURED → DISPATCH_READY
/// - DISPATCH_READY → DISPATCHED | REFUSED
/// - DISPATCHED → ACKNOWLEDGED | TIMED_OUT
/// - ACKNOWLEDGED → IN_PROGRESS | TIMED_OUT
/// - IN_PROGRESS → RESULT_RETURNED | TIMED_OUT | BLOCKED
/// - RESULT_RETURNED → ADMITTED | REFUSED
/// - ADMITTED → COMPLETED
/// - REFUSED → COMPENSATING | BLOCKED
/// - TIMED_OUT → COMPENSATING | BLOCKED
/// - COMPENSATING → COMPLETED | BLOCKED
/// - COMPLETED, BLOCKED, UNKNOWN → (terminal; no lawful exits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DispatchState {
    Manufactured,
    DispatchReady,
    Dispatched,
    Acknowledged,
    InProgress,
    ResultReturned,
    Admitted,
    Completed,
    Refused,
    TimedOut,
    Compensating,
    Blocked,
    /// Declared to mirror the 13th `disp:DispatchState` individual
    /// (dispatch-shapes.ttl); the broker never constructs it — an unknowable
    /// state is a refusal, not a value — but the vocabulary mirror is total.
    #[allow(dead_code)]
    Unknown,
}

impl DispatchState {
    /// Shape-vocabulary name (dispatch-shapes.ttl individual local name).
    pub(super) fn as_str(self) -> &'static str {
        match self {
            DispatchState::Manufactured => "MANUFACTURED",
            DispatchState::DispatchReady => "DISPATCH_READY",
            DispatchState::Dispatched => "DISPATCHED",
            DispatchState::Acknowledged => "ACKNOWLEDGED",
            DispatchState::InProgress => "IN_PROGRESS",
            DispatchState::ResultReturned => "RESULT_RETURNED",
            DispatchState::Admitted => "ADMITTED",
            DispatchState::Completed => "COMPLETED",
            DispatchState::Refused => "REFUSED",
            DispatchState::TimedOut => "TIMED_OUT",
            DispatchState::Compensating => "COMPENSATING",
            DispatchState::Blocked => "BLOCKED",
            DispatchState::Unknown => "UNKNOWN",
        }
    }

    /// Whether `self → to` is in the lawful transition table (see type
    /// docs). O(1).
    fn lawful_to(self, to: DispatchState) -> bool {
        use DispatchState as S;
        matches!(
            (self, to),
            (S::Manufactured, S::DispatchReady)
                | (S::DispatchReady, S::Dispatched | S::Refused)
                | (S::Dispatched, S::Acknowledged | S::TimedOut)
                | (S::Acknowledged, S::InProgress | S::TimedOut)
                | (S::InProgress, S::ResultReturned | S::TimedOut | S::Blocked)
                | (S::ResultReturned, S::Admitted | S::Refused)
                | (S::Admitted, S::Completed)
                | (S::Refused, S::Compensating | S::Blocked)
                | (S::TimedOut, S::Compensating | S::Blocked)
                | (S::Compensating, S::Completed | S::Blocked)
        )
    }
}

/// A typed dispatch contract: the 20 required fields of
/// `DispatchContractShape` (dispatch-shapes.ttl) plus the state-machine
/// cursor, execution class, optional closure law, and the parent dispatch
/// id (observation lineage; "none" at top level). Deadlines are LOGICAL
/// ticks, never wall clock.
#[derive(Debug, Clone)]
pub(super) struct DispatchContract {
    pub(super) dispatch_id: String,
    pub(super) workflow_instance: String,
    pub(super) parent_workflow: String,
    pub(super) recursive_depth: u32,
    pub(super) target_actor: String,
    pub(super) required_role: String,
    pub(super) declared_authority: String,
    pub(super) input_artifact_set: String,
    pub(super) expected_output_artifact_set: String,
    pub(super) activity_identity: String,
    pub(super) deadline_ticks: u64,
    pub(super) idempotency_key: String,
    pub(super) correlation_id: String,
    pub(super) collection_surface: String,
    pub(super) retry_law: String,
    pub(super) escalation_law: String,
    pub(super) compensation_law: String,
    pub(super) refusal_conditions: String,
    pub(super) receipt_requirements: String,
    pub(super) replay_requirements: String,
    /// State-machine cursor; advanced only through [`Self::advance`].
    pub(super) state: DispatchState,
    pub(super) execution_class: ExecutionClass,
    /// Parent-child closure law (PROJ-620); `None` for childless contracts.
    pub(super) closure_law: Option<&'static str>,
    /// Parent DISPATCH id for observation lineage ("none" at top level).
    pub(super) parent_dispatch: String,
}

impl DispatchContract {
    /// Advances the state machine or refuses `CNG_R16` on an unlawful
    /// transition. O(1).
    pub(super) fn advance(&mut self, to: DispatchState) -> Result<(), CngRefusal> {
        if !self.state.lawful_to(to) {
            return Err(CngRefusal::DispatchStateUnlawful {
                dispatch: self.dispatch_id.clone(),
                from: self.state.as_str().to_string(),
                to: to.as_str().to_string(),
            });
        }
        self.state = to;
        Ok(())
    }

    /// The 18 string-valued required fields as (template placeholder,
    /// value) pairs, in template order. O(1).
    fn string_fields(&self) -> [(&'static str, &str); 18] {
        [
            ("DISPATCH_ID", &self.dispatch_id),
            ("WORKFLOW_INSTANCE", &self.workflow_instance),
            ("PARENT_WORKFLOW", &self.parent_workflow),
            ("TARGET_ACTOR", &self.target_actor),
            ("REQUIRED_ROLE", &self.required_role),
            ("DECLARED_AUTHORITY", &self.declared_authority),
            ("INPUT_ARTIFACT_SET", &self.input_artifact_set),
            (
                "EXPECTED_OUTPUT_ARTIFACT_SET",
                &self.expected_output_artifact_set,
            ),
            ("ACTIVITY_IDENTITY", &self.activity_identity),
            ("IDEMPOTENCY_KEY", &self.idempotency_key),
            ("CORRELATION_ID", &self.correlation_id),
            ("COLLECTION_SURFACE", &self.collection_surface),
            ("RETRY_LAW", &self.retry_law),
            ("ESCALATION_LAW", &self.escalation_law),
            ("COMPENSATION_LAW", &self.compensation_law),
            ("REFUSAL_CONDITIONS", &self.refusal_conditions),
            ("RECEIPT_REQUIREMENTS", &self.receipt_requirements),
            ("REPLAY_REQUIREMENTS", &self.replay_requirements),
        ]
    }

    /// Renders the contract through the on-disk
    /// `templates/dispatch-contract.template.ttl`. Every missing/empty
    /// required field refuses `CNG_R15 DispatchContractIncomplete` BEFORE
    /// the contract can leave the broker.
    ///
    /// # Complexity
    /// O(|template| × fields) placeholder substitution.
    pub(super) fn render(&self, template: &str) -> Result<String, CngRefusal> {
        let missing: Vec<&str> = self
            .string_fields()
            .iter()
            .filter(|(_, v)| v.trim().is_empty())
            .map(|(k, _)| *k)
            .collect();
        if !missing.is_empty() {
            return Err(CngRefusal::DispatchContractIncomplete {
                dispatch: self.dispatch_id.clone(),
                missing: missing.join(","),
            });
        }
        let depth = self.recursive_depth.to_string();
        let deadline = self.deadline_ticks.to_string();
        let mut pairs: Vec<(&str, &str)> = self.string_fields().to_vec();
        pairs.push(("RECURSIVE_DEPTH", depth.as_str()));
        pairs.push(("DEADLINE_TICKS", deadline.as_str()));
        pairs.push(("DISPATCH_STATE", self.state.as_str()));
        pairs.push(("EXECUTION_CLASS", self.execution_class.as_str()));
        Ok(fill_template(template, &pairs))
    }
}

/// Short content-derived key: `blake3(text)` first 12 hex chars. O(|text|).
fn content_key(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex()[..12].to_string()
}

/// Builds the standard workday dispatch contract for one manufactured set.
/// Every field is content-derived from `(set_id, category, tick)` — never
/// path- or time-derived — so two same-seed runs render byte-identical
/// contracts. O(1).
pub(super) fn workday_contract(
    set_id: &str,
    category: &str,
    tick: usize,
    class: ExecutionClass,
) -> DispatchContract {
    let dispatch_id = format!("disp-{set_id}");
    let (target_actor, required_role, depth, closure_law) = match class {
        ExecutionClass::ExternalHumanDispatch => (
            // MOCKED-HUMAN: the loopback adapter synthesizes this actor's
            // consequence deterministically; the granting human is simulated.
            "external-human-approver",
            "approver",
            0,
            None,
        ),
        _ => (
            "external-machine-executor",
            "operator",
            1,
            Some("ALL_CHILDREN_REQUIRED"),
        ),
    };
    DispatchContract {
        dispatch_id: dispatch_id.clone(),
        workflow_instance: set_id.to_string(),
        parent_workflow: set_id.to_string(),
        recursive_depth: depth,
        target_actor: target_actor.to_string(),
        required_role: required_role.to_string(),
        declared_authority: format!("workday-operator-authority-{category}"),
        input_artifact_set: format!("inputs-{set_id}"),
        expected_output_artifact_set: format!("outputs-{dispatch_id}"),
        activity_identity: format!("{category}-external-{tick}"),
        deadline_ticks: SYNTH_DELAY_MOD * 2,
        idempotency_key: format!("idem-{}", content_key(&format!("idem|{dispatch_id}"))),
        correlation_id: format!("corr-{}", content_key(&format!("corr|{dispatch_id}"))),
        collection_surface: "dispatch/inbox".to_string(),
        retry_law: "retry:limit=0;declarative-only".to_string(),
        escalation_law: "escalate:manufacture-escalation-workflow".to_string(),
        compensation_law: "compensate:manufacture-compensation-workflow".to_string(),
        refusal_conditions: "provenance,correlation,authority,structural,semantic".to_string(),
        receipt_requirements: "obs-receipt-per-transition".to_string(),
        replay_requirements: "byte-identical-same-seed".to_string(),
        state: DispatchState::Manufactured,
        execution_class: class,
        closure_law,
        parent_dispatch: "none".to_string(),
    }
}

/// Derives child dispatch contract `c` of `parent` (PROJ-620): depth − 1,
/// authority propagated from the parent, ids content-derived from the
/// parent's ids. O(1).
fn child_contract(parent: &DispatchContract, c: usize) -> DispatchContract {
    let dispatch_id = format!("{}-c{c}", parent.dispatch_id);
    DispatchContract {
        dispatch_id: dispatch_id.clone(),
        workflow_instance: format!("{}-c{c}", parent.workflow_instance),
        parent_workflow: parent.workflow_instance.clone(),
        recursive_depth: parent.recursive_depth.saturating_sub(1),
        // Declared authority propagates from the parent contract.
        declared_authority: parent.declared_authority.clone(),
        expected_output_artifact_set: format!("outputs-{dispatch_id}"),
        input_artifact_set: format!("inputs-{dispatch_id}"),
        idempotency_key: format!("idem-{}", content_key(&format!("idem|{dispatch_id}"))),
        correlation_id: format!("corr-{}", content_key(&format!("corr|{dispatch_id}"))),
        activity_identity: format!("{}-child-{c}", parent.activity_identity),
        state: DispatchState::Manufactured,
        closure_law: None,
        parent_dispatch: parent.dispatch_id.clone(),
        ..parent.clone()
    }
}

/// Shape-driven structural validation: loads `shapes_ttl_path` and
/// `instance_ttl` into ONE store and runs the two generic shape queries
/// (`registry-missing-fields.rq` for required fields,
/// `shape-closed-violations.rq` for closedness honoring
/// `sh:ignoredProperties`). Returns the `(entry, field)` violation rows;
/// callers map them to their typed refusal.
///
/// # Complexity
/// O(shape + instance triples) load + two SELECTs over the fixed graph.
pub(super) fn shape_violations(
    instance_ttl: &str,
    shapes_ttl_path: &Path,
    queries: &QuerySet,
) -> Result<Vec<(String, String)>, CngRefusal> {
    let store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("shape store construction: {e}")))?;
    let shapes_ttl = fs::read_to_string(shapes_ttl_path)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", shapes_ttl_path.display())))?;
    for text in [shapes_ttl.as_str(), instance_ttl] {
        store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), text.as_bytes())
            .map_err(|e| CngRefusal::MalformedTtl(format!("shape-validation load: {e}")))?;
    }
    let mut out = Vec::new();
    for query_name in ["registry-missing-fields", "shape-closed-violations"] {
        for row in select_rows(&store, queries.get(query_name)?)? {
            let bound = |var: &str| -> Result<String, CngRefusal> {
                row.get(var).cloned().ok_or_else(|| {
                    CngRefusal::MalformedTtl(format!("{query_name}.rq row missing ?{var}"))
                })
            };
            out.push((bound("entry")?, bound("field")?));
        }
    }
    Ok(out)
}

/// The disp: vocabulary prefix (templates/dispatch-*.template.ttl).
const DISP_PREFIX: &str = "https://truex.io/ontology/dispatch#";
/// PROV-O prefix (consequence provenance layering).
const PROV_PREFIX: &str = "http://www.w3.org/ns/prov#";

/// First object value of `(?, <DISP_PREFIX+local>, ?o)` in `store`, plain.
/// O(1) pattern lookup.
fn disp_object(
    store: &Store,
    local: &str,
    full_prefix: &str,
) -> Result<Option<String>, CngRefusal> {
    let iri = format!("{full_prefix}{local}");
    let pred =
        NamedNodeRef::new(&iri).map_err(|e| CngRefusal::MalformedTtl(format!("{iri}: {e}")))?;
    match store.quads_for_pattern(None, Some(pred), None, None).next() {
        Some(Ok(quad)) => Ok(Some(super::manufacture::term_value(&quad.object))),
        Some(Err(e)) => Err(CngRefusal::MalformedTtl(format!("consequence scan: {e}"))),
        None => Ok(None),
    }
}

/// Lawful re-entry pipeline for one inbound consequence, enforced IN ORDER:
/// 1. provenance — `disp:producingActor` matches the contract's
///    `disp:targetActor`;
/// 2. correlation — `disp:consequenceOf` equals the contract's
///    `correlationId` (forged/unknown correlation refuses here);
/// 3. authority — the consequence derives from the contract
///    (`prov:wasDerivedFrom` names the dispatch) and carries a provenance
///    bundle, satisfying the contract's declared authority;
/// 4. structural — the consequence conforms to `DispatchConsequenceShape`
///    (shape-driven SPARQL, `shapes/dispatch-shapes.ttl`);
/// 5. semantic — the returned artifact is the contract's expected output
///    artifact set.
///
/// # Errors
/// `CNG_R17 ExternalConsequenceRefused { dispatch, stage }` at the FIRST
/// failing stage; nothing later runs. On `Ok(())` the caller may admit.
///
/// # Complexity
/// O(consequence triples) load + O(1) pattern scans + two shape SELECTs.
pub(super) fn collect_consequence(
    consequence_ttl: &str,
    contract: &DispatchContract,
    shapes_ttl_path: &Path,
    queries: &QuerySet,
) -> Result<(), CngRefusal> {
    let refuse = |stage: &str| CngRefusal::ExternalConsequenceRefused {
        dispatch: contract.dispatch_id.clone(),
        stage: stage.to_string(),
    };
    let store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("consequence store construction: {e}")))?;
    store
        .load_from_slice(
            RdfParser::from_format(RdfFormat::Turtle),
            consequence_ttl.as_bytes(),
        )
        .map_err(|_| refuse("structural"))?;

    // Stage 1: provenance — producing actor must be the contracted target.
    let producing = disp_object(&store, "producingActor", DISP_PREFIX)?;
    let expected_actor = format!("{RWAI_PREFIX}{}", contract.target_actor);
    if producing.as_deref() != Some(expected_actor.as_str()) {
        return Err(refuse("provenance"));
    }
    // Stage 2: identity/correlation — consequenceOf must equal the sealed
    // correlation id; a forged or unknown correlation refuses here.
    let correlation = disp_object(&store, "consequenceOf", DISP_PREFIX)?;
    if correlation.as_deref() != Some(contract.correlation_id.as_str()) {
        return Err(refuse("correlation"));
    }
    // Stage 3: authority — the consequence must derive from THIS dispatch
    // (prov:wasDerivedFrom) and carry a provenance bundle; the contract's
    // declared authority must be non-empty (sealed at manufacture).
    let derived_from = disp_object(&store, "wasDerivedFrom", PROV_PREFIX)?;
    let expected_dispatch = format!("{RWAI_PREFIX}{}", contract.dispatch_id);
    let provenance = disp_object(&store, "provenance", DISP_PREFIX)?;
    if derived_from.as_deref() != Some(expected_dispatch.as_str())
        || provenance.is_none()
        || contract.declared_authority.trim().is_empty()
    {
        return Err(refuse("authority"));
    }
    // Stage 4: structural — DispatchConsequenceShape (shape-driven SPARQL).
    if !shape_violations(consequence_ttl, shapes_ttl_path, queries)?.is_empty() {
        return Err(refuse("structural"));
    }
    // Stage 5: semantic conformance — the returned artifact must be the
    // contract's expected output artifact set.
    let returned = disp_object(&store, "returnedArtifact", DISP_PREFIX)?;
    let expected_out = format!("{RWAI_PREFIX}{}", contract.expected_output_artifact_set);
    if returned.as_deref() != Some(expected_out.as_str()) {
        return Err(refuse("semantic"));
    }
    Ok(())
}

/// How the inbound consequence is produced. `LoopbackDeterministic` is the
/// benchmark's real mechanism (content-derived synthesis); `FixtureFile`
/// injects an on-disk candidate (negative tests: forged correlation,
/// non-conformant artifact). A future network adapter replaces exactly this
/// enum's producer — nothing else in the pipeline changes.
#[derive(Debug, Clone, Copy)]
pub(super) enum SynthesisMode<'p> {
    LoopbackDeterministic,
    /// Constructed only by negative tests (forged/non-conformant inbox
    /// candidates); the production loopback never reads fixtures.
    #[allow(dead_code)]
    FixtureFile(&'p Path),
}

/// Terminal outcome of one dispatch lifecycle (never a silent state).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DispatchOutcome {
    /// Consequence admitted; contract COMPLETED.
    Admitted,
    /// Consequence refused at `stage`; refusal is OCEL evidence and the
    /// declared compensation workflow was manufactured (if remediation
    /// budget remained).
    Refused { stage: String },
    /// Deadline expired; TIMED_OUT and the declared escalation workflow
    /// was manufactured (if remediation budget remained).
    TimedOut,
    /// Parent closure law unsatisfied; the parent stays open (BLOCKED).
    Open,
}

/// In-process dispatch telemetry (reconciled against the evidence graph by
/// the workday refuse gate; the graph is the authority).
#[derive(Debug, Default)]
pub(super) struct DispatchTelemetry {
    pub(super) sent: usize,
    pub(super) acknowledged: usize,
    pub(super) polls: usize,
    pub(super) returned: usize,
    pub(super) admitted: usize,
    pub(super) refused: usize,
    pub(super) timeouts: usize,
    pub(super) remediations: usize,
}

/// The broker's dispatch adapter: owns the loopback surface directories,
/// the on-disk contract/consequence templates, the shape law, and the
/// day's dispatch receipt digests (folded into the evidence chain).
pub(super) struct DispatchAdapter<'a> {
    outbox_dir: PathBuf,
    inbox_dir: PathBuf,
    contract_template: String,
    consequence_template: String,
    shapes_path: PathBuf,
    standing_query: String,
    queries: &'a QuerySet,
    pub(super) telemetry: DispatchTelemetry,
    /// dispatch id → (contract digest, consequence digest or "refused"/
    /// "timed-out"). BTreeMap: canonical fold order for the chain.
    pub(super) receipt_digests: BTreeMap<String, (String, String)>,
}

impl<'a> DispatchAdapter<'a> {
    /// Constructs the adapter: creates `<out_dir>/dispatch/{outbox,inbox}`,
    /// reads both dispatch templates and the standing query from disk.
    ///
    /// # Complexity
    /// O(template bytes) reads; two mkdirs.
    pub(super) fn new(out_dir: &Path, queries: &'a QuerySet) -> Result<Self, CngRefusal> {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let read = |name: &str| -> Result<String, CngRefusal> {
            let path = manifest.join("templates").join(name);
            fs::read_to_string(&path)
                .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))
        };
        let outbox_dir = out_dir.join("dispatch").join("outbox");
        let inbox_dir = out_dir.join("dispatch").join("inbox");
        for dir in [&outbox_dir, &inbox_dir] {
            fs::create_dir_all(dir)
                .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", dir.display())))?;
        }
        Ok(DispatchAdapter {
            outbox_dir,
            inbox_dir,
            contract_template: read("dispatch-contract.template.ttl")?,
            consequence_template: read("dispatch-consequence.template.ttl")?,
            shapes_path: manifest.join("shapes").join("dispatch-shapes.ttl"),
            standing_query: queries.get("standing-next-action")?.to_string(),
            queries,
            telemetry: DispatchTelemetry::default(),
            receipt_digests: BTreeMap::new(),
        })
    }

    /// The adapter's on-disk query set (shared with the Arazzo projection).
    /// O(1).
    pub(super) fn queries(&self) -> &'a QuerySet {
        self.queries
    }

    /// Deterministic loopback consequence synthesis — the SEAM where a real
    /// network adapter replaces this function (mechanism ALIVE; live
    /// endpoints out of scope). Human-dispatch consequences carry a
    /// MOCKED-HUMAN provenance bundle name: the re-entry MECHANISM is real
    /// and receipted, the producing human is simulated. O(|template|).
    fn synthesize_consequence(&self, contract: &DispatchContract) -> String {
        let provenance_bundle = match contract.execution_class {
            ExecutionClass::ExternalHumanDispatch => {
                format!("mocked-human-prov-{}", contract.dispatch_id)
            }
            _ => format!("prov-{}", contract.dispatch_id),
        };
        fill_template(
            &self.consequence_template,
            &[
                ("CONSEQUENCE_ID", &format!("cons-{}", contract.dispatch_id)),
                ("CORRELATION_ID", &contract.correlation_id),
                ("PRODUCING_ACTOR", &contract.target_actor),
                ("PROVENANCE_BUNDLE", &provenance_bundle),
                ("RETURNED_ARTIFACT", &contract.expected_output_artifact_set),
                ("CONFORMANCE_VERDICT", "PENDING"),
                ("ADMISSION_VERDICT", "PENDING"),
                ("DISPATCH_ID", &contract.dispatch_id),
            ],
        )
    }

    /// Executes one full dispatch lifecycle through the loopback adapter:
    /// render + shape-validate (CNG_R15 before the contract leaves the
    /// broker) → outbox → dispatch_sent (receipted) → optional standing
    /// consult ("collect") → acknowledge → recursive child dispatches +
    /// closure-law query (PROJ-620) → bounded polling (loop bound =
    /// deadlineTicks) → consequence synthesis/fixture → lawful re-entry →
    /// admission or refusal-with-remediation; deadline expiry →
    /// TIMED_OUT + escalation.
    ///
    /// `consult_standing` is true only for TOP-LEVEL dispatches (the
    /// operator's standing surface); child and remediation dispatches run
    /// inside the external executor's boundary and never consult standing.
    /// `remediation_budget` bounds remediation recursion (a remediation of
    /// a remediation is never manufactured).
    ///
    /// # Errors
    /// `CNG_R15/R16` for contract/state-machine violations; `CNG_R09` when
    /// standing disagrees with the dispatch surface; I/O and query refusals
    /// propagate. Consequence-stage failures are NOT errors here — they are
    /// receipted evidence returned as `DispatchOutcome::Refused/TimedOut`.
    ///
    /// # Complexity
    /// O(deadline_ticks) polls + O(CHILD_FAN_OUT^recursive_depth) child
    /// lifecycles + two shape SELECTs; recursion depth bounded by the
    /// contract's `recursive_depth` and `remediation_budget`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn dispatch(
        &mut self,
        writer: &mut ObsWriter<'_>,
        obs_store: &Store,
        mut contract: DispatchContract,
        tick: usize,
        consult_standing: bool,
        synthesis: SynthesisMode<'_>,
        remediation_budget: u8,
    ) -> Result<DispatchOutcome, CngRefusal> {
        // --- Render + shape law: an incomplete contract never leaves the
        // broker (CNG_R15), whether the gap is an empty Rust field or a
        // template/shape drift caught by the shape-driven SPARQL.
        let rendered = contract.render(&self.contract_template)?;
        let violations = shape_violations(&rendered, &self.shapes_path, self.queries)?;
        if let Some((entry, field)) = violations.first() {
            return Err(CngRefusal::DispatchContractIncomplete {
                dispatch: entry.clone(),
                missing: field.clone(),
            });
        }
        contract.advance(DispatchState::DispatchReady)?;

        // --- Outbound: outbox artifact + dispatch_sent receipt.
        let outbox_path = self
            .outbox_dir
            .join(format!("{}.ttl", contract.dispatch_id));
        fs::write(&outbox_path, &rendered)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", outbox_path.display())))?;
        let contract_digest = format!("blake3:{}", blake3::hash(rendered.as_bytes()).to_hex());
        contract.advance(DispatchState::Dispatched)?;
        let tick_text = tick.to_string();
        let deadline_text = contract.deadline_ticks.to_string();
        writer.emit(
            "dispatch-sent",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("TICK", tick_text.as_str()),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
                ("PARENT_DISPATCH", contract.parent_dispatch.as_str()),
                ("EXECUTION_CLASS", contract.execution_class.as_str()),
                ("CORRELATION_ID", contract.correlation_id.as_str()),
                ("CLOSURE_LAW", contract.closure_law.unwrap_or("NONE")),
                ("DEADLINE_TICKS", deadline_text.as_str()),
            ],
        )?;
        self.telemetry.sent += 1;

        // --- Standing: with one dispatch awaited, "what now?" must derive
        // exactly the collect action for exactly this dispatch.
        if consult_standing {
            self.expect_standing_action(obs_store, tick, "collect", &contract.dispatch_id)?;
        }

        // --- Loopback acknowledgement.
        contract.advance(DispatchState::Acknowledged)?;
        writer.emit(
            "dispatch-acknowledged",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
            ],
        )?;
        self.telemetry.acknowledged += 1;
        contract.advance(DispatchState::InProgress)?;

        // --- Recursive child dispatches + closure law (PROJ-620). The
        // dispatched workflow manufactures CHILD_FAN_OUT children per depth
        // level through THIS SAME broker path; the parent may only complete
        // when its declared closure law — evaluated by the on-disk
        // dispatch-closure.rq over admitted child consequences, never by
        // Rust inference — is satisfied.
        //
        // # Complexity
        // O(CHILD_FAN_OUT) child lifecycles per level; total bounded by
        // CHILD_FAN_OUT^recursive_depth.
        if contract.recursive_depth > 0 {
            for c in 0..CHILD_FAN_OUT {
                let child = child_contract(&contract, c);
                self.dispatch(
                    writer,
                    obs_store,
                    child,
                    tick,
                    false,
                    SynthesisMode::LoopbackDeterministic,
                    0,
                )?;
            }
            let satisfied = select_rows(obs_store, self.queries.get("dispatch-closure")?)?
                .iter()
                .any(|row| {
                    row.get("dispatch").map(String::as_str) == Some(contract.dispatch_id.as_str())
                });
            if !satisfied {
                contract.advance(DispatchState::Blocked)?;
                return Ok(DispatchOutcome::Open);
            }
        }

        // --- Bounded polling: loop bound IS the deadline (logical ticks);
        // unbounded polling is structurally impossible. Every poll is a
        // receipted observation. The loopback consequence arrives at the
        // content-derived poll number `blake3(correlationId) %
        // SYNTH_DELAY_MOD` (fixtures arrive at poll 0).
        //
        // # Complexity
        // O(deadline_ticks) iterations.
        let delay = match synthesis {
            SynthesisMode::LoopbackDeterministic => {
                let hash = blake3::hash(contract.correlation_id.as_bytes());
                let mut b = [0u8; 8];
                b.copy_from_slice(&hash.as_bytes()[..8]);
                u64::from_le_bytes(b) % SYNTH_DELAY_MOD
            }
            SynthesisMode::FixtureFile(_) => 0,
        };
        let mut consequence_ttl: Option<String> = None;
        for poll in 0..contract.deadline_ticks {
            let poll_text = poll.to_string();
            writer.emit(
                "dispatch-poll",
                &[
                    ("SET_ID", contract.dispatch_id.as_str()),
                    ("DISPATCH_ID", contract.dispatch_id.as_str()),
                    ("POLL_NO", poll_text.as_str()),
                ],
            )?;
            self.telemetry.polls += 1;
            if poll == delay {
                let body = match synthesis {
                    SynthesisMode::LoopbackDeterministic => self.synthesize_consequence(&contract),
                    SynthesisMode::FixtureFile(path) => fs::read_to_string(path).map_err(|e| {
                        CngRefusal::IoRefused(format!("read {}: {e}", path.display()))
                    })?,
                };
                consequence_ttl = Some(body);
                break;
            }
        }

        // --- Deadline expiry: TIMED_OUT, then the contract's declared
        // escalation law is manufactured as a workflow through this same
        // broker (compensation-as-workflow doctrine).
        let Some(consequence_ttl) = consequence_ttl else {
            contract.advance(DispatchState::TimedOut)?;
            writer.emit(
                "dispatch-timed-out",
                &[
                    ("SET_ID", contract.dispatch_id.as_str()),
                    ("DISPATCH_ID", contract.dispatch_id.as_str()),
                    ("DEADLINE_TICKS", deadline_text.as_str()),
                ],
            )?;
            self.telemetry.timeouts += 1;
            self.receipt_digests.insert(
                contract.dispatch_id.clone(),
                (contract_digest, "timed-out".to_string()),
            );
            if consult_standing {
                self.expect_standing_action(obs_store, tick, "remediate", &contract.dispatch_id)?;
            }
            if remediation_budget > 0 {
                contract.advance(DispatchState::Compensating)?;
                self.remediate(writer, obs_store, &contract, tick, "escalation")?;
                contract.advance(DispatchState::Completed)?;
            } else {
                contract.advance(DispatchState::Blocked)?;
            }
            return Ok(DispatchOutcome::TimedOut);
        };

        // --- Inbound: inbox artifact + consequence_returned receipt. The
        // candidate is receipted BEFORE the re-entry pipeline runs, so a
        // later refusal never silently discards partial external execution.
        let inbox_path = self.inbox_dir.join(format!("{}.ttl", contract.dispatch_id));
        fs::write(&inbox_path, &consequence_ttl)
            .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", inbox_path.display())))?;
        let consequence_digest = format!(
            "blake3:{}",
            blake3::hash(consequence_ttl.as_bytes()).to_hex()
        );
        contract.advance(DispatchState::ResultReturned)?;
        writer.emit(
            "consequence-returned",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
                ("CORRELATION_ID", contract.correlation_id.as_str()),
            ],
        )?;
        self.telemetry.returned += 1;

        // --- Lawful re-entry (staged, in order) → admission or refusal.
        match collect_consequence(&consequence_ttl, &contract, &self.shapes_path, self.queries) {
            Ok(()) => {
                contract.advance(DispatchState::Admitted)?;
                writer.emit(
                    "consequence-admitted",
                    &[
                        ("SET_ID", contract.dispatch_id.as_str()),
                        ("DISPATCH_ID", contract.dispatch_id.as_str()),
                        ("CORRELATION_ID", contract.correlation_id.as_str()),
                    ],
                )?;
                self.telemetry.admitted += 1;
                contract.advance(DispatchState::Completed)?;
                self.receipt_digests.insert(
                    contract.dispatch_id.clone(),
                    (contract_digest, consequence_digest),
                );
                Ok(DispatchOutcome::Admitted)
            }
            Err(CngRefusal::ExternalConsequenceRefused { stage, .. }) => {
                contract.advance(DispatchState::Refused)?;
                writer.emit(
                    "consequence-refused",
                    &[
                        ("SET_ID", contract.dispatch_id.as_str()),
                        ("DISPATCH_ID", contract.dispatch_id.as_str()),
                        ("STAGE", stage.as_str()),
                    ],
                )?;
                self.telemetry.refused += 1;
                self.receipt_digests.insert(
                    contract.dispatch_id.clone(),
                    (contract_digest, "refused".to_string()),
                );
                if remediation_budget > 0 {
                    contract.advance(DispatchState::Compensating)?;
                    self.remediate(writer, obs_store, &contract, tick, "compensation")?;
                    contract.advance(DispatchState::Completed)?;
                } else {
                    contract.advance(DispatchState::Blocked)?;
                }
                Ok(DispatchOutcome::Refused { stage })
            }
            Err(other) => Err(other),
        }
    }

    /// Manufactures the contract's declared remediation workflow
    /// (escalation on timeout, compensation on refused conformance) and
    /// routes it back through THIS broker with a zero remediation budget —
    /// a remediation of a remediation is structurally impossible.
    ///
    /// # Complexity
    /// One nested dispatch lifecycle (O(deadline_ticks) polls).
    fn remediate(
        &mut self,
        writer: &mut ObsWriter<'_>,
        obs_store: &Store,
        contract: &DispatchContract,
        tick: usize,
        remedy: &str,
    ) -> Result<(), CngRefusal> {
        let remedy_id = format!("{remedy}-{}", contract.dispatch_id);
        writer.emit(
            "remediation-manufactured",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
                ("REMEDY", remedy),
                ("REMEDY_DISPATCH", remedy_id.as_str()),
            ],
        )?;
        self.telemetry.remediations += 1;
        let remedy_contract = DispatchContract {
            dispatch_id: remedy_id.clone(),
            workflow_instance: format!("{}-{remedy}", contract.workflow_instance),
            parent_workflow: contract.workflow_instance.clone(),
            recursive_depth: 0,
            activity_identity: format!("{}-{remedy}", contract.activity_identity),
            // Remediation is a WORKFLOW: authority, inputs, expected
            // consequence, receipt, replay — through the same broker.
            input_artifact_set: format!("inputs-{remedy_id}"),
            expected_output_artifact_set: format!("outputs-{remedy_id}"),
            idempotency_key: format!("idem-{}", content_key(&format!("idem|{remedy_id}"))),
            correlation_id: format!("corr-{}", content_key(&format!("corr|{remedy_id}"))),
            // Deadline covers every possible synthesized delay (0..MOD).
            deadline_ticks: SYNTH_DELAY_MOD,
            state: DispatchState::Manufactured,
            closure_law: None,
            parent_dispatch: contract.dispatch_id.clone(),
            ..contract.clone()
        };
        let outcome = self.dispatch(
            writer,
            obs_store,
            remedy_contract,
            tick,
            false,
            SynthesisMode::LoopbackDeterministic,
            0,
        )?;
        if outcome != DispatchOutcome::Admitted {
            return Err(CngRefusal::HardcodingSuspicion(format!(
                "loopback remediation {remedy_id} did not admit ({outcome:?}); \
                 the deterministic remediation mechanism is broken"
            )));
        }
        Ok(())
    }

    /// Asserts standing derives exactly one lawful next action and that it
    /// is `(action, set_id)`; anything else is `CNG_R12`/`CNG_R09`.
    ///
    /// # Complexity
    /// One SELECT over O(obs facts).
    fn expect_standing_action(
        &self,
        obs_store: &Store,
        tick: usize,
        action: &str,
        set_id: &str,
    ) -> Result<(), CngRefusal> {
        let rows = expect_standing_rows(obs_store, &self.standing_query, tick, 1)?;
        let row = rows.first().ok_or(CngRefusal::StandingAmbiguous {
            tick,
            candidate_count: 0,
        })?;
        let got_action = row.get("action").cloned().unwrap_or_default();
        let got_set = row.get("setId").cloned().unwrap_or_default();
        if got_action != action || got_set != set_id {
            return Err(CngRefusal::HardcodingSuspicion(format!(
                "standing derived ({got_action}, {got_set}) but the dispatch surface \
                 expected ({action}, {set_id})"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "dispatch_test.rs"]
mod dispatch_test;
