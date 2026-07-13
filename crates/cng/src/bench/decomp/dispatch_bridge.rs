//! Decompose-to-dispatch bridge (v26.7.10-revised closure work): converts
//! one Track P [`super::SubworkflowPlan`] (a decomposed helper/main
//! subworkflow, PROJ-706..710) into a Track E `DispatchContract` (PROJ-720)
//! addressed to a named target engine, writes it into that engine's REAL
//! filesystem inbox, and — once a spawned `cng engine serve` process has
//! executed it — collects and lawfully re-admits the consequence from that
//! engine's outbox.
//!
//! This is the bridge the two tracks lacked: before this file, nothing
//! carried a [`super::SubworkflowPlan`]'s identity into a
//! [`crate::bench::dispatch`] contract, and nothing routed a decomposition
//! candidate onto the multi-engine transport
//! (`crates/cng/src/bench/engine.rs`). `DispatchContract` and its
//! surrounding machinery stay crate-private (`pub(super)` to
//! [`crate::bench`]); this module's two entry points —
//! [`dispatch_subworkflow_to_engine`] and [`collect_subworkflow_consequence`]
//! — are the only parts exposed outside the crate, each returning an opaque
//! handle/outcome so no private dispatch type leaks across the crate
//! boundary.
//!
//! Payload-carrying dispatch (PROJ-710 → PROJ-723 closure): when a bridged
//! [`SubworkflowPlan`] carries a real manufactured PDDL problem
//! (`problem_pddl` non-empty — every split-candidate `helper`/`main`
//! subworkflow; empty only for the `single`-role fallback, which has
//! nothing of its own to send), [`dispatch_subworkflow_to_engine`] writes
//! TWO sibling files into the SAME target-engine inbox directory the
//! contract `.ttl` lands in: `<dispatch_id>.domain.pddl` (the shared
//! decomposition domain's PDDL text, supplied by the caller — decomposition
//! never rewrites the domain, only the per-subworkflow problem) and
//! `<dispatch_id>.pddl` (the subworkflow's own `problem_pddl`). The
//! contract's `disp:inputArtifactSet` is set to `payload-<digest>`, a
//! length-prefixed BLAKE3 fold over both texts — a real content digest, not
//! the old synthetic `inputs-<dispatch_id>` label — so the receiving engine
//! can verify the sibling payload it reads is exactly what was dispatched
//! (`engine.rs:run_serve_loop`, `CNG_R11 AuditMismatch` on any divergence).
//! The payload files are written and fsync-renamed (`write_atomic`) BEFORE
//! the contract `.ttl` itself, so a concurrent engine's sorted inbox scan
//! never observes a contract whose declared payload isn't there yet.
//!
//! `crate::bench::engine::run_serve_loop` grounds and plans directly from
//! this payload when present; contracts with no sibling payload (workday's
//! own synthetic contracts, the multi-engine coordinator's
//! `remote_contract`) execute exactly as before — the payload path is
//! purely additive. See
//! `crates/cng/tests/cng_decompose_to_dispatch_integration.rs` for the test
//! that proves the engine's own manufactured evidence traces back to the
//! specific dispatched subworkflow's content, not a shared synthetic seed.
//!
//! No wall clock enters any digest here: every dispatch id, key, and digest
//! is content-derived (BLAKE3 over subworkflow identity / rendered text /
//! payload text); the only real-time element is the bounded inter-poll wait
//! behind [`crate::bench::dispatch::RealTimeWait`], never serialized.

use std::fs;
use std::path::{Path, PathBuf};

use crate::bench::dispatch::{
    collect_consequence, shape_violations, write_atomic, DispatchContract, DispatchState,
    ExecutionClass, NoWait, RealTimeWait, ThreadSleepWait,
};
use crate::bench::engine::EngineBundle;
use crate::bench::templates::QuerySet;
use crate::powl::CngRefusal;

use super::SubworkflowPlan;

/// Short content-derived key: first 12 hex chars of `blake3(text)`. O(|text|).
fn short_key(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex()[..12].to_string()
}

