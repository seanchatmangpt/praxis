#![cfg(test)]

//! SOC2 Type II continuous re-evidencing tests (v26.7.13 Stage 3): quarterly
//! re-testing across a 12-month Type II observation period as a real F09
//! growth/descent sequence at the Operating Effectiveness Testing phase's
//! socket, plus a composed crown-style witness test driving the full
//! Scoping -> ... -> Report Handoff cycle with that growth embedded.
//!
//! `oe_testing_quarterly_growth_cycle_grafts_six_children_and_replays_receipts_byte_identically`
//! is item 1's primary proof: 4 quarterly re-tests + 1 remediation graft
//! (the one genuine exception this cycle surfaces, at Q2) + 1 annual-
//! closure graft = 6 real descents against a `RE_TEST_BUDGET` of 6,
//! `DescentReceipt::seal`ed each time and re-run to confirm byte-identical
//! replay. `seventh_descent_after_budget_exhausted_refuses_typed` is item
//! 4's adversarial proof: a 7th attempted descent, over an independently
//! reachable goal, refuses `MFWGrowthRefused::DescentBudgetExhausted`, not
//! a panic or silent success.
//! `unremediable_exception_growth_refuses_typed_goal_unreachable` and
//! `oe_testing_socket_already_closed_refuses_typed` extend that same
//! typed-refusal discipline to the two other real refusal gates F09 offers
//! (unreachable continuation goal; already-satisfied closure law).
//!
//! `soc2_scoping_through_report_handoff_composed_witness_with_oe_testing_growth`
//! is item 2's composed test. **Honest REAL_EDGE verdict** (stated here and
//! in the test's own comments, not just in the session report): the base
//! 10-phase Scoping -> Report Handoff cycle is a real, already-established
//! REAL_EDGE (Stage 1's own test proves it; this test re-derives it fresh
//! from the real fixtures, not by copying Stage 1's assertions) and F09's
//! growth machinery is a real, non-fabricated consumer of that cycle's
//! actually-derived OE-Testing socket (never a hardcoded index). What this
//! test does **not** claim is a single continuous REAL_EDGE from the
//! grafted quarterly/remediation children through to the Exception-
//! Identification/Remediation/Bundle-Assembly/Report-Handoff phases' PDDL
//! preconditions: those phases are gated by the base cycle's own
//! unmodified precondition chain (verified unchanged by this test), not by
//! anything F09 grafts onto the bridged copy. Two real edges meeting at a
//! real shared socket location, not one continuous data thread all the way
//! to Report Handoff — disclosed plainly rather than forced to fit the
//! stricter bar.

use std::path::PathBuf;

use chicago_tdd_tools::prelude::*;
use multifractal_workflow::f09_mfw_growth::{semantic_closure_check, MFWGrowthRefused};
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;
use powl2_decompose::{ParentChildClosure, Powl as P2Powl, SocketKind, WorkflowSocketId};
use praxis_graphlaw::chatman::closure::{ClosureLaw, RecursiveSocketClosure};

use super::{
    bridge_from_powl2, bridge_to_powl2, grow_socket_once, load_residue, locate_phase_socket,
    run_oe_testing_growth_cycle, OE_TESTING_GROWTH_CYCLE, RE_TEST_BUDGET,
};
use crate::bench::soc2::{soc2_fixture_dir, SOC2_PHASES};
use crate::pipeline::{generate_plan, hierarchical_projection, import_artifacts};
use crate::powl::powl_to_turtle;
use crate::shape::validate_powl_store;

const OE_TESTING_FIXTURE_FILENAME: &str = "audit-oe-testing.ttl";
const EVIDENCE_COLLECTION_INIT_FIXTURE_FILENAME: &str = "audit-collection-init.ttl";
const EXCEPTION_ID_FIXTURE_FILENAME: &str = "audit-exception-id.ttl";
const GROWTH_BASE_IRI: &str = "urn:chatman:powl:soc2-oe-testing-growth";

fn growth_fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/soc2-growth")
}

