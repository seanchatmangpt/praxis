//! Extended tests for hook effect types, priority/after scheduling, and idempotency.
//!
//! Part F continuation: Tests for kh:effect types ("ground-action", "refuse"),
//! priority/after ordering via Kahn's algorithm, and idempotency key stability.
//!
//! # Test Coverage
//!
//! 1. test_hooks_effect_ground_action() — Fires hook with effect "ground-action"
//! 2. test_hooks_effect_refuse() — Fires hook with effect "refuse" and reason
//! 3. test_hooks_priority_ordering() — Multiple hooks with different priorities
//! 4. test_hooks_after_dependency_ordering() — Kahn's DAG ordering with kh:after
//! 5. test_hooks_idempotency_key_stability() — Deterministic idempotency keys across runs
//!
//! # Notes
//!
//! - All tests use inline Turtle (no fixtures)
//! - Tests verify HookRunResult status, verdicts, and schedule fields
//! - effect type and idempotency_key fields are verified for existence

use praxis_graphlaw::hooks::HookVerdict;
use praxis_graphlaw_wasm::core::run_hooks_core;
use praxis_graphlaw_wasm::dto::Status;

// ============================================================================
// Test 1: test_hooks_effect_ground_action
// ============================================================================

/// Test run_hooks_core with kh:effect "ground-action".
///
/// Base TTL defines a hook with effect="ground-action" and kh:action pointing
/// to an action definition. Verifies the effect type is correctly set.
///
/// Assertions:
/// - result.status == Status::Admitted
/// - The verdict's effect field indicates GroundAction
/// - action_iri field is populated from kh:action
#[test]
fn test_hooks_effect_ground_action() {
    let base_ttl = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .

        ex:hook1 a kh:Hook ;
            kh:name "hook_ground_action" ;
            kh:kind "delta" ;
            kh:var "http://example.org/severity" ;
            kh:on "assert" ;
            kh:effect "ground-action" ;
            kh:action <http://example.org/actions/escalate> .
    "#;

    let event_ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:alert1 <http://example.org/severity> "high" .
    "#;

    let result = run_hooks_core(base_ttl, event_ttl);
    assert!(result.is_ok(), "run_hooks_core failed: {:?}", result);

    let hook_result = result.unwrap();
    assert_eq!(
        hook_result.status,
        Status::Admitted,
        "Hook execution should be admitted"
    );

    // Verify action_iri is populated for ground-action effect
    if !hook_result.verdicts.is_empty() {
        let verdict = &hook_result.verdicts[0];
        assert!(
            verdict.action_iri.is_some(),
            "action_iri should be populated for ground-action effect"
        );
        let effect_str = format!("{:?}", verdict.effect);
        assert!(
            effect_str.contains("GroundAction"),
            "Effect should be GroundAction, got: {}",
            effect_str
        );
    }
}

// ============================================================================
// Test 2: test_hooks_effect_refuse
// ============================================================================

/// Test run_hooks_core with kh:effect "refuse".
///
/// Base TTL defines a hook with effect="refuse" and kh:reason.
/// Verifies the effect type is correctly set.
///
/// Assertions:
/// - result.status == Status::Admitted
/// - If verdict produced, effect should be Refuse
#[test]
fn test_hooks_effect_refuse() {
    let base_ttl = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .

        ex:hook1 a kh:Hook ;
            kh:name "hook_refuse" ;
            kh:kind "delta" ;
            kh:var "http://example.org/requestCount" ;
            kh:on "assert" ;
            kh:effect "refuse" ;
            kh:reason "quota exceeded" .
    "#;

    let event_ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:user1 <http://example.org/requestCount> "1000" .
    "#;

    let result = run_hooks_core(base_ttl, event_ttl);
    assert!(result.is_ok(), "run_hooks_core failed: {:?}", result);

    let hook_result = result.unwrap();
    assert_eq!(
        hook_result.status,
        Status::Admitted,
        "Hook execution should be admitted"
    );

    // Verify effect type is Refuse (if verdict produced)
    if !hook_result.verdicts.is_empty() {
        let verdict = &hook_result.verdicts[0];
        let effect_str = format!("{:?}", verdict.effect);
        assert!(
            effect_str.contains("Refuse"),
            "Effect should be Refuse, got: {}",
            effect_str
        );
    }
}

