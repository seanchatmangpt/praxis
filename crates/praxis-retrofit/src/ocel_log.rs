//! In-process OCEL 2.0 event log for retrofit/ecosystem activity.
//!
//! ## What this module IS
//!
//! - A thin, in-process builder over [`wasm4pm_compat::ocel::OCEL`]'s existing
//!   OCEL 2.0 structural shape (`OCELEvent`, `OCELObject`, `OCELRelationship`,
//!   `OCELAttributeValue`, ...). It reuses those types verbatim; it does not
//!   redefine a parallel schema.
//! - Instrumentation/telemetry for the retrofit/ecosystem domain: discovery,
//!   audit, apply, validate, and admission activity emitted as OCEL events
//!   against `Repository`, `RetrofitPlan`, `ComplianceReport`, and
//!   `EcosystemSystem` objects.
//! - Opt-in via the `PRAXIS_RETROFIT_OCEL_LOG` environment variable
//!   ([`RetrofitOcelLog::enabled`] / [`RetrofitOcelLog::log_path`]); call sites
//!   are expected to check `enabled()` before doing any instrumentation work,
//!   so this module is a true no-op unless the operator opts in.
//!
//! ## What this module is **NOT**
//!
//! - **Not** a receipt or hash path. `emit()` timestamps events with
//!   `chrono::Utc::now()` (real wall-clock time) — this is acceptable *only*
//!   because these are telemetry timestamps, never inputs to a BLAKE3 digest
//!   or a `Refusal`/admission decision. Praxis's no-wall-clock-in-hash-paths
//!   invariant governs receipt/hash code, not instrumentation logs.
//! - **Not** an OCEL discovery/conformance engine. It only appends structurally
//!   valid OCEL 2.0 records; it does not mine process models, flatten logs, or
//!   compute conformance.
//! - **Not** `chicago-tdd-tools`'s `ocel-generation` feature. That feature's
//!   `TestActivity`/`TestObjectType` vocabulary is closed to test-suite
//!   execution events (assertions, fixtures, wave phases) and is unrelated to
//!   the retrofit/ecosystem domain modeled here.

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::Utc;
use wasm4pm_compat::ocel::{
    OCELAttributeValue, OCELEvent, OCELEventAttribute, OCELObject, OCELObjectAttribute,
    OCELRelationship, OCELType, OCELTypeAttribute, OCEL,
};

/// Environment variable that opts in to retrofit OCEL logging.
///
/// Unset (default) means instrumentation is disabled and [`RetrofitOcelLog`]
/// call sites should skip all instrumentation work.
pub const PRAXIS_RETROFIT_OCEL_LOG_ENV: &str = "PRAXIS_RETROFIT_OCEL_LOG";

/// Retrofit/ecosystem object types recognized by this log.
pub mod object_types {
    /// A repository under retrofit (attrs: `github_url`, `source`, `retrofit_phase`).
    pub const REPOSITORY: &str = "Repository";
    /// A retrofit plan for a repository (attrs: `phase`, `risk_level`).
    pub const RETROFIT_PLAN: &str = "RetrofitPlan";
    /// A compliance report (attrs: `status`, `pass_rate`).
    pub const COMPLIANCE_REPORT: &str = "ComplianceReport";
    /// The overall ecosystem system object (no required attributes).
    pub const ECOSYSTEM_SYSTEM: &str = "EcosystemSystem";
}

/// Retrofit/ecosystem event types recognized by this log.
pub mod event_types {
    /// A repository or capability was discovered.
    pub const DISCOVER: &str = "Discover";
    /// A repository was audited for compliance.
    pub const AUDIT: &str = "Audit";
    /// A retrofit plan/action was applied.
    pub const APPLY: &str = "Apply";
    /// A retrofit result was validated.
    pub const VALIDATE: &str = "Validate";
    /// An observation or artifact was admitted (or refused).
    pub const ADMIT: &str = "Admit";
}

/// A builder over an in-memory [`OCEL`] log for retrofit/ecosystem activity.
///
/// Construct a private instance with [`RetrofitOcelLog::new`] for tests, or
/// use the process-wide singleton via [`RetrofitOcelLog::global`].
pub struct RetrofitOcelLog {
    inner: Mutex<OCEL>,
}

