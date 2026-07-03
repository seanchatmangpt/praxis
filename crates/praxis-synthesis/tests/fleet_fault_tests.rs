//! Node-level fault injection tests: a faulted pipeline loses its cache and
//! genuinely re-derives — retries are novelty-bound, not size-bound. Plus the
//! withheld measurement (`#[ignore]`): novelty scaling under a 10% fault rate,
//! written to `receipts/novelty_under_faults.json` whether the claim survives
//! or is refuted.

use praxis_synthesis::dag::MemoCache;
use praxis_synthesis::fleet::{
    overlap_curve, overlap_curve_faulted, overlap_curve_recovering, run_fleet,
    run_fleet_faulted, run_fleet_faulted_recovering, template, FleetReport, NodeFaults,
    RecoveryMode,
};
use praxis_synthesis::sequence::{plan_hash_of, SequenceProblem};
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

// ── Verified-replay recovery (v2 mechanism) ─────────────────────────────

/// Rebuild template `t`'s sequencing problem exactly as the fleet does.
fn rebuild_problem(t: usize) -> SequenceProblem {
    let (mut program, caps, goal, constraints) = template(t);
    program.saturate().expect("template saturates");
    SequenceProblem::with_constraints(&program, caps, goal, 8, constraints)
        .expect("template problem builds")
}

#[test]
fn blind_resolve_mode_is_byte_identical_to_v1() {
    let faults = NodeFaults { seed: 0xDEAD_BEEF, fault_per_mille: 250 };
    let (mut memo_a, mut cores_a) = fresh();
    let mut a = run_fleet_faulted(60, 6, faults, &mut memo_a, &mut cores_a);
    let (mut memo_b, mut cores_b) = fresh();
    let mut b = run_fleet_faulted_recovering(
        60,
        6,
        faults,
        RecoveryMode::BlindResolve,
        &mut memo_b,
        &mut cores_b,
    );
    a.elapsed_ns = 0;
    b.elapsed_ns = 0;
    let ja = serde_json::to_string(&a).expect("report serializes");
    let jb = serde_json::to_string(&b).expect("report serializes");
    assert_eq!(ja, jb, "BlindResolve must be byte-identical to v1");
    assert_eq!(a.recovered_by_replay, 0);
    assert_eq!(a.replay_verify_cost, 0);
    assert_eq!(a.replay_verification_root, "");
}

#[test]
fn verified_replay_recovers_without_search() {
    let faults = NodeFaults { seed: 7, fault_per_mille: 1000 };

    // Blind control on its own warm caches: the admissions baseline.
    let (mut memo_c, mut cores_c) = fresh();
    let _ = run_fleet(8, 4, &mut memo_c, &mut cores_c);
    let _ = run_fleet(8, 4, &mut memo_c, &mut cores_c);
    let blind = run_fleet_faulted(8, 4, faults, &mut memo_c, &mut cores_c);

    // Warm, then fault everything under verified replay.
    let (mut memo, mut cores) = fresh();
    let _ = run_fleet(8, 4, &mut memo, &mut cores);
    let _ = run_fleet(8, 4, &mut memo, &mut cores);
    let r = run_fleet_faulted_recovering(
        8,
        4,
        faults,
        RecoveryMode::VerifiedReplay,
        &mut memo,
        &mut cores,
    );
    assert_eq!(r.faults_injected, 8);
    assert_eq!(r.fault_resolve_nodes, 0, "recovery must not search");
    assert_eq!(r.solver_nodes, 0, "recovery must not search");
    assert_eq!(r.recovered_by_replay, 8);
    assert_eq!(r.replay_rejected, 0);
    assert!(r.replay_verify_cost > 0, "verification is real, declared work");
    assert!(!r.replay_verification_root.is_empty());
    assert!(r.executed_nodes > 0, "faulted DAGs still re-ran cold");
    assert_eq!(r.admitted, blind.admitted);
    assert_eq!(r.refused, blind.refused);
}

