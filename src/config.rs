//! `PraxisConfig` — layered, admitted configuration for the praxis root binary.
//!
//! Follows the star-toml `TrustedLoader` idiom already established in
//! [`praxis_retrofit::repo_registry::RepositoryRegistry`]: every field carries
//! `#[serde(default)]` so partial TOML layers admit cleanly under
//! `load_admitted`'s strict unknown-field rejection (CE-8), invariants are
//! checked via [`Validate`], and lifecycle normalization (trimming strings,
//! expanding `~/`) happens via [`ConfigLifecycle`] before validation runs.
//!
//! # Layering
//!
//! [`load_config`] composes, in increasing precedence:
//! 1. baked-in defaults ([`DEFAULTS_TOML`])
//! 2. `~/.praxis/config.toml` (if it exists)
//! 3. `./praxis.toml` (if it exists)
//! 4. `PRAXIS_CONFIG__*` environment variable overrides (e.g.
//!    `PRAXIS_CONFIG__PLANNER__ATTENTION_CAPACITY=9` → `planner.attention_capacity`)
//!
//! The env prefix is deliberately `PRAXIS_CONFIG__`, not `PRAXIS_`: the
//! project documents *operational* `PRAXIS_*` variables that are not config
//! fields (`PRAXIS_SIGNING_KEY` — the signing lane's key source named by
//! `law.signing_key_env` — plus the walkthrough's `PRAXIS_BIN` /
//! `PRAXIS_ONTOLOGY` / `PRAXIS_WALKTHROUGH_KEEP`). Under a bare `PRAXIS_`
//! prefix the strict unknown-field admission would (correctly, per its own
//! rules) reject a process that set the very signing key the config points
//! at — config admission must not veto the mechanisms it names.
//!
//! and feeds a real [`star_toml::EvidenceGate`] built from
//! [`praxis_retrofit::preventive_gate::GateValidator`] output against the
//! project's `./Cargo.toml`: only `Fail`-severity `Critical` findings map to
//! [`star_toml::OracleVerdict::Fail`], which blocks admission.
//!
//! Access the admitted config lazily via [`config`] (a `OnceLock`-backed
//! accessor, since verbs are free functions with no shared app state).

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use praxis_retrofit::preventive_gate::{
    GateValidator, Severity, ValidationResult, ValidationStatus,
};
use serde::{Deserialize, Serialize};
use star_toml::{
    loader::{ConfigLifecycle, TrustedLoader},
    nouns::{EvidenceGate, OracleGateVerdict, OracleVerdict},
    Validate, Validator,
};

/// Baked-in default configuration, expressed as TOML so it participates in
/// the same layered-merge/provenance machinery as file and env layers (and so
/// the witness hash covers the documented defaults, not just `Default::default()`).
const DEFAULTS_TOML: &str = r#"
[law]
default_policy = "default"
signing_key_env = "PRAXIS_SIGNING_KEY"

[receipts]
dir = "receipts"

[planner]
attention_capacity = 7

[mfg]
ontology_dir = "ontology"
template_dir = "templates"

[gate]
preventive_checks = true
strict = false
"#;

/// Top-level praxis configuration schema.
///
/// Every nested struct is `#[serde(default)]` so a config layer may specify
/// only the fields it cares about; missing sections/fields fall back to
/// [`Default`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PraxisConfig {
    /// `law judge`/`admit`/signing settings.
    pub law: LawConfig,
    /// Receipt store settings.
    pub receipts: ReceiptsConfig,
    /// Planner (attention/capacity) settings.
    pub planner: PlannerConfig,
    /// Manufacturing/ontology-projection (`mfg`) settings.
    pub mfg: MfgConfig,
    /// Preventive-gate/admission settings.
    pub gate: GateConfig,
}

/// `[law]` — default policy name and signing key resolution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LawConfig {
    /// Default law/policy name used by `law judge`/`admit` when `--law` is omitted.
    pub default_policy: String,
    /// Env var consulted for the Ed25519 signing key before `signing_key_path`.
    pub signing_key_env: String,
    /// Optional path to an Ed25519 signing key file. `None` means unsigned
    /// unless `signing_key_env` resolves at runtime.
    pub signing_key_path: Option<PathBuf>,
}

