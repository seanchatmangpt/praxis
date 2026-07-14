//! Multi-engine execution surface (PROJ-722/723/724): `cng engine serve` /
//! `cng engine resume` run a bounded, receipted poll loop over a per-engine
//! filesystem bundle. Filesystem is the transport BINDING of this
//! increment (mechanism ALIVE); HTTP is deliberately absent — an async
//! runtime and wall clock would enter the digest path and break
//! byte-identical replay.
//!
//! Engine identity is deterministic ([`EngineIdentity`]): `engine_id`,
//! [`ENGINE_VERSION`], and `instance_nonce = splitmix64(seed ^
//! blake3(engine_id))` — never a PID or wall clock. Every observation a
//! serve loop emits carries `obs:producedByEngine`.
//!
//! Per-engine bundle layout (each independently replayable):
//! `<root>/engines/<id>/{inbox,outbox,control,ticks,admissions,receipts,
//! ledger}` — contracts arrive in `inbox/` (written atomically by the
//! coordinator; sorted lexicographic scan), consequences leave through
//! `outbox/` (atomic tmp+rename), the durable state ledger + processed set
//! live in `ledger/` (PROJ-721), observations flush eagerly to `ticks/`,
//! and the SHACL-validated `control/quiesce.ttl` ends the loop.
//!
//! Execution boundary (stated honestly, PROJ-710 → PROJ-723 closure): an
//! admitted inbox contract executes through the REAL cng manufacture chain.
//! When the admitted contract's inbox entry carries a sibling PDDL payload
//! (`<dispatch_id>.domain.pddl` + `<dispatch_id>.pddl`, written by
//! `decomp::dispatch_bridge::dispatch_subworkflow_to_engine` BEFORE the
//! contract itself becomes visible), the engine grounds and plans directly
//! from THAT content — the specific subworkflow's own domain+problem
//! text — after verifying its BLAKE3 fold against the admitted contract's
//! `disp:inputArtifactSet` (`CNG_R11 AuditMismatch` on any divergence: a
//! tampered or truncated payload is refused, never silently substituted).
//! When no sibling payload is present (the common case: workday's own
//! synthetic contracts, the multi-engine coordinator's `remote_contract`),
//! the engine falls back UNCHANGED to deterministically deriving its own
//! PDDL artifact set from the contract's content (`write_set`, seeded by
//! `blake3(dispatch_id)`), manufacturing it (`manufacture_set`: import →
//! plan → project → validate → conformance) — the payload path is purely
//! additive, never a precondition for any contract that doesn't carry one.
//!
//! The ONLY real-time element (inter-poll sleep) sits behind the
//! [`RealTimeWait`](super::dispatch::RealTimeWait) seam and never enters
//! any digest; poll counts (logical) are the receipted facts.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;
use bcinr_pddl::ground::IndexedGroundProblem;

use crate::powl::CngRefusal;

use super::decomp::dispatch_bridge::payload_digest;
use super::decomp::DECOMP_MAX_GROUND;
use super::dispatch::{
    collect_consequence, disp_object, read_ledger_entries, shape_violations, write_atomic,
    DispatchContract, DispatchState, ExecutionClass, FileLedgerSink, LedgerSink, NoWait,
    RealTimeWait, ThreadSleepWait, DISP_PREFIX,
};
use super::generate::write_set;
use super::hooks::WorkdayHookBroker;
use super::manufacture::manufacture_set;
use super::roles::{collect_ttl_paths_recursive, metric_count, run_construct, ObsWriter};
use super::run::{evidence_digest, OCEL_CONSTRUCT_STEMS};
use super::templates::{load_templates, QuerySet, Templates};
use super::workday::{build_marker_store, evaluate_marker_map, DISTRIBUTED_MARKER_MAP};
use super::{fill_template, rwai_local, splitmix64, RWAI_PREFIX};

/// Deterministic engine version tag (precedent: chatman `ENGINE_VERSION`).
pub const ENGINE_VERSION: &str = "cng-engine/26.7.10";

/// Deterministic engine identity (PROJ-722): no PID, no wall clock, no
/// path-derived value — two same-seed serves of the same engine id are the
/// same instance for every digest purpose.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EngineIdentity {
    /// The declared engine id (e.g. `H`, `M`).
    pub engine_id: String,
    /// [`ENGINE_VERSION`].
    pub engine_version: &'static str,
    /// `splitmix64(seed ^ blake3(engine_id)[0..8])`.
    pub instance_nonce: u64,
}

impl EngineIdentity {
    /// Derives the identity from `(engine_id, seed)`. O(|engine_id|).
    pub fn new(engine_id: &str, seed: u64) -> EngineIdentity {
        let hash = blake3::hash(engine_id.as_bytes());
        let mut b = [0u8; 8];
        b.copy_from_slice(&hash.as_bytes()[..8]);
        let mut state = seed ^ u64::from_le_bytes(b);
        EngineIdentity {
            engine_id: engine_id.to_string(),
            engine_version: ENGINE_VERSION,
            instance_nonce: splitmix64(&mut state),
        }
    }
}

/// Per-engine bundle directory layout (PROJ-722). Constructing it creates
/// every subdirectory; each accessor is O(1).
pub struct EngineBundle {
    root: PathBuf,
    /// The owning engine id.
    pub engine_id: String,
}

impl EngineBundle {
    /// The fixed bundle subdirectories, in canonical order.
    pub const DIRS: [&'static str; 7] = [
        "inbox",
        "outbox",
        "control",
        "ticks",
        "admissions",
        "receipts",
        "ledger",
    ];