/// Real content digest of a dispatched PDDL payload: length-prefixed
/// `blake3(len(domain_pddl) | domain_pddl | len(problem_pddl) | problem_pddl)`,
/// first 16 hex chars. Length-prefixing (rather than bare concatenation)
/// means two different `(domain, problem)` splits can never fold to the same
/// digest by an accidental shared boundary. `pub(crate)` — the receiving
/// side (`crate::bench::engine::run_serve_loop`) recomputes this SAME fold
/// over the bytes it reads off the sibling files and refuses `CNG_R11` on
/// any mismatch, so both sides must call the one function, never reimplement
/// it. O(|domain_pddl| + |problem_pddl|).
pub(crate) fn payload_digest(domain_pddl: &str, problem_pddl: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(domain_pddl.len() as u64).to_le_bytes());
    hasher.update(domain_pddl.as_bytes());
    hasher.update(&(problem_pddl.len() as u64).to_le_bytes());
    hasher.update(problem_pddl.as_bytes());
    hasher.finalize().to_hex()[..16].to_string()
}

/// Builds the sealed dispatch contract for one decomposed subworkflow,
/// addressed to `target_engine`. Every field is content-derived from
/// `(subworkflow.id, subworkflow.role, subworkflow.problem_digest,
/// target_engine)` — no wall clock, no PID, no path — matching the
/// determinism discipline of `dispatch::workday_contract` /
/// `engine::remote_contract`. Crate-private: `DispatchContract` cannot
/// leave the crate boundary (see module docs).
///
/// When `subworkflow.problem_pddl` is non-empty (every `helper`/`main`
/// split subworkflow; empty for `single`), `disp:inputArtifactSet` carries
/// the REAL payload digest (`payload-<16 hex>` via [`payload_digest`]) over
/// `(domain_pddl, subworkflow.problem_pddl)` instead of the synthetic
/// `inputs-<dispatch_id>` label — the value [`dispatch_subworkflow_to_engine`]
/// later verifies the sibling payload files against.
///
/// # Complexity
/// O(1) when `subworkflow.problem_pddl` is empty; otherwise
/// O(|domain_pddl| + |subworkflow.problem_pddl|) for the payload digest.
pub(super) fn subworkflow_to_contract(
    subworkflow: &SubworkflowPlan,
    domain_pddl: &str,
    target_engine: &str,
) -> DispatchContract {
    let identity_key = short_key(&format!(
        "{}|{}|{}",
        subworkflow.id, subworkflow.role, subworkflow.problem_digest
    ));
    let dispatch_id = format!("disp-decomp-{}-{identity_key}", subworkflow.role);
    let input_artifact_set = if subworkflow.problem_pddl.is_empty() {
        format!("inputs-{dispatch_id}")
    } else {
        format!(
            "payload-{}",
            payload_digest(domain_pddl, &subworkflow.problem_pddl)
        )
    };
    DispatchContract {
        dispatch_id: dispatch_id.clone(),
        workflow_instance: format!("wf-{dispatch_id}"),
        parent_workflow: format!("wf-{dispatch_id}"),
        recursive_depth: 0,
        target_actor: "external-machine-executor".to_string(),
        target_engine: target_engine.to_string(),
        required_role: "operator".to_string(),
        declared_authority: format!("decomp-dispatch-authority-{}", subworkflow.role),
        input_artifact_set,
        expected_output_artifact_set: format!("outputs-{dispatch_id}"),
        activity_identity: format!("decomp-{}-{dispatch_id}", subworkflow.role),
        deadline_ticks: 64,
        idempotency_key: format!("idem-{}", short_key(&format!("idem|{dispatch_id}"))),
        correlation_id: format!("corr-{}", short_key(&format!("corr|{dispatch_id}"))),
        collection_surface: format!("engines/{target_engine}/outbox"),
        retry_law: "retry:limit=0;declarative-only".to_string(),
        escalation_law: "escalate:manufacture-escalation-workflow".to_string(),
        compensation_law: "compensate:manufacture-compensation-workflow".to_string(),
        refusal_conditions: "provenance,correlation,authority,structural,semantic".to_string(),
        receipt_requirements: "obs-receipt-per-transition".to_string(),
        replay_requirements: "byte-identical-same-seed".to_string(),
        state: DispatchState::Manufactured,
        execution_class: ExecutionClass::ExternalMachineDispatch,
        closure_law: None,
        parent_dispatch: "none".to_string(),
    }
}

/// Opaque handle to one dispatched subworkflow contract. Carries no private
/// dispatch-machinery type across the crate boundary (the `DispatchContract`
/// field is private to this struct); callers outside the crate can hold and
/// pass this value but cannot construct or inspect it directly.
#[derive(Debug)]
pub struct SubworkflowDispatchHandle {
    /// The rendered contract's `disp:dispatchId` (also its inbox/outbox
    /// filename stem).
    pub dispatch_id: String,
    /// The subworkflow role this handle was dispatched for (`helper` |
    /// `main` | `single`).
    pub role: String,
    /// The engine this contract was addressed to.
    pub target_engine: String,
    contract: DispatchContract,
}

