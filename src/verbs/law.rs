//! `law` verb dispatcher — judge, admit, receipt, show, promote.
//!
//! Dispatches to the real admission/receipt/standing crates for the praxis
//! law object lifecycle: Raw → Validated → Admitted → Receipted.
//!
//! Each subcommand accepts a JSON payload via stdin or argument, wraps it in
//! a [`praxis_core::LawObject`], runs [`DefaultLaw`]'s [`Judge`]/[`Admit`]
//! implementations (and, when an `atom`/`rule` is present, prolog8
//! admission), and returns the result as JSON.
//!
//! Malformed input (bad JSON, bad hex, an unparseable standing name, an
//! `atom`/`rule` without a `catalog`) is a hard `Err`. A *domain* denial
//! (halted obligations, a prolog8 rejection, a missing auditor) is `Ok(json)`
//! with a `status`/`verdict` field describing the denial.

use std::time::{SystemTime, UNIX_EPOCH};

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::{arg, verb};
use praxis_core::{
    law::ReceiptMeta,
    lifecycle::{Raw, Validated},
    Admit, Andon, DefaultLaw, Judge, LawObject, Obligation,
};
use prolog8::{admit_atom, admit_rule, Atom8, Catalog, RejectionCode, Rule8};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use wasm4pm_cognition::breeds::standing::BreedStanding;

// ── Input schemas ─────────────────────────────────────────────────────────

/// Wire schema for an obligation, tagged by `type`. Distinct from
/// [`Obligation`]'s own (untagged-by-variant-name) `Serialize`/`Deserialize`
/// impl so callers can write `{"type": "blocking_constraint", ...}`.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ObligationInput {
    /// A predicate that must be satisfied; `params_hash_hex` defaults to 32 zero bytes.
    Precondition {
        predicate_id: String,
        #[serde(default)]
        params_hash_hex: Option<String>,
    },
    /// A hard constraint that can never be satisfied by payload content alone.
    BlockingConstraint { reason: String },
    /// External evidence that must appear in the payload's `evidence` array.
    EvidenceRequired { evidence_type: String },
}

/// Shared input schema for `judge`/`admit`/`receipt`.
#[derive(Deserialize)]
struct LawInput {
    value: Value,
    #[serde(default)]
    obligations: Vec<ObligationInput>,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    satisfied_predicates: Vec<String>,
    #[serde(default)]
    atom: Option<Value>,
    #[serde(default)]
    rule: Option<Value>,
    #[serde(default)]
    catalog: Option<Value>,
}

/// Extra fields accepted only by `receipt`, layered on top of [`LawInput`].
#[derive(Deserialize)]
struct ReceiptFields {
    #[serde(default)]
    prev_chain_hash: Option<String>,
    #[serde(default)]
    ts_ns: Option<u64>,
    #[serde(default)]
    instruction_id: Option<u64>,
    #[serde(default)]
    activity_idx: Option<u16>,
    #[serde(default)]
    node_kind: Option<u8>,
}

/// Input schema for `promote`.
#[derive(Deserialize)]
struct PromoteInput {
    standing: String,
}

// ── Parsing helpers ───────────────────────────────────────────────────────

/// Parse a payload string into JSON. Malformed or empty input is a hard error.
fn parse_value(payload: &str) -> std::result::Result<Value, String> {
    if payload.trim().is_empty() {
        return Err("empty payload".to_string());
    }
    serde_json::from_str(payload).map_err(|e| format!("invalid JSON: {e}"))
}

/// Decode exactly 32 bytes of hex (64 hex characters).
fn parse_hex32(hex_str: &str) -> std::result::Result<[u8; 32], String> {
    let bytes = hex::decode(hex_str).map_err(|e| format!("invalid hex: {e}"))?;
    bytes
        .try_into()
        .map_err(|v: Vec<u8>| format!("expected 32 bytes (64 hex chars), got {} bytes", v.len()))
}