    /// Creates `<root>/engines/<engine_id>/{inbox,outbox,control,ticks,
    /// admissions,receipts,ledger}`.
    ///
    /// # Complexity
    /// O(1) — seven mkdirs.
    pub fn new(root: &Path, engine_id: &str) -> Result<EngineBundle, CngRefusal> {
        // Swarm audit wnl2yhbgm finding #32: `engine_id` used to be joined into the filesystem
        // root with no validation (`root.join("engines").join(engine_id)`), reachable from the
        // `cng engine serve`/`resume` CLI verbs (argv, unsanitized) and from
        // `target_engine: &str` params in multifractal-workflow's F20 dispatch entry points. An
        // `engine_id` of "../../../../tmp/evil-engine" (or an absolute path, which replaces the
        // whole joined `PathBuf` on Unix) relocates the entire per-engine bundle -- inbox,
        // outbox, ledger, everything -- outside the intended root. Same character-class
        // restriction already established for `disp:dispatchId` (dispatch-shapes.ttl's
        // `sh:pattern`); real engine ids used throughout this crate (single letters like "H"/"M"
        // in tests and CLI usage) already fit it.
        if engine_id.is_empty()
            || !engine_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(CngRefusal::IoRefused(format!(
                "invalid engine_id {engine_id:?}: must be non-empty and match \
                 ^[A-Za-z0-9_-]+$ (rejected to prevent relocating the engine bundle outside its \
                 intended root via path traversal)"
            )));
        }
        let bundle = EngineBundle {
            root: root.join("engines").join(engine_id),
            engine_id: engine_id.to_string(),
        };
        for dir in Self::DIRS {
            let path = bundle.root.join(dir);
            let mut attempts = 0;
            loop {
                if let Err(e) = fs::create_dir_all(&path) {
                    if attempts < 10 {
                        attempts += 1;
                        std::thread::yield_now();
                        continue;
                    }
                    return Err(CngRefusal::IoRefused(format!(
                        "mkdir {}: {e}",
                        path.display()
                    )));
                }
                if path.exists() {
                    break;
                }
                if attempts < 10 {
                    attempts += 1;
                    std::thread::yield_now();
                    continue;
                }
                return Err(CngRefusal::IoRefused(format!(
                    "mkdir {}: returned Ok but does not exist",
                    path.display()
                )));
            }
        }
        Ok(bundle)
    }

    /// Bundle root (`<root>/engines/<id>`). O(1).
    pub fn dir(&self) -> &Path {
        &self.root
    }

    /// Inbound dispatch-contract surface. O(1).
    pub fn inbox_dir(&self) -> PathBuf {
        self.root.join("inbox")
    }

    /// Outbound consequence surface. O(1).
    pub fn outbox_dir(&self) -> PathBuf {
        self.root.join("outbox")
    }

    /// Control-file surface (quiescence). O(1).
    pub fn control_dir(&self) -> PathBuf {
        self.root.join("control")
    }

    /// Eagerly flushed observation partitions. O(1).
    pub fn ticks_dir(&self) -> PathBuf {
        self.root.join("ticks")
    }

    /// Per-contract manufactured artifact sets. O(1).
    pub fn admissions_dir(&self) -> PathBuf {
        self.root.join("admissions")
    }

    /// Serve/resume receipts (`serve-report.json`). O(1).
    pub fn receipts_dir(&self) -> PathBuf {
        self.root.join("receipts")
    }

    /// Durable state ledger + processed set (PROJ-721). O(1).
    pub fn ledger_dir(&self) -> PathBuf {
        self.root.join("ledger")
    }

    /// The SHACL-validated quiescence control file. O(1).
    pub fn quiesce_path(&self) -> PathBuf {
        self.control_dir().join("quiesce.ttl")
    }
}

/// Report of one `cng engine serve`/`resume` pass. Digests are
/// content-derived (BLAKE3 fold over `(dispatch_id, consequence digest)`
/// pairs in dispatch-id order); nothing real-time is serialized.
#[derive(Debug, serde::Serialize)]
pub struct EngineServeReport {
    /// Always "MEASURED_CNG_RESULT".
    pub measurement_class: &'static str,
    pub engine_id: String,
    pub engine_version: &'static str,
    pub instance_nonce: u64,
    /// Whether this pass resumed from an existing ledger (PROJ-724).
    pub resumed: bool,
    /// Chain-verified ledger entries reloaded at resume (0 on fresh serve).
    pub ledger_entries_verified: u64,
    /// Logical polls executed (bounded by `max_polls`).
    pub polls: u64,
    /// Inbox contracts admitted + executed this pass.
    pub contracts_executed: usize,
    /// Whether the loop ended on a validated quiescence file (`false` =
    /// poll budget exhausted; honest partial, not an error).
    pub quiesced: bool,
    /// BLAKE3 fold over executed `(dispatch_id, consequence digest)` pairs.
    pub receipt_chain_digest: String,
}

/// The contract fields the engine needs from one admitted inbox contract.
struct InboxContract {
    dispatch_id: String,
    correlation_id: String,
    target_actor_local: String,
    expected_output_local: String,
    idempotency_key: String,
    /// `disp:inputArtifactSet` local name. For a payload-carrying dispatch
    /// (`decomp::dispatch_bridge::dispatch_subworkflow_to_engine`) this is
    /// `payload-<16 hex>`, the real BLAKE3 fold over the dispatched
    /// domain+problem PDDL text — verified against the sibling payload
    /// files before they are trusted (see `run_serve_loop`). For every
    /// other contract shape it stays the synthetic `inputs-<dispatch_id>`
    /// label and is unused.
    input_artifact_local: String,
}

/// Reads the engine-relevant fields out of one shape-valid contract graph.
/// A missing field is a torn/foreign artifact — typed refusal, never a
/// default.
///
/// # Complexity
/// O(contract triples) load + O(1) pattern scans.
fn read_inbox_contract(contract_ttl: &str, path: &Path) -> Result<InboxContract, CngRefusal> {
    let store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("contract store construction: {e}")))?;
    store
        .load_from_slice(
            RdfParser::from_format(RdfFormat::Turtle),
            contract_ttl.as_bytes(),
        )
        .map_err(|e| CngRefusal::MalformedTtl(format!("inbox contract {}: {e}", path.display())))?;
    let field = |local: &str| -> Result<String, CngRefusal> {
        disp_object(&store, local, DISP_PREFIX)?.ok_or_else(|| {
            CngRefusal::MalformedTtl(format!(
                "inbox contract {} has no disp:{local}",
                path.display()
            ))
        })
    };
    Ok(InboxContract {
        dispatch_id: field("dispatchId")?,
        correlation_id: field("correlationId")?,
        target_actor_local: rwai_local(&field("targetActor")?).to_string(),
        expected_output_local: rwai_local(&field("expectedOutputArtifactSet")?).to_string(),
        idempotency_key: field("idempotencyKey")?,
        input_artifact_local: rwai_local(&field("inputArtifactSet")?).to_string(),
    })
}

/// Real manufacture evidence for one payload-carrying dispatch: grounded
/// and planned from the dispatched domain+problem PDDL text itself (never
/// `blake3(dispatch_id)`-seeded synthetic content). The parsed domain/
/// problem names themselves are written into `set_dir/manifest.txt`
/// (durable evidence a test or auditor can inspect), not carried here.
struct PayloadOutcome {
    /// Grounded plan length.
    tape_ops: usize,
    /// `blake3:<hex>` over the grounded plan's ordered action labels.
    powl_digest: String,
}