/// Terminal outcome of one bridged subworkflow dispatch, collected from a
/// real engine process's outbox.
#[derive(Debug)]
pub struct SubworkflowDispatchOutcome {
    pub dispatch_id: String,
    pub role: String,
    pub target_engine: String,
    /// Whether a consequence was found on the target engine's outbox within
    /// the poll budget AND passed the lawful re-entry pipeline
    /// (`collect_consequence`: provenance, correlation, authority,
    /// structural, semantic — in order).
    pub admitted: bool,
    /// Logical polls consumed before a consequence was found (or the budget
    /// was exhausted).
    pub polls_taken: u64,
    /// `blake3:<hex>` of the raw consequence Turtle, present whenever a
    /// consequence file was found (independent of `admitted`).
    pub consequence_digest: Option<String>,
    /// The raw consequence Turtle text itself, present under the same
    /// condition as `consequence_digest` (a file was found within the poll
    /// budget, independent of `admitted`). Exists so a caller that needs the
    /// actual content -- not just its digest -- can re-admit it through its
    /// own downstream pipeline without re-reading the outbox file directly
    /// (which would require reconstructing `EngineBundle`'s private layout).
    pub consequence_turtle: Option<String>,
}

/// Renders and writes one subworkflow's dispatch contract directly into
/// `target_engine`'s real inbox under `root` (`EngineBundle` layout,
/// PROJ-722) — the same on-disk contract format `engine_dispatch_remote`
/// writes and a real `cng engine serve` process organically scans and
/// admits (sorted lexicographic inbox scan, `dispatch-contract.template.ttl`
/// + `DispatchContractShape`). CNG_R15 refuses BEFORE any file is written
/// if the rendered contract is incomplete or shape-violating.
///
/// `domain_pddl` is the decomposition's shared domain PDDL text (the SAME
/// text across every subworkflow of one `decompose()` run — decomposition
/// only manufactures per-subworkflow PROBLEMs, never a new domain). When
/// `subworkflow.problem_pddl` is non-empty, two sibling payload files are
/// written into the SAME inbox directory as the contract, BEFORE the
/// contract itself becomes visible: `<dispatch_id>.domain.pddl` and
/// `<dispatch_id>.pddl` (the problem). `domain_pddl` is ignored (may be
/// empty) when `subworkflow.problem_pddl` is empty (the `single`-role
/// fallback carries nothing of its own to dispatch).
///
/// # Errors
/// `CNG_R10` for template/shape-file IO or payload-file IO; `CNG_R15` for an
/// incomplete or shape-violating contract; `CNG_R16` should the internal
/// state-cursor advance ever leave the lawful transition table (defensive;
/// unreachable in this fixed sequence).
///
/// # Complexity
/// O(template bytes) render + one shape SELECT pair + one atomic contract
/// write + (when a payload is carried) two atomic payload writes, O(|domain_
/// pddl| + |subworkflow.problem_pddl|).
pub fn dispatch_subworkflow_to_engine(
    root: &Path,
    subworkflow: &SubworkflowPlan,
    domain_pddl: &str,
    target_engine: &str,
) -> Result<SubworkflowDispatchHandle, CngRefusal> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let contract_template_path = manifest
        .join("templates")
        .join("dispatch-contract.template.ttl");
    let contract_template = fs::read_to_string(&contract_template_path).map_err(|e| {
        CngRefusal::IoRefused(format!("read {}: {e}", contract_template_path.display()))
    })?;
    let shapes_path = manifest.join("shapes").join("dispatch-shapes.ttl");
    let queries = QuerySet::load(&QuerySet::default_dir())?;

    let mut contract = subworkflow_to_contract(subworkflow, domain_pddl, target_engine);
    // Render fixes the on-disk `disp:dispatchState` at MANUFACTURED (mirrors
    // `DispatchAdapter::dispatch`: render happens before any state advance).
    let rendered = contract.render(&contract_template)?;
    let violations = shape_violations(&rendered, &shapes_path, &queries)?;
    if let Some((entry, field)) = violations.first() {
        return Err(CngRefusal::DispatchContractIncomplete {
            dispatch: entry.clone(),
            missing: field.clone(),
        });
    }
    contract.advance(DispatchState::ArazzoRendered)?;
    contract.advance(DispatchState::DispatchReady)?;

    let bundle = EngineBundle::new(root, target_engine)?;
    // Payload BEFORE contract: a sorted inbox scan (`engine.rs:run_serve_loop`)
    // that observes the `.ttl` must always find its sibling payload already
    // in place — never a torn/partial dispatch.
    if !subworkflow.problem_pddl.is_empty() {
        let domain_payload_path = bundle
            .inbox_dir()
            .join(format!("{}.domain.pddl", contract.dispatch_id));
        let problem_payload_path = bundle
            .inbox_dir()
            .join(format!("{}.pddl", contract.dispatch_id));
        write_atomic(&domain_payload_path, domain_pddl)?;
        write_atomic(&problem_payload_path, &subworkflow.problem_pddl)?;
    }
    let path = bundle
        .inbox_dir()
        .join(format!("{}.ttl", contract.dispatch_id));
    write_atomic(&path, &rendered)?;
    contract.advance(DispatchState::Dispatched)?;

    Ok(SubworkflowDispatchHandle {
        dispatch_id: contract.dispatch_id.clone(),
        role: subworkflow.role.clone(),
        target_engine: target_engine.to_string(),
        contract,
    })
}

