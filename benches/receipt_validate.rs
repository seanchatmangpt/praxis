//! Criterion benchmark for `praxis_core::ReceiptValidator::validate`.
//!
//! CPHY_ROADMAP's Phase 2 target: receipt validation must run in well under
//! 5ms (5,000,000 ns) for ~100 records. This benchmark builds pre-chained
//! ledgers of increasing size (100/500/1000 records) and measures
//! `ReceiptValidator::validate`'s wall-clock cost at each size, reporting
//! ns/iteration via Criterion's standard output — all stages are pure BLAKE3
//! + integer arithmetic (no I/O), so the target is expected to be met with
//! several orders of magnitude of headroom.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use praxis_core::{
    law::ReceiptMeta,
    lifecycle::Raw,
    receipt_record::ReceiptRecord,
    receipt_validator::{ReceiptValidator, SystemClock},
    Admit, DefaultLaw, Judge, LawObject,
};

/// If built with `--features law-signed`, `receipt_with_record` signs the
/// chain hash and fails closed without a key; set a fixed one so the
/// benchmark can build its ledger regardless of which features are active.
#[cfg(feature = "law-signed")]
fn ensure_signing_key() {
    if std::env::var("PRAXIS_SIGNING_KEY").is_err() {
        std::env::set_var(
            "PRAXIS_SIGNING_KEY",
            "d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaecedeeeff01",
        );
    }
}
#[cfg(not(feature = "law-signed"))]
fn ensure_signing_key() {}

/// Build `n` chained `ReceiptRecord`s via the real `receipt_with_record`
/// path (judge -> admit -> receipt), deterministic in `ts_ns`/`instruction_id`.
fn build_ledger(n: u64) -> Vec<ReceiptRecord> {
    ensure_signing_key();
    let mut records = Vec::with_capacity(n as usize);
    let mut prev = [0u8; 32];
    for i in 1..=n {
        let raw = LawObject::<serde_json::Value, Raw, DefaultLaw>::new(
            serde_json::json!({"i": i}),
            vec![],
        );
        // `Judge::judge`'s `Err` variant is the raw `LawObject` itself, which
        // intentionally does not derive `Debug` (its phantom stage/law
        // markers aren't all `Debug`) — so unwrap via `match`, not
        // `.expect()`, mirroring `praxis-core`'s own `default_law.rs` tests
        // and `tests/receipt_lane.rs`'s `admitted_value` helper.
        let validated = match DefaultLaw::judge(raw) {
            Ok(v) => v,
            Err(_) => panic!("no obligations must always judge cleanly"),
        };
        let admitted = DefaultLaw::admit(validated).expect("green andon must admit");
        let meta = ReceiptMeta {
            instruction_id: i,
            activity_idx: 0,
            node_kind: 0,
            ts_ns: Some(i * 1_000_000),
            ..Default::default()
        };
        let (receipted, record) =
            admitted.receipt_with_record(&prev, meta).expect("receipt_with_record");
        prev = *receipted.chain_hash().expect("chain hash set");
        records.push(record);
    }
    records
}

fn bench_receipt_validate(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_validate");

    for size in [100_u64, 500, 1000] {
        let ledger = build_ledger(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &ledger, |b, ledger| {
            b.iter(|| {
                let verdict = ReceiptValidator::validate(black_box(ledger), &SystemClock);
                assert!(verdict.ok, "benchmark ledger must validate clean");
                black_box(verdict);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_receipt_validate);
criterion_main!(benches);
