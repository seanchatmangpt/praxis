//! `plan run` — the composed planner vertical slice (feature `ggen`).
//!
//! One call runs: graph facts (`pddl:` Turtle, the law-admitted planning
//! ontology) -> manufactured PDDL8 (`mfg::manufacture`) -> classical solve
//! (`ops::plan_solve_payload`, indexed/naive grounder auto-select) -> POWL
//! sequence compile (`bcinr_powl::compiler::compile_powl`, which enforces
//! acyclicity and all-ops-reachable) -> receipted execution
//! (`bcinr_powl::scheduler::scheduler_tick`, one
//! `bcinr_powl_receipt::causal_receipt::OcelCausalFrame` chained per fired
//! atom) -> artifact write (`domain.pddl`/`problem.pddl`/`plan.json`) behind
//! the `mfg::validate` shape-verifier gate -> a final ledger receipt
//! (`ops::receipt_issue_payload`).
//!
//! Determinism contract (invariant 3): no wall clock anywhere in the hash
//! path. Every frame carries `ts_ns = 0`, the run id is BLAKE3 of the source
//! graph hash, and the ledger receipt is issued with `ts_ns: 0` — two runs
//! over the same ontology produce byte-identical `powl_chain_hash` values
//! (pinned by `tests/plan_run_e2e.rs::two_runs_identical_chain_hashes`).
//!
//! Refusal convention: malformed input (unreadable path, bad TTL, PDDL that
//! fails to manufacture) is a hard `Err(String)`; domain infeasibility (the
//! solver's "no", or the verifier gate reporting unsolvable) is
//! `Ok(json)` with `"admitted": false` and a `refusal_reason` — matching the
//! rest of the CLI.

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::{OpKind, PowlTape};
use bcinr_powl_receipt::causal_receipt::{OcelCausalFrame, OcelCausalReceipt, PackedObjRef};
use bcinr_powl_receipt::denial::DenialPolarity;
use serde_json::{json, Value};

use crate::mfg;
use crate::ops;

/// Upper bound on scheduler ticks. A 64-slot sequence tape completes in at
/// most 64 ticks; anything past 128 is a livelock and refused by name.
const MAX_TICKS: u32 = 128;

/// Extract the ordered action names from a `plan solve` classical result.
///
/// `bcinr-pddl`'s `Pddl8Tape` serializes as `{"ops": [{"action":
/// {"schema_name": ...}, ...}]}` (both grounders return the same tape type
/// through `ops::plan_solve_payload`). A step without a string schema name
/// is a contract violation — hard `Err`.
fn plan_step_names(solved: &Value) -> Result<Vec<String>, String> {
    let steps = solved["plan"]["ops"]
        .as_array()
        .ok_or_else(|| "plan solve result has no `plan.ops` array".to_string())?;
    steps
        .iter()
        .enumerate()
        .map(|(i, s)| {
            s["action"]["schema_name"]
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("plan op {i} has no string `action.schema_name` field"))
        })
        .collect()
}

/// Compile an ordered plan into a POWL sequence tape.
///
/// A classical plan is a total order, so the workflow is `Sequence` of
/// `Atom`s: slot index == plan step index. The compiler's own post-passes
/// refuse cycles and unreachable ops.
pub fn compile_plan_to_powl(step_names: &[String]) -> Result<PowlTape, String> {
    if step_names.is_empty() {
        return Err("cannot compile an empty plan to POWL".to_string());
    }
    if step_names.len() > 64 {
        return Err(format!(
            "plan has {} steps; POWL tape holds at most 64 slots",
            step_names.len()
        ));
    }
    let atoms: Vec<PowlAstNode<'_>> = step_names
        .iter()
        .map(|n| PowlAstNode::Atom(n.as_str()))
        .collect();
    compile_powl(&PowlAstNode::Sequence(atoms)).map_err(|e| format!("POWL compile refused: {e:?}"))
}