/// Grounds and plans a dispatched PDDL domain/problem payload (PROJ-710 →
/// PROJ-723 closure) and writes the manufactured evidence — the received
/// domain/problem text verbatim plus the grounded plan's ordered action
/// labels — under `set_dir`, the same per-dispatch admissions-dir
/// convention `write_set`/`manufacture_set` use for the synthetic path, so
/// both paths are auditable the same way (`bundle.admissions_dir()/
/// <dispatch_id>/`).
///
/// `max_ground` mirrors `decomp::DECOMP_MAX_GROUND`, the SAME ceiling the
/// dispatching side's own `decompose()` used to derive this payload in the
/// first place — a bound mismatch here would let the engine refuse a
/// payload `decompose()` itself could ground.
///
/// # Errors
/// `CNG_R01 MalformedTtl` when either PDDL text fails to parse; `CNG_R09
/// UnsupportedConstruct` when grounding fails (bound exceeded / empty
/// grounding); `CNG_R04 PlanUnsolvable` when the dispatched problem admits
/// no plan within the depth bound; `CNG_R10 IoRefused` for evidence-write
/// IO.
///
/// # Complexity
/// O(|domain_pddl| + |problem_pddl|) parse + bounded grounding/BFS
/// (`DECOMP_MAX_GROUND` ceiling) + O(plan ops) evidence write.
fn manufacture_from_payload(
    set_dir: &Path,
    domain_pddl: &str,
    problem_pddl: &str,
) -> Result<PayloadOutcome, CngRefusal> {
    let domain = domain_from_pddl(domain_pddl).map_err(|e| {
        CngRefusal::MalformedTtl(format!("dispatched domain payload failed to parse: {e:?}"))
    })?;
    let problem = problem_from_pddl(problem_pddl).map_err(|e| {
        CngRefusal::MalformedTtl(format!("dispatched problem payload failed to parse: {e:?}"))
    })?;
    let ground = IndexedGroundProblem::build(&domain, &problem, Some(DECOMP_MAX_GROUND))
        .map_err(|e| CngRefusal::UnsupportedConstruct(format!("payload grounding failed: {e}")))?;
    let tape = ground.find_plan().into_result().map_err(|e| {
        CngRefusal::PlanUnsolvable(format!("dispatched payload admits no plan: {e}"))
    })?;

    fs::create_dir_all(set_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", set_dir.display())))?;
    write_atomic(&set_dir.join("domain.pddl"), domain_pddl)?;
    write_atomic(&set_dir.join("problem.pddl"), problem_pddl)?;
    let plan_text = tape
        .ops
        .iter()
        .map(|op| op.label.clone())
        .collect::<Vec<_>>()
        .join("\n");
    write_atomic(&set_dir.join("plan.txt"), &plan_text)?;
    // Real evidence of WHICH content was executed (not decorative — this is
    // exactly what a load-bearing test inspects to tell a payload-carrying
    // execution apart from the synthetic `wf-email-routing-*` path).
    let manifest = format!(
        "domain: {}\nproblem: {}\ntape_ops: {}\n",
        domain.name,
        problem.name,
        tape.ops.len()
    );
    write_atomic(&set_dir.join("manifest.txt"), &manifest)?;

    Ok(PayloadOutcome {
        tape_ops: tape.ops.len(),
        powl_digest: format!("blake3:{}", blake3::hash(plan_text.as_bytes()).to_hex()),
    })
}

/// Shared serve loop for `serve` (fresh ledger) and `resume` (reloaded,
/// chain-verified ledger). See the module docs for the per-poll sequence.
///
/// # Errors
/// `CNG_R25 DoubleAdmit` on a replayed idempotency key; `CNG_R15` (shape
/// violation) / `CNG_R01` (unparseable) for malformed inbox contracts or
/// quiescence files; `CNG_R09` when the deterministic manufacture of an
/// admitted contract refuses; I/O refusals propagate.
///
/// # Complexity
/// O(max_polls × inbox files) directory scans + O(executed contracts)
/// pipeline-bounded manufactures + O(obs) template renders.
#[allow(clippy::too_many_arguments)]
fn run_serve_loop(
    bundle: &EngineBundle,
    identity: &EngineIdentity,
    templates: &Templates,
    queries: &QuerySet,
    mut ledger: FileLedgerSink,
    max_polls: u64,
    wait: &dyn RealTimeWait,
    resumed: bool,
    ledger_entries_verified: u64,
) -> Result<EngineServeReport, CngRefusal> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shapes_path = manifest.join("shapes").join("dispatch-shapes.ttl");
    let consequence_template = {
        let path = manifest
            .join("templates")
            .join("dispatch-consequence.template.ttl");
        fs::read_to_string(&path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?
    };
    let obs_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("engine obs store construction: {e}")))?;
    // Eager flush (threshold 1): every observation is durable before the
    // loop takes another step (PROJ-721 crash-resume input).
    let mut writer = ObsWriter::new(templates, &obs_store, &bundle.ticks_dir(), "engine")?
        .with_flush_threshold(1);

    let nonce_text = identity.instance_nonce.to_string();
    let resumed_text = if resumed { "true" } else { "false" };
    // A resumed pass first receipts its chain-prefix verification, then the
    // (re)start; a fresh pass receipts only the start.
    if resumed {
        let entries_text = ledger_entries_verified.to_string();
        writer.emit(
            "resume-verified",
            &[
                ("SET_ID", bundle.engine_id.as_str()),
                ("ENGINE_ID", identity.engine_id.as_str()),
                ("ENTRIES", entries_text.as_str()),
                // PROJ-727: truthful by construction — a divergent chain
                // refused CNG_R11 before this observation could exist;
                // marker-replay-divergence.rq counts "true" values.
                ("DIVERGENCE", "false"),
            ],
        )?;
    }
    writer.emit(
        "engine-started",
        &[
            ("SET_ID", bundle.engine_id.as_str()),
            ("ENGINE_ID", identity.engine_id.as_str()),
            ("ENGINE_VERSION", identity.engine_version),
            ("NONCE", nonce_text.as_str()),
            ("RESUMED", resumed_text),
        ],
    )?;

    // dispatch id → consequence digest, in canonical (BTreeMap) order.
    let mut executed: BTreeMap<String, String> = BTreeMap::new();
    let mut polls = 0u64;
    let mut quiesced = false;
    let worker_iri = format!("{RWAI_PREFIX}w-engine-{}", bundle.engine_id);

    // Bounded receipted poll loop: the loop bound IS the poll budget, so an
    // unbounded server is structurally impossible.
    //
    // # Complexity
    // O(max_polls) iterations; each is one sorted inbox scan.
    for poll in 0..max_polls {
        polls = poll + 1;
        let poll_text = poll.to_string();
        writer.emit(
            "engine-poll",
            &[
                ("SET_ID", bundle.engine_id.as_str()),
                ("ENGINE_ID", identity.engine_id.as_str()),
                ("POLL_NO", poll_text.as_str()),
            ],
        )?;

        // Quiescence: the control file ends the loop only after it
        // validates against QuiescenceShape (a malformed quiesce file is
        // refused, never silently obeyed).
        let quiesce_path = bundle.quiesce_path();
        if quiesce_path.is_file() {
            let quiesce_ttl = fs::read_to_string(&quiesce_path).map_err(|e| {
                CngRefusal::IoRefused(format!("read {}: {e}", quiesce_path.display()))
            })?;
            let violations = shape_violations(&quiesce_ttl, &shapes_path, queries)?;
            if let Some((entry, field)) = violations.first() {
                return Err(CngRefusal::MalformedTtl(format!(
                    "quiesce.ttl violates QuiescenceShape (entry {entry}, field {field})"
                )));
            }
            let quiesce_store = Store::new()
                .map_err(|e| CngRefusal::IoRefused(format!("quiesce store construction: {e}")))?;
            quiesce_store
                .load_from_slice(
                    RdfParser::from_format(RdfFormat::Turtle),
                    quiesce_ttl.as_bytes(),
                )
                .map_err(|e| CngRefusal::MalformedTtl(format!("quiesce.ttl: {e}")))?;
            let reason = disp_object(&quiesce_store, "reason", DISP_PREFIX)?.ok_or_else(|| {
                CngRefusal::MalformedTtl("quiesce.ttl has no disp:reason".to_string())
            })?;
            writer.emit(
                "engine-quiesced",
                &[
                    ("SET_ID", bundle.engine_id.as_str()),
                    ("ENGINE_ID", identity.engine_id.as_str()),
                    ("POLL_NO", poll_text.as_str()),
                    ("REASON", reason.as_str()),
                ],
            )?;
            quiesced = true;
            break;
        }

        // Sorted lexicographic inbox scan (determinism pin: zero-padded ids
        // + sorted scan = one canonical admission order); *.tmp files are
        // invisible by extension filter (atomic-writer discipline).
        let inbox = bundle.inbox_dir();
        let mut contract_paths: Vec<PathBuf> = fs::read_dir(&inbox)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", inbox.display())))?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ttl"))
            .collect();
        contract_paths.sort();

        let mut worked = false;
        for path in contract_paths {
            let contract_ttl = fs::read_to_string(&path)
                .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
            // Admission: the contract must conform to DispatchContractShape
            // before the engine executes anything.
            let violations = shape_violations(&contract_ttl, &shapes_path, queries)?;
            if let Some((entry, field)) = violations.first() {
                return Err(CngRefusal::DispatchContractIncomplete {
                    dispatch: entry.clone(),
                    missing: field.clone(),
                });
            }
            let contract = read_inbox_contract(&contract_ttl, &path)?;
            if executed.contains_key(&contract.dispatch_id) {
                continue;
            }
            // Idempotent consume (PROJ-721): an inbox contract whose key is
            // already in the durable processed set was executed by a prior
            // (possibly killed) pass — resume SKIPS it, executing it again
            // would be the double-admit bug this set exists to prevent.
            // (The refusing side of the law — CNG_R25 DoubleAdmit — lives
            // at the coordinator's consequence-admission gate,
            // `DispatchAdapter::guard_double_admit`.)
            if ledger.is_processed(&contract.idempotency_key) {
                continue;
            }

            // Engine-side remote lifecycle, ledgered per transition
            // (PROJ-721): ACKNOWLEDGED → REMOTE_STARTED →
            // REMOTE_IN_PROGRESS → RESULT_AVAILABLE at logical tick =
            // poll number.
            let tick = poll as usize;
            ledger.append(
                &contract.dispatch_id,
                "ACKNOWLEDGED",
                "REMOTE_STARTED",
                tick,
            )?;
            ledger.append(
                &contract.dispatch_id,
                "REMOTE_STARTED",
                "REMOTE_IN_PROGRESS",
                tick,
            )?;

            // REAL execution. PROJ-710 → PROJ-723: a sibling PDDL payload
            // next to THIS contract's `.ttl` (written by
            // `decomp::dispatch_bridge::dispatch_subworkflow_to_engine`)
            // takes priority over the synthetic path — purely additive, no
            // contract that never carried a payload changes behavior.
            let set_dir = bundle.admissions_dir().join(&contract.dispatch_id);
            let domain_payload_path =
                path.with_file_name(format!("{}.domain.pddl", contract.dispatch_id));
            let problem_payload_path =
                path.with_file_name(format!("{}.pddl", contract.dispatch_id));
            let powl_digest = if problem_payload_path.is_file() {
                if !domain_payload_path.is_file() {
                    return Err(CngRefusal::MalformedTtl(format!(
                        "dispatch {} carries a problem payload ({}) with no paired \
                         domain payload ({})",
                        contract.dispatch_id,
                        problem_payload_path.display(),
                        domain_payload_path.display()
                    )));
                }
                let domain_pddl = fs::read_to_string(&domain_payload_path).map_err(|e| {
                    CngRefusal::IoRefused(format!("read {}: {e}", domain_payload_path.display()))
                })?;
                let problem_pddl = fs::read_to_string(&problem_payload_path).map_err(|e| {
                    CngRefusal::IoRefused(format!("read {}: {e}", problem_payload_path.display()))
                })?;
                // Integrity: the admitted contract's disp:inputArtifactSet
                // must equal the SAME length-prefixed BLAKE3 fold over the
                // bytes actually read here — a tampered/truncated sibling
                // payload refuses CNG_R11, never silently substitutes the
                // synthetic path.
                let expected = format!("payload-{}", payload_digest(&domain_pddl, &problem_pddl));
                if contract.input_artifact_local != expected {
                    return Err(CngRefusal::AuditMismatch(format!(
                        "dispatch {} declared disp:inputArtifactSet {} but its sibling \
                         payload digests to {expected}",
                        contract.dispatch_id, contract.input_artifact_local
                    )));
                }
                let payload_outcome =
                    manufacture_from_payload(&set_dir, &domain_pddl, &problem_pddl)?;
                if payload_outcome.tape_ops == 0 {
                    return Err(CngRefusal::PlanUnsolvable(format!(
                        "dispatch {} payload grounded to an empty plan",
                        contract.dispatch_id
                    )));
                }
                payload_outcome.powl_digest
            } else {
                // Synthetic path (unchanged): deterministic artifact set
                // from the contract's content (splitmix64 seeded by
                // blake3(dispatch_id)), manufactured through the full cng
                // chain.
                let hash = blake3::hash(contract.dispatch_id.as_bytes());
                let mut b = [0u8; 8];
                b.copy_from_slice(&hash.as_bytes()[..8]);
                let mut rng = u64::from_le_bytes(b) ^ identity.instance_nonce;
                write_set(
                    templates,
                    &set_dir,
                    &mut rng,
                    &contract.dispatch_id,
                    &worker_iri,
                    "email-routing",
                    0,
                    false,
                    None,
                )?;
                let outcome = manufacture_set(&set_dir, None);
                if let Some(code) = outcome.refusal_code {
                    // The engine derived its own deterministic, complete
                    // set; a refusal means the mechanism is broken, not the
                    // input.
                    return Err(CngRefusal::HardcodingSuspicion(format!(
                        "engine {} manufacture of {} refused {code}; the \
                         deterministic engine execution mechanism is broken",
                        identity.engine_id, contract.dispatch_id
                    )));
                }
                outcome.powl_digest
            };

            // Consequence: template-rendered, atomically written to the
            // outbox (the coordinator's collection surface).
            let consequence = fill_template(
                &consequence_template,
                &[
                    ("CONSEQUENCE_ID", &format!("cons-{}", contract.dispatch_id)),
                    ("CORRELATION_ID", &contract.correlation_id),
                    ("PRODUCING_ACTOR", &contract.target_actor_local),
                    (
                        "PROVENANCE_BUNDLE",
                        &format!(
                            "engine-{}-prov-{}",
                            identity.engine_id, contract.dispatch_id
                        ),
                    ),
                    ("RETURNED_ARTIFACT", &contract.expected_output_local),
                    ("CONFORMANCE_VERDICT", "PENDING"),
                    ("ADMISSION_VERDICT", "PENDING"),
                    ("DISPATCH_ID", &contract.dispatch_id),
                ],
            );
            let out_path = bundle
                .outbox_dir()
                .join(format!("{}.ttl", contract.dispatch_id));
            super::dispatch::write_atomic(&out_path, &consequence)?;
            ledger.append(
                &contract.dispatch_id,
                "REMOTE_IN_PROGRESS",
                "RESULT_AVAILABLE",
                tick,
            )?;
            ledger.mark_processed(&contract.idempotency_key, &contract.dispatch_id)?;

            let consequence_digest =
                format!("blake3:{}", blake3::hash(consequence.as_bytes()).to_hex());
            writer.emit(
                "engine-executed",
                &[
                    ("SET_ID", bundle.engine_id.as_str()),
                    ("ENGINE_ID", identity.engine_id.as_str()),
                    ("DISPATCH_ID", contract.dispatch_id.as_str()),
                    ("POWL_DIGEST", powl_digest.as_str()),
                    ("CONSEQUENCE_DIGEST", consequence_digest.as_str()),
                ],
            )?;
            executed.insert(contract.dispatch_id.clone(), consequence_digest);
            worked = true;
        }

        if !worked {
            // Real time behind the seam only; the poll count above is the
            // logical, receipted fact.
            wait.wait();
        }
    }
    writer.flush()?;

    // Receipt chain: BLAKE3 fold over executed pairs in dispatch-id
    // (BTreeMap) order — content-derived, replayable from the outbox alone.
    //
    // # Complexity
    // O(executed) hash updates.
    let mut chain = blake3::Hasher::new();
    for (dispatch_id, digest) in &executed {
        chain.update(dispatch_id.as_bytes());
        chain.update(digest.as_bytes());
    }
    let report = EngineServeReport {
        measurement_class: "MEASURED_CNG_RESULT",
        engine_id: identity.engine_id.clone(),
        engine_version: identity.engine_version,
        instance_nonce: identity.instance_nonce,
        resumed,
        ledger_entries_verified,
        polls,
        contracts_executed: executed.len(),
        quiesced,
        receipt_chain_digest: format!("blake3:{}", chain.finalize().to_hex()),
    };
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| CngRefusal::IoRefused(format!("serve report serialize: {e}")))?;
    fs::write(bundle.receipts_dir().join("serve-report.json"), &json)
        .map_err(|e| CngRefusal::IoRefused(format!("write serve-report.json: {e}")))?;
    Ok(report)
}

