//! Weaver live-check integration tests for the `otel-live` campaign binary.
//!
//! Mirrors the chicago-tdd-tools upstream pattern
//! (`/Users/sac/chicago-tdd-tools/tests/weaver_integration.rs` and
//! `src/core/macros/weaver_test.rs`): every test is `#[ignore]` (requires the
//! live `weaver` binary and the `registry/otel/` registry) and honors
//! `WEAVER_ALLOW_SKIP=1` for intentional bypass. Run explicitly with:
//!
//! `cargo test -p cng --features otel-live --test weaver_live -- --ignored --nocapture`
//!
//! The whole file is gated on the `otel-live` feature so `just cng-test`
//! (default features) skips it entirely; the chicago-tdd-tools dev-dependency
//! is built with `weaver` + `otel` features (see crates/cng/Cargo.toml), which
//! the `weaver_test!` macro and the fixture APIs require.
//!
//! Artifact boundary: no inline Turtle/SPARQL/fixture strings live here — the
//! plan inputs the `otel-live` binary emits telemetry for come from
//! `crates/cng/plans/joseph/*.ttl` on disk (see no_inline_ttl_guard.rs).
#![cfg(feature = "otel-live")]

use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use chicago_tdd_tools::observability::fixtures::ValidationResults;
use chicago_tdd_tools::observability::weaver::types::WeaverLiveCheck;
use chicago_tdd_tools::weaver_test;
use opentelemetry::trace::{Span as _, Tracer as _};

/// Ports offset from the justfile recipe defaults (4317/4320) so a manually
/// running `just otel-weaver-live` campaign and this test never collide.
const TEST_OTLP_GRPC_PORT: u16 = 4327;
const TEST_ADMIN_PORT: u16 = 4330;

/// Upstream skip convention, verbatim semantics from
/// chicago-tdd-tools/tests/weaver_integration.rs.
fn allow_weaver_skip() -> bool {
    matches!(
        std::env::var("WEAVER_ALLOW_SKIP"),
        Ok(value) if matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES")
    )
}

/// Workspace-root `registry/otel` path. Tests run with cwd = crates/cng, so
/// the path is derived from CARGO_MANIFEST_DIR, not the cwd-relative
/// `registry/` that `WeaverLiveCheck::check_registry_available()` probes.
fn registry_otel_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../registry/otel")
}

/// Plan inputs directory the otel-live binary derives its emissions from.
fn joseph_plans_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plans/joseph")
}

/// Poll a TCP port until it accepts connections (readiness signal, no bare
/// sleep-as-signal): max ~15s at 500ms steps, same budget as the justfile
/// `otel-weaver-live-start` recipe.
fn wait_for_port(port: u16) -> bool {
    for _ in 0..30 {
        if TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            Duration::from_millis(250),
        )
        .is_ok()
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    false
}

/// Weaver binary + registry/otel + plan inputs are all present.
///
/// `check_registry_available()` is an associated fn probing a cwd-relative
/// `registry/` dir (crates/cng/registry here), so the workspace-root
/// `registry/otel` is verified directly; the upstream fn is still exercised
/// for the weaver-binary half of the contract via `check_weaver_available()`.
#[test]
#[ignore = "requires live Weaver binary — run manually with WEAVER_ALLOW_SKIP=0"]
fn registry_available() {
    if allow_weaver_skip() {
        eprintln!("⏭️  Skipping Weaver test: WEAVER_ALLOW_SKIP set");
        return;
    }

    // Builder half of the contract: construction with the campaign registry
    // must be expressible (compile-time API check + no panic at runtime).
    let _configured =
        WeaverLiveCheck::new().with_registry(registry_otel_path().display().to_string());

    if let Err(err) = WeaverLiveCheck::check_weaver_available() {
        panic!("weaver binary unavailable: {err}");
    }

    let registry = registry_otel_path();
    assert!(
        registry.is_dir(),
        "registry/otel missing at {} — the registry agent has not landed yet (campaign BLOCKED)",
        registry.display()
    );
    let yaml_count = std::fs::read_dir(&registry)
        .expect("registry/otel must be readable")
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yaml"))
        .count();
    assert!(yaml_count > 0, "registry/otel contains no .yaml files");

    let plans = joseph_plans_dir();
    assert!(plans.is_dir(), "plan inputs missing at {}", plans.display());
}