impl RetrofitOcelLog {
    /// Create a fresh, empty log. Prefer this in tests over the global
    /// singleton so test runs do not share mutable state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(OCEL {
                event_types: Vec::new(),
                object_types: Vec::new(),
                events: Vec::new(),
                objects: Vec::new(),
            }),
        }
    }

    /// The process-wide retrofit OCEL log singleton.
    #[must_use]
    pub fn global() -> &'static RetrofitOcelLog {
        static GLOBAL: OnceLock<RetrofitOcelLog> = OnceLock::new();
        GLOBAL.get_or_init(RetrofitOcelLog::new)
    }

    /// Whether retrofit OCEL logging is enabled via `PRAXIS_RETROFIT_OCEL_LOG`.
    ///
    /// Call sites should check this before doing any instrumentation work so
    /// this module is a true no-op by default.
    #[must_use]
    pub fn enabled() -> bool {
        Self::log_path().is_some()
    }

    /// The configured log output path, or `None` if `PRAXIS_RETROFIT_OCEL_LOG`
    /// is unset (the default, disabled state).
    #[must_use]
    pub fn log_path() -> Option<PathBuf> {
        std::env::var_os(PRAXIS_RETROFIT_OCEL_LOG_ENV).map(PathBuf::from)
    }

    /// Idempotently upsert an object into the log: updates the object's
    /// attributes in place if `id` already exists, else appends a new
    /// [`OCELObject`]. Also registers `object_type` in the log's object-type
    /// registry if it is not already present.
    pub fn ensure_object(&self, id: &str, object_type: &str, attrs: &[(&str, OCELAttributeValue)]) {
        let now = Utc::now().into();
        let attributes: Vec<OCELObjectAttribute> = attrs
            .iter()
            .map(|(name, value)| OCELObjectAttribute {
                name: (*name).to_string(),
                value: value.clone(),
                time: now,
            })
            .collect();

        let mut log = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if !log.object_types.iter().any(|t| t.name == object_type) {
            let type_attributes: Vec<OCELTypeAttribute> = attrs
                .iter()
                .map(|(name, value)| OCELTypeAttribute {
                    name: (*name).to_string(),
                    value_type: attribute_value_type_name(value).to_string(),
                })
                .collect();
            log.object_types.push(OCELType {
                name: object_type.to_string(),
                attributes: type_attributes,
            });
        }

        if let Some(existing) = log.objects.iter_mut().find(|o| o.id == id) {
            existing.object_type = object_type.to_string();
            existing.attributes = attributes;
        } else {
            log.objects.push(OCELObject {
                id: id.to_string(),
                object_type: object_type.to_string(),
                attributes,
                relationships: Vec::new(),
            });
        }
    }

    /// Emit an OCEL event of `event_type`, related to the given
    /// `(object_id, qualifier)` pairs, carrying `attrs` as event attributes.
    ///
    /// Assigns a fresh UUID v4 event id and timestamps the event with the
    /// current wall-clock time (`chrono::Utc::now()`). This is acceptable
    /// here because this is a telemetry/instrumentation path, not a
    /// receipt/hash path — see the module-level doc comment.
    ///
    /// Also registers `event_type` in the log's event-type registry if it is
    /// not already present.
    pub fn emit(
        &self,
        event_type: &str,
        relationships: &[(&str, &str)],
        attrs: &[(&str, OCELAttributeValue)],
    ) {
        let event_id = uuid::Uuid::new_v4().to_string();
        let time = Utc::now().into();
        let attributes: Vec<OCELEventAttribute> = attrs
            .iter()
            .map(|(name, value)| OCELEventAttribute {
                name: (*name).to_string(),
                value: value.clone(),
            })
            .collect();
        let event_relationships: Vec<OCELRelationship> = relationships
            .iter()
            .map(|(object_id, qualifier)| OCELRelationship {
                object_id: (*object_id).to_string(),
                qualifier: (*qualifier).to_string(),
            })
            .collect();

        let mut log = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if !log.event_types.iter().any(|t| t.name == event_type) {
            let type_attributes: Vec<OCELTypeAttribute> = attrs
                .iter()
                .map(|(name, value)| OCELTypeAttribute {
                    name: (*name).to_string(),
                    value_type: attribute_value_type_name(value).to_string(),
                })
                .collect();
            log.event_types.push(OCELType {
                name: event_type.to_string(),
                attributes: type_attributes,
            });
        }

        log.events.push(OCELEvent {
            id: event_id,
            event_type: event_type.to_string(),
            time,
            attributes,
            relationships: event_relationships,
        });
    }

    /// Serialize the current log state as pretty-printed OCEL 2.0 JSON to
    /// `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if `path` cannot be created or if serialization
    /// fails.
    pub fn write_json(&self, path: &Path) -> anyhow::Result<()> {
        let log = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &*log)?;
        Ok(())
    }
}

