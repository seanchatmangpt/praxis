//! The hook-firing receipt — the outer envelope over the v1 workflow chain.
//!
//! One firing = one admitted event judged by the full registry, its fired
//! ground-actions executed, and everything folded into ONE outer chain
//! (`praxis:hook-firing:v1`):
//!
//! ```text
//! genesis ⊳ event_hash ⊳ admission_hash ⊳ handler_hash ⊳ hook_hash
//!         ⊳ inner chain per fired action (or the no-action sentinel)
//!         ⊳ assignment/outcome record hash
//! ```
//!
//! The inner v1 chain is folded AS AN EVENT — it is never mutated, so every
//! existing receipt, replay, and foreign verifier keeps working unchanged.
//! Declared refusals (a `refuse`-effect hook firing, an unknown handler, a
//! delegability violation) produce a receipt with `outcome: Refused` and
//! the chain folded through the refusing stage: refusals are chained, never
//! silent (knhk Covenant-2, imported as policy).

use serde::{Deserialize, Serialize};

use chatman_common::provenance::{content_address, fold_event, genesis_seed};

use crate::delta::{delta_ttl_hash, GraphDelta};
use crate::graph::WorkflowReceipt;
use crate::ground::ground_fired_action;
use crate::handlers::{extract_bindings, handler_hash, HandlerBinding, HandlerRegistry};
use crate::hooks::{evaluate_hooks, extract_hooks, hook_hash, EffectKind, HookVerdict,
    HookVerdictRecord};
use crate::quarantine::{Admission, AdmissionRecord, MeaningSource, Reference, RiceQuarantine};
use crate::Refusal;

/// Domain-separation tag for the firing chain.
pub const FIRING_CHAIN_DOMAIN: &str = "praxis:hook-firing:v1";

/// Folded when a firing executes no ground-action.
const NO_ACTION_SENTINEL: &str = "praxis:no-action";

/// Outcome of one firing. A refusal here is a first-class, chained result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiringOutcome {
    /// Every fired action executed; receipts embedded.
    Completed,
    /// The firing was lawfully refused at a named stage.
    Refused {
        /// The stage that refused: `handler` | `delegability` | `declared-refusal`.
        stage: String,
        /// The refusal, rendered (typed refusals carry their own data).
        reason: String,
    },
}

/// The outer receipt for one hook firing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookFiringReceipt {
    /// Content address of the exact delta surface bytes (field only,
    /// never folded — the `ttl_hash` doctrine).
    pub delta_ttl_hash: String,
    /// The delta's computed canonical event hash (fold 1).
    pub event_hash: String,
    /// The admission record and its hash (fold 2).
    pub admission: AdmissionRecord,
    /// Computed hash of `admission`.
    pub admission_hash: String,
    /// Graph-declared handler bindings and their canonical hash (fold 3).
    pub bindings: Vec<HandlerBinding>,
    /// Computed hash of the canonical binding form.
    pub handler_hash: String,
    /// Every registered hook's verdict (NotFired/Gated included) — fold 4
    /// hashes this list.
    pub verdicts: Vec<HookVerdictRecord>,
    /// Computed hash of `verdicts`.
    pub hook_hash: String,
    /// Inner v1 receipts, one per fired ground-action, in verdict order
    /// (each `chain` folded as one event; empty = sentinel fold).
    pub inner: Vec<WorkflowReceipt>,
    /// Outcome (fold 6 hashes outcome + assignment view).
    pub outcome: FiringOutcome,
    /// Computed hash of the outcome rendering.
    pub outcome_hash: String,
    /// The outer chain.
    pub chain: String,
}

fn json_hash<T: Serialize>(value: &T, what: &str) -> Result<String, Refusal> {
    let json = serde_json::to_string(value).map_err(|e| Refusal::InvalidInput {
        detail: format!("{what} failed to serialize: {e}"),
    })?;
    Ok(content_address(json.as_bytes()))
}

fn fold_firing_chain(
    event_hash: &str,
    admission_hash: &str,
    handler_hash: &str,
    hook_hash: &str,
    inner_chains: &[&str],
    outcome_hash: &str,
) -> String {
    let mut chain = genesis_seed(FIRING_CHAIN_DOMAIN);
    chain = fold_event(&chain, event_hash.as_bytes());
    chain = fold_event(&chain, admission_hash.as_bytes());
    chain = fold_event(&chain, handler_hash.as_bytes());
    chain = fold_event(&chain, hook_hash.as_bytes());
    if inner_chains.is_empty() {
        chain = fold_event(&chain, NO_ACTION_SENTINEL.as_bytes());
    } else {
        for inner in inner_chains {
            chain = fold_event(&chain, inner.as_bytes());
        }
    }
    fold_event(&chain, outcome_hash.as_bytes())
}

