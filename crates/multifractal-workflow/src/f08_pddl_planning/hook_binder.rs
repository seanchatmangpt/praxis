//! Action-Hook Binder (F08-L2 stage 3): **wired to `crate::f19_hooks`, real**.
//!
//! The family invariant is "every PDDL action effect is hook-achievable or
//! declared external dispatch"; this stage enforces it -- for every
//! grounded action, [`bind_actions`] calls
//! [`crate::f19_hooks::resolve_hook_for_action`] (F19's real Capability
//! Matcher: catalog parse + SHACL validation + Kahn scheduling + exact
//! `kh:action` IRI match, all real, not a keyword sweep) against the
//! admitted `hook_pack_turtle` catalog, and refuses
//! [`Refusal::NoAdmissiblePlan`] the moment any action fails to bind --
//! never a fabricated success.
//!
//! # History (disclosed, not silently corrected)
//! This module previously stated "no general hook-capability registry ...
//! was found anywhere ... F19 ... is itself unwired as of this pass" and
//! `bind_actions` always refused. That was true when written; F19 has
//! since been wired for real (`crate::f19_hooks::resolve_hook_for_action`,
//! exercised by that module's own test suite). Separately, this file's
//! `bind_actions` body was found corrupted -- a swept-in scratch script
//! (`patch_placeholders.py`) had mechanically rewritten the honest
//! `Err(Refusal::NoAdmissiblePlan)` body into a fabricated
//! `Ok(ActionCapabilityMap{content_digest:"",iri:""})` without touching
//! this doc comment or the (still-`#[ignore]`d) tests that assumed `Err`.
//! Fixed forward in the same pass that wired the real F19 integration.
//!
//! # Errors
//! [`Refusal::NoAdmissiblePlan`] if `hook_pack_turtle` fails to parse/
//! validate/schedule, or if any action has zero or more than one
//! candidate hook -- see [`crate::f19_hooks::HookResolutionRefused`] for
//! the exact sub-cases, folded here into F08's single named refusal type
//! per the atlas invariant, with the F19 detail preserved in `reason`.

use bcinr_pddl::Pddl8GroundAction;
use wasm4pm_compat::hash::blake3_combined;

use super::generated::entity::ActionCapabilityMap;
use super::refusal::Refusal;
use crate::f19_hooks::{InMemoryReceiptLedger, ReceiptLedger};

/// Binds every grounded action to exactly one registered hook capability
/// via F19's real Capability Matcher, or refuses.
///
/// A fresh [`InMemoryReceiptLedger`] is used for the duration of one call
/// (L7 durable-idempotency-across-restarts is F19's own disclosed gap, not
/// re-solved here); a caller needing idempotency across multiple
/// `bind_actions` calls must supply and reuse a ledger via
/// [`bind_actions_with_ledger`].
///
/// # Errors
/// See the module doc comment.
pub fn bind_actions(
    actions: &[Pddl8GroundAction],
    hook_pack_turtle: &str,
) -> Result<ActionCapabilityMap, Refusal> {
    let mut ledger = InMemoryReceiptLedger::default();
    bind_actions_with_ledger(actions, hook_pack_turtle, &mut ledger)
}

