#![cfg(test)]

//! Workday hook broker tests (PROJ-612/613): the determinism spike, the
//! zero-unreceipted-actuation negative (CNG_R13), and the Dialect Registry
//! closed-shape negative (CNG_R14). All Turtle enters from on-disk pack /
//! registry / fixture files; all SPARQL from the on-disk query set.

use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;

use super::WorkdayHookBroker;
use crate::bench::templates::QuerySet;
use crate::powl::CngRefusal;

/// Crate-root path helper. O(1).
fn crate_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Standard broker over the real pack + registry. O(pack + registry).
fn real_broker() -> WorkdayHookBroker {
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    WorkdayHookBroker::new(
        &crate_path("hooks/dialect-registry.ttl"),
        &crate_path("hooks/dialect-registry.shape.ttl"),
        &[
            crate_path("hooks/workday-pack.ttl"),
            crate_path("hooks/workday-pack-2.ttl"),
        ],
        &queries,
    )
    .expect("broker constructs over the real pack and registry")
}

test!(
    spike_delta_hook_fires_and_receipts_are_byte_deterministic,
    {
        // Arrange: two independent brokers over the same on-disk pack — the
        // PROJ-612 determinism spike. Verifies (a) `kind delta` hooks fire
        // through TripleStore::materialize, (b) the receipt is byte-identical
        // across two runs of the same actuation.
        let mut broker_a = real_broker();
        let mut broker_b = real_broker();

        // Act: the identical transition actuated in both brokers.
        let receipt_a = broker_a
            .actuate("tick-0001", "email-routing", 1, 0)
            .expect("delta hook fires and receipts");
        let receipt_b = broker_b
            .actuate("tick-0001", "email-routing", 1, 0)
            .expect("delta hook fires and receipts on the second run");

        // Assert: content-derived, byte-deterministic evidence.
        assert_eq!(receipt_a.hook_name, "email-routing");
        assert!(!receipt_a.delta_hash.is_empty());
        assert!(!receipt_a.idempotency_key.is_empty());
        assert_eq!(receipt_a.delta_hash, receipt_b.delta_hash);
        assert_eq!(receipt_a.idempotency_key, receipt_b.idempotency_key);
        assert_eq!(
            broker_a.run_hook_hash().expect("hook hash a"),
            broker_b.run_hook_hash().expect("hook hash b")
        );
        // A DIFFERENT transition yields a DIFFERENT delta hash (content-derived,
        // never canned).
        let receipt_c = broker_a
            .actuate("tick-0002", "email-routing", 2, 0)
            .expect("second transition receipts");
        assert_ne!(receipt_a.delta_hash, receipt_c.delta_hash);
    }
);

test!(broker_covers_every_bench_category, {
    // Arrange: the real pack.
    let broker = real_broker();

    // Act: sorted hook names from the pack scan.
    let names = broker.hook_names().to_vec();

    // Assert: exactly the category set, one hook per category.
    let mut expected: Vec<String> = crate::bench::CATEGORIES
        .iter()
        .map(|c| c.to_string())
        .collect();
    expected.sort();
    assert_eq!(names, expected);
});

test!(missing_category_hook_refuses_cng_r13, {
    // Arrange: fixture pack with the planning hook removed.
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");
    let mut broker = WorkdayHookBroker::new(
        &crate_path("hooks/dialect-registry.ttl"),
        &crate_path("hooks/dialect-registry.shape.ttl"),
        &[crate_path(
            "tests/fixtures/negative/workday-pack-missing-planning.ttl",
        )],
        &queries,
    )
    .expect("fixture pack admits (it is a valid, smaller pack)");

    // Act: actuate a planning transition against the hookless category.
    let result = broker.actuate("tick-0003", "planning", 3, 0);

    // Assert: typed CNG_R13 naming the workflow and category.
    match result {
        Err(CngRefusal::UnreceiptedActuation { workflow, category }) => {
            assert_eq!(workflow, "tick-0003");
            assert_eq!(category, "planning");
            assert_eq!(
                CngRefusal::UnreceiptedActuation { workflow, category }.code(),
                "CNG_R13"
            );
        }
        other => panic!("expected UnreceiptedActuation, got {other:?}"),
    }
});

test!(registry_missing_field_refuses_cng_r14, {
    // Arrange: fixture registry whose one entry lacks dreg:receiptSchema.
    let queries = QuerySet::load(&QuerySet::default_dir()).expect("query set loads");

    // Act: broker construction runs the registry gate BEFORE any tick.
    let result = WorkdayHookBroker::new(
        &crate_path("tests/fixtures/negative/dialect-registry-missing-field.ttl"),
        &crate_path("hooks/dialect-registry.shape.ttl"),
        &[crate_path("hooks/workday-pack.ttl")],
        &queries,
    );

    // Assert: typed CNG_R14 naming the entry and the missing field.
    match result {
        Err(CngRefusal::DialectRegistryRefused { entry, missing }) => {
            assert!(
                entry.contains("broken-entry"),
                "refusal names the violating entry, got {entry}"
            );
            assert!(
                missing.contains("receiptSchema"),
                "refusal names the missing field, got {missing}"
            );
            assert_eq!(
                CngRefusal::DialectRegistryRefused { entry, missing }.code(),
                "CNG_R14"
            );
        }
        other => panic!("expected DialectRegistryRefused, got {other:?}"),
    }
});

test!(real_registry_passes_the_closed_shape_gate, {
    // Arrange + Act: the real registry and shape (construction is the gate).
    let broker = real_broker();

    // Assert: the pack admitted and all sixteen hooks are present (one per
    // CATEGORIES entry — v26.7.12/13 Stage 2 added "soc2-audit" as the
    // 16th).
    assert_eq!(broker.hook_names().len(), crate::bench::CATEGORIES.len());
    assert_eq!(broker.actuations(), 0);
});