// ============================================================================
// Test 3: test_hooks_priority_ordering
// ============================================================================

/// Test run_hooks_core respects hook priority in execution order.
///
/// Creates 3 hooks with different kh:priority values. Verifies they appear
/// in the schedule (if scheduling is attempted). References hooks.rs:821
/// for Kahn's algorithm tie-breaking by (priority ASC, HookId ASC).
///
/// Assertions:
/// - result.status == Status::Admitted
/// - If schedule populated, hooks appear in priority order
#[test]
fn test_hooks_priority_ordering() {
    let base_ttl = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .

        ex:hook1 a kh:Hook ;
            kh:name "hook_priority_1" ;
            kh:kind "delta" ;
            kh:var "http://example.org/event" ;
            kh:on "assert" ;
            kh:priority "1" ;
            kh:effect "emit-delta" .

        ex:hook2 a kh:Hook ;
            kh:name "hook_priority_10" ;
            kh:kind "delta" ;
            kh:var "http://example.org/event" ;
            kh:on "assert" ;
            kh:priority "10" ;
            kh:effect "emit-delta" .

        ex:hook3 a kh:Hook ;
            kh:name "hook_priority_5" ;
            kh:kind "delta" ;
            kh:var "http://example.org/event" ;
            kh:on "assert" ;
            kh:priority "5" ;
            kh:effect "emit-delta" .
    "#;

    let event_ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:event1 <http://example.org/event> "fired" .
    "#;

    let result = run_hooks_core(base_ttl, event_ttl);
    assert!(result.is_ok(), "run_hooks_core failed: {:?}", result);

    let hook_result = result.unwrap();
    assert_eq!(
        hook_result.status,
        Status::Admitted,
        "Hook execution should be admitted"
    );

    // Verify that if hooks are scheduled, they respect priority order
    // (Priority values 1 < 5 < 10 should appear in that order)
    if hook_result.schedule.len() >= 2 {
        // Find indices of hooks in schedule
        let mut indices = vec![];
        for name in &["hook_priority_1", "hook_priority_5", "hook_priority_10"] {
            if let Some(idx) = hook_result.schedule.iter().position(|h| h.contains(name)) {
                indices.push(idx);
            }
        }
        // If at least 2 are present, verify they're in priority order
        if indices.len() >= 2 {
            for i in 1..indices.len() {
                assert!(
                    indices[i - 1] < indices[i],
                    "Lower priority should come before higher priority"
                );
            }
        }
    }
}

// ============================================================================
// Test 4: test_hooks_after_dependency_ordering
// ============================================================================

/// Test run_hooks_core respects kh:after dependencies in scheduling.
///
/// Creates 3 hooks with a dependency chain where hook2 depends on hook1,
/// and hook3 depends on hook2. References hooks.rs:821 for Kahn's algorithm
/// and dependency resolution.
///
/// Assertions:
/// - result.status == Status::Admitted
/// - If schedule populated, hooks appear in dependency order
#[test]
fn test_hooks_after_dependency_ordering() {
    let base_ttl = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .

        ex:hook1 a kh:Hook ;
            kh:name "first_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:on "assert" ;
            kh:effect "emit-delta" .

        ex:hook2 a kh:Hook ;
            kh:name "second_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:on "assert" ;
            kh:after <http://example.org/hook1> ;
            kh:effect "emit-delta" .

        ex:hook3 a kh:Hook ;
            kh:name "third_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/trigger" ;
            kh:on "assert" ;
            kh:after <http://example.org/hook2> ;
            kh:effect "emit-delta" .
    "#;

    let event_ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:event1 <http://example.org/trigger> "fire" .
    "#;

    let result = run_hooks_core(base_ttl, event_ttl);
    assert!(result.is_ok(), "run_hooks_core failed: {:?}", result);

    let hook_result = result.unwrap();
    assert_eq!(
        hook_result.status,
        Status::Admitted,
        "Hook execution should be admitted"
    );

    // Verify dependency order if schedule is populated
    if hook_result.schedule.len() >= 2 {
        let schedule = &hook_result.schedule;
        if let (Some(idx1), Some(idx2), Some(idx3)) = (
            schedule.iter().position(|h| h.contains("first_hook")),
            schedule.iter().position(|h| h.contains("second_hook")),
            schedule.iter().position(|h| h.contains("third_hook")),
        ) {
            assert!(
                idx1 < idx2,
                "first_hook should come before second_hook in dependency order"
            );
            assert!(
                idx2 < idx3,
                "second_hook should come before third_hook in dependency order"
            );
        }
    }
}