/// Bounded poll loop over `target_engine`'s real outbox for the consequence
/// of one dispatched subworkflow, then the SAME lawful re-entry pipeline the
/// coordinator uses (`collect_consequence`: provenance → correlation →
/// authority → structural → semantic, first failing stage wins). Never
/// blocks past `max_polls`; the inter-poll wait is real time behind
/// [`RealTimeWait`] and is never serialized into `polls_taken` or the
/// returned digest.
///
/// # Errors
/// `CNG_R10` for template/shape-file IO; `CNG_R16` should the internal
/// state-cursor advance ever leave the lawful transition table (defensive).
/// A refused/absent consequence is NOT a `Result::Err` here — it is the
/// typed `admitted: false` outcome (mirrors
/// `DispatchAdapter::dispatch`'s own "consequence-stage failures are
/// receipted evidence, not errors" rule).
///
/// # Complexity
/// O(`max_polls`) outbox stats + one lawful-re-entry pass on the poll that
/// finds a file.
pub fn collect_subworkflow_consequence(
    root: &Path,
    handle: &SubworkflowDispatchHandle,
    max_polls: u64,
    poll_wait_ms: Option<u64>,
) -> Result<SubworkflowDispatchOutcome, CngRefusal> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shapes_path = manifest.join("shapes").join("dispatch-shapes.ttl");
    let queries = QuerySet::load(&QuerySet::default_dir())?;
    let wait: Box<dyn RealTimeWait> = match poll_wait_ms {
        Some(millis) => Box::new(ThreadSleepWait { millis }),
        None => Box::new(NoWait),
    };

    let bundle = EngineBundle::new(root, &handle.target_engine)?;
    let path = bundle
        .outbox_dir()
        .join(format!("{}.ttl", handle.dispatch_id));

    let mut polls_taken = 0u64;
    let mut consequence_ttl: Option<String> = None;
    for _ in 0..max_polls {
        polls_taken += 1;
        if path.is_file() {
            consequence_ttl = Some(
                fs::read_to_string(&path)
                    .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?,
            );
            break;
        }
        wait.wait();
    }

    let Some(consequence_ttl) = consequence_ttl else {
        return Ok(SubworkflowDispatchOutcome {
            dispatch_id: handle.dispatch_id.clone(),
            role: handle.role.clone(),
            target_engine: handle.target_engine.clone(),
            admitted: false,
            polls_taken,
            consequence_digest: None,
            consequence_turtle: None,
        });
    };
    let consequence_digest = format!(
        "blake3:{}",
        blake3::hash(consequence_ttl.as_bytes()).to_hex()
    );

    let mut contract = handle.contract.clone();
    contract.advance(DispatchState::Acknowledged)?;
    contract.advance(DispatchState::RemoteStarted)?;
    contract.advance(DispatchState::RemoteInProgress)?;
    contract.advance(DispatchState::ResultAvailable)?;
    contract.advance(DispatchState::ResultReceived)?;

    let admitted = match collect_consequence(&consequence_ttl, &contract, &shapes_path, &queries) {
        Ok(()) => {
            contract.advance(DispatchState::ResultAdmitted)?;
            contract.advance(DispatchState::Completed)?;
            true
        }
        Err(_) => false,
    };

    Ok(SubworkflowDispatchOutcome {
        dispatch_id: handle.dispatch_id.clone(),
        role: handle.role.clone(),
        target_engine: handle.target_engine.clone(),
        admitted,
        polls_taken,
        consequence_digest: Some(consequence_digest),
        consequence_turtle: Some(consequence_ttl),
    })
}
