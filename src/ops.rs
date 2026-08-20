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
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
            ObligationInput::Precondition {
                predicate_id,
                params_hash_hex,
            } => {
                let params_hash = match params_hash_hex {
                    Some(hex_str) => parse_hex32(&hex_str)?,
                    None => [0u8; 32],
                };
                Ok(Obligation::Precondition {
                    predicate_id,
                    params_hash,
                })
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
pub fn to_json<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

/// Parse a payload string into `T`. Empty or invalid JSON is a hard error.
///
/// Shared by the `plan` and `propose` verb families (both the CLI verbs and
/// the MCP tools) so there is one deserialization seam, not one per entry
/// point.
pub fn parse_payload<T: DeserializeOwned>(payload: &str) -> std::result::Result<T, String> {
    if payload.trim().is_empty() {
        return Err("empty payload".to_string());
    }
    serde_json::from_str(payload).map_err(|e| format!("invalid JSON: {e}"))
}

/// Current time in milliseconds since the UNIX epoch (mirrors
/// `default_law::now_ms`, duplicated here since that one is private to
/// `praxis-core`).
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
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
    m.insert(
        "refusal_categories".to_string(),
        refusal_categories_json(andon),
    );
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
    let catalog: Catalog = serde_json::from_value(catalog_value.clone())
        .map_err(|e| format!("invalid catalog: {e}"))?;

    let mut out = Map::new();
    if let Some(atom_value) = &input.atom {
        let atom: Atom8 =
            serde_json::from_value(atom_value.clone()).map_err(|e| format!("invalid atom: {e}"))?;
        out.insert(
            "atom".to_string(),
            admit_result_json(admit_atom(&atom, &catalog)),
        );
    }
    if let Some(rule_value) = &input.rule {
        let rule: Rule8 =
            serde_json::from_value(rule_value.clone()).map_err(|e| format!("invalid rule: {e}"))?;
        out.insert(
            "rule".to_string(),
            admit_result_json(admit_rule(&rule, &catalog)),
        );
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
            Andon::Halted {
                unmet, refusals, ..
            } => (unmet.clone(), refusals.clone(), None),
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

    let outcome = match maybe_validated {
        Some(validated) if refusals.is_empty() => JudgeOutcome::Validated(validated),
        _ => JudgeOutcome::Halted(Andon::Halted {
            unmet: obligation_unmet,
            refusals,
            at: now_ms(),
        }),
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

    let catalog_value = input
        .catalog
        .as_ref()
        .ok_or_else(|| "query provided without a catalog".to_string())?;
    let catalog: Catalog = serde_json::from_value(catalog_value.clone())
        .map_err(|e| format!("invalid catalog: {e}"))?;
    let mut kernel = Kernel::new(catalog);

    for (i, fact_value) in input.facts.iter().enumerate() {
        let block: FactBlock8 = serde_json::from_value(fact_value.clone())
            .map_err(|e| format!("invalid facts[{i}]: {e}"))?;
        kernel
            .load_facts(block)
            .map_err(|code| format!("facts[{i}] rejected: {code}"))?;
    }

    if let Some(rule_value) = &input.rule {
        let rule: Rule8 =
            serde_json::from_value(rule_value.clone()).map_err(|e| format!("invalid rule: {e}"))?;
        kernel
            .load_rule(rule)
            .map_err(|code| format!("rule rejected by kernel: {code}"))?;
    }

    let q: QueryAtom8 =
        serde_json::from_value(query_value.clone()).map_err(|e| format!("invalid query: {e}"))?;

    let (verdict, result, refusals): (&'static str, Value, Vec<RefusalScenario>) =
        match kernel.query(&q) {
            QueryResult::Answered(decisions) => ("answered", to_json(&decisions), Vec::new()),
            QueryResult::Denied(decision) => {
                let refusal = RefusalScenario::KernelDenied {
                    pred_id: q.atom.pred_id.0,
                };
                ("denied", to_json(&decision), vec![refusal])
            }
            QueryResult::Invalid(code) => {
                let refusal = RefusalScenario::KernelInvalid {
                    rejection: code.to_string(),
                };
                ("invalid", json!(code.to_string()), vec![refusal])
            }
        };

    Ok(Some((
        json!({"verdict": verdict, "result": result}),
        refusals,
    )))
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
    let (prolog8, prolog8_query, andon_ring) = (
        ctx.prolog8.clone(),
        ctx.prolog8_query.clone(),
        andon_ring_of(&ctx),
    );

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
    Ok(with_query_context(
        Value::Object(obj),
        prolog8,
        prolog8_query,
        andon_ring,
    ))
}

/// Domain logic for `admit`: Validated → Admitted (or denied).
pub fn admit_payload(payload: &str, policy: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let ctx = judge_from_value(&value)?;
    let (prolog8, prolog8_query, andon_ring) = (
        ctx.prolog8.clone(),
        ctx.prolog8_query.clone(),
        andon_ring_of(&ctx),
    );

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
    Ok(with_query_context(
        result,
        prolog8,
        prolog8_query,
        andon_ring,
    ))
}

/// Domain logic for `receipt`: Admitted → Receipted, BLAKE3-chained (or denied).
pub fn receipt_payload(payload: &str) -> std::result::Result<Value, String> {
    let value = parse_value(payload)?;
    let fields: ReceiptFields = serde_json::from_value(value.clone())
        .map_err(|e| format!("invalid receipt fields: {e}"))?;
    let prev_chain_hash = match &fields.prev_chain_hash {
        Some(hex_str) => parse_hex32(hex_str)?,
        None => [0u8; 32],
    };

    let ctx = judge_from_value(&value)?;
    let (prolog8, prolog8_query, andon_ring) = (
        ctx.prolog8.clone(),
        ctx.prolog8_query.clone(),
        andon_ring_of(&ctx),
    );
    let ctx_value = ctx.value.clone();

    let validated = match ctx.outcome {
        JudgeOutcome::Halted(andon) => {
            let denied = denied_object(&andon);
            return Ok(with_query_context(
                denied,
                prolog8,
                prolog8_query,
                andon_ring,
            ));
        }
        JudgeOutcome::Validated(validated) => validated,
    };
    let admitted = match DefaultLaw::admit(validated) {
        Ok(admitted) => admitted,
        Err(andon) => {
            let denied = denied_object(&andon);
            return Ok(with_query_context(
                denied,
                prolog8,
                prolog8_query,
                andon_ring,
            ));
        }
    };

    let ts_ns = fields.ts_ns.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
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

    let receipted = admitted
        .receipt(&prev_chain_hash, meta)
        .map_err(|e| e.to_string())?;

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
                obj.insert(
                    "verifying_key".to_string(),
                    signed_receipt["verifying_key"].clone(),
                );
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
    let sig_bytes = serde_json::to_vec(&input.signed_receipt)
        .map_err(|e| format!("invalid signed_receipt: {e}"))?;

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
    let fields: ReceiptFields = serde_json::from_value(value.clone())
        .map_err(|e| format!("invalid receipt fields: {e}"))?;

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
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    });
    let meta = ReceiptMeta {
        instruction_id: fields.instruction_id.unwrap_or(0),
        activity_idx: fields.activity_idx.unwrap_or(0),
        node_kind: fields.node_kind.unwrap_or(0),
        ts_ns: Some(ts_ns),
        ..Default::default()
    };

    let (_receipted, record) = admitted
        .receipt_with_record(&prev_chain_hash, meta)
        .map_err(|e| e.to_string())?;
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
        let file = dir.join(format!(
            "receipt-{}-{short_hash}.json",
            record.instruction_id
        ));
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
        .map(
            |record| match replay_adapter::replay_receipt_lifecycle(record) {
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
            },
        )
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

// ═══════════════════════════════════════════════════════════════════════════
// PLAN SOLVE (Day 2 pipe): PDDL8 classical/temporal solving.
//
// Promoted here from `verbs::plan` so the single `plan solve` implementation is
// callable from both the CLI verb and the MCP `plan_solve` tool (AR-2:
// one implementation, no drift). The route/analyze/execute verbs stay in
// `verbs::plan` but reuse the shared helpers below, so there is still exactly
// one copy of `resolve_pddl_source`/`is_infeasible`/`refusal_json`/etc.
// ═══════════════════════════════════════════════════════════════════════════

use bcinr_pddl::{
    compute_plan_chain, domain_from_pddl, problem_from_pddl, GroundProblem, GroundTemporalProblem,
    Pddl8Error,
};

/// Read a file to a string, mapping IO errors to a hard-error message.
pub fn read_file(path: &str) -> std::result::Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))
}

