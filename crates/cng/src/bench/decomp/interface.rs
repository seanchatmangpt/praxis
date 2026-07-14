//! PROJ-707 — Interface state s′ = E(s, π_h): replay the helper tape over
//! the grounded initial state with PER-STEP precondition verification.
//!
//! This is a clean-room reimplementation of BTreeSet STRIPS effect
//! application (state − del + add per step; semantics as proven in
//! `bcinr-pddl/src/ground.rs:375-414`), NOT a dependency on bcinr-pddl
//! internals (bcinr-pddl is not a cng dependency). The verification is the
//! s′ proof obligation: a tape that does not replay lawfully never yields
//! an interface state — `CNG_R23 InterfaceStateMismatch`, never trust in
//! the planner's BFS.

use std::collections::BTreeSet;

use bcinr_pddl::{Pddl8GroundAtom, Pddl8Tape};

use crate::powl::CngRefusal;

/// Replays `tape` from `init`, verifying every step's preconditions before
/// applying `state − del + add`. Returns the surviving atom set s′.
///
/// # Errors
/// `CNG_R23 InterfaceStateMismatch { step, atom }` naming the first tape
/// step whose precondition does not hold in the replayed state.
///
/// # Complexity
/// O(steps · c · log |state|) over c ≤ 8 conjuncts per step (STRIPS8 cap).
pub fn replay_to_interface_state(
    init: &BTreeSet<Pddl8GroundAtom>,
    tape: &Pddl8Tape,
) -> Result<BTreeSet<Pddl8GroundAtom>, CngRefusal> {
    let mut state = init.clone();
    for (step, op) in tape.ops.iter().enumerate() {
        for pre in &op.action.preconditions {
            if !state.contains(pre) {
                return Err(CngRefusal::InterfaceStateMismatch {
                    step,
                    atom: pre.label(),
                });
            }
        }
        for del in &op.action.del_effects {
            state.remove(del);
        }
        for add in &op.action.add_effects {
            state.insert(add.clone());
        }
    }
    Ok(state)
}
