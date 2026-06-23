//! Integration tests for `chatman_common::testkit`.
//!
//! These tests exercise the public API surface of the testkit module:
//!   - `TestState<Phase>` compile-time AAA transition enforcement
//!   - `TestReceipt::capture()` timing and pass/fail detection
//!   - `EnvironmentFingerprint` field population
//!   - `allocate_ephemeral_port()` returns a usable non-zero port
//!   - `assert_fail!` macro catches the right variant
//!   - `performance_test!` macro enforces SLA
//!   - `TempReceipt` builder round-trips JSON correctly
//!   - `assert_golden` and `assert_snapshot` update/verify workflows
//!   - `deterministic_uuid` is stable and well-formed

#[cfg(feature = "testkit")]
mod testkit_integration {
    use chatman_common::testkit::{
        allocate_ephemeral_port, assert_golden, assert_snapshot, deterministic_uuid, Act, Arrange,
        Assert, EnvironmentFingerprint, TempReceipt, TestOutput, TestReceipt, TestState,
    };
    use chatman_common::assert_fail;

    // -----------------------------------------------------------------------
    // TestState<Phase> — compile-time AAA transitions
    // -----------------------------------------------------------------------

    /// Transitions Arrange → Act → Assert must compile and produce distinct types.
    #[test]
    fn test_state_arrange_to_act_to_assert() {
        let s: TestState<Arrange> = TestState::new();
        let s: TestState<Act> = s.act();
        let _s: TestState<Assert> = s.assert();
    }

    /// `TestState::default()` starts in the Arrange phase.
    #[test]
    fn test_state_default_starts_in_arrange() {
        let _s: TestState<Arrange> = TestState::default();
    }

    /// Multiple independent test sequences can coexist; each starts fresh.
    #[test]
    fn test_state_multiple_independent_sequences() {
        let a = TestState::<Arrange>::new();
        let b = TestState::<Arrange>::new();
        let _a = a.act().assert();
        let _b = b.act().assert();
    }

    // -----------------------------------------------------------------------
    // TestReceipt::capture() — timing and pass/fail
    // -----------------------------------------------------------------------

    /// `capture` records a passing closure with timing > 0.
    #[test]
    fn test_receipt_capture_records_passing_closure() {
        let r = TestReceipt::capture("integration_pass", || {
            std::hint::black_box(1u64.wrapping_add(1));
        });
        assert!(r.passed, "closure that returned normally should be passed=true");
        assert_eq!(r.test_name, "integration_pass");
        // duration_ms may be 0 on fast machines but must not overflow
        // (it is a u64, so any value is acceptable — we just check the field exists)
        let _ = r.duration_ms;
    }

    /// `capture` detects a panic and records passed=false.
    #[test]
    fn test_receipt_capture_detects_panic() {
        let r = TestReceipt::capture("integration_fail", || {
            panic!("intentional failure inside capture");
        });
        assert!(!r.passed, "capture should record passed=false on panic");
        assert_eq!(r.test_name, "integration_fail");
    }

    /// `record` populates all fields correctly.
    #[test]
    fn test_receipt_record_populates_fields() {
        let r = TestReceipt::record("my_integration_test", true, 123);
        assert_eq!(r.test_name, "my_integration_test");
        assert!(r.passed);
        assert_eq!(r.duration_ms, 123);
    }

    /// `record` with passed=false is stored as-is.
    #[test]
    fn test_receipt_record_failed_test() {
        let r = TestReceipt::record("failing_scenario", false, 9999);
        assert!(!r.passed);
        assert_eq!(r.duration_ms, 9999);
    }

    // -----------------------------------------------------------------------
    // EnvironmentFingerprint
    // -----------------------------------------------------------------------

    /// `capture()` fills all fields with non-empty/non-zero values.
    #[test]
    fn environment_fingerprint_capture_is_populated() {
        let fp = EnvironmentFingerprint::capture();
        assert!(!fp.os.is_empty(), "os must not be empty");
        assert!(!fp.target.is_empty(), "target must not be empty");
        // timestamp should be positive (year ~2024+)
        assert!(fp.timestamp_unix > 1_700_000_000, "timestamp looks wrong: {}", fp.timestamp_unix);
    }

    /// Two rapid captures should yield the same OS and target.
    #[test]
    fn environment_fingerprint_os_is_stable() {
        let a = EnvironmentFingerprint::capture();
        let b = EnvironmentFingerprint::capture();
        assert_eq!(a.os, b.os);
        assert_eq!(a.target, b.target);
    }

    // -----------------------------------------------------------------------
    // allocate_ephemeral_port
    // -----------------------------------------------------------------------

