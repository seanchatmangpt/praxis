//! Fuzz the root-crate admission boundaries with proptest.
//!
//! Genesis Day 3, phase 2 (fuzzing) — the root-crate half of the perimeter that
//! `crates/praxis-core/tests/fuzz_boundaries.rs` covers for the core. The
//! load-bearing property everywhere here is the same: **no input may ever
//! panic**, and **every refusal must carry a reason** (a non-empty `Err`
//! message), never a silent failure or an unwind.
//!
//! Surfaces fuzzed:
//!   * every `ops::*_payload` entry point (`judge`/`admit`/`receipt`/`promote`/
//!     `inspect_obligations`/`show`) — arbitrary bytes/strings and JSON-biased
//!     strings.
//!   * the `PraxisConfig` admission loader (`star_toml::TrustedLoader::layer_str`
//!     — the exact mechanism `config::load_config` uses) — arbitrary TOML.
//!   * the PDDL parse + solve pipeline (`bcinr_pddl::{domain_from_pddl,
//!     problem_from_pddl, GroundProblem, GroundTemporalProblem}`) — arbitrary
//!     PDDL-ish text. These are the exact functions `verbs::plan::solve_payload`
//!     delegates to; that fn is `pub(crate)` and unreachable from an integration
//!     test, so we fuzz its delegates directly (AR-2: one implementation, no
//!     lookalike parser).
//!   * the RevTAC mission parser (`revtac::Mission::parse`, JSON + TOML).
//!
//! ## Case counts
//!
//! Each property runs `FUZZ_CASES` (default **1024**) iterations. The default is
//! lower than the pure-parser file because the PDDL `solve` path runs a bounded
//! forward search per case. Override with `PROPTEST_CASES=<n>`, e.g.
//! `PROPTEST_CASES=20000 cargo test --test fuzz_ops --all-features`.

use my_conforming_project::{config::PraxisConfig, ops, revtac::Mission};
use proptest::prelude::*;
use star_toml::{loader::TrustedLoader, nouns::EvidenceGate};

/// Default proptest iteration count. Override with `PROPTEST_CASES`.
const FUZZ_CASES: u32 = 1024;

fn cfg() -> ProptestConfig {
    ProptestConfig::with_cases(FUZZ_CASES)
}

/// `ops::receipt_payload` seals a chain hash, and under `--features law-signed`
/// (on with `--all-features`) `receipt()` signs it fail-closed and needs a key.
/// Set a deterministic process-wide key once — same house pattern as
/// `crates/praxis-core/tests/prop_law.rs`. Harmless when the feature is off.
fn ensure_signing_key() {
    static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        std::env::set_var(
            "PRAXIS_SIGNING_KEY",
            "a1a2a3a4a5a6a7a8a9aaabacadaeaff0f1f2f3f4f5f6f7f8f9fafbfcfdfe8765",
        );
    });
}

/// A refused (`Err`) result from any pure payload fn must carry a non-empty
/// diagnostic — never an empty or whitespace-only reason.
fn assert_reasoned(r: &Result<serde_json::Value, String>) {
    if let Err(msg) = r {
        assert!(!msg.trim().is_empty(), "refusal must carry a non-empty reason");
    }
}

