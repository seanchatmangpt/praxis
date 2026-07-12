//! Multifractal partition-function module (Rail G Track 2b; `PRD.md`
//! sections 16/22; `docs/jira/v26.7.11/RAIL_G_MEASUREMENT_DESIGN.md` §1,
//! §3, §4 item 1). Implements the `Z(q,epsilon)` -> `tau(q)` -> `D(q)` ->
//! `alpha(q)`/`f(alpha)` machinery this repo has never had before (the
//! design doc: "None of Z(q,epsilon), tau(q), or f(alpha) exists anywhere
//! in this codebase today"). The math half ([`box_masses`],
//! [`partition_function`], [`mass_exponent`], [`singularity_spectrum`],
//! [`is_multifractal`]) is pure over a "normalized box mass at scale
//! epsilon" input and does not know where the mass came from — it is
//! Track-1-reusable once that track is built, without depending on any
//! cng-specific data source itself.
//!
//! Lives inside `bench` rather than a shared crate (the design doc's own
//! suggestion, `praxis-core::multifractal`) because (a) `praxis-core` is
//! out of scope for this change (owned by a concurrently-running agent)
//! and (b) its only real-data consumer here, [`track2b_tick_tape_ops`],
//! needs `bench::manufacture`'s `pub(super) SetOutcome`/`manufacture_set`
//! — keeping the module inside `bench` avoids loosening that visibility
//! boundary for no reason.
//!
//! ## Track 2b real data source
//!
//! Box size epsilon = window width in ticks. Mass `mu_i(epsilon)` = the
//! summed `tape_ops` (see "why `tape_ops`, not `transitions`" below) of
//! the ticks falling in window `i`, normalized so weights sum to 1 at
//! every epsilon. [`track2b_tick_tape_ops`] reuses the REAL,
//! already-existing, fully-gated `super::workday::workday` driver
//! (`WorkdayConfig{seed, ticks, refusal_per_mille}`) to manufacture one
//! real artifact set per tick through the actual cng chain, then
//! re-manufactures every tick's on-disk artifact-set directory the SAME
//! way `workday()`'s own producer replay-verification step does
//! (`workday.rs`, "Producer replay verification") to recover a
//! `SetOutcome` per tick — no new execution wiring, no parallel driver
//! reimplementing category selection or tick sequencing. `refusal_per_
//! mille` is fixed at 0 for this measurement: a nonzero rate would make
//! some ticks resolve their `manufacture_set` call on a LATER tick (the
//! bounded-admission → resume law), breaking the 1:1 tick-index -> mass
//! correspondence the box-counting partition below assumes.
//!
//! Why `tape_ops`, not `transitions`: for every successfully manufactured
//! tick, `runner::validate_run_hierarchical` requires `fired_atoms ==
//! labels.len()` before returning `Ok` (`runner.rs:278-287`), and
//! `labels.len() == tape.ops.len()`, so `outcome.transitions ==
//! outcome.tape_ops` whenever a tick succeeds — the two are numerically
//! interchangeable on the zero-refusal success path used here. `tape_ops`
//! is chosen as the more primitive quantity: it is the declared/admitted
//! planned-work size (fixed the moment the tape exists, before the
//! runner's scheduler executes anything), matching this design doc's
//! "declared bounded-work unit" framing (ticks, not wall time) more
//! directly than a runtime execution count would.

use std::path::Path;

use crate::powl::CngRefusal;

use super::manufacture::manufacture_set;
use super::workday::{workday, WorkdayConfig};

// ---------------------------------------------------------------------
// Pure math: Z(q, epsilon) -> tau(q) -> D(q) -> alpha(q)/f(alpha)
// ---------------------------------------------------------------------

