//! Adversarial refusal suite for the Solvane Global bribery-compliance case
//! (`crates/multifractal-workflow/src/bin/crown-bribery-case.rs`, the
//! `crown-bribery-case` CLI).
//!
//! # Scope, honestly stated
//!
//! `crown-bribery-case` (Stage 2, verified live this session:
//! `CARGO_TARGET_DIR=target/agent-crown-bribery-case cargo run -p
//! multifractal-workflow --bin crown-bribery-case -- --run-id
//! adversarial-scenario-pre-check`, exit 0, `receipt_hash=
//! 5d9d5fa29c21e18b161f498545a3d4de401a5e226fe19177f634805136e60300`)
//! currently drives exactly: **F02 admission -> Knowledge Hook obligation
//! derivation -> F08 PDDL planning (Problem Projector, Domain Resolver,
//! Action-Hook Binder, Planner, Plan Validator/Effect Trace/Plan Receipt) ->
//! F09/F10 growth+geometry -> F13 Arazzo manufacture -> F14 AIR compile**.
//! Stage 3 (an F15/F16 real-Erlang-escript dispatch tail through F18
//! Broker -> F20 external dispatch -> F02 re-admission -> F21/F24/F25) was
//! attempted this session but is **BLOCKED**: plan mode interrupted the
//! prior agent turn before any code was written, so
//! `crown-bribery-case.rs` does NOT yet call `f18_broker_law::Broker`,
//! `f16_otp_runner::bridge`, `f20_external_dispatch`, or
//! `f25_receipts_replay` at all (confirmed this session: `grep -c
//! "f18_broker_law\|f16_otp_runner\|f20_external_dispatch\|f25_receipts_replay"
//! src/bin/crown-bribery-case.rs` = 0).
//!
//! Every scenario below is therefore built one of two ways, disclosed
//! per-test:
//!
//! 1. **Directly against the real CLI chain** (F02 admission, hook
//!    obligation derivation + evidence-type catalog projection, F08
//!    `run_pipeline`) -- by duplicating the same private fixture-loading
//!    helpers `crown-bribery-case.rs` itself uses, the SAME pattern that
//!    file's own module doc discloses for `build_policy` ("byte-for-byte
//!    ... duplicated here ... rather than imported") and
//!    `tests/bribery_case_fixture.rs` already established for this exact
//!    crate. These scenarios exercise the binary's real, wired chain.
//! 2. **Directly against `f18_broker_law::Broker` / `f25_receipts_replay`**
//!    -- real, in-process, fully unit-tested library code (no escript, no
//!    Erlang/BEAM dependency) that Stage 3's own blocked implementation
//!    plan named as exactly what it intended to wire into this CLI. These
//!    scenarios prove the underlying mechanism genuinely refuses; they are
//!    NOT yet reachable by running the `crown-bribery-case` binary itself
//!    (that wiring is the disclosed Stage-3 gap), and each such test's doc
//!    comment says so explicitly rather than implying CLI reachability it
//!    does not have.
//!
//! No scenario reaches into F15/F16 (real Erlang `escript` dispatch) --
//! that hop is real and independently tested elsewhere
//! (`crown_external_test.rs`, `#[ignore]`-gated, requires `just
//! erlang-compile`), but nothing in `crown-bribery-case.rs` calls it, so no
//! adversarial claim is made about it here.
//!
//! All 7 scenarios below are plain `#[test]` (no `#[ignore]`, no external
//! process dependency) -- run with `cargo test -p multifractal-workflow
//! --test bribery_case_adversarial`.

use std::collections::{BTreeMap, BTreeSet};

use multifractal_workflow::f02_observation_admission::{
    admit_observation, AdmissionLedger, AdmissionPolicy, ObservationAdmissionRefused,
    RawObservation,
};
use multifractal_workflow::f08_pddl_planning::projector::{
    AdmittedTriple, HOOK_PACK_PREDICATE, PDDL_DOMAIN_PREDICATE, PDDL_PROBLEM_PREDICATE,
};
use multifractal_workflow::f08_pddl_planning::refusal::Refusal as PlanningRefusal;
use multifractal_workflow::f08_pddl_planning::run_pipeline;
use multifractal_workflow::f18_broker_law::{
    ActionId, Broker, BrokerSecret, BrokerState, UnreceiptedActuationRefused,
};
use multifractal_workflow::f25_receipts_replay::{
    independent_verifier, receipt_builder, DigestKind, Materials, ReceiptReplayRefused,
};

use bcinr_pddl::Pddl8Error;
use praxis_graphlaw::parser::{Parser, Syntax};
use praxis_graphlaw::triples::{Term, VarOrTerm};
use praxis_graphlaw::TripleStore;

// ---------------------------------------------------------------------------
// Fixture content -- byte-identical to `crown-bribery-case.rs`'s own
// `include_str!`s (same relative path, one directory up from `src/bin/`).
// ---------------------------------------------------------------------------

const CASE_TTL: &str = include_str!("../fixtures/bribery-case/case.ttl");
const HOOK_TTL: &str = include_str!("../fixtures/bribery-case/hook.ttl");
const SHAPES_TTL: &str = include_str!("../fixtures/bribery-case/shapes.ttl");
const DOMAIN_TTL: &str = include_str!("../fixtures/bribery-case/pddl-domain.ttl");