/// Convert wire-schema obligations into the core `Obligation` type.
fn parse_obligations(raw: Vec<ObligationInput>) -> std::result::Result<Vec<Obligation>, String> {
    raw.into_iter()
        .map(|o| match o {
            ObligationInput::Precondition { predicate_id, params_hash_hex } => {
                let params_hash = match params_hash_hex {
                    Some(hex_str) => parse_hex32(&hex_str)?,
                    None => [0u8; 32],
                };
                Ok(Obligation::Precondition { predicate_id, params_hash })
            }
            ObligationInput::BlockingConstraint { reason } => {
                Ok(Obligation::BlockingConstraint { reason })
            }
            ObligationInput::EvidenceRequired { evidence_type } => {
                Ok(Obligation::EvidenceRequired { evidence_type })
            }
        })
        .collect()
}

/// Serialize a value to JSON, falling back to `null` (never panics).
fn to_json<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

/// Extract the unmet-obligations list from an `Andon`, empty if not `Halted`.
fn unmet_from(andon: &Andon) -> Vec<Obligation> {
    match andon {
        Andon::Halted { unmet, .. } => unmet.clone(),
        _ => Vec::new(),
    }
}

/// Render a prolog8 admission result as `"admitted"` or the rejection's Display text.
fn admit_result_json(result: std::result::Result<(), RejectionCode>) -> Value {
    match result {
        Ok(()) => json!("admitted"),
        Err(code) => json!(code.to_string()),
    }
}

/// Run prolog8 admission on an optional `atom`/`rule` against a required `catalog`.
///
/// Returns `Ok(None)` if neither `atom` nor `rule` was supplied. Returns
/// `Err` if `atom`/`rule` is present without a `catalog`, or if any of
/// `atom`/`rule`/`catalog` fail to deserialize into their prolog8 types.
fn run_prolog8_checks(input: &LawInput) -> std::result::Result<Option<Value>, String> {
    if input.atom.is_none() && input.rule.is_none() {
        return Ok(None);
    }
    let catalog_value = input
        .catalog
        .as_ref()
        .ok_or_else(|| "atom or rule provided without a catalog".to_string())?;
    let catalog: Catalog =
        serde_json::from_value(catalog_value.clone()).map_err(|e| format!("invalid catalog: {e}"))?;

    let mut out = Map::new();
    if let Some(atom_value) = &input.atom {
        let atom: Atom8 =
            serde_json::from_value(atom_value.clone()).map_err(|e| format!("invalid atom: {e}"))?;
        out.insert("atom".to_string(), admit_result_json(admit_atom(&atom, &catalog)));
    }
    if let Some(rule_value) = &input.rule {
        let rule: Rule8 =
            serde_json::from_value(rule_value.clone()).map_err(|e| format!("invalid rule: {e}"))?;
        out.insert("rule".to_string(), admit_result_json(admit_rule(&rule, &catalog)));
    }
    Ok(Some(Value::Object(out)))
}

// ── Judge pipeline ────────────────────────────────────────────────────────

/// Outcome of running `DefaultLaw::judge` on a freshly constructed `LawObject`.
enum JudgeOutcome {
    Validated(LawObject<Value, Validated, DefaultLaw>),
    Halted(Andon),
}

/// Everything downstream verbs need after judging a payload.
struct JudgeContext {
    outcome: JudgeOutcome,
    prolog8: Option<Value>,
    /// The original `value` field, kept separately from the `LawObject`'s
    /// payload (which also carries `evidence`/`satisfied_predicates`) so
    /// callers can hash or display just the domain value.
    value: Value,
}

/// Parse a `LawInput`, run prolog8 checks, build a `LawObject`, and judge it.
fn judge_from_value(value: &Value) -> std::result::Result<JudgeContext, String> {
    let input: LawInput =
        serde_json::from_value(value.clone()).map_err(|e| format!("invalid law payload: {e}"))?;
    let prolog8 = run_prolog8_checks(&input)?;
    let obligations = parse_obligations(input.obligations)?;
    let law_payload = json!({
        "value": input.value,
        "evidence": input.evidence,
        "satisfied_predicates": input.satisfied_predicates,
    });

    let raw = LawObject::<Value, Raw, DefaultLaw>::new(law_payload, obligations);
    let outcome = match DefaultLaw::judge(raw) {
        Ok(validated) => JudgeOutcome::Validated(validated),
        Err(halted) => JudgeOutcome::Halted(halted.andon().clone()),
    };

    Ok(JudgeContext { outcome, prolog8, value: input.value })
}

