//! Integration tests for the layered `PraxisConfig` admission pipeline.
//!
//! Tests that touch the process cwd and/or environment serialize on
//! `ISOLATION_LOCK` and run inside a scratch `HOME`/cwd so they never read a
//! developer's real `~/.praxis/config.toml` or `./praxis.toml`.

use std::{env, sync::Mutex};

use my_conforming_project::config::{
    gate_verdict_from_validation, load_config_with_gate, PraxisConfig,
};
use praxis_retrofit::preventive_gate::{
    Severity, ValidateCategory, ValidationResult, ValidationStatus,
};
use star_toml::nouns::{EvidenceGate, OracleGateVerdict, OracleVerdict};

static ISOLATION_LOCK: Mutex<()> = Mutex::new(());

/// Run `f` with cwd and `HOME` pointed at a fresh, empty temp directory, then
/// restore both. Guarded by a mutex since cwd/env are process-global.
fn in_isolated_scratch<F: FnOnce(&std::path::Path)>(f: F) {
    let _guard = ISOLATION_LOCK.lock().unwrap();
    let original_cwd = env::current_dir().unwrap();
    let original_home = env::var("HOME").ok();

    let tmp = tempfile::tempdir().unwrap();
    env::set_var("HOME", tmp.path());
    env::set_current_dir(tmp.path()).unwrap();

    f(tmp.path());

    env::set_current_dir(&original_cwd).unwrap();
    match original_home {
        Some(h) => env::set_var("HOME", h),
        None => env::remove_var("HOME"),
    }
}

#[test]
fn defaults_alone_admit_successfully() {
    in_isolated_scratch(|_dir| {
        let admitted =
            load_config_with_gate(EvidenceGate::new()).expect("bare defaults must admit");
        assert_eq!(admitted.value().planner.attention_capacity, 7);
        assert_eq!(admitted.value().law.default_policy, "default");
        assert_eq!(admitted.value().law.signing_key_env, "PRAXIS_SIGNING_KEY");
        assert_eq!(admitted.value().receipts.dir, "receipts");
        assert_eq!(admitted.value().mfg.ontology_dir, "ontology");
        assert_eq!(admitted.value().mfg.template_dir, "templates");
        assert!(admitted.value().gate.preventive_checks);
        assert!(!admitted.value().gate.strict);
    });
}

#[test]
fn unknown_toml_field_is_rejected() {
    in_isolated_scratch(|dir| {
        std::fs::write(dir.join("praxis.toml"), "[law]\nbogus_key = 1\n").unwrap();
        let err = load_config_with_gate(EvidenceGate::new())
            .expect_err("unknown field must be rejected");
        let msg = err.to_string();
        assert!(msg.contains("unknown_field"), "expected unknown_field in: {msg}");
        assert!(msg.contains("law.bogus_key") || msg.contains("bogus_key"), "expected field path in: {msg}");
    });
}

#[test]
fn witness_hash_is_deterministic_for_identical_input() {
    in_isolated_scratch(|_dir| {
        let a = load_config_with_gate(EvidenceGate::new()).unwrap();
        let b = load_config_with_gate(EvidenceGate::new()).unwrap();
        assert_eq!(a.witness().hash(), b.witness().hash());
        assert!(!a.witness().hash().is_empty());
    });
}

#[test]
fn witness_hash_changes_when_value_changes() {
    in_isolated_scratch(|dir| {
        let baseline = load_config_with_gate(EvidenceGate::new()).unwrap();

        std::fs::write(dir.join("praxis.toml"), "[planner]\nattention_capacity = 12\n").unwrap();
        let changed = load_config_with_gate(EvidenceGate::new()).unwrap();

        assert_ne!(baseline.witness().hash(), changed.witness().hash());
        assert_eq!(changed.value().planner.attention_capacity, 12);
    });
}

#[test]
fn env_var_override_wins_over_file_and_defaults() {
    in_isolated_scratch(|dir| {
        std::fs::write(dir.join("praxis.toml"), "[planner]\nattention_capacity = 12\n").unwrap();
        // Nested keys use `__` as the path separator (single `_` stays inside a
        // segment name), per star-toml's env-override convention. The prefix is
        // PRAXIS_CONFIG__ so operational vars (PRAXIS_SIGNING_KEY et al.) never
        // enter the strict config admission surface — see src/config.rs docs.
        env::set_var("PRAXIS_CONFIG__PLANNER__ATTENTION_CAPACITY", "9");

        let admitted = load_config_with_gate(EvidenceGate::new());

        env::remove_var("PRAXIS_CONFIG__PLANNER__ATTENTION_CAPACITY");

        let admitted = admitted.expect("env override must still admit");
        assert_eq!(admitted.value().planner.attention_capacity, 9);
    });
}

