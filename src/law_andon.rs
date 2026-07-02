//! Optional second admission gate: an lsp-max `andon` ring layered on top of
//! [`praxis_core::DefaultLaw`]'s obligation judging.
//!
//! Feature-gated behind the (lightweight) `andon` Cargo feature — just the
//! `lsp-max` dependency, none of the heavier `lsp` feature's tree-sitter /
//! tokio / dashmap surface, since lsp-max's `andon` module isn't internally
//! feature-gated.
//!
//! This is opt-in *per payload*: callers set `"andon_ring": true` in the
//! judge/admit JSON input (see [`crate::ops`]). When off (the default), a
//! payload built and compiled with this feature enabled behaves exactly as
//! it did before this module existed.
//!
//! # What actually gates
//!
//! `lsp_max::andon::analysis::AnalysisPipeline::evaluate_registry` only
//! flags *structurally incomplete* invariants (e.g. an `AndonInvariant`
//! missing a `true_probe`). The patterns seeded here
//! ([`lsp_max::andon::patterns::build_receipt_required`] and
//! [`lsp_max::andon::patterns::build_non_empty_check_set`]) are fully
//! populated, so that check alone would never block anything — it's a
//! registry-health check, not a runtime condition. The actual runtime gate
//! is [`AndonRing::evaluate`], which probes the payload directly against
//! each seeded invariant's stated condition (does it carry a `"receipt"`
//! field? a non-empty `"checks_run"` array?) and synthesizes an
//! [`AndonEvent`] when a probe fails, before handing everything to
//! [`AdmissionGate::evaluate`].

use lsp_max::andon::{
    analysis::AnalysisPipeline,
    andon::{AdmissionGate, AdmissionStatus, AndonEvent},
    core::InvariantRegistry,
    patterns,
};
use praxis_core::RefusalScenario;
use serde_json::Value;

/// The lsp-max andon second gate ring: an [`InvariantRegistry`] seeded with
/// a fixed set of praxis invariants, evaluated against a judge/admit
/// payload.
pub struct AndonRing {
    registry: InvariantRegistry,
}

impl AndonRing {
    /// The standard ring: seeded with `build_receipt_required()` (a receipt
    /// must be present) and `build_non_empty_check_set()` (at least one
    /// check must have run).
    pub fn standard() -> Self {
        let mut registry = InvariantRegistry::new();
        registry.register(patterns::build_receipt_required());
        registry.register(patterns::build_non_empty_check_set());
        Self { registry }
    }

    /// Evaluate the ring against a payload value.
    ///
    /// Returns the resulting [`AdmissionStatus`] plus every [`AndonEvent`]
    /// that fired — both the (normally empty) registry-health events from
    /// `evaluate_registry`, and the payload-specific probe events described
    /// in the module docs.
    pub fn evaluate(&self, payload: &Value) -> (AdmissionStatus, Vec<AndonEvent>) {
        let mut events = AnalysisPipeline::evaluate_registry(&self.registry);

        let has_receipt =
            payload.get("receipt").map(|v| !v.is_null()).unwrap_or(false);
        if !has_receipt {
            events.push(missing_probe_event(
                "ReceiptRequired",
                "receipt_required_probe_failed",
                "payload has no non-null \"receipt\" field: test output is not a receipt",
            ));
        }

        let has_checks = payload
            .get("checks_run")
            .and_then(Value::as_array)
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        if !has_checks {
            events.push(missing_probe_event(
                "NonEmptyCheckSet",
                "non_empty_check_set_probe_failed",
                "payload has no non-empty \"checks_run\" array: empty checks_run is ANDON",
            ));
        }

        let mut gate = AdmissionGate::new();
        gate.evaluate(&events);
        (gate.status, events)
    }
}

/// Build a blocking, non-admission-allowed [`AndonEvent`] for a failed
/// runtime probe against invariant `invariant_id`.
fn missing_probe_event(invariant_id: &str, code: &str, message: &str) -> AndonEvent {
    AndonEvent {
        id: format!("andon-{invariant_id}-{code}"),
        severity: lsp_max::andon::core::Severity::Stop,
        code: code.to_string(),
        title: format!("{invariant_id} probe failed"),
        message: message.to_string(),
        invariant_id: Some(invariant_id.to_string()),
        observed_state: Some("probe_failed".to_string()),
        expected_state: Some("probe_passed".to_string()),
        blocking: true,
        requires_ack: true,
        admission_allowed: false,
        next_lawful_step: None,
        required_command: None,
        evidence_uri: None,
        virtual_doc_uri: None,
        receipt_required: false,
    }
}

/// Map every blocking, non-admission-allowed event to a
/// [`RefusalScenario::AndonInvariantViolated`]. Events that are informational
/// or that still allow admission are not refusals.
pub fn ring_refusals(events: &[AndonEvent]) -> Vec<RefusalScenario> {
    events
        .iter()
        .filter(|e| e.blocking && !e.admission_allowed)
        .map(|e| RefusalScenario::AndonInvariantViolated {
            invariant_id: e.invariant_id.clone().unwrap_or_else(|| e.id.clone()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn standard_ring_blocks_on_missing_receipt_and_checks() {
        let ring = AndonRing::standard();
        let (status, events) = ring.evaluate(&json!({"value": 1}));
        // `missing_probe_event` uses `Severity::Stop`, which `AdmissionGate`
        // maps to `Stopped` (not `Refused`, which is reserved for
        // `Severity::Refuse` events, or `Blocked`, the fallback for
        // non-`Stop`/`Refuse` blocking severities).
        assert!(matches!(
            status,
            AdmissionStatus::Stopped | AdmissionStatus::Refused | AdmissionStatus::Blocked
        ));
        let refusals = ring_refusals(&events);
        assert!(!refusals.is_empty());
        assert!(refusals.iter().any(|r| matches!(
            r,
            RefusalScenario::AndonInvariantViolated { invariant_id } if invariant_id == "ReceiptRequired"
        )));
    }

    #[test]
    fn standard_ring_admits_when_receipt_and_checks_present() {
        let ring = AndonRing::standard();
        let (status, events) =
            ring.evaluate(&json!({"receipt": "blake3:abc", "checks_run": ["format"]}));
        assert!(matches!(status, AdmissionStatus::Candidate));
        assert!(ring_refusals(&events).is_empty());
    }
}