/// Builds the wait seam: a real inter-poll sleep when `poll_wait_ms` is
/// set, else no wait (loopback/tests). O(1).
fn make_wait(poll_wait_ms: Option<u64>) -> Box<dyn RealTimeWait> {
    match poll_wait_ms {
        Some(millis) => Box::new(ThreadSleepWait { millis }),
        None => Box::new(NoWait),
    }
}

/// `cng engine serve` (PROJ-723): a bounded, receipted poll loop over the
/// engine's inbox; every admitted contract executes through the real cng
/// manufacture chain and its consequence is written atomically to the
/// outbox; a SHACL-validated `control/quiesce.ttl` ends the loop.
///
/// Before entering the poll loop, verifies the arazzo-pack's rendered
/// OpenAPI/AsyncAPI capability documents (this engine's declared
/// capability/event contract; `packs/arazzo-pack/templates/
/// {engine-openapi,engine-asyncapi}.yaml.tmpl`) against their ggen sync
/// receipt digests — `api_docs::verify_api_docs_render_digest_if_present`.
/// A stale or tampered capability description is a real correctness
/// problem, exactly parallel to why Arazzo's render is verified before
/// dispatch (`arazzo::verify_arazzo_render_digest`, PROJ-745). Engines
/// whose `root` has no pre-generated capability docs (the common case
/// today; arazzo-pack has not been synced against every engine root) skip
/// the check — absence is not a refusal.
///
/// # Errors
/// `CNG_R11 AuditMismatch` when the capability documents ARE present but
/// stale/tampered/unreceipted; then see [`run_serve_loop`].
///
/// # Complexity
/// O(max_polls × inbox files) + O(executed) manufactures + O(1) capability-
/// doc presence check (O(rendered bytes) x2 when present).
pub fn engine_serve(
    root: &Path,
    engine_id: &str,
    seed: u64,
    max_polls: u64,
    poll_wait_ms: Option<u64>,
) -> Result<EngineServeReport, CngRefusal> {
    let templates = load_templates()?;
    let queries = QuerySet::load(&QuerySet::default_dir())?;
    let identity = EngineIdentity::new(engine_id, seed);
    let bundle = EngineBundle::new(root, engine_id)?;
    super::api_docs::verify_api_docs_render_digest_if_present(root)?;
    let ledger = FileLedgerSink::new(&bundle.ledger_dir())?;
    run_serve_loop(
        &bundle,
        &identity,
        &templates,
        &queries,
        ledger,
        max_polls,
        make_wait(poll_wait_ms).as_ref(),
        false,
        0,
    )
}

