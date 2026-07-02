//! Pure domain logic for the `law` verb family.
//!
//! This module holds every payload-shape type and pure `*_payload` function
//! used by [`crate::verbs::law`]. It has no dependency on `clap_noun_verb`
//! and returns plain `Result<Value, String>`, so it can be called from both
//! the CLI verbs and (in a later lane) an MCP server, with one source of
//! truth for judge/admit/receipt/show/promote semantics.
//!
//! Malformed input (bad JSON, bad hex, an unparseable standing name, an
//! `atom`/`rule` without a `catalog`) is a hard `Err`. A *domain* denial
//! (halted obligations, a prolog8 rejection, a missing auditor) is `Ok(json)`
//! with a `status`/`verdict` field describing the denial.

use std::time::{SystemTime, UNIX_EPOCH};

use praxis_core::{
    law::ReceiptMeta,
    lifecycle::{Raw, Validated},
    Admit, Andon, DefaultLaw, Judge, LawObject, Obligation, RefusalScenario,
};
use prolog8::{
    admit_atom, admit_rule, Atom8, Catalog, FactBlock8, Kernel, QueryAtom8, QueryResult,
    RejectionCode, Rule8,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use wasm4pm_cognition::breeds::standing::BreedStanding;

// ── Input schemas ─────────────────────────────────────────────────────────

/// Wire schema for an obligation, tagged by `type`. Distinct from
/// [`Obligation`]'s own (untagged-by-variant-name) `Serialize`/`Deserialize`
/// impl so callers can write `{"type": "blocking_constraint", ...}`.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ObligationInput {
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
pub struct LawInput {
    value: Value,
    #[serde(default)]
    obligations: Vec<ObligationInput>,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    satisfied_predicates: Vec<String>,
    #[serde(default)]
    atom: Option<Value>,
    /// A single `prolog8::Rule8`, shared by two independent checks: the
    /// legacy `admit_rule` validation-only check below (fires whenever
    /// `atom`/`rule` is present, regardless of `query`), and — when `query`
    /// is also present — loaded into the `Kernel` via `load_rule` before the
    /// query runs (see [`run_kernel_query`]).
    #[serde(default)]
    rule: Option<Value>,
    #[serde(default)]
    catalog: Option<Value>,
    /// A `prolog8::QueryAtom8`. When present, `judge`/`admit`/`receipt`
    /// build a real `prolog8::Kernel` (loading `facts` and `rule` first)
    /// and run `Kernel::query` against it — a proof-carrying execution
    /// path, additional to (not a replacement for) the validation-only
    /// `atom`/`rule` admission check above.
    #[serde(default)]
    query: Option<Value>,
    /// `prolog8::FactBlock8` JSON blocks loaded into the `Kernel` before
    /// `query` runs. Ignored unless `query` is present.
    #[serde(default)]
    facts: Vec<Value>,
    /// Opt-in per-payload flag for the lsp-max andon second gate ring
    /// ([`crate::law_andon`]). Only present/consulted when compiled with
    /// the (lightweight) `andon` Cargo feature; default `false` so this is
    /// fully backward compatible.
    #[cfg(feature = "andon")]
    #[serde(default)]
    andon_ring: bool,
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

/// Input schema for `verify-signature` (feature `law-signed` only).
#[cfg(feature = "law-signed")]
#[derive(Deserialize)]
struct VerifySignatureInput {
    /// The 64-hex-char BLAKE3 chain hash the signature is claimed to cover.
    chain_hash: String,
    /// The `signed_receipt` object as returned by `receipt` (chain_hash hex,
    /// base64 signature, verifying_key hex).
    signed_receipt: Value,
    /// Optional explicit verifying key (64 hex chars) to check authenticity
    /// against, instead of trusting the key embedded in `signed_receipt`.
    #[serde(default)]
    verifying_key: Option<String>,
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

/// Current time in milliseconds since the UNIX epoch (mirrors
/// `default_law::now_ms`, duplicated here since that one is private to
/// `praxis-core`).
fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

/// Extract the unmet-obligations list from an `Andon`, empty if not `Halted`.
fn unmet_from(andon: &Andon) -> Vec<Obligation> {
    match andon {
        Andon::Halted { unmet, .. } => unmet.clone(),
        _ => Vec::new(),
    }
}

/// Extract the refusal-taxonomy scenarios from an `Andon`, empty if not `Halted`.
fn refusals_from(andon: &Andon) -> Vec<RefusalScenario> {
    match andon {
        Andon::Halted { refusals, .. } => refusals.clone(),
        _ => Vec::new(),
    }
}

/// Deduplicated (order-preserving-per-first-sight) `RefusalCategory` names
/// for an andon's refusals, as a JSON array of strings.
fn refusal_categories_json(andon: &Andon) -> Value {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for r in refusals_from(andon) {
        let cat = r.category().to_string();
        if seen.insert(cat.clone()) {
            out.push(json!(cat));
        }
    }
    Value::Array(out)
}

/// Shared JSON fields describing a halted `Andon`: the raw `andon` value,
/// `unmet` obligations, and the derived `refusals`/`refusal_categories`.
/// Callers add their own `status`/`verdict` keys on top.
fn halted_fields(andon: &Andon) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("andon".to_string(), to_json(andon));
    m.insert("unmet".to_string(), to_json(&unmet_from(andon)));
    m.insert("refusals".to_string(), to_json(&refusals_from(andon)));
    m.insert("refusal_categories".to_string(), refusal_categories_json(andon));
    m
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
pub fn run_prolog8_checks(input: &LawInput) -> std::result::Result<Option<Value>, String> {
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
pub enum JudgeOutcome {
    Validated(LawObject<Value, Validated, DefaultLaw>),
    Halted(Andon),
}

/// Everything downstream verbs need after judging a payload.
pub struct JudgeContext {
    outcome: JudgeOutcome,
    /// Legacy `admit_atom`/`admit_rule` validation-only result (fires
    /// whenever `atom`/`rule` is present in the input, independent of
    /// `query`). Unchanged by the Kernel query path below.
    prolog8: Option<Value>,
    /// `{"verdict": "answered"|"denied"|"invalid", "result": ..}` from a
    /// real `prolog8::Kernel::query` run, when `query` was present. Carries
    /// the full `Decision` (bindings, `proof: Vec<ProofNode>`, `Receipt`)
    /// on every verdict, not just success.
    prolog8_query: Option<Value>,
    /// `{"status": .., "events": [..]}` from the optional lsp-max andon
    /// second gate ring, when compiled with the `andon` feature and the
    /// payload set `"andon_ring": true`.
    #[cfg(feature = "andon")]
    andon_ring: Option<Value>,
    /// The original `value` field, kept separately from the `LawObject`'s
    /// payload (which also carries `evidence`/`satisfied_predicates`) so
    /// callers can hash or display just the domain value.
    value: Value,
}

/// Parse a `LawInput`, run prolog8 checks (legacy admission-only, Kernel
/// query, and — if enabled — the andon ring), build a `LawObject`, and
/// judge it.
///
/// The final `Andon` is `Halted` if *either* an obligation was unmet *or*
/// any refusal was observed from the Kernel query / andon ring, even when
/// every `Obligation` was individually satisfied — a payload can be
/// obligation-clean and still be refused by a deeper check.
pub fn judge_from_value(value: &Value) -> std::result::Result<JudgeContext, String> {
    let input: LawInput =
        serde_json::from_value(value.clone()).map_err(|e| format!("invalid law payload: {e}"))?;
    let prolog8 = run_prolog8_checks(&input)?;
    let kernel_result = run_kernel_query(&input)?;
    let obligations = parse_obligations(input.obligations)?;
    let law_payload = json!({
        "value": input.value,
        "evidence": input.evidence,
        "satisfied_predicates": input.satisfied_predicates,
    });

    let raw = LawObject::<Value, Raw, DefaultLaw>::new(law_payload, obligations);

    // Run DefaultLaw's obligation judge first; extract its unmet
    // obligations + refusals without holding onto the (non-Clone)
    // `LawObject` past this match, since we may need to override its
    // verdict below (a Validated object with post-hoc kernel/andon
    // refusals must still end up Halted).
    let (obligation_unmet, obligation_refusals, maybe_validated) = match DefaultLaw::judge(raw) {
        Ok(validated) => (Vec::new(), Vec::new(), Some(validated)),
        Err(halted) => match halted.andon() {
            Andon::Halted { unmet, refusals, .. } => (unmet.clone(), refusals.clone(), None),
            _ => (Vec::new(), Vec::new(), None),
        },
    };

    let mut refusals = obligation_refusals;

    let mut prolog8_query = None;
    if let Some((kernel_json, kernel_refusals)) = kernel_result {
        prolog8_query = Some(kernel_json);
        refusals.extend(kernel_refusals);
    }

    #[cfg(feature = "andon")]
    let andon_ring = if input.andon_ring {
        let ring = crate::law_andon::AndonRing::standard();
        let (status, events) = ring.evaluate(&input.value);
        refusals.extend(crate::law_andon::ring_refusals(&events));
        Some(json!({"status": to_json(&status), "events": to_json(&events)}))
    } else {
        None
    };

    let outcome = if maybe_validated.is_none() || !refusals.is_empty() {
        JudgeOutcome::Halted(Andon::Halted { unmet: obligation_unmet, refusals, at: now_ms() })
    } else {
        JudgeOutcome::Validated(maybe_validated.expect("checked is_none() above"))
    };

    Ok(JudgeContext {
        outcome,
        prolog8,
        prolog8_query,
        #[cfg(feature = "andon")]
        andon_ring,
        value: input.value,
    })
}

/// A denied-pipeline response shared by `admit` and `receipt`.
pub fn denied_object(andon: &Andon) -> Value {
    let mut m = halted_fields(andon);
    m.insert("status".to_string(), json!("denied"));
    m.insert("verdict".to_string(), json!("halted"));
    Value::Object(m)
}

/// Run a prolog8 `Kernel::query` when `query` is present in `input`.
///
/// Builds a `Kernel` from `catalog`, loads every `facts` block (and `rule`,
/// if present — this is where 26.7.1's `StratifiedNegation` enforcement in
/// `admit_rule` fires for free, before the kernel ever sees the rule), then
/// runs `query`. Returns `Ok(None)` if `query` is absent.
///
/// A kernel-level `Denied`/`Invalid` outcome is `Ok(Some((json, refusals)))`
/// with non-empty `refusals` — that's a domain answer (a Kernel query result
/// carrying full proof + receipt), not a hard error. Malformed input (bad
/// JSON shapes, a `query` without a `catalog`, a rejected `facts`/`rule`
/// block) is a hard `Err`.
pub fn run_kernel_query(
    input: &LawInput,
) -> std::result::Result<Option<(Value, Vec<RefusalScenario>)>, String> {
    let Some(query_value) = &input.query else {
        return Ok(None);
    };

    let catalog_value =
        input.catalog.as_ref().ok_or_else(|| "query provided without a catalog".to_string())?;
    let catalog: Catalog =
        serde_json::from_value(catalog_value.clone()).map_err(|e| format!("invalid catalog: {e}"))?;
    let mut kernel = Kernel::new(catalog);

    for (i, fact_value) in input.facts.iter().enumerate() {
        let block: FactBlock8 = serde_json::from_value(fact_value.clone())
            .map_err(|e| format!("invalid facts[{i}]: {e}"))?;
        kernel.load_facts(block).map_err(|code| format!("facts[{i}] rejected: {code}"))?;
    }

    if let Some(rule_value) = &input.rule {
        let rule: Rule8 =
            serde_json::from_value(rule_value.clone()).map_err(|e| format!("invalid rule: {e}"))?;
        kernel.load_rule(rule).map_err(|code| format!("rule rejected by kernel: {code}"))?;
    }

    let q: QueryAtom8 =
        serde_json::from_value(query_value.clone()).map_err(|e| format!("invalid query: {e}"))?;

    let (verdict, result, refusals): (&'static str, Value, Vec<RefusalScenario>) =
        match kernel.query(&q) {
            QueryResult::Answered(decisions) => ("answered", to_json(&decisions), Vec::new()),
            QueryResult::Denied(decision) => {
                let refusal = RefusalScenario::KernelDenied { pred_id: q.atom.pred_id.0 };
                ("denied", to_json(&decision), vec![refusal])
            }
            QueryResult::Invalid(code) => {
                let refusal = RefusalScenario::KernelInvalid { rejection: code.to_string() };
                ("invalid", json!(code.to_string()), vec![refusal])
            }
        };

    Ok(Some((json!({"verdict": verdict, "result": result}), refusals)))
}

// ── Domain logic ──────────────────────────────────────────────────────────

/// Splice the shared kernel-query/andon-ring context sections (present on
/// every outcome, not just success) onto an already-built response object.
///
/// Takes owned `Option<Value>`s rather than `&JudgeContext` so callers can
/// extract them *before* moving `ctx.outcome` out of `ctx` (needed to hand
/// the non-`Clone` `LawObject` inside `JudgeOutcome::Validated` to
/// `DefaultLaw::admit`) — after a partial move of one field, the whole
/// struct can no longer be borrowed by reference.
fn with_query_context(
    mut obj: Value,
    prolog8: Option<Value>,
    prolog8_query: Option<Value>,
    andon_ring: Option<Value>,
) -> Value {
    if let Value::Object(m) = &mut obj {
        if let Some(p) = prolog8 {
            m.insert("prolog8".to_string(), p);
        }
        if let Some(p) = prolog8_query {
            m.insert("prolog8_query".to_string(), p);
        }
        if let Some(a) = andon_ring {
            m.insert("andon_ring".to_string(), a);
        }
    }
    obj
}

/// The `andon_ring` JSON section, cloned out of a `JudgeContext` before its
/// `outcome` field is moved. Always `None` when the `andon` feature is off
/// (the field doesn't exist on `JudgeContext` at all in that build).
fn andon_ring_of(_ctx: &JudgeContext) -> Option<Value> {
    #[cfg(feature = "andon")]
    {
        _ctx.andon_ring.clone()
    }
    #[cfg(not(feature = "andon"))]
    {
        None
    }
}

/// Domain logic for `judge`: Raw → Validated or Halted.
pub fn judge_payload(payload: &str, law: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let ctx = judge_from_value(&value)?;
    let (prolog8, prolog8_query, andon_ring) =
        (ctx.prolog8.clone(), ctx.prolog8_query.clone(), andon_ring_of(&ctx));

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
            for (k, v) in halted_fields(&andon) {
                obj.insert(k, v);
            }
        }
    }
    Ok(with_query_context(Value::Object(obj), prolog8, prolog8_query, andon_ring))
}

/// Domain logic for `admit`: Validated → Admitted (or denied).
pub fn admit_payload(payload: &str, policy: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let ctx = judge_from_value(&value)?;
    let (prolog8, prolog8_query, andon_ring) =
        (ctx.prolog8.clone(), ctx.prolog8_query.clone(), andon_ring_of(&ctx));

    let result = match ctx.outcome {
        JudgeOutcome::Halted(andon) => denied_object(&andon),
        JudgeOutcome::Validated(validated) => match DefaultLaw::admit(validated) {
            Ok(_admitted) => json!({
                "status": "admitted",
                "state": "Admitted",
                "policy": policy,
            }),
            Err(andon) => denied_object(&andon),
        },
    };
    Ok(with_query_context(result, prolog8, prolog8_query, andon_ring))
}

/// Domain logic for `receipt`: Admitted → Receipted, BLAKE3-chained (or denied).
pub fn receipt_payload(payload: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let fields: ReceiptFields =
        serde_json::from_value(value.clone()).map_err(|e| format!("invalid receipt fields: {e}"))?;
    let prev_chain_hash = match &fields.prev_chain_hash {
        Some(hex_str) => parse_hex32(hex_str)?,
        None => [0u8; 32],
    };

    let ctx = judge_from_value(&value)?;
    let (prolog8, prolog8_query, andon_ring) =
        (ctx.prolog8.clone(), ctx.prolog8_query.clone(), andon_ring_of(&ctx));
    let ctx_value = ctx.value.clone();

    let validated = match ctx.outcome {
        JudgeOutcome::Halted(andon) => {
            let denied = denied_object(&andon);
            return Ok(with_query_context(denied, prolog8, prolog8_query, andon_ring));
        }
        JudgeOutcome::Validated(validated) => validated,
    };
    let admitted = match DefaultLaw::admit(validated) {
        Ok(admitted) => admitted,
        Err(andon) => {
            let denied = denied_object(&andon);
            return Ok(with_query_context(denied, prolog8, prolog8_query, andon_ring));
        }
    };

    let ts_ns = fields.ts_ns.unwrap_or_else(|| {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64
    });
    // `denial` stays at its `ReceiptMeta::default()` value (`ADMITTED`):
    // reaching this point means `judge_from_value` observed zero refusals
    // (any refusal — obligation, Kernel, or andon-ring — forces
    // `JudgeOutcome::Halted` before admission is ever attempted, so there is
    // nothing non-`ADMITTED` to compose here). See `ReceiptMeta::denial`'s
    // doc comment for the (currently unreachable via this pipeline, but
    // supported by `receipt()` itself) non-`ADMITTED` case.
    let meta = ReceiptMeta {
        instruction_id: fields.instruction_id.unwrap_or(0),
        activity_idx: fields.activity_idx.unwrap_or(0),
        node_kind: fields.node_kind.unwrap_or(0),
        ts_ns: Some(ts_ns),
        ..Default::default()
    };

    let receipted = admitted.receipt(&prev_chain_hash, meta).map_err(|e| e.to_string())?;

    let chain_hash = receipted.chain_hash().copied().unwrap_or([0u8; 32]);
    let chain_hex = hex::encode(chain_hash);
    let payload_bytes = serde_json::to_vec(&ctx_value).map_err(|e| e.to_string())?;
    let payload_hash_hex = blake3::hash(&payload_bytes).to_hex().to_string();

    #[cfg_attr(not(feature = "law-signed"), allow(unused_mut))]
    let mut out = json!({
        "status": "receipted",
        "state": "Receipted",
        "chain_hash": chain_hex,
        "canonical": format!("blake3:{chain_hex}"),
        "prev_chain_hash": hex::encode(prev_chain_hash),
        "payload_hash": payload_hash_hex,
        "ts_ns": ts_ns,
    });

    // When the `law-signed` feature is on, `receipt()` (praxis-core, feature
    // `signed`) always populates `signature` with the serialized JSON bytes
    // of a `chatman_common::signed_receipt::SignedReceipt` (or the whole
    // pipeline already failed closed above via the `?`). Surface it so
    // callers don't have to know the internal byte encoding.
    #[cfg(feature = "law-signed")]
    if let Some(sig_bytes) = receipted_signature(&receipted) {
        if let Ok(signed_receipt) = serde_json::from_slice::<Value>(&sig_bytes) {
            if let Some(obj) = out.as_object_mut() {
                obj.insert("signature".to_string(), signed_receipt["signature"].clone());
                obj.insert("verifying_key".to_string(), signed_receipt["verifying_key"].clone());
                obj.insert("signed_receipt".to_string(), signed_receipt);
            }
        }
    }

    Ok(with_query_context(out, prolog8, prolog8_query, andon_ring))
}

/// Extract the raw signature bytes from a receipted `LawObject` (feature
/// `law-signed` only). Kept as a tiny free function so `receipt_payload`
/// doesn't need to know the field is `pub` on `LawObject` vs. some accessor.
#[cfg(feature = "law-signed")]
fn receipted_signature(
    receipted: &LawObject<Value, praxis_core::lifecycle::Receipted, DefaultLaw>,
) -> Option<Vec<u8>> {
    receipted.signature.clone()
}

/// Domain logic for `verify-signature` (feature `law-signed` only): check
/// that `signed_receipt` is a valid ed25519 signature over `chain_hash`.
///
/// Malformed input (bad hex, an unparseable `signed_receipt`) is a hard
/// `Err`; a signature that fails to verify is `Ok(json)` with
/// `"status": "invalid"` — the caller asked a legitimate question and got a
/// legitimate (negative) answer, so this is domain output, not an error.
#[cfg(feature = "law-signed")]
pub fn verify_signature_payload(payload: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let input: VerifySignatureInput = serde_json::from_value(value)
        .map_err(|e| format!("invalid verify-signature payload: {e}"))?;
    let chain_hash = parse_hex32(&input.chain_hash)?;
    let sig_bytes =
        serde_json::to_vec(&input.signed_receipt).map_err(|e| format!("invalid signed_receipt: {e}"))?;

    let result = match &input.verifying_key {
        Some(vk) => praxis_core::signing::verify_chain_hash_with_key(&chain_hash, &sig_bytes, vk),
        None => praxis_core::signing::verify_chain_hash(&chain_hash, &sig_bytes),
    };

    Ok(match result {
        Ok(()) => json!({
            "status": "valid",
            "chain_hash": input.chain_hash,
        }),
        Err(e) => json!({
            "status": "invalid",
            "chain_hash": input.chain_hash,
            "reason": e.to_string(),
        }),
    })
}

/// Domain logic for `inspect_obligation`: parse a JSON array of
/// wire-schema [`ObligationInput`] and describe what each one requires
/// before judgment, without running any judge/admit transition.
///
/// Shares the exact obligations wire schema `judge`/`admit`/`receipt` embed
/// under their `LawInput::obligations` field (`{"type": "precondition", ...}`
/// tagged JSON), so a caller can inspect a payload's obligations with the
/// same JSON it would otherwise pass inline. Malformed input (bad JSON, an
/// unrecognized `type`, bad `params_hash_hex`) is a hard `Err`.
pub fn inspect_obligations_payload(payload: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let raw: Vec<ObligationInput> =
        serde_json::from_value(value).map_err(|e| format!("invalid obligations: {e}"))?;
    let obligations = parse_obligations(raw)?;
    Ok(json!({
        "count": obligations.len(),
        "obligations": to_json(&obligations),
    }))
}

/// Domain logic for `show`: render a payload as json or text.
pub fn show_payload(payload: &str, format: &str) -> std::result::Result<Value, String> {
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
pub fn promote_payload(payload: &str, auditor: &str) -> std::result::Result<Value, String> {
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

// ── `receipt` noun: issue/validate/show/replay/export-ocel ─────────────────
//
// Distinct from `receipt_payload` above (which backs the `law receipt` verb
// and returns a one-shot JSON receipt without persisting it): these
// functions back the `receipt` noun's verbs, which persist `ReceiptRecord`s
// to an append-only JSONL ledger (`praxis_core::ReceiptStore`) and operate
// on that ledger (validate/show/replay/export-ocel). Both ultimately run the
// same judge -> admit -> receipt pipeline via `praxis_core`.

use praxis_core::{
    receipt_store::ReceiptStore,
    receipt_validator::{ReceiptValidator, SystemClock},
    replay_adapter, ReceiptRecord,
};

/// Run the judge -> admit -> receipt pipeline (same `LawInput`+`ReceiptFields`
/// wire shape as [`receipt_payload`]) and append the resulting
/// [`ReceiptRecord`] to the JSONL ledger at `dir`.
///
/// `prev_chain_hash` in the payload takes precedence; otherwise the ledger's
/// own `last_chain_hash()` is used, so callers issuing a sequence of
/// receipts don't need to track the running chain hash themselves.
pub fn receipt_issue_payload(payload: &str, dir: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let fields: ReceiptFields =
        serde_json::from_value(value.clone()).map_err(|e| format!("invalid receipt fields: {e}"))?;

    let store = ReceiptStore::open(dir).map_err(|e| e.to_string())?;
    let prev_chain_hash = match &fields.prev_chain_hash {
        Some(hex_str) => parse_hex32(hex_str)?,
        None => store.last_chain_hash().map_err(|e| e.to_string())?,
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
        ..Default::default()
    };

    let (_receipted, record) =
        admitted.receipt_with_record(&prev_chain_hash, meta).map_err(|e| e.to_string())?;
    store.append(&record).map_err(|e| e.to_string())?;

    Ok(json!({
        "status": "issued",
        "record": to_json(&record),
    }))
}

/// Copy every record's canonical JSON to
/// `data/validated_receipts/receipt-<instruction_id>-<chain_hash[..8]>.json`.
/// Called by [`receipt_validate_payload`] only after the ledger validates
/// clean; failures here are surfaced as a non-fatal `archive_warning` field
/// rather than failing the whole `validate` call (the validation verdict
/// itself already succeeded).
fn archive_validated_records(records: &[ReceiptRecord]) -> std::result::Result<(), String> {
    let dir = std::path::Path::new("data/validated_receipts");
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    for record in records {
        let short_hash = &record.chain_hash_hex[..record.chain_hash_hex.len().min(8)];
        let file = dir.join(format!("receipt-{}-{short_hash}.json", record.instruction_id));
        let bytes = serde_json::to_vec_pretty(record).map_err(|e| e.to_string())?;
        std::fs::write(file, bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Load the full ledger at `dir` and run [`ReceiptValidator::validate`]
/// against it (schema, chain-tamper recompute, chain linkage, monotonicity,
/// and POWL token-replay conformance), using [`SystemClock`] for the
/// monotonic stage's "not in the future" check.
///
/// On a clean verdict, also archives the records to `data/validated_receipts/`.
pub fn receipt_validate_payload(dir: &str) -> std::result::Result<Value, String> {
    let store = ReceiptStore::open(dir).map_err(|e| e.to_string())?;
    let records = store.load_all().map_err(|e| e.to_string())?;
    let verdict = ReceiptValidator::validate(&records, &SystemClock);

    if verdict.ok {
        if let Err(archive_warning) = archive_validated_records(&records) {
            return Ok(json!({ "verdict": to_json(&verdict), "archive_warning": archive_warning }));
        }
    }
    Ok(json!({ "verdict": to_json(&verdict) }))
}

/// Show the trailing `last` receipts in the ledger at `dir` (all of them, if
/// `last == 0`).
pub fn receipt_show_payload(dir: &str, last: usize) -> std::result::Result<Value, String> {
    let store = ReceiptStore::open(dir).map_err(|e| e.to_string())?;
    let records = store.load_all().map_err(|e| e.to_string())?;
    let total = records.len();
    let tail: Vec<&ReceiptRecord> = if last == 0 || last >= total {
        records.iter().collect()
    } else {
        records[total - last..].iter().collect()
    };
    Ok(json!({
        "total": total,
        "shown": tail.len(),
        "records": to_json(&tail),
    }))
}

/// Replay every receipt in the ledger at `dir` through the fixed
/// judge->admit->receipt POWL lifecycle model
/// ([`praxis_core::replay_adapter::replay_receipt_lifecycle`]) and report
/// per-record conformance metrics (fitness/precision rendered as `f64` in
/// `[0.0, 1.0]`, decoded from the underlying Q16.16 fixed-point values).
pub fn receipt_replay_payload(dir: &str) -> std::result::Result<Value, String> {
    const Q16_16_ONE: f64 = 65536.0;

    let store = ReceiptStore::open(dir).map_err(|e| e.to_string())?;
    let records = store.load_all().map_err(|e| e.to_string())?;

    let results: Vec<Value> = records
        .iter()
        .map(|record| match replay_adapter::replay_receipt_lifecycle(record) {
            Ok(metrics) => json!({
                "instruction_id": record.instruction_id,
                "chain_hash": record.chain_hash_hex,
                "fitness": metrics.fitness as f64 / Q16_16_ONE,
                "precision": metrics.precision as f64 / Q16_16_ONE,
            }),
            Err(violation) => json!({
                "instruction_id": record.instruction_id,
                "chain_hash": record.chain_hash_hex,
                "violation": format!("{violation:?}"),
            }),
        })
        .collect();

    Ok(json!({
        "records_replayed": records.len(),
        "results": results,
    }))
}

/// Export the full ledger at `dir` as an OCEL 2.0 event log
/// ([`praxis_core::ocel_export::to_ocel`]). If `out` is `Some`, also writes
/// the pretty-printed OCEL JSON to that path.
pub fn receipt_export_ocel_payload(
    dir: &str,
    out: Option<&str>,
) -> std::result::Result<Value, String> {
    let store = ReceiptStore::open(dir).map_err(|e| e.to_string())?;
    let records = store.load_all().map_err(|e| e.to_string())?;
    let ocel = praxis_core::ocel_export::to_ocel(&records);
    let ocel_json = serde_json::to_value(&ocel).map_err(|e| e.to_string())?;

    if let Some(path) = out {
        let bytes = serde_json::to_vec_pretty(&ocel).map_err(|e| e.to_string())?;
        std::fs::write(path, bytes).map_err(|e| e.to_string())?;
    }

    Ok(json!({
        "status": "exported",
        "event_count": ocel.events.len(),
        "object_count": ocel.objects.len(),
        "out": out,
        "ocel": ocel_json,
    }))
}

#[cfg(test)]
mod tests {
    use prolog8::{
        types::PlanId, CatalogId, EpochId, FactBlock8, FactRow8, FeatureBit, PredicateId,
        PredicateMeta, PredicateProofPolicy, ProofMode, QueryAtom8, Rule8, RuleId, SourceId,
        TermId,
    };

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

    // ── kernel query ──────────────────────────────────────────────────────

    /// Catalog with `candidate`/`excluded`/`eligible` (all arity 1) plus
    /// two interned terms `a`/`b`. Shared by the NAF tests below.
    fn naf_catalog() -> (Catalog, TermId, TermId) {
        let mut catalog = Catalog::new(CatalogId(1));
        for (id, label) in [(1u32, "candidate"), (2, "excluded"), (3, "eligible")] {
            catalog.add_predicate(PredicateMeta {
                pred_id: PredicateId(id),
                label: label.to_string(),
                arity: 1,
                access_orders: vec![],
                proof_policy: PredicateProofPolicy::OnRequest,
                materialized: false,
            });
        }
        let a = catalog.intern_term("a");
        let b = catalog.intern_term("b");
        (catalog, a, b)
    }

    /// `eligible(?0) :- candidate(?0), NOT excluded(?0)`, with (`with_feature
    /// = true`) or without the `StratifiedNegation` feature bit its
    /// `negation_mask` requires.
    fn naf_rule(with_feature: bool) -> Rule8 {
        const VAR_BASE: u32 = 0x8000_0000;
        let v0 = TermId(VAR_BASE);
        let head = Atom8::new(PredicateId(3), 1, &[v0]);
        let body0 = Atom8::new(PredicateId(1), 1, &[v0]); // candidate(?0)
        let body1 = Atom8::new(PredicateId(2), 1, &[v0]); // excluded(?0), negated
        let mut body = [Atom8::new(PredicateId(0), 0, &[]); 8];
        body[0] = body0;
        body[1] = body1;
        let feature_mask = if with_feature {
            FeatureBit::Facts.mask() | FeatureBit::HornRules.mask() | FeatureBit::StratifiedNegation.mask()
        } else {
            FeatureBit::Facts.mask() | FeatureBit::HornRules.mask()
        };
        Rule8 {
            rule_id: RuleId(1),
            head,
            body,
            body_len: 2,
            body_mask: 0b11,
            negation_mask: 0b10,
            builtin_mask: 0,
            var_count: 1,
            var_live_mask: 0b1,
            feature_mask,
            proof_mask: 0,
            plan_id: PlanId::default(),
        }
    }

    fn eligible_query(term: TermId) -> QueryAtom8 {
        let mut atom = Atom8::new(PredicateId(3), 1, &[term]);
        atom.binding_mask = 0b1;
        QueryAtom8 { atom, output_mask: 0, proof_mode: ProofMode::Both, epoch: EpochId(0) }
    }

    fn naf_payload(catalog: &Catalog, term: TermId) -> String {
        let candidate_facts =
            FactBlock8::new(PredicateId(1), 1, vec![FactRow8::new(PredicateId(1), 1, &[catalog.term_id("a").unwrap()], SourceId(0)), FactRow8::new(PredicateId(1), 1, &[catalog.term_id("b").unwrap()], SourceId(0))]);
        let excluded_facts = FactBlock8::new(
            PredicateId(2),
            1,
            vec![FactRow8::new(PredicateId(2), 1, &[catalog.term_id("b").unwrap()], SourceId(0))],
        );
        json!({
            "value": {"id": 1},
            "catalog": catalog,
            "facts": [candidate_facts, excluded_facts],
            "rule": naf_rule(true),
            "query": eligible_query(term),
        })
        .to_string()
    }

    #[test]
    fn naf_query_answers_for_non_excluded_and_denies_for_excluded() {
        let (catalog, a, b) = naf_catalog();

        let answered = judge_payload(&naf_payload(&catalog, a), "default").expect("should judge");
        assert_eq!(answered["prolog8_query"]["verdict"], json!("answered"));
        assert_eq!(answered["verdict"], json!("validated"));

        let denied = judge_payload(&naf_payload(&catalog, b), "default").expect("should judge");
        assert_eq!(denied["prolog8_query"]["verdict"], json!("denied"));
        assert_eq!(denied["verdict"], json!("halted"));
        let categories = denied["refusal_categories"].as_array().expect("categories array");
        assert!(categories.contains(&json!("authorization")));
    }

    #[test]
    fn naf_rule_without_stratified_negation_feature_is_rejected_by_kernel() {
        let (catalog, a, _b) = naf_catalog();
        let candidate_facts = FactBlock8::new(
            PredicateId(1),
            1,
            vec![FactRow8::new(PredicateId(1), 1, &[a], SourceId(0))],
        );
        let payload = json!({
            "value": {"id": 1},
            "catalog": catalog,
            "facts": [candidate_facts],
            "rule": naf_rule(false),
            "query": eligible_query(a),
        })
        .to_string();

        let err = judge_payload(&payload, "default").expect_err("unstratified negation must be a hard error");
        assert!(err.to_lowercase().contains("negation"), "error was: {err}");
    }

    #[test]
    fn kernel_invalid_query_maps_to_identity_category() {
        let catalog = build_catalog(1);
        // Predicate 99 isn't in the catalog at all -> Invalid(PredicateNotInCatalog).
        let mut atom = Atom8::new(PredicateId(99), 0, &[]);
        atom.binding_mask = 0;
        let query = QueryAtom8 { atom, output_mask: 0, proof_mode: ProofMode::Both, epoch: EpochId(0) };
        let payload = json!({"value": {"id": 1}, "catalog": catalog, "query": query}).to_string();

        let result = judge_payload(&payload, "default").expect("should judge");
        assert_eq!(result["verdict"], json!("halted"));
        assert_eq!(result["prolog8_query"]["verdict"], json!("invalid"));
        let categories = result["refusal_categories"].as_array().expect("categories array");
        assert!(categories.contains(&json!("identity")), "categories: {categories:?}");
    }

    #[test]
    fn kernel_query_without_catalog_is_hard_error() {
        let atom = Atom8::new(PredicateId(1), 0, &[]);
        let query = QueryAtom8 { atom, output_mask: 0, proof_mode: ProofMode::Both, epoch: EpochId(0) };
        let payload = json!({"value": {"id": 1}, "query": query}).to_string();
        assert!(judge_payload(&payload, "default").is_err());
    }

    #[cfg(feature = "andon")]
    #[test]
    fn andon_ring_blocks_admission_when_payload_has_no_receipt_or_checks() {
        let payload = json!({"value": {"id": 1}, "andon_ring": true}).to_string();
        let result = judge_payload(&payload, "default").expect("should judge");
        assert_eq!(result["verdict"], json!("halted"));
        let categories = result["refusal_categories"].as_array().expect("categories array");
        assert!(categories.contains(&json!("topology")), "categories: {categories:?}");
        assert!(result["andon_ring"]["status"].is_string());
    }

    #[cfg(feature = "andon")]
    #[test]
    fn andon_ring_off_by_default_even_when_compiled_in() {
        // Same payload shape as the blocking test above, minus `andon_ring`:
        // must validate cleanly, proving the feature is opt-in per payload.
        let payload = json!({"value": {"id": 1}}).to_string();
        let result = judge_payload(&payload, "default").expect("should judge");
        assert_eq!(result["verdict"], json!("validated"));
        assert!(result.get("andon_ring").is_none());
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

    /// Fixed 64-hex-char (32-byte) ed25519 seed used only by tests. Not
    /// security-sensitive: it exists so `receipt_payload` has a deterministic
    /// `PRAXIS_SIGNING_KEY` to sign against when built `--features law-signed`.
    #[cfg(feature = "law-signed")]
    const TEST_SIGNING_KEY_HEX: &str =
        "8bb5514c228cf4275a64aba09f3da77ef7de8b74a4424d670e71c26b0557e293";

    /// Set `PRAXIS_SIGNING_KEY` for the duration of the returned guard.
    ///
    /// `std::env` is process-global, so every test in this module that ends
    /// up calling `receipt_payload` under `--features law-signed` serializes
    /// on this lock rather than racing another test's env mutation.
    #[cfg(feature = "law-signed")]
    fn with_test_signing_key() -> std::sync::MutexGuard<'static, ()> {
        use std::sync::{Mutex, MutexGuard, OnceLock};
        fn env_lock() -> MutexGuard<'static, ()> {
            static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner())
        }
        let guard = env_lock();
        std::env::set_var("PRAXIS_SIGNING_KEY", TEST_SIGNING_KEY_HEX);
        guard
    }

    #[test]
    fn receipt_with_no_prev_hash_defaults_to_genesis() {
        #[cfg(feature = "law-signed")]
        let _guard = with_test_signing_key();
        let result = receipt_payload(r#"{"value":{"id":1}}"#).expect("should receipt");
        let chain_hash = result["chain_hash"].as_str().expect("chain_hash string");
        assert_eq!(chain_hash.len(), 64);
        let canonical = result["canonical"].as_str().expect("canonical string");
        assert!(canonical.starts_with("blake3:"));
        assert_eq!(result["prev_chain_hash"], json!("0".repeat(64)));
    }

    #[test]
    fn receipt_called_twice_with_same_prev_and_ts_ns_is_deterministic() {
        #[cfg(feature = "law-signed")]
        let _guard = with_test_signing_key();
        let payload = format!(
            r#"{{"value":{{"id":1}},"prev_chain_hash":"{}","ts_ns":42}}"#,
            "11".repeat(32)
        );
        let r1 = receipt_payload(&payload).expect("should receipt");
        let r2 = receipt_payload(&payload).expect("should receipt");
        assert_eq!(r1["chain_hash"], r2["chain_hash"]);
    }

    #[cfg(feature = "law-signed")]
    #[test]
    fn receipt_includes_signature_fields_when_signed() {
        let _guard = with_test_signing_key();
        let result = receipt_payload(r#"{"value":{"id":1}}"#).expect("should receipt");
        assert!(result["signature"].is_string(), "expected base64 signature field");
        assert!(result["verifying_key"].is_string(), "expected hex verifying_key field");
        assert!(result["signed_receipt"].is_object(), "expected signed_receipt object");
        assert_eq!(result["signed_receipt"]["chain_hash"], result["chain_hash"]);
    }

    #[cfg(feature = "law-signed")]
    #[test]
    fn verify_signature_round_trip_succeeds() {
        let _guard = with_test_signing_key();
        let result = receipt_payload(r#"{"value":{"id":1}}"#).expect("should receipt");
        let chain_hash = result["chain_hash"].clone();
        let signed_receipt = result["signed_receipt"].clone();
        let payload = json!({"chain_hash": chain_hash, "signed_receipt": signed_receipt}).to_string();
        let verdict = verify_signature_payload(&payload).expect("should verify");
        assert_eq!(verdict["status"], json!("valid"));
    }

    #[cfg(feature = "law-signed")]
    #[test]
    fn verify_signature_rejects_tampered_chain_hash() {
        let _guard = with_test_signing_key();
        let result = receipt_payload(r#"{"value":{"id":1}}"#).expect("should receipt");
        let signed_receipt = result["signed_receipt"].clone();
        let payload =
            json!({"chain_hash": "0".repeat(64), "signed_receipt": signed_receipt}).to_string();
        let verdict = verify_signature_payload(&payload).expect("should verify");
        assert_eq!(verdict["status"], json!("invalid"));
    }

    #[cfg(feature = "law-signed")]
    #[test]
    fn verify_signature_rejects_tampered_signature() {
        let _guard = with_test_signing_key();
        let result = receipt_payload(r#"{"value":{"id":1}}"#).expect("should receipt");
        let chain_hash = result["chain_hash"].clone();
        let mut signed_receipt = result["signed_receipt"].clone();
        let sig = signed_receipt["signature"].as_str().expect("signature string").to_string();
        let mut chars: Vec<char> = sig.chars().collect();
        let idx = chars.iter().position(|&c| c != 'A').unwrap_or(0);
        chars[idx] = if chars[idx] == 'B' { 'C' } else { 'B' };
        signed_receipt["signature"] = json!(chars.into_iter().collect::<String>());
        let payload = json!({"chain_hash": chain_hash, "signed_receipt": signed_receipt}).to_string();
        let verdict = verify_signature_payload(&payload).expect("should verify");
        assert_eq!(verdict["status"], json!("invalid"));
    }

    #[cfg(feature = "law-signed")]
    #[test]
    fn verify_signature_rejects_wrong_explicit_verifying_key() {
        let _guard = with_test_signing_key();
        let result = receipt_payload(r#"{"value":{"id":1}}"#).expect("should receipt");
        let chain_hash = result["chain_hash"].clone();
        let signed_receipt = result["signed_receipt"].clone();
        let payload = json!({
            "chain_hash": chain_hash,
            "signed_receipt": signed_receipt,
            "verifying_key": "0".repeat(64),
        })
        .to_string();
        let verdict = verify_signature_payload(&payload).expect("should verify");
        assert_eq!(verdict["status"], json!("invalid"));
    }

    #[cfg(feature = "law-signed")]
    #[test]
    fn verify_signature_with_malformed_hex_is_error() {
        let payload = json!({"chain_hash": "zz", "signed_receipt": {}}).to_string();
        assert!(verify_signature_payload(&payload).is_err());
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
        let result = show_payload(r#"{"id":1}"#, "json");
        assert!(result.is_ok());
    }

    #[test]
    fn show_with_text_format() {
        let result = show_payload(r#"{"id":1}"#, "text");
        assert!(result.is_ok());
    }

    #[test]
    fn show_invalid_format() {
        let result = show_payload(r#"{"id":1}"#, "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn show_invalid_json_is_error() {
        let result = show_payload("not json", "json");
        assert!(result.is_err());
    }

    // ── inspect_obligation ────────────────────────────────────────────────

    #[test]
    fn inspect_obligations_describes_each_kind() {
        let payload = json!([
            {"type": "precondition", "predicate_id": "p1"},
            {"type": "blocking_constraint", "reason": "stop"},
            {"type": "evidence_required", "evidence_type": "e1"},
        ])
        .to_string();
        let result = inspect_obligations_payload(&payload).expect("should inspect");
        assert_eq!(result["count"], json!(3));
        assert_eq!(result["obligations"].as_array().expect("array").len(), 3);
    }

    #[test]
    fn inspect_obligations_with_invalid_json_is_error() {
        assert!(inspect_obligations_payload("not json").is_err());
    }

    #[test]
    fn inspect_obligations_with_bad_hex_is_error() {
        let payload = json!([
            {"type": "precondition", "predicate_id": "p1", "params_hash_hex": "zz"},
        ])
        .to_string();
        assert!(inspect_obligations_payload(&payload).is_err());
    }

    // ── receipt noun: issue/validate/show/replay/export-ocel ─────────────

    /// Fixed 64-hex-char (32-byte) ed25519 seed used only by these tests.
    #[cfg(feature = "law-signed")]
    const RECEIPT_NOUN_SIGNING_KEY_HEX: &str =
        "1a2a3a4a5a6a7a8a9aaaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf11";

    #[cfg(feature = "law-signed")]
    fn with_receipt_noun_signing_key() -> std::sync::MutexGuard<'static, ()> {
        use std::sync::{Mutex, MutexGuard, OnceLock};
        fn env_lock() -> MutexGuard<'static, ()> {
            static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner())
        }
        let guard = env_lock();
        std::env::set_var("PRAXIS_SIGNING_KEY", RECEIPT_NOUN_SIGNING_KEY_HEX);
        guard
    }

    fn temp_receipts_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "praxis-ops-receipt-tests-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("create temp receipts dir");
        dir
    }

    #[test]
    fn receipt_issue_appends_to_ledger_and_chains() {
        #[cfg(feature = "law-signed")]
        let _guard = with_receipt_noun_signing_key();
        let dir = temp_receipts_dir();
        let dir_str = dir.to_string_lossy().to_string();

        let r1 = receipt_issue_payload(r#"{"value":{"id":1}}"#, &dir_str).expect("issue r1");
        assert_eq!(r1["status"], json!("issued"));
        let r1_chain = r1["record"]["chain_hash_hex"].as_str().expect("chain hash").to_string();
        assert_eq!(r1["record"]["prev_chain_hash_hex"], json!("0".repeat(64)));

        let r2 = receipt_issue_payload(r#"{"value":{"id":2}}"#, &dir_str).expect("issue r2");
        assert_eq!(r2["record"]["prev_chain_hash_hex"], json!(r1_chain));
    }

    #[test]
    fn receipt_issue_with_halting_obligation_is_denied_and_not_persisted() {
        let dir = temp_receipts_dir();
        let dir_str = dir.to_string_lossy().to_string();
        let payload = r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
        let result = receipt_issue_payload(payload, &dir_str).expect("should not hard-error");
        assert_eq!(result["status"], json!("denied"));

        let show = receipt_show_payload(&dir_str, 0).expect("show empty ledger");
        assert_eq!(show["total"], json!(0));
    }

    #[test]
    fn receipt_validate_on_clean_ledger_is_ok() {
        #[cfg(feature = "law-signed")]
        let _guard = with_receipt_noun_signing_key();
        let dir = temp_receipts_dir();
        let dir_str = dir.to_string_lossy().to_string();

        for i in 0..3 {
            // `instruction_id` must be given explicitly and strictly
            // increasing: `check_monotonic` rejects a ledger where it
            // isn't, and it defaults to 0 (not the loop index) when absent.
            let payload = format!(r#"{{"value":{{"id":{i}}},"instruction_id":{i}}}"#);
            receipt_issue_payload(&payload, &dir_str).expect("issue");
        }

        let verdict = receipt_validate_payload(&dir_str).expect("validate");
        assert_eq!(verdict["verdict"]["ok"], json!(true));
        assert_eq!(verdict["verdict"]["records_checked"], json!(3));
    }

    #[test]
    fn receipt_show_respects_last_n() {
        #[cfg(feature = "law-signed")]
        let _guard = with_receipt_noun_signing_key();
        let dir = temp_receipts_dir();
        let dir_str = dir.to_string_lossy().to_string();

        for i in 0..5 {
            let payload = format!(r#"{{"value":{{"id":{i}}}}}"#);
            receipt_issue_payload(&payload, &dir_str).expect("issue");
        }

        let all = receipt_show_payload(&dir_str, 0).expect("show all");
        assert_eq!(all["total"], json!(5));
        assert_eq!(all["shown"], json!(5));

        let last_two = receipt_show_payload(&dir_str, 2).expect("show last 2");
        assert_eq!(last_two["total"], json!(5));
        assert_eq!(last_two["shown"], json!(2));
    }

    #[test]
    fn receipt_replay_reports_fitness_one_for_a_lawful_ledger() {
        #[cfg(feature = "law-signed")]
        let _guard = with_receipt_noun_signing_key();
        let dir = temp_receipts_dir();
        let dir_str = dir.to_string_lossy().to_string();

        receipt_issue_payload(r#"{"value":{"id":1}}"#, &dir_str).expect("issue");
        let replay = receipt_replay_payload(&dir_str).expect("replay");
        assert_eq!(replay["records_replayed"], json!(1));
        let results = replay["results"].as_array().expect("results array");
        assert_eq!(results[0]["fitness"], json!(1.0));
    }

    #[test]
    fn receipt_export_ocel_writes_file_and_reports_counts() {
        #[cfg(feature = "law-signed")]
        let _guard = with_receipt_noun_signing_key();
        let dir = temp_receipts_dir();
        let dir_str = dir.to_string_lossy().to_string();

        receipt_issue_payload(r#"{"value":{"id":1}}"#, &dir_str).expect("issue");
        let out_path = dir.join("out.ocel.json");
        let out_str = out_path.to_string_lossy().to_string();
        let result =
            receipt_export_ocel_payload(&dir_str, Some(&out_str)).expect("export-ocel");
        assert_eq!(result["status"], json!("exported"));
        assert_eq!(result["event_count"], json!(1));
        assert!(out_path.exists());
    }
}