impl Default for LawConfig {
    fn default() -> Self {
        Self {
            default_policy: "default".to_string(),
            signing_key_env: "PRAXIS_SIGNING_KEY".to_string(),
            signing_key_path: None,
        }
    }
}

/// `[receipts]` — receipt store location.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReceiptsConfig {
    /// Receipt store directory (relative to the process cwd unless absolute).
    pub dir: String,
}

impl Default for ReceiptsConfig {
    fn default() -> Self {
        Self {
            dir: "receipts".to_string(),
        }
    }
}

/// `[planner]` — planning/attention settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlannerConfig {
    /// Default attention capacity (valid range `1..=64`).
    pub attention_capacity: u32,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            attention_capacity: 7,
        }
    }
}

/// `[mfg]` — ontology/template directories for the `mfg` noun.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MfgConfig {
    /// Directory containing `.ttl` ontology sources.
    pub ontology_dir: String,
    /// Directory containing codegen templates.
    pub template_dir: String,
}

impl Default for MfgConfig {
    fn default() -> Self {
        Self {
            ontology_dir: "ontology".to_string(),
            template_dir: "templates".to_string(),
        }
    }
}

/// `[gate]` — preventive-gate admission behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GateConfig {
    /// Run `GateValidator` preventive checks against `./Cargo.toml` during admission.
    pub preventive_checks: bool,
    /// Treat gate `Warn` verdicts as `Fail` (blocking).
    pub strict: bool,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            preventive_checks: true,
            strict: false,
        }
    }
}

// ── Validation ──────────────────────────────────────────────────────────

impl Validate for PraxisConfig {
    fn validate(&self, v: &mut Validator) {
        v.field("law", |v| {
            v.check_non_empty("default_policy", &self.law.default_policy);
            v.check_non_empty("signing_key_env", &self.law.signing_key_env);
            if let Some(p) = &self.law.signing_key_path {
                check_path_looks_safe(v, "signing_key_path", p);
            }
        });
        v.field("receipts", |v| {
            check_path_str_looks_safe(v, "dir", &self.receipts.dir);
        });
        v.field("planner", |v| {
            v.check_range(
                "attention_capacity",
                self.planner.attention_capacity,
                1u32..=64u32,
            );
        });
        v.field("mfg", |v| {
            check_path_str_looks_safe(v, "ontology_dir", &self.mfg.ontology_dir);
            check_path_str_looks_safe(v, "template_dir", &self.mfg.template_dir);
        });
    }
}

/// Reject empty, null-byte-containing, or `..`-traversing path strings.
///
/// Deliberately simpler than [`star_toml::path::resolve_and_validate`]: our
/// paths are plain config-relative directory names anchored to the process
/// cwd, not file-relative includes, so there is no natural `source_path` to
/// anchor a sandbox/relative-only policy against.
fn check_path_str_looks_safe(v: &mut Validator, field: &str, value: &str) {
    v.check_path(field, value, None);
}

fn check_path_looks_safe(v: &mut Validator, field: &str, value: &Path) {
    check_path_str_looks_safe(v, field, &value.to_string_lossy());
}

impl ConfigLifecycle for PraxisConfig {
    fn normalize(&mut self) {
        self.law.default_policy = self.law.default_policy.trim().to_string();
        self.law.signing_key_env = self.law.signing_key_env.trim().to_string();
        self.law.signing_key_path = self.law.signing_key_path.take().map(|p| expand_tilde(&p));
        self.receipts.dir = expand_tilde_str(self.receipts.dir.trim());
        self.mfg.ontology_dir = expand_tilde_str(self.mfg.ontology_dir.trim());
        self.mfg.template_dir = expand_tilde_str(self.mfg.template_dir.trim());
    }
}

/// Expand a leading `~/` to `$HOME/`. Leaves the path untouched if `HOME` is
/// unset or the path does not start with `~/`.
fn expand_tilde_str(value: &str) -> String {
    expand_tilde(Path::new(value))
        .to_string_lossy()
        .into_owned()
}