/// `cng engine resume` (PROJ-724): reloads the durable ledger tail +
/// processed set, verifies every per-dispatch receipt-chain prefix (a torn
/// ledger tail — truncated last entry, missing field, or chain-hash
/// mismatch — refuses `CNG_R11 AuditMismatch`), receipts the verification
/// as a `resume_verified` observation, and continues the serve loop.
///
/// # Errors
/// `CNG_R11` for a torn/tampered ledger; then see [`run_serve_loop`].
///
/// # Complexity
/// O(ledger bytes) verification + the serve-loop bounds.
pub fn engine_resume(
    root: &Path,
    engine_id: &str,
    seed: u64,
    max_polls: u64,
    poll_wait_ms: Option<u64>,
) -> Result<EngineServeReport, CngRefusal> {
    let templates = load_templates()?;
    let queries = QuerySet::load(&QuerySet::default_dir())?;
    let identity = EngineIdentity::new(engine_id, seed);
    let bundle = EngineBundle::new(root, engine_id)?;
    // FileLedgerSink::new re-reads every ledger file and verifies every
    // chain prefix; a torn tail refuses here, before any new work.
    let ledger = FileLedgerSink::new(&bundle.ledger_dir())?;
    let verified = ledger.total_entries();
    run_serve_loop(
        &bundle,
        &identity,
        &templates,
        &queries,
        ledger,
        max_polls,
        make_wait(poll_wait_ms).as_ref(),
        true,
        verified,
    )
}

// ---------------------------------------------------------------------------
// Multi-engine coordinator (PROJ-727/728/729)
// ---------------------------------------------------------------------------

/// One remote dispatch the coordinator will address to a target engine.
/// Specs are derived deterministically from `(engine_ids, per_engine,
/// depth, fan_out)` by [`remote_specs`], so the dispatch and collect phases
/// rebuild byte-identical contracts without shared in-memory state.
struct RemoteSpec {
    /// Zero-padded, content-free local dispatch id (determinism pin).
    local: String,
    /// Target engine id (`disp:targetEngine`).
    engine: String,
    /// Parent dispatch id for observation lineage ("none" at roots).
    parent: String,
    /// Remaining recursion depth below this node.
    depth: u32,
}

/// Expands one spec subtree: children alternate to the NEXT engine in the
/// cycle, so every parent→child edge of a depth ≥ 1 tree crosses engines.
///
/// # Complexity
/// O(fan_out^depth) nodes; recursion depth = `depth` (caller-bounded).
fn expand_specs(
    specs: &mut Vec<RemoteSpec>,
    engine_ids: &[&str],
    engine_idx: usize,
    id: String,
    parent: &str,
    depth: u32,
    fan_out: usize,
) {
    specs.push(RemoteSpec {
        local: id.clone(),
        engine: engine_ids[engine_idx % engine_ids.len()].to_string(),
        parent: parent.to_string(),
        depth,
    });
    if depth > 0 {
        for c in 0..fan_out {
            let child = format!("{id}-c{c}");
            expand_specs(
                specs,
                engine_ids,
                engine_idx + 1,
                child,
                &id,
                depth - 1,
                fan_out,
            );
        }
    }
}

/// The deterministic remote-dispatch plan: `per_engine` root contracts per
/// engine, each fanning out `fan_out` children per level for `depth`
/// levels, children round-robined to the next engine (cross-engine
/// recursion evidence). Ids are zero-padded and content-free.
///
/// # Complexity
/// O(engines × per_engine × fan_out^depth) specs.
fn remote_specs(
    engine_ids: &[&str],
    per_engine: usize,
    depth: u32,
    fan_out: usize,
) -> Vec<RemoteSpec> {
    let mut specs = Vec::new();
    for (ei, engine) in engine_ids.iter().enumerate() {
        for k in 0..per_engine {
            let root_id = format!("disp-remote-{engine}-{k:04}");
            expand_specs(&mut specs, engine_ids, ei, root_id, "none", depth, fan_out);
        }
    }
    specs
}

