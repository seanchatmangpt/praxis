//! The supervised-cell measurement run: honest numbers to
//! `receipts/supervised_cell.json`. `#[ignore]` by default; run with
//! `--ignored --release` to regenerate.
//!
//! Measurement discipline (the anti-2025-theatre rules, receipted here):
//! - Overhead comes from MEDIANS over multiple paired runs, never a single
//!   sample; all raw samples are published in the receipt.
//! - A flattering result self-refutes: if supervision measures faster than
//!   baseline beyond the baseline's own run-to-run spread, the harness
//!   FAILS instead of publishing a negative overhead (the 2025
//!   aggregate-masking signature).
//! - Per-member latency is reported as p50/p90/p99/worst-case — never a
//!   minimum (min-of-samples PASS was 2025 defect class B2).
//! - Every throughput figure is recomputed from (count, elapsed) by the
//!   shared helper; the composition ratio (whole cell vs sum of members)
//!   is measured, not assumed (the 84-vs-8 composition penalty).

mod common;

use std::collections::BTreeSet;
use std::time::Instant;

use common::stats;
use praxis_synthesis::cell::{run_cell, verify_cell};
use praxis_synthesis::cell_supervise::{
    run_cell_supervised, run_member_supervised, FaultScript, SupervisorPolicy,
};
use praxis_synthesis::dag::MemoCache;
use praxis_synthesis::solver8::CoreCache;

/// Paired-run count for the overhead measurement.
const RUNS: usize = 5;