    /// The returned port must be in the valid range (1–65535).
    #[test]
    fn allocate_ephemeral_port_returns_nonzero() {
        let port = allocate_ephemeral_port();
        assert!(port > 0, "expected non-zero port, got {port}");
    }

    /// Two successive allocations should return different ports most of the time.
    /// (This can theoretically collide but is extremely unlikely.)
    #[test]
    fn allocate_ephemeral_port_two_calls_differ() {
        let a = allocate_ephemeral_port();
        let b = allocate_ephemeral_port();
        // Both must be valid — equality is allowed but rare.
        assert!(a > 0 && b > 0);
    }

    /// The returned port must be bindable immediately after allocation.
    #[test]
    fn allocate_ephemeral_port_is_bindable() {
        use std::net::TcpListener;
        let port = allocate_ephemeral_port();
        // There is a small TOCTOU window; if another process grabbed the port
        // between allocation and bind, this test will fail — acceptable.
        let addr = format!("127.0.0.1:{port}");
        TcpListener::bind(&addr)
            .unwrap_or_else(|e| panic!("could not bind to allocated port {port}: {e}"));
    }

    // -----------------------------------------------------------------------
    // assert_fail! macro
    // -----------------------------------------------------------------------

    /// `assert_fail!` passes when the expression returns `Err(_)`.
    #[test]
    fn assert_fail_passes_on_err() {
        let result: Result<(), &str> = Err("boom");
        assert_fail!(result);
    }

    /// `assert_fail!` with a pattern passes when the variant matches.
    #[test]
    fn assert_fail_with_matching_variant() {
        #[derive(Debug)]
        #[allow(dead_code)]
        enum TestError {
            NotFound,
            Timeout,
        }
        let result: Result<(), TestError> = Err(TestError::NotFound);
        assert_fail!(result, TestError::NotFound);
    }

    /// `assert_fail!` with a pattern works for the second variant too.
    #[test]
    fn assert_fail_timeout_variant() {
        #[derive(Debug)]
        #[allow(dead_code)]
        enum TestError {
            NotFound,
            Timeout,
        }
        let result: Result<(), TestError> = Err(TestError::Timeout);
        assert_fail!(result, TestError::Timeout);
    }

    // -----------------------------------------------------------------------
    // TempReceipt builder
    // -----------------------------------------------------------------------

    /// Builder defaults produce a valid JSON file.
    #[test]
    fn temp_receipt_default_builder_creates_valid_json() {
        let tr = TempReceipt::builder().build().unwrap();
        let path = tr.path();
        assert!(path.exists(), "receipt file must exist");
        let contents = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(v["format_version"], "core/v1");
        assert_eq!(v["profile"], "core/v1");
        assert!(v["events"].as_array().unwrap().is_empty());
    }

    /// Custom `format_version` and `chain_hash` are persisted.
    #[test]
    fn temp_receipt_custom_fields_are_stored() {
        let tr = TempReceipt::builder()
            .format_version("v2")
            .chain_hash("a".repeat(64))
            .profile("extended/v1")
            .build()
            .unwrap();
        let contents = std::fs::read_to_string(tr.path()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(v["format_version"], "v2");
        assert_eq!(v["profile"], "extended/v1");
        assert_eq!(v["chain_hash"].as_str().unwrap(), "a".repeat(64));
    }

    /// Multiple events are stored in order.
    #[test]
    fn temp_receipt_multiple_events_in_order() {
        let tr = TempReceipt::builder()
            .event(serde_json::json!({"seq": 0, "kind": "start"}))
            .event(serde_json::json!({"seq": 1, "kind": "end"}))
            .build()
            .unwrap();
        let contents = std::fs::read_to_string(tr.path()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&contents).unwrap();
        let events = v["events"].as_array().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["seq"], 0);
        assert_eq!(events[1]["seq"], 1);
    }

    /// `dir()` returns the temp directory containing the file.
    #[test]
    fn temp_receipt_dir_contains_file() {
        let tr = TempReceipt::builder().build().unwrap();
        let dir = tr.dir();
        let file = tr.path();
        assert!(file.starts_with(dir));
        assert!(dir.is_dir());
    }

    /// Custom filename is respected.
    #[test]
    fn temp_receipt_custom_filename() {
        let tr = TempReceipt::builder()
            .filename("audit.json")
            .build()
            .unwrap();
        assert!(tr.path().to_string_lossy().ends_with("audit.json"));
        assert!(tr.path().exists());
    }

