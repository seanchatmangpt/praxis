//! The honesty-audit suite — the 2025 failure modes turned into tests
//! against praxis's OWN measurement discipline.
//!
//! The 2025 lineage (CNS/BitActor/ByteActor/knhk) recorded a benchmark
//! failure register that is design law: min-of-samples PASS, dummy-loop
//! harnesses, aggregate throughput masking per-op FAIL, silent unit
//! redefinition, and green verdicts over dead work. Each test here refutes
//! one of those classes in this crate's measurement kit or artifacts.

mod common;

use std::collections::BTreeSet;

use common::stats;
use praxis_synthesis::budget::{BudgetStatus, TickBudget, Ticks};
use praxis_synthesis::cell::{run_cell, run_member, verify_cell, verify_group};
use praxis_synthesis::cell_supervise::{
    run_member_supervised, FaultScript, SupervisorPolicy,
};
use praxis_synthesis::dag::MemoCache;
use praxis_synthesis::solver8::CoreCache;

/// 2025 defect class B2: "Minimum allocation cycles: 0 ✓ PASS" — a
/// min-of-samples verdict against a hard bound is always gameable. The
/// verdict percentile must never be the minimum on skewed data.
#[test]
fn verdict_percentiles_are_never_the_minimum() {
    // Heavily skewed sample: 99 fast ops hiding one catastrophic tail.
    let mut samples: Vec<u128> = (0..99).map(|_| 10u128).collect();
    samples.push(10_000);
    let p99 = stats::percentile(&mut samples.clone(), 99.0);
    let p50 = stats::percentile(&mut samples.clone(), 50.0);
    let min = *samples.iter().min().expect("nonempty");
    assert_eq!(p50, 10);
    assert!(p99 >= p50);
    // The tail is visible at p100 and must never collapse to the min.
    let p100 = stats::percentile(&mut samples, 100.0);
    assert_eq!(p100, 10_000, "worst case must survive aggregation");
    assert_ne!(p100, min, "a min-of-samples verdict is a benchmark bug");
}

/// 2025 defect class B2 (the other half): a measured 0 means the operation
/// was optimized away or fell below timer resolution — "unmeasurable", not
/// "pass". Zero elapsed time is refused, not graded.
#[test]
#[should_panic(expected = "zero elapsed time is not a measurement")]
fn zero_elapsed_time_is_refused_not_passed() {
    let _ = stats::per_second(1_000, 0);
}

/// An empty sample set is refused the same way.
#[test]
#[should_panic(expected = "no samples is not a measurement")]
fn empty_sample_sets_are_refused() {
    let _ = stats::percentile(&mut [], 99.0);
}

/// 2025 defect class B1: the dummy-loop template — benchmarks that time a
/// synthetic filler loop and report the contract as met. The praxis
/// counterpart: the measured artifact must be VERIFIED work product, not
/// loop completion. Tampering with any member or any group root must be
/// caught by the same verification the measurement run asserts.
#[test]
fn work_product_is_verified_not_loop_completion() {
    let (cell, mut groups) = run_cell(64, 4, 8);
    // The work is real and mixed: admissions AND certified refusals.
    assert!(cell.admitted > 0, "no admitted work — dummy loop?");
    assert!(cell.refused > 0, "no refusals — the t%4==3 unsat class vanished");
    // Terminal hashes vary across template classes (not a constant).
    let distinct: BTreeSet<&str> = groups
        .iter()
        .flat_map(|g| &g.members)
        .map(|m| m.terminal_hash.as_str())
        .collect();
    assert!(distinct.len() > 1, "constant hashes: work product is fictional");
    assert!(verify_cell(&cell, &groups));
    assert!(groups.iter().all(verify_group));
    // Tamper one member's terminal hash → its group fails verification.
    groups[0].members[0].terminal_hash =
        "0000000000000000000000000000000000000000000000000000000000000000".into();
    assert!(!verify_group(&groups[0]), "tampered interior went unnoticed");
    // Tamper a group root → the cell fails verification.
    groups[1].replay_root.clear();
    assert!(!verify_cell(&cell, &groups), "tampered root went unnoticed");
}

/// 2025 defect class B3/unit redefinition: "8-tick compliant" quoted at
/// 7,000ns silently redefined the tick ×1000. In praxis a tick is a
/// DECLARED operation count — wall-clock time passing consumes nothing,
/// and the ninth declared tick exhausts regardless of how fast it ran.
#[test]
fn ticks_are_declared_operations_not_wall_clock() {
    let mut b = TickBudget::chatman();
    // Any amount of real time with zero declared work consumes nothing.
    for _ in 0..10_000 {
        assert_eq!(b.consume(Ticks(0)), BudgetStatus::Ok);
    }
    assert_eq!(b.remaining(), 8);
    // Nine declared ticks exhaust — no clock, no frequency, no redefinition.
    assert_eq!(b.consume(Ticks(9)), BudgetStatus::Exhausted);
}

/// Throughput figures must recompute from (count, elapsed) exactly — the
/// same arithmetic the measurement harness uses to report `members_per_sec`.
#[test]
fn throughput_recomputes_from_count_and_elapsed() {
    let v = stats::per_second(15_000, 1_000_000_000);
    assert!((v - 15_000.0).abs() < f64::EPSILON);
}

/// Self-refutation attempt on the flagship claim "recovery is
/// byte-invisible / supervision does not alter the artifact": at zero
/// injected faults, the supervised member must be BYTE-IDENTICAL to the
/// unsupervised member — same status byte, same terminal hash, zero
/// restarts. If supervision leaks into the artifact, this test is the
/// refutation.
#[test]
fn supervision_at_zero_faults_is_artifact_invisible() {
    let quarantine = BTreeSet::new();
    let policy = SupervisorPolicy::default();
    let script =
        FaultScript { seed: 99, transient_per_mille: 0, crashloop_template: None };
    for agent in 0..24 {
        let mut memo = MemoCache::new();
        let mut cores = CoreCache::new();
        let base = run_member(agent, 8, &mut memo, &mut cores);
        let mut memo2 = MemoCache::new();
        let mut cores2 = CoreCache::new();
        let sup = run_member_supervised(
            agent, 8, &quarantine, policy, script, &mut memo2, &mut cores2,
        );
        assert_eq!(base.byte, sup.byte, "agent {agent}: supervision leaked");
        assert_eq!(base.terminal_hash, sup.terminal_hash, "agent {agent}");
        assert_eq!(sup.restarts, 0, "agent {agent}");
    }
}
