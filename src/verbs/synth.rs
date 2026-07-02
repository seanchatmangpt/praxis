//! `synth` verb dispatcher — the declare-and-derive surface.
//!
//! `synth run --payload '<synth/v1 json>'` runs the full pipeline
//! (saturate → sequence → content-addressed DAG → refinement admission) and
//! returns the `SynthesisReceipt`; `synth solve` stops at the discovered
//! plan. Refusals — including Solver8's certified unsat proofs with named
//! culprits — are success-shaped results (`status: "refused"`), never
//! process errors: domain denials are data.
//!
//! Wire format: one page, `docs/SYNTH_V1.md`. Thin wrappers over
//! [`my_conforming_project::synth_ops`], the single implementation shared
//! with the MCP membrane.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use serde_json::Value;

/// Run the full synthesis pipeline on a `synth/v1` payload: facts + rules
/// saturate, capabilities + goal are sequenced by the chosen solver
/// (`solver8` default | `brute`), the plan executes as a content-addressed
/// DAG, and refinements admit or refuse the whole run.
#[verb]
pub fn run(payload: String) -> Result<Value> {
    my_conforming_project::synth_ops::synth_run_payload(&payload)
        .map_err(NounVerbError::execution_error)
}

/// Saturate and sequence only: returns the discovered plan (order and
/// parameter bindings) with its solve receipt, or the refusal — which under
/// Solver8 may carry a minimal unsat core a second agent can verify without
/// searching.
#[verb]
pub fn solve(payload: String) -> Result<Value> {
    my_conforming_project::synth_ops::synth_solve_payload(&payload)
        .map_err(NounVerbError::execution_error)
}