/// One box-counting scale: window width `epsilon` (native domain unit —
/// ticks for Track 2b) and the resulting normalized box masses
/// (`mu_i(epsilon)`, summing to 1 over the full partition; boxes with
/// zero mass are dropped by [`box_masses`] since a box outside the
/// measure's support contributes nothing to `Z(q,epsilon)` for any `q`).
#[derive(Debug, Clone)]
pub(super) struct ScaleSample {
    pub(super) epsilon: f64,
    pub(super) masses: Vec<f64>,
}

/// Partitions a raw per-tick mass sequence into non-overlapping windows
/// of width `epsilon` (the final window may be narrower than `epsilon`
/// when `values.len()` does not divide evenly — it is still a legitimate,
/// smaller box, not dropped), normalizes each window's summed mass by the
/// TOTAL mass across the whole sequence (so `Z(1, epsilon) == 1` at every
/// epsilon, the identity `tau(1) == 0` relies on), and drops zero-mass
/// boxes (outside the support of the measure — `mu^q` for negative `q` is
/// undefined at `mu == 0`).
///
/// # Errors
/// `CNG_R26 MultifractalFitDegenerate` if `epsilon == 0`, `values` is
/// empty, or the total mass is not positive (no measure to partition).
///
/// # Complexity
/// O(n) in `values.len()`.
pub(super) fn box_masses(values: &[f64], epsilon: usize) -> Result<Vec<f64>, CngRefusal> {
    if epsilon == 0 {
        return Err(CngRefusal::MultifractalFitDegenerate {
            stage: "box_masses".to_string(),
            reason: "epsilon window width must be >= 1 tick".to_string(),
        });
    }
    if values.is_empty() {
        return Err(CngRefusal::MultifractalFitDegenerate {
            stage: "box_masses".to_string(),
            reason: "no values to partition".to_string(),
        });
    }
    let total: f64 = values.iter().sum();
    if !(total > 0.0) {
        return Err(CngRefusal::MultifractalFitDegenerate {
            stage: "box_masses".to_string(),
            reason: format!("total mass {total} is not positive; nothing to partition"),
        });
    }
    let masses = values
        .chunks(epsilon)
        .map(|chunk| chunk.iter().sum::<f64>() / total)
        .filter(|mass| *mass > 0.0)
        .collect();
    Ok(masses)
}

/// `Z(q, epsilon) = sum_i mu_i(epsilon)^q`, over the boxes with positive
/// mass ([`box_masses`] has already dropped `mu_i == 0` boxes, which are
/// outside the measure's support).
///
/// # Complexity
/// O(|masses|).
pub(super) fn partition_function(masses: &[f64], q: f64) -> f64 {
    masses.iter().map(|mu| mu.powf(q)).sum()
}

/// Ordinary least-squares slope + intercept of `ys` against `xs`.
///
/// # Errors
/// `CNG_R26 MultifractalFitDegenerate` if there are fewer than 2 matched
/// points, or `xs` has zero variance (a regression needs spread in the
/// independent variable — the log-epsilon sweep must have >= 2 distinct
/// values).
///
/// # Complexity
/// O(n).
pub(super) fn linear_regression(xs: &[f64], ys: &[f64]) -> Result<(f64, f64), CngRefusal> {
    if xs.len() != ys.len() || xs.len() < 2 {
        return Err(CngRefusal::MultifractalFitDegenerate {
            stage: "linear_regression".to_string(),
            reason: format!(
                "need >= 2 matched (x, y) points, got {} x and {} y",
                xs.len(),
                ys.len()
            ),
        });
    }
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    for (x, y) in xs.iter().zip(ys) {
        let dx = x - mean_x;
        sxx += dx * dx;
        sxy += dx * (y - mean_y);
    }
    if !(sxx > 0.0) {
        return Err(CngRefusal::MultifractalFitDegenerate {
            stage: "linear_regression".to_string(),
            reason: "independent variable (log epsilon) has zero variance".to_string(),
        });
    }
    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    Ok((slope, intercept))
}

