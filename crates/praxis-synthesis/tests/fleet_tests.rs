//! P2 tests: the overlap curve — the fleet's reasoning cost must track
//! novelty (K), not size (N). Plus the receipt writer.

use praxis_synthesis::dag::MemoCache;
use praxis_synthesis::fleet::{lane, overlap_curve, run_fleet, template};
use praxis_synthesis::solver8::CoreCache;

#[test]
fn fleet_bytes_project_terminal_states() {
    let mut memo = MemoCache::new();
    let mut cores = CoreCache::new();
    // 8 pipelines over 8 templates: templates 3 and 7 are certified-unsat.
    let r = run_fleet(8, 8, &mut memo, &mut cores);
    assert_eq!(r.bytes.len(), 8);
    assert_eq!(r.admitted, 6);
    assert_eq!(r.refused, 2);
    for (i, b) in r.bytes.iter().enumerate() {
        if i % 4 == 3 {
            assert!(b & lane::H_HALTED != 0, "template {i} must halt");
            assert!(b & lane::U_UNSAT_CERTIFIED != 0, "template {i} carries a proof");
        } else {
            assert!(b & lane::A_ADMITTED != 0, "template {i} must admit: {b:#010b}");
            assert!(b & lane::P_SATURATED != 0);
            assert!(b & lane::R_PLANNED != 0);
            assert!(b & lane::C_EXECUTED != 0);
        }
    }
}

#[test]
fn shared_caches_absorb_repeated_deliberation() {
    let mut memo = MemoCache::new();
    let mut cores = CoreCache::new();
    // 40 pipelines, 4 templates → each template runs 10 times.
    let r = run_fleet(40, 4, &mut memo, &mut cores);
    // Template 3 is unsat: derived once, replayed 9 times from the core cache.
    assert_eq!(r.core_hits, 9, "9 of 10 dead ends replay from the shared core");
    // Solvable templates: the DAG executes cold once per template, then replays.
    assert!(
        r.replayed_nodes > r.executed_nodes * 8,
        "repeat deliberation must be nearly free: cold {} vs replayed {}",
        r.executed_nodes,
        r.replayed_nodes
    );
}

#[test]
fn the_overlap_curve_falls_with_novelty() {
    // Fixed fleet size, novelty descending: K = N (all unique) down to K = 1.
    let n = 64;
    let curve = overlap_curve(n, &[64, 16, 4, 1]);
    let costs: Vec<f64> = curve.iter().map(|r| r.work_per_pipeline()).collect();
    // The thesis's falsifiable prediction: marginal deliberation cost is
    // monotone non-increasing as overlap rises (novelty falls).
    for w in costs.windows(2) {
        assert!(
            w[1] <= w[0],
            "marginal cost must not rise as novelty falls: {costs:?}"
        );
    }
    // And the collapse is substantial, not marginal: full-overlap fleets
    // deliberate at a small fraction of all-unique cost.
    assert!(
        costs[3] < costs[0] * 0.5,
        "K=1 must cost < 50% of K=N per pipeline: {costs:?}"
    );
}

/// The receipt writer: run the curve at real size and persist the numbers.
/// `#[ignore]` by default — run once with `--ignored` to (re)generate
/// `target/synthesis-fleet-receipt.json`.
#[test]
#[ignore = "measurement run; execute explicitly to regenerate the fleet receipt"]
fn overlap_curve_receipt() {
    let n = 10_000;
    let ks = [10_000, 1_000, 100, 10, 1];
    let curve = overlap_curve(n, &ks);
    let receipt = serde_json::json!({
        "what": "marginal deliberation cost vs fleet novelty (K/N)",
        "n": n,
        "points": curve.iter().map(|r| serde_json::json!({
            "k": r.k,
            "novelty": r.k as f64 / r.n as f64,
            "work_per_pipeline": r.work_per_pipeline(),
            "elapsed_ns": r.elapsed_ns,
            "pipelines_per_sec": r.n as f64 / (r.elapsed_ns as f64 / 1e9),
            "executed_nodes": r.executed_nodes,
            "replayed_nodes": r.replayed_nodes,
            "solver_nodes": r.solver_nodes,
            "core_hits": r.core_hits,
            "admitted": r.admitted,
            "refused": r.refused,
        })).collect::<Vec<_>>(),
    });
    std::fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/synthesis-fleet-receipt.json"),
        serde_json::to_string_pretty(&receipt).expect("serialize"),
    )
    .expect("write receipt");
}

#[test]
fn templates_are_deterministic() {
    // Same template id → identical problem content (the memo/core sharing
    // precondition). Interning order is fixed, so hashes agree.
    let (mut p1, c1, g1, k1) = template(2);
    let (mut p2, c2, g2, k2) = template(2);
    p1.saturate().expect("sat");
    p2.saturate().expect("sat");
    assert_eq!(p1.fixpoint_hash(), p2.fixpoint_hash());
    assert_eq!(c1, c2);
    assert_eq!(g1, g2);
    assert_eq!(k1, k2);
}
