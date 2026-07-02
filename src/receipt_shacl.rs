//! Bridge a praxis [`ReceiptRecord`] to the open-ontologies canonical receipt
//! format `sr:SharedReceiptV1` and validate it against the shared SHACL
//! shapes (`ontology/shared-receipt-shapes.ttl` in `/Users/sac/open-ontologies`).
//!
//! # Two receipt vocabularies, honestly reconciled
//!
//! praxis's [`ReceiptRecord`] is a *law-chain* receipt: a payload hash, the
//! previous chain hash, the resulting chain hash, an emission instant, and an
//! Andon outcome. `SharedReceiptV1` is an *execution* receipt exchanged
//! between wasm4pm and mcpp: a UUID run id, a start/end/duration triple, a
//! status, and a five-way hash taxonomy (config / input / plan / output /
//! proof_pack). The two do not line up field-for-field, so this adapter maps
//! what genuinely corresponds and is explicit about what it synthesizes. Each
//! target field below is tagged **[native]**, **[derived]**, or
//! **[synthesized]**:
//!
//! | `sr:` field              | source in praxis `ReceiptRecord`                         |
//! |--------------------------|----------------------------------------------------------|
//! | `run_id`                 | **[synthesized]** deterministic UUID-v4 shape from `chain_hash_hex` (praxis identifies receipts by chain hash, not UUID) |
//! | `schema_version`         | **[synthesized]** constant `"shared/v1"` (adapter sentinel) |
//! | `start_time`             | **[native]** `ts_ns` rendered as ISO-8601 UTC             |
//! | `end_time`               | **[derived]** `start_time + duration_ms` (praxis records an instant, not a span) |
//! | `duration_ms`            | **[native, newly added]** `ReceiptRecord::duration_ms`, `0` when unmeasured |
//! | `status`                 | **[derived]** from `andon`: Green→`admitted`, Overridden→`partial`, Halted→`refused` |
//! | `hash_format`            | **[native]** constant `"blake3-hex-64"` — praxis hashes *are* bare 64-char blake3 hex |
//! | `hashes.input`           | **[native]** `payload_hash_hex` (both hash the input payload) |
//! | `hashes.proof_pack`      | **[native]** `chain_hash_hex` (the chain hash *is* praxis's cryptographic proof) |
//! | `hashes.output`          | **[derived]** `chain_hash_hex` (the receipt step's output is the new chain hash) |
//! | `hashes.config`          | **[derived]** `prev_chain_hash_hex` (the chained-onto state is the config context) |
//! | `hashes.plan`            | **[derived]** `payload_hash_hex` (praxis has no separate plan artifact; the admitted payload *is* the plan of record) |
//! | `chain_predecessor`      | **[native]** `prev_chain_hash_hex`, or `"genesis"` when all-zero |
//! | `otel_run_id_attribute`  | **[synthesized]** constant `"mcpp.run_id"` — praxis emits receipts from its MCP tool server (`mcp_lawobject_server`), so it adopts the mcpp attribute name when bridging |
//!
//! # What praxis genuinely lacks (receipted, not papered over)
//!
//! - **`duration_ms`** was a real gap: praxis's law layer records only the
//!   emission instant `ts_ns`. Rather than fabricate a span, we *added*
//!   [`ReceiptRecord::duration_ms`] as an optional field (see its doc), so the
//!   mapping is honest — it is `0` on today's live path (unmeasured) and can
//!   be populated by callers that time admission.
//! - The **five-way hash taxonomy** is a genuine vocabulary mismatch: praxis's
//!   chain has three hashes with *chain* semantics, not five with *execution*
//!   semantics. The `output`/`config`/`plan` mappings above are documented
//!   re-uses of praxis's three hashes, not distinct artifacts praxis produces.
//! - The **`conformance` dimensions** (fitness/precision/lifecycle/…) are
//!   *not* mapped: in praxis they are computed by
//!   [`praxis_core::receipt_validator`] as a *separate* replay concern, not
//!   stored on the record. `sr:conformance` is optional (no `sh:minCount`), so
//!   omitting it is conformant; adding conformance to `ReceiptRecord` would
//!   duplicate state the validator already owns, so we deliberately do not.
//!
//! The validation test below therefore proves a *faithful* mapping conforms —
//! not that praxis already speaks `SharedReceiptV1`.

use praxis_core::law::Andon;
use praxis_core::ReceiptRecord;

/// The `sr:` namespace the SHACL shapes target.
const SR: &str = "urn:ontostar:shared-receipt:";