#[test]
fn poisoned_cache_is_detected_never_trusted() {
    let faults = NodeFaults { seed: 7, fault_per_mille: 1000 };

    // Honest baseline for the admissions count.
    let (mut memo_h, mut cores_h) = fresh();
    let _ = run_fleet(8, 4, &mut memo_h, &mut cores_h);
    let honest = run_fleet_faulted_recovering(
        8,
        4,
        faults,
        RecoveryMode::VerifiedReplay,
        &mut memo_h,
        &mut cores_h,
    );

    let (mut memo, mut cores) = fresh();
    let _ = run_fleet(8, 4, &mut memo, &mut cores);
    // Poison template 1's cached plan: truncate the steps, KEEP the old
    // receipt (its plan_hash no longer matches the body).
    let problem = rebuild_problem(1);
    let h = problem.problem_hash().to_string();
    let mut poisoned = cores.cached_plan(&h).expect("warm plan cached").clone();
    assert!(poisoned.steps.len() > 1);
    poisoned.steps.truncate(1);
    cores.insert_plan(h, poisoned);

    let r = run_fleet_faulted_recovering(
        8,
        4,
        faults,
        RecoveryMode::VerifiedReplay,
        &mut memo,
        &mut cores,
    );
    assert!(r.replay_rejected >= 1, "poison must be detected, never trusted");
    assert!(r.fault_resolve_nodes > 0, "poison forces a genuine re-solve");
    assert_eq!(r.admitted, honest.admitted, "the fleet still ends honest");
}

#[test]
fn poisoned_plan_with_recomputed_hash_still_caught() {
    let faults = NodeFaults { seed: 7, fault_per_mille: 1000 };
    let (mut memo, mut cores) = fresh();
    let _ = run_fleet(8, 4, &mut memo, &mut cores);
    // Internally consistent forgery: corrupt steps AND recompute the hash so
    // the receipt matches the forged body.
    let problem = rebuild_problem(1);
    let h = problem.problem_hash().to_string();
    let mut forged = cores.cached_plan(&h).expect("warm plan cached").clone();
    forged.steps.truncate(1);
    forged.receipt.plan_hash = plan_hash_of(&forged.steps);
    cores.insert_plan(h, forged);

    let r = run_fleet_faulted_recovering(
        8,
        4,
        faults,
        RecoveryMode::VerifiedReplay,
        &mut memo,
        &mut cores,
    );
    assert!(
        r.replay_rejected >= 1,
        "re-simulation must catch a hash-consistent forgery"
    );
}

#[test]
fn novel_templates_still_resolve_for_real() {
    let (mut memo, mut cores) = fresh();
    let r = run_fleet_faulted_recovering(
        8,
        8,
        NodeFaults { seed: 7, fault_per_mille: 1000 },
        RecoveryMode::VerifiedReplay,
        &mut memo,
        &mut cores,
    );
    assert_eq!(r.recovered_by_replay, 0, "first encounters have nothing to replay");
    assert!(r.solver_nodes > 0, "novelty must be solved for real");
}

