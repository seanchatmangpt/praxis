//! PROJ-709 — POWL composition: helper ∥ main as a NESTED
//! `Powl::PartialOrder` (no enum change).
//!
//! Each subworkflow is its own total-order `PartialOrder` (via the existing
//! `project_tape_to_powl`); the top level is one `PartialOrder` whose order
//! set contains ONLY the derived cross-workflow `mustPrecede` coupling
//! (absent pair = parallel). Serialization goes through the existing
//! `powl::powl_to_turtle` serializer, whose `emit_powl_node` recursion
//! already handles arbitrary nesting, and `shape::validate_powl_store`'s
//! depth-independent binding law (`bindings == nodes − 1`) accepts the
//! nested shape.
//!
//! SPIKE FINDINGS (downstream acceptance, read before building):
//! - `powl::powl_to_turtle` — nested OK (recursive emitter, deterministic
//!   child/binding IRIs at every depth).
//! - `shape::validate_powl_store` — nested OK (depth-independent law).
//! - `runner::validate_run` (`model_to_labels_and_edges`) — REFUSES nested
//!   children (`CNG_R05`, runner.rs:90-95): flat-linear only.
//! - `runner::linearize_hierarchical` — accepts two-level nesting but
//!   REQUIRES the root order to be the FULL transitive closure of a total
//!   phase order (runner.rs:342-354). A composed model whose root order is
//!   a genuine partial order (helper ∥ main, cross edges only) is NOT
//!   executable through the published bcinr-powl 26.6.25 adapter — by
//!   design: the dispatch side consumes the powl2 RDF graph
//!   (`decomp:DecompositionResult`), never the Rust enum, and executes the
//!   subworkflows on separate engines.

use std::collections::BTreeSet;

use bcinr_pddl::Pddl8Tape;

use crate::powl::{powl_to_turtle, project_tape_to_powl, CngRefusal, Powl};

use super::rules::DerivedEdges;

/// Composes the nested two-subworkflow model. Returns the model plus the
/// cross-workflow action-level `mustPrecede` edges (from-label, to-label)
/// that justify the root order.
///
/// # Errors
/// `CNG_R21 DecompositionInadmissible` when the derived cross-workflow
/// coupling is cyclic (edges in both directions at subworkflow granularity);
/// `CNG_R04` for empty tapes (via `project_tape_to_powl`).
///
/// # Complexity
/// O(h · m · log |edges|) cross-edge scan plus O(h² + m²) for the closed
/// intra-subworkflow orders.
pub fn compose_two(
    candidate_id: &str,
    helper: &Pddl8Tape,
    main: &Pddl8Tape,
    edges: &DerivedEdges,
) -> Result<(Powl, Vec<(String, String)>), CngRefusal> {
    let helper_model = project_tape_to_powl(helper)?;
    let main_model = project_tape_to_powl(main)?;

    let mut cross: Vec<(String, String)> = Vec::new();
    let mut helper_before_main = false;
    let mut main_before_helper = false;
    // O(h · m) pairs.
    for h in &helper.ops {
        for m in &main.ops {
            if edges
                .must_precede
                .contains(&(h.action.label.clone(), m.action.label.clone()))
            {
                cross.push((h.action.label.clone(), m.action.label.clone()));
                helper_before_main = true;
            }
            if edges
                .must_precede
                .contains(&(m.action.label.clone(), h.action.label.clone()))
            {
                cross.push((m.action.label.clone(), h.action.label.clone()));
                main_before_helper = true;
            }
        }
    }
    cross.sort();
    cross.dedup();

    let mut order: BTreeSet<(usize, usize)> = BTreeSet::new();
    match (helper_before_main, main_before_helper) {
        (true, true) => {
            return Err(CngRefusal::DecompositionInadmissible {
                candidate: candidate_id.to_string(),
                reason: "cyclic cross-workflow mustPrecede coupling".to_string(),
            });
        }
        (true, false) => {
            order.insert((0, 1));
        }
        (false, true) => {
            order.insert((1, 0));
        }
        (false, false) => {}
    }

    Ok((
        Powl::PartialOrder {
            children: vec![helper_model, main_model],
            order,
        },
        cross,
    ))
}

/// Serializes the composed model as powl2 RDF Turtle (the graph the
/// dispatch side consumes), root `<base>/n0` with `powl2:derivedFrom`.
///
/// # Complexity
/// O(nodes + |order|).
pub fn composed_to_turtle(model: &Powl, base_iri: &str, derived_from: &str) -> String {
    powl_to_turtle(model, base_iri, Some(derived_from))
}
