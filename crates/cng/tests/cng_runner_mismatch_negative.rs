//! CNG_R07 `RunnerMismatch` negative-path proof (docs/releases/v26.7.10/GAP_AUDIT.md
//! section 3.1/3.4 item 4): `RunnerMismatch` is constructed at 5 sites in
//! `crates/cng/src/runner.rs`, guarding the conformance check that binds a
//! projected POWL v2 model to its source `Pddl8Tape` and then admits/executes
//! it on the published `bcinr-powl 26.6.25` runtime — yet before this file
//! only the happy path was ever exercised (`tests/cng_hierarchical.rs`'s
//! `.expect(...)`-on-success calls). Zero test proved the refusal actually
//! fires.
//!
//! The 5 construction sites (grep-verified, `crates/cng/src/runner.rs`):
//!   1. line 118 (`compile_error_to_refusal`) — `bcinr_powl::compiler::compile_powl`
//!      itself refused the lowered AST (its own Kahn's-algorithm acyclicity /
//!      reachability admission check).
//!   2. line 177 (`run_labels_and_edges`) — model activity-leaf count != source
//!      tape op count ("detached from its plan").
//!   3. line 186 (`run_labels_and_edges`) — a model leaf label != the
//!      corresponding tape op's label at the same index.
//!   4. line 269 (`run_labels_and_edges`) — the scheduler did not fire every
//!      activity slot within the bounded tick budget.
//!   5. line 279 (`run_labels_and_edges`) — an activity fired before a
//!      projected predecessor (conformance violated).
//!
//! This file exercises two of the five sites end to end through the real
//! `cng::runner::validate_run` entry point — no mocked compiler, no mocked
//! scheduler — chosen because they are independently constructible from the
//! public `Powl`/`Pddl8Tape` APIs without needing to defeat the bounded-tick
//! scheduler loop:
//!
//!   - Site 3 (label mismatch): a `Powl` model naming an activity leaf that
//!     does not match the source tape's op label at the same index.
//!   - Site 1 (Kahn cycle): a `Powl::PartialOrder` whose `order` set is a
//!     genuine 2-cycle. `model_to_labels_and_edges` only range-checks order
//!     pairs (`crates/cng/src/powl.rs` — no acyclicity check of its own), so
//!     the cycle survives into the lowered `bcinr_powl::compiler::PowlAstNode`
//!     and is caught by the *published runtime's own* Kahn's-algorithm check
//!     (`bcinr-powl-26.6.25/src/compiler.rs::kahn_check`), whose own unit
//!     test `kahn_check_rejects_non_loop_cycle` proves the identical
//!     `edges: vec![(0, 1), (1, 0)]` shape yields `CompileError::Cycle`.
//!
//! Both are constructed directly against the public `Powl` enum and
//! `Pddl8Tape::from_plan`, matching `cng_hierarchical.rs`'s and
//! `cng_ipc_corpus.rs`'s established tape/model construction convention —
//! no inline Turtle, no pipeline detour, the real `validate_run` path only.

use std::collections::BTreeSet;

use bcinr_pddl::{Pddl8GroundAction, Pddl8Tape};

use cng::powl::{CngRefusal, Powl};
use cng::runner::validate_run;

/// Build a minimal ground action with no preconditions/effects: `validate_run`
/// only reads `label`, so an empty precondition/effect surface is sufficient
/// and keeps the fixture free of any planning-domain machinery.
fn ground_action(label: &str) -> Pddl8GroundAction {
    Pddl8GroundAction {
        schema_name: label.to_string(),
        label: label.to_string(),
        preconditions: Vec::new(),
        add_effects: Vec::new(),
        del_effects: Vec::new(),
    }
}