#[test]
fn recovery_is_seed_deterministic() {
    let faults = NodeFaults { seed: 0xDEAD_BEEF, fault_per_mille: 250 };
    let run = || {
        let (mut memo, mut cores) = fresh();
        let _ = run_fleet(60, 6, &mut memo, &mut cores);
        let mut r = run_fleet_faulted_recovering(
            60,
            6,
            faults,
            RecoveryMode::VerifiedReplay,
            &mut memo,
            &mut cores,
        );
        r.elapsed_ns = 0;
        serde_json::to_string(&r).expect("report serializes")
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "verified replay must render byte-identically per seed");
    assert!(a.contains("replay_verification_root"));
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

// ── The v2 measurement: same tolerances, verified-replay recovery ───────

#[allow(clippy::cast_precision_loss)]
fn point_json_v2(r: &FleetReport) -> serde_json::Value {
    let mut v = point_json(r);
    let obj = v.as_object_mut().expect("point is an object");
    obj.insert("recovered_by_replay".into(), r.recovered_by_replay.into());
    obj.insert("replay_rejected".into(), r.replay_rejected.into());
    obj.insert("replay_verify_cost".into(), r.replay_verify_cost.into());
    obj.insert(
        "replay_verification_root".into(),
        r.replay_verification_root.clone().into(),
    );
    obj.insert("work_v2".into(), r.work_v2().into());
    v
}

#[test]
#[ignore = "measurement run; execute explicitly to regenerate receipts/novelty_under_faults_v2.json"]
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn novelty_under_faults_v2_receipt() {
    let n = 10_000usize;
    let ks = [10_000usize, 1_000, 100, 10, 1];
    let faults = NodeFaults { seed: 0xC0FF_EE00, fault_per_mille: 100 };

    // Tolerances verbatim from v1, declared before any numbers exist.
    const MAX_REL_RESIDUAL: f64 = 0.25;
    const SLOPE_RATIO_LO: f64 = 0.75;
    const SLOPE_RATIO_HI: f64 = 1.25;
    const COST_MODEL: &str = "work = solver_nodes + replay_verify_cost. One \
        replay-verified plan costs steps + steps*constraints + 3 units (one \
        unit per step independently re-simulated against the problem's \
        preconditions/effects, one per step x constraint checked by \
        plan_respects_constraints, one for the plan-hash recomputation, one \
        for the problem-hash equality check, one for the plan-cost \
        recomputation); one re-derived unsat certificate costs constraints + \
        core_len + 1 units (the certificate detail and core are RE-DERIVED by \
        propagation, never served from cache). One unit is declared \
        equivalent to one solver search node (both are one bounded state \
        operation); this equivalence is a declaration, not a measurement.";

    let baseline = overlap_curve(n, &ks);
    let faulted =
        overlap_curve_recovering(n, &ks, faults, RecoveryMode::VerifiedReplay);

    // Work metric: work_v2 = solver nodes + declared replay-verify units —
    // deterministic, never wall clock. Fit vs K for each curve.
    let xs: Vec<f64> = ks.iter().map(|&k| k as f64).collect();
    let ys_base: Vec<f64> = baseline.iter().map(|r| r.work_v2() as f64).collect();
    let ys_fault: Vec<f64> = faulted.iter().map(|r| r.work_v2() as f64).collect();
    let (slope_b, icept_b, resid_b) = linear_fit(&xs, &ys_base);
    let (slope_f, icept_f, resid_f) = linear_fit(&xs, &ys_fault);
    let slope_ratio = if slope_b == 0.0 { f64::NAN } else { slope_f / slope_b };

    // Self-refutation guard: faulted work_v2 below baseline at any K means
    // the instrument, not the claim, is broken — withhold any verdict.
    let self_refuting = ys_fault.iter().zip(&ys_base).any(|(f, b)| f < b);

    let linear_ok = resid_f <= MAX_REL_RESIDUAL;
    let slope_ok = (SLOPE_RATIO_LO..=SLOPE_RATIO_HI).contains(&slope_ratio);
    let (claim_survives, verdict) = if self_refuting {
        (false, "WITHHELD: self-refuting — faulted work_v2 below baseline at some K; \
                 no survival verdict is offered".to_string())
    } else if !linear_ok {
        (false, format!(
            "REFUTED: faulted work_v2 curve is not linear in K — max relative \
             residual {resid_f:.4} exceeds bound {MAX_REL_RESIDUAL}"
        ))
    } else if !slope_ok {
        (false, format!(
            "REFUTED: slope ratio faulted/baseline {slope_ratio:.4} outside \
             [{SLOPE_RATIO_LO}, {SLOPE_RATIO_HI}] — faults changed the slope, \
             not just the intercept"
        ))
    } else {
        (true, format!(
            "SURVIVES: under verified-replay recovery, faulted work_v2 stays \
             linear in K (max relative residual {resid_f:.4} <= \
             {MAX_REL_RESIDUAL}) and slope ratio {slope_ratio:.4} within \
             [{SLOPE_RATIO_LO}, {SLOPE_RATIO_HI}] — recovery is verification \
             work, not search work"
        ))
    };

    let receipt = serde_json::json!({
        "what": "novelty scaling under node faults, v2 mechanism: recovery is \
                 VERIFIED REPLAY (fetch cached artifact, independently re-verify \
                 hashes + constraints + full O(plan) re-simulation), not blind \
                 re-solve; work metric is work_v2 = solver_nodes + \
                 replay_verify_cost (deterministic), wall time reported but \
                 carries no verdict; v1 blind-resolve receipt remains the \
                 honest baseline",
        "criterion": format!(
            "criterion stated before numbers: max_relative_residual_faulted \
             {MAX_REL_RESIDUAL}, slope_ratio_bounds [{SLOPE_RATIO_LO}, \
             {SLOPE_RATIO_HI}] — tolerances verbatim from v1, unedited"
        ),
        "cost_model": COST_MODEL,
        "recovery_mode": "VerifiedReplay",
        "n": n,
        "fault_per_mille": faults.fault_per_mille,
        "seed": faults.seed,
        "points": ks.iter().enumerate().map(|(i, _)| serde_json::json!({
            "baseline": point_json_v2(&baseline[i]),
            "faulted": point_json_v2(&faulted[i]),
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
    let path = format!("{dir}/novelty_under_faults_v2.json");
    let pretty = serde_json::to_string_pretty(&receipt).expect("receipt renders");
    std::fs::write(&path, pretty).expect("receipt written");
    eprintln!("wrote {path}: {verdict}");
}
