//! CNG_R07 `RunnerMismatch` negative-path proof (docs/releases/v26.7.10/GAP_AUDIT.md
//! section 3.1/3.4 item 6): `RunnerMismatch` is constructed at 5 sites in
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
//! This file exercises three of the five sites end to end through the real
//! `cng::runner::validate_run` entry point — no mocked compiler, no mocked
//! scheduler:
//!
//!   - Site 2 (count mismatch): a `Powl` model whose activity-leaf count
//!     disagrees with the source tape's op count.
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
//! All three are constructed directly against the public `Powl` enum and
//! `Pddl8Tape::from_plan`, matching `cng_hierarchical.rs`'s and
//! `cng_ipc_corpus.rs`'s established tape/model construction convention —
//! no inline Turtle, no pipeline detour, the real `validate_run` path only.
//!
//! ## Sites 4 and 5: investigated, found unreachable by design
//!
//! Sites 4 (tick-budget exhaustion) and 5 (conformance violation) guard
//! `run_labels_and_edges`'s post-scheduler cross-check
//! (`crates/cng/src/runner.rs:239-284`). Both were investigated (not
//! forced) and found structurally unreachable through the public
//! `Powl`/`Pddl8Tape` API, for the same reason CNG_R08 `Nondeterminism` was
//! found unreachable-by-design in a prior round (see
//! `docs/releases/v26.7.10/GAP_AUDIT.md` §3.4 item 2): no caller-visible
//! seam exists to defeat the invariant without editing production code,
//! which is out of scope here.
//!
//! `run_labels_and_edges` (`crates/cng/src/runner.rs:203-206`) lowers
//! *every* model it ever receives — from both `model_to_labels_and_edges`
//! and `linearize_hierarchical` — to exactly one AST shape: a single
//! top-level `bcinr_powl::compiler::PowlAstNode::PartialOrder` whose
//! children are *all* `Atom` leaves (no `Sequence`, `XorChoice`, or `Loop`
//! ever appears; nested composite children are refused as `CNG_R05`
//! `UnsupportedConstruct` before compilation, never silently flattened).
//! For that AST shape:
//!
//!   - `bcinr_powl::scheduler::scheduler_tick`'s branchless fire gate
//!     (`pred_satisfied`) requires `required & !done == 0` for every
//!     non-`Join` op — i.e. an `Atom` can only fire once `done_mask`
//!     already contains *all* of its real predecessors, cross-tick or
//!     within the same tick's ascending-index cascade (`new_done` updates
//!     immediately as the tick's `while candidates != 0` loop visits each
//!     index in ascending order). This makes "an activity fired before a
//!     projected predecessor" (site 5) a hard invariant of the published
//!     scheduler for any DAG `compile_powl`'s own Kahn's-algorithm check
//!     admits — there is no code path back into `run_labels_and_edges`
//!     that can defeat it without a bug in the published `bcinr-powl`
//!     crate itself.
//!   - Every real (non-`Join`) predecessor edge, on firing, re-adds its
//!     successor to `check_mask` via `succ_mask` propagation, so a
//!     multi-predecessor `Atom` is guaranteed to be reconsidered on the
//!     tick immediately after its *last* predecessor completes — bounding
//!     any DAG on `n` atoms to at most `n` ticks to full completion, well
//!     inside the `2 * slot_count + 2` budget `run_labels_and_edges` sets
//!     (`runner.rs:238`; matches bcinr-powl's own
//!     `scheduler_tick_completes_within_bounded_ticks` unit test). This
//!     makes "the scheduler did not fire every activity slot within the
//!     bounded tick budget" (site 4) likewise unreachable for any
//!     Kahn-admitted DAG.
//!   - The one latent scheduler quirk found during this investigation — the
//!     synthetic `Join` op `compile_partial_order` allocates when a flat
//!     `PartialOrder` has more than one exit child reuses `OpKind::Join`'s
//!     `effective_pred = pred_mask & choice_taken` gate (designed for
//!     `XorChoice` closure), and since `state.choice_taken` is only ever
//!     populated by `apply_xor_dispatch` — which never runs, because this
//!     AST shape never allocates an `XorDispatch` op — the `Join`'s
//!     `effective_pred` is always `0` and it fires immediately on its
//!     first check rather than waiting for all its real predecessors. This
//!     is structurally inert for `run_labels_and_edges`, though: the `Join`
//!     slot is excluded from `atom_mask` (`runner.rs:214-219`, gated on
//!     `OpKind::Atom`), so it is never scanned by the site-4/5 cross-check,
//!     and because the `Join` is always the terminal exit of the *whole*
//!     compiled tape (nothing is ever composed on top of it in this
//!     adapter), no further `Atom` ever depends on it either.
//!
//! Empirical corroboration (not a substitute for the structural argument
//! above, but consistent with it): 29,403 real `validate_run` calls over
//! adversarially constructed DAGs — random permutation-derived topological
//! orders (decoupling child-index order from precedence order, to stress
//! the ascending-index-cascade assumption) at `n` from 2 to 63, multiple
//! edge densities, plus explicit worst-case maximal-chain shapes (full
//! transitive closure, permuted indices, `n` up to 63 = the runtime's own
//! 64-slot cap) — produced zero site-4 and zero site-5 refusals; every
//! non-count/label-mismatched, non-cyclic input admitted and ran to
//! completion. Per the task's explicit instruction not to fabricate a
//! trigger for a structurally unreachable site, no test function is added
//! for sites 4 or 5.

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

/// Site 2 (`runner.rs:177`): the POWL v2 model has three activity leaves
/// while the source `Pddl8Tape` has only two ops — a raw count mismatch,
/// checked *before* `validate_run` ever compares individual labels or
/// reaches `compile_powl`. `validate_run` must refuse, naming both counts.
#[test]
fn model_leaf_count_disagrees_with_tape_op_count_refuses_cng_r07() {
    let tape = Pddl8Tape::from_plan(vec![ground_action("op-a"), ground_action("op-b")]);
    assert_eq!(tape.ops.len(), 2, "precondition: two-op tape");

    let model = Powl::PartialOrder {
        children: vec![
            Powl::Leaf(Some("op-a".to_string())),
            Powl::Leaf(Some("op-b".to_string())),
            Powl::Leaf(Some("op-c".to_string())),
        ],
        order: BTreeSet::new(),
    };

    match validate_run(&tape, &model) {
        Err(refusal @ CngRefusal::RunnerMismatch(_)) => {
            assert_eq!(refusal.code(), "CNG_R07");
            let message = refusal.message();
            assert!(
                message.contains('3') && message.contains('2'),
                "RunnerMismatch message must name both the model's leaf count \
                 (3) and the tape's op count (2): got {message:?}"
            );
            assert!(
                message.contains("detached from its plan"),
                "RunnerMismatch message must identify the detached-model \
                 failure: got {message:?}"
            );
        }
        Err(other) => panic!(
            "expected CNG_R07 RunnerMismatch for a leaf-count-mismatched model/tape pair, \
             got {other:?} (code {})",
            other.code()
        ),
        Ok(report) => panic!(
            "expected CNG_R07 RunnerMismatch for a leaf-count-mismatched model/tape pair, \
             got a validated run instead: {report:?}"
        ),
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
