//! # pddl-index — dictionary-encoded lazy grounding for PDDL8 (the qlever treatment)
//!
//! A drop-in alternative to `bcinr_pddl`'s naive grounder that treats action
//! grounding as a **relational join over a compact integer ID space**, so a
//! domain whose ground actions are dominated by dead (never-firing) instances
//! materializes only the reachable ones.
//!
//! The pipeline mirrors a triple store / datalog evaluator:
//!
//! | stage | module | role |
//! |-------|--------|------|
//! | dictionary encoding | [`dict`] | intern predicate/object/type strings → dense `u32` IDs |
//! | sorted-ID storage   | [`facts`] | per-predicate sorted argument-tuple relations (permutation-friendly) |
//! | membership pruning  | [`xorf`] | XOR filter (ported from bytestar `tables.h`), no false negatives |
//! | grounding-as-join   | [`ground`] | relaxed-reachability fixpoint + join-driven materialization |
//!
//! ## Correctness
//!
//! The grounder emits exactly the naive grounder's action list with the
//! never-firing entries removed, in the same order, so BFS forward search
//! ([`IndexedGroundProblem::find_plan`]) returns the *identical* plan. This is
//! checked differentially against `bcinr_pddl::GroundProblem::find_plan` on the
//! shared corpus in praxis's test suite.
//!
//! ## When to use it
//!
//! [`candidate_estimate`] gives an O(schemas) upper bound on naive
//! materialization; a caller can auto-select the indexed path above a threshold
//! and fall back to the small-domain BFS below it — see [`GROUND_INDEX_THRESHOLD`].

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod dict;
pub mod facts;
pub mod ground;
pub mod xorf;

pub use dict::{Dict, SymId};
pub use facts::FactStore;
pub use ground::{GroundError, GroundStats, IndexedGroundProblem};
pub use xorf::XorFilter;

use wasm4pm_compat::pddl::{Pddl8Domain, Pddl8Problem, Pddl8Tape};

/// Default auto-select cutoff: if the naive grounder would materialize more than
/// this many candidate actions, the indexed path is worth its fixed setup cost.
/// Below it, the difference is noise and the simpler BFS grounder is preferred.
pub const GROUND_INDEX_THRESHOLD: usize = 256;

/// An O(number of schemas) upper bound on how many ground actions the naive
/// grounder would materialize — the product of per-parameter type-compatible
/// object counts, summed over schemas. Cheap enough to compute before deciding
/// which grounder to run.
#[must_use]
pub fn candidate_estimate(domain: &Pddl8Domain, problem: &Pddl8Problem) -> usize {
    let object_type: std::collections::HashMap<&str, &str> = problem
        .object_types
        .iter()
        .map(|(o, t)| (o.as_str(), t.as_str()))
        .collect();
    let parent: std::collections::HashMap<&str, &str> = domain
        .types
        .iter()
        .filter_map(|t| t.parent.as_deref().map(|p| (t.name.as_str(), p)))
        .collect();
    let satisfies = |obj: &str, required: &str| -> bool {
        if required == "object" {
            return true;
        }
        let mut cur = *object_type.get(obj).unwrap_or(&"object");
        loop {
            if cur == required {
                return true;
            }
            match parent.get(cur) {
                Some(p) => cur = p,
                None => return false,
            }
        }
    };

    let mut total: usize = 0;
    for schema in &domain.actions {
        let typed: std::collections::HashMap<&str, &str> = schema
            .typed_params
            .iter()
            .map(|(p, t)| (p.as_str(), t.as_str()))
            .collect();
        let mut prod: usize = 1;
        for p in &schema.params {
            let required = typed.get(p.as_str()).copied().unwrap_or("object");
            let count = problem
                .objects
                .iter()
                .filter(|o| satisfies(o, required))
                .count();
            prod = prod.saturating_mul(count);
        }
        total = total.saturating_add(prod);
    }
    total
}

/// Whether the indexed grounder should be preferred for this problem, per
/// [`GROUND_INDEX_THRESHOLD`].
#[must_use]
pub fn should_use_indexed(domain: &Pddl8Domain, problem: &Pddl8Problem) -> bool {
    candidate_estimate(domain, problem) > GROUND_INDEX_THRESHOLD
}

/// Convenience: build the indexed grounding and run BFS, returning the plan and
/// the grounding statistics. Grounding *infeasibility* (empty grounding, no
/// plan) is an `Err(GroundError)` the caller classifies exactly as it would the
/// naive path's `Pddl8Error`.
pub fn solve_indexed(
    domain: &Pddl8Domain,
    problem: &Pddl8Problem,
) -> Result<(Pddl8Tape, GroundStats), GroundError> {
    let gp = IndexedGroundProblem::build(domain, problem, None)?;
    let stats = gp.stats();
    let tape = gp.find_plan()?;
    Ok((tape, stats))
}
