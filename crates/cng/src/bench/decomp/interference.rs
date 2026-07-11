//! PROJ-708 — Non-interference proof + resource-release closure.
//!
//! Non-interference: for every helper action h and main action m with no
//! derived `mustPrecede` path between them in either direction (i.e. the
//! pair is concurrent under the composed partial order), the delete effects
//! of each must be disjoint from the other's preconditions (the protected
//! preconditions) — Effects ∩ ProtectedPreconditions = ∅ BOTH directions.
//! Violation is `CNG_R22 InterferenceDetected`.
//!
//! Release closure: every resource-classified atom that the helper leaves
//! true in s′ beyond the initial state must be consumed by some main-side
//! precondition — otherwise the helper holds custody it never released,
//! `CNG_R24 ResourceUnreleased`.

use std::collections::BTreeSet;

use bcinr_pddl::{Pddl8GroundAtom, Pddl8Tape};

use crate::powl::CngRefusal;

use super::rules::DerivedEdges;

/// Checks the non-interference proof obligation over all concurrent
/// (unordered) helper/main action pairs, both directions.
///
/// # Errors
/// `CNG_R22 InterferenceDetected` naming the first clobbering pair + atom.
///
/// # Complexity
/// O(h · m · c log |edges|) over h helper ops, m main ops, c ≤ 8 conjuncts.
pub fn check_interference(
    helper: &Pddl8Tape,
    main: &Pddl8Tape,
    edges: &DerivedEdges,
) -> Result<(), CngRefusal> {
    for h in &helper.ops {
        for m in &main.ops {
            let ordered = edges
                .must_precede
                .contains(&(h.action.label.clone(), m.action.label.clone()))
                || edges
                    .must_precede
                    .contains(&(m.action.label.clone(), h.action.label.clone()));
            if ordered {
                continue;
            }
            // helper deletes vs main protected preconditions.
            for del in &h.action.del_effects {
                if m.action.preconditions.contains(del) {
                    return Err(CngRefusal::InterferenceDetected {
                        helper_action: h.action.label.clone(),
                        main_action: m.action.label.clone(),
                        atom: del.label(),
                    });
                }
            }
            // main deletes vs helper protected preconditions.
            for del in &m.action.del_effects {
                if h.action.preconditions.contains(del) {
                    return Err(CngRefusal::InterferenceDetected {
                        helper_action: h.action.label.clone(),
                        main_action: m.action.label.clone(),
                        atom: del.label(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// The release-closure augmentation atoms: initial-state atoms classified
/// as resource atoms. Added to every helper goal so a lawful helper plan
/// must RESTORE the initial custody surface (release what it acquires) —
/// derivation over admitted facts, never a Rust constant list.
///
/// # Complexity
/// O(|init| log n).
pub fn augmentation_atoms(
    init: &BTreeSet<Pddl8GroundAtom>,
    edges: &DerivedEdges,
) -> Vec<Pddl8GroundAtom> {
    let mut out: Vec<Pddl8GroundAtom> = init
        .iter()
        .filter(|atom| edges.resource_atoms.contains(&atom.label()))
        .cloned()
        .collect();
    out.sort();
    out
}

/// Checks the resource-release closure over s′ and returns the discharged
/// release obligations (resource atoms the helper acquired and then
/// released) as evidence labels.
///
/// # Errors
/// `CNG_R24 ResourceUnreleased` for the first resource-classified atom that
/// is held in s′ beyond init without a consuming main precondition.
///
/// # Complexity
/// O(|s′| · m · c) over m main ops with c ≤ 8 preconditions each, plus
/// O(h · c log n) obligation collection over h helper ops.
pub fn check_release_closure(
    s_prime: &BTreeSet<Pddl8GroundAtom>,
    init: &BTreeSet<Pddl8GroundAtom>,
    helper: &Pddl8Tape,
    main: &Pddl8Tape,
    edges: &DerivedEdges,
) -> Result<Vec<String>, CngRefusal> {
    for atom in s_prime {
        if init.contains(atom) || !edges.resource_atoms.contains(&atom.label()) {
            continue;
        }
        let consumed = main
            .ops
            .iter()
            .any(|op| op.action.preconditions.contains(atom));
        if !consumed {
            return Err(CngRefusal::ResourceUnreleased {
                resource: atom.label(),
                holder: "helper".to_string(),
            });
        }
    }

    // Discharged obligations: resource atoms the helper added that do not
    // survive into s′ (acquired then released). O(h · c log n).
    let mut released: BTreeSet<String> = BTreeSet::new();
    for op in &helper.ops {
        for add in &op.action.add_effects {
            if edges.resource_atoms.contains(&add.label()) && !s_prime.contains(add) {
                released.insert(add.label());
            }
        }
    }
    Ok(released.into_iter().collect())
}
