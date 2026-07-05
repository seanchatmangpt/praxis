//! The hook-firing receipt — the outer envelope over the v1 workflow chain.
//!
//! One firing = one admitted event judged by the full registry, its fired
//! ground-actions executed, and everything folded into ONE outer chain
//! (`praxis:hook-firing:v1`):
//!
//! ```text
//! genesis ⊳ event_hash ⊳ admission_hash ⊳ handler_hash ⊳ hook_hash
//!         ⊳ history_hash (the window-history commitment)
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

use crate::agent_registry::{agent_registry_hash, extract_agents, spawn_depth_law, AgentProfile};
use crate::delta::{delta_ttl_hash, GraphDelta};
use crate::graph::WorkflowReceipt;
use crate::ground::ground_fired_action;
use crate::handlers::{extract_bindings, handler_hash, HandlerBinding, HandlerRegistry};
use crate::hooks::{
    evaluate_hooks, extract_hooks, hook_hash, EffectKind, HookVerdict, HookVerdictRecord,
};
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
        /// The stage that refused: `handler` | `kernel-boundary` |
        /// `delegability` | `declared-refusal`.
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
    /// Graph-declared agent registry (tool sets, spawn edges, layer depth)
    /// — judged by [`spawn_depth_law`] in the same pre-solve, global phase
    /// as handler existence (fold 3.5, folded between `handler_hash` and
    /// `hook_hash`: graph-declared config judged before hook evaluation).
    pub agents: Vec<AgentProfile>,
    /// Computed hash of the canonical agent registry form.
    pub agent_registry_hash: String,
    /// Every registered hook's verdict (NotFired/Gated included) — fold 4
    /// hashes this list.
    pub verdicts: Vec<HookVerdictRecord>,
    /// Computed hash of `verdicts`.
    pub hook_hash: String,
    /// Computed commitment to the window history the verdicts were judged
    /// against (fold 5): only up to the first 7 deltas (max window − 1) can
    /// influence any verdict, so exactly those are committed. Replaying
    /// against a different history is a verification failure even when the
    /// verdicts coincide.
    pub history_hash: String,
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

/// Domain-separation tag for the window-history commitment.
const HISTORY_DOMAIN: &str = "praxis:window-history:v1";

/// Only this many history deltas (max `window` − 1) can influence any
/// window verdict, so exactly this prefix is committed.
const HISTORY_COMMIT_LEN: usize = 7;

/// Computed commitment to the effective window history: the domain tag
/// plus each committed delta's computed `event_hash`, one per line. An
/// empty history commits to the bare domain line.
#[must_use]
pub fn window_history_hash(history: &[GraphDelta]) -> String {
    let mut lines = String::from(HISTORY_DOMAIN);
    for delta in history.iter().take(HISTORY_COMMIT_LEN) {
        lines.push('\n');
        lines.push_str(&delta.event_hash());
    }
    content_address(lines.as_bytes())
}

// One argument per named fold stage, in fold order — deliberately explicit
// rather than bundled into a struct, so the fold order at each call site
// reads directly against this signature.
#[allow(clippy::too_many_arguments)]
fn fold_firing_chain(
    event_hash: &str,
    admission_hash: &str,
    handler_hash: &str,
    agent_registry_hash: &str,
    hook_hash: &str,
    history_hash: &str,
    inner_chains: &[&str],
    outcome_hash: &str,
) -> String {
    let mut chain = genesis_seed(FIRING_CHAIN_DOMAIN);
    chain = fold_event(&chain, event_hash.as_bytes());
    chain = fold_event(&chain, admission_hash.as_bytes());
    chain = fold_event(&chain, handler_hash.as_bytes());
    chain = fold_event(&chain, agent_registry_hash.as_bytes());
    chain = fold_event(&chain, hook_hash.as_bytes());
    chain = fold_event(&chain, history_hash.as_bytes());
    if inner_chains.is_empty() {
        chain = fold_event(&chain, NO_ACTION_SENTINEL.as_bytes());
    } else {
        for inner in inner_chains {
            chain = fold_event(&chain, inner.as_bytes());
        }
    }
    fold_event(&chain, outcome_hash.as_bytes())
}

