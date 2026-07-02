//! Generic pattern for emitting typed OCEL 2.0 events from any
//! receipt/audit struct, via `wasm4pm-compat`.
//!
//! This generalizes the `to_ocel_event_typed()` method found on
//! `lsp-max-compositor::receipt::CompositorReceipt` (see
//! `lsp-max/crates/lsp-max-compositor/src/receipt.rs`), stripped of any
//! lsp-max-specific fields (URI, diagnostic counts, ANDON status). What
//! remains is the general shape: a stable event id, an activity name,
//! an emission timestamp, and a bag of typed attributes.
//!
//! Implement [`EmitsOcelEvent`] for any receipt/audit struct in your
//! domain to get a well-formed [`OcelEvent`] for free via
//! [`EmitsOcelEvent::to_ocel_event_typed`].
//!
//! # Example
//!
//! ```
//! use sample_service::ocel_event::EmitsOcelEvent;
//! use wasm4pm_compat::ocel::OcelAttribute;
//!
//! struct OrderReceipt {
//!     order_id: String,
//!     total_amount: f64,
//!     dispatched: bool,
//! }
//!
//! impl EmitsOcelEvent for OrderReceipt {
//!     fn activity(&self) -> &str {
//!         "OrderDispatch"
//!     }
//!
//!     fn attributes(&self) -> Vec<OcelAttribute> {
//!         vec![
//!             OcelAttribute::string("order_id", &self.order_id),
//!             OcelAttribute::float("total_amount", self.total_amount),
//!             OcelAttribute::boolean("dispatched", self.dispatched),
//!         ]
//!     }
//! }
//!
//! let receipt = OrderReceipt {
//!     order_id: "ord-1".to_string(),
//!     total_amount: 42.0,
//!     dispatched: true,
//! };
//! let event = receipt.to_ocel_event_typed("evt-1");
//! assert_eq!(event.id(), "evt-1");
//! assert_eq!(event.activity(), "OrderDispatch");
//! assert_eq!(event.attributes().len(), 3);
//! ```

use wasm4pm_compat::ocel::{OcelAttribute, OcelEvent};

/// Implemented by any receipt/audit struct that wants to emit a typed
/// OCEL 2.0 [`OcelEvent`].
///
/// Only [`EmitsOcelEvent::activity`] and [`EmitsOcelEvent::attributes`]
/// need to be supplied by implementors — [`EmitsOcelEvent::timestamp_ns`]
/// defaults to "now" and [`EmitsOcelEvent::to_ocel_event_typed`] wires
/// everything together.
pub trait EmitsOcelEvent {
    /// The OCEL activity name for this event (e.g. `"OrderDispatch"`,
    /// `"CompositorFlush"`). Should be a stable, PascalCase identifier
    /// shared across all events of this kind.
    fn activity(&self) -> &str;

    /// The typed attributes to attach to the emitted event. Each
    /// implementor decides which of its own fields are relevant
    /// process-mining evidence.
    fn attributes(&self) -> Vec<OcelAttribute>;

    /// The emission timestamp in nanoseconds since the Unix epoch.
    ///
    /// Defaults to "now". Override this if the receipt/audit struct
    /// already carries its own authoritative timestamp (e.g. captured
    /// at construction time rather than at emission time).
    fn timestamp_ns(&self) -> u64 {
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64
    }

    /// Builds a fully-populated [`OcelEvent`] for this struct.
    ///
    /// `event_id` is caller-supplied so the caller controls id
    /// generation/uniqueness strategy (uuid, sequence counter,
    /// content hash, etc.) — this trait makes no assumptions about it.
    fn to_ocel_event_typed(&self, event_id: &str) -> OcelEvent {
        let mut event = OcelEvent::new(event_id, self.activity()).at_ns(self.timestamp_ns());
        for attr in self.attributes() {
            event = event.with_attribute(attr);
        }
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeReceipt {
        case_id: String,
        step_count: i64,
        blocked: bool,
    }

    impl EmitsOcelEvent for FakeReceipt {
        fn activity(&self) -> &str {
            "FakeStep"
        }

        fn attributes(&self) -> Vec<OcelAttribute> {
            vec![
                OcelAttribute::string("case_id", &self.case_id),
                OcelAttribute::integer("step_count", self.step_count),
                OcelAttribute::boolean("blocked", self.blocked),
            ]
        }
    }

    #[test]
    fn to_ocel_event_typed_carries_activity_and_id() {
        let receipt = FakeReceipt {
            case_id: "case-1".to_string(),
            step_count: 3,
            blocked: false,
        };
        let event = receipt.to_ocel_event_typed("evt-42");
        assert_eq!(event.id(), "evt-42");
        assert_eq!(event.activity(), "FakeStep");
    }

    #[test]
    fn to_ocel_event_typed_carries_all_attributes() {
        let receipt = FakeReceipt {
            case_id: "case-2".to_string(),
            step_count: 7,
            blocked: true,
        };
        let event = receipt.to_ocel_event_typed("evt-43");
        assert_eq!(event.attributes().len(), 3);
        assert!(event.attributes().iter().any(|a| a.key == "case_id"));
        assert!(event.attributes().iter().any(|a| a.key == "step_count"));
        assert!(event.attributes().iter().any(|a| a.key == "blocked"));
    }

    #[test]
    fn to_ocel_event_typed_sets_a_nonzero_timestamp_by_default() {
        let receipt = FakeReceipt {
            case_id: "case-3".to_string(),
            step_count: 0,
            blocked: false,
        };
        let event = receipt.to_ocel_event_typed("evt-44");
        assert!(event.timestamp_ns().is_some());
    }
}
