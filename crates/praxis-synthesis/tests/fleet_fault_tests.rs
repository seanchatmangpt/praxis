//! Node-level fault injection tests: a faulted pipeline loses its cache and
//! genuinely re-derives — retries are novelty-bound, not size-bound. Plus the
//! withheld measurement (`#[ignore]`): novelty scaling under a 10% fault rate,
//! written to `receipts/novelty_under_faults.json` whether the claim survives
//! or is refuted.

use praxis_synthesis::dag::MemoCache;
use praxis_synthesis::fleet::{
    overlap_curve, overlap_curve_faulted, run_fleet, run_fleet_faulted, FleetReport,
    NodeFaults,
};
use praxis_synthesis::solver8::CoreCache;

fn fresh() -> (MemoCache, CoreCache) {
    (MemoCache::new(), CoreCache::new())
}

fn assert_reports_identical(a: &FleetReport, b: &FleetReport) {
    assert_eq!(a.n, b.n);
    assert_eq!(a.k, b.k);
    assert_eq!(a.executed_nodes, b.executed_nodes);
    assert_eq!(a.replayed_nodes, b.replayed_nodes);
    assert_eq!(a.solver_nodes, b.solver_nodes);
    assert_eq!(a.core_hits, b.core_hits);
    assert_eq!(a.admitted, b.admitted);
    assert_eq!(a.refused, b.refused);
    assert_eq!(a.bytes, b.bytes);
    assert_eq!(a.faults_injected, b.faults_injected);
    assert_eq!(a.fault_resolve_nodes, b.fault_resolve_nodes);
}

#[test]
fn zero_fault_rate_is_the_identity() {
    let (mut memo_a, mut cores_a) = fresh();
    let base = run_fleet(40, 4, &mut memo_a, &mut cores_a);
    let (mut memo_b, mut cores_b) = fresh();
    let faulted = run_fleet_faulted(
        40,
        4,
        NodeFaults { seed: 12345, fault_per_mille: 0 },
        &mut memo_b,
        &mut cores_b,
    );
    assert_reports_identical(&base, &faulted);
    assert_eq!(faulted.faults_injected, 0);
    assert_eq!(faulted.fault_resolve_nodes, 0);
}

#[test]
fn a_faulted_node_genuinely_re_runs_work() {
    let (mut memo, mut cores) = fresh();
    // Warm: templates 0 and 1 (both solvable) get solved and executed once.
    let warm = run_fleet(4, 2, &mut memo, &mut cores);
    assert!(warm.solver_nodes > 0, "the warm-up run must search for real");

    // Control on warm caches: pure replay, zero search.
    let control = run_fleet(4, 2, &mut memo, &mut cores);
    assert_eq!(control.solver_nodes, 0, "warm caches must replay for free");

    // Fault every pipeline: caches evicted, real work where replay was free.
    let faulted = run_fleet_faulted(
        4,
        2,
        NodeFaults { seed: 7, fault_per_mille: 1000 },
        &mut memo,
        &mut cores,
    );
    assert_eq!(faulted.faults_injected, 4);
    assert!(faulted.solver_nodes > 0, "faulted pipelines must re-solve for real");
    assert!(faulted.fault_resolve_nodes > 0);
    assert!(faulted.executed_nodes > 0, "faulted DAGs must execute cold");
}

#[test]
fn fault_injection_is_seed_deterministic() {
    let faults = NodeFaults { seed: 0xDEAD_BEEF, fault_per_mille: 250 };
    let (mut memo_a, mut cores_a) = fresh();
    let mut a = run_fleet_faulted(60, 6, faults, &mut memo_a, &mut cores_a);
    let (mut memo_b, mut cores_b) = fresh();
    let mut b = run_fleet_faulted(60, 6, faults, &mut memo_b, &mut cores_b);
    // Wall time is the only nondeterministic field; zero it before comparing.
    a.elapsed_ns = 0;
    b.elapsed_ns = 0;
    let ja = serde_json::to_string(&a).expect("report serializes");
    let jb = serde_json::to_string(&b).expect("report serializes");
    assert_eq!(ja, jb, "same (n, k, seed, rate) must render byte-identically");
}

#[test]
fn eviction_is_repopulating_not_cascading() {
    let (mut memo, mut cores) = fresh();
    let faulted = run_fleet_faulted(
        8,
        4,
        NodeFaults { seed: 3, fault_per_mille: 1000 },
        &mut memo,
        &mut cores,
    );
    assert_eq!(faulted.faults_injected, 8);
    // The retries repopulated the caches: a zero-fault run now replays.
    let after = run_fleet(8, 4, &mut memo, &mut cores);
    assert_eq!(
        after.solver_nodes, 0,
        "post-fault run must replay from the re-warmed cache"
    );
}

// ── The withheld measurement ────────────────────────────────────────────

/// Least-squares fit y = intercept + slope·x. Returns
/// (slope, intercept, max relative residual against the observed y).
fn linear_fit(xs: &[f64], ys: &[f64]) -> (f64, f64, f64) {
    let n = xs.len() as f64;
    let sx: f64 = xs.iter().sum();
    let sy: f64 = ys.iter().sum();
    let sxx: f64 = xs.iter().map(|x| x * x).sum();
    let sxy: f64 = xs.iter().zip(ys).map(|(x, y)| x * y).sum();
    let denom = n * sxx - sx * sx;
    let slope = if denom == 0.0 { 0.0 } else { (n * sxy - sx * sy) / denom };
    let intercept = (sy - slope * sx) / n;
    let max_rel = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| {
            let fitted = intercept + slope * x;
            if *y == 0.0 { 0.0 } else { ((y - fitted) / y).abs() }
        })
        .fold(0.0_f64, f64::max);
    (slope, intercept, max_rel)
}