/// `tau(q)` at one `q`, fit by log-log regression across the epsilon
/// sweep; carries the `(log epsilon, log Z)` points the fit was drawn
/// from for auditability. `q` is an integer (the design doc's standard
/// sweep is over `q in {-5,...,5}`); keeping it `i64` rather than `f64`
/// avoids float-key sorting/equality pitfalls in [`singularity_spectrum`]
/// with no loss of expressiveness for this module's required range.
#[derive(Debug, Clone)]
pub(super) struct TauPoint {
    pub(super) q: i64,
    pub(super) tau: f64,
    pub(super) points: Vec<(f64, f64)>,
}

/// Fits `tau(q) = d(log Z(q, epsilon)) / d(log epsilon)` by ordinary
/// least-squares regression across `scales` (design doc §1: "tau(q), fit
/// from log Z(q,epsilon) vs. log epsilon across several epsilon (slope of
/// the linear regression)").
///
/// # Errors
/// `CNG_R26 MultifractalFitDegenerate` if fewer than 2 scales are given,
/// a scale's `epsilon` is not positive, a scale's `Z(q, epsilon)` is not
/// positive (its box masses summed to zero after filtering), or the
/// underlying [`linear_regression`] refuses.
///
/// # Complexity
/// O(|scales| * |masses per scale|).
pub(super) fn mass_exponent(scales: &[ScaleSample], q: i64) -> Result<TauPoint, CngRefusal> {
    if scales.len() < 2 {
        return Err(CngRefusal::MultifractalFitDegenerate {
            stage: "mass_exponent".to_string(),
            reason: format!(
                "need >= 2 distinct epsilon scales for a log-log fit, got {}",
                scales.len()
            ),
        });
    }
    let mut points = Vec::with_capacity(scales.len());
    for scale in scales {
        if !(scale.epsilon > 0.0) {
            return Err(CngRefusal::MultifractalFitDegenerate {
                stage: "mass_exponent".to_string(),
                reason: format!("epsilon {} is not positive", scale.epsilon),
            });
        }
        let z = partition_function(&scale.masses, q as f64);
        if !(z > 0.0) {
            return Err(CngRefusal::MultifractalFitDegenerate {
                stage: "mass_exponent".to_string(),
                reason: format!("Z(q={q}, epsilon={}) = {z} is not positive", scale.epsilon),
            });
        }
        points.push((scale.epsilon.ln(), z.ln()));
    }
    let xs: Vec<f64> = points.iter().map(|(x, _)| *x).collect();
    let ys: Vec<f64> = points.iter().map(|(_, y)| *y).collect();
    let (slope, _intercept) = linear_regression(&xs, &ys)?;
    Ok(TauPoint {
        q,
        tau: slope,
        points,
    })
}

/// Fits `tau(q)` at every `q` in `q_values`, in the given order (the
/// design doc's standard range is `q in {-5,...,5}`, skipping `q=1`:
/// `Z(1, epsilon) == 1` identically at every epsilon by [`box_masses`]'s
/// own normalization, so `tau(1) == 0` by construction and a regression
/// there is a zero-variance-in-Y degenerate fit, not informative — see
/// [`generalized_dimension`] for how `D(1)` is recovered by finite
/// difference instead of dividing by `q - 1 == 0`).
///
/// # Errors
/// Propagates the first [`mass_exponent`] refusal.
///
/// # Complexity
/// O(|q_values| * |scales| * |masses per scale|).
pub(super) fn tau_curve(
    scales: &[ScaleSample],
    q_values: &[i64],
) -> Result<Vec<TauPoint>, CngRefusal> {
    q_values.iter().map(|q| mass_exponent(scales, *q)).collect()
}

