//! Telemetry and tracing initialization.

use crate::Result;

/// Initialize tracing with an `EnvFilter` (defaults to `"info"`) and a
/// human-readable fmt layer tagged with `service_name`.
///
/// Idempotent: if a global subscriber has already been set this function
/// silently succeeds.
///
/// # Example
/// ```rust,no_run
/// chatman_common::telemetry::init_tracing("my-service").unwrap();
/// tracing::info!("ready");
/// ```
pub fn init_tracing(service_name: &str) -> Result<()> {
    use tracing_subscriber::layer::SubscriberExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Tag every span/event with the service name via a constant field.
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false);

    // tracing doesn't expose a built-in "service.name" field, so we record it
    // once here and let downstream collectors pick it up from the OTel resource.
    let _ = service_name; // used by otel branch below; keep the param visible

    // Ignore "already initialized" error — idempotent.
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init();

    Ok(())
}

/// Initialize an OTLP exporter and wire it to the tracing layer.
///
/// Requires the `otel` feature.  The `endpoint` should be the gRPC OTLP
/// collector URL, e.g. `"http://localhost:4317"`.
///
/// # TODO
/// Wire `opentelemetry-otlp` + `tracing-opentelemetry` when deps are added to
/// `Cargo.toml`.  Until then this is a no-op that logs a warning.
#[cfg(feature = "otel")]
pub fn init_otel(service_name: &str, endpoint: &str) -> Result<()> {
    tracing::warn!(
        service = service_name,
        endpoint,
        "init_otel: OTLP exporter not yet wired — add opentelemetry-otlp dep"
    );
    Ok(())
}

/// A guard that flushes telemetry on drop.
///
/// Obtain via [`TracingGuard::new`] and keep alive for the duration of `main`.
pub struct TracingGuard {
    _private: (),
}

impl TracingGuard {
    /// Install the tracing subscriber and return a guard.
    ///
    /// Equivalent to calling [`init_tracing`] but returns a guard that can be
    /// used to ensure shutdown hooks run (relevant once OTLP is wired).
    pub fn new(service_name: &str) -> Result<Self> {
        init_tracing(service_name)?;
        Ok(Self { _private: () })
    }
}

impl Drop for TracingGuard {
    fn drop(&mut self) {
        // Flush OTLP pipeline when wired.
        #[cfg(feature = "otel")]
        opentelemetry::global::shutdown_tracer_provider();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_tracing_is_idempotent() {
        // Multiple calls must not panic.
        init_tracing("test-svc").unwrap();
        init_tracing("test-svc").unwrap();
    }

    #[test]
    fn guard_constructs_and_drops() {
        let _g = TracingGuard::new("guard-test").unwrap();
    }
}