#[allow(clippy::cast_precision_loss)]
fn point_json(r: &FleetReport) -> serde_json::Value {
    // Throughput recomputed from (count, elapsed) — reported, never a verdict.
    let secs = r.elapsed_ns as f64 / 1e9;
    let pps = if secs > 0.0 { r.n as f64 / secs } else { 0.0 };
    serde_json::json!({
        "k": r.k,
        "novelty_k_over_n": r.k as f64 / r.n as f64,
        "solver_nodes": r.solver_nodes,
        "fault_resolve_nodes": r.fault_resolve_nodes,
        "faults_injected": r.faults_injected,
        "executed_nodes": r.executed_nodes,
        "replayed_nodes": r.replayed_nodes,
        "core_hits": r.core_hits,
        "admitted": r.admitted,
        "refused": r.refused,
        "elapsed_ns": r.elapsed_ns,
        "pipelines_per_sec_recomputed": pps,
    })
}

#[test]
#[ignore = "measurement run; execute explicitly to regenerate receipts/novelty_under_faults.json"]
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn novelty_under_faults_receipt() {
    let n = 10_000usize;
    let ks = [10_000usize, 1_000, 100, 10, 1];
    let faults = NodeFaults { seed: 0xC0FF_EE00, fault_per_mille: 100 };

    let baseline = overlap_curve(n, &ks);
    let faulted = overlap_curve_faulted(n, &ks, faults);

    // Work metric: total solver nodes per point — deterministic, never wall
    // clock. Fit total work vs K for each curve.
    let xs: Vec<f64> = ks.iter().map(|&k| k as f64).collect();
    let ys_base: Vec<f64> = baseline.iter().map(|r| r.solver_nodes as f64).collect();
    let ys_fault: Vec<f64> = faulted.iter().map(|r| r.solver_nodes as f64).collect();
    let (slope_b, icept_b, resid_b) = linear_fit(&xs, &ys_base);
    let (slope_f, icept_f, resid_f) = linear_fit(&xs, &ys_fault);
    let slope_ratio = if slope_b == 0.0 { f64::NAN } else { slope_f / slope_b };

    const MAX_REL_RESIDUAL: f64 = 0.25;
    const SLOPE_RATIO_LO: f64 = 0.75;
    const SLOPE_RATIO_HI: f64 = 1.25;

    // Self-refutation guard: faults measured CHEAPER than none at any K means
    // the instrument, not the claim, is broken — withhold any verdict.
    let self_refuting = ys_fault.iter().zip(&ys_base).any(|(f, b)| f < b);

    let linear_ok = resid_f <= MAX_REL_RESIDUAL;
    let slope_ok = (SLOPE_RATIO_LO..=SLOPE_RATIO_HI).contains(&slope_ratio);
    let (claim_survives, verdict) = if self_refuting {
        (false, "WITHHELD: self-refuting — faulted total work below baseline at some K; \
                 no survival verdict is offered".to_string())
    } else if !linear_ok {
        (false, format!(
            "REFUTED: faulted curve is not linear in K — max relative residual \
             {resid_f:.4} exceeds bound {MAX_REL_RESIDUAL}"
        ))
    } else if !slope_ok {
        (false, format!(
            "REFUTED: slope ratio faulted/baseline {slope_ratio:.4} outside \
             [{SLOPE_RATIO_LO}, {SLOPE_RATIO_HI}] — faults changed the slope, \
             not just the intercept"
        ))
    } else {
        (true, format!(
            "SURVIVES: faulted work stays linear in K (max relative residual \
             {resid_f:.4} <= {MAX_REL_RESIDUAL}) and slope ratio {slope_ratio:.4} \
             within [{SLOPE_RATIO_LO}, {SLOPE_RATIO_HI}] — faults add a constant \
             retry term, not a slope change"
        ))
    };

    let receipt = serde_json::json!({
        "what": "novelty scaling under node faults: total solver work vs K at n=10000, \
                 baseline vs 10% seed-deterministic node-fault lottery; work metric is \
                 solver nodes (deterministic), wall time reported but carries no verdict",
        "n": n,
        "fault_per_mille": faults.fault_per_mille,
        "seed": faults.seed,
        "points": ks.iter().enumerate().map(|(i, _)| serde_json::json!({
            "baseline": point_json(&baseline[i]),
            "faulted": point_json(&faulted[i]),
        })).collect::<Vec<_>>(),
        "fit": {
            "baseline": { "slope": slope_b, "intercept": icept_b,
                          "max_relative_residual": resid_b },
            "faulted": { "slope": slope_f, "intercept": icept_f,
                         "max_relative_residual": resid_f },
            "slope_ratio_faulted_over_baseline": slope_ratio,
        },
        "tolerances": {
            "max_relative_residual_faulted": MAX_REL_RESIDUAL,
            "slope_ratio_bounds": [SLOPE_RATIO_LO, SLOPE_RATIO_HI],
        },
        "self_refuting": self_refuting,
        "claim_survives": claim_survives,
        "verdict": verdict,
    });

    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../receipts");
    std::fs::create_dir_all(dir).expect("receipts dir");
    let path = format!("{dir}/novelty_under_faults.json");
    let pretty = serde_json::to_string_pretty(&receipt).expect("receipt renders");
    std::fs::write(&path, pretty).expect("receipt written");
    eprintln!("wrote {path}: {verdict}");
}
