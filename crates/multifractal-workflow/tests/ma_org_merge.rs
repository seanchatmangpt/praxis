//! Integration test for `multifractal_workflow::f31_org_merge` -- the first real,
//! working instance of `docs/standing/BOOTSTRAP_COLD_START_LIMITATIONS.md` item 14
//! (multi-org first-merge bootstrap).
//!
//! Drives two concrete, named fictional entities from
//! `packs/ma-case-study-pack/fixtures/` (Meridian Holdings Group, Inc. as
//! Acquirer; Corvantis Systems, Inc. as Target) through:
//!
//! 1. Two SEPARATE F02 admissions (`multifractal_workflow::f31_org_merge::
//!    admit_org_graph`, itself a thin wrapper over the real
//!    `f02_observation_admission::admit_observation`), each with its own
//!    `AdmissionPolicy`, its own `AdmissionLedger`, and its own receipt hash --
//!    proving "each graph was validated under its own shapes... its own receipt
//!    chain" is not skipped.
//! 2. A real, no-conflict merge (`merge_org_graphs`) that produces one
//!    re-validated fused graph.
//! 3. A deliberately-constructed colliding-identifier adversarial case
//!    (`target-org-colliding.ttl`, which reuses Acquirer's own Azure resource IRIs
//!    with different property values) that is DETECTED and REFUSED by the merge
//!    step, not silently corrupted.
//!
//! Run via `just multifractal-workflow-test-org-merge` (never bare `cargo`).

use std::collections::{BTreeMap, BTreeSet};

use multifractal_workflow::f02_observation_admission::{
    AdmissionLedger, AdmissionPolicy, RawObservation,
};
use multifractal_workflow::f31_org_merge::{admit_org_graph, merge_org_graphs, OrgMergeRefused};

const ACQUIRER_TTL: &str =
    include_str!("../../../packs/ma-case-study-pack/fixtures/acquirer-org.ttl");
const ACQUIRER_SHAPES_TTL: &str =
    include_str!("../../../packs/ma-case-study-pack/fixtures/acquirer-org-shapes.ttl");
const TARGET_TTL: &str = include_str!("../../../packs/ma-case-study-pack/fixtures/target-org.ttl");
const TARGET_SHAPES_TTL: &str =
    include_str!("../../../packs/ma-case-study-pack/fixtures/target-org-shapes.ttl");
const TARGET_COLLIDING_TTL: &str =
    include_str!("../../../packs/ma-case-study-pack/fixtures/target-org-colliding.ttl");

const ACQUIRER_SOURCE: &str = "acquirer-ea-intake";
const ACQUIRER_PRINCIPAL: &str = "https://intake.acquirer-meridian.example.org/ea-intake-1";
const ACQUIRER_SUBJECT: &str =
    "https://acquirer-meridian.example.org/entity/meridian-holdings-group";

const TARGET_SOURCE: &str = "target-ea-intake";
const TARGET_PRINCIPAL: &str = "https://intake.target-corvantis.example.org/ea-intake-1";
const TARGET_SUBJECT: &str = "https://target-corvantis.example.org/entity/corvantis-systems";

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const SKOS_NOTATION: &str = "http://www.w3.org/2004/02/skos/core#notation";
const DCTERMS_DESCRIPTION: &str = "http://purl.org/dc/terms/description";

/// Builds the F02 [`AdmissionPolicy`] one organization would independently stand
/// up for its own EA-intake source. `known_principals`/`authorized_predicates`
/// are scoped exactly to what F02's Gate 1/3/5 check against `declared_subject`
/// (the org root) -- per `f02_observation_admission::admit_observation`'s own
/// implementation, Gate 3 (Authority) and Gate 5 (Semantic Conformance) only
/// inspect triples whose subject is the declared subject itself, so predicates
/// used only on the TOGAF capability blank nodes or the Azure resource IRIs
/// (`prov:wasAttributedTo`, `aztf:*`, etc.) are correctly outside this policy's
/// authorized-predicate/vocabulary sets and still admit cleanly -- only Gate 4
/// (real SHACL, via `shapes_turtle`) inspects the whole payload.
fn org_policy(source_id: &str, principal_iri: &str, shapes_turtle: &str) -> AdmissionPolicy {
    let mut known_principals = BTreeMap::new();
    known_principals.insert(source_id.to_string(), principal_iri.to_string());

    let mut authorized = BTreeSet::new();
    authorized.insert(RDF_TYPE.to_string());
    authorized.insert(SKOS_NOTATION.to_string());
    authorized.insert(DCTERMS_DESCRIPTION.to_string());
    let mut authorized_predicates = BTreeMap::new();
    authorized_predicates.insert(source_id.to_string(), authorized);

    AdmissionPolicy::new(
        known_principals,
        authorized_predicates,
        vec![
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string(),
            "http://www.w3.org/2004/02/skos/core#".to_string(),
            "http://purl.org/dc/terms/".to_string(),
        ],
        vec!["https://".to_string()],
        shapes_turtle,
    )
    .expect("packs/ma-case-study-pack/fixtures/*-shapes.ttl must be valid SHACL")
}