const SOURCE_ID: &str = "solvane-case-intake-1";
const SOURCE_PRINCIPAL_IRI: &str = "https://intake.solvane-global.example.org/case-intake-1";
const SUBJECT: &str = "https://cases.solvane-global.example.org/case/BRB-2026-0417";
const SC: &str = "https://cases.solvane-global.example.org/vocab#";

/// F08's Action-Hook Binder catalog -- byte-identical to
/// `crown-bribery-case.rs`'s own `ACTION_HOOK_PACK_TTL` (duplicated per the
/// same established pattern; not part of the crate's public API to import).
const ACTION_HOOK_PACK_TTL: &str = r#"
@prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
@prefix ex: <urn:mfw:crown-bribery-case:hooks#> .

ex:hook-supply-evidence a kh:Hook ;
  kh:name "supply-evidence-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-supply-evidence" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:supply-evidence> ;
  kh:reason "solvane-compliance-officer-authority-supply-evidence" ;
  kh:priority 1 .

ex:hook-clear-transaction-obligation a kh:Hook ;
  kh:name "clear-transaction-obligation-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-clear-transaction-obligation" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:clear-transaction-obligation> ;
  kh:reason "solvane-compliance-officer-authority-clear-transaction-obligation" ;
  kh:priority 1 .

ex:hook-clear-authorization-obligation a kh:Hook ;
  kh:name "clear-authorization-obligation-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-clear-authorization-obligation" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:clear-authorization-obligation> ;
  kh:reason "solvane-compliance-officer-authority-clear-authorization-obligation" ;
  kh:priority 1 .

ex:hook-clear-policy-obligation a kh:Hook ;
  kh:name "clear-policy-obligation-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-clear-policy-obligation" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:clear-policy-obligation> ;
  kh:reason "solvane-compliance-officer-authority-clear-policy-obligation" ;
  kh:priority 1 .

ex:hook-close-obligations a kh:Hook ;
  kh:name "close-obligations-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-close-obligations" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:close-obligations> ;
  kh:reason "solvane-compliance-officer-authority-close-obligations" ;
  kh:priority 1 .

ex:hook-judge a kh:Hook ;
  kh:name "judge-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-judge" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:judge> ;
  kh:reason "solvane-compliance-officer-authority-judge" ;
  kh:priority 1 .

ex:hook-admit a kh:Hook ;
  kh:name "admit-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-admit" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:admit> ;
  kh:reason "solvane-general-counsel-authority-admit" ;
  kh:priority 1 .

ex:hook-receipt a kh:Hook ;
  kh:name "receipt-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-receipt" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:receipt> ;
  kh:reason "solvane-general-counsel-authority-receipt" ;
  kh:priority 1 .

ex:hook-block-for-missing-evidence a kh:Hook ;
  kh:name "block-for-missing-evidence-hook" ;
  kh:kind "delta" ;
  kh:var "urn:mfw:crown-bribery-case:hooks#actuates-block-for-missing-evidence" ;
  kh:on "assert" ;
  kh:effect "ground-action" ;
  kh:action <urn:pddl:action:block-for-missing-evidence> ;
  kh:reason "solvane-compliance-officer-authority-block-for-missing-evidence" ;
  kh:priority 1 .
"#;

// ---------------------------------------------------------------------------
// Duplicated CLI helpers (see module doc comment: same pattern
// `crown-bribery-case.rs` and `tests/bribery_case_fixture.rs` both already
// use for this exact fixture).
// ---------------------------------------------------------------------------

