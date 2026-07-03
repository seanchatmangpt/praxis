//! Action grounding — the connector the archaeology found missing.
//!
//! A fired hook whose effect is `ground-action` names a `hook:action` IRI:
//! a `wf:Workflow` fragment that ALREADY LIVES in the admitted graph. This
//! module restricts the graph to that fragment and runs it through the
//! existing chain (extract IR → lower → Solver8 → derived topology/geometry
//! → supervised execution) — no new solver, no synthesized actions at
//! firing time. Actions are declared before deviations, never invented
//! during them.
//!
//! It also projects a fired hook into a [`CapabilityTaskSpec`] — the plain
//! data bridge toward `bcinr_pddl::route_capability_plan` (consumed by the
//! praxis root crate; praxis-synthesis takes no bcinr dependency).
//!
//! Lineage: `docs/ggen_rdf_to_pddl_sketch.rs` (the "DO NOT IMPLEMENT"
//! sketch, now implemented for the workflow-fragment half);
//! wasm4pm-compat `pddl.rs` grounding discipline (explosion guard = the
//! existing solver bounds).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::graph::{execute_from_triples, Object, Triple, WorkflowReceipt, WF_NS};
use crate::hooks::{EffectKind, HookVerdict, HookVerdictRecord, KnowledgeHook};
use crate::quarantine::AdmittedEvent;
use crate::Refusal;

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// The plain-data bridge toward a temporal-PDDL router: desired effects as
/// (kind, argument) pairs, projected from the fired hook. The praxis root
/// crate maps these onto `bcinr_pddl::CapabilityTask`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityTaskSpec {
    /// The fired hook's IRI (provenance).
    pub hook_iri: String,
    /// The action fragment IRI.
    pub action_iri: String,
    /// Desired effects: the action workflow's goal atoms as
    /// (predicate, first-arg-or-empty) pairs, sorted.
    pub desired_effects: Vec<(String, String)>,
}

/// Restrict the admitted graph to one action fragment: the closure of the
/// action's `wf:Workflow` node under `wf:` object-IRI references, PLUS the
/// graph's `wf:Capability` and `wf:Constraint` nodes and their closures
/// (capabilities are the shared action vocabulary), MINUS every OTHER
/// `wf:Workflow` typing — so the fragment satisfies the exactly-one-
/// workflow law of the extractor.
fn restrict_to_fragment(triples: &[Triple], action_iri: &str) -> Result<Vec<Triple>, Refusal> {
    let workflow_class = format!("{WF_NS}Workflow");
    let is_wf_typed = |s: &str| {
        triples.iter().any(|t| {
            t.s == s && t.p == RDF_TYPE && matches!(&t.o, Object::Iri(c) if *c == workflow_class)
        })
    };
    if !is_wf_typed(action_iri) {
        return Err(Refusal::WorkflowIllFormed {
            subject: action_iri.to_string(),
            detail: "hook:action does not reference a node declared 'a wf:Workflow'".to_string(),
        });
    }

    // Seeds: the action node + every capability/constraint node.
    let cap_class = format!("{WF_NS}Capability");
    let con_class = format!("{WF_NS}Constraint");
    let mut seeds: BTreeSet<&str> = BTreeSet::new();
    seeds.insert(action_iri);
    for t in triples {
        if t.p == RDF_TYPE {
            if let Object::Iri(c) = &t.o {
                if *c == cap_class || *c == con_class {
                    seeds.insert(&t.s);
                }
            }
        }
    }
    // Transitive closure over wf: object-IRI references.
    let mut included: BTreeSet<&str> = seeds.clone();
    loop {
        let mut grew = false;
        for t in triples {
            if included.contains(t.s.as_str()) && t.p.starts_with(WF_NS) {
                if let Object::Iri(o) = &t.o {
                    if included.insert(o.as_str()) {
                        grew = true;
                    }
                }
            }
        }
        if !grew {
            break;
        }
    }
    // Keep triples of included subjects, dropping other Workflow typings
    // (their fragments are not this action).
    let fragment: Vec<Triple> = triples
        .iter()
        .filter(|t| included.contains(t.s.as_str()))
        .filter(|t| {
            !(t.p == RDF_TYPE
                && t.s != action_iri
                && matches!(&t.o, Object::Iri(c) if *c == workflow_class))
        })
        .cloned()
        .collect();
    Ok(fragment)
}

/// Ground and execute the action of one FIRED `ground-action` verdict:
/// restrict the admitted post-graph to the declared fragment and run it
/// through the existing derived chain. The returned [`WorkflowReceipt`] is
/// the standard inner v1 receipt — grounding adds no new receipt shape.
pub fn ground_fired_action(
    event: &AdmittedEvent,
    record: &HookVerdictRecord,
) -> Result<WorkflowReceipt, Refusal> {
    if record.verdict != HookVerdict::Fired {
        return Err(Refusal::InvalidInput {
            detail: format!("hook '{}' did not fire; nothing to ground", record.hook_name),
        });
    }
    if record.effect != EffectKind::GroundAction {
        return Err(Refusal::InvalidInput {
            detail: format!("hook '{}' effect is not ground-action", record.hook_name),
        });
    }
    let action_iri = record.action_iri.as_deref().ok_or_else(|| Refusal::InvalidInput {
        detail: format!("hook '{}' has no action IRI", record.hook_name),
    })?;
    let fragment = restrict_to_fragment(&event.post, action_iri)?;
    execute_from_triples(&fragment)
}

/// Project a fired hook into the PDDL-router bridge spec: the action
/// fragment's goal atoms become desired effects.
pub fn capability_task_spec(
    event: &AdmittedEvent,
    hook: &KnowledgeHook,
) -> Result<CapabilityTaskSpec, Refusal> {
    let action_iri = hook.action.as_deref().ok_or_else(|| Refusal::InvalidInput {
        detail: format!("hook '{}' has no action IRI", hook.name),
    })?;
    let goal_pred = format!("{WF_NS}goal");
    let predicate_pred = format!("{WF_NS}predicate");
    let arg0_pred = format!("{WF_NS}arg0");
    let mut desired_effects = Vec::new();
    for t in &event.post {
        if t.s == action_iri && t.p == goal_pred {
            if let Object::Iri(goal_atom) = &t.o {
                let field = |pred: &str| {
                    event.post.iter().find_map(|u| {
                        (u.s == *goal_atom && u.p == pred)
                            .then(|| match &u.o {
                                Object::Str(s) => s.clone(),
                                Object::Iri(i) => i.clone(),
                                Object::Int(v) => v.to_string(),
                            })
                    })
                };
                let predicate = field(&predicate_pred).ok_or_else(|| Refusal::WorkflowIllFormed {
                    subject: goal_atom.clone(),
                    detail: "goal atom missing wf:predicate".to_string(),
                })?;
                desired_effects.push((predicate, field(&arg0_pred).unwrap_or_default()));
            }
        }
    }
    if desired_effects.is_empty() {
        return Err(Refusal::WorkflowIllFormed {
            subject: action_iri.to_string(),
            detail: "action fragment declares no wf:goal atoms".to_string(),
        });
    }
    desired_effects.sort_unstable();
    Ok(CapabilityTaskSpec {
        hook_iri: hook.iri.clone(),
        action_iri: action_iri.to_string(),
        desired_effects,
    })
}