/// Execute a compiled tape through the bcinr dispatcher, chaining one
/// genesis-folded `OcelCausalFrame` per fired `Atom` slot.
///
/// Returns `(fired_activity_names_in_order, canonical_chain_hash)`.
/// `run_id` binds the receipt chain to the source graph; `ts_ns` is always 0.
pub fn execute_receipted(
    tape: &PowlTape,
    step_names: &[String],
    run_id: [u8; 32],
) -> Result<(Vec<String>, String), String> {
    let mut state = PowlRunState::new(tape);
    let mut receipt = OcelCausalReceipt::genesis(run_id);
    let mut fired_names: Vec<String> = Vec::new();
    let ops_slice = &tape.ops[..tape.len as usize];

    for _ in 0..MAX_TICKS {
        if state.check_mask == 0 && state.active_mask == 0 {
            break;
        }
        let fired = scheduler_tick(ops_slice, &mut state);
        let mut mask = fired.0;
        while mask != 0 {
            let slot = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            if ops_slice[slot].kind != OpKind::Atom {
                continue;
            }
            let name = step_names
                .get(slot)
                .ok_or_else(|| format!("fired slot {slot} has no plan step name"))?;
            let frame = OcelCausalFrame {
                instruction_id: fired_names.len() as u64,
                fired_mask: 1u64 << slot,
                denial: DenialPolarity::ADMITTED,
                obj_refs: [PackedObjRef::default(); 8],
                ts_ns: 0, // invariant 3: no wall clock in the hash path
                activity_idx: slot as u16,
                node_kind: OpKind::Atom as u8,
                pad: [0u8; 5],
                prior_hash: receipt.chain_hash,
            };
            receipt.chain(&frame);
            fired_names.push(name.clone());
        }
    }

    if state.done_mask.count_ones() as u8 != tape.len {
        return Err(format!(
            "workflow livelock refused: {}/{} slots done after {MAX_TICKS} ticks",
            state.done_mask.count_ones(),
            tape.len
        ));
    }
    let canonical = receipt.canonical_hash();
    let hash = std::str::from_utf8(&canonical)
        .map_err(|e| format!("canonical hash is not UTF-8: {e}"))?
        .to_string();
    Ok((fired_names, hash))
}