/// Generalized dimension `D(q) = tau(q)/(q-1)`, `q != 1` (design doc §1).
/// Returns `None` at `q == 1` (division by zero); callers wanting `D(1)`
/// use the finite-difference `alpha(1)` from [`singularity_spectrum`]
/// instead (L'Hopital: `lim tau(q)/(q-1) = d(tau)/dq` as `q -> 1`).
///
/// # Complexity
/// O(1).
pub(super) fn generalized_dimension(tau: f64, q: i64) -> Option<f64> {
    if q == 1 {
        None
    } else {
        Some(tau / (q as f64 - 1.0))
    }
}

/// One point of the Legendre-transformed singularity spectrum.
#[derive(Debug, Clone, Copy)]
pub(super) struct SpectrumPoint {
    pub(super) q: i64,
    pub(super) tau: f64,
    /// Generalized dimension `D(q)` (at `q == 1`, the L'Hopital value
    /// `alpha(1)` — see [`generalized_dimension`]).
    pub(super) d: f64,
    pub(super) alpha: f64,
    pub(super) f_alpha: f64,
}

/// Legendre transform of the fitted `tau(q)` curve: `alpha(q) =
/// d(tau)/dq` (finite difference — the secant slope through neighboring
/// `q`'s actual `tau` values, correct for non-uniform `q` spacing since
/// it is the definition of a finite-difference derivative estimate, not
/// an assumption of a uniform step; design doc §1: "finite-difference
/// derivative of the fitted tau(q) curve is sufficient, no closed form
/// needed"), `f(alpha) = q * alpha(q) - tau(q)`. `tau_points` MUST be
/// strictly ascending in `q` (the caller-controlled sweep order —
/// [`tau_curve`] preserves `q_values`'s order, so pass a pre-sorted
/// `q_values`).
///
/// # Errors
/// `CNG_R26 MultifractalFitDegenerate` if fewer than 2 tau points are
/// given, or `tau_points` is not strictly ascending in `q`.
///
/// # Complexity
/// O(|tau_points|).
pub(super) fn singularity_spectrum(
    tau_points: &[TauPoint],
) -> Result<Vec<SpectrumPoint>, CngRefusal> {
    if tau_points.len() < 2 {
        return Err(CngRefusal::MultifractalFitDegenerate {
            stage: "singularity_spectrum".to_string(),
            reason: format!(
                "need >= 2 tau(q) points for a Legendre transform, got {}",
                tau_points.len()
            ),
        });
    }
    for pair in tau_points.windows(2) {
        if pair[1].q <= pair[0].q {
            return Err(CngRefusal::MultifractalFitDegenerate {
                stage: "singularity_spectrum".to_string(),
                reason: format!(
                    "tau_points must be strictly ascending in q, got {} then {}",
                    pair[0].q, pair[1].q
                ),
            });
        }
    }
    let n = tau_points.len();
    let mut spectrum = Vec::with_capacity(n);
    for i in 0..n {
        let (lo, hi) = if i == 0 {
            (0, 1)
        } else if i == n - 1 {
            (n - 2, n - 1)
        } else {
            (i - 1, i + 1)
        };
        let dq = (tau_points[hi].q - tau_points[lo].q) as f64;
        let alpha = (tau_points[hi].tau - tau_points[lo].tau) / dq;
        let q = tau_points[i].q;
        let tau = tau_points[i].tau;
        let d = match generalized_dimension(tau, q) {
            Some(d) => d,
            // L'Hopital: D(1) = lim tau(q)/(q-1) = d(tau)/dq|_{q=1} = alpha(1).
            None => alpha,
        };
        spectrum.push(SpectrumPoint {
            q,
            tau,
            d,
            alpha,
            f_alpha: q as f64 * alpha - tau,
        });
    }
    Ok(spectrum)
}

