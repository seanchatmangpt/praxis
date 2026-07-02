//! OCEL 2.0 event conversion for law objects (feature-gated).

use serde::{Deserialize, Serialize};

use crate::{law::LawObject, lifecycle::Receipted};

/// Reference to an OCEL object (with type and optional qualifier).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelObjectRef {
    /// Object identifier.
    pub id: String,
    /// Object type.
    pub r#type: String,
    /// Optional qualifier (e.g., "ROLE" or "ACTIVITY").
    pub qualifier: Option<String>,
}

/// OCEL 2.0 event representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEvent {
    /// Event identifier.
    pub id: String,
    /// Event type (e.g., "lifecycle:judgment", "lifecycle:receipt").
    pub r#type: String,
    /// ISO 8601 timestamp.
    pub time: String,
    /// Event attributes as JSON.
    pub attributes: serde_json::Value,
    /// Related objects.
    pub relationships: Vec<OcelObjectRef>,
}

/// Trait for converting a type into an OCEL event.
pub trait ToOcelEvent {
    /// Convert self into an OCEL event representation.
    fn to_ocel_event(&self) -> OcelEvent;
}

impl<Payload: Serialize, Law: std::fmt::Debug> ToOcelEvent for LawObject<Payload, Receipted, Law> {
    fn to_ocel_event(&self) -> OcelEvent {
        let payload_hash =
            blake3::hash(serde_json::to_vec(&self.payload).unwrap_or_default().as_slice());
        let payload_hex = format!(
            "{}",
            payload_hash.as_bytes().iter().map(|b| format!("{:02x}", b)).collect::<String>()
        );
        let event_id = format!("receipt:{}", payload_hex);

        let chain_hash_hex = self
            .chain_hash
            .map(|h| format!("{}", h.iter().map(|b| format!("{:02x}", b)).collect::<String>()));

        let attributes = serde_json::json!({
            "obligations": self.obligations,
            "andon": self.andon,
            "chain_hash": chain_hash_hex,
        });

        OcelEvent {
            id: event_id,
            r#type: "lifecycle:receipt".to_string(),
            time: "2026-07-01T00:00:00Z".to_string(),
            attributes,
            relationships: vec![],
        }
    }
}
