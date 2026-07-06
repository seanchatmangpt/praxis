//! Blue River Dam control benchmarks (divan).
//!
//! Measures the deterministic control surfaces that sit between an AI agent
//! and the ledger: standing/status transitions, planner masks and filters,
//! POWL scheduler ticks, receipt chain links, verifier gate dispatch, and the
//! Little's Law snapshot. Every function is the real production surface (or a
//! named proxy documented in
//! `docs/releases/v26.7.6/BLUE_RIVER_DAM_BENCHMARKS.md`).
//!
//! Runs alongside the existing criterion benches (`bench_main.rs`,
//! `receipt_validate.rs`) — it does not replace them.

// Recorded lint debt (v26.7.6 verification gate) -- see src/lib.rs and
// docs/releases/v26.7.6/RELEASE_CONTROL.md Sec. 9.
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::style, clippy::complexity, clippy::perf)]

use std::collections::HashMap;
use std::hint::black_box;

use bcinr_pddl::ground::{eval_condition, GroundProblem};
use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};
use bcinr_powl::admit::{admit, AdmissionContext};
use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::{OpKind, PowlTape};
use bcinr_powl_receipt::causal_receipt::{OcelCausalFrame, OcelCausalReceipt, PackedObjRef};
use bcinr_powl_receipt::denial::DenialPolarity;
use praxis_core::lifecycle::Raw;
use praxis_core::verify::run_pipeline;
use praxis_core::{DefaultLaw, Judge, LawObject, ReceiptRecord};
use serde_json::{json, Value};
use wasm4pm_compat::pddl::{Pddl8Domain, Pddl8Problem, PddlCondition};

fn main() {
    divan::main();
}

// ---------------------------------------------------------------------------
// Shared fixtures — the golden law lifecycle (judge → admit → receipt), the
// same shape `plan run` manufactures from ontology/lawobject.ttl.
// ---------------------------------------------------------------------------

/// Bench-owned STRIPS mirror of the golden law-lifecycle domain.
const GOLDEN_DOMAIN: &str = "\
(define (domain blue-river-law)
  (:predicates (raw ?o) (validated ?o) (admitted ?o) (receipted ?o)
               (evidence ?o) (cleared ?o))
  (:action supply-evidence
    :parameters (?o)
    :precondition (and (raw ?o))
    :effect (and (evidence ?o)))
  (:action clear-obligations
    :parameters (?o)
    :precondition (and (evidence ?o))
    :effect (and (cleared ?o)))
  (:action judge
    :parameters (?o)
    :precondition (and (raw ?o) (evidence ?o) (cleared ?o))
    :effect (and (validated ?o) (not (raw ?o))))
  (:action admit
    :parameters (?o)
    :precondition (and (validated ?o))
    :effect (and (admitted ?o) (not (validated ?o))))
  (:action receipt
    :parameters (?o)
    :precondition (and (admitted ?o))
    :effect (and (receipted ?o) (not (admitted ?o)))))";

const GOLDEN_PROBLEM: &str = "\
(define (problem claim-001)
  (:domain blue-river-law)
  (:objects claim)
  (:init (raw claim))
  (:goal (and (receipted claim))))";

const GOLDEN_STEPS: [&str; 5] = [
    "supply-evidence",
    "clear-obligations",
    "judge",
    "admit",
    "receipt",
];

fn golden_parsed() -> (Pddl8Domain, Pddl8Problem) {
    let domain = domain_from_pddl(GOLDEN_DOMAIN).expect("golden domain parses");
    let problem = problem_from_pddl(GOLDEN_PROBLEM).expect("golden problem parses");
    (domain, problem)
}

fn golden_tape() -> PowlTape {
    let atoms: Vec<PowlAstNode<'_>> = GOLDEN_STEPS.iter().map(|n| PowlAstNode::Atom(n)).collect();
    compile_powl(&PowlAstNode::Sequence(atoms)).expect("golden plan compiles")
}

/// A lawful chained JSONL-shaped ledger of `n` records, `1 µs` apart, each
/// sealing a 2 ms admission span (same construction as the verify_ops tests).
fn chained_records(n: u64) -> Vec<ReceiptRecord> {
    let mut records = Vec::new();
    let mut prev = [0u8; 32];
    for i in 1..=n {
        let payload_hash_hex = format!("{:02x}", (i % 251) as u8).repeat(32)[..64].to_string();
        let mut record = ReceiptRecord {
            version: 1,
            instruction_id: i,
            activity_idx: 0,
            activity: None,
            node_kind: 0,
            ts_ns: i * 1000,
            duration_ms: Some(2),
            payload_hash_hex,
            prev_chain_hash_hex: hex::encode(prev),
            chain_hash_hex: String::new(),
            andon: praxis_core::Andon::Green,
            obligation_count: 0,
            object_ids: vec![format!("law:instr{i}")],
        };
        let chain_hash = record.recompute_chain_hash().expect("recompute chain hash");
        record.chain_hash_hex = hex::encode(chain_hash);
        prev = chain_hash;
        records.push(record);
    }
    records
}

// ---------------------------------------------------------------------------
// Deterministic transition benchmarks
// ---------------------------------------------------------------------------

/// One standing-state transition (Raw → Validated) through the
/// `praxis_core::LawObject` status path — the same judge step every ledger
/// receipt issued by `plan run` (`ops::receipt_issue_payload`) goes through.
#[divan::bench]
fn standing_transition(bencher: divan::Bencher) {
    let payload = json!({"kind": "plan-run", "plan_len": 5});
    bencher
        .with_inputs(|| LawObject::<Value, Raw, DefaultLaw>::new(payload.clone(), Vec::new()))
        .bench_local_values(|raw| DefaultLaw::judge(black_box(raw)));
}