/// Short content-derived key: first 12 hex chars of `blake3(text)`.
/// O(|text|).
fn short_key(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex()[..12].to_string()
}

/// Builds the sealed contract for one [`RemoteSpec`]. Every field is
/// content-derived from `(spec, seed)` — no PID, path, or wall clock — so
/// the dispatch and collect phases (and two same-seed runs) render
/// byte-identical contracts, and a permuted seed changes every digest
/// causally. O(1).
fn remote_contract(spec: &RemoteSpec, seed: u64) -> DispatchContract {
    let dispatch_id = spec.local.clone();
    DispatchContract {
        dispatch_id: dispatch_id.clone(),
        workflow_instance: format!("wf-{dispatch_id}"),
        parent_workflow: if spec.parent == "none" {
            format!("wf-{dispatch_id}")
        } else {
            format!("wf-{}", spec.parent)
        },
        recursive_depth: spec.depth,
        target_actor: "external-machine-executor".to_string(),
        target_engine: spec.engine.clone(),
        required_role: "operator".to_string(),
        declared_authority: format!("coordinator-authority-{seed}"),
        input_artifact_set: format!("inputs-{dispatch_id}"),
        expected_output_artifact_set: format!("outputs-{dispatch_id}"),
        activity_identity: format!("remote-{dispatch_id}"),
        // Rendered into the sealed contract; the collect phase's own poll
        // budget (max_polls) is the operative bound on this path.
        deadline_ticks: 64,
        idempotency_key: format!("idem-{}", short_key(&format!("idem|{seed}|{dispatch_id}"))),
        correlation_id: format!("corr-{}", short_key(&format!("corr|{seed}|{dispatch_id}"))),
        collection_surface: format!("engines/{}/outbox", spec.engine),
        retry_law: "retry:limit=0;declarative-only".to_string(),
        escalation_law: "escalate:manufacture-escalation-workflow".to_string(),
        compensation_law: "compensate:manufacture-compensation-workflow".to_string(),
        refusal_conditions: "provenance,correlation,authority,structural,semantic".to_string(),
        receipt_requirements: "obs-receipt-per-transition".to_string(),
        replay_requirements: "byte-identical-same-seed".to_string(),
        state: DispatchState::Manufactured,
        execution_class: ExecutionClass::ExternalMachineDispatch,
        closure_law: None,
        parent_dispatch: spec.parent.clone(),
    }
}

/// Lawful, ledgered state advance for the coordinator path (same law as
/// `DispatchAdapter::advance_ledgered` — `CNG_R16` on an unlawful
/// transition, one durable `disp:StateEntry` per advance). O(ledger file
/// bytes) per append.
fn coord_advance(
    ledger: &mut FileLedgerSink,
    contract: &mut DispatchContract,
    to: DispatchState,
    tick: usize,
) -> Result<(), CngRefusal> {
    let from = contract.state;
    contract.advance(to)?;
    ledger.append(&contract.dispatch_id, from.as_str(), to.as_str(), tick)
}

/// Report of one multi-engine coordination (dispatch phase count folded
/// into the collect-phase report). Contains NO filesystem paths, PIDs, or
/// wall-clock values — two same-seed serialized runs serialize
/// byte-identically.
#[derive(Debug, serde::Serialize)]
pub struct EngineCoordinateReport {
    /// Always "MEASURED_CNG_RESULT".
    pub measurement_class: &'static str,
    pub coordinator_id: String,
    /// Target engine ids, in dispatch order.
    pub engines: Vec<String>,
    /// Contracts addressed to remote engines (dispatch phase).
    pub contracts_dispatched: usize,
    /// Consequences read off remote collection surfaces (collect phase).
    pub remote_consequences_received: usize,
    /// Consequences that passed the lawful re-entry pipeline + idempotent
    /// consume and were admitted.
    pub consequences_admitted: usize,
    /// DISTINCT engine identities with an engine_started event in the
    /// materialized evidence graph (metric-engine-instances.rq authority).
    pub engine_instances: u64,
    /// The distributed marker set ([`DISTRIBUTED_MARKER_MAP`]), all true by
    /// construction — a false marker refused `CNG_R20` before this struct
    /// exists. Includes the INVERTED existence markers (see query headers).
    pub markers: BTreeMap<String, bool>,
    /// Sorted-N-Triples BLAKE3 of the coordinator ∪ engine evidence graph.
    pub ocel_graph_digest: String,
    /// BLAKE3 fold over admitted `(dispatch_id, consequence digest)` pairs
    /// in dispatch-id (BTreeMap) order.
    pub receipt_chain_digest: String,
}

/// Multi-engine dispatch phase (PROJ-727/728): renders, shape-gates,
/// ledgers, and atomically writes one sealed contract per [`RemoteSpec`]
/// into its target engine's inbox, receipting arazzo_workflow_generated /
/// dispatch_sent / arazzo_workflow_dispatched / remote_dispatch_sent
/// observations (eager per-emit flush) into the coordinator's own engine
/// bundle. Returns the number of contracts dispatched.
///
/// Phase split (determinism pin): dispatching and collecting are separate
/// calls so a harness can run the engines to completion in between —
/// collect-phase polls then find every consequence at poll 0 and two
/// same-seed serialized runs are byte-identical. The concurrent harness
/// calls collect while engines run; its poll COUNTS are then
/// arrival-dependent (receipted logical facts, honestly nondeterministic
/// between runs — byte-identity is asserted only for serialized runs).
///
/// # Errors
/// `CNG_R15` for an incomplete/shape-violating contract; `CNG_R16` for an
/// unlawful state transition; I/O refusals propagate.
///
/// # Complexity
/// O(specs) renders + shape SELECTs, specs = engines × per_engine ×
/// fan_out^depth.
pub fn engine_dispatch_remote(
    root: &Path,
    coordinator_id: &str,
    engine_ids: &[&str],
    per_engine: usize,
    depth: u32,
    fan_out: usize,
    seed: u64,
) -> Result<usize, CngRefusal> {
    let templates = load_templates()?;
    let queries = QuerySet::load(&QuerySet::default_dir())?;
    let bundle = EngineBundle::new(root, coordinator_id)?;
    let mut ledger = FileLedgerSink::new(&bundle.ledger_dir())?;
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shapes_path = manifest.join("shapes").join("dispatch-shapes.ttl");
    let contract_template = {
        let path = manifest
            .join("templates")
            .join("dispatch-contract.template.ttl");
        fs::read_to_string(&path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?
    };
    let obs_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("coordinator obs store construction: {e}")))?;
    let mut writer = ObsWriter::new(
        &templates,
        &obs_store,
        &bundle.ticks_dir(),
        "coord-dispatch",
    )?
    .with_flush_threshold(1);

    // Target bundles exist before their serve loops start (idempotent).
    for engine_id in engine_ids.iter().copied() {
        EngineBundle::new(root, engine_id)?;
    }

    let specs = remote_specs(engine_ids, per_engine, depth, fan_out);
    for spec in &specs {
        let mut contract = remote_contract(spec, seed);
        let rendered = contract.render(&contract_template)?;
        coord_advance(&mut ledger, &mut contract, DispatchState::ArazzoRendered, 0)?;
        let contract_digest = format!("blake3:{}", blake3::hash(rendered.as_bytes()).to_hex());
        writer.emit(
            "arazzo-workflow-generated",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("TICK", "0"),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
                ("EXECUTION_CLASS", contract.execution_class.as_str()),
                ("CONTRACT_DIGEST", contract_digest.as_str()),
            ],
        )?;
        let violations = shape_violations(&rendered, &shapes_path, &queries)?;
        if let Some((entry, field)) = violations.first() {
            return Err(CngRefusal::DispatchContractIncomplete {
                dispatch: entry.clone(),
                missing: field.clone(),
            });
        }
        coord_advance(&mut ledger, &mut contract, DispatchState::DispatchReady, 0)?;
        let inbox_path = root
            .join("engines")
            .join(&spec.engine)
            .join("inbox")
            .join(format!("{}.ttl", contract.dispatch_id));
        write_atomic(&inbox_path, &rendered)?;
        coord_advance(&mut ledger, &mut contract, DispatchState::Dispatched, 0)?;
        let deadline_text = contract.deadline_ticks.to_string();
        writer.emit(
            "dispatch-sent",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("TICK", "0"),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
                ("PARENT_DISPATCH", contract.parent_dispatch.as_str()),
                ("EXECUTION_CLASS", contract.execution_class.as_str()),
                ("CORRELATION_ID", contract.correlation_id.as_str()),
                ("CLOSURE_LAW", "NONE"),
                ("DEADLINE_TICKS", deadline_text.as_str()),
            ],
        )?;
        writer.emit(
            "arazzo-workflow-dispatched",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("TICK", "0"),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
                ("TARGET_ENGINE", contract.target_engine.as_str()),
            ],
        )?;
        writer.emit(
            "remote-dispatch-sent",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("ENGINE_ID", coordinator_id),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
                ("TARGET_ENGINE", contract.target_engine.as_str()),
                ("CORRELATION_ID", contract.correlation_id.as_str()),
            ],
        )?;
    }
    writer.flush()?;
    Ok(specs.len())
}