/// A denied-pipeline response shared by `admit` and `receipt`.
fn denied_object(andon: &Andon) -> Value {
    json!({
        "status": "denied",
        "verdict": "halted",
        "andon": to_json(andon),
        "unmet": to_json(&unmet_from(andon)),
    })
}

// ── Domain logic ──────────────────────────────────────────────────────────

/// Domain logic for `judge`: Raw → Validated or Halted.
fn judge_payload(payload: &str, law: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let ctx = judge_from_value(&value)?;

    let mut obj = Map::new();
    obj.insert("status".to_string(), json!("judged"));
    obj.insert("law".to_string(), json!(law));
    match ctx.outcome {
        JudgeOutcome::Validated(_) => {
            obj.insert("verdict".to_string(), json!("validated"));
            obj.insert("andon".to_string(), to_json(&Andon::Green));
        }
        JudgeOutcome::Halted(andon) => {
            obj.insert("verdict".to_string(), json!("halted"));
            obj.insert("unmet".to_string(), to_json(&unmet_from(&andon)));
            obj.insert("andon".to_string(), to_json(&andon));
        }
    }
    if let Some(p) = ctx.prolog8 {
        obj.insert("prolog8".to_string(), p);
    }
    Ok(Value::Object(obj))
}

/// Domain logic for `admit`: Validated → Admitted (or denied).
fn admit_payload(payload: &str, policy: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let ctx = judge_from_value(&value)?;

    match ctx.outcome {
        JudgeOutcome::Halted(andon) => Ok(denied_object(&andon)),
        JudgeOutcome::Validated(validated) => match DefaultLaw::admit(validated) {
            Ok(_admitted) => Ok(json!({
                "status": "admitted",
                "state": "Admitted",
                "policy": policy,
            })),
            Err(andon) => Ok(denied_object(&andon)),
        },
    }
}

/// Domain logic for `receipt`: Admitted → Receipted, BLAKE3-chained (or denied).
fn receipt_payload(payload: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let fields: ReceiptFields =
        serde_json::from_value(value.clone()).map_err(|e| format!("invalid receipt fields: {e}"))?;
    let prev_chain_hash = match &fields.prev_chain_hash {
        Some(hex_str) => parse_hex32(hex_str)?,
        None => [0u8; 32],
    };

    let ctx = judge_from_value(&value)?;
    let validated = match ctx.outcome {
        JudgeOutcome::Halted(andon) => return Ok(denied_object(&andon)),
        JudgeOutcome::Validated(validated) => validated,
    };
    let admitted = match DefaultLaw::admit(validated) {
        Ok(admitted) => admitted,
        Err(andon) => return Ok(denied_object(&andon)),
    };

    let ts_ns = fields.ts_ns.unwrap_or_else(|| {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64
    });
    let meta = ReceiptMeta {
        instruction_id: fields.instruction_id.unwrap_or(0),
        activity_idx: fields.activity_idx.unwrap_or(0),
        node_kind: fields.node_kind.unwrap_or(0),
        ts_ns: Some(ts_ns),
    };

    let receipted = admitted.receipt(&prev_chain_hash, meta).map_err(|e| e.to_string())?;

    let chain_hash = receipted.chain_hash().copied().unwrap_or([0u8; 32]);
    let chain_hex = hex::encode(chain_hash);
    let payload_bytes = serde_json::to_vec(&ctx.value).map_err(|e| e.to_string())?;
    let payload_hash_hex = blake3::hash(&payload_bytes).to_hex().to_string();

    Ok(json!({
        "status": "receipted",
        "state": "Receipted",
        "chain_hash": chain_hex,
        "canonical": format!("blake3:{chain_hex}"),
        "prev_chain_hash": hex::encode(prev_chain_hash),
        "payload_hash": payload_hash_hex,
        "ts_ns": ts_ns,
    }))
}