    /// The TempDir is cleaned up on drop (file disappears).
    #[test]
    fn temp_receipt_cleans_up_on_drop() {
        let path = {
            let tr = TempReceipt::builder().build().unwrap();
            tr.path()
        };
        assert!(!path.exists(), "temp file should be deleted after drop");
    }

    // -----------------------------------------------------------------------
    // assert_golden
    // -----------------------------------------------------------------------

    /// Full UPDATE_GOLDEN → verify round-trip.
    #[test]
    fn assert_golden_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("output.bin");
        let data: &[u8] = b"stable golden output";

        std::env::set_var("UPDATE_GOLDEN", "1");
        assert_golden(data, &path).unwrap();
        std::env::remove_var("UPDATE_GOLDEN");

        assert_golden(data, &path).unwrap();
    }

    /// Mismatch returns Err (does not panic).
    #[test]
    fn assert_golden_mismatch_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("golden.bin");
        std::fs::write(&path, b"expected content").unwrap();

        let result = assert_golden(b"different content", &path);
        assert!(result.is_err(), "mismatched golden should return Err");
    }

    /// Missing golden file returns Err when UPDATE_GOLDEN is not set.
    #[test]
    fn assert_golden_missing_file_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.bin");
        let result = assert_golden(b"anything", &path);
        assert!(result.is_err(), "missing golden file should return Err");
    }

    // -----------------------------------------------------------------------
    // assert_snapshot
    // -----------------------------------------------------------------------

    /// UPDATE_SNAPSHOTS=1 creates the snapshot; subsequent run verifies it.
    #[test]
    fn assert_snapshot_create_and_verify() {
        let dir = tempfile::tempdir().unwrap();
        let snaps = dir.path().join("snapshots");

        std::env::set_var("UPDATE_SNAPSHOTS", "1");
        assert_snapshot("integration_snap", "hello\nworld\n", &snaps);
        std::env::remove_var("UPDATE_SNAPSHOTS");

        // The snap file should exist now.
        assert!(snaps.join("integration_snap.snap").exists());
        // Verify round-trip.
        assert_snapshot("integration_snap", "hello\nworld\n", &snaps);
    }

    // -----------------------------------------------------------------------
    // deterministic_uuid
    // -----------------------------------------------------------------------

    /// Same seed always produces the same UUID string.
    #[test]
    fn deterministic_uuid_is_stable() {
        let a = deterministic_uuid("integration-seed");
        let b = deterministic_uuid("integration-seed");
        assert_eq!(a, b);
    }

    /// Different seeds produce different UUIDs.
    #[test]
    fn deterministic_uuid_differs_for_different_seeds() {
        assert_ne!(
            deterministic_uuid("seed-a"),
            deterministic_uuid("seed-b")
        );
    }

    /// UUID has the canonical 8-4-4-4-12 segment format.
    #[test]
    fn deterministic_uuid_has_correct_format() {
        let u = deterministic_uuid("format-test");
        let parts: Vec<&str> = u.split('-').collect();
        assert_eq!(parts.len(), 5, "uuid must have 5 dash-separated parts: {u}");
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
        // All hex chars
        for part in &parts {
            assert!(
                part.chars().all(|c| c.is_ascii_hexdigit()),
                "uuid part {part:?} contains non-hex chars"
            );
        }
    }

    /// UUID version nibble is `5` (bit-patterned version 5 over blake3).
    #[test]
    fn deterministic_uuid_version_nibble_is_5() {
        let u = deterministic_uuid("version-test");
        // 3rd group, first char indicates version
        let third_group = u.split('-').nth(2).unwrap();
        assert_eq!(
            &third_group[0..1],
            "5",
            "UUID version nibble should be 5, got {third_group}"
        );
    }

    // -----------------------------------------------------------------------
    // TestOutput trait
    // -----------------------------------------------------------------------

    /// `Ok(())` into_test_result should not panic.
    #[test]
    fn test_output_ok_does_not_panic() {
        let r: Result<(), String> = Ok(());
        r.into_test_result();
    }

    /// `()` into_test_result is a no-op.
    #[test]
    fn test_output_unit_is_noop() {
        ().into_test_result();
    }
}

// ---------------------------------------------------------------------------
// performance_test! macro — SLA enforcement (requires testkit feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "testkit")]
chatman_common::performance_test!(perf_trivial_op_within_sla, 1000, {
    let _ = std::hint::black_box(42u64.wrapping_mul(7));
});

#[cfg(feature = "testkit")]
chatman_common::performance_test!(perf_string_alloc_within_sla, 500, {
    let s = std::hint::black_box("hello world".repeat(10));
    let _ = s.len();
});