/// A finite `f64` as a JSON number; a non-finite one (`INFINITY`, `NAN`) as
/// JSON `null` — `serde_json` cannot represent non-finite floats.
pub fn finite_or_null(v: f64) -> Value {
    if v.is_finite() {
        json!(v)
    } else {
        Value::Null
    }
}

/// Marker splitting a combined single-file domain+problem PDDL text.
const PROBLEM_MARKER: &str = "(define (problem";

/// Split `text` at the last `(define (problem` occurrence — the last, not the
/// first, so an illustrative marker inside a leading comment block doesn't get
/// mistaken for the real split point.
pub fn split_combined(text: &str) -> std::result::Result<(String, String), String> {
    match text.rfind(PROBLEM_MARKER) {
        Some(0) => Err(format!(
            "combined PDDL text starts with `{PROBLEM_MARKER}` — no domain text precedes it"
        )),
        Some(idx) => Ok((text[..idx].to_string(), text[idx..].to_string())),
        None => Err(format!(
            "combined PDDL text has no `{PROBLEM_MARKER}` block; supply domain and problem separately"
        )),
    }
}

/// Resolve a domain/problem PDDL source from any combination of inline text and
/// file paths. If exactly one of `{domain-ish, problem-ish}` is given, its text
/// is treated as a single combined file and split via [`split_combined`].
pub fn resolve_pddl_source(
    domain: Option<String>,
    problem: Option<String>,
    domain_file: Option<String>,
    problem_file: Option<String>,
) -> std::result::Result<(String, String), String> {
    if domain.is_some() && domain_file.is_some() {
        return Err("cannot supply both `domain` and `domain_file`".to_string());
    }
    if problem.is_some() && problem_file.is_some() {
        return Err("cannot supply both `problem` and `problem_file`".to_string());
    }
    let domain_text = match domain {
        Some(t) => Some(t),
        None => match domain_file {
            Some(p) => Some(read_file(&p)?),
            None => None,
        },
    };
    let problem_text = match problem {
        Some(t) => Some(t),
        None => match problem_file {
            Some(p) => Some(read_file(&p)?),
            None => None,
        },
    };
    match (domain_text, problem_text) {
        (Some(d), Some(p)) => Ok((d, p)),
        (Some(combined), None) | (None, Some(combined)) => split_combined(&combined),
        (None, None) => Err(
            "must supply `domain`/`domain_file` and `problem`/`problem_file`, or \
                             a single combined text/file containing both"
                .to_string(),
        ),
    }
}