/// Site 3 (`runner.rs:186`): the POWL v2 model's second activity leaf is
/// named `"op-b-wrong"` while the source `Pddl8Tape`'s second op is actually
/// labelled `"op-b"` — same count (2 == 2), same order shape, but the labels
/// at index 1 disagree. `validate_run` must refuse before ever reaching
/// `compile_powl`, naming the exact leaf index and both conflicting labels.
#[test]
fn model_leaf_label_disagrees_with_tape_op_refuses_cng_r07() {
    let tape = Pddl8Tape::from_plan(vec![ground_action("op-a"), ground_action("op-b")]);
    assert_eq!(tape.ops.len(), 2, "precondition: two-op tape");
    assert_eq!(tape.ops[1].label, "op-b", "precondition: tape op 1 is op-b");

    let model = Powl::PartialOrder {
        children: vec![
            Powl::Leaf(Some("op-a".to_string())),
            Powl::Leaf(Some("op-b-wrong".to_string())),
        ],
        order: BTreeSet::from([(0, 1)]),
    };

    match validate_run(&tape, &model) {
        Err(refusal @ CngRefusal::RunnerMismatch(_)) => {
            assert_eq!(refusal.code(), "CNG_R07");
            let message = refusal.message();
            assert!(
                message.contains("op-b-wrong") && message.contains("op-b"),
                "RunnerMismatch message must name both conflicting labels: got {message:?}"
            );
            assert!(
                message.contains("does not correspond to the plan"),
                "RunnerMismatch message must identify the detached-model failure: got {message:?}"
            );
        }
        Err(other) => panic!(
            "expected CNG_R07 RunnerMismatch for a label-mismatched model/tape pair, \
             got {other:?} (code {})",
            other.code()
        ),
        Ok(report) => panic!(
            "expected CNG_R07 RunnerMismatch for a label-mismatched model/tape pair, \
             got a validated run instead: {report:?}"
        ),
    }
}

/// Site 1 (`runner.rs:118`): a `Powl::PartialOrder` whose `order` set is a
/// genuine 2-cycle (`{(0, 1), (1, 0)}`). `model_to_labels_and_edges`
/// (`crates/cng/src/powl.rs`) only range-checks order-pair indices — it
/// performs no acyclicity check of its own — so the cycle survives lowering
/// into `bcinr_powl::compiler::PowlAstNode::PartialOrder` unchanged and is
/// caught by the *published runtime's own* Kahn's-algorithm admission check.
/// This is the same edge shape as bcinr-powl 26.6.25's own
/// `kahn_check_rejects_non_loop_cycle` unit test, which asserts
/// `compile_powl` returns exactly `Err(CompileError::Cycle)` for
/// `edges: vec![(0, 1), (1, 0)]` — proving the trigger here is not an
/// artifact of cng's adapter but the real runtime's own admission gate.
#[test]
fn cyclic_order_relation_refuses_cng_r07_via_compile_powl_kahn_check() {
    let tape = Pddl8Tape::from_plan(vec![ground_action("op-a"), ground_action("op-b")]);
    assert_eq!(tape.ops.len(), 2, "precondition: two-op tape");

    let model = Powl::PartialOrder {
        children: vec![
            Powl::Leaf(Some("op-a".to_string())),
            Powl::Leaf(Some("op-b".to_string())),
        ],
        // A genuine 2-cycle: op-a precedes op-b AND op-b precedes op-a.
        // `Powl`'s type does not itself forbid this (the "pre-closed total
        // order" convention is enforced by the *producing* pipeline code,
        // not by the enum), so this is a legitimate, if malformed, model.
        order: BTreeSet::from([(0, 1), (1, 0)]),
    };

    match validate_run(&tape, &model) {
        Err(refusal @ CngRefusal::RunnerMismatch(_)) => {
            assert_eq!(refusal.code(), "CNG_R07");
            let message = refusal.message();
            assert!(
                message.contains("compile_powl refused the workflow"),
                "RunnerMismatch message must identify the compile_powl admission \
                 failure: got {message:?}"
            );
            assert!(
                message.contains("Cycle"),
                "RunnerMismatch message must surface bcinr-powl's own \
                 CompileError::Cycle verdict: got {message:?}"
            );
        }
        Err(other) => panic!(
            "expected CNG_R07 RunnerMismatch for a cyclic order relation, \
             got {other:?} (code {})",
            other.code()
        ),
        Ok(report) => panic!(
            "expected CNG_R07 RunnerMismatch for a cyclic order relation \
             (bcinr-powl's own Kahn check should reject it), got a validated \
             run instead: {report:?}"
        ),
    }
}