/// Refuse a delta that tries to introduce a brand-new `hook:Hook` /
/// `wf:Workflow` / `wf:Capability` class definition. Hook and capability LAW
/// is graph-declared at genesis only — a delta may assert DATA (facts a hook
/// condition watches) but must never mint new executable definitions, which
/// would let a proposer smuggle an unreviewed workflow straight into a
/// firing.
fn vocab_check(delta: &GraphDelta) -> Result<(), Refusal> {
    let hook_hook = format!("{}Hook", crate::hooks::HOOK_NS);
    let wf_workflow = format!("{}Workflow", crate::graph::WF_NS);
    let wf_capability = format!("{}Capability", crate::graph::WF_NS);
    let rdf_type = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

    for t in delta.additions() {
        if t.s == hook_hook || t.s == wf_workflow || t.s == wf_capability {
            return Err(Refusal::AdmissionRefused {
                subject: t.s.clone(),
                detail: format!(
                    "modifying or defining the vocabulary class '{}' is forbidden in deltas",
                    t.s
                ),
            });
        }
        if t.p == rdf_type {
            if let crate::graph::Object::Iri(class) = &t.o {
                if class == &hook_hook || class == &wf_workflow || class == &wf_capability {
                    return Err(Refusal::AdmissionRefused {
                        subject: t.s.clone(),
                        detail: format!(
                            "proposing a new class definition of type '{}' is forbidden in deltas",
                            class
                        ),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Fire the full pipeline for one meaning source: quarantine → admission →
/// handler-existence judgment (global, BEFORE any solving) → hook
/// evaluation → grounded execution of fired actions with delegability
/// judged per fired action against its plan's capabilities → one chained
/// receipt.
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
    vocab_check(&delta)?;
    let delta_ttl_hash = delta_ttl_hash(&source.adds_ttl, &source.removes_ttl);
    let event_hash = delta.event_hash();
    let event = Admission::admit(reference, &delta)?;
    let admission = event.record().clone();
    let admission_hash = admission.admission_hash()?;

    // Handler EXISTENCE judgment is global and runs BEFORE any solving:
    // an unknown handler IRI anywhere in the graph refuses the firing.
    // Delegability is judged later, PER FIRED ACTION, against the
    // capabilities that action's derived plan actually uses.
    let bindings = extract_bindings(reference.triples())?;
    let handler_hash_v = handler_hash(&bindings);
    let history_hash_v = window_history_hash(history);

    // Graph-declared agent registry (tool sets, spawn edges, layer depth)
    // is extracted in the SAME pre-solve, global phase as handler bindings
    // — it is graph-declared config judged before hook evaluation, not a
    // per-fired-action judgment. `agents` is malformed-shape data, so a
    // hard `?` (never a lawful, chained refusal) mirrors `extract_bindings`.
    let agents = extract_agents(event.post())?;
    let agent_registry_hash_v = agent_registry_hash(&agents);

    // Pre-evaluation lawful refusals share one receipt shape: hooks were
    // never evaluated, so the verdict list is empty and its hash covers
    // exactly that emptiness.
    let pre_evaluation_refusal =
        |stage: &str, reason: String| -> Result<HookFiringReceipt, Refusal> {
            let outcome = FiringOutcome::Refused {
                stage: stage.to_string(),
                reason,
            };
            let outcome_hash = json_hash(&outcome, "outcome")?;
            let verdicts: Vec<HookVerdictRecord> = Vec::new();
            let hook_hash_v = hook_hash(&verdicts)?;
            let chain = fold_firing_chain(
                &event_hash,
                &admission_hash,
                &handler_hash_v,
                &agent_registry_hash_v,
                &hook_hash_v,
                &history_hash_v,
                &[],
                &outcome_hash,
            );
            Ok(HookFiringReceipt {
                delta_ttl_hash: delta_ttl_hash.clone(),
                event_hash: event_hash.clone(),
                admission: admission.clone(),
                admission_hash: admission_hash.clone(),
                bindings: bindings.clone(),
                handler_hash: handler_hash_v.clone(),
                agents: agents.clone(),
                agent_registry_hash: agent_registry_hash_v.clone(),
                verdicts,
                hook_hash: hook_hash_v,
                history_hash: history_hash_v.clone(),
                inner: Vec::new(),
                outcome,
                outcome_hash,
                chain,
            })
        };

    if let Err(refusal) = registry.judge_known(&bindings) {
        return pre_evaluation_refusal("handler", refusal.to_string());
    }

    // The depth-5 spawn law: no agent at layer depth 5 may declare a
    // `agent:canSpawn` edge (terminal by absence of the spawn predicate).
    // Judged in the same global, pre-solve phase as handler existence —
    // before `extract_hooks`/`evaluate_hooks` — because it is a structural
    // graph law, not a per-fired-action judgment.
    if let Err(refusal) = spawn_depth_law(&agents) {
        return pre_evaluation_refusal("agent-spawn-depth", refusal.to_string());
    }

    let hooks = extract_hooks(reference.triples())?;

    // The surrender boundary is a runtime law, judged BEFORE any hook
    // evaluation: if the post-state declares a prayer kernel, no
    // god-receives-unbounded clause may be routed toward computation —
    // neither by mutating its refuse-hook nor by a second hook siphoning
    // the surrendered predicate into a ground-action.
    if let Err(refusal) = crate::kernel::enforce_surrender_boundary(event.post(), &hooks) {
        return pre_evaluation_refusal("kernel-boundary", refusal.to_string());
    }
    let verdicts = evaluate_hooks(&hooks, &event, history)?;
    let hook_hash_v = hook_hash(&verdicts)?;

    // Declared refusal: the highest-priority fired refuse-effect hook wins.
    let declared_refusal = verdicts
        .iter()
        .find(|r| r.verdict == HookVerdict::Fired && r.effect == EffectKind::Refuse);
    let (inner, outcome) = if let Some(r) = declared_refusal {
        let hook = hooks.iter().find(|h| h.iri == r.hook_iri);
        let reason = hook
            .and_then(|h| h.reason.clone())
            .unwrap_or_else(|| "declared refusal".to_string());
        (
            Vec::new(),
            FiringOutcome::Refused {
                stage: "declared-refusal".to_string(),
                reason,
            },
        )
    } else {
        // Ground each fired action, then judge delegability SCOPED to the
        // capabilities that action's derived plan uses (grounding is a pure
        // derivation — no side effects — so deriving the plan to discover
        // the used capabilities is lawful before the judgment). A
        // human-only binding on a capability no fired plan touches does
        // not refuse the firing.
        let mut inner = Vec::new();
        let mut violation: Option<Refusal> = None;
        for record in &verdicts {
            if record.verdict == HookVerdict::Fired && record.effect == EffectKind::GroundAction {
                let receipt = ground_fired_action(&event, record)?;
                let used: std::collections::BTreeSet<String> = receipt
                    .plan
                    .steps
                    .iter()
                    .map(|s| s.capability.clone())
                    .collect();
                if let Err(refusal) = registry.judge_delegability(&bindings, &used) {
                    violation = Some(refusal);
                    break;
                }
                inner.push(receipt);
            }
        }
        if let Some(refusal) = violation {
            (
                Vec::new(),
                FiringOutcome::Refused {
                    stage: "delegability".to_string(),
                    reason: refusal.to_string(),
                },
            )
        } else {
            (inner, FiringOutcome::Completed)
        }
    };
    let outcome_hash = json_hash(&outcome, "outcome")?;
    let inner_chains: Vec<&str> = inner.iter().map(|r| r.chain.as_str()).collect();
    let chain = fold_firing_chain(
        &event_hash,
        &admission_hash,
        &handler_hash_v,
        &agent_registry_hash_v,
        &hook_hash_v,
        &history_hash_v,
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
        agents,
        agent_registry_hash: agent_registry_hash_v,
        verdicts,
        hook_hash: hook_hash_v,
        history_hash: history_hash_v,
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
    let stages: [(&str, &str, &str); 8] = [
        ("event_hash", &rederived.event_hash, &receipt.event_hash),
        (
            "admission_hash",
            &rederived.admission_hash,
            &receipt.admission_hash,
        ),
        (
            "handler_hash",
            &rederived.handler_hash,
            &receipt.handler_hash,
        ),
        (
            "agent_registry_hash",
            &rederived.agent_registry_hash,
            &receipt.agent_registry_hash,
        ),
        ("hook_hash", &rederived.hook_hash, &receipt.hook_hash),
        (
            "history_hash",
            &rederived.history_hash,
            &receipt.history_hash,
        ),
        (
            "outcome_hash",
            &rederived.outcome_hash,
            &receipt.outcome_hash,
        ),
        ("chain", &rederived.chain, &receipt.chain),
    ];
    for (name, computed, claimed) in stages {
        if computed != claimed {
            return Err(Refusal::VerificationFailed {
                failed: vec![name.to_string()],
            });
        }
    }
    // Payload bindings: bodies must reproduce the verified hashes.
    if receipt.admission.admission_hash()? != receipt.admission_hash {
        return Err(Refusal::VerificationFailed {
            failed: vec!["admission payload".to_string()],
        });
    }
    if handler_hash(&receipt.bindings) != receipt.handler_hash {
        return Err(Refusal::VerificationFailed {
            failed: vec!["binding payload".to_string()],
        });
    }
    if agent_registry_hash(&receipt.agents) != receipt.agent_registry_hash {
        return Err(Refusal::VerificationFailed {
            failed: vec!["agent registry payload".to_string()],
        });
    }
    if hook_hash(&receipt.verdicts)? != receipt.hook_hash {
        return Err(Refusal::VerificationFailed {
            failed: vec!["verdict payload".to_string()],
        });
    }
    if json_hash(&receipt.outcome, "outcome")? != receipt.outcome_hash {
        return Err(Refusal::VerificationFailed {
            failed: vec!["outcome payload".to_string()],
        });
    }
    if rederived.inner.len() != receipt.inner.len()
        || rederived
            .inner
            .iter()
            .zip(receipt.inner.iter())
            .any(|(a, b)| a.chain != b.chain)
    {
        return Err(Refusal::VerificationFailed {
            failed: vec!["inner chains".to_string()],
        });
    }
    Ok(())
}

/// Render one [`HookFiringReceipt`] as an OCEL 2.0-shaped JSON event.
///
/// Read-only projection of data the receipt already carries — no I/O, no new
/// hash folded into the firing chain. `time` is populated from `reality`'s
/// `time_anchor` only if the caller bound a [`crate::reality::RealityAddressRecord`]
/// for this firing (typically on the fired action's IRI); a firing with no
/// such anchor omits `time` entirely rather than inventing a wall-clock
/// value (PROJ-301's no-invented-time doctrine, honored here).
#[must_use]
pub fn to_ocel_event(
    receipt: &HookFiringReceipt,
    reality: Option<&crate::reality::RealityAddressRecord>,
) -> serde_json::Value {
    let (outcome_str, mut attributes) = match &receipt.outcome {
        FiringOutcome::Completed => ("Completed", serde_json::Map::new()),
        FiringOutcome::Refused { stage, reason } => {
            let mut m = serde_json::Map::new();
            m.insert(
                "stage".to_string(),
                serde_json::Value::String(stage.clone()),
            );
            m.insert(
                "reason".to_string(),
                serde_json::Value::String(reason.clone()),
            );
            ("Refused", m)
        }
    };
    attributes.insert(
        "outcome".to_string(),
        serde_json::Value::String(outcome_str.to_string()),
    );
    attributes.insert(
        "hook_hash".to_string(),
        serde_json::Value::String(receipt.hook_hash.clone()),
    );
    attributes.insert(
        "event_hash".to_string(),
        serde_json::Value::String(receipt.event_hash.clone()),
    );

    let relationships: Vec<serde_json::Value> = receipt
        .bindings
        .iter()
        .map(|b| {
            serde_json::json!({
                "objectId": b.handler,
                "qualifier": "handler-binding",
            })
        })
        .collect();

    let mut event = serde_json::json!({
        "id": receipt.outcome_hash,
        "type": "hook-firing",
        "relationships": relationships,
        "attributes": attributes,
    });
    if let Some(time) = reality.and_then(|r| r.time_anchor()) {
        event
            .as_object_mut()
            .expect("event is always a JSON object")
            .insert(
                "time".to_string(),
                serde_json::Value::String(time.to_string()),
            );
    }
    event
}