/// Domain logic for `show`: render a payload as json or text.
fn show_payload(payload: &str, format: &str) -> std::result::Result<Value, String> {
    let payload_json = parse_value(payload)?;
    match format {
        "json" => Ok(payload_json),
        "text" => {
            let pretty = serde_json::to_string_pretty(&payload_json).map_err(|e| e.to_string())?;
            Ok(json!({"format": "text", "content": pretty}))
        }
        _ => Err("unknown format; use 'json' or 'text'".to_string()),
    }
}

/// The ten-rung `BreedStanding` ladder in ascending order.
const LADDER: [BreedStanding; 10] = [
    BreedStanding::Named,
    BreedStanding::Registered,
    BreedStanding::Dispatchable,
    BreedStanding::Bounded,
    BreedStanding::Oracled,
    BreedStanding::Traceable,
    BreedStanding::Canonical,
    BreedStanding::Refusable,
    BreedStanding::Replayable,
    BreedStanding::Certified,
];

/// The rung immediately above `current`, or `None` if `current` is the top rung.
fn next_standing(current: BreedStanding) -> Option<BreedStanding> {
    let idx = LADDER.iter().position(|&s| s == current)?;
    LADDER.get(idx + 1).copied()
}

/// The SCREAMING_SNAKE_CASE registry name for a standing.
fn standing_name(standing: BreedStanding) -> String {
    match to_json(&standing) {
        Value::String(s) => s,
        _ => String::new(),
    }
}

/// Domain logic for `promote`: auditor-gated standing promotion.
fn promote_payload(payload: &str, auditor: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let input: PromoteInput =
        serde_json::from_value(value).map_err(|e| format!("invalid promote payload: {e}"))?;
    let current = BreedStanding::from_registry_str(&input.standing)
        .ok_or_else(|| format!("unrecognized standing: {}", input.standing))?;

    if current == BreedStanding::Certified {
        return Ok(json!({
            "status": "denied",
            "reason": "already at top rung (CERTIFIED)",
            "from": standing_name(current),
        }));
    }

    let next = match next_standing(current) {
        Some(next) => next,
        None => {
            return Ok(json!({
                "status": "denied",
                "reason": "no further rung available",
                "from": standing_name(current),
            }));
        }
    };

    let auditor_required = next >= BreedStanding::Replayable;
    if auditor_required && auditor.is_empty() {
        return Ok(json!({
            "status": "denied",
            "reason": format!("promotion to {} requires an auditor", standing_name(next)),
            "from": standing_name(current),
            "to": standing_name(next),
        }));
    }

    Ok(json!({
        "status": "promoted",
        "from": standing_name(current),
        "to": standing_name(next),
        "is_partial_alive_eligible": next.is_partial_alive_eligible(),
        "auditor": if auditor.is_empty() { Value::Null } else { json!(auditor) },
    }))
}

// ── Verb registration ─────────────────────────────────────────────────────

/// Judge a raw LawObject: Raw → Validated or Halted.
///
/// Reads a JSON payload, wraps it in a `LawObject`, applies `DefaultLaw`'s
/// `Judge` to validate obligations (and, if `atom`/`rule` are present, runs
/// prolog8 admission), and returns the Validated/Halted verdict.
#[verb]
pub fn judge(
    payload: String,
    #[arg(default_value = "default", help = "Policy name to judge against")] law: String,
) -> Result<Value> {
    judge_payload(&payload, &law).map_err(NounVerbError::argument_error)
}

/// Admit a validated LawObject: Validated → Admitted.
///
/// Transitions a payload through judgment and admission via `DefaultLaw`.
/// Denial (halted at judge or admit) is returned as `Ok` with a `denied` shape.
#[verb]
pub fn admit(
    payload: String,
    #[arg(default_value = "default", help = "Admission policy name")] policy: String,
) -> Result<Value> {
    admit_payload(&payload, &policy).map_err(NounVerbError::argument_error)
}