/// Whether a `Pddl8Error` represents domain *infeasibility* (a legitimate "no"
/// answer) rather than malformed input or an internal bug.
pub fn is_infeasible(e: &Pddl8Error) -> bool {
    matches!(
        e,
        Pddl8Error::EmptyGrounding
            | Pddl8Error::NoAdmittedPlan
            | Pddl8Error::StepDenied { .. }
            | Pddl8Error::GoalNotReached
            // `find_plan`/`find_temporal_plan` now return
            // `PlannerOutcome<T>` (bcinr_mfw_ir); `.into_result()?` wraps
            // any non-`Found` outcome in `PlanningFailed` (see that
            // variant's doc comment) rather than collapsing straight to
            // `NoAdmittedPlan` as the old local `PlannerOutcome` did.
            // Restored 2026-08-19 alongside crates/pddl-index.
            | Pddl8Error::PlanningFailed(_)
    )
}

/// A structured refusal (`admitted: false`) for a domain-infeasible PDDL result.
pub fn refusal_json(mode: &str, e: &Pddl8Error) -> Value {
    json!({ "mode": mode, "admitted": false, "refusal_reason": e.to_string() })
}

fn default_mode() -> String {
    "classical".to_string()
}

/// Wire schema for `plan solve` — inline PDDL text and/or file paths plus mode.
#[derive(Deserialize)]
struct SolveInput {
    domain: Option<String>,
    problem: Option<String>,
    domain_file: Option<String>,
    problem_file: Option<String>,
    #[serde(default = "default_mode")]
    mode: String,
}