/// The full vertical slice. See module docs for the stage list.
///
/// * `goal_ttl_path` — a `pddl:` Turtle ontology (domain + problem facts).
/// * `out_dir` — where the manufactured artifact is written.
/// * `receipts_dir` — the append-only JSONL receipt ledger directory.
pub fn plan_run_payload(
    goal_ttl_path: &str,
    out_dir: &str,
    receipts_dir: &str,
) -> Result<Value, String> {
    // (a) Graph facts: load + hash the law ontology.
    let ttl = ops::read_file(goal_ttl_path)?;
    let graph = mfg::load_graph(&ttl).map_err(|e| e.to_string())?;
    let graph_hash = mfg::graph_hash_hex(&graph).map_err(|e| e.to_string())?;

    // (a, cont.) Manufacture PDDL8 domain + problem from the graph.
    let manufactured = mfg::manufacture(&ttl, goal_ttl_path).map_err(|e| e.to_string())?;

    // (b) Bounded deterministic solve (classical; grounder auto-selected).
    let solve_payload = json!({
        "domain": manufactured.project_domain_text(),
        "problem": manufactured.project_problem_text(),
        "mode": "classical",
    })
    .to_string();
    let solved = ops::plan_solve_payload(&solve_payload)?;
    if solved["admitted"] != json!(true) {
        return Ok(json!({
            "admitted": false,
            "stage": "solve",
            "refusal_reason": solved["refusal_reason"],
            "graph_hash": graph_hash,
        }));
    }
    let step_names = plan_step_names(&solved)?;

    // (c) Compile the plan to a POWL sequence tape (acyclic, all reachable).
    let tape = compile_plan_to_powl(&step_names)?;

    // (d) Execute through the bcinr scheduler with per-step causal receipts.
    let run_id: [u8; 32] = *blake3::hash(graph_hash.as_bytes()).as_bytes();
    let (fired, powl_chain_hash) = execute_receipted(&tape, &step_names, run_id)?;
    if fired != step_names {
        return Err(format!(
            "execution order diverged from plan: fired {fired:?}, plan {step_names:?}"
        ));
    }

    // (e) Manufacture the artifact, gated by the shape verifier.
    let report = mfg::solve_ir(&manufactured);
    if !report.solvable {
        return Ok(json!({
            "admitted": false,
            "stage": "verify",
            "refusal_reason": report
                .error
                .unwrap_or_else(|| "manufactured PDDL failed the solvability gate".to_string()),
            "graph_hash": graph_hash,
        }));
    }
    std::fs::create_dir_all(out_dir).map_err(|e| format!("create {out_dir}: {e}"))?;
    let domain_path = format!("{out_dir}/domain.pddl");
    let problem_path = format!("{out_dir}/problem.pddl");
    let plan_path = format!("{out_dir}/plan.json");
    std::fs::write(&domain_path, &manufactured.project_domain_text())
        .map_err(|e| format!("write {domain_path}: {e}"))?;
    std::fs::write(&problem_path, &manufactured.project_problem_text())
        .map_err(|e| format!("write {problem_path}: {e}"))?;
    let plan_artifact = json!({
        "graph_hash": graph_hash,
        "plan": step_names,
        "powl_chain_hash": powl_chain_hash,
    });
    std::fs::write(&plan_path, format!("{plan_artifact:#}"))
        .map_err(|e| format!("write {plan_path}: {e}"))?;

    // (f) Fold the run into the existing receipt ledger (ts_ns pinned to 0).
    let receipt_payload = json!({
        "value": {
            "kind": "plan-run",
            "goal": goal_ttl_path,
            "graph_hash": graph_hash,
            "plan_len": step_names.len(),
            "powl_chain_hash": powl_chain_hash,
            "artifact_dir": out_dir,
        },
        "ts_ns": 0,
    })
    .to_string();
    let ledger_receipt = ops::receipt_issue_payload(&receipt_payload, receipts_dir)?;

    Ok(json!({
        "admitted": true,
        "goal": goal_ttl_path,
        "graph_hash": graph_hash,
        "solve": {
            "grounder": solved["grounder"],
            "plan_len": step_names.len(),
            "plan": step_names,
        },
        "powl": {
            "slots": tape.len,
            "entry_mask_hex": format!("{:016x}", tape.entry_mask),
        },
        "execution": {
            "fired": fired,
            "powl_chain_hash": powl_chain_hash,
        },
        "artifact": {
            "dir": out_dir,
            "files": ["domain.pddl", "problem.pddl", "plan.json"],
            "verifier": { "solvable": true, "plan_len": report.plan_len },
        },
        "ledger_receipt": ledger_receipt,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAWOBJECT_TTL: &str = include_str!("../ontology/lawobject.ttl");

    fn golden_steps() -> Vec<String> {
        [
            "supply-evidence",
            "clear-obligations",
            "judge",
            "admit",
            "receipt",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    #[test]
    fn compile_rejects_empty_plan() {
        assert!(compile_plan_to_powl(&[]).is_err());
    }

    #[test]
    fn compile_sequence_all_slots_reachable() {
        // compile_powl's post-passes refuse cycles/unreachable ops; Ok here
        // is the reachability proof for the golden plan's workflow.
        let tape = compile_plan_to_powl(&golden_steps()).expect("golden plan compiles");
        assert_eq!(tape.len, 5);
        assert_eq!(tape.entry_mask, 1, "sequence enters at slot 0");
    }

    #[test]
    fn execute_fires_plan_order_and_is_deterministic() {
        let steps = golden_steps();
        let tape = compile_plan_to_powl(&steps).expect("compiles");
        let run_id = [7u8; 32];
        let (fired_a, hash_a) = execute_receipted(&tape, &steps, run_id).expect("run a");
        let (fired_b, hash_b) = execute_receipted(&tape, &steps, run_id).expect("run b");
        assert_eq!(fired_a, steps);
        assert_eq!(fired_a, fired_b);
        assert_eq!(hash_a, hash_b, "no wall clock in the frame hash path");
        assert!(hash_a.starts_with("blake3:"));
    }

    #[test]
    fn plan_legality_golden_ontology() {
        // Legality: the manufactured problem grounds and every solved step
        // is applicable in order (bcinr-pddl's find_plan only emits legal
        // tapes; mfg::validate re-grounds and re-solves as the check).
        let manufactured =
            mfg::manufacture(LAWOBJECT_TTL, "ontology/lawobject.ttl").expect("manufactures");
        let report = mfg::solve_ir(&manufactured);
        assert!(
            report.parsed && report.solvable,
            "error: {:?}",
            report.error
        );
        assert_eq!(report.plan_steps, golden_steps());
    }
}