/// Convert a nanoseconds-since-Unix-epoch instant to an ISO-8601 UTC string
/// (`YYYY-MM-DDTHH:MM:SSZ`), using Howard Hinnant's `civil_from_days`
/// algorithm so no date library is needed and the result is exact.
fn ts_ns_to_iso8601(ts_ns: u64) -> String {
    let secs = (ts_ns / 1_000_000_000) as i64;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (hh, mm, ss) = (sod / 3600, (sod % 3600) / 60, sod % 60);

    // civil_from_days: days since 1970-01-01 -> (year, month, day).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = y + i64::from(m <= 2);

    format!("{year:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// Synthesize a UUID-v4-shaped run id deterministically from a 64-char hex
/// chain hash: the version nibble is forced to `4` and the variant nibble to
/// `[89ab]`, so the result matches the `sr:run_id` UUID-v4 pattern while
/// remaining a pure function of the chain hash.
fn synth_run_id(chain_hash_hex: &str) -> String {
    // Pad defensively so a short/degenerate hash still yields a well-formed id.
    let h = format!("{chain_hash_hex:0<32}");
    let b = h.as_bytes();
    let s = |lo: usize, hi: usize| std::str::from_utf8(&b[lo..hi]).unwrap_or("0").to_string();
    let variant = match b[16] {
        c @ (b'8' | b'9' | b'a' | b'b') => c as char,
        _ => '8',
    };
    format!(
        "{}-{}-4{}-{}{}-{}",
        s(0, 8),
        s(8, 12),
        s(13, 16),
        variant,
        s(17, 20),
        s(20, 32)
    )
}

/// Map an [`Andon`] outcome to an `sr:status` enum value.
fn andon_status(andon: &Andon) -> &'static str {
    match andon {
        Andon::Green => "admitted",
        Andon::Overridden { .. } => "partial",
        Andon::Halted { .. } => "refused",
    }
}

/// `prev_chain_hash_hex` as an `sr:chain_predecessor`: `"genesis"` when the
/// all-zero root, otherwise the bare hex.
fn chain_predecessor(prev_hex: &str) -> String {
    if prev_hex.chars().all(|c| c == '0') {
        "genesis".to_string()
    } else {
        prev_hex.to_string()
    }
}

/// Map a praxis [`ReceiptRecord`] to `SharedReceiptV1` Turtle (see the module
/// docs for the field-by-field, native/derived/synthesized mapping).
///
/// Both the receipt node and its nested hashes node are emitted as IRIs (not
/// blank nodes) and typed `a sr:SharedReceiptV1` / `a sr:HashesShape` so the
/// minCount-based SHACL validator actually visits and checks them.
pub fn receipt_record_to_shared_receipt_turtle(record: &ReceiptRecord) -> String {
    let start = ts_ns_to_iso8601(record.ts_ns);
    let duration_ms = record.duration_ms.unwrap_or(0);
    let end = ts_ns_to_iso8601(record.ts_ns + duration_ms.saturating_mul(1_000_000));
    let run_id = synth_run_id(&record.chain_hash_hex);
    let status = andon_status(&record.andon);
    let predecessor = chain_predecessor(&record.prev_chain_hash_hex);

    let node = format!("urn:praxis:receipt:{}", record.chain_hash_hex);
    let hashes = format!("{node}:hashes");

    format!(
        r#"@prefix sr:  <{SR}> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<{node}> a sr:SharedReceiptV1 ;
    sr:run_id "{run_id}" ;
    sr:schema_version "shared/v1" ;
    sr:start_time "{start}" ;
    sr:end_time "{end}" ;
    sr:duration_ms "{duration_ms}"^^xsd:nonNegativeInteger ;
    sr:status "{status}" ;
    sr:hash_format "blake3-hex-64" ;
    sr:otel_run_id_attribute "mcpp.run_id" ;
    sr:chain_predecessor "{predecessor}" ;
    sr:hashes <{hashes}> .

<{hashes}> a sr:HashesShape ;
    sr:config "{config}" ;
    sr:input "{input}" ;
    sr:plan "{plan}" ;
    sr:output "{output}" ;
    sr:proof_pack "{proof_pack}" .
"#,
        // hashes.config  <- prev_chain (the config context this step ran against)
        config = record.prev_chain_hash_hex,
        // hashes.input   <- payload hash (the input payload)
        input = record.payload_hash_hex,
        // hashes.plan    <- payload hash (admitted payload is the plan of record)
        plan = record.payload_hash_hex,
        // hashes.output  <- chain hash (the receipt step's output)
        output = record.chain_hash_hex,
        // hashes.proof_pack <- chain hash (the cryptographic proof)
        proof_pack = record.chain_hash_hex,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ggen_graph::prelude::validate_shacl;
    use praxis_core::law::Andon;
    use praxis_core::receipt_record::RECEIPT_RECORD_VERSION;

    /// The canonical SharedReceiptV1 SHACL shapes, compiled in from the
    /// open-ontologies repo so this test validates against the *real* shapes
    /// (a moved/renamed file breaks the build loudly rather than silently
    /// validating against a stale vendored copy).
    const SHARED_RECEIPT_SHAPES: &str =
        include_str!("../../open-ontologies/ontology/shared-receipt-shapes.ttl");

    /// A receipt record with realistic hashes and a fixed instant. `andon` is
    /// Green (the ordinary admitted receipt) and `duration_ms` is set, so the
    /// derived `end_time`/`status` are exercised, not just their defaults.
    fn sample_record() -> ReceiptRecord {
        ReceiptRecord {
            version: RECEIPT_RECORD_VERSION,
            instruction_id: 7,
            activity_idx: 0,
            activity: Some("law.receipt".to_string()),
            node_kind: 0,
            ts_ns: 1_751_328_000_000_000_000, // 2025-07-01T00:00:00Z
            duration_ms: Some(42),
            payload_hash_hex: "a".repeat(64),
            prev_chain_hash_hex: "0".repeat(64),
            chain_hash_hex: "b3f1c2d4e5a6978089a0b1c2d3e4f5061728394a5b6c7d8e9f0a1b2c3d4e5f60"
                .to_string(),
            andon: Andon::Green,
            obligation_count: 0,
            object_ids: vec!["law:aaaaaaaaaaaaaaaa".to_string()],
        }
    }

    #[test]
    fn mapped_receipt_conforms_to_shared_receipt_v1_shapes() {
        let record = sample_record();
        let turtle = receipt_record_to_shared_receipt_turtle(&record);
        let violations = validate_shacl(&turtle, &[SHARED_RECEIPT_SHAPES])
            .expect("data + shapes must parse as Turtle");
        assert!(
            violations.is_empty(),
            "mapped SharedReceiptV1 must satisfy the canonical SHACL shapes, got: {violations:#?}\n\nturtle:\n{turtle}"
        );
    }

    #[test]
    fn dropping_a_required_hash_is_detected_as_a_violation() {
        // Guard against a vacuous pass: if the validator were not actually
        // visiting the typed IRI nodes, a missing required field would still
        // "conform". Hand-build a hashes node missing `proof_pack` and assert
        // the minCount violation is raised.
        let broken = format!(
            r#"@prefix sr: <{SR}> .
<urn:praxis:receipt:x> a sr:SharedReceiptV1 ;
    sr:run_id "00000000-0000-4000-8000-000000000000" ;
    sr:schema_version "shared/v1" ;
    sr:start_time "1970-01-01T00:00:00Z" ;
    sr:end_time "1970-01-01T00:00:00Z" ;
    sr:duration_ms "0" ;
    sr:status "admitted" ;
    sr:hash_format "blake3-hex-64" ;
    sr:otel_run_id_attribute "mcpp.run_id" ;
    sr:hashes <urn:praxis:receipt:x:hashes> .

<urn:praxis:receipt:x:hashes> a sr:HashesShape ;
    sr:config "{h}" ;
    sr:input "{h}" ;
    sr:plan "{h}" ;
    sr:output "{h}" .
"#,
            h = "a".repeat(64)
        );
        let violations =
            validate_shacl(&broken, &[SHARED_RECEIPT_SHAPES]).expect("still valid Turtle");
        assert!(
            violations.iter().any(|v| v.path.ends_with("proof_pack")),
            "missing hashes.proof_pack should raise a proof_pack minCount violation, got: {violations:#?}"
        );
    }

    #[test]
    fn iso8601_conversion_is_correct_for_a_known_instant() {
        // 2025-07-01T00:00:00Z in ns since epoch.
        assert_eq!(ts_ns_to_iso8601(1_751_328_000_000_000_000), "2025-07-01T00:00:00Z");
        // Unix epoch itself.
        assert_eq!(ts_ns_to_iso8601(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn synth_run_id_matches_uuid_v4_shape() {
        let id = synth_run_id(&"b".repeat(64));
        // 8-4-4-4-12 with version nibble 4 and variant in [89ab].
        let groups: Vec<&str> = id.split('-').collect();
        assert_eq!(groups.iter().map(|g| g.len()).collect::<Vec<_>>(), vec![8, 4, 4, 4, 12]);
        assert!(groups[2].starts_with('4'), "version nibble must be 4: {id}");
        assert!(
            matches!(groups[3].chars().next(), Some('8' | '9' | 'a' | 'b')),
            "variant nibble must be [89ab]: {id}"
        );
    }
}