fn build_policy() -> AdmissionPolicy {
    let mut known_principals = BTreeMap::new();
    known_principals.insert(SOURCE_ID.to_string(), SOURCE_PRINCIPAL_IRI.to_string());

    let mut authorized = BTreeSet::new();
    authorized.insert("http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string());
    authorized.insert("http://www.w3.org/ns/prov#wasAssociatedWith".to_string());
    authorized.insert("http://www.w3.org/ns/prov#used".to_string());
    authorized.insert("http://www.w3.org/ns/prov#startedAtTime".to_string());
    authorized.insert("http://purl.org/dc/terms/identifier".to_string());
    authorized.insert("http://purl.org/dc/terms/description".to_string());
    authorized.insert(format!("{SC}caseStatus"));
    let mut authorized_predicates = BTreeMap::new();
    authorized_predicates.insert(SOURCE_ID.to_string(), authorized);

    AdmissionPolicy::new(
        known_principals,
        authorized_predicates,
        vec![
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
            "http://www.w3.org/ns/prov#".to_string(),
            "http://purl.org/dc/terms/".to_string(),
            SC.to_string(),
        ],
        vec!["https://".to_string()],
        SHAPES_TTL,
    )
    .expect("bribery-case AdmissionPolicy: SHACL shapes.ttl must parse")
}

fn raw_observation(correlation_id: &str, payload: String) -> RawObservation {
    RawObservation {
        correlation_id: correlation_id.to_string(),
        source_id: SOURCE_ID.to_string(),
        declared_subject: SUBJECT.to_string(),
        payload_turtle: payload,
    }
}

fn bare_iri(vt: &VarOrTerm) -> Option<String> {
    match vt {
        VarOrTerm::Term(t @ Term::Iri(_)) => {
            let displayed = t.to_string();
            Some(
                displayed
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .to_string(),
            )
        }
        _ => None,
    }
}

/// # Complexity
/// O(len(line)).
fn extract_last_angle_iri(line: &str) -> Option<String> {
    let after_last_lt = line.rsplit_once('<')?.1;
    let iri = after_last_lt.split('>').next()?;
    Some(iri.to_string())
}

struct DerivedObligations {
    turtle: String,
    local_names: Vec<String>,
}

/// Byte-for-byte the same real mechanism `crown-bribery-case.rs::derive_obligations`
/// and `tests/bribery_case_fixture.rs` use: `TripleStore::load_hook_pack` +
/// `.materialize()`, real SPARQL-CONSTRUCT, not simulated.
fn derive_obligations(hook_ttl: &str, case_ttl: &str) -> DerivedObligations {
    let mut store = TripleStore::new();
    store
        .load_hook_pack(hook_ttl)
        .expect("hook.ttl (or a catalog-only mutation of it) must load as a valid kh: hook pack");
    store
        .load_triples(case_ttl, Syntax::Turtle)
        .expect("case.ttl must load as valid Turtle");
    store
        .materialize()
        .expect("materialize() must succeed (no refusing hooks in this pack)");

    let dump = store.content_to_string();
    let predicate = format!("{SC}hasObligation");
    let mut names = BTreeSet::new();
    let mut lines = Vec::new();
    for line in dump.lines() {
        if line.contains(SUBJECT) && line.contains(&predicate) {
            if let Some(obj_iri) = extract_last_angle_iri(line) {
                if let Some(local) = obj_iri.rsplit('#').next() {
                    names.insert(local.to_string());
                }
            }
            lines.push(line);
        }
    }
    assert!(
        !names.is_empty(),
        "hook.ttl must derive >=1 sc:hasObligation triple for the real case.ttl fixture"
    );
    let mut turtle = String::new();
    for line in &lines {
        turtle.push_str(line);
        turtle.push('\n');
    }
    DerivedObligations {
        turtle,
        local_names: names.into_iter().collect(),
    }
}

/// Real error type this scenario forces: [`ObligationEvidenceRefused`]
/// mirrors `crown-bribery-case.rs`'s private `CliError::EvidenceTypeCatalogMissing`
/// (same name, same field, duplicated per this file's disclosed pattern).
#[derive(Debug, PartialEq, Eq)]
enum ObligationEvidenceRefused {
    EvidenceTypeCatalogMissing { obligation_local_name: String },
}

/// Byte-for-byte the same real mechanism `crown-bribery-case.rs::evidence_type_for_obligation`
/// uses: a direct triple scan over a fresh re-parse of `hook_ttl` for
/// `sc:requiresEvidenceType`.
fn evidence_type_for_obligation(
    hook_ttl: &str,
    obligation_local_name: &str,
) -> Result<String, ObligationEvidenceRefused> {
    let triples =
        Parser::parse_triples(hook_ttl, Syntax::Turtle).expect("hook_ttl must parse as Turtle");
    let predicate = VarOrTerm::convert(format!("{SC}requiresEvidenceType"));
    for t in &triples {
        if t.p != predicate {
            continue;
        }
        let Some(subject_iri) = bare_iri(&t.s) else {
            continue;
        };
        if subject_iri.rsplit('#').next() != Some(obligation_local_name) {
            continue;
        }
        if let Some(object_iri) = bare_iri(&t.o) {
            if let Some(local) = object_iri.rsplit('#').next() {
                return Ok(local.to_string());
            }
        }
    }
    Err(ObligationEvidenceRefused::EvidenceTypeCatalogMissing {
        obligation_local_name: obligation_local_name.to_string(),
    })
}

/// Real, working PDDL8 domain+problem text for `run_pipeline`, manufactured
/// through the SAME `my_conforming_project::mfg::manufacture` real RDF ->
/// PDDL8 pipeline `crown-bribery-case.rs` itself calls -- not a hand-authored
/// substitute.
fn manufacture_real_case_pddl() -> my_conforming_project::mfg::Manufactured {
    let derived = derive_obligations(HOOK_TTL, CASE_TTL);
    let case_local_name = "case-brb-2026-0417";
    let evidence_types: Vec<String> = derived
        .local_names
        .iter()
        .map(|o| {
            evidence_type_for_obligation(HOOK_TTL, o)
                .expect("real hook.ttl carries every obligation's evidence-type catalog fact")
        })
        .collect();
    let mut init_atoms = Vec::new();
    for o in &derived.local_names {
        init_atoms.push(format!("(has-obligation {case_local_name} {o})"));
    }
    for (o, e) in derived.local_names.iter().zip(evidence_types.iter()) {
        init_atoms.push(format!("(requires-evidence {o} {e})"));
    }

    let mut out = String::new();
    out.push_str("@prefix pddl: <http://seanchatmangpt.github.io/praxis/pddl#> .\n\n");
    out.push_str(&format!(
        "<urn:mfw:crown-bribery-case:problem:{case_local_name}>\n    a pddl:Problem ;\n"
    ));
    out.push_str(&format!(
        "    pddl:name \"bribery-case-{case_local_name}-adversarial\" ;\n"
    ));
    out.push_str("    pddl:domain \"solvane-bribery-compliance-pddl8\" ;\n");
    let mut objects: Vec<String> = vec![format!(
        "[ pddl:name \"{case_local_name}\" ; pddl:ofType \"law-object\" ]"
    )];
    for o in &derived.local_names {
        objects.push(format!("[ pddl:name \"{o}\" ; pddl:ofType \"obligation\" ]"));
    }
    for e in &evidence_types {
        objects.push(format!(
            "[ pddl:name \"{e}\" ; pddl:ofType \"evidence-type\" ]"
        ));
    }
    objects.push(
        "[ pddl:name \"compliance-officer-shreya-patel\" ; pddl:ofType \"validator\" ]".to_string(),
    );
    objects.push(
        "[ pddl:name \"general-counsel-marcus-webb\" ; pddl:ofType \"authority\" ]".to_string(),
    );
    objects.push(format!(
        "[ pddl:name \"tok-genesis-{case_local_name}\" ; pddl:ofType \"chain-token\" ]"
    ));
    for stage in ["raw", "validated", "admitted", "receipted", "blocked"] {
        objects.push(format!(
            "[ pddl:name \"{stage}\" ; pddl:ofType \"lifecycle-stage\" ]"
        ));
    }
    out.push_str("    pddl:object ");
    out.push_str(&objects.join(" ,\n               "));
    out.push_str(" ;\n");
    let mut init: Vec<String> = vec![format!("\"(in-stage {case_local_name} raw)\"")];
    for atom in &init_atoms {
        init.push(format!("\"{atom}\""));
    }
    init.push(format!(
        "\"(prev-chain-valid tok-genesis-{case_local_name})\""
    ));
    out.push_str("    pddl:init ");
    out.push_str(&init.join(" ,\n             "));
    out.push_str(" ;\n");
    out.push_str(&format!(
        "    pddl:goal \"(in-stage {case_local_name} receipted)\" .\n"
    ));

    let combined = format!("{DOMAIN_TTL}\n{out}");
    my_conforming_project::mfg::manufacture(&combined, "bribery-case-adversarial-suite")
        .expect("real bribery-case domain+problem RDF must manufacture to valid PDDL8 text")
}

fn f08_graph(
    manufactured: &my_conforming_project::mfg::Manufactured,
    hook_pack_ttl: &str,
) -> Vec<AdmittedTriple> {
    vec![
        AdmittedTriple {
            subject: SUBJECT.to_string(),
            predicate: PDDL_DOMAIN_PREDICATE.to_string(),
            object_literal: manufactured.project_domain_text().clone(),
        },
        AdmittedTriple {
            subject: SUBJECT.to_string(),
            predicate: PDDL_PROBLEM_PREDICATE.to_string(),
            object_literal: manufactured.project_problem_text().clone(),
        },
        AdmittedTriple {
            subject: SUBJECT.to_string(),
            predicate: HOOK_PACK_PREDICATE.to_string(),
            object_literal: hook_pack_ttl.to_string(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Scenario 1 -- Missing approval: dispatch a step without a bound approval
// matching the plan digest.
//
// NOT reachable via the CLI binary today (Stage 3/F18 wiring is BLOCKED --
// see module doc). Exercises the real `f18_broker_law::Broker` directly,
// using this case's own REAL F02 admission receipt hash as the "plan
// digest" the approval must be bound to (not a fabricated string).
// ---------------------------------------------------------------------------

#[test]
fn scenario1_dispatch_without_a_bound_approval_matching_the_plan_digest_is_refused() {
    let policy = build_policy();
    let ledger = AdmissionLedger::new();

    // Real admission #1: this run's real digest ("the plan").
    let real_admission = admit_observation(
        &policy,
        &ledger,
        raw_observation("corr-scenario1-real-run", CASE_TTL.to_string()),
    )
    .expect("real admission must succeed");

    // Real admission #2 (different correlation_id -> different real digest):
    // stands in for a DIFFERENT, unrelated prior plan run whose approval a
    // caller might mistakenly (or maliciously) try to reuse here.
    let stale_admission = admit_observation(
        &policy,
        &ledger,
        raw_observation("corr-scenario1-stale-prior-run", CASE_TTL.to_string()),
    )
    .expect("stale admission must also succeed (a real, different prior run)");
    assert_ne!(
        real_admission.receipt_hash, stale_admission.receipt_hash,
        "the two admissions must carry genuinely different real digests for this test to be meaningful"
    );

    let broker = Broker::new(BrokerSecret::new([0x51; 32]));

    let real_action = ActionId::new(
        "crown-bribery-case-workflow",
        "supply-evidence",
        real_admission.receipt_hash.clone(),
    );
    let stale_action = ActionId::new(
        "crown-bribery-case-workflow",
        "supply-evidence",
        stale_admission.receipt_hash.clone(),
    );
    // The one legitimate approval that exists is bound to the STALE plan
    // digest, not the real one being dispatched now.
    let (_, stale_bound_token) = broker.authorize(&stale_action);

    let err = broker
        .claim_idempotency(real_action.clone(), stale_bound_token)
        .expect_err(
            "an approval token bound to a different plan digest must never be accepted for this action",
        );
    assert!(
        matches!(err, UnreceiptedActuationRefused::AuthorityInvalid { .. }),
        "expected AuthorityInvalid (approval does not match this plan's digest), got {err:?}"
    );
    assert_eq!(
        broker.state_of(&real_action),
        None,
        "the real action must remain unclaimed -- no phantom approval was created"
    );
}

// ---------------------------------------------------------------------------
// Scenario 2 -- Tampered receipt: flip a byte in a receipt/digest on a
// scratch copy; confirm replay/verify detects and refuses.
//
// NOT reachable via the CLI binary today (F25 wiring is BLOCKED -- see
// module doc). Exercises the real `f25_receipts_replay` replay/verify
// mechanism directly, over this case's own REAL Stage-2 artifacts (F02
// admitted case Turtle, the real derived-obligation Turtle, the real
// domain/hook-catalog Turtle). Mirrors the dogfood-lifecycle-pack tamper
// pattern (two INDEPENDENT digest flips, each independently detected) --
// adapted to F25's six CTQ digest kinds rather than a literal
// chain_hash/prev_chain_hash field pair, since F25's `Digest` type has no
// public mutable constructor (by design: "no receipt is ornamental" -- see
// that module's own doc comment), so tampering is applied to a SCRATCH COPY
// of the underlying material text (which is what a real digest is a
// content-address OF), the only lawful way an external caller can force a
// different digest through the real hashing code, not a hand-forged hash.
// ---------------------------------------------------------------------------

#[test]
fn scenario2_tampered_material_is_detected_and_refused_on_independent_replay() {
    let derived = derive_obligations(HOOK_TTL, CASE_TTL);
    let recorded_materials = Materials {
        source: CASE_TTL.to_string(),
        query: HOOK_TTL.to_string(),
        template: DOMAIN_TTL.to_string(),
        program: ACTION_HOOK_PACK_TTL.to_string(),
        event: "crown-bribery-case-run-1-intake".to_string(),
        output: derived.turtle.clone(),
    };
    let recorded = receipt_builder::build(&recorded_materials)
        .expect("real crown-bribery-case artifacts must build a real F25 receipt");

    // -- Tamper 1: flip a byte in the OUTPUT material (a scratch copy of the
    // real derived-obligation Turtle) -- the "chain_hash" analog.
    let mut tampered_output_materials = recorded_materials.clone();
    tampered_output_materials.output = flip_one_byte(&recorded_materials.output);
    let err = independent_verifier::verify(&recorded, || Ok(tampered_output_materials.clone()))
        .expect_err(
            "a one-byte-flipped OUTPUT material must be detected on replay, not silently accepted",
        );
    match err {
        ReceiptReplayRefused::EquivalenceMismatch { kind, .. } => {
            assert_eq!(kind, DigestKind::Output);
        }
        other => panic!("expected EquivalenceMismatch{{kind: Output, ..}}, got {other:?}"),
    }

    // -- Tamper 2: flip a byte in the SOURCE material (a scratch copy of the
    // real admitted case Turtle) -- the "prev_chain_hash" analog (the prior
    // state this receipt is chained from).
    let mut tampered_source_materials = recorded_materials.clone();
    tampered_source_materials.source = flip_one_byte(&recorded_materials.source);
    let err = independent_verifier::verify(&recorded, || Ok(tampered_source_materials.clone()))
        .expect_err(
            "a one-byte-flipped SOURCE material must be detected on replay, not silently accepted",
        );
    match err {
        ReceiptReplayRefused::EquivalenceMismatch { kind, .. } => {
            assert_eq!(kind, DigestKind::Source);
        }
        other => panic!("expected EquivalenceMismatch{{kind: Source, ..}}, got {other:?}"),
    }

    // Positive control: an UNTAMPERED replay of the identical real
    // materials must verify clean -- proves the two refusals above are
    // genuine tamper detection, not a broken/always-refusing comparator.
    let (_, report) = independent_verifier::verify(&recorded, || Ok(recorded_materials.clone()))
        .expect("an untampered replay of the real materials must verify clean");
    assert!(report.receipt_root_matched);
    assert_eq!(report.matched_kinds.len(), 6);
}

/// Flips the low bit of the first ASCII-alphanumeric byte found -- a real,
/// minimal one-byte mutation of a scratch `String` copy (never the
/// original), analogous to "flip a byte in chain_hash/prev_chain_hash".
///
/// # Complexity
/// O(n) to locate the first mutable byte.
fn flip_one_byte(material: &str) -> String {
    let mut bytes = material.as_bytes().to_vec();
    let idx = bytes
        .iter()
        .position(|b| b.is_ascii_alphanumeric())
        .expect("material must contain at least one alphanumeric byte to tamper");
    bytes[idx] ^= 0x01;
    String::from_utf8(bytes).expect("flipping one bit of an ASCII byte stays valid UTF-8")
}

// ---------------------------------------------------------------------------
// Scenario 3 -- Missing evidence: an obligation whose required
// evidence-type is never supplied; confirm a typed BLOCKED/REFUSED result
// naming the exact missing evidence, never a fabricated closure.
//
// Directly against the real CLI chain's own Stage-2 mechanism
// (`evidence_type_for_obligation`, the exact function
// `crown-bribery-case.rs` itself calls before manufacturing PDDL8 text).
// ---------------------------------------------------------------------------

#[test]
fn scenario3_obligation_with_no_catalog_evidence_type_is_typed_refused_not_fabricated() {
    let derived = derive_obligations(HOOK_TTL, CASE_TTL);
    assert!(
        derived
            .local_names
            .iter()
            .any(|n| n == "verify-transaction-authenticity"),
        "this test targets the real 'verify-transaction-authenticity' obligation \
         hook.ttl derives for case.ttl; got {:?}",
        derived.local_names
    );

    // Real hook.ttl content, minus ONLY the one
    // `sc:requiresEvidenceType sc:etype-card-statement` catalog fact for
    // `verify-transaction-authenticity` -- everything else (including the
    // obligation's own `a sc:Obligation` declaration and the hook's SPARQL
    // derivation logic) is untouched.
    let mutated_hook_ttl = HOOK_TTL.replacen(
        "sc:requiresEvidenceType sc:etype-card-statement ;\n    rdfs:comment \"Confirm the alleged card transaction is real and matches the reported amount/date.\" .",
        "rdfs:comment \"Confirm the alleged card transaction is real and matches the reported amount/date.\" .",
        1,
    );
    assert_ne!(
        mutated_hook_ttl, HOOK_TTL,
        "the targeted sc:requiresEvidenceType fact must actually have been removed"
    );

    // The hook still derives the SAME 3 obligations (the SPARQL trigger
    // pattern does not depend on the evidence-type catalog) -- proving the
    // refusal below is precisely localized to the missing catalog fact, not
    // a side effect of a broken fixture.
    let mutated_derived = derive_obligations(&mutated_hook_ttl, CASE_TTL);
    assert_eq!(mutated_derived.local_names, derived.local_names);

    for obligation in &mutated_derived.local_names {
        let result = evidence_type_for_obligation(&mutated_hook_ttl, obligation);
        if obligation == "verify-transaction-authenticity" {
            let err = result.expect_err(
                "an obligation whose evidence-type catalog fact was removed must be a typed refusal, never a fabricated evidence type",
            );
            assert_eq!(
                err,
                ObligationEvidenceRefused::EvidenceTypeCatalogMissing {
                    obligation_local_name: "verify-transaction-authenticity".to_string(),
                },
                "refusal must name the EXACT missing obligation"
            );
        } else {
            result.unwrap_or_else(|e| {
                panic!("untouched obligation {obligation:?} must still resolve its real evidence type, got {e:?}")
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 4 -- Reused boundary/dispatch request: replay the identical
// dispatch/actuation request twice; confirm idempotency/dedup refuses or
// safely no-ops the duplicate rather than double-actuating.
//
// Part A: directly against the real CLI chain's own F02 admission (L7
// idempotency-by-correlation_id). Part B: directly against the real
// `f18_broker_law::Broker` (the Rust in-process analog of
// `arazzo_runner_broker.erl`'s `ets:insert_new/2` atomic dedup guard --
// NOT reachable via the CLI binary today, see module doc).
// ---------------------------------------------------------------------------

#[test]
fn scenario4a_f02_replay_same_correlation_id_same_payload_safely_no_ops() {
    let policy = build_policy();
    let ledger = AdmissionLedger::new();

    let first = admit_observation(
        &policy,
        &ledger,
        raw_observation("corr-scenario4a-dedup", CASE_TTL.to_string()),
    )
    .expect("first admission must succeed");
    let second = admit_observation(
        &policy,
        &ledger,
        raw_observation("corr-scenario4a-dedup", CASE_TTL.to_string()),
    )
    .expect("byte-identical replay under the same correlation_id must safely no-op, not refuse");

    assert_eq!(
        first, second,
        "a safe no-op duplicate must return the SAME receipt, not a second/different one"
    );
    assert_eq!(
        ledger.len().unwrap(),
        1,
        "no second ledger row -- the duplicate was deduped, not double-admitted"
    );
}

#[test]
fn scenario4a_f02_reused_correlation_id_with_different_payload_is_refused_not_double_actuated() {
    let policy = build_policy();
    let ledger = AdmissionLedger::new();

    admit_observation(
        &policy,
        &ledger,
        raw_observation("corr-scenario4a-conflict", CASE_TTL.to_string()),
    )
    .expect("first admission must succeed");

    // Same correlation_id, genuinely DIFFERENT payload (an extra assertion
    // appended) -- a forged/stale "replay" of a different actuation reusing
    // the same dispatch identity. Must be refused, never silently admitted
    // as if it were the same request.
    let mut different_payload = CASE_TTL.to_string();
    different_payload.push_str(&format!(
        "\n<{SUBJECT}> <{SC}caseStatus> \"forged-reused-dispatch-attempt\" .\n"
    ));
    let err = admit_observation(
        &policy,
        &ledger,
        raw_observation("corr-scenario4a-conflict", different_payload),
    )
    .expect_err(
        "reusing a correlation_id with a genuinely different payload must refuse, never double-actuate",
    );
    assert!(
        matches!(err, ObservationAdmissionRefused::CorrelationConflict { .. }),
        "expected CorrelationConflict, got {err:?}"
    );
    assert_eq!(
        ledger.len().unwrap(),
        1,
        "the conflicting reuse attempt must not create a second ledger row"
    );
}

#[test]
fn scenario4b_broker_duplicate_idempotency_claim_for_the_same_action_is_refused() {
    let policy = build_policy();
    let ledger = AdmissionLedger::new();
    let admission = admit_observation(
        &policy,
        &ledger,
        raw_observation("corr-scenario4b-broker-dedup", CASE_TTL.to_string()),
    )
    .expect("real admission must succeed");

    let broker = Broker::new(BrokerSecret::new([0x52; 32]));
    let action = ActionId::new(
        "crown-bribery-case-workflow",
        "supply-evidence",
        admission.receipt_hash.clone(),
    );
    let (_, token) = broker.authorize(&action);

    broker
        .claim_idempotency(action.clone(), token)
        .expect("first claim (the real dispatch) must succeed");
    let err = broker
        .claim_idempotency(action.clone(), token)
        .expect_err("a replayed/duplicate dispatch request for the SAME action must be refused, never double-actuated");
    match err {
        UnreceiptedActuationRefused::DuplicateIdempotencyClaim { existing_state, .. } => {
            assert_eq!(existing_state, BrokerState::IdempotencyClaimed);
        }
        other => panic!("expected DuplicateIdempotencyClaim, got {other:?}"),
    }
    // Still exactly one claimed entry -- the duplicate did not fork state.
    assert_eq!(
        broker.state_of(&action),
        Some(BrokerState::IdempotencyClaimed)
    );
}

// ---------------------------------------------------------------------------
// Scenario 5 -- Unknown capability: an action/step outside the admitted
// plan's known (hook-)capability set; confirm typed refusal.
//
// Directly against the real CLI chain's own F08 `run_pipeline` (Action-Hook
// Binder, stage 3) -- the same real function `crown-bribery-case.rs`
// itself calls. Mirrors `cng`'s CNG_R31 `ActionNotNextApprovedStep`
// "action not in the admitted/approved set" semantics
// (`crates/cng/src/plan_approval.rs`), realized here through F08's OWN
// admitted-capability boundary rather than cng's plan-approval module
// (crown-bribery-case does not use cng's plan_approval at all -- this is
// the equivalent real gate inside the chain actually under test).
// ---------------------------------------------------------------------------

#[test]
fn scenario5_grounded_action_with_no_registered_capability_is_refused_by_action_hook_binder() {
    let manufactured = manufacture_real_case_pddl();

    // Positive control: the REAL, full ACTION_HOOK_PACK_TTL (byte-identical
    // to the CLI's own) covers every grounded action -- run_pipeline must
    // succeed, proving the mutation below is what causes the refusal below,
    // not an unrelated fixture problem.
    let case_id_ok = "crown-bribery-adversarial-s5-control";
    let ok_graph = f08_graph(&manufactured, ACTION_HOOK_PACK_TTL);
    run_pipeline(&ok_graph, case_id_ok).expect(
        "the real, full ACTION_HOOK_PACK_TTL must let run_pipeline succeed (positive control)",
    );

    // Remove ONLY the "judge" action's hook capability -- one grounded PDDL8
    // action (`judge`) this real plan needs now has NO registered capability
    // binding it, i.e. it is outside the admitted plan's known capability
    // set.
    let start = ACTION_HOOK_PACK_TTL
        .find("ex:hook-judge a kh:Hook ;")
        .expect("ACTION_HOOK_PACK_TTL must declare ex:hook-judge");
    let rest = &ACTION_HOOK_PACK_TTL[start..];
    let end_offset = rest
        .find(" .\n")
        .expect("ex:hook-judge's declaration must be terminated with ' .'");
    let mutated_hook_pack = format!(
        "{}{}",
        &ACTION_HOOK_PACK_TTL[..start],
        &ACTION_HOOK_PACK_TTL[start + end_offset + 3..]
    );
    assert!(
        !mutated_hook_pack.contains("ex:hook-judge"),
        "the judge capability must actually have been removed"
    );

    let case_id_refused = "crown-bribery-adversarial-s5-refused";
    let refused_graph = f08_graph(&manufactured, &mutated_hook_pack);
    let err = run_pipeline(&refused_graph, case_id_refused).expect_err(
        "a grounded action with no registered hook capability must refuse, never silently skip binding",
    );
    match err {
        PlanningRefusal::NoAdmissiblePlan { stage, reason } => {
            assert_eq!(stage, "ActionHookBinder");
            assert!(
                reason.contains("judge"),
                "refusal must name the specific unbound action, got: {reason}"
            );
        }
        other => {
            panic!("expected NoAdmissiblePlan{{stage: \"ActionHookBinder\", ..}}, got {other:?}")
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 6 -- Bounded planning result: force the PDDL8 planner to hit a
// real structural bound and confirm it is reported as BOUNDED
// (`Pddl8Error::BoundExceeded`, surfaced verbatim as
// `Refusal::Underlying`), NEVER folded into `Refusal::NoAdmissiblePlan`
// (the "search exhausted" case) -- the hard invariant.
//
// Directly against the real CLI chain's own F08 `run_pipeline` /
// `projector::project_and_resolve` -> `bcinr_pddl::parse::domain_from_pddl`
// -- the exact real function `crown-bribery-case.rs` itself calls (via
// `run_pipeline`), with a hand-authored domain text that exceeds
// `wasm4pm_compat::pddl::PDDL8_MAX_ARITY` (confirmed this session,
// `/Users/sac/wasm4pm-compat/wasm4pm-core/src/pddl.rs:32`: `pub const
// PDDL8_MAX_ARITY: usize = 8;`) by declaring a 9-ary predicate.
// ---------------------------------------------------------------------------

#[test]
fn scenario6_predicate_arity_bound_is_reported_bounded_never_exhausted() {
    const OVER_ARITY_DOMAIN: &str = r#"
(define (domain crown-bribery-case-bound-probe)
  (:requirements :strips)
  (:predicates (over-arity ?a ?b ?c ?d ?e ?f ?g ?h ?i) (goal-reached))
  (:action noop
    :parameters ()
    :precondition (goal-reached)
    :effect (and (goal-reached))))
"#;
    const MINIMAL_PROBLEM: &str = r#"
(define (problem crown-bribery-case-bound-probe-problem)
  (:domain crown-bribery-case-bound-probe)
  (:objects)
  (:init (goal-reached))
  (:goal (and (goal-reached))))
"#;

    let graph = vec![
        AdmittedTriple {
            subject: SUBJECT.to_string(),
            predicate: PDDL_DOMAIN_PREDICATE.to_string(),
            object_literal: OVER_ARITY_DOMAIN.to_string(),
        },
        AdmittedTriple {
            subject: SUBJECT.to_string(),
            predicate: PDDL_PROBLEM_PREDICATE.to_string(),
            object_literal: MINIMAL_PROBLEM.to_string(),
        },
        AdmittedTriple {
            subject: SUBJECT.to_string(),
            predicate: HOOK_PACK_PREDICATE.to_string(),
            object_literal: ACTION_HOOK_PACK_TTL.to_string(),
        },
    ];

    let err = run_pipeline(&graph, "crown-bribery-adversarial-s6").expect_err(
        "a 9-ary predicate (PDDL8_MAX_ARITY = 8) must refuse -- it must never silently ground/plan",
    );

    match &err {
        PlanningRefusal::Underlying {
            stage,
            source:
                Pddl8Error::BoundExceeded {
                    what,
                    limit,
                    got,
                },
        } => {
            assert_eq!(*stage, "DomainResolver");
            assert_eq!(*what, "predicate arity");
            assert_eq!(*limit, 8);
            assert_eq!(*got, 9);
        }
        other => panic!(
            "expected Refusal::Underlying{{source: Pddl8Error::BoundExceeded{{what: \"predicate arity\", limit: 8, got: 9}}, ..}}, got {other:?}"
        ),
    }
    // The hard invariant: a structural bound violation must NEVER be
    // reported as the (unrelated) "search exhausted" refusal.
    assert!(
        !matches!(err, PlanningRefusal::NoAdmissiblePlan { .. }),
        "BoundExceeded must be reported as Bounded (Refusal::Underlying), never folded into \
         NoAdmissiblePlan/'exhausted'"
    );
    let display = err.to_string();
    assert!(
        display.contains("BoundExceeded") || display.contains("bound exceeded"),
        "refusal text must name the bound, not describe search exhaustion: {display}"
    );
}

// ---------------------------------------------------------------------------
// Scenario 7 -- Broker-bypass attempt: try to reach the dispatch/actuation
// path WITHOUT going through the broker's admission checks; confirm it is
// refused ("the broker is the only mutating DO path").
//
// NOT reachable via the CLI binary today (F18 wiring is BLOCKED -- see
// module doc). Exercises the real `f18_broker_law::Broker` directly: two
// independent bypass attempts, (a) calling the dispatch/actuation stage
// with NO prior ledger entry at all, and (b) calling it after standing
// verification + authorization but skipping the idempotency-claim and
// correlation-binding gates.
// ---------------------------------------------------------------------------

#[test]
fn scenario7a_actuate_with_no_prior_broker_admission_is_refused() {
    let broker = Broker::new(BrokerSecret::new([0x53; 32]));
    let action = ActionId::new(
        "crown-bribery-case-workflow",
        "clear-transaction-obligation",
        "bypass-attempt-no-ledger-entry",
    );
    // No verify_standing, no authorize, no claim_idempotency, no
    // bind_correlation -- attempt to run the dispatch/actuation closure
    // directly, as an external actuator trying to reach the "DO" path
    // around the broker entirely.
    let mut dispatched = false;
    let err = broker
        .actuate(&action, || {
            dispatched = true;
            b"unauthorized-side-effect".to_vec()
        })
        .expect_err("actuate() must refuse an action with no prior broker admission at all");
    assert!(
        matches!(err, UnreceiptedActuationRefused::UnknownAction { .. }),
        "expected UnknownAction, got {err:?}"
    );
    assert!(
        !dispatched,
        "the dispatch closure (the real-world side effect) must NEVER run for a bypass attempt"
    );
}

#[test]
fn scenario7b_actuate_after_claim_but_skipping_correlation_binder_is_refused() {
    let broker = Broker::new(BrokerSecret::new([0x54; 32]));
    let action = ActionId::new(
        "crown-bribery-case-workflow",
        "clear-authorization-obligation",
        "bypass-attempt-skip-correlation",
    );
    let (_, token) = broker.authorize(&action);
    broker
        .claim_idempotency(action.clone(), token)
        .expect("idempotency claim itself is lawful");
    assert_eq!(
        broker.state_of(&action),
        Some(BrokerState::IdempotencyClaimed)
    );

    // Attempt to jump straight to actuation, skipping the Correlation
    // Binder gate entirely -- a partial bypass, not a full one.
    let mut dispatched = false;
    let err = broker
        .actuate(&action, || {
            dispatched = true;
            b"unauthorized-side-effect".to_vec()
        })
        .expect_err("actuate() must refuse when the Correlation Binder gate was skipped");
    match err {
        UnreceiptedActuationRefused::UnlawfulTransition { from, to, .. } => {
            assert_eq!(from, BrokerState::IdempotencyClaimed);
            assert_eq!(to, BrokerState::Actuating);
        }
        other => panic!("expected UnlawfulTransition, got {other:?}"),
    }
    assert!(
        !dispatched,
        "the dispatch closure must NEVER run when the correlation gate was bypassed"
    );
    // The ledger entry is untouched (still IdempotencyClaimed) -- the
    // bypass attempt did not corrupt real broker state.
    assert_eq!(
        broker.state_of(&action),
        Some(BrokerState::IdempotencyClaimed)
    );
}