/// Generate a receipt for an admitted LawObject.
///
/// Runs the full judge → admit → receipt pipeline and produces a
/// BLAKE3 chain hash bound to both the payload and the previous link.
#[verb]
pub fn receipt(payload: String) -> Result<Value> {
    receipt_payload(&payload).map_err(NounVerbError::argument_error)
}

/// Show a law object receipt as JSON or human-readable format.
#[verb]
pub fn show(
    payload: String,
    #[arg(default_value = "json", help = "Output format: json or text")] format: String,
) -> Result<Value> {
    show_payload(&payload, &format).map_err(NounVerbError::argument_error)
}

/// Promote a law object via the `BreedStanding` ladder.
///
/// Promotions to `Replayable` or `Certified` require a non-empty `auditor`.
#[verb]
pub fn promote(
    payload: String,
    #[arg(default_value = "", help = "Auditor name endorsing the promotion")] auditor: String,
) -> Result<Value> {
    promote_payload(&payload, &auditor).map_err(NounVerbError::argument_error)
}

#[cfg(test)]
mod tests {
    use prolog8::{CatalogId, PredicateId, PredicateMeta, PredicateProofPolicy};

    use super::*;

    fn build_catalog(pred_arity: u8) -> Catalog {
        let mut catalog = Catalog::new(CatalogId(1));
        catalog.add_predicate(PredicateMeta {
            pred_id: PredicateId(1),
            label: "p".to_string(),
            arity: pred_arity,
            access_orders: vec![],
            proof_policy: PredicateProofPolicy::OnRequest,
            materialized: false,
        });
        catalog
    }

    // ── judge ───────────────────────────────────────────────────────────