fn solve_classical(domain_text: &str, problem_text: &str) -> std::result::Result<Value, String> {
    let domain = domain_from_pddl(domain_text).map_err(|e| e.to_string())?;
    let problem = problem_from_pddl(problem_text).map_err(|e| e.to_string())?;
    // Auto-select the grounder: a domain whose naive Cartesian product blows
    // past `GROUND_INDEX_THRESHOLD` is grounded lazily (dictionary-encoded
    // join, materializing only reachable actions); small domains keep the
    // simpler bcinr BFS grounder. Both find the identical plan.
    if pddl_index::should_use_indexed(&domain, &problem) {
        return solve_classical_indexed(&domain, &problem);
    }
    let ground = match GroundProblem::build(&domain, &problem, None) {
        Ok(g) => g,
        Err(e) if is_infeasible(&e) => return Ok(refusal_json("classical", &e)),
        Err(e) => return Err(e.to_string()),
    };
    match ground.find_plan().into_result() {
        Ok(tape) => Ok(json!({
            "mode": "classical",
            "grounder": "naive",
            "admitted": true,
            "plan_len": tape.len(),
            "plan": to_json(&tape),
        })),
        Err(failure) => {
            let e: Pddl8Error = failure.into();
            if is_infeasible(&e) {
                Ok(refusal_json("classical", &e))
            } else {
                Err(e.to_string())
            }
        }
    }
}

/// Classify a `pddl_index::GroundError` as domain infeasibility versus a hard
/// error — mirrors [`is_infeasible`] for the indexed path.
fn indexed_refusal(e: &pddl_index::GroundError) -> Option<Value> {
    use pddl_index::GroundError::{BoundExceeded, EmptyGrounding, NoAdmittedPlan};
    match e {
        EmptyGrounding | NoAdmittedPlan => Some(json!({
            "mode": "classical",
            "grounder": "indexed",
            "admitted": false,
            "refusal_reason": e.to_string(),
        })),
        BoundExceeded { .. } => None,
    }
}

/// Lazy dictionary-encoded grounding path. Reports the same fields as the naive
/// path plus a `grounder: "indexed"` tag and the `grounding` savings stats.
fn solve_classical_indexed(
    domain: &bcinr_pddl::Pddl8Domain,
    problem: &bcinr_pddl::Pddl8Problem,
) -> std::result::Result<Value, String> {
    let gp = match pddl_index::IndexedGroundProblem::build(domain, problem, None) {
        Ok(g) => g,
        Err(e) => return indexed_refusal(&e).ok_or_else(|| e.to_string()),
    };
    let stats = gp.stats();
    let grounding = json!({
        "candidate_groundings": stats.candidate_groundings,
        "materialized_groundings": stats.materialized_groundings,
        "reachable_atoms": stats.reachable_atoms,
        "materialization_ratio": finite_or_null(stats.materialization_ratio()),
    });
    match gp.find_plan() {
        Ok(tape) => Ok(json!({
            "mode": "classical",
            "grounder": "indexed",
            "admitted": true,
            "plan_len": tape.len(),
            "plan": to_json(&tape),
            "grounding": grounding,
        })),
        Err(e) => indexed_refusal(&e)
            .map(|mut v| {
                v["grounding"] = grounding;
                v
            })
            .ok_or_else(|| e.to_string()),
    }
}