fn acquirer_observation(correlation_id: &str, payload: &str) -> RawObservation {
    RawObservation {
        correlation_id: correlation_id.to_string(),
        source_id: ACQUIRER_SOURCE.to_string(),
        declared_subject: ACQUIRER_SUBJECT.to_string(),
        payload_turtle: payload.to_string(),
    }
}

fn target_observation(correlation_id: &str, payload: &str) -> RawObservation {
    RawObservation {
        correlation_id: correlation_id.to_string(),
        source_id: TARGET_SOURCE.to_string(),
        declared_subject: TARGET_SUBJECT.to_string(),
        payload_turtle: payload.to_string(),
    }
}

/// (a) Each entity graph admits independently, through the real F02 6-gate
/// pipeline, each producing its OWN receipt hash under its OWN policy/ledger.
#[test]
fn acquirer_and_target_admit_independently_with_their_own_receipts() {
    let acquirer_policy = org_policy(ACQUIRER_SOURCE, ACQUIRER_PRINCIPAL, ACQUIRER_SHAPES_TTL);
    let acquirer_ledger = AdmissionLedger::new();
    let acquirer = admit_org_graph(
        &acquirer_policy,
        &acquirer_ledger,
        acquirer_observation("corr-acquirer-ea-1", ACQUIRER_TTL),
        ACQUIRER_SHAPES_TTL.to_string(),
        "acquirer",
    )
    .expect("Meridian Holdings Group's own EA/Azure graph must admit under its own shapes");

    let target_policy = org_policy(TARGET_SOURCE, TARGET_PRINCIPAL, TARGET_SHAPES_TTL);
    let target_ledger = AdmissionLedger::new();
    let target = admit_org_graph(
        &target_policy,
        &target_ledger,
        target_observation("corr-target-ea-1", TARGET_TTL),
        TARGET_SHAPES_TTL.to_string(),
        "target",
    )
    .expect("Corvantis Systems' own EA/Azure graph must admit under its own shapes");

    // Each organization's admission is a fully independent, self-contained event:
    // distinct receipt hashes, distinct ledgers, one entry each.
    assert_ne!(acquirer.receipt.receipt_hash, target.receipt.receipt_hash);
    assert_eq!(acquirer.receipt.subject_iri, ACQUIRER_SUBJECT);
    assert_eq!(target.receipt.subject_iri, TARGET_SUBJECT);
    assert_eq!(acquirer_ledger.len().expect("ledger readable"), 1);
    assert_eq!(target_ledger.len().expect("ledger readable"), 1);
    assert!(
        acquirer.triples.len() >= 8,
        "acquirer graph should carry its full TOGAF+Azure fact set"
    );
    assert!(
        target.triples.len() >= 8,
        "target graph should carry its full TOGAF+Azure fact set"
    );
}

/// (b) The merge succeeds and produces a single valid, re-validated fused graph
/// when there is no genuine conflict (acquirer-org.ttl / target-org.ttl use
/// disjoint base namespaces for every locally-minted individual).
#[test]
fn merge_of_non_colliding_org_graphs_succeeds_and_revalidates() {
    let acquirer_policy = org_policy(ACQUIRER_SOURCE, ACQUIRER_PRINCIPAL, ACQUIRER_SHAPES_TTL);
    let acquirer = admit_org_graph(
        &acquirer_policy,
        &AdmissionLedger::new(),
        acquirer_observation("corr-acquirer-happy", ACQUIRER_TTL),
        ACQUIRER_SHAPES_TTL.to_string(),
        "acquirer",
    )
    .expect("acquirer admission must pass");

    let target_policy = org_policy(TARGET_SOURCE, TARGET_PRINCIPAL, TARGET_SHAPES_TTL);
    let target = admit_org_graph(
        &target_policy,
        &AdmissionLedger::new(),
        target_observation("corr-target-happy", TARGET_TTL),
        TARGET_SHAPES_TTL.to_string(),
        "target",
    )
    .expect("target admission must pass");

    let merge_receipt =
        merge_org_graphs(&acquirer, &target).expect("no genuine collision: merge must succeed");

    // Merged triple count is the union with no dedup loss expected (disjoint
    // graphs): exactly acquirer's + target's own triple counts.
    assert_eq!(
        merge_receipt.merged_triple_count,
        acquirer.triples.len() + target.triples.len()
    );
    assert_eq!(
        merge_receipt.acquirer_receipt_hash,
        acquirer.receipt.receipt_hash
    );
    assert_eq!(
        merge_receipt.target_receipt_hash,
        target.receipt.receipt_hash
    );
    // The merge receipt is a NEW hash, not equal to either prior chain head --
    // an append, never an in-place rewrite of either original receipt.
    assert_ne!(
        merge_receipt.merge_receipt_hash,
        acquirer.receipt.receipt_hash
    );
    assert_ne!(
        merge_receipt.merge_receipt_hash,
        target.receipt.receipt_hash
    );
    assert!(!merge_receipt.merge_receipt_hash.is_empty());

    // Determinism: re-running the merge over the SAME two admitted graphs must
    // produce a byte-identical receipt (invariant #5: same inputs, same output).
    let merge_receipt_again =
        merge_org_graphs(&acquirer, &target).expect("re-merge of the same inputs must succeed");
    assert_eq!(merge_receipt, merge_receipt_again);
}