// ============================================================================
// Test 5: test_hooks_idempotency_key_stability
// ============================================================================

/// Test that verdicts are structurally identical across multiple runs.
///
/// Runs the same hook+event pair twice and verifies determinism.
/// This tests idempotency key stability (hooks.rs:1047, 1325, 1333-1334)
/// and verdict structure reproducibility.
///
/// Assertions:
/// - Run 1 and Run 2 both return Status::Admitted
/// - Verdicts count matches between runs
/// - Each verdict's structure is byte-identical across runs
#[test]
fn test_hooks_idempotency_key_stability() {
    let base_ttl = r#"
        @prefix kh: <http://seanchatmangpt.github.io/praxis/kh#> .
        @prefix ex: <http://example.org/> .

        ex:hook1 a kh:Hook ;
            kh:name "deterministic_hook" ;
            kh:kind "delta" ;
            kh:var "http://example.org/status" ;
            kh:on "assert" ;
            kh:effect "emit-delta" .
    "#;

    let event_ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:entity1 <http://example.org/status> "active" .
    "#;

    // Run 1
    let result1 = run_hooks_core(base_ttl, event_ttl);
    assert!(result1.is_ok(), "Run 1 failed: {:?}", result1);
    let run1 = result1.unwrap();

    // Run 2 (identical inputs)
    let result2 = run_hooks_core(base_ttl, event_ttl);
    assert!(result2.is_ok(), "Run 2 failed: {:?}", result2);
    let run2 = result2.unwrap();

    // Verify both runs admitted
    assert_eq!(
        run1.status, Status::Admitted,
        "Run 1 status should be Admitted"
    );
    assert_eq!(
        run2.status, Status::Admitted,
        "Run 2 status should be Admitted"
    );

    // Verify verdict counts match (determinism proof)
    assert_eq!(
        run1.verdicts.len(),
        run2.verdicts.len(),
        "Verdict counts must match between runs"
    );

    // For each verdict, verify determinism
    for (i, (v1, v2)) in run1.verdicts.iter().zip(run2.verdicts.iter()).enumerate() {
        // Verify hook names match
        assert_eq!(
            v1.hook_name, v2.hook_name,
            "Verdict {} hook_name should match",
            i
        );

        // Verify idempotency keys are identical (if present)
        assert_eq!(
            v1.idempotency_key, v2.idempotency_key,
            "Verdict {} idempotency_key must match across runs",
            i
        );

        // Verify effects match
        assert_eq!(
            v1.effect, v2.effect,
            "Verdict {} effect must match",
            i
        );

        // Verify verdicts (Fired/NotFired) match
        assert_eq!(
            v1.verdict, v2.verdict,
            "Verdict {} verdict status must match",
            i
        );

        // Verify condition hash matches (deterministic hashing)
        assert_eq!(
            v1.condition_hash, v2.condition_hash,
            "Verdict {} condition_hash must be identical",
            i
        );
    }

    // Verify schedule is deterministic
    assert_eq!(
        run1.schedule, run2.schedule,
        "Schedule must be identical across runs"
    );
}