proptest! {
    #![proptest_config(cfg())]

    // ── ops::*_payload never panic; refusals are reasoned ──────────────────

    /// Arbitrary strings into every payload fn: never panics, always Ok/Err,
    /// every Err reasoned. Covers the whole `law` verb surface at once.
    #[test]
    fn ops_payloads_never_panic_on_arbitrary_strings(s in ".*") {
        ensure_signing_key();
        assert_reasoned(&ops::judge_payload(&s, "default"));
        assert_reasoned(&ops::admit_payload(&s, "default"));
        assert_reasoned(&ops::receipt_payload(&s));
        assert_reasoned(&ops::promote_payload(&s, ""));
        assert_reasoned(&ops::inspect_obligations_payload(&s));
        assert_reasoned(&ops::show_payload(&s, "json"));
        assert_reasoned(&ops::show_payload(&s, "text"));
    }

    /// JSON-biased strings so proptest spends its budget past the `parse_value`
    /// gate and inside the judge/admit/receipt machinery.
    #[test]
    fn ops_payloads_never_panic_on_json_like(s in r#"[{}\[\]"a-zA-Z0-9:,._ +-]{0,256}"#) {
        ensure_signing_key();
        assert_reasoned(&ops::judge_payload(&s, "default"));
        assert_reasoned(&ops::admit_payload(&s, "default"));
        assert_reasoned(&ops::receipt_payload(&s));
        assert_reasoned(&ops::inspect_obligations_payload(&s));
    }

    /// Well-shaped `LawInput` JSON with an arbitrary `value` scalar drives the
    /// full judge -> admit -> receipt pipeline (no obligations, so it should
    /// reach `receipted`), exercising the real chain-sealing path — proving the
    /// never-panic properties have signal past mere parse rejection.
    #[test]
    fn ops_receipt_pipeline_seals_wellformed_input(v in any::<i64>()) {
        ensure_signing_key();
        let payload = serde_json::json!({"value": {"v": v}}).to_string();
        let r = ops::receipt_payload(&payload);
        prop_assert!(r.is_ok(), "well-formed receipt must not hard-error: {r:?}");
        let out = r.unwrap();
        prop_assert_eq!(out["status"].as_str(), Some("receipted"), "out: {}", out);
    }

    /// Arbitrary standing strings into `promote_payload`: an unrecognized rung
    /// is a reasoned `Err`, a real rung is `Ok` with a status — never a panic.
    #[test]
    fn promote_never_panics_on_arbitrary_standing(
        standing in "[A-Za-z_]{0,24}",
        auditor in "[a-z ]{0,16}",
    ) {
        let payload = serde_json::json!({"standing": standing}).to_string();
        let r = ops::promote_payload(&payload, &auditor);
        assert_reasoned(&r);
        if let Ok(v) = r {
            prop_assert!(v.get("status").is_some(), "promote Ok must have a status: {v}");
        }
    }

    // ── PraxisConfig TOML admission never panics; refusals are reasoned ─────

    /// Arbitrary strings into the real `TrustedLoader` admission surface (the
    /// same `layer_str` mechanism `config::load_config` uses): never panics;
    /// every rejection (parse error, unknown field, invariant violation) carries
    /// a reason.
    #[test]
    fn config_admission_never_panics_on_arbitrary_toml(s in ".*") {
        let result = TrustedLoader::new()
            .layer_str(&s, "fuzz")
            .with_oracle_gate(EvidenceGate::new())
            .load_admitted::<PraxisConfig>();
        if let Err(e) = result {
            prop_assert!(!e.to_string().trim().is_empty(), "config refusal must carry a reason");
        }
    }

    /// TOML-biased strings (keys, sections, values, quotes) so proptest reaches
    /// deserialize + unknown-field + `Validate` invariant checks rather than
    /// bouncing off the TOML lexer.
    #[test]
    fn config_admission_never_panics_on_toml_like(
        s in r#"[a-z_]{1,12}( *= *("[^"\n]{0,12}"|[0-9]{1,6}|true|false))?\n?"#
    ) {
        let result = TrustedLoader::new()
            .layer_str(&s, "fuzz")
            .with_oracle_gate(EvidenceGate::new())
            .load_admitted::<PraxisConfig>();
        if let Err(e) = result {
            prop_assert!(!e.to_string().trim().is_empty(), "config refusal must carry a reason");
        }
    }

    /// An out-of-range `attention_capacity` (the `1..=64` invariant) must be
    /// *rejected*, not admitted — proves the fuzz surface actually reaches the
    /// `Validate` stage, so a passing never-panic run isn't vacuous.
    #[test]
    fn config_rejects_out_of_range_capacity(cap in 65u32..=u32::MAX) {
        let toml = format!("[planner]\nattention_capacity = {cap}\n");
        let result = TrustedLoader::new()
            .layer_str(&toml, "fuzz")
            .with_oracle_gate(EvidenceGate::new())
            .load_admitted::<PraxisConfig>();
        prop_assert!(result.is_err(), "attention_capacity {cap} > 64 must be rejected");
    }

    // ── PDDL parse + solve never panics ────────────────────────────────────

    /// Arbitrary text into the PDDL parsers (`domain_from_pddl` /
    /// `problem_from_pddl` — the exact parsers `solve_payload` uses): never
    /// panics; a malformed source is an `Err`, never an unwind.
    #[test]
    fn pddl_parsers_never_panic_on_arbitrary_text(s in ".{0,256}") {
        let _ = bcinr_pddl::domain_from_pddl(&s);
        let _ = bcinr_pddl::problem_from_pddl(&s);
    }

    /// PDDL-ish text (s-expression punctuation + keywords) into the full parse +
    /// ground + solve pipeline. When both sides parse, grounding and bounded
    /// search must also never panic (they may legitimately return infeasibility
    /// or an error — we only assert the absence of an unwind).
    #[test]
    fn pddl_solve_pipeline_never_panics_on_pddl_like(
        d in r#"[()a-z0-9:?_ \-\n]{0,200}"#,
        p in r#"[()a-z0-9:?_ \-\n]{0,200}"#,
    ) {
        if let (Ok(domain), Ok(problem)) =
            (bcinr_pddl::domain_from_pddl(&d), bcinr_pddl::problem_from_pddl(&p))
        {
            if let Ok(g) = bcinr_pddl::GroundProblem::build(&domain, &problem, None) {
                let _ = g.find_plan();
            }
            if let Ok(gt) = bcinr_pddl::GroundTemporalProblem::build(&domain, &problem) {
                let _ = gt.find_temporal_plan();
            }
        }
    }

    // ── RevTAC mission parsing never panics; refusals are reasoned ──────────

    /// Arbitrary strings into `Mission::parse` under every format selector:
    /// never panics; every `Err` carries a reason.
    #[test]
    fn mission_parse_never_panics(s in ".*", fmt in prop::sample::select(vec!["auto", "json", "toml", ""])) {
        if let Err(e) = Mission::parse(&s, fmt) {
            prop_assert!(!e.trim().is_empty(), "mission parse refusal must carry a reason");
        }
    }

    /// TOML-biased mission text (auto format) — reaches the TOML deserializer
    /// rather than bouncing off the leading-`{` JSON heuristic.
    #[test]
    fn mission_parse_never_panics_on_toml_like(
        s in r#"[a-z_]{1,12}( *= *"[^"\n]{0,16}")?\n?"#
    ) {
        if let Err(e) = Mission::parse(&s, "auto") {
            prop_assert!(!e.trim().is_empty(), "mission parse refusal must carry a reason");
        }
    }
}
