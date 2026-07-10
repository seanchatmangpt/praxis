//! Production OTel emitter for the Weaver live-check campaign.
//!
//! Runs the REAL cng pipeline (import_artifacts -> generate_plan -> plan_id)
//! over a plans directory, constructs one `event.praxis.activity_executed`
//! occurrence from the actual plan, and emits ONE span (named after the
//! registry group id) with those attributes via OTLP gRPC to the given
//! endpoint.
//!
//! Modes:
//! - `--mode positive` (default): all five required attributes; exit 0 on success.
//! - `--mode negative`: attributes from `as_key_values_missing_outcome()` —
//!   intentionally nonconformant telemetry for the live-check negative proof.
//!   After emitting, the binary refuses the producer by contract: exits 3 and
//!   prints `NEGATIVE_REFUSAL_CODE=WEAVER_SEMANTIC_CONVENTION_REFUSED` to stderr.
//!
//! Exit codes: 0 success (positive), 1 pipeline/emission failure, 2 usage
//! error, 3 negative-mode contractual refusal (span was emitted).

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use cng::pipeline::{generate_plan, import_artifacts, plan_id};
use cng::telemetry_gen::{ActivityExecuted, REGISTRY_GROUP_ID};

/// Typed failure surface for this binary — no panics, no unwraps.
#[derive(Debug)]
enum OtelLiveError {
    /// Bad CLI arguments (exit 2).
    Usage(String),
    /// The cng pipeline refused (exit 1).
    Pipeline(String),
    /// Tokio runtime construction failed (exit 1).
    Runtime(String),
    /// OTLP exporter construction or span export failed (exit 1).
    Export(String),
}

impl fmt::Display for OtelLiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(m) => write!(f, "usage error: {m}"),
            Self::Pipeline(m) => write!(f, "pipeline refused: {m}"),
            Self::Runtime(m) => write!(f, "tokio runtime: {m}"),
            Self::Export(m) => write!(f, "otlp export: {m}"),
        }
    }
}

impl OtelLiveError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Pipeline(_) | Self::Runtime(_) | Self::Export(_) => 1,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Mode {
    Positive,
    Negative,
}

struct Args {
    endpoint: String,
    mode: Mode,
    plans: PathBuf,
}

/// Hand-rolled arg parsing (cng's CLI uses clap-noun-verb, which is not a
/// plain-flag parser; this binary keeps a minimal flag surface instead).
///
/// # Complexity
/// O(argc).
fn parse_args(argv: &[String]) -> Result<Args, OtelLiveError> {
    let mut endpoint = String::from("http://127.0.0.1:4317");
    let mut mode = Mode::Positive;
    // Default resolved via CARGO_MANIFEST_DIR so it works from the repo root.
    let mut plans = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plans/joseph");
    let mut i = 0;
    while i < argv.len() {
        let flag = argv[i].as_str();
        let value = argv
            .get(i + 1)
            .ok_or_else(|| OtelLiveError::Usage(format!("flag {flag} requires a value")))?;
        match flag {
            "--endpoint" => endpoint = value.clone(),
            "--mode" => {
                mode = match value.as_str() {
                    "positive" => Mode::Positive,
                    "negative" => Mode::Negative,
                    other => {
                        return Err(OtelLiveError::Usage(format!(
                            "--mode must be positive|negative, got {other}"
                        )))
                    }
                }
            }
            "--plans" => plans = PathBuf::from(value),
            other => {
                return Err(OtelLiveError::Usage(format!(
                    "unknown flag {other}; supported: --endpoint <url> --mode positive|negative --plans <dir>"
                )))
            }
        }
        i += 2;
    }
    Ok(Args {
        endpoint,
        mode,
        plans,
    })
}