fn expand_tilde(path: &Path) -> PathBuf {
    let Some(raw) = path.to_str() else {
        return path.to_path_buf();
    };
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    path.to_path_buf()
}

// ── Gate adapter: praxis-retrofit ValidationResult → star-toml OracleGateVerdict ──

/// Map a single [`ValidationResult`] to an [`OracleGateVerdict`].
///
/// Only `(Fail, Critical)` blocks admission (`Fail`). Everything else is a
/// non-blocking `Admit`/`Suggest`/`Warn` so day-to-day repo drift never
/// prevents `praxis` from running — it only prevents running with a
/// Cargo.toml that violates a *critical* house invariant (e.g. CalVer format).
pub fn gate_verdict_from_validation(result: &ValidationResult) -> OracleGateVerdict {
    let verdict = match (result.status, result.severity) {
        (ValidationStatus::Pass, _) => OracleVerdict::Admit,
        (ValidationStatus::Warn, Severity::Info) => OracleVerdict::Suggest,
        (ValidationStatus::Warn, _) => OracleVerdict::Warn,
        (ValidationStatus::Fail, Severity::Critical) => OracleVerdict::Fail,
        (ValidationStatus::Fail, _) => OracleVerdict::Warn,
    };
    OracleGateVerdict {
        verdict,
        reason: format!("{}: {}", result.check_name, result.message),
        evidence_refs: result.remediation.iter().cloned().collect(),
        source: format!("praxis-retrofit GateValidator/{:?}", result.category),
    }
}

/// Map a batch of `GateValidator` results to `OracleGateVerdict`s.
pub fn gate_verdicts_from_validation(results: &[ValidationResult]) -> Vec<OracleGateVerdict> {
    results.iter().map(gate_verdict_from_validation).collect()
}

/// Build the `EvidenceGate` fed into `load_admitted`, from `GateValidator`
/// output against `./Cargo.toml` (skipped silently if the file is absent —
/// e.g. when running outside a checked-out praxis tree).
pub fn build_default_gate() -> anyhow::Result<EvidenceGate> {
    let mut gate = EvidenceGate::new();
    let cargo_toml = Path::new("Cargo.toml");
    if cargo_toml.exists() {
        let results = GateValidator::new().validate_cargo_toml(cargo_toml)?;
        for verdict in gate_verdicts_from_validation(&results) {
            gate.push(verdict);
        }
    }
    Ok(gate)
}

// ── Layered admission loader ───────────────────────────────────────────────

/// Load and admit [`PraxisConfig`] from the standard layer set, feeding a
/// real preventive-gate [`EvidenceGate`] built from `./Cargo.toml`.
///
/// # Errors
/// Returns an error if any layer fails to parse, an unknown field is present,
/// a validation invariant fails, or the gate produces a blocking `Fail` verdict.
pub fn load_config() -> anyhow::Result<star_toml::loader::AdmittedConfig<PraxisConfig>> {
    let gate = build_default_gate()?;
    load_config_with_gate(gate)
}

/// Like [`load_config`] but with a caller-supplied [`EvidenceGate`] — used by
/// tests to inject synthetic verdicts without touching the real `Cargo.toml`.
///
/// # Errors
/// See [`load_config`].
pub fn load_config_with_gate(
    gate: EvidenceGate,
) -> anyhow::Result<star_toml::loader::AdmittedConfig<PraxisConfig>> {
    let home_config =
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".praxis/config.toml"));

    let mut loader = TrustedLoader::new().layer_str(DEFAULTS_TOML, "built-in defaults");
    if let Some(home_config) = home_config {
        loader = loader.layer_file_if_exists(home_config);
    }
    loader = loader
        .layer_file_if_exists("./praxis.toml")
        // `PRAXIS_CONFIG__`, not `PRAXIS_`: operational vars like
        // PRAXIS_SIGNING_KEY must not enter (and be rejected by) the strict
        // config admission surface — see the module docs.
        .env_prefix("PRAXIS_CONFIG__")
        .with_oracle_gate(gate);

    loader
        .load_admitted::<PraxisConfig>()
        .map_err(|e| anyhow::anyhow!("{e}"))
}

