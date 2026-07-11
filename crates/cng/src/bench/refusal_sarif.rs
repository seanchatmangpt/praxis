//! SARIF 2.1.0-shaped JSON for `CngRefusal` errors — a CI-annotation
//! primitive, not a CI wiring. Converts any typed refusal
//! (`crate::powl::CngRefusal`) into a minimal SARIF log: `ruleId` = the
//! refusal's `code()` (`CNG_R01`..`CNG_R25`), `message.text` = the
//! refusal's `Display` rendering (code-prefixed diagnostic including all
//! structured fields, e.g. tick/candidate counts, dispatch ids), `level`
//! = `"error"` always (every `CngRefusal` variant is a typed refusal by
//! construction — there is no warning-severity refusal in this crate).
//!
//! SARIF 2.1.0 top-level shape (OASIS spec,
//! <https://docs.oasis-open.org/sarif/sarif/v2.1.0/os/sarif-v2.1.0-os.html>,
//! §3.13 `sarifLog`): `$schema`, `version`, and `runs` (an array of
//! `run` objects, each with a `tool.driver.name` and a `results` array).
//! This module emits exactly that shape with one run and one result per
//! call; a caller that needs to batch multiple refusals into one log
//! concatenates the `results` arrays under a single `SarifRun`.
//!
//! CI wiring status (checked, not modified, at the time this module was
//! added): `.github/workflows/` exists and `ci.yml` runs
//! `cargo test --workspace --all-features`, which already exercises the
//! `cng` crate's `bench` feature — but no workflow step consumes
//! SARIF-shaped output today. This module is the conversion primitive a
//! future CI step (e.g. a `cng bench report --sarif` verb, or an
//! upload-sarif action) would call; wiring it in is out of scope here.

use crate::powl::CngRefusal;
use serde::Serialize;

/// SARIF 2.1.0 schema URL, per the OASIS spec's recommended `$schema`
/// value for conforming logs.
const SARIF_SCHEMA: &str =
    "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";
const TOOL_NAME: &str = "cng";

/// Top-level SARIF log (`sarifLog`, SARIF 2.1.0 §3.13).
#[derive(Debug, Clone, Serialize)]
pub struct SarifLog {
    #[serde(rename = "$schema")]
    pub schema: &'static str,
    pub version: &'static str,
    pub runs: Vec<SarifRun>,
}

/// A single analysis run (`run`, §3.14). This crate always emits exactly
/// one run per `refusal_to_sarif` call.
#[derive(Debug, Clone, Serialize)]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
}

/// `tool` object (§3.18): identifies the analysis tool.
#[derive(Debug, Clone, Serialize)]
pub struct SarifTool {
    pub driver: SarifDriver,
}

/// `toolComponent` (§3.19), reduced to the one field this module sets.
#[derive(Debug, Clone, Serialize)]
pub struct SarifDriver {
    pub name: &'static str,
}

/// A single `result` object (§3.27): one finding. `rule_id` maps to
/// `ruleId`, `level` is always `"error"` for a `CngRefusal`, and
/// `message` carries the human-readable text.
#[derive(Debug, Clone, Serialize)]
pub struct SarifResult {
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    pub level: &'static str,
    pub message: SarifMessage,
}

/// `message` object (§3.11), reduced to the plain-text form.
#[derive(Debug, Clone, Serialize)]
pub struct SarifMessage {
    pub text: String,
}