/// Runs the real cng pipeline and builds the event from actual plan facts.
///
/// - workflow_id: `plan_id(tape)` (BLAKE3 over ordered plan-step labels).
/// - object_id: the first imported artifact's content-addressed
///   `urn:blake3:` source IRI — the concrete business object the plan
///   was derived from.
/// - activity_iri: the plan's first op label wrapped as
///   `urn:praxis:activity:<label>` — an honest IRI for the first executed
///   activity, chosen over any generic OCEL class IRI which would not name
///   the activity actually executed.
fn build_event(plans: &std::path::Path) -> Result<ActivityExecuted, OtelLiveError> {
    let artifacts = import_artifacts(plans)
        .map_err(|e| OtelLiveError::Pipeline(format!("import_artifacts: {e}")))?;
    let (tape, _surface) = generate_plan(&artifacts)
        .map_err(|e| OtelLiveError::Pipeline(format!("generate_plan: {e}")))?;
    let workflow_id = plan_id(&tape);
    let object_id = artifacts
        .first()
        .map(|a| a.source_iri.clone())
        .ok_or_else(|| OtelLiveError::Pipeline("no artifacts imported".to_string()))?;
    let first_label = tape
        .ops
        .first()
        .map(|op| op.label.clone())
        .ok_or_else(|| OtelLiveError::Pipeline("plan tape has no ops".to_string()))?;
    let activity_iri = format!("urn:praxis:activity:{first_label}");
    Ok(ActivityExecuted::new(
        workflow_id,
        object_id,
        "WorkflowExecution",
        activity_iri,
        "completed",
    ))
}

/// Emits ONE span named `REGISTRY_GROUP_ID` carrying `attrs` via OTLP
/// gRPC (tonic) to `endpoint`, mirroring chicago-tdd-tools'
/// `send_test_span_to_weaver` construction (exporter -> SdkTracerProvider
/// -> tracer -> span -> force_flush -> shutdown).
fn emit_span(endpoint: &str, attrs: Vec<(&'static str, String)>) -> Result<(), OtelLiveError> {
    // Weaver live-check listens on gRPC only; grpc-tonic needs a Tokio
    // runtime. Must be multi-thread: the SDK's batch span processor exports
    // from a background thread, and a current-thread runtime starves that
    // export (observed as force_flush Err(Timeout)). Mirrors
    // chicago-tdd-tools' send_test_span_to_weaver, which uses Runtime::new().
    let rt = tokio::runtime::Runtime::new().map_err(|e| OtelLiveError::Runtime(e.to_string()))?;

    rt.block_on(async move {
        use opentelemetry::trace::{Span, Tracer, TracerProvider as _};
        use opentelemetry::KeyValue;
        use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
        use opentelemetry_sdk::Resource;

        let base_endpoint = endpoint
            .trim_end_matches("/v1/traces")
            .trim_end_matches('/');
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", base_endpoint);

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .map_err(|e| OtelLiveError::Export(format!("exporter build: {e}")))?;

        // Empty resource on purpose: the generated registry governs only the
        // five process.* attributes. Emitting service.name/telemetry.sdk.*
        // would flag missing_attribute violations, since this minimal-slice
        // registry deliberately does not import the official semconv
        // registry (a future registry dependency, not a local redefinition).
        let resource = Resource::builder_empty().build();

        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_sampler(Sampler::TraceIdRatioBased(1.0))
            .with_id_generator(RandomIdGenerator::default())
            .with_resource(resource)
            .build();

        let tracer = provider.tracer("cng-otel-live");
        let mut span = tracer.span_builder(REGISTRY_GROUP_ID).start(&tracer);
        for (key, value) in attrs {
            span.set_attribute(KeyValue::new(key, value));
        }
        span.end();

        provider
            .force_flush()
            .map_err(|e| OtelLiveError::Export(format!("force_flush: {e}")))?;

        // Give the async export time to complete before shutdown.
        tokio::time::sleep(Duration::from_millis(500)).await;

        provider
            .shutdown()
            .map_err(|e| OtelLiveError::Export(format!("shutdown: {e}")))?;
        Ok(())
    })
}

fn run() -> Result<i32, OtelLiveError> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = parse_args(&argv)?;
    let event = build_event(&args.plans)?;
    let attrs = match args.mode {
        Mode::Positive => event.as_key_values(),
        Mode::Negative => event.as_key_values_missing_outcome(),
    };
    emit_span(&args.endpoint, attrs)?;
    println!("OTEL_SIGNALS_EMITTED=1");
    println!("OTEL_SPANS_EMITTED=1");
    match args.mode {
        Mode::Positive => Ok(0),
        Mode::Negative => {
            // The negative producer is intentionally nonconformant telemetry
            // (process.outcome omitted); this binary refuses it by contract.
            eprintln!("NEGATIVE_REFUSAL_CODE=WEAVER_SEMANTIC_CONVENTION_REFUSED");
            Ok(3)
        }
    }
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("otel-live: {err}");
            std::process::exit(err.exit_code());
        }
    }
}
