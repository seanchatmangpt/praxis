#![cfg(test)]

//! Multifractal partition-function module tests (Rail G Track 2b). Two
//! independent things are proven, kept in separate tests per the task
//! design: (1) the `Z(q,epsilon) -> tau(q) -> D(q) -> alpha(q)/f(alpha)`
//! machinery is mathematically correct, checked against a hand-verifiable
//! synthetic binomial cascade with a KNOWN closed-form spectrum
//! (`synthetic_binomial_cascade_...`) and a KNOWN-monofractal uniform
//! measure as a discriminating negative control
//! (`uniform_mass_sequence_is_monofractal`); (2) what Track 2b's real
//! measurement over an actual cng workday shows
//! (`track2b_real_workday_tape_ops_measurement`) — reported honestly
//! whichever way it comes out (design doc §1: "A flat D(q) is a
//! legitimate, honest result").

use std::fs;
use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;

use super::{
    binomial_cascade, box_masses, generalized_dimension, is_multifractal, linear_regression,
    mass_exponent, measure_track2b, singularity_spectrum, standard_q_range, tau_curve, ScaleSample,
    TauPoint, MULTIFRACTAL_TOLERANCE, TRACK2B_EPSILON_SWEEP,
};
use crate::powl::CngRefusal;

/// Scratch root for this test file. O(1).
fn scratch_dir(test_name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!(
            "../../target/chatman/cng-tests/multifractal_{}",
            std::process::id()
        ))
        .join(test_name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

// ---------------------------------------------------------------------
// Refusal-path (negative) tests — CNG_R26 on degenerate input.
// ---------------------------------------------------------------------

test!(box_masses_rejects_zero_epsilon, {
    // Arrange: any nonempty values, epsilon = 0.
    let values = [1.0, 2.0, 3.0];

    // Act.
    let result = box_masses(&values, 0);

    // Assert: typed CNG_R26 naming the box_masses stage.
    match result {
        Err(CngRefusal::MultifractalFitDegenerate { stage, .. }) => {
            assert_eq!(stage, "box_masses");
            assert_eq!(
                CngRefusal::MultifractalFitDegenerate {
                    stage,
                    reason: String::new()
                }
                .code(),
                "CNG_R26"
            );
        }
        other => panic!("expected MultifractalFitDegenerate, got {other:?}"),
    }
});

test!(box_masses_rejects_empty_values, {
    // Arrange/Act: no values to partition.
    let result = box_masses(&[], 1);

    // Assert.
    match result {
        Err(CngRefusal::MultifractalFitDegenerate { stage, .. }) => {
            assert_eq!(stage, "box_masses");
        }
        other => panic!("expected MultifractalFitDegenerate, got {other:?}"),
    }
});

test!(box_masses_rejects_zero_total_mass, {
    // Arrange: an all-zero measure has nothing to partition.
    let values = [0.0, 0.0, 0.0, 0.0];

    // Act.
    let result = box_masses(&values, 2);

    // Assert.
    match result {
        Err(CngRefusal::MultifractalFitDegenerate { stage, .. }) => {
            assert_eq!(stage, "box_masses");
        }
        other => panic!("expected MultifractalFitDegenerate, got {other:?}"),
    }
});

test!(box_masses_normalizes_and_drops_zero_boxes, {
    // Arrange: three empty ticks and one tick carrying the whole mass.
    let values = [0.0, 0.0, 4.0, 0.0];

    // Act: epsilon = 1 (finest resolution — one box per tick).
    let masses = box_masses(&values, 1)?;

    // Assert: only the nonzero box survives, normalized to 1 (it is the
    // whole measure).
    assert_eq!(masses.len(), 1);
    assert!((masses[0] - 1.0).abs() < 1e-12, "got {:?}", masses);
});

test!(linear_regression_rejects_single_point, {
    // Arrange/Act: one point cannot determine a slope.
    let result = linear_regression(&[1.0], &[1.0]);

    // Assert.
    match result {
        Err(CngRefusal::MultifractalFitDegenerate { stage, .. }) => {
            assert_eq!(stage, "linear_regression");
        }
        other => panic!("expected MultifractalFitDegenerate, got {other:?}"),
    }
});

test!(linear_regression_rejects_zero_variance_x, {
    // Arrange: every x identical — no spread to fit a slope from.
    let xs = [2.0, 2.0, 2.0];
    let ys = [1.0, 5.0, 9.0];

    // Act.
    let result = linear_regression(&xs, &ys);

    // Assert.
    match result {
        Err(CngRefusal::MultifractalFitDegenerate { stage, .. }) => {
            assert_eq!(stage, "linear_regression");
        }
        other => panic!("expected MultifractalFitDegenerate, got {other:?}"),
    }
});

test!(linear_regression_recovers_known_exact_slope, {
    // Arrange: y = 3x + 2 exactly, no noise.
    let xs = [0.0, 1.0, 2.0, 3.0];
    let ys: Vec<f64> = xs.iter().map(|x| 3.0 * x + 2.0).collect();

    // Act.
    let (slope, intercept) = linear_regression(&xs, &ys)?;

    // Assert: OLS recovers the exact generating line.
    assert!((slope - 3.0).abs() < 1e-9, "slope {slope}");
    assert!((intercept - 2.0).abs() < 1e-9, "intercept {intercept}");
});

test!(mass_exponent_rejects_fewer_than_two_scales, {
    // Arrange: one scale is not enough for a log-log fit.
    let scales = [ScaleSample {
        epsilon: 1.0,
        masses: vec![0.5, 0.5],
    }];

    // Act.
    let result = mass_exponent(&scales, 2);

    // Assert.
    match result {
        Err(CngRefusal::MultifractalFitDegenerate { stage, .. }) => {
            assert_eq!(stage, "mass_exponent");
        }
        other => panic!("expected MultifractalFitDegenerate, got {other:?}"),
    }
});

test!(singularity_spectrum_rejects_too_few_points, {
    // Arrange: one tau(q) point cannot feed a finite-difference Legendre
    // transform.
    let points = [TauPoint {
        q: 0,
        tau: -1.0,
        points: vec![],
    }];

    // Act.
    let result = singularity_spectrum(&points);

    // Assert.
    match result {
        Err(CngRefusal::MultifractalFitDegenerate { stage, .. }) => {
            assert_eq!(stage, "singularity_spectrum");
        }
        other => panic!("expected MultifractalFitDegenerate, got {other:?}"),
    }
});

test!(singularity_spectrum_rejects_unsorted_q, {
    // Arrange: descending q violates the strictly-ascending precondition.
    let points = [
        TauPoint {
            q: 2,
            tau: 0.5,
            points: vec![],
        },
        TauPoint {
            q: 0,
            tau: -1.0,
            points: vec![],
        },
    ];

    // Act.
    let result = singularity_spectrum(&points);

    // Assert.
    match result {
        Err(CngRefusal::MultifractalFitDegenerate { stage, .. }) => {
            assert_eq!(stage, "singularity_spectrum");
        }
        other => panic!("expected MultifractalFitDegenerate, got {other:?}"),
    }
});

// ---------------------------------------------------------------------
// Correctness (1): a hand-verifiable synthetic multiplicative cascade
// with a KNOWN, non-constant D(q) closed form.
//
// A binomial cascade with left/right multipliers p0, p1 (p0 + p1 = 1)
// has the well-known analytic mass exponent
//     tau(q) = -log2(p0^q + p1^q)
// (derivation: at cascade level k, Z(q, epsilon=2^-k) = (p0^q + p1^q)^k
// exactly, by the multiplicative/self-similar construction; log Z / log
// epsilon = -log2(p0^q + p1^q) for every k, so the log-log "regression"
// is exact, not approximate, for this input). D(q) = tau(q)/(q-1) is
// non-constant whenever p0 != p1 -- this is the textbook multifractal
// example, independent of this module's own implementation.
// ---------------------------------------------------------------------

test!(
    synthetic_binomial_cascade_matches_closed_form_tau_and_is_multifractal,
    {
        // Arrange: a 6-level cascade (64 finest boxes, matching the real
        // Track 2b epsilon sweep 1..32) with an asymmetric split so D(q) is
        // provably non-constant.
        let levels = 6u32;
        let p0 = 0.3_f64;
        let p1 = 0.7_f64;
        let masses = binomial_cascade(levels, p0, p1);
        assert_eq!(masses.len(), 64);
        let total: f64 = masses.iter().sum();
        assert!((total - 1.0).abs() < 1e-9, "cascade mass sums to {total}");

        let mut scales = Vec::new();
        for &epsilon in TRACK2B_EPSILON_SWEEP {
            let boxed = box_masses(&masses, epsilon)?;
            scales.push(ScaleSample {
                epsilon: epsilon as f64,
                masses: boxed,
            });
        }
        let q_values = standard_q_range();

        // Act.
        let tau_points = tau_curve(&scales, &q_values)?;
        let spectrum = singularity_spectrum(&tau_points)?;

        // Assert: tau(q) and D(q) match the closed form at every sampled q,
        // to near machine precision (the cascade is an EXACT power law under
        // this module's own `box_masses` chunking, so the regression fit
        // should have ~zero residual).
        for point in &tau_points {
            let q = point.q as f64;
            let expected_tau = -(p0.powf(q) + p1.powf(q)).log2();
            assert!(
                (point.tau - expected_tau).abs() < 1e-8,
                "tau({q}) = {}, expected {expected_tau}",
                point.tau
            );
            // The (log epsilon, log Z) points the regression was fit from
            // must themselves lie exactly on the analytic closed-form line
            // log Z(q,epsilon) = tau(q) * (log epsilon - log 64) (64 finest
            // boxes at epsilon=1; derived from Z(q,epsilon) = (p0^q +
            // p1^q)^(levels - log2 epsilon) via the binomial theorem),
            // confirming the cascade is a genuine exact power law here, not
            // merely a fit whose SLOPE happens to match with a nonzero
            // residual absorbed into the intercept.
            let log_n = (masses.len() as f64).ln();
            for &(log_eps, log_z) in &point.points {
                let expected_log_z = expected_tau * (log_eps - log_n);
                assert!(
                    (log_z - expected_log_z).abs() < 1e-8,
                    "q={q}: (log_eps={log_eps}, log_z={log_z}) vs analytic {expected_log_z}"
                );
            }
            if let Some(expected_d) = generalized_dimension(expected_tau, point.q) {
                let spectrum_d = spectrum
                    .iter()
                    .find(|s| s.q == point.q)
                    .unwrap_or_else(|| panic!("no spectrum point for q={q}"))
                    .d;
                assert!(
                    (spectrum_d - expected_d).abs() < 1e-8,
                    "D({q}) = {spectrum_d}, expected {expected_d}"
                );
            }
        }

        // Assert: D(0) is the box-counting dimension of the (full-support)
        // cascade, exactly 1.
        let d0 = spectrum.iter().find(|s| s.q == 0).expect("q=0 present").d;
        assert!((d0 - 1.0).abs() < 1e-8, "D(0) = {d0}, expected 1.0");

        // Assert: D(2) (correlation dimension) is strictly less than D(0) --
        // the hand-computable signature of a p0 != p1 binomial cascade.
        let d2 = spectrum.iter().find(|s| s.q == 2).expect("q=2 present").d;
        let expected_d2 = -(p0.powi(2) + p1.powi(2)).log2();
        assert!(
            (d2 - expected_d2).abs() < 1e-8,
            "D(2) = {d2}, expected {expected_d2}"
        );
        assert!(
            d2 < d0 - 0.1,
            "expected D(2) ({d2}) clearly below D(0) ({d0})"
        );

        // Assert: the multifractality test reports true — D(q) is genuinely
        // non-constant, not a rounding artifact (the spread is ~0.2, far
        // above MULTIFRACTAL_TOLERANCE).
        assert!(
            is_multifractal(&spectrum, MULTIFRACTAL_TOLERANCE),
            "known-multifractal binomial cascade (p0={p0}) reported monofractal"
        );
    }
);

test!(uniform_mass_sequence_is_monofractal, {
    // Arrange: a perfectly uniform measure (every tick carries identical
    // mass) is the discriminating negative control -- it MUST come out
    // monofractal (D(q) == 1 for every q), proving `is_multifractal`
    // does not just always return true.
    let values = vec![1.0_f64; 64];
    let mut scales = Vec::new();
    for &epsilon in TRACK2B_EPSILON_SWEEP {
        let boxed = box_masses(&values, epsilon)?;
        scales.push(ScaleSample {
            epsilon: epsilon as f64,
            masses: boxed,
        });
    }
    let q_values = standard_q_range();

    // Act.
    let tau_points = tau_curve(&scales, &q_values)?;
    let spectrum = singularity_spectrum(&tau_points)?;

    // Assert: tau(q) = q - 1 exactly for a uniform measure (closed form:
    // Z(q, epsilon) = (N/epsilon)^(1-q) for N ticks partitioned into
    // N/epsilon equal boxes), so D(q) = 1 for every q.
    for point in &tau_points {
        let expected_tau = point.q as f64 - 1.0;
        assert!(
            (point.tau - expected_tau).abs() < 1e-8,
            "tau({}) = {}, expected {expected_tau}",
            point.q,
            point.tau
        );
    }
    for s in &spectrum {
        assert!(
            (s.d - 1.0).abs() < 1e-6,
            "D({}) = {}, expected 1.0",
            s.q,
            s.d
        );
    }
    assert!(
        !is_multifractal(&spectrum, MULTIFRACTAL_TOLERANCE),
        "uniform measure reported multifractal"
    );
});

// ---------------------------------------------------------------------
// Correctness (2): Track 2b's real measurement over an actual cng
// workday. Whatever tau(q)/D(q)/f(alpha) this measures is reported
// honestly -- a flat D(q) here is a legitimate monofractal finding, not
// a failed test (design doc §1).
// ---------------------------------------------------------------------

test!(track2b_real_workday_tape_ops_measurement, {
    // Arrange: a real 64-tick single-operator workday (zero injected
    // refusals -- see `track2b_tick_tape_ops`'s doc comment for why),
    // driven through the actual cng manufacture chain via `workday()`.
    let out_dir = scratch_dir("track2b_real");
    let seed = 20260711;
    let ticks = 64;
    let q_values = standard_q_range();

    // At 64 ticks, splitmix64 category selection over the 15-category
    // roster draws "api-orchestration" with near certainty (1 -
    // (14/15)^64 ~= 98.6%); `workday()`'s dispatch broker gates that
    // category's Arazzo projection on `verify_arazzo_render_digest`
    // (`arazzo.rs:337`) finding a rendered `generated/arazzo.yaml` whose
    // bytes match a `.ggen-v2/receipt.json` digest at `out_dir` -- the
    // SAME precondition `arazzo_projection_dispatches_every_step_through_
    // the_loopback_adapter` (`arazzo_test.rs`) seeds for its own isolated
    // unit coverage. This is workday()'s own existing requirement, not
    // something Track 2b invented; seed it here rather than inside
    // `measure_track2b`/`track2b_tick_tape_ops` (which stay scoped to
    // manufacturing + partition-function measurement, not dispatch-fixture
    // setup).
    fs::create_dir_all(out_dir.join("generated"))
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir generated: {e}")))?;
    fs::create_dir_all(out_dir.join(".ggen-v2"))
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir .ggen-v2: {e}")))?;
    let rendered_yaml: &[u8] = b"arazzo: \"1.1.0\"\ninfo:\n  title: track2b_real\n";
    fs::write(out_dir.join("generated/arazzo.yaml"), rendered_yaml)
        .map_err(|e| CngRefusal::IoRefused(format!("write generated/arazzo.yaml: {e}")))?;
    let rendered_digest = blake3::hash(rendered_yaml).to_hex().to_string();
    fs::write(
        out_dir.join(".ggen-v2/receipt.json"),
        format!(
            "{{\"payload\":{{\"outputs\":{{\"generated/arazzo.yaml\":\"{rendered_digest}\"}}}}}}"
        ),
    )
    .map_err(|e| CngRefusal::IoRefused(format!("write .ggen-v2/receipt.json: {e}")))?;

    // Act: the full Z(q,epsilon) -> tau(q) -> D(q) -> alpha(q)/f(alpha)
    // pipeline over the real per-tick tape_ops series.
    let measurement = measure_track2b(&out_dir, seed, ticks, TRACK2B_EPSILON_SWEEP, &q_values)?;

    // Assert: structural sanity -- this is a REAL data point, not a
    // decorative one: exactly `ticks` real tape_ops values, six epsilon
    // scales, one tau(q)/spectrum point per requested q.
    assert_eq!(measurement.tick_series.len(), ticks);
    assert!(
        measurement.tick_series.iter().all(|v| *v > 0.0),
        "every successfully manufactured tick must have produced a nonempty tape: {:?}",
        measurement.tick_series
    );
    assert_eq!(measurement.scales.len(), TRACK2B_EPSILON_SWEEP.len());
    assert_eq!(measurement.tau_points.len(), q_values.len());
    assert_eq!(measurement.spectrum.len(), q_values.len());
    // D(0) is the box-counting dimension of the tick-window support; with
    // no injected refusals every window has positive mass, so this
    // should be close to 1 (the full log(1/epsilon)-scaling regime).
    let d0 = measurement
        .spectrum
        .iter()
        .find(|s| s.q == 0)
        .expect("q=0 present")
        .d;
    assert!(
        (0.5..=1.5).contains(&d0),
        "D(0) = {d0} is outside a sane box-counting-dimension range for a \
         fully-populated tick-window support"
    );

    // Report: write the actual measured numbers to disk (this is the one
    // real, non-trivial Track 2b data point the task asks for) rather
    // than asserting a predetermined multifractal/monofractal verdict --
    // per the design doc, either outcome is a legitimate, reportable
    // result.
    let mut report = String::new();
    report.push_str(&format!(
        "Track 2b measurement: seed={seed} ticks={ticks} mass=tape_ops \
         epsilon_sweep={TRACK2B_EPSILON_SWEEP:?}\n"
    ));
    report.push_str(&format!("tick_series={:?}\n", measurement.tick_series));
    for scale in &measurement.scales {
        report.push_str(&format!(
            "epsilon={} boxes={} masses={:?}\n",
            scale.epsilon,
            scale.masses.len(),
            scale.masses
        ));
    }
    report.push_str("q\ttau(q)\tD(q)\talpha(q)\tf(alpha)\n");
    for s in &measurement.spectrum {
        report.push_str(&format!(
            "{}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\n",
            s.q, s.tau, s.d, s.alpha, s.f_alpha
        ));
    }
    report.push_str(&format!("multifractal={}\n", measurement.multifractal));
    fs::create_dir_all(&out_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", out_dir.display())))?;
    fs::write(out_dir.join("track2b-measurement.txt"), &report)
        .map_err(|e| CngRefusal::IoRefused(format!("write track2b-measurement.txt: {e}")))?;
});