/// Multifractality test (design doc §1): true iff `D(q)` spans more than
/// `tolerance` across `spectrum`. A flat `D(q)` (spread <= `tolerance`)
/// is a legitimate monofractal finding, not a refusal — an empty
/// `spectrum` is trivially reported `false` (no data, no claim).
///
/// # Complexity
/// O(|spectrum|).
pub(super) fn is_multifractal(spectrum: &[SpectrumPoint], tolerance: f64) -> bool {
    let mut min_d = f64::INFINITY;
    let mut max_d = f64::NEG_INFINITY;
    for point in spectrum {
        min_d = min_d.min(point.d);
        max_d = max_d.max(point.d);
    }
    (max_d - min_d) > tolerance
}

/// The design doc's standard `q` sweep: `-5..=5`, skipping `q=1` (see
/// [`tau_curve`]'s doc comment for why).
///
/// # Complexity
/// O(1) (11 candidates minus the skip).
pub(super) fn standard_q_range() -> Vec<i64> {
    (-5..=5).filter(|q| *q != 1).collect()
}

/// `D(q)` spread beyond which [`is_multifractal`] reports `true`. Fixed
/// here rather than made a CLI/config parameter: this task's scope is the
/// module + one Track 2b data point, not the "declared scale profile"
/// schema artifact (design doc §4 item 4), which is future work.
pub(super) const MULTIFRACTAL_TOLERANCE: f64 = 1e-3;

/// Track 2b's standard epsilon sweep: window widths in ticks, geometric
/// spacing (design doc §3: "e.g. 1, 2, 4, 8, 16, 32 ... geometric spacing
/// gives the log-log regression real dynamic range").
pub(super) const TRACK2B_EPSILON_SWEEP: &[usize] = &[1, 2, 4, 8, 16, 32];

// ---------------------------------------------------------------------
// Track 2b real data source
// ---------------------------------------------------------------------

/// Runs one real single-operator workday (`super::workday::workday`,
/// `WorkdayConfig{seed, ticks, refusal_per_mille: 0}`) into `out_dir`,
/// then re-manufactures every tick's on-disk artifact-set directory the
/// SAME way `workday()`'s own producer replay-verification step does,
/// returning the resulting `tape_ops` count per tick in tick order.
///
/// `refusal_per_mille` is fixed at 0: with no injected refusals, every
/// tick's directory is complete from the start and `workday()`'s
/// tick-loop `manufacture_set` call succeeds in the same tick it is
/// written, so tick index and artifact-set directory correspond 1:1 —
/// the bounded-admission → resume law (a refused tick's set is granted
/// and re-manufactured at a LATER tick) would otherwise decouple "tick i"
/// from "the mass produced during tick i".
///
/// # Errors
/// Propagates `workday()`'s refusal; `CNG_R08 Nondeterminism` if a
/// tick's re-manufacture here refuses (that tick already succeeded once
/// inside `workday()`, so a second refusal means the chain is not
/// deterministic, not that the tick was always unlawful).
///
/// # Complexity
/// O(ticks) manufactures (workday's own tick loop) + O(ticks) replay
/// manufactures (workday's own producer replay-verification step) +
/// O(ticks) re-manufactures (this function), each pipeline-bounded.
pub(super) fn track2b_tick_tape_ops(
    out_dir: &Path,
    seed: u64,
    ticks: usize,
) -> Result<Vec<f64>, CngRefusal> {
    let cfg = WorkdayConfig {
        seed,
        ticks,
        refusal_per_mille: 0,
    };
    workday(out_dir, &cfg, None)?;
    let ticks_dir = out_dir.join("ticks");
    let mut series = Vec::with_capacity(ticks);
    for tick in 0..ticks {
        let set_dir = ticks_dir.join(format!("tick-{tick:04}"));
        let outcome = manufacture_set(&set_dir, None);
        if let Some(code) = outcome.refusal_code {
            return Err(CngRefusal::Nondeterminism(format!(
                "tick-{tick:04} re-manufactured for the Track 2b mass series but \
                 refused {code}; workday() already manufactured this tick \
                 successfully once, so a second refusal is nondeterminism, not an \
                 expected refusal"
            )));
        }
        series.push(outcome.tape_ops as f64);
    }
    Ok(series)
}