#[test]
#[ignore = "measurement run; execute with --ignored --release to regenerate the receipt"]
fn supervised_cell_receipt() {
    let (n, g, templates) = (10_000, 100, 8);
    let policy = SupervisorPolicy::default();
    let zero_faults =
        FaultScript { seed: 1, transient_per_mille: 0, crashloop_template: None };

    // Paired multi-run overhead: alternate baseline/supervised so drift
    // (thermal, background load) hits both arms.
    let mut base_ns: Vec<u128> = Vec::with_capacity(RUNS);
    let mut sup0_ns: Vec<u128> = Vec::with_capacity(RUNS);
    let mut base_admitted = 0usize;
    let mut sup0_admitted = 0usize;
    for _ in 0..RUNS {
        let t0 = Instant::now();
        let (base, _) = run_cell(n, g, templates);
        base_ns.push(t0.elapsed().as_nanos());
        base_admitted = base.admitted;

        let t0 = Instant::now();
        let (sup0, _, _) =
            run_cell_supervised(n, g, templates, 1, policy, zero_faults);
        sup0_ns.push(t0.elapsed().as_nanos());
        sup0_admitted = sup0.admitted;
    }
    let base_med = stats::median(&mut base_ns.clone());
    let sup_med = stats::median(&mut sup0_ns.clone());
    let base_spread = base_ns.iter().max().expect("runs")
        - base_ns.iter().min().expect("runs");
    #[allow(clippy::cast_precision_loss)]
    let overhead_pct =
        (sup_med as f64 - base_med as f64) / (base_med as f64) * 100.0;
    // Self-refutation guard: a negative overhead beyond the baseline's own
    // spread is a measurement artifact, not a result. Fail loudly.
    assert!(
        sup_med + base_spread >= base_med,
        "flattering artifact: supervision measured faster than baseline \
         beyond run spread (sup_med={sup_med}ns base_med={base_med}ns \
         spread={base_spread}ns) — fix the measurement, don't publish it"
    );

    // Per-member latency distribution under 10% transient faults: each
    // member timed individually (fresh caches per group, mirroring the
    // cell's sharding). Verdicts from percentiles/worst-case only.
    let faults_10 =
        FaultScript { seed: 42, transient_per_mille: 100, crashloop_template: None };
    let quarantine = BTreeSet::new();
    let per = n / g;
    let mut member_ns: Vec<u128> = Vec::with_capacity(n);
    let t0 = Instant::now();
    for gi in 0..g {
        let mut memo = MemoCache::new();
        let mut cores = CoreCache::new();
        for m in 0..per {
            let t1 = Instant::now();
            let _ = run_member_supervised(
                gi * per + m,
                templates,
                &quarantine,
                policy,
                faults_10,
                &mut memo,
                &mut cores,
            );
            member_ns.push(t1.elapsed().as_nanos());
        }
    }
    let sum_members_ns: u128 = member_ns.iter().sum();
    let timed_pass_ns = t0.elapsed().as_nanos();
    let member_p50 = stats::percentile(&mut member_ns.clone(), 50.0);
    let member_p90 = stats::percentile(&mut member_ns.clone(), 90.0);
    let member_p99 = stats::percentile(&mut member_ns.clone(), 99.0);
    let member_worst = stats::percentile(&mut member_ns, 100.0);

    // Composition ratio: one whole supervised cell run (same fault script,
    // one epoch) against the sum of individually timed members. Anything
    // above ~1.0 is the priced cost of roll-ups + receipts — the 2025
    // lesson is to measure this, never to assert flatness.
    let t0 = Instant::now();
    let (comp_cell, comp_groups, _) =
        run_cell_supervised(n, g, templates, 1, policy, faults_10);
    let composed_ns = t0.elapsed().as_nanos();
    assert!(verify_cell(&comp_cell, &comp_groups));
    #[allow(clippy::cast_precision_loss)]
    let composition_ratio = composed_ns as f64 / sum_members_ns as f64;

    // Fault sweep: 1% / 10% / 50% transient rates + one crashloop template,
    // two epochs (so the MAPE-K quarantine fires and is measured).
    let mut sweep = Vec::new();
    for per_mille in [10u16, 100, 500] {
        let t0 = Instant::now();
        let (cell, groups, plans) = run_cell_supervised(
            n,
            g,
            templates,
            2,
            policy,
            FaultScript {
                seed: 42,
                transient_per_mille: per_mille,
                crashloop_template: Some(1),
            },
        );
        let ns = t0.elapsed().as_nanos();
        // Work product verified, not loop completion.
        assert!(verify_cell(&cell, &groups));
        // Unit consistency: throughput is the shared recomputation, and the
        // invariant admitted+refused == n holds per epoch.
        assert_eq!(cell.admitted + cell.refused, n);
        let members_processed = n * 2; // two epochs
        let members_per_sec = stats::per_second(members_processed, ns);
        sweep.push(serde_json::json!({
            "fault_per_mille": per_mille,
            "elapsed_ns": ns,
            "admitted": cell.admitted,
            "refused": cell.refused,
            "recovered": cell.recovered,
            "parked": cell.parked,
            "geometry_gaps": cell.geometry_gaps,
            "quarantined_templates": cell.quarantined_templates,
            "epoch_plans": plans.len(),
            "members_processed": members_processed,
            "members_per_sec": members_per_sec,
            "cell_verified": true,
        }));
    }

    let receipt = serde_json::json!({
        "domain": "praxis-synthesis/cell/supervise/v2",
        "what": "supervised cell — median overhead over paired runs, per-member tail latency, composition ratio, recovery accounting, quarantine, honest scope",
        "discipline": [
            "overhead from medians over paired runs; raw samples published",
            "self-refutation guard: negative overhead beyond baseline spread fails the harness",
            "per-member verdicts are p50/p90/p99/worst — never min",
            "throughput recomputed from (count, elapsed); admitted+refused==n asserted",
            "composition ratio measured (whole cell vs sum of members), not assumed flat",
        ],
        "n": n, "g": g, "templates": templates, "runs": RUNS,
        "policy": {"max_restarts": policy.max_restarts},
        "baseline": {
            "elapsed_ns_samples": base_ns,
            "elapsed_ns_median": base_med,
            "spread_ns": base_spread,
            "admitted": base_admitted,
        },
        "supervised_fault_0": {
            "elapsed_ns_samples": sup0_ns,
            "elapsed_ns_median": sup_med,
            "overhead_pct_of_medians": overhead_pct,
            "admitted": sup0_admitted,
        },
        "per_member_latency_at_10pct_faults_ns": {
            "p50": member_p50,
            "p90": member_p90,
            "p99": member_p99,
            "worst": member_worst,
            "timed_pass_total_ns": timed_pass_ns,
        },
        "composition": {
            "sum_of_member_ns": sum_members_ns,
            "whole_cell_ns": composed_ns,
            "ratio": composition_ratio,
            "note": "ratio > 1 is the priced cost of roll-ups + receipts (2025's 84-vs-8 lesson: measure the composition penalty, never assert flatness)",
        },
        "fault_sweep": sweep,
        "deferred_receipted": [
            "novelty-cost curve under faults: member-level supervision injects synthetic transients without re-running solver work, so a work-proxy re-measurement would overstate cache dividends — deferred until node-level fault injection is wired through the fleet path",
            "beat-level scheduling", "full R1/W1/C1 SLO matrix", "quarantine parole", "supra-cell supervision"
        ],
    });
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../receipts");
    std::fs::create_dir_all(dir).expect("receipts dir");
    std::fs::write(
        format!("{dir}/supervised_cell.json"),
        serde_json::to_string_pretty(&receipt).expect("serialize"),
    )
    .expect("write receipt");
}