// ── Lazy global accessor ────────────────────────────────────────────────────

type ConfigCell = OnceLock<Result<star_toml::loader::AdmittedConfig<PraxisConfig>, String>>;

static CONFIG: ConfigCell = OnceLock::new();

/// Lazily admit the effective [`PraxisConfig`] (once per process), returning
/// a shared reference. A failed admission is cached and re-reported on every
/// call rather than retried — the failure is deterministic given fixed
/// files/env, and this keeps `praxis --help` usable even with a broken config
/// (nothing calls `config()` on that path).
///
/// # Errors
/// Returns an error if admission failed (unknown field, validation failure,
/// or blocking gate verdict).
pub fn config() -> anyhow::Result<&'static star_toml::loader::AdmittedConfig<PraxisConfig>> {
    match CONFIG.get_or_init(|| load_config().map_err(|e| e.to_string())) {
        Ok(cfg) => Ok(cfg),
        Err(e) => Err(anyhow::anyhow!("{e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_admit_with_no_gate() {
        let admitted = load_config_with_gate(EvidenceGate::new()).expect("defaults must admit");
        assert_eq!(admitted.value().planner.attention_capacity, 7);
        assert_eq!(admitted.value().law.default_policy, "default");
        assert_eq!(admitted.value().receipts.dir, "receipts");
        assert_eq!(admitted.value().mfg.ontology_dir, "ontology");
        assert_eq!(admitted.value().gate.preventive_checks, true);
    }

    #[test]
    fn gate_adapter_maps_pass_to_admit() {
        let result = ValidationResult {
            status: ValidationStatus::Pass,
            category: praxis_retrofit::preventive_gate::ValidateCategory::Versioning,
            check_name: "test".to_string(),
            message: "ok".to_string(),
            remediation: None,
            severity: Severity::Info,
        };
        assert_eq!(
            gate_verdict_from_validation(&result).verdict,
            OracleVerdict::Admit
        );
    }

    #[test]
    fn gate_adapter_maps_warn_info_to_suggest() {
        let result = ValidationResult {
            status: ValidationStatus::Warn,
            category: praxis_retrofit::preventive_gate::ValidateCategory::Licensing,
            check_name: "test".to_string(),
            message: "suggest".to_string(),
            remediation: None,
            severity: Severity::Info,
        };
        assert_eq!(
            gate_verdict_from_validation(&result).verdict,
            OracleVerdict::Suggest
        );
    }

    #[test]
    fn gate_adapter_maps_warn_other_to_warn() {
        let result = ValidationResult {
            status: ValidationStatus::Warn,
            category: praxis_retrofit::preventive_gate::ValidateCategory::Licensing,
            check_name: "test".to_string(),
            message: "warn".to_string(),
            remediation: None,
            severity: Severity::Warning,
        };
        assert_eq!(
            gate_verdict_from_validation(&result).verdict,
            OracleVerdict::Warn
        );
    }

    #[test]
    fn gate_adapter_maps_fail_critical_to_fail() {
        let result = ValidationResult {
            status: ValidationStatus::Fail,
            category: praxis_retrofit::preventive_gate::ValidateCategory::Versioning,
            check_name: "test".to_string(),
            message: "critical fail".to_string(),
            remediation: Some("fix it".to_string()),
            severity: Severity::Critical,
        };
        assert_eq!(
            gate_verdict_from_validation(&result).verdict,
            OracleVerdict::Fail
        );
    }

    #[test]
    fn gate_adapter_maps_fail_non_critical_to_warn() {
        let result = ValidationResult {
            status: ValidationStatus::Fail,
            category: praxis_retrofit::preventive_gate::ValidateCategory::Msrv,
            check_name: "test".to_string(),
            message: "non-critical fail".to_string(),
            remediation: None,
            severity: Severity::Error,
        };
        assert_eq!(
            gate_verdict_from_validation(&result).verdict,
            OracleVerdict::Warn
        );
    }
}
