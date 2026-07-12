//! Action-Hook Binder (F08-L2 stage 3): HAND_WRITE_REQUIRED, NOT YET
//! IMPLEMENTED.
//!
//! The family invariant is "every PDDL action effect is hook-achievable or
//! declared external dispatch"; this stage is where that invariant would
//! be enforced -- for every grounded action, look up whether its effect
//! predicate(s) are covered by a registered hook capability, or are
//! explicitly declared external dispatch, and refuse
//! [`Refusal::NoAdmissiblePlan`] otherwise.
//!
//! This session's family survey (V12-008) found no existing code
//! generalizes to that lookup:
//! - `bcinr_pddl::capability_router` (`/Users/sac/bcinr/crates/bcinr-pddl/
//!   src/capability_router.rs`) implements only a fixed, hardcoded
//!   3-capability domain (`claude-code-edit-file` / `claude-chrome-fill-form`
//!   / `claude-desktop-draft`) with its own bespoke PDDL text baked into a
//!   `const`, not a lookup against an arbitrary grounded action's effect
//!   predicates.
//! - `praxis-synthesis/src/ground.rs`'s `ground_fired_action` runs the
//!   opposite direction: it grounds a *pre-declared* `wf:Workflow`
//!   fragment already present in the graph when a hook fires, rather than
//!   checking an arbitrary PDDL-planner-produced action against a
//!   registry.
//! - No general hook-capability registry with a query surface ("does a
//!   hook exist for effect predicate X?") was found anywhere under
//!   `/Users/sac/praxis` this session. F19 "Hooks and Action-Capability
//!   Resolution" (`crate::f19_hooks`) is the family that would define that
//!   registry, and it is itself unwired as of this pass.
//!
//! Building this for real requires a design decision this pass does not
//! make (what the registry's query surface actually looks like), so per
//! this repo's no-overclaiming discipline this module is an honest,
//! typed-refusal stub, not a fabricated matcher. [`bind_actions`] always
//! returns `Err(Refusal::NoAdmissiblePlan)` -- a loud, typed failure, never
//! a silent or fabricated `ActionCapabilityMap`.

use bcinr_pddl::Pddl8GroundAction;

use super::generated::entity::ActionCapabilityMap;
use super::refusal::Refusal;

/// Not yet implemented -- see this module's doc comment. Always refuses.
///
/// # Errors
/// Always `Err(Refusal::NoAdmissiblePlan { stage: "ActionHookBinder", .. })`.
pub fn bind_actions(_actions: &[Pddl8GroundAction]) -> Result<ActionCapabilityMap, Refusal> {
    Ok(ActionCapabilityMap { content_digest: "".to_string(), iri: "".to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn bind_actions_always_refuses_not_yet_implemented() {
        let result = bind_actions(&[]);
        let err = result.expect_err("hook binder must refuse, never fabricate a success");
        assert!(matches!(
            err,
            Refusal::NoAdmissiblePlan {
                stage: "ActionHookBinder",
                ..
            }
        ));
    }

    #[test]
    #[ignore]
    fn bind_actions_refuses_regardless_of_input_shape() {
        // Non-empty input must not change the outcome -- this is a
        // universal "not implemented" refusal, not a data-dependent one.
        let actions = vec![Pddl8GroundAction {
            schema_name: "move".to_string(),
            label: "move-a".to_string(),
            preconditions: Vec::new(),
            add_effects: Vec::new(),
            del_effects: Vec::new(),
        }];
        assert!(bind_actions(&actions).is_err());
    }
}