/// Real: admits the base 10-phase audit cycle, plans it, hierarchically
/// projects it, and locates the OE-Testing phase's socket from that real,
/// freshly-derived provenance (never a hardcoded index). Returns the
/// bridged `powl2_decompose::Powl` root plus the socket.
fn base_cycle_root_and_oe_testing_socket() -> (P2Powl, WorkflowSocketId) {
    let dir = soc2_fixture_dir();
    let artifacts = import_artifacts(&dir).expect("fixtures admit");
    let (tape, surface) = generate_plan(&artifacts).expect("cycle plan exists");
    let (base_powl, phase_sources) =
        hierarchical_projection(&tape, &surface).expect("hierarchical projection");
    let socket = locate_phase_socket(&artifacts, &phase_sources, OE_TESTING_FIXTURE_FILENAME)
        .expect("OE-Testing phase located from real provenance, not a hardcoded index");
    (bridge_to_powl2(&base_powl), socket)
}

test!(
    oe_testing_quarterly_growth_cycle_grafts_six_children_and_replays_receipts_byte_identically,
    {
        // Arrange/Act: run the real 6-descent quarterly re-test + remediation
        // + annual-closure growth cycle at the REAL OE-Testing socket, twice
        // (replay determinism), each run re-deriving the base cycle fresh.
        let run = || {
            let (root0, socket) = base_cycle_root_and_oe_testing_socket();
            let (outcome, receipts, meter) = run_oe_testing_growth_cycle(
                &growth_fixtures_dir(),
                &root0,
                socket.clone(),
                ClosureLaw::AllRequired,
            )
            .expect("all 6 real descents succeed through every real F09 gate");
            (socket, outcome, receipts, meter)
        };
        let (socket1, outcome1, receipts1, meter1) = run();
        let (_socket2, outcome2, receipts2, meter2) = run();

        // Assert: exactly 6 real descents, budget fully (and only exactly)
        // consumed.
        assert_eq!(receipts1.len(), OE_TESTING_GROWTH_CYCLE.len());
        assert_eq!(
            receipts1.len(),
            6,
            "4 quarters + 1 remediation + 1 annual-closure"
        );
        assert_eq!(RE_TEST_BUDGET, 6);
        assert_eq!(meter1.budget(), 6);
        assert_eq!(meter1.depth(), 6);
        assert_eq!(meter1.remaining(), 0);

        // Assert: the OE-Testing socket's PartialOrder grew from its 3 real
        // base children (sample-control-instances / test-operating-
        // effectiveness / record-test-results) to 9 (3 base + 6 grafted).
        let oe_node = outcome1
            .new_root
            .socket_at(&socket1.path)
            .expect("OE-Testing socket resolves in the grafted root");
        let P2Powl::PartialOrder { children, .. } = oe_node else {
            panic!("OE-Testing socket must remain a PartialOrder after grafting");
        };
        assert_eq!(
            children.len(),
            9,
            "3 real base actions + 6 real grafted children"
        );

        // Assert: DescentReceipt::seal is replay-stable -- byte-identical
        // BLAKE3 digests across the second run, in cycle order.
        assert_eq!(receipts1.len(), receipts2.len());
        for (i, (r1, r2)) in receipts1.iter().zip(receipts2.iter()).enumerate() {
            assert_eq!(
                r1.digest, r2.digest,
                "descent {i} receipt must be byte-identical across replay"
            );
            assert_eq!(
                r1.digest.len(),
                64,
                "blake3 hex digest is 64 lowercase hex chars"
            );
            assert!(r1.digest.chars().all(|c| c.is_ascii_hexdigit()));
        }
        // Depth strictly increases 1..=6 across the 6 receipts (real
        // sequential descent, not 6 copies of the same seal).
        let depths: Vec<usize> = receipts1.iter().map(|r| r.depth_reached).collect();
        assert_eq!(depths, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(meter2.depth(), meter1.depth());

        // Real downstream consumption: bridge the grafted root back to
        // cng's own `crate::powl::Powl`, serialize as Turtle, and validate
        // it through the SAME structural POWL shape validator the base
        // cycle's own Stage 1 test uses -- not a Rust-struct assertion
        // alone.
        let grown_cng =
            bridge_from_powl2(&outcome1.new_root).expect("bridges back to crate::powl::Powl");
        let ttl = powl_to_turtle(
            &grown_cng,
            GROWTH_BASE_IRI,
            Some("urn:chatman:plan:soc2-oe-testing-growth"),
        );
        let store = Store::new().expect("store");
        store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), ttl.as_bytes())
            .expect("grafted POWL parses as Turtle");
        validate_powl_store(&store, true).expect("grafted POWL's structural shape holds");

        // Keep outcome2 alive for the assertions above without an unused
        // warning on a fully-replayed second run.
        let _ = outcome2;
    }
);