#[test]
fn operational_praxis_env_vars_do_not_break_admission() {
    in_isolated_scratch(|_dir| {
        // These are documented *operational* variables (signing key source,
        // walkthrough conveniences) — not config fields. Config admission
        // must ignore them rather than reject them as unknown fields; a bare
        // `PRAXIS_` env prefix once made setting the signing key the config
        // itself names at `law.signing_key_env` a fatal admission error.
        env::set_var("PRAXIS_SIGNING_KEY", "00".repeat(32));
        env::set_var("PRAXIS_BIN", "/tmp/nonexistent-bin");
        env::set_var("PRAXIS_ONTOLOGY", "/tmp/nonexistent.ttl");

        let admitted = load_config_with_gate(EvidenceGate::new());

        env::remove_var("PRAXIS_SIGNING_KEY");
        env::remove_var("PRAXIS_BIN");
        env::remove_var("PRAXIS_ONTOLOGY");

        let admitted = admitted.expect("operational env vars must not block admission");
        assert_eq!(admitted.value().law.signing_key_env, "PRAXIS_SIGNING_KEY");
    });
}

#[test]
fn out_of_range_attention_capacity_is_rejected() {
    in_isolated_scratch(|dir| {
        std::fs::write(dir.join("praxis.toml"), "[planner]\nattention_capacity = 0\n").unwrap();
        let err = load_config_with_gate(EvidenceGate::new())
            .expect_err("attention_capacity = 0 must fail validation");
        assert!(err.to_string().contains("attention_capacity"));

        std::fs::write(dir.join("praxis.toml"), "[planner]\nattention_capacity = 65\n").unwrap();
        let err = load_config_with_gate(EvidenceGate::new())
            .expect_err("attention_capacity = 65 must fail validation");
        assert!(err.to_string().contains("attention_capacity"));
    });
}

#[test]
fn fail_severity_gate_verdict_blocks_admission() {
    in_isolated_scratch(|_dir| {
        let mut gate = EvidenceGate::new();
        gate.push(OracleGateVerdict {
            verdict: OracleVerdict::Fail,
            reason: "synthetic critical failure for test".to_string(),
            evidence_refs: vec![],
            source: "test".to_string(),
        });

        let err = load_config_with_gate(gate).expect_err("Fail verdict must block admission");
        assert!(err.to_string().contains("oracle_gate_fail"));
    });
}

#[test]
fn non_fail_gate_verdicts_do_not_block_admission() {
    in_isolated_scratch(|_dir| {
        let mut gate = EvidenceGate::new();
        gate.push(OracleGateVerdict {
            verdict: OracleVerdict::Warn,
            reason: "synthetic warning".to_string(),
            evidence_refs: vec![],
            source: "test".to_string(),
        });
        gate.push(OracleGateVerdict {
            verdict: OracleVerdict::Suggest,
            reason: "synthetic suggestion".to_string(),
            evidence_refs: vec![],
            source: "test".to_string(),
        });

        load_config_with_gate(gate).expect("Warn/Suggest verdicts must not block admission");
    });
}

/// Gate-verdict adapter mapping table:
/// `(ValidationStatus, Severity) -> OracleVerdict`, per the plan's spec —
/// only `(Fail, Critical)` produces a blocking `Fail`.
#[test]
fn gate_verdict_adapter_mapping_table() {
    let case = |status: ValidationStatus, severity: Severity| {
        gate_verdict_from_validation(&ValidationResult {
            status,
            category: ValidateCategory::Versioning,
            check_name: "check".to_string(),
            message: "msg".to_string(),
            remediation: None,
            severity,
        })
        .verdict
    };

    assert_eq!(case(ValidationStatus::Pass, Severity::Info), OracleVerdict::Admit);
    assert_eq!(case(ValidationStatus::Pass, Severity::Warning), OracleVerdict::Admit);
    assert_eq!(case(ValidationStatus::Pass, Severity::Error), OracleVerdict::Admit);
    assert_eq!(case(ValidationStatus::Pass, Severity::Critical), OracleVerdict::Admit);

    assert_eq!(case(ValidationStatus::Warn, Severity::Info), OracleVerdict::Suggest);
    assert_eq!(case(ValidationStatus::Warn, Severity::Warning), OracleVerdict::Warn);
    assert_eq!(case(ValidationStatus::Warn, Severity::Error), OracleVerdict::Warn);
    assert_eq!(case(ValidationStatus::Warn, Severity::Critical), OracleVerdict::Warn);

    assert_eq!(case(ValidationStatus::Fail, Severity::Info), OracleVerdict::Warn);
    assert_eq!(case(ValidationStatus::Fail, Severity::Warning), OracleVerdict::Warn);
    assert_eq!(case(ValidationStatus::Fail, Severity::Error), OracleVerdict::Warn);
    assert_eq!(case(ValidationStatus::Fail, Severity::Critical), OracleVerdict::Fail);
}

#[test]
fn praxis_config_default_matches_baked_in_defaults() {
    let default_value = PraxisConfig::default();
    in_isolated_scratch(|_dir| {
        let admitted = load_config_with_gate(EvidenceGate::new()).unwrap();
        assert_eq!(admitted.value(), &default_value);
    });
}