impl Default for RetrofitOcelLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Best-effort OCEL type-attribute `value_type` label for an
/// [`OCELAttributeValue`], used only when auto-registering a new
/// object/event type from the first attribute set seen for it.
fn attribute_value_type_name(value: &OCELAttributeValue) -> &'static str {
    match value {
        OCELAttributeValue::Integer(_) => "integer",
        OCELAttributeValue::Float(_) => "float",
        OCELAttributeValue::Boolean(_) => "boolean",
        OCELAttributeValue::Time(_) => "time",
        OCELAttributeValue::String(_) => "string",
        OCELAttributeValue::Null => "string",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm4pm_compat::ocel::OCEL as WireOcel;

    #[test]
    fn ensure_object_appends_new_object_and_registers_type() {
        let log = RetrofitOcelLog::new();
        log.ensure_object(
            "repo:affidavit",
            object_types::REPOSITORY,
            &[
                (
                    "github_url",
                    OCELAttributeValue::String(
                        "https://github.com/seanchatmangpt/affidavit".into(),
                    ),
                ),
                (
                    "source",
                    OCELAttributeValue::String("ecosystem-scan".into()),
                ),
                ("retrofit_phase", OCELAttributeValue::Integer(2)),
            ],
        );

        let inner = log.inner.lock().unwrap();
        assert_eq!(inner.objects.len(), 1);
        assert_eq!(inner.objects[0].id, "repo:affidavit");
        assert_eq!(inner.objects[0].object_type, object_types::REPOSITORY);
        assert_eq!(inner.objects[0].attributes.len(), 3);
        assert!(inner
            .object_types
            .iter()
            .any(|t| t.name == object_types::REPOSITORY));
    }

    #[test]
    fn ensure_object_is_idempotent_on_repeat_id() {
        let log = RetrofitOcelLog::new();
        log.ensure_object(
            "repo:affidavit",
            object_types::REPOSITORY,
            &[("retrofit_phase", OCELAttributeValue::Integer(1))],
        );
        log.ensure_object(
            "repo:affidavit",
            object_types::REPOSITORY,
            &[("retrofit_phase", OCELAttributeValue::Integer(2))],
        );

        let inner = log.inner.lock().unwrap();
        assert_eq!(inner.objects.len(), 1);
        assert_eq!(
            inner.objects[0].attributes[0].value,
            OCELAttributeValue::Integer(2)
        );
        assert_eq!(
            inner
                .object_types
                .iter()
                .filter(|t| t.name == object_types::REPOSITORY)
                .count(),
            1
        );
    }

    #[test]
    fn emit_pushes_event_with_relationships_and_registers_event_type() {
        let log = RetrofitOcelLog::new();
        log.ensure_object("repo:affidavit", object_types::REPOSITORY, &[]);
        log.emit(
            event_types::AUDIT,
            &[("repo:affidavit", "audited")],
            &[("status", OCELAttributeValue::String("pass".into()))],
        );

        let inner = log.inner.lock().unwrap();
        assert_eq!(inner.events.len(), 1);
        assert_eq!(inner.events[0].event_type, event_types::AUDIT);
        assert_eq!(inner.events[0].relationships.len(), 1);
        assert_eq!(inner.events[0].relationships[0].object_id, "repo:affidavit");
        assert_eq!(inner.events[0].relationships[0].qualifier, "audited");
        assert!(!inner.events[0].id.is_empty());
        assert!(inner
            .event_types
            .iter()
            .any(|t| t.name == event_types::AUDIT));
    }

    #[test]
    fn write_json_round_trips_through_real_ocel_wire_type() {
        let log = RetrofitOcelLog::new();
        log.ensure_object(
            "repo:affidavit",
            object_types::REPOSITORY,
            &[("retrofit_phase", OCELAttributeValue::Integer(3))],
        );
        log.ensure_object("plan:affidavit-1", object_types::RETROFIT_PLAN, &[]);
        log.emit(
            event_types::APPLY,
            &[
                ("repo:affidavit", "target"),
                ("plan:affidavit-1", "executed"),
            ],
            &[("phase", OCELAttributeValue::String("apply".into()))],
        );

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("retrofit_ocel.json");
        log.write_json(&path).expect("write_json succeeds");

        let raw = std::fs::read_to_string(&path).expect("read back log file");
        let wire: WireOcel = serde_json::from_str(&raw).expect("valid OCEL 2.0 JSON");

        assert_eq!(wire.events.len(), 1);
        assert_eq!(wire.objects.len(), 2);
        assert_eq!(wire.events[0].event_type, event_types::APPLY);
        assert_eq!(wire.events[0].relationships.len(), 2);
        assert!(wire.events[0]
            .relationships
            .iter()
            .any(|r| r.object_id == "repo:affidavit" && r.qualifier == "target"));
        assert!(wire
            .objects
            .iter()
            .any(|o| o.id == "plan:affidavit-1" && o.object_type == object_types::RETROFIT_PLAN));
    }

    #[test]
    fn enabled_and_log_path_reflect_env_var() {
        // Isolate from other tests / the environment by using a unique-ish
        // sentinel var name via the real env, saving and restoring state.
        let previous = std::env::var_os(PRAXIS_RETROFIT_OCEL_LOG_ENV);
        std::env::remove_var(PRAXIS_RETROFIT_OCEL_LOG_ENV);
        assert!(!RetrofitOcelLog::enabled());
        assert_eq!(RetrofitOcelLog::log_path(), None);

        std::env::set_var(PRAXIS_RETROFIT_OCEL_LOG_ENV, "/tmp/retrofit-ocel.json");
        assert!(RetrofitOcelLog::enabled());
        assert_eq!(
            RetrofitOcelLog::log_path(),
            Some(PathBuf::from("/tmp/retrofit-ocel.json"))
        );

        match previous {
            Some(v) => std::env::set_var(PRAXIS_RETROFIT_OCEL_LOG_ENV, v),
            None => std::env::remove_var(PRAXIS_RETROFIT_OCEL_LOG_ENV),
        }
    }

    #[test]
    fn global_returns_same_singleton_instance() {
        let a = RetrofitOcelLog::global() as *const RetrofitOcelLog;
        let b = RetrofitOcelLog::global() as *const RetrofitOcelLog;
        assert_eq!(a, b);
    }
}