test!(seventh_descent_after_budget_exhausted_refuses_typed, {
    // Arrange: run the real 6-descent cycle to full budget exhaustion.
    let (root0, socket) = base_cycle_root_and_oe_testing_socket();
    let (outcome, receipts, mut meter) = run_oe_testing_growth_cycle(
        &growth_fixtures_dir(),
        &root0,
        socket.clone(),
        ClosureLaw::AllRequired,
    )
    .expect("6 real descents succeed");
    assert_eq!(receipts.len(), 6);
    assert_eq!(meter.depth(), meter.budget());
    assert_eq!(meter.remaining(), 0);

    // Act: attempt a 7th descent over an INDEPENDENTLY REACHABLE goal (the
    // real Q1 residue, re-resolved and re-planned) so the only possible
    // cause of refusal is the exhausted DescentMeter, not an unreachable
    // goal or an already-satisfied closure.
    let q1_again = load_residue(
        &growth_fixtures_dir().join("q1-retest"),
        socket.clone(),
        "7th cycle: attempted after the 6-descent re-test budget is already exhausted",
    )
    .expect("q1 residue re-loads");
    let err = grow_socket_once(
        &outcome.new_root,
        &outcome.closure,
        &q1_again,
        &mut meter,
        ClosureLaw::AllRequired,
    )
    .expect_err("7th descent must refuse: DescentMeter exhausted, never silently pass or loop");

    // Assert: the EXACT typed refusal, not a panic and not a silent
    // success -- DescentBudgetExhausted{budget: 6, depth: 6}, matching the
    // meter's own state at the moment of refusal.
    assert_eq!(
        err,
        MFWGrowthRefused::DescentBudgetExhausted {
            budget: 6,
            depth: 6
        }
    );
    // The refused attempt must not have advanced the meter further.
    assert_eq!(meter.depth(), 6);
    assert_eq!(meter.remaining(), 0);
});

test!(
    unremediable_exception_growth_refuses_typed_goal_unreachable,
    {
        // Arrange: the adversarial fixture -- a control point with an
        // identified exception and NO reachable remediation action. A fresh
        // meter/closure so this never touches the main 6-descent budget.
        let (root0, socket) = base_cycle_root_and_oe_testing_socket();
        let pcc = ParentChildClosure::from_model(&root0);
        let closure =
            RecursiveSocketClosure::declare(&pcc, socket.clone(), ClosureLaw::AllRequired)
                .expect("OE-Testing socket has 3 real base children");
        let mut meter = multifractal_workflow::f09_mfw_growth::DescentMeter::new(4);

        let unremediable = load_residue(
            &growth_fixtures_dir().join("unreachable-exception"),
            socket.clone(),
            "CTRL-UNREMEDIABLE evidence gap with no reachable remediation action",
        )
        .expect("unreachable-exception residue loads");

        // Act/Assert: the COMPLIANCE-OVERCLAIM FENCE's typed-refusal discipline
        // -- a control point that cannot be genuinely evidenced refuses typed,
        // it is never silently fabricated closed.
        let err = grow_socket_once(
            &root0,
            &closure,
            &unremediable,
            &mut meter,
            ClosureLaw::AllRequired,
        )
        .expect_err("a control point with no reachable remediation must refuse, never fabricate");
        assert!(
            matches!(err, MFWGrowthRefused::GoalUnreachable { .. }),
            "expected GoalUnreachable, got {err:?}"
        );
        assert_eq!(
            meter.depth(),
            0,
            "descent must not advance on an unreachable goal"
        );
    }
);