/// Planner action-eligibility mask: evaluate every ground action's
/// precondition set against the initial state, producing a u64 fired-set
/// style eligibility mask (the check `find_plan`'s BFS performs per node).
#[divan::bench]
fn action_precondition_mask(bencher: divan::Bencher) {
    let (domain, problem) = golden_parsed();
    let gp = GroundProblem::build(&domain, &problem, None).expect("golden grounds");
    let state = gp.initial_state.clone();
    let actions = gp.actions.clone();
    bencher.bench_local(|| {
        let mut mask = 0u64;
        for (i, a) in black_box(&actions).iter().enumerate() {
            if a.preconditions.iter().all(|p| state.contains(p)) {
                mask |= 1u64 << (i % 64);
            }
        }
        mask
    });
}

/// Candidate action admit/refuse filtering: type-checked grounding of every
/// action schema over the problem objects (`GroundProblem::build`), the
/// filter `ops::plan_solve_payload` runs before search.
#[divan::bench]
fn pddl_action_filter(bencher: divan::Bencher) {
    let (domain, problem) = golden_parsed();
    bencher.bench_local(|| {
        GroundProblem::build(black_box(&domain), black_box(&problem), None)
            .expect("golden grounds")
            .actions
            .len()
    });
}

/// Raw transition-table lookup: `bcinr_powl::admit::admit` — a single
/// branch-free index into the 256-entry compile-time topology LUT.
#[divan::bench]
fn bcinr_transition_table(bencher: divan::Bencher) {
    let mut ctx: AdmissionContext = 0;
    bencher.bench_local(move || {
        ctx = ctx.wrapping_add(0x9E37_79B9_7F4A_7C15);
        admit(black_box(ctx))
    });
}

/// Also exercise `eval_condition` (the recursive precondition evaluator the
/// temporal path uses) over the golden goal — reported alongside
/// `action_precondition_mask` as the condition-tree variant.
#[divan::bench]
fn action_precondition_mask_condition_tree(bencher: divan::Bencher) {
    let (domain, problem) = golden_parsed();
    let gp = GroundProblem::build(&domain, &problem, None).expect("golden grounds");
    let state = gp.initial_state.clone();
    let fn_vals: HashMap<String, f64> = HashMap::new();
    let goal = PddlCondition::And(
        problem
            .goal
            .iter()
            .map(|a| PddlCondition::Atom(a.clone()))
            .collect(),
    );
    bencher.bench_local(|| eval_condition(black_box(&goal), &state, &fn_vals));
}

// ---------------------------------------------------------------------------
// Workflow control benchmarks
// ---------------------------------------------------------------------------

/// One POWL workflow step advance: `scheduler_tick` over the compiled golden
/// 5-slot sequence tape from a fresh run state (fires slot 0).
#[divan::bench]
fn powl_step_tick(bencher: divan::Bencher) {
    let tape = golden_tape();
    let ops = tape.ops[..tape.len as usize].to_vec();
    bencher
        .with_inputs(|| PowlRunState::new(&tape))
        .bench_local_values(|mut state| scheduler_tick(black_box(&ops), &mut state));
}

// ---------------------------------------------------------------------------
// Receipt-link benchmarks
// ---------------------------------------------------------------------------

/// Building + chaining one `OcelCausalFrame` onto a genesis-folded
/// `OcelCausalReceipt` — the exact per-fired-atom BLAKE3 chain-hash step
/// `plan_run::execute_receipted` performs.
#[divan::bench]
fn receipt_frame_link(bencher: divan::Bencher) {
    let run_id: [u8; 32] = *blake3::hash(b"blue-river-dam").as_bytes();
    bencher
        .with_inputs(|| OcelCausalReceipt::genesis(run_id))
        .bench_local_values(|mut receipt| {
            let frame = OcelCausalFrame {
                instruction_id: 0,
                fired_mask: 1u64,
                denial: DenialPolarity::ADMITTED,
                obj_refs: [PackedObjRef::default(); 8],
                ts_ns: 0, // invariant 3: no wall clock in the hash path
                activity_idx: 0,
                node_kind: OpKind::Atom as u8,
                pad: [0u8; 5],
                prior_hash: receipt.chain_hash,
            };
            receipt.chain(black_box(&frame));
            receipt.chain_hash
        });
}

// ---------------------------------------------------------------------------
// Verifier gate + Little's Law snapshot
// ---------------------------------------------------------------------------

/// Selecting/dispatching the verifier gates: the full 5-stage
/// `praxis_core::verify::run_pipeline` (format, chain integrity, continuity,
/// commitments, profile) over an 8-record lawful ledger.
#[divan::bench]
fn verify_gate_dispatch(bencher: divan::Bencher) {
    let records = chained_records(8);
    bencher.bench_local(|| run_pipeline(black_box(&records), "default"));
}

/// WIP/throughput/cycle-time snapshot (L = λ·W) over a 64-record in-memory
/// receipt ledger — `verify_ops::little_law_snapshot`.
#[divan::bench]
fn little_law_snapshot(bencher: divan::Bencher) {
    let records = chained_records(64);
    bencher.bench_local(|| {
        my_conforming_project::verify_ops::little_law_snapshot(black_box(&records))
            .expect("snapshot")
    });
}