/// One full Track 2b measurement: the real per-tick `tape_ops` series,
/// the epsilon-sweep box-mass partition, the fitted `tau(q)` curve, the
/// Legendre-transformed singularity spectrum, and the multifractality
/// verdict.
#[derive(Debug)]
pub(super) struct Track2bMeasurement {
    pub(super) tick_series: Vec<f64>,
    pub(super) scales: Vec<ScaleSample>,
    pub(super) tau_points: Vec<TauPoint>,
    pub(super) spectrum: Vec<SpectrumPoint>,
    pub(super) multifractal: bool,
}

/// Runs the full Track 2b pipeline end to end: real per-tick mass series
/// -> epsilon-sweep box masses -> `tau(q)` sweep -> singularity spectrum
/// -> multifractality verdict (design doc §6 item 1: "Track 2b first ...
/// to get one real, non-trivial tau(q)/f(alpha) data point end to end,
/// proving the partition-function module itself is correct").
///
/// # Errors
/// Propagates [`track2b_tick_tape_ops`], [`box_masses`], [`tau_curve`],
/// or [`singularity_spectrum`] refusals.
///
/// # Complexity
/// O(ticks) manufacture cost (see [`track2b_tick_tape_ops`]) + O(|epsilon
/// sweep| * ticks) partitioning + O(|q range| * |epsilon sweep|) fitting.
pub(super) fn measure_track2b(
    out_dir: &Path,
    seed: u64,
    ticks: usize,
    epsilon_sweep: &[usize],
    q_values: &[i64],
) -> Result<Track2bMeasurement, CngRefusal> {
    let tick_series = track2b_tick_tape_ops(out_dir, seed, ticks)?;
    let mut scales = Vec::with_capacity(epsilon_sweep.len());
    for &epsilon in epsilon_sweep {
        let masses = box_masses(&tick_series, epsilon)?;
        scales.push(ScaleSample {
            epsilon: epsilon as f64,
            masses,
        });
    }
    let tau_points = tau_curve(&scales, q_values)?;
    let spectrum = singularity_spectrum(&tau_points)?;
    let multifractal = is_multifractal(&spectrum, MULTIFRACTAL_TOLERANCE);
    Ok(Track2bMeasurement {
        tick_series,
        scales,
        tau_points,
        spectrum,
        multifractal,
    })
}

/// Builds the finest-resolution mass array of a `levels`-level binomial
/// (Bernoulli) multiplicative cascade with left/right multipliers `p0`,
/// `p1` (callers pass `p0 + p1 == 1` for a normalized cascade; this
/// function does not enforce it). `mass[i]` for `i` in `0..2^levels` is
/// `p0^(levels - ones(i)) * p1^(ones(i))`, where `ones(i)` is the
/// popcount of `i`'s `levels`-bit binary representation — the classic
/// construction where consecutive array positions correspond to
/// binary-counting order, so summing `2^k`-wide contiguous chunks of the
/// array EXACTLY reproduces the `(levels - k)`-level cascade's own box
/// masses (self-similarity is exact under [`box_masses`]'s `chunks`
/// partition, not approximate) — this is what makes the cascade a
/// hand-verifiable ground truth for [`mass_exponent`]/[`tau_curve`]: its
/// analytic `tau(q) = -log2(p0^q + p1^q)` is known in closed form.
///
/// # Complexity
/// O(2^levels).
#[cfg(test)]
pub(super) fn binomial_cascade(levels: u32, p0: f64, p1: f64) -> Vec<f64> {
    let n = 1usize << levels;
    (0..n)
        .map(|i| {
            let ones = (i as u32).count_ones();
            let zeros = levels - ones;
            p0.powi(zeros as i32) * p1.powi(ones as i32)
        })
        .collect()
}

#[cfg(test)]
#[path = "multifractal_test.rs"]
mod multifractal_test;