/// Multi-engine collect phase (PROJ-727/728/729): for every contract of
/// the SAME deterministic plan (rebuilt from identical arguments), polls
/// the target engine's outbox (bounded by `max_polls`, real time only
/// behind the wait seam), receipts the boundary crossing
/// (remote_consequence_received), verifies the engine actually ledgered
/// the work (its per-dispatch ledger file replays non-empty), runs the
/// lawful re-entry pipeline + idempotent consume, and admits — every step
/// ledgered through the coordinator's own chain (reloaded and
/// prefix-verified from the dispatch phase; a torn tail refuses
/// `CNG_R11`). Then writes the SHACL-conformant quiescence file into each
/// engine's control dir, gates full admission, materializes the
/// coordinator ∪ engine evidence graph, and evaluates the DISTRIBUTED
/// marker set (`CNG_R20` on any false marker).
///
/// # Errors
/// `CNG_R25 DoubleAdmit` on a replayed idempotency key (calling collect
/// twice over the same root is the live falsifier); `CNG_R19
/// EvidenceGateFailed { gate: "remote-admission" }` when any contract
/// timed out or was refused; `CNG_R19 { gate: "remote-ledger-missing" }`
/// when a consequence exists without engine ledger evidence; `CNG_R17`
/// stages surface as consequence_refused evidence then the admission gate
/// refuses; `CNG_R20 MarkerFalse` for any false distributed marker;
/// `CNG_R11` for a torn coordinator ledger.
///
/// # Complexity
/// O(specs × max_polls) outbox checks + O(specs) re-entry pipelines +
/// O(obs + evidence triples) for materialization and marker evaluation.
#[allow(clippy::too_many_arguments)]
pub fn engine_collect_remote(
    root: &Path,
    coordinator_id: &str,
    engine_ids: &[&str],
    per_engine: usize,
    depth: u32,
    fan_out: usize,
    seed: u64,
    max_polls: u64,
    poll_wait_ms: Option<u64>,
) -> Result<EngineCoordinateReport, CngRefusal> {
    let templates = load_templates()?;
    let queries = QuerySet::load(&QuerySet::default_dir())?;
    let bundle = EngineBundle::new(root, coordinator_id)?;
    // Reload verifies every phase-1 chain prefix (torn tail = CNG_R11).
    let mut ledger = FileLedgerSink::new(&bundle.ledger_dir())?;
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shapes_path = manifest.join("shapes").join("dispatch-shapes.ttl");
    let quiesce_template = {
        let path = manifest
            .join("templates")
            .join("dispatch-quiesce.template.ttl");
        fs::read_to_string(&path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?
    };
    let obs_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("coordinator obs store construction: {e}")))?;
    let mut writer = ObsWriter::new(&templates, &obs_store, &bundle.ticks_dir(), "coord-collect")?
        .with_flush_threshold(1);
    let wait = make_wait(poll_wait_ms);

    let specs = remote_specs(engine_ids, per_engine, depth, fan_out);
    let mut received = 0usize;
    let mut admitted = 0usize;
    // dispatch id → consequence digest, canonical (BTreeMap) fold order.
    let mut receipts: BTreeMap<String, String> = BTreeMap::new();
    // Bounded collection: the loop bound IS the poll budget per contract.
    //
    // # Complexity
    // O(specs × max_polls) worst case.
    for spec in &specs {
        let mut contract = remote_contract(spec, seed);
        // Phase split: the MANUFACTURED→…→DISPATCHED prefix was ledgered by
        // the dispatch phase and chain-verified on reload above; the cursor
        // resumes at DISPATCHED without re-appending those entries.
        contract.state = DispatchState::Dispatched;
        let outbox_path = root
            .join("engines")
            .join(&spec.engine)
            .join("outbox")
            .join(format!("{}.ttl", contract.dispatch_id));
        let mut consequence_ttl: Option<String> = None;
        for _poll in 0..max_polls {
            if outbox_path.is_file() {
                consequence_ttl = Some(fs::read_to_string(&outbox_path).map_err(|e| {
                    CngRefusal::IoRefused(format!("read {}: {e}", outbox_path.display()))
                })?);
                break;
            }
            // Real time behind the seam only; nothing slept is receipted.
            wait.wait();
        }
        let Some(consequence_ttl) = consequence_ttl else {
            let deadline_text = contract.deadline_ticks.to_string();
            coord_advance(&mut ledger, &mut contract, DispatchState::TimedOut, 0)?;
            writer.emit(
                "dispatch-timed-out",
                &[
                    ("SET_ID", contract.dispatch_id.as_str()),
                    ("DISPATCH_ID", contract.dispatch_id.as_str()),
                    ("DEADLINE_TICKS", deadline_text.as_str()),
                ],
            )?;
            coord_advance(&mut ledger, &mut contract, DispatchState::Blocked, 0)?;
            continue;
        };
        received += 1;
        writer.emit(
            "remote-consequence-received",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("ENGINE_ID", coordinator_id),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
                ("SOURCE_ENGINE", spec.engine.as_str()),
            ],
        )?;
        // Engine acknowledgment evidence: the remote engine's own durable
        // per-dispatch ledger must replay non-empty (its serve loop appends
        // ACKNOWLEDGED→… before executing). A consequence file without
        // ledger evidence is exactly the bypass the isolation law forbids.
        let engine_ledger = root
            .join("engines")
            .join(&spec.engine)
            .join("ledger")
            .join(format!("{}.ttl", contract.dispatch_id));
        if read_ledger_entries(&engine_ledger)?.is_empty() {
            return Err(CngRefusal::EvidenceGateFailed {
                gate: "remote-ledger-missing".to_string(),
                count: 1,
            });
        }
        coord_advance(&mut ledger, &mut contract, DispatchState::Acknowledged, 0)?;
        writer.emit(
            "dispatch-acknowledged",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
            ],
        )?;
        coord_advance(&mut ledger, &mut contract, DispatchState::RemoteStarted, 0)?;
        coord_advance(
            &mut ledger,
            &mut contract,
            DispatchState::RemoteInProgress,
            0,
        )?;
        coord_advance(
            &mut ledger,
            &mut contract,
            DispatchState::ResultAvailable,
            0,
        )?;
        writer.emit(
            "consequence-returned",
            &[
                ("SET_ID", contract.dispatch_id.as_str()),
                ("DISPATCH_ID", contract.dispatch_id.as_str()),
                ("CORRELATION_ID", contract.correlation_id.as_str()),
            ],
        )?;
        coord_advance(&mut ledger, &mut contract, DispatchState::ResultReceived, 0)?;
        match collect_consequence(&consequence_ttl, &contract, &shapes_path, &queries) {
            Ok(()) => {
                // Idempotent consume (PROJ-721 law on the coordinator side):
                // a replayed key is CNG_R25 BEFORE admission has any effect.
                if ledger.is_processed(&contract.idempotency_key) {
                    return Err(CngRefusal::DoubleAdmit {
                        dispatch: contract.dispatch_id.clone(),
                        idempotency_key: contract.idempotency_key.clone(),
                    });
                }
                ledger.mark_processed(&contract.idempotency_key, &contract.dispatch_id)?;
                coord_advance(&mut ledger, &mut contract, DispatchState::ResultAdmitted, 0)?;
                writer.emit(
                    "consequence-admitted",
                    &[
                        ("SET_ID", contract.dispatch_id.as_str()),
                        ("DISPATCH_ID", contract.dispatch_id.as_str()),
                        ("CORRELATION_ID", contract.correlation_id.as_str()),
                    ],
                )?;
                coord_advance(&mut ledger, &mut contract, DispatchState::Completed, 0)?;
                admitted += 1;
                receipts.insert(
                    contract.dispatch_id.clone(),
                    format!(
                        "blake3:{}",
                        blake3::hash(consequence_ttl.as_bytes()).to_hex()
                    ),
                );
            }
            Err(CngRefusal::ExternalConsequenceRefused { stage, .. }) => {
                coord_advance(&mut ledger, &mut contract, DispatchState::Refused, 0)?;
                writer.emit(
                    "consequence-refused",
                    &[
                        ("SET_ID", contract.dispatch_id.as_str()),
                        ("DISPATCH_ID", contract.dispatch_id.as_str()),
                        ("STAGE", stage.as_str()),
                    ],
                )?;
                coord_advance(&mut ledger, &mut contract, DispatchState::Blocked, 0)?;
            }
            Err(other) => return Err(other),
        }
    }
    writer.flush()?;

    // Quiescence: end each engine's serve loop through the SHACL-validated
    // control surface (deterministic content; harmless if the engine
    // already exited on its poll budget). O(engines).
    for engine_id in engine_ids.iter().copied() {
        let body = fill_template(
            &quiesce_template,
            &[
                ("SUBJECT", &format!("quiesce-{coordinator_id}-{engine_id}")),
                ("ENGINE_ID", engine_id),
                ("REASON", "coordinator-complete"),
            ],
        );
        let path = root
            .join("engines")
            .join(engine_id)
            .join("control")
            .join("quiesce.ttl");
        write_atomic(&path, &body)?;
    }

    // Admission gate: every remote workflow must have completed lawfully.
    if admitted != specs.len() {
        return Err(CngRefusal::EvidenceGateFailed {
            gate: "remote-admission".to_string(),
            count: (specs.len() - admitted) as i64,
        });
    }

    // --- Evidence: union of the coordinator's AND every engine's eagerly
    // flushed observation partitions, materialized through the same OCEL
    // constructs as run()/workday()/audit_replay().
    //
    // # Complexity
    // O(obs bytes) parse + O(evidence triples log t) serialization.
    let union_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("union obs store construction: {e}")))?;
    let mut obs_paths: Vec<PathBuf> = Vec::new();
    collect_ttl_paths_recursive(&bundle.ticks_dir(), &mut obs_paths)?;
    for engine_id in engine_ids.iter().copied() {
        collect_ttl_paths_recursive(
            &root.join("engines").join(engine_id).join("ticks"),
            &mut obs_paths,
        )?;
    }
    obs_paths.sort();
    for path in &obs_paths {
        let turtle = fs::read_to_string(path)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
        union_store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .map_err(|e| CngRefusal::MalformedTtl(format!("obs load {}: {e}", path.display())))?;
    }
    let evidence_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("evidence store construction: {e}")))?;
    for construct in OCEL_CONSTRUCT_STEMS {
        run_construct(&union_store, queries.get(construct)?, &evidence_store)?;
    }
    let (evidence_nt, ocel_graph_digest) = evidence_digest(&evidence_store)?;
    fs::write(bundle.receipts_dir().join("ocel.nt"), &evidence_nt)
        .map_err(|e| CngRefusal::IoRefused(format!("write ocel.nt: {e}")))?;
    let engine_instances = metric_count(
        &evidence_store,
        queries.get("metric-engine-instances")?,
        "metric-engine-instances",
    )?;

    // --- Distributed markers (CNG_R20 on any false marker; the inverted
    // existence markers prove ≥ 2 engines + inter-engine Arazzo dispatch).
    let marker_queries = QuerySet::load(&QuerySet::default_dir().join("markers"))?;
    let marker_store = build_marker_store(
        &union_store,
        &evidence_store,
        &WorkdayHookBroker::default_hooks_dir().join("dialect-registry.ttl"),
    )?;
    let markers = evaluate_marker_map(&marker_store, &marker_queries, &DISTRIBUTED_MARKER_MAP)?;

    // Receipt chain: BLAKE3 fold over admitted pairs in dispatch-id order.
    //
    // # Complexity
    // O(admitted) hash updates.
    let mut chain = blake3::Hasher::new();
    for (dispatch_id, digest) in &receipts {
        chain.update(dispatch_id.as_bytes());
        chain.update(digest.as_bytes());
    }
    let report = EngineCoordinateReport {
        measurement_class: "MEASURED_CNG_RESULT",
        coordinator_id: coordinator_id.to_string(),
        engines: engine_ids.iter().map(|e| e.to_string()).collect(),
        contracts_dispatched: specs.len(),
        remote_consequences_received: received,
        consequences_admitted: admitted,
        engine_instances,
        markers,
        ocel_graph_digest,
        receipt_chain_digest: format!("blake3:{}", chain.finalize().to_hex()),
    };
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| CngRefusal::IoRefused(format!("coordinate report serialize: {e}")))?;
    fs::write(bundle.receipts_dir().join("coordinate-report.json"), &json)
        .map_err(|e| CngRefusal::IoRefused(format!("write coordinate-report.json: {e}")))?;
    Ok(report)
}

#[cfg(test)]
#[path = "engine_test.rs"]
mod engine_test;