/// Fire the full pipeline for one meaning source: quarantine → admission →
/// handler judgment (BEFORE any solving) → hook evaluation → grounded
/// execution of fired actions → one chained receipt.
///
/// Hard failures of the door itself (malformed bytes, cap violations,
/// admission refusals) are `Err` — there is no admitted event to receipt.
/// LAWFUL refusals downstream of admission (unknown handler, delegability,
/// a declared `refuse`-effect firing) return `Ok` with
/// [`FiringOutcome::Refused`]: the refusal is part of the chain.
pub fn fire_hooks(
    reference: &Reference,
    source: &MeaningSource,
    registry: &HandlerRegistry,
    history: &[GraphDelta],
) -> Result<HookFiringReceipt, Refusal> {
    let delta = RiceQuarantine::inspect(source)?;
    let delta_ttl_hash = delta_ttl_hash(&source.adds_ttl, &source.removes_ttl);
    let event_hash = delta.event_hash();
    let event = Admission::admit(reference, &delta)?;
    let admission = event.record.clone();
    let admission_hash = admission.admission_hash()?;

    // Handler judgment BEFORE solving.
    let bindings = extract_bindings(&event.post)?;
    let handler_hash_v = handler_hash(&bindings);
    if let Err(refusal) = registry.judge(&bindings) {
        let stage = match &refusal {
            Refusal::UnknownHandler { .. } => "handler",
            _ => "delegability",
        };
        let outcome =
            FiringOutcome::Refused { stage: stage.to_string(), reason: refusal.to_string() };
        let outcome_hash = json_hash(&outcome, "outcome")?;
        // Hooks were never evaluated: the verdict list is empty and its
        // hash covers exactly that emptiness.
        let verdicts: Vec<HookVerdictRecord> = Vec::new();
        let hook_hash_v = hook_hash(&verdicts)?;
        let chain = fold_firing_chain(
            &event_hash,
            &admission_hash,
            &handler_hash_v,
            &hook_hash_v,
            &[],
            &outcome_hash,
        );
        return Ok(HookFiringReceipt {
            delta_ttl_hash,
            event_hash,
            admission,
            admission_hash,
            bindings,
            handler_hash: handler_hash_v,
            verdicts,
            hook_hash: hook_hash_v,
            inner: Vec::new(),
            outcome,
            outcome_hash,
            chain,
        });
    }

    let hooks = extract_hooks(&event.post)?;
    let verdicts = evaluate_hooks(&hooks, &event, history)?;
    let hook_hash_v = hook_hash(&verdicts)?;

    // Declared refusal: the highest-priority fired refuse-effect hook wins.
    let declared_refusal = verdicts.iter().find(|r| {
        r.verdict == HookVerdict::Fired && r.effect == EffectKind::Refuse
    });
    let (inner, outcome) = if let Some(r) = declared_refusal {
        let hook = hooks.iter().find(|h| h.iri == r.hook_iri);
        let reason = hook
            .and_then(|h| h.reason.clone())
            .unwrap_or_else(|| "declared refusal".to_string());
        (Vec::new(), FiringOutcome::Refused { stage: "declared-refusal".to_string(), reason })
    } else {
        let mut inner = Vec::new();
        for record in &verdicts {
            if record.verdict == HookVerdict::Fired && record.effect == EffectKind::GroundAction {
                inner.push(ground_fired_action(&event, record)?);
            }
        }
        (inner, FiringOutcome::Completed)
    };
    let outcome_hash = json_hash(&outcome, "outcome")?;
    let inner_chains: Vec<&str> = inner.iter().map(|r| r.chain.as_str()).collect();
    let chain = fold_firing_chain(
        &event_hash,
        &admission_hash,
        &handler_hash_v,
        &hook_hash_v,
        &inner_chains,
        &outcome_hash,
    );
    Ok(HookFiringReceipt {
        delta_ttl_hash,
        event_hash,
        admission,
        admission_hash,
        bindings,
        handler_hash: handler_hash_v,
        verdicts,
        hook_hash: hook_hash_v,
        inner,
        outcome,
        outcome_hash,
        chain,
    })
}

/// Independently re-derive the whole firing from (base TTL, delta docs) and
/// compare stage by stage in fold order; then bind every embedded payload
/// to the hash just verified — a receipt whose hashes are honest but whose
/// bodies are forged must not pass. A receipt cannot vouch for itself.
pub fn replay_firing(
    receipt: &HookFiringReceipt,
    base_ttl: &str,
    source: &MeaningSource,
    registry: &HandlerRegistry,
    history: &[GraphDelta],
) -> Result<(), Refusal> {
    let reference = Reference::genesis(base_ttl)?;
    let rederived = fire_hooks(&reference, source, registry, history)?;
    let stages: [(&str, &str, &str); 6] = [
        ("event_hash", &rederived.event_hash, &receipt.event_hash),
        ("admission_hash", &rederived.admission_hash, &receipt.admission_hash),
        ("handler_hash", &rederived.handler_hash, &receipt.handler_hash),
        ("hook_hash", &rederived.hook_hash, &receipt.hook_hash),
        ("outcome_hash", &rederived.outcome_hash, &receipt.outcome_hash),
        ("chain", &rederived.chain, &receipt.chain),
    ];
    for (name, computed, claimed) in stages {
        if computed != claimed {
            return Err(Refusal::VerificationFailed { failed: vec![name.to_string()] });
        }
    }
    // Payload bindings: bodies must reproduce the verified hashes.
    if receipt.admission.admission_hash()? != receipt.admission_hash {
        return Err(Refusal::VerificationFailed { failed: vec!["admission payload".to_string()] });
    }
    if handler_hash(&receipt.bindings) != receipt.handler_hash {
        return Err(Refusal::VerificationFailed { failed: vec!["binding payload".to_string()] });
    }
    if hook_hash(&receipt.verdicts)? != receipt.hook_hash {
        return Err(Refusal::VerificationFailed { failed: vec!["verdict payload".to_string()] });
    }
    if json_hash(&receipt.outcome, "outcome")? != receipt.outcome_hash {
        return Err(Refusal::VerificationFailed { failed: vec!["outcome payload".to_string()] });
    }
    if rederived.inner.len() != receipt.inner.len()
        || rederived
            .inner
            .iter()
            .zip(receipt.inner.iter())
            .any(|(a, b)| a.chain != b.chain)
    {
        return Err(Refusal::VerificationFailed { failed: vec!["inner chains".to_string()] });
    }
    Ok(())
}