/// As [`bind_actions`], but with a caller-supplied [`ReceiptLedger`] so
/// idempotency state can be carried across multiple calls (e.g. repeated
/// `run_pipeline` invocations within one process).
///
/// # Errors
/// [`Refusal::NoAdmissiblePlan { stage: "ActionHookBinder", .. }`] wrapping
/// the first action's F19 [`crate::f19_hooks::HookResolutionRefused`], or
/// wrapping F19's own idempotency-gate refusal on a replayed action.
pub fn bind_actions_with_ledger(
    actions: &[Pddl8GroundAction],
    hook_pack_turtle: &str,
    ledger: &mut dyn ReceiptLedger,
) -> Result<ActionCapabilityMap, Refusal> {
    let mut receipt_hashes = Vec::with_capacity(actions.len());
    for action in actions {
        let resolution =
            crate::f19_hooks::resolve_hook_for_action(hook_pack_turtle, action, ledger).map_err(
                |e| Refusal::NoAdmissiblePlan {
                    stage: "ActionHookBinder",
                    reason: format!(
                        "no capability binding for action '{}': {e}",
                        action.schema_name
                    ),
                },
            )?;
        receipt_hashes.push(resolution.receipt_hash);
    }
    // Canonical content: action count, then each binding's own real F19
    // receipt hash in the actions' own (caller-determined, already
    // deterministic) order -- no HashMap, no re-sorting needed since
    // `actions` is itself an ordered plan-derived slice.
    let mut parts: Vec<&str> = vec!["mfw:f08:action-capability-map:v1"];
    let count = actions.len().to_string();
    parts.push(&count);
    for h in &receipt_hashes {
        parts.push(h);
    }
    let content_digest = blake3_combined(&parts);
    Ok(ActionCapabilityMap {
        iri: format!("urn:mfw:f08:action_capability_map:{content_digest}"),
        content_digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NO_HOOKS_PACK: &str = "";

    const MOVE_HOOK_PACK: &str = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/f08#> .
        ex:hook-move a kh:Hook ;
          kh:name "move-hook" ;
          kh:kind "delta" ;
          kh:var "http://example.org/f08#actuates-move" ;
          kh:on "assert" ;
          kh:effect "ground-action" ;
          kh:action <urn:pddl:action:move> ;
          kh:reason "f08-test-authority-move" ;
          kh:priority 1 .
    "#;

    fn move_action() -> Pddl8GroundAction {
        Pddl8GroundAction {
            schema_name: "move".to_string(),
            label: "move-a".to_string(),
            preconditions: Vec::new(),
            add_effects: Vec::new(),
            del_effects: Vec::new(),
        }
    }

    #[test]
    fn bind_actions_refuses_when_no_hook_pack_is_admitted() {
        let result = bind_actions(&[move_action()], NO_HOOKS_PACK);
        let err = result.expect_err("empty catalog must refuse, never fabricate a binding");
        assert!(matches!(
            err,
            Refusal::NoAdmissiblePlan {
                stage: "ActionHookBinder",
                ..
            }
        ));
    }

    #[test]
    fn bind_actions_succeeds_for_a_real_registered_capability() {
        let map = bind_actions(&[move_action()], MOVE_HOOK_PACK)
            .expect("move is a real registered capability in MOVE_HOOK_PACK");
        assert!(!map.content_digest.is_empty());
        assert_eq!(map.content_digest.len(), 64, "blake3 hex digest");
        assert!(map.iri.starts_with("urn:mfw:f08:action_capability_map:"));
    }

    #[test]
    fn bind_actions_is_deterministic_across_independent_ledgers() {
        let a = bind_actions(&[move_action()], MOVE_HOOK_PACK).expect("first call binds");
        let b = bind_actions(&[move_action()], MOVE_HOOK_PACK).expect("second, fresh call binds");
        assert_eq!(
            a.content_digest, b.content_digest,
            "same actions + same catalog must yield the same digest across independent \
             InMemoryReceiptLedgers"
        );
    }

    #[test]
    fn bind_actions_with_ledger_refuses_a_replayed_action_via_f19s_idempotency_gate() {
        let mut ledger = InMemoryReceiptLedger::default();
        let first = bind_actions_with_ledger(&[move_action()], MOVE_HOOK_PACK, &mut ledger);
        assert!(first.is_ok());
        let second = bind_actions_with_ledger(&[move_action()], MOVE_HOOK_PACK, &mut ledger)
            .expect_err("replaying the same action against the same ledger must refuse");
        assert!(matches!(
            second,
            Refusal::NoAdmissiblePlan {
                stage: "ActionHookBinder",
                ..
            }
        ));
    }

    #[test]
    fn bind_actions_empty_action_list_succeeds_vacuously() {
        // No actions to bind is not the same claim as "everything bound" --
        // it is the honest empty case (an F08 pipeline with a zero-action
        // grounded plan, e.g. an already-satisfied goal).
        let map = bind_actions(&[], NO_HOOKS_PACK).expect("vacuous binding over zero actions");
        assert!(!map.content_digest.is_empty());
    }
}
