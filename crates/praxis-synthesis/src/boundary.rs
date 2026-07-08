//! Boundary request artifacts, graph delta projection, and EmitDelta execution.
//! Consumed by external adapters to bridge hook outcomes across boundaries.

use crate::graph::{Object, Triple};
use crate::quarantine::{
    Admission, AdmittedEvent, MeaningSource, Origin, Reference, RiceQuarantine,
};
use crate::Refusal;
use chatman_common::provenance::content_address;
use serde::{Deserialize, Serialize};

/// Boundary request artifact containing identities, freshness, and idempotency materials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryRequest {
    /// Epoch of the state when the hook fired.
    pub state_epoch: u64,
    /// Hash of the base state graph.
    pub base_graph_hash: String,
    /// Hook IRI.
    pub hook_iri: String,
    /// Hook name.
    pub hook_name: String,
    /// Event hash of the triggering delta.
    pub event_hash: String,
    /// TTL hash of the triggering delta.
    pub delta_ttl_hash: String,
    /// Freshness token (e.g. committing window history hash).
    pub freshness_token: String,
    /// Deterministic idempotency key generated from hook, payload, and epoch.
    pub idempotency_key: String,
}

impl BoundaryRequest {
    /// Create a new boundary request artifact.
    #[must_use]
    pub fn new(
        reference: &Reference,
        hook_iri: &str,
        hook_name: &str,
        event_hash: &str,
        delta_ttl_hash: &str,
        history_hash: &str,
    ) -> Self {
        let freshness_token = history_hash.to_string();

        // Generate a deterministic idempotency key.
        let idempotency_material = format!(
            "praxis:boundary-idempotency:v1\n\
             epoch={}\n\
             base_graph_hash={}\n\
             hook_iri={}\n\
             hook_name={}\n\
             event_hash={}\n\
             delta_ttl_hash={}\n\
             freshness_token={}",
            reference.epoch(),
            reference.graph_hash(),
            hook_iri,
            hook_name,
            event_hash,
            delta_ttl_hash,
            freshness_token
        );
        let idempotency_key = content_address(idempotency_material.as_bytes());

        Self {
            state_epoch: reference.epoch(),
            base_graph_hash: reference.graph_hash().to_string(),
            hook_iri: hook_iri.to_string(),
            hook_name: hook_name.to_string(),
            event_hash: event_hash.to_string(),
            delta_ttl_hash: delta_ttl_hash.to_string(),
            freshness_token,
            idempotency_key,
        }
    }
}

/// Retrieve the delta template (adds/removes TTL) declared by a delta action node.
#[must_use]
pub fn get_delta_template(triples: &[Triple], action_iri: &str) -> (String, String) {
    let adds_pred = "http://seanchatmangpt.github.io/praxis/hook#adds_ttl";
    let removes_pred = "http://seanchatmangpt.github.io/praxis/hook#removes_ttl";
    let mut adds_ttl = String::new();
    let mut removes_ttl = String::new();
    for t in triples {
        if t.s == action_iri {
            if t.p == adds_pred {
                if let Object::Str(s) = &t.o {
                    adds_ttl = s.clone();
                }
            } else if t.p == removes_pred {
                if let Object::Str(s) = &t.o {
                    removes_ttl = s.clone();
                }
            }
        }
    }
    (adds_ttl, removes_ttl)
}

/// Project variables derived from a trigger result into a delta template (asserts/retracts).
#[must_use]
pub fn project_delta_template(
    adds_template: &str,
    removes_template: &str,
    vars: &[String],
) -> (String, String) {
    let mut adds = adds_template.to_string();
    let mut removes = removes_template.to_string();
    for (i, val) in vars.iter().enumerate() {
        let placeholder = format!("?{}", i);
        let formatted = if val.starts_with("http://") || val.starts_with("https://") {
            format!("<{}>", val)
        } else {
            val.clone()
        };
        adds = adds.replace(&placeholder, &formatted);
        removes = removes.replace(&placeholder, &formatted);
    }
    (adds, removes)
}

/// Process and execute EmitDelta by feeding it back to the quarantine gate and admitting it.
pub fn execute_emit_delta(
    reference: &Reference,
    adds_ttl: &str,
    removes_ttl: &str,
) -> Result<AdmittedEvent, Refusal> {
    let source = MeaningSource {
        origin: Origin::Bridge,
        adds_ttl: adds_ttl.to_string(),
        removes_ttl: removes_ttl.to_string(),
    };
    // Re-enter the quarantine gate.
    let delta = RiceQuarantine::inspect(&source)?;
    // Admit the delta.
    let admitted = Admission::admit(reference, &delta)?;
    Ok(admitted)
}