fn solve_temporal(domain_text: &str, problem_text: &str) -> std::result::Result<Value, String> {
    let domain = domain_from_pddl(domain_text).map_err(|e| e.to_string())?;
    let problem = problem_from_pddl(problem_text).map_err(|e| e.to_string())?;
    let gtp = match GroundTemporalProblem::build(&domain, &problem) {
        Ok(g) => g,
        Err(e) if is_infeasible(&e) => return Ok(refusal_json("temporal", &e)),
        Err(e) => return Err(e.to_string()),
    };
    match gtp.find_temporal_plan().into_result() {
        Ok(plan) => {
            let plan_chain = compute_plan_chain(&plan.steps);
            Ok(json!({
                "mode": "temporal",
                "admitted": true,
                "makespan": finite_or_null(plan.makespan),
                "plan_chain": plan_chain,
                "plan": to_json(&plan),
            }))
        }
        Err(failure) => {
            let e: Pddl8Error = failure.into();
            if is_infeasible(&e) {
                Ok(refusal_json("temporal", &e))
            } else {
                Err(e.to_string())
            }
        }
    }
}

/// Solve a classical (`GroundProblem::find_plan`, BFS) or temporal
/// (`GroundTemporalProblem::find_temporal_plan`) PDDL8 problem.
///
/// `domain`/`problem` accept inline PDDL text, `domain_file`/`problem_file`
/// accept paths; if only one side is given, its text is treated as a single
/// combined domain+problem file and split at the last `(define (problem`
/// occurrence. Malformed input is a hard `Err`; domain infeasibility is
/// `Ok(json)` with `"admitted": false` and a `refusal_reason`.
///
/// This is the single `plan solve` implementation shared by the CLI verb
/// (`verbs::plan::solve`) and the MCP `plan_solve` tool.
pub fn plan_solve_payload(payload: &str) -> std::result::Result<Value, String> {
    let input: SolveInput = parse_payload(payload)?;
    let (domain_text, problem_text) = resolve_pddl_source(
        input.domain,
        input.problem,
        input.domain_file,
        input.problem_file,
    )?;
    match input.mode.as_str() {
        "classical" => solve_classical(&domain_text, &problem_text),
        "temporal" => solve_temporal(&domain_text, &problem_text),
        other => Err(format!(
            "unknown mode `{other}` (expected `classical` or `temporal`)"
        )),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PROPOSE (Day 2 pipe): observe → rank candidate goal states (feature proposer).
//
// Promoted here from `verbs::propose` (`revenue`/`goal`) so the same ranking
// implementation backs the CLI verb and the MCP `propose_revenue`/
// `propose_goal` tools. Output is proposal (O), never authority (O*) — AR-9.
// ═══════════════════════════════════════════════════════════════════════════

/// Wire schema shared by `propose revenue` and `propose goal`.
#[cfg(feature = "proposer")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposeInput {
    state: praxis_proposer::RevenueState,
    #[serde(default)]
    objective: Option<Value>,
    #[serde(default)]
    objective_file: Option<String>,
}

/// Resolve the authored objective from exactly one of: the `objective_path`
/// argument, the payload's `objective_file` path, or the payload's inline
/// `objective` object. None ⇒ hard error (Non-goal 1: the system never invents
/// values); more than one ⇒ hard error (ambiguous authorship).
#[cfg(feature = "proposer")]
fn resolve_objective(
    arg_path: &str,
    input: &ProposeInput,
) -> std::result::Result<praxis_proposer::ObjectiveFunction, String> {
    let mut sources = 0;
    if !arg_path.is_empty() {
        sources += 1;
    }
    if input.objective.is_some() {
        sources += 1;
    }
    if input.objective_file.is_some() {
        sources += 1;
    }
    match sources {
        0 => {
            return Err(
                "no objective supplied: pass --objective <path>, or put `objective` \
                        (inline) or `objective_file` (path) in the payload — the objective \
                        function is domain-authored data the system never invents (Non-goal 1)"
                    .to_string(),
            )
        }
        1 => {}
        _ => {
            return Err("multiple objective sources supplied: use exactly one of \
                        --objective, payload `objective`, payload `objective_file`"
                .to_string())
        }
    }
    if let Some(inline) = &input.objective {
        return praxis_proposer::ObjectiveFunction::from_json_str(&inline.to_string())
            .map_err(|e| e.to_string());
    }
    let path = if !arg_path.is_empty() {
        arg_path
    } else {
        input.objective_file.as_deref().unwrap_or_default()
    };
    praxis_proposer::ObjectiveFunction::from_path(std::path::Path::new(path))
        .map_err(|e| e.to_string())
}

/// A [`praxis_proposer::Proposal`] as JSON, with the derived `pddl_goal` atom
/// attached so a caller can splice it into a PDDL problem `(:goal ...)` block
/// without re-deriving it.
#[cfg(feature = "proposer")]
fn proposal_json(p: &praxis_proposer::Proposal) -> Value {
    let mut v = serde_json::to_value(p).unwrap_or(Value::Null);
    if let Value::Object(map) = &mut v {
        map.insert("pddl_goal".to_string(), json!(p.pddl_goal()));
    }
    v
}

/// `{name, version}` summary of an objective function.
#[cfg(feature = "proposer")]
pub fn objective_summary(obj: &praxis_proposer::ObjectiveFunction) -> Value {
    json!({ "name": obj.name, "version": obj.version })
}

/// Enumerate, score, and rank candidate goal states for a revenue snapshot.
///
/// Returns the full ranked proposal list. `objective_path` is the CLI
/// `--objective` argument (empty string when unset); the objective may instead
/// arrive inline or as a path inside `payload`. Output is observation (O),
/// never authority (O*) — AR-9.
#[cfg(feature = "proposer")]
pub fn propose_revenue_payload(
    payload: &str,
    objective_path: &str,
) -> std::result::Result<Value, String> {
    let input: ProposeInput = parse_payload(payload)?;
    let objective = resolve_objective(objective_path, &input)?;
    let proposer = praxis_proposer::Proposer::new(objective);
    let proposals = proposer.propose(&input.state);
    let status = if proposals.is_empty() {
        "no_lawful_candidates"
    } else {
        "proposed"
    };
    Ok(json!({
        "status": status,
        "objective": objective_summary(proposer.objective()),
        "count": proposals.len(),
        "proposals": proposals.iter().map(proposal_json).collect::<Vec<_>>(),
    }))
}

/// Emit only the top-ranked proposal's PDDL goal atom (plus its hash and
/// rationale), ready to splice into a problem `(:goal ...)` block for
/// `plan_solve`. A state with no lawful candidates is a domain "no"
/// (`Ok(json)` with `"status": "no_lawful_candidates"`), not an error.
#[cfg(feature = "proposer")]
pub fn propose_goal_payload(
    payload: &str,
    objective_path: &str,
) -> std::result::Result<Value, String> {
    let input: ProposeInput = parse_payload(payload)?;
    let objective = resolve_objective(objective_path, &input)?;
    let proposer = praxis_proposer::Proposer::new(objective);
    let proposals = proposer.propose(&input.state);
    let Some(top) = proposals.first() else {
        return Ok(json!({
            "status": "no_lawful_candidates",
            "objective": objective_summary(proposer.objective()),
        }));
    };
    Ok(json!({
        "status": "proposed",
        "objective": objective_summary(proposer.objective()),
        "goal": top.pddl_goal(),
        "goal_description": top.goal_description,
        "target_account": top.target_account,
        "target_stage": top.target_stage,
        "score": top.score,
        "proposal_hash": top.proposal_hash,
        "rationale": top.rationale,
        "candidates_considered": proposals.len(),
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
        let payload =
            r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
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
            FeatureBit::Facts.mask()
                | FeatureBit::HornRules.mask()
                | FeatureBit::StratifiedNegation.mask()
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
        QueryAtom8 {
            atom,
            output_mask: 0,
            proof_mode: ProofMode::Both,
            epoch: EpochId(0),
        }
    }

    fn naf_payload(catalog: &Catalog, term: TermId) -> String {
        let candidate_facts = FactBlock8::new(
            PredicateId(1),
            1,
            vec![
                FactRow8::new(
                    PredicateId(1),
                    1,
                    &[catalog.term_id("a").unwrap()],
                    SourceId(0),
                ),
                FactRow8::new(
                    PredicateId(1),
                    1,
                    &[catalog.term_id("b").unwrap()],
                    SourceId(0),
                ),
            ],
        );
        let excluded_facts = FactBlock8::new(
            PredicateId(2),
            1,
            vec![FactRow8::new(
                PredicateId(2),
                1,
                &[catalog.term_id("b").unwrap()],
                SourceId(0),
            )],
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
        let categories = denied["refusal_categories"]
            .as_array()
            .expect("categories array");
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

        let err = judge_payload(&payload, "default")
            .expect_err("unstratified negation must be a hard error");
        assert!(err.to_lowercase().contains("negation"), "error was: {err}");
    }

    #[test]
    fn kernel_invalid_query_maps_to_identity_category() {
        let catalog = build_catalog(1);
        // Predicate 99 isn't in the catalog at all -> Invalid(PredicateNotInCatalog).
        let mut atom = Atom8::new(PredicateId(99), 0, &[]);
        atom.binding_mask = 0;
        let query = QueryAtom8 {
            atom,
            output_mask: 0,
            proof_mode: ProofMode::Both,
            epoch: EpochId(0),
        };
        let payload = json!({"value": {"id": 1}, "catalog": catalog, "query": query}).to_string();

        let result = judge_payload(&payload, "default").expect("should judge");
        assert_eq!(result["verdict"], json!("halted"));
        assert_eq!(result["prolog8_query"]["verdict"], json!("invalid"));
        let categories = result["refusal_categories"]
            .as_array()
            .expect("categories array");
        assert!(
            categories.contains(&json!("identity")),
            "categories: {categories:?}"
        );
    }

    #[test]
    fn kernel_query_without_catalog_is_hard_error() {
        let atom = Atom8::new(PredicateId(1), 0, &[]);
        let query = QueryAtom8 {
            atom,
            output_mask: 0,
            proof_mode: ProofMode::Both,
            epoch: EpochId(0),
        };
        let payload = json!({"value": {"id": 1}, "query": query}).to_string();
        assert!(judge_payload(&payload, "default").is_err());
    }

    #[cfg(feature = "andon")]
    #[test]
    fn andon_ring_blocks_admission_when_payload_has_no_receipt_or_checks() {
        let payload = json!({"value": {"id": 1}, "andon_ring": true}).to_string();
        let result = judge_payload(&payload, "default").expect("should judge");
        assert_eq!(result["verdict"], json!("halted"));
        let categories = result["refusal_categories"]
            .as_array()
            .expect("categories array");
        assert!(
            categories.contains(&json!("topology")),
            "categories: {categories:?}"
        );
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
        let payload =
            r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
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

    /// Acquire the single process-wide lock that serializes every test which
    /// mutates the `PRAXIS_SIGNING_KEY` env var.
    ///
    /// `std::env` is process-global, so ALL signing-key writers in this module
    /// — the receipt group (`with_test_signing_key`) and the receipt-noun group
    /// (`with_receipt_noun_signing_key`) — must contend on the *same* mutex.
    /// Two separate locks would each serialize only their own group and still
    /// race across groups, flipping the key under a concurrently-running test.
    #[cfg(feature = "law-signed")]
    fn signing_key_env_guard() -> std::sync::MutexGuard<'static, ()> {
        use std::sync::{Mutex, MutexGuard, OnceLock};
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let guard: MutexGuard<'static, ()> = LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        guard
    }

    /// Set `PRAXIS_SIGNING_KEY` to the receipt-group key for the duration of the
    /// returned guard. Serializes on the shared [`signing_key_env_guard`] lock.
    #[cfg(feature = "law-signed")]
    fn with_test_signing_key() -> std::sync::MutexGuard<'static, ()> {
        let guard = signing_key_env_guard();
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
        assert!(
            result["signature"].is_string(),
            "expected base64 signature field"
        );
        assert!(
            result["verifying_key"].is_string(),
            "expected hex verifying_key field"
        );
        assert!(
            result["signed_receipt"].is_object(),
            "expected signed_receipt object"
        );
        assert_eq!(result["signed_receipt"]["chain_hash"], result["chain_hash"]);
    }

    #[cfg(feature = "law-signed")]
    #[test]
    fn verify_signature_round_trip_succeeds() {
        let _guard = with_test_signing_key();
        let result = receipt_payload(r#"{"value":{"id":1}}"#).expect("should receipt");
        let chain_hash = result["chain_hash"].clone();
        let signed_receipt = result["signed_receipt"].clone();
        let payload =
            json!({"chain_hash": chain_hash, "signed_receipt": signed_receipt}).to_string();
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
        let sig = signed_receipt["signature"]
            .as_str()
            .expect("signature string")
            .to_string();
        let mut chars: Vec<char> = sig.chars().collect();
        let idx = chars.iter().position(|&c| c != 'A').unwrap_or(0);
        chars[idx] = if chars[idx] == 'B' { 'C' } else { 'B' };
        signed_receipt["signature"] = json!(chars.into_iter().collect::<String>());
        let payload =
            json!({"chain_hash": chain_hash, "signed_receipt": signed_receipt}).to_string();
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
        let payload =
            r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
        let result = receipt_payload(payload).expect("should receipt");
        assert_eq!(result["status"], json!("denied"));
        assert_eq!(result["verdict"], json!("halted"));
    }

    // ── promote ─────────────────────────────────────────────────────────

    #[test]
    fn promote_from_lowest_rung_to_next_with_eligibility_false() {
        let result = promote_payload(r#"{"standing":"NAMED"}"#, "").expect("should promote");
        assert_eq!(result["status"], json!("promoted"));
        assert_eq!(result["from"], json!("NAMED"));
        assert_eq!(result["to"], json!("REGISTERED"));
        assert_eq!(result["is_partial_alive_eligible"], json!(false));
    }

    #[test]
    fn promote_to_a_rung_that_makes_it_eligible() {
        let result = promote_payload(r#"{"standing":"REGISTERED"}"#, "").expect("should promote");
        assert_eq!(result["to"], json!("DISPATCHABLE"));
        assert_eq!(result["is_partial_alive_eligible"], json!(true));
    }

    #[test]
    fn promote_to_rung_requiring_auditor_without_one_is_denied() {
        let result = promote_payload(r#"{"standing":"REFUSABLE"}"#, "").expect("should promote");
        assert_eq!(result["status"], json!("denied"));
        assert_eq!(result["to"], json!("REPLAYABLE"));
    }

    #[test]
    fn promote_to_that_rung_with_auditor_succeeds() {
        let result =
            promote_payload(r#"{"standing":"REFUSABLE"}"#, "alice").expect("should promote");
        assert_eq!(result["status"], json!("promoted"));
        assert_eq!(result["to"], json!("REPLAYABLE"));
        assert_eq!(result["auditor"], json!("alice"));
    }

    #[test]
    fn promote_from_top_rung_is_denied() {
        let result = promote_payload(r#"{"standing":"CERTIFIED"}"#, "").expect("should promote");
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

    /// Set `PRAXIS_SIGNING_KEY` to the receipt-noun-group key for the duration
    /// of the returned guard. Serializes on the *same* shared
    /// [`signing_key_env_guard`] lock as [`with_test_signing_key`], so the two
    /// groups can never flip the env var underneath one another.
    #[cfg(feature = "law-signed")]
    fn with_receipt_noun_signing_key() -> std::sync::MutexGuard<'static, ()> {
        let guard = signing_key_env_guard();
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
        let r1_chain = r1["record"]["chain_hash_hex"]
            .as_str()
            .expect("chain hash")
            .to_string();
        assert_eq!(r1["record"]["prev_chain_hash_hex"], json!("0".repeat(64)));

        let r2 = receipt_issue_payload(r#"{"value":{"id":2}}"#, &dir_str).expect("issue r2");
        assert_eq!(r2["record"]["prev_chain_hash_hex"], json!(r1_chain));
    }

    #[test]
    fn receipt_issue_with_halting_obligation_is_denied_and_not_persisted() {
        let dir = temp_receipts_dir();
        let dir_str = dir.to_string_lossy().to_string();
        let payload =
            r#"{"value":{"id":1},"obligations":[{"type":"blocking_constraint","reason":"stop"}]}"#;
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
        let result = receipt_export_ocel_payload(&dir_str, Some(&out_str)).expect("export-ocel");
        assert_eq!(result["status"], json!("exported"));
        assert_eq!(result["event_count"], json!(1));
        assert!(out_path.exists());
    }
}