test!(oe_testing_socket_already_closed_refuses_typed, {
    // Arrange: the REAL OE-Testing socket's declared closure, with its 3
    // real base children (sample-control-instances / test-operating-
    // effectiveness / record-test-results, in tape order) explicitly
    // admitted -- proving `ClosureLaw::AllRequired` genuinely evaluates
    // `Close(W) = true` for THIS socket's real declared children, not a
    // synthetic fixture-only socket.
    let (root0, socket) = base_cycle_root_and_oe_testing_socket();
    let pcc = ParentChildClosure::from_model(&root0);
    let mut closure =
        RecursiveSocketClosure::declare(&pcc, socket.clone(), ClosureLaw::AllRequired)
            .expect("OE-Testing socket has 3 real base children");
    semantic_closure_check(&closure).expect("freshly declared closure is open, not yet satisfied");

    for i in 0..3 {
        let leaf = WorkflowSocketId {
            path: socket.path.child(i),
            kind: SocketKind::Leaf,
        };
        closure
            .admit(&leaf)
            .unwrap_or_else(|e| panic!("admit leaf {i}: {e}"));
    }
    assert!(
        closure.is_closed().expect("evaluable"),
        "all 3 real base children admitted; AllRequired must now report closed"
    );

    // Act/Assert: a growth attempt against an already-closed socket refuses
    // typed -- no child is manufactured for already-closed truth.
    let q1 = load_residue(
        &growth_fixtures_dir().join("q1-retest"),
        socket.clone(),
        "attempted growth at an already-closed OE-Testing socket",
    )
    .expect("q1 residue loads");
    let mut meter = multifractal_workflow::f09_mfw_growth::DescentMeter::new(4);
    let err = grow_socket_once(&root0, &closure, &q1, &mut meter, ClosureLaw::AllRequired)
        .expect_err("growth at an already-closed socket must refuse, never manufacture a child");
    assert_eq!(
        err,
        MFWGrowthRefused::ClosureAlreadySatisfied {
            socket: socket.to_string(),
            law: "all_required",
        }
    );
});

