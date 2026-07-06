//! Integration test for the frontier `capability_source` × `praxis_socket`
//! DfCM matrix (Lane 10, Phase 4 of the CPhy frontier plan).
//!
//! # Coverage threshold — why not 1.0
//!
//! The full Cartesian product is `CAPABILITY_SOURCES.len()` capability
//! sources × `PRAXIS_SOCKETS.len()` praxis sockets (22 × 13 = 286 as of the
//! Genesis Day 7 survey sweep). The overwhelming majority of those pairs
//! (e.g. `stpnt` × `plan-noun`) were never a candidate integration this
//! session — no lane proposed wiring stpnt into the planner. Marking all of
//! them `Impossible` with a blanket "out of scope" reason would inflate
//! `coverage`/`pass_rate` to 1.0 for free without anyone having decided
//! anything about those cells, which is exactly the kind of silent,
//! fabricated pass this project's combinatorial-maximalism discipline
//! forbids.
//!
//! Instead, [`build_frontier_matrix`] evaluates *only* the cells that this
//! session actually touched: 14 admitted/executed integrations (each
//! verified by running the real command or test suite — see each cell's
//! `fixture` string) plus 16 refusals (one per row of the frontier plan's
//! refusal register plus the Day-7 survey sweep, each with a reason +
//! salvage note). That is 30/286 ≈ 10.5% coverage — low against the full
//! theoretical product, but 100% of the cells this session made a real
//! decision about. `pass_rate` is measured only over those 30 evaluated
//! cells, and — because every admitted integration was independently
//! re-verified and every refusal is a deliberate, reasoned "no" — it is
//! exactly 1.0, not fudged.
//!
//! The threshold below (`0.09`) is set with a margin under the observed
//! 30/286 ≈ 0.105, so the test fails loudly (rather than silently passing)
//! if a future edit removes verified cells without adding replacements,
//! while tolerating axis growth (more sources/sockets surveyed but not yet
//! evaluated) without needlessly flapping.

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use my_conforming_project::frontier::{
    build_frontier_matrix, evaluated_count, frontier_report, refused_count, write_report,
    CAPABILITY_SOURCES, PRAXIS_SOCKETS,
};

/// Coverage floor: 30/286 ≈ 0.105 observed; threshold set with margin.
/// See the module doc above for why 1.0 coverage is not the right target.
const MIN_COVERAGE: f64 = 0.09;

#[test]
fn matrix_is_the_full_cartesian_product_of_both_axes() {
    let m = build_frontier_matrix();
    assert_eq!(
        m.axes.len(),
        2,
        "frontier matrix must have exactly two axes"
    );
    assert_eq!(m.axes[0].name, "capability_source");
    assert_eq!(m.axes[1].name, "praxis_socket");
    assert_eq!(m.total(), CAPABILITY_SOURCES.len() * PRAXIS_SOCKETS.len());
}

#[test]
fn matrix_validates_clean() {
    let m = build_frontier_matrix();
    let errors = m.validate();
    assert!(
        errors.is_empty(),
        "frontier matrix validate() errors: {errors:?}"
    );
}

#[test]
fn coverage_meets_the_justified_threshold() {
    let m = build_frontier_matrix();
    assert!(
        m.coverage() >= MIN_COVERAGE,
        "frontier coverage {} fell below the justified floor {MIN_COVERAGE} — see module docs",
        m.coverage()
    );
    assert_eq!(evaluated_count(), m.evaluated());
}

#[test]
fn pass_rate_is_one_over_evaluated_cells() {
    let m = build_frontier_matrix();
    assert!(
        (m.pass_rate() - 1.0).abs() < f64::EPSILON,
        "frontier pass_rate over evaluated cells must be 1.0 (every admitted integration was \
         independently re-verified and every refusal is a deliberate, reasoned no); was {}",
        m.pass_rate()
    );
    let report = frontier_report();
    assert!(
        report.failures.is_empty(),
        "frontier report must have zero failures, got {:?}",
        report.failures
    );
    assert!((report.pass_rate - 1.0).abs() < f64::EPSILON);
    assert_eq!(report.evaluated, m.evaluated());
}

#[test]
fn every_capability_source_and_socket_from_the_directive_is_an_axis_variant() {
    let m = build_frontier_matrix();
    let sources = &m.axes[0].variants;
    let sockets = &m.axes[1].variants;
    for s in CAPABILITY_SOURCES {
        assert!(
            sources.iter().any(|v| v == s),
            "missing capability_source variant: {s}"
        );
    }
    for s in PRAXIS_SOCKETS {
        assert!(
            sockets.iter().any(|v| v == s),
            "missing praxis_socket variant: {s}"
        );
    }
}

#[test]
fn refused_cells_carry_a_reason_and_a_salvage_witness() {
    let m = build_frontier_matrix();
    let refused: Vec<_> = m.cells.iter().filter(|c| c.is_impossible).collect();
    assert_eq!(
        refused.len(),
        16,
        "expected exactly the 16 refusal-register rows wired up"
    );
    for cell in refused {
        assert!(
            cell.refusal_reason.as_ref().is_some_and(|r| !r.is_empty()),
            "refused cell {:?} must record a reason, got {:?}",
            cell.coords,
            cell.refusal_reason
        );
        assert!(
            cell.manufacture_witness
                .as_ref()
                .is_some_and(|w| !w.is_empty()),
            "refused cell {:?} must record what was salvaged instead, got {:?}",
            cell.coords,
            cell.manufacture_witness
        );
        assert_eq!(
            cell.actual_standing,
            wasm4pm_compat::dfcm::Standing::Refused,
            "refused cell {:?} must have actual_standing == Refused (a refusal is a first-class \
             receipt, not a gap)",
            cell.coords
        );
    }
}

#[test]
fn admitted_cells_carry_a_fixture_describing_the_real_check() {
    let m = build_frontier_matrix();
    let admitted: Vec<_> = m
        .cells
        .iter()
        .filter(|c| {
            !c.is_impossible
                && matches!(
                    c.actual_standing,
                    wasm4pm_compat::dfcm::Standing::Admitted
                        | wasm4pm_compat::dfcm::Standing::Executed
                )
        })
        .collect();
    assert_eq!(
        admitted.len(),
        14,
        "expected exactly the 14 verified admitted integrations"
    );
    for cell in admitted {
        assert!(
            cell.fixture.as_ref().is_some_and(|f| !f.is_empty()),
            "admitted cell {:?} must record the fixture that verified it, got {:?}",
            cell.coords,
            cell.fixture
        );
    }
}

#[test]
fn admitted_and_refused_are_disjoint_and_account_for_every_evaluated_cell() {
    let m = build_frontier_matrix();
    let admitted_count = m
        .cells
        .iter()
        .filter(|c| {
            !c.is_impossible && c.actual_standing != wasm4pm_compat::dfcm::Standing::Unknown
        })
        .count();
    assert_eq!(admitted_count + refused_count(), evaluated_count());
}

#[test]
fn frontier_report_serializes_to_target_directory() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/frontier-report.json");
    let report = write_report(&path).expect("write_report should succeed");
    assert!(
        path.exists(),
        "target/frontier-report.json must exist after write_report"
    );
    assert_eq!(report.summary.total, build_frontier_matrix().total());
    assert!((report.summary.pass_rate - 1.0).abs() < f64::EPSILON);
}
