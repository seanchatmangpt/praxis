//! The supervised-cell measurement run: honest numbers to
//! `receipts/supervised_cell.json`. `#[ignore]` by default; run with
//! `--ignored --release` to regenerate.

use praxis_synthesis::cell::run_cell;
use praxis_synthesis::cell_supervise::{run_cell_supervised, FaultScript, SupervisorPolicy};

#[test]
#[ignore = "measurement run; execute with --ignored --release to regenerate the receipt"]
fn supervised_cell_receipt() {
    let (n, g, templates) = (10_000, 100, 8);
    let policy = SupervisorPolicy::default();

    // Baseline: the unsupervised cell.
    let t0 = std::time::Instant::now();
    let (base, _) = run_cell(n, g, templates);
    let base_ns = t0.elapsed().as_nanos();

    // Supervision overhead at fault rate 0.
    let t0 = std::time::Instant::now();
    let (sup0, _, _) = run_cell_supervised(
        n,
        g,
        templates,
        1,
        policy,
        FaultScript { seed: 1, transient_per_mille: 0, crashloop_template: None },
    );
    let sup0_ns = t0.elapsed().as_nanos();

    // Fault sweep: 1% / 10% / 50% transient rates + one crashloop template,
    // two epochs (so the MAPE-K quarantine fires and is measured).
    let mut sweep = Vec::new();
    for per_mille in [10u16, 100, 500] {
        let t0 = std::time::Instant::now();
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
        assert!(praxis_synthesis::cell::verify_cell(&cell, &groups));
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
            "members_per_sec": (cell.n * 2) as f64 / (ns as f64 / 1e9),
            "cell_verified": true,
        }));
    }

    #[allow(clippy::cast_precision_loss)]
    let overhead_pct =
        (sup0_ns as f64 - base_ns as f64) / (base_ns as f64) * 100.0;
    let receipt = serde_json::json!({
        "domain": "praxis-synthesis/cell/supervise/v1",
        "what": "supervised cell — overhead, recovery accounting, quarantine, honest scope",
        "n": n, "g": g, "templates": templates,
        "policy": {"max_restarts": policy.max_restarts},
        "baseline": {"elapsed_ns": base_ns, "admitted": base.admitted, "refused": base.refused},
        "supervised_fault_0": {
            "elapsed_ns": sup0_ns,
            "overhead_pct": overhead_pct,
            "admitted": sup0.admitted,
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