/// (c) The adversarial proof: `target-org-colliding.ttl` admits cleanly on its
/// own (its own root organization/TOGAF facts are unchanged and its own SHACL
/// shapes are satisfied), but at MERGE time its Azure resource IRIs -- copy-pasted
/// verbatim from `acquirer-org.ttl` with different property values -- collide
/// with the Acquirer's own facts about the SAME resource IRIs. The merge step
/// must detect and REFUSE this, not silently overwrite either organization's
/// facts.
#[test]
fn merge_detects_and_refuses_colliding_azure_resource_identifiers() {
    let acquirer_policy = org_policy(ACQUIRER_SOURCE, ACQUIRER_PRINCIPAL, ACQUIRER_SHAPES_TTL);
    let acquirer = admit_org_graph(
        &acquirer_policy,
        &AdmissionLedger::new(),
        acquirer_observation("corr-acquirer-adversarial", ACQUIRER_TTL),
        ACQUIRER_SHAPES_TTL.to_string(),
        "acquirer",
    )
    .expect("acquirer admission must pass");

    let target_policy = org_policy(TARGET_SOURCE, TARGET_PRINCIPAL, TARGET_SHAPES_TTL);
    let colliding_target = admit_org_graph(
        &target_policy,
        &AdmissionLedger::new(),
        target_observation("corr-target-colliding", TARGET_COLLIDING_TTL),
        TARGET_SHAPES_TTL.to_string(),
        "target",
    )
    .expect(
        "target-org-colliding.ttl admits standalone (its own root org facts are unchanged and \
         its Azure resources are individually well-shaped -- the collision is only visible at \
         merge time, per that fixture's own header comment)",
    );

    let err = merge_org_graphs(&acquirer, &colliding_target)
        .expect_err("colliding Azure resource IRIs must be detected and refused at merge time");

    match err {
        OrgMergeRefused::IdentifierCollision {
            ref subject,
            ref acquirer_org,
            ref target_org,
            ..
        } => {
            assert!(
                subject.starts_with("https://acquirer-meridian.example.org/azure/"),
                "collision must be reported on the shared Azure resource IRI, got: {subject}"
            );
            assert_eq!(acquirer_org, "acquirer");
            assert_eq!(target_org, "target");
        }
        other => panic!("expected IdentifierCollision, got: {other:?}"),
    }
}

/// Every [`OrgMergeRefused`] variant this test file's scenarios can reach is
/// exercised above; this test additionally proves a fresh (non-colliding) rebuild
/// of the collision case with a genuinely EMPTY payload is impossible to reach via
/// the public `admit_org_graph` entry point at all (F02's own Gate 0 refuses an
/// empty payload before this module's own `EmptyMergedGraph` branch could ever be
/// reached for any F02-admitted input) -- documenting, not just asserting, why
/// `OrgMergeRefused::EmptyMergedGraph` has no dedicated end-to-end test through the
/// public API: it is a defensive check for a state F02 admission already makes
/// unreachable, not a gap in this test file's coverage.
#[test]
fn empty_payload_is_refused_by_f02_before_merge_is_ever_reached() {
    let acquirer_policy = org_policy(ACQUIRER_SOURCE, ACQUIRER_PRINCIPAL, ACQUIRER_SHAPES_TTL);
    let mut empty_obs = acquirer_observation("corr-empty", ACQUIRER_TTL);
    empty_obs.payload_turtle = "@prefix ex: <https://example.org/vocab#> .".to_string();

    let result = admit_org_graph(
        &acquirer_policy,
        &AdmissionLedger::new(),
        empty_obs,
        ACQUIRER_SHAPES_TTL.to_string(),
        "acquirer",
    );
    assert!(
        result.is_err(),
        "an empty payload must never reach admit_org_graph's Ok arm"
    );
}