    #[test]
    fn judge_with_valid_value_only_becomes_validated() {
        let result = judge_payload(r#"{"value":{"id":1}}"#, "default").expect("should judge");
        assert_eq!(result["verdict"], json!("validated"));
        assert_eq!(result["status"], json!("judged"));
    }

    #[test]
    fn judge_with_blocking_constraint_becomes_halted_with_unmet() {
        let payload = r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
        let result = judge_payload(payload, "default").expect("should judge");
        assert_eq!(result["verdict"], json!("halted"));
        assert_eq!(result["unmet"].as_array().expect("unmet array").len(), 1);
    }

    #[test]
    fn judge_with_atom_and_matching_catalog_reports_admitted() {
        let catalog = build_catalog(0);
        let atom = Atom8::new(PredicateId(1), 0, &[]);
        let payload = json!({
            "value": {"id": 1},
            "atom": atom,
            "catalog": catalog,
        });
        let result = judge_payload(&payload.to_string(), "default").expect("should judge");
        assert_eq!(result["prolog8"]["atom"], json!("admitted"));
    }

    #[test]
    fn judge_with_atom_missing_predicate_reports_rejection() {
        let catalog = build_catalog(0);
        let atom = Atom8::new(PredicateId(99), 0, &[]);
        let payload = json!({
            "value": {"id": 1},
            "atom": atom,
            "catalog": catalog,
        });
        let result = judge_payload(&payload.to_string(), "default").expect("should judge");
        let atom_result = result["prolog8"]["atom"].as_str().expect("string result");
        assert_ne!(atom_result, "admitted");
        assert!(atom_result.contains("not registered"));
    }

    #[test]
    fn judge_with_atom_but_no_catalog_is_error() {
        let atom = Atom8::new(PredicateId(1), 0, &[]);
        let payload = json!({"value": {"id": 1}, "atom": atom});
        assert!(judge_payload(&payload.to_string(), "default").is_err());
    }

    #[test]
    fn judge_with_invalid_json_is_error() {
        assert!(judge_payload("not json", "default").is_err());
    }

    // ── admit ───────────────────────────────────────────────────────────

    #[test]
    fn admit_succeeds_when_nothing_blocks() {
        let result = admit_payload(r#"{"value":{"id":1}}"#, "default").expect("should admit");
        assert_eq!(result["status"], json!("admitted"));
    }

    #[test]
    fn admit_is_denied_with_halted_andon_when_something_blocks() {
        let payload = r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
        let result = admit_payload(payload, "default").expect("should admit");
        assert_eq!(result["status"], json!("denied"));
        assert_eq!(result["verdict"], json!("halted"));
    }

    // ── receipt ─────────────────────────────────────────────────────────

    #[test]
    fn receipt_with_no_prev_hash_defaults_to_genesis() {
        let result = receipt_payload(r#"{"value":{"id":1}}"#).expect("should receipt");
        let chain_hash = result["chain_hash"].as_str().expect("chain_hash string");
        assert_eq!(chain_hash.len(), 64);
        let canonical = result["canonical"].as_str().expect("canonical string");
        assert!(canonical.starts_with("blake3:"));
        assert_eq!(result["prev_chain_hash"], json!("0".repeat(64)));
    }

    #[test]
    fn receipt_called_twice_with_same_prev_and_ts_ns_is_deterministic() {
        let payload = format!(
            r#"{{"value":{{"id":1}},"prev_chain_hash":"{}","ts_ns":42}}"#,
            "11".repeat(32)
        );
        let r1 = receipt_payload(&payload).expect("should receipt");
        let r2 = receipt_payload(&payload).expect("should receipt");
        assert_eq!(r1["chain_hash"], r2["chain_hash"]);
    }

    #[test]
    fn receipt_with_malformed_hex_is_error() {
        let payload = r#"{"value":{"id":1},"prev_chain_hash":"zz"}"#;
        assert!(receipt_payload(payload).is_err());
    }

    #[test]
    fn receipt_with_halting_obligation_returns_denied_shape() {
        let payload = r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
        let result = receipt_payload(payload).expect("should receipt");
        assert_eq!(result["status"], json!("denied"));
        assert_eq!(result["verdict"], json!("halted"));
    }

    // ── promote ─────────────────────────────────────────────────────────

    #[test]
    fn promote_from_lowest_rung_to_next_with_eligibility_false() {
        let result =
            promote_payload(r#"{"standing":"NAMED"}"#, "").expect("should promote");
        assert_eq!(result["status"], json!("promoted"));
        assert_eq!(result["from"], json!("NAMED"));
        assert_eq!(result["to"], json!("REGISTERED"));
        assert_eq!(result["is_partial_alive_eligible"], json!(false));
    }

    #[test]
    fn promote_to_a_rung_that_makes_it_eligible() {
        let result =
            promote_payload(r#"{"standing":"REGISTERED"}"#, "").expect("should promote");
        assert_eq!(result["to"], json!("DISPATCHABLE"));
        assert_eq!(result["is_partial_alive_eligible"], json!(true));
    }

    #[test]
    fn promote_to_rung_requiring_auditor_without_one_is_denied() {
        let result =
            promote_payload(r#"{"standing":"REFUSABLE"}"#, "").expect("should promote");
        assert_eq!(result["status"], json!("denied"));
        assert_eq!(result["to"], json!("REPLAYABLE"));
    }

    #[test]
    fn promote_to_that_rung_with_auditor_succeeds() {
        let result = promote_payload(r#"{"standing":"REFUSABLE"}"#, "alice").expect("should promote");
        assert_eq!(result["status"], json!("promoted"));
        assert_eq!(result["to"], json!("REPLAYABLE"));
        assert_eq!(result["auditor"], json!("alice"));
    }

    #[test]
    fn promote_from_top_rung_is_denied() {
        let result =
            promote_payload(r#"{"standing":"CERTIFIED"}"#, "").expect("should promote");
        assert_eq!(result["status"], json!("denied"));
    }

    #[test]
    fn promote_with_unparseable_standing_is_error() {
        assert!(promote_payload(r#"{"standing":"NOT_A_RUNG"}"#, "").is_err());
    }

    // ── show ────────────────────────────────────────────────────────────

    #[test]
    fn show_with_json_format() {
        let result = show(r#"{"id":1}"#.to_string(), "json".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn show_with_text_format() {
        let result = show(r#"{"id":1}"#.to_string(), "text".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn show_invalid_format() {
        let result = show(r#"{"id":1}"#.to_string(), "invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn show_invalid_json_is_error() {
        let result = show("not json".to_string(), "json".to_string());
        assert!(result.is_err());
    }
}