/// Full roundtrip: start live-check on test-local ports, emit one positive
/// ActivityExecuted batch via the otel-live binary (inputs from
/// crates/cng/plans/joseph on disk), stop, parse the report, assert >0
/// received entities and 0 violations.
#[test]
#[ignore = "requires live Weaver binary — run manually with WEAVER_ALLOW_SKIP=0"]
fn live_check_roundtrip() {
    if allow_weaver_skip() {
        eprintln!("⏭️  Skipping Weaver roundtrip test: WEAVER_ALLOW_SKIP set");
        return;
    }
    if let Err(err) = WeaverLiveCheck::check_weaver_available() {
        panic!("weaver binary unavailable: {err}");
    }
    let registry = registry_otel_path();
    assert!(
        registry.is_dir(),
        "registry/otel missing at {}",
        registry.display()
    );

    // std-only temp report dir (tempfile is not a cng dev-dependency);
    // pid-suffixed to avoid cross-run collisions, cleaned up on next run.
    let report_dir =
        std::env::temp_dir().join(format!("cng-weaver-live-roundtrip-{}", std::process::id()));
    if report_dir.exists() {
        std::fs::remove_dir_all(&report_dir).expect("stale report dir must be removable");
    }
    std::fs::create_dir_all(&report_dir).expect("report dir must be creatable");

    let live = WeaverLiveCheck::new()
        .with_registry(registry.display().to_string())
        .with_otlp_address("127.0.0.1".to_string())
        .with_otlp_port(TEST_OTLP_GRPC_PORT)
        .with_admin_port(TEST_ADMIN_PORT)
        .with_inactivity_timeout(60)
        .with_format("json".to_string())
        // Trailing slash: weaver treats --output as a directory sink.
        .with_output(format!("{}/", report_dir.display()));

    let mut child = live.start().expect("weaver live-check must start");
    assert!(
        wait_for_port(TEST_ADMIN_PORT),
        "weaver admin port {TEST_ADMIN_PORT} never opened within ~15s"
    );

    // CARGO_BIN_EXE_<name> is populated for this package's bins at test build
    // time; the `otel-live` bin has required-features = ["otel-live"], which
    // is satisfied because this whole file only compiles under that feature,
    // so the bin is built alongside this test binary.
    let emitter = env!("CARGO_BIN_EXE_otel-live");
    let status = Command::new(emitter)
        .args([
            "--endpoint",
            &format!("http://127.0.0.1:{TEST_OTLP_GRPC_PORT}"),
            "--mode",
            "positive",
        ])
        .status()
        .expect("otel-live emitter must spawn");
    assert!(
        status.success(),
        "otel-live --mode positive must exit 0 (contract)"
    );

    live.stop().expect("weaver /stop must succeed");
    let _ = child.wait();

    let results = ValidationResults::from_report_dir(&report_dir)
        .expect("live_check.json must be parseable from the report dir");

    let received = results
        .statistics()
        .and_then(|s| s.total_entities)
        .unwrap_or_else(|| results.advices().count() as u64);
    assert!(
        received > 0,
        "no telemetry entities received by weaver (report: {})",
        results.report_path().display()
    );
    assert!(
        !results.has_violations(),
        "{}",
        results.violations_summary()
    );
}

// Fixture-managed smoke, using the upstream macro exactly as documented in
// chicago-tdd-tools/src/core/macros/weaver_test.rs. The fixture owns its own
// weaver lifecycle, temp report dir, and default registry resolution; this
// verifies the macro plumbing (skip handling, finish/validation) is wired for
// this crate independently of the campaign registry.
weaver_test!(weaver_macro_span_smoke, |fixture| {
    let tracer = fixture.tracer("cng-weaver-live", "cng-otel-live-test")?;
    let mut span = tracer.tracer().start("cng-macro-smoke-span");
    span.end();
    Ok(())
});