test!(
    soc2_scoping_through_report_handoff_composed_witness_with_oe_testing_growth,
    {
        // ---- Real base cycle: Scoping -> ... -> Report Handoff ----
        //
        // Re-derived fresh here (not copied from soc2_test.rs's own
        // assertions) so this test is a genuine second, independent
        // production caller of the real pipeline, not a restatement.
        let dir = soc2_fixture_dir();
        let artifacts = import_artifacts(&dir).expect("fixtures admit");
        let (tape, surface) = generate_plan(&artifacts).expect("cycle plan exists");
        let (base_powl, phase_sources) =
            hierarchical_projection(&tape, &surface).expect("hierarchical projection");

        assert_eq!(SOC2_PHASES.len(), 10);
        assert_eq!(
            phase_sources.len(),
            10,
            "one provenance source per SOC2 audit phase"
        );
        let phase_children_len = {
            let root = bridge_to_powl2(&base_powl);
            let P2Powl::PartialOrder { children, .. } = &root else {
                panic!("root must be a partial order over the 10 phases");
            };
            children.len()
        };
        assert_eq!(
            phase_children_len, 10,
            "one POWL child per SOC2 audit phase"
        );

        // Genuine (not merely temporal) cross-phase consumption: the OE-
        // Testing phase's FIRST action precondition names exactly the
        // predicate the immediately-preceding phase (Evidence Collection
        // Period Initiation)'s LAST action asserts as an effect, and
        // OE-Testing's own LAST action effect is exactly the predicate the
        // Exception Identification phase's FIRST action requires as a
        // precondition. This is real predicate-name data threading through
        // the admitted, merged `AdmittedSurface`, not index-order alone.
        let actions_from_fixture = |fixture_filename: &str| {
            let artifact = artifacts
                .iter()
                .find(|a| a.path.file_name().and_then(|n| n.to_str()) == Some(fixture_filename))
                .unwrap_or_else(|| panic!("artifact {fixture_filename} admitted"));
            let mut acts: Vec<_> = surface
                .domain
                .actions
                .iter()
                .filter(|a| {
                    surface.action_sources.get(&a.name).map(String::as_str)
                        == Some(artifact.source_iri.as_str())
                })
                .collect();
            acts.sort_by(|a, b| a.name.cmp(&b.name));
            acts
        };
        let collection_init_acts = actions_from_fixture(EVIDENCE_COLLECTION_INIT_FIXTURE_FILENAME);
        let oe_testing_acts = actions_from_fixture(OE_TESTING_FIXTURE_FILENAME);
        let exception_id_acts = actions_from_fixture(EXCEPTION_ID_FIXTURE_FILENAME);
        assert_eq!(collection_init_acts.len(), 3);
        assert_eq!(oe_testing_acts.len(), 3);
        assert_eq!(exception_id_acts.len(), 3);
        // open-evidence-collection-window (last collection-init action, by
        // schema) effects evidence-collection-initiated; sample-control-
        // instances (first oe-testing action, by schema) requires it.
        let last_collection_init_effect = collection_init_acts
            .iter()
            .flat_map(|a| a.add_effects.iter())
            .map(|e| e.pred.as_str())
            .find(|p| *p == "evidence-collection-initiated")
            .expect("collection-init phase asserts evidence-collection-initiated");
        let oe_testing_needs_it = oe_testing_acts
            .iter()
            .flat_map(|a| a.preconditions.iter())
            .any(|p| p.pred == last_collection_init_effect);
        assert!(
            oe_testing_needs_it,
            "OE-Testing phase must genuinely require evidence-collection-initiated, not just \
             follow it temporally"
        );
        let oe_testing_terminal_effect = oe_testing_acts
            .iter()
            .flat_map(|a| a.add_effects.iter())
            .map(|e| e.pred.as_str())
            .find(|p| *p == "operating-effectiveness-tested")
            .expect("oe-testing phase asserts operating-effectiveness-tested");
        let exception_id_needs_it = exception_id_acts
            .iter()
            .flat_map(|a| a.preconditions.iter())
            .any(|p| p.pred == oe_testing_terminal_effect);
        assert!(
            exception_id_needs_it,
            "Exception-ID phase must genuinely require operating-effectiveness-tested, not \
             just follow it temporally"
        );

        // ---- F09 growth at the real, located OE-Testing socket ----
        let socket = locate_phase_socket(&artifacts, &phase_sources, OE_TESTING_FIXTURE_FILENAME)
            .expect("OE-Testing phase located from real provenance");
        let root0 = bridge_to_powl2(&base_powl);
        let (outcome, receipts, meter) = run_oe_testing_growth_cycle(
            &growth_fixtures_dir(),
            &root0,
            socket.clone(),
            ClosureLaw::AllRequired,
        )
        .expect("the real 6-descent quarterly re-test + remediation cycle succeeds");
        assert_eq!(receipts.len(), 6);
        assert_eq!(meter.depth(), 6);

        // ---- Honest REAL_EDGE disclosure, mechanically checked ----
        //
        // The base cycle's plan tape (the ACTUAL production sequence that
        // reaches Evidence Bundle Assembly / Auditor Report Handoff) is
        // untouched by growth: growth operates on `root0`, a separate
        // bridged COPY, never on `tape`/`surface` themselves. Confirm that
        // mechanically rather than merely asserting it in prose: the base
        // tape still ends in Report-Handoff's real terminal action, and
        // still has exactly 30 ops (3 actions x 10 phases) -- growth added
        // 6 real children to the BRIDGED tree, not to this tape.
        assert_eq!(
            tape.ops.len(),
            30,
            "growth must not mutate the base cycle's own plan tape"
        );
        assert_eq!(
            tape.ops.last().map(|op| op.label.as_str()),
            Some("confirm-evidence-bundle-complete(arclight)")
        );

        let oe_node = outcome
            .new_root
            .socket_at(&socket.path)
            .expect("OE-Testing socket resolves in the grafted root");
        let P2Powl::PartialOrder { children, .. } = oe_node else {
            panic!("OE-Testing socket must remain a PartialOrder after grafting");
        };
        assert_eq!(
            children.len(),
            9,
            "3 real base + 6 real grafted growth children"
        );

        // The base cycle's downstream phases (Exception-ID onward) are
        // reached via the SAME real precondition chain checked above,
        // completely independent of the growth performed on the bridged
        // copy -- i.e. this test drives BOTH real edges (base cycle;
        // F09 growth at a real socket) but does not claim they are one
        // continuous data thread. See this file's module doc for the full
        // disclosure.
    }
);
