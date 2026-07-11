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
//! Honest scope: the target engine's serve loop (`engine.rs:run_serve_loop`)
//! derives its OWN deterministic PDDL artifact set from
//! `blake3(dispatch_id)` (`write_set`, seeded, category hardcoded to
//! `"email-routing"`) — contracts do not yet carry their own PDDL payload
//! (see the module doc on `crate::bench::engine`: "the decomposition
//! track's integration point (PROJ-710 → PROJnn-723)" is still open). So a
//! contract built here identifies WHICH subworkflow was dispatched and
//! WHERE (content-derived `dispatch_id`/`activity_identity`/
//! `problem_digest`-keyed idempotency), and proves that identity round-trips
//! through a REAL second OS process's lawful re-entry pipeline — it does
//! NOT prove the remote engine executed that subworkflow's own PDDL plan.
//! See `crates/cng/tests/cng_decompose_to_dispatch_integration.rs` for the
//! test that exercises this end to end and states exactly this boundary.
//!
//! No wall clock enters any digest here: every dispatch id, key, and digest
//! is content-derived (BLAKE3 over subworkflow identity / rendered text);
//! the only real-time element is the bounded inter-poll wait behind
//! [`crate::bench::dispatch::RealTimeWait`], never serialized.

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

/// Builds the sealed dispatch contract for one decomposed subworkflow,
/// addressed to `target_engine`. Every field is content-derived from
/// `(subworkflow.id, subworkflow.role, subworkflow.problem_digest,
/// target_engine)` — no wall clock, no PID, no path — matching the
/// determinism discipline of `dispatch::workday_contract` /
/// `engine::remote_contract`. Crate-private: `DispatchContract` cannot
/// leave the crate boundary (see module docs).
///
/// # Complexity
/// O(1).
pub(super) fn subworkflow_to_contract(
    subworkflow: &SubworkflowPlan,
    target_engine: &str,
) -> DispatchContract {
    let identity_key = short_key(&format!(
        "{}|{}|{}",
        subworkflow.id, subworkflow.role, subworkflow.problem_digest
    ));
    let dispatch_id = format!("disp-decomp-{}-{identity_key}", subworkflow.role);
    DispatchContract {
        dispatch_id: dispatch_id.clone(),
        workflow_instance: format!("wf-{dispatch_id}"),
        parent_workflow: format!("wf-{dispatch_id}"),
        recursive_depth: 0,
        target_actor: "external-machine-executor".to_string(),
        target_engine: target_engine.to_string(),
        required_role: "operator".to_string(),
        declared_authority: format!("decomp-dispatch-authority-{}", subworkflow.role),
        input_artifact_set: format!("inputs-{dispatch_id}"),
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
}

/// Renders and writes one subworkflow's dispatch contract directly into
/// `target_engine`'s real inbox under `root` (`EngineBundle` layout,
/// PROJ-722) — the same on-disk contract format `engine_dispatch_remote`
/// writes and a real `cng engine serve` process organically scans and
/// admits (sorted lexicographic inbox scan, `dispatch-contract.template.ttl`
/// + `DispatchContractShape`). CNG_R15 refuses BEFORE any file is written
/// if the rendered contract is incomplete or shape-violating.
///
/// # Errors
/// `CNG_R10` for template/shape-file IO; `CNG_R15` for an incomplete or
/// shape-violating contract; `CNG_R16` should the internal state-cursor
/// advance ever leave the lawful transition table (defensive; unreachable
/// in this fixed sequence).
///
/// # Complexity
/// O(template bytes) render + one shape SELECT pair + one atomic file
/// write.
pub fn dispatch_subworkflow_to_engine(
    root: &Path,
    subworkflow: &SubworkflowPlan,
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

    let mut contract = subworkflow_to_contract(subworkflow, target_engine);
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
    })
}