/// Converts a single `CngRefusal` into a one-run, one-result SARIF-lite
/// log.
///
/// `ruleId` is `refusal.code()` (the stable `CNG_Rxx` identifier);
/// `message.text` is `refusal.to_string()` (the `Display` rendering,
/// which is `code(): message() (<structured fields>)` for struct-variant
/// refusals and `code(): message()` for tuple-variant refusals) so the
/// SARIF result carries the same diagnostic detail as the refusal's own
/// `Display` impl, not just the bare `message()` string.
///
/// # Complexity
/// O(1): one refusal in, one result out, no iteration.
pub fn refusal_to_sarif(refusal: &CngRefusal) -> SarifLog {
    SarifLog {
        schema: SARIF_SCHEMA,
        version: SARIF_VERSION,
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver { name: TOOL_NAME },
            },
            results: vec![SarifResult {
                rule_id: refusal.code().to_string(),
                level: "error",
                message: SarifMessage {
                    text: refusal.to_string(),
                },
            }],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_ttl_tuple_variant_converts_to_cng_r01() {
        let refusal = CngRefusal::MalformedTtl("bad turtle at line 3".to_string());
        let log = refusal_to_sarif(&refusal);

        assert_eq!(log.schema, SARIF_SCHEMA);
        assert_eq!(log.version, "2.1.0");
        assert_eq!(log.runs.len(), 1);
        assert_eq!(log.runs[0].tool.driver.name, "cng");
        assert_eq!(log.runs[0].results.len(), 1);

        let result = &log.runs[0].results[0];
        assert_eq!(result.rule_id, "CNG_R01");
        assert_eq!(result.level, "error");
        assert!(result.message.text.contains("CNG_R01"));
        assert!(result.message.text.contains("bad turtle at line 3"));
    }

    #[test]
    fn standing_ambiguous_struct_variant_converts_to_cng_r12() {
        let refusal = CngRefusal::StandingAmbiguous {
            tick: 7,
            candidate_count: 3,
        };
        let log = refusal_to_sarif(&refusal);

        let result = &log.runs[0].results[0];
        assert_eq!(result.rule_id, "CNG_R12");
        assert_eq!(result.level, "error");
        // Display embeds the struct fields; the SARIF message must carry
        // them too (not just the bare `message()` prose).
        assert!(result.message.text.contains("tick 7"));
        assert!(result.message.text.contains("3 candidates"));
    }

    #[test]
    fn double_admit_struct_variant_converts_to_cng_r25() {
        let refusal = CngRefusal::DoubleAdmit {
            dispatch: "disp-42".to_string(),
            idempotency_key: "key-abc".to_string(),
        };
        let log = refusal_to_sarif(&refusal);

        let result = &log.runs[0].results[0];
        assert_eq!(result.rule_id, "CNG_R25");
        assert_eq!(result.level, "error");
        assert!(result.message.text.contains("disp-42"));
        assert!(result.message.text.contains("key-abc"));
    }

    #[test]
    fn evidence_gate_failed_converts_to_cng_r19() {
        let refusal = CngRefusal::EvidenceGateFailed {
            gate: "unreceipted-actuations".to_string(),
            count: 4,
        };
        let log = refusal_to_sarif(&refusal);

        let result = &log.runs[0].results[0];
        assert_eq!(result.rule_id, "CNG_R19");
        assert!(result.message.text.contains("unreceipted-actuations"));
        assert!(result.message.text.contains('4'));
    }

    /// Round-trips a converted log through `serde_json::to_string` and
    /// back, then checks the SARIF 2.1.0 top-level shape by key name
    /// (`$schema`, `version`, `runs`) rather than by re-deserializing
    /// into `SarifLog` — this catches a `#[serde(rename)]` regression
    /// that a same-struct round-trip would silently pass.
    #[test]
    fn json_round_trips_and_is_sarif_shaped() {
        let refusal = CngRefusal::PlanUnsolvable("empty tape".to_string());
        let log = refusal_to_sarif(&refusal);

        let json = serde_json::to_string(&log).expect("SarifLog must serialize to JSON");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("emitted JSON must parse as valid JSON");

        assert!(value.get("$schema").is_some(), "missing top-level $schema");
        assert!(value.get("version").is_some(), "missing top-level version");
        let runs = value
            .get("runs")
            .and_then(|r| r.as_array())
            .expect("runs must be a JSON array");
        assert_eq!(runs.len(), 1);

        let run0 = &runs[0];
        assert!(run0.get("tool").is_some(), "run missing tool");
        let results = run0
            .get("results")
            .and_then(|r| r.as_array())
            .expect("run.results must be a JSON array");
        assert_eq!(results.len(), 1);

        let result0 = &results[0];
        assert_eq!(
            result0.get("ruleId").and_then(|v| v.as_str()),
            Some("CNG_R04")
        );
        assert_eq!(result0.get("level").and_then(|v| v.as_str()), Some("error"));
        assert!(result0
            .get("message")
            .and_then(|m| m.get("text"))
            .and_then(|t| t.as_str())
            .is_some_and(|t| t.contains("empty tape")));
    }
}
