//! Family F06 -- "N3 Quarantine and Refinement" (atlas ticket V12-006).
//!
//! Survey verdict: **MIXED**. This module is a Wire-phase-1 pass over the survey's
//! `ALREADY_BUILT` / `REUSE_ADAPT` / `HAND_WRITE_REQUIRED` breakdown, not a from-scratch
//! implementation -- every type below is a re-export of real, independently-tested code in
//! `praxis_graphlaw::chatman::{router, abi}` (verified this session: `praxis-graphlaw` compiles
//! as a workspace member and is now a real `path` dependency of this crate, see `Cargo.toml`).
//! Per `.claude/rules/no-overclaiming.md`, everything under "What is ALIVE below" was verified
//! this session by a real command; everything under "What is still not wired" is disclosed as a
//! gap, not dressed up as done.
//!
//! # What is ALIVE below (re-exports + one composition function, this session)
//!
//! 1. **Routing-level gate** (`ALREADY_BUILT` per the survey) -- [`Dialect`], [`Route`],
//!    [`ProfileGates`], [`QueryShape`], [`RouteDecision`], [`DialectRouter`]. Verified wired to
//!    a real production entrypoint this session: `grep -n "self.router.decide"
//!    crates/praxis-graphlaw/src/chatman/engine.rs` -> `1215: let decision =
//!    self.router.decide(&shape)?;`, with `router: DialectRouter` held as an engine field
//!    (`engine.rs:545`, constructed `engine.rs:607`). `Dialect::N3.route() == Route::Cold` and
//!    `ProfileGates::DEFAULT_ENABLED_MASK` excludes the N3 bit are asserted directly in
//!    `router_test.rs` (`routes_map_hot_warm_cold`, `default_enabled_mask_excludes_n3`).
//! 2. **Execution-level quarantine** (`REUSE_ADAPT` per the survey) -- [`N3Ticks`],
//!    [`N3CostBound`], [`N3Builtin`], [`N3ActuationBuiltin`], [`N3ExecutionProfile`],
//!    [`N3Rule`], [`N3ExecutionReceipt`], [`N3Executor`]. Real, deterministic, unit-tested
//!    library code (22 `#[test] fn n3_*` functions in `router_test.rs`, verified this session via
//!    `grep -B1 "^fn n3_" ... | grep -c "#\[test\]"`; two more `n3_*`-prefixed items in that file
//!    are shared test helpers, not tests themselves), enforcing the builtin whitelist, saturating
//!    incremental cost bound,
//!    unconditional direct-actuation refusal, and BLAKE3 execution receipt. Verified this session
//!    that it has zero non-test callers in `praxis-graphlaw` itself (`grep -rn "N3Executor"
//!    crates/praxis-graphlaw/src/` outside `router.rs`/`router_test.rs` hits only doc-comment
//!    mentions in `abi.rs`).
//! 3. [`route_and_execute_n3`] -- a genuine (not decorative) composition function added in this
//!    module: it calls the real [`DialectRouter::decide`], refuses (via
//!    [`Refusal::UnsupportedDialect`]) if the least-expressive-permitted dialect for the shape is
//!    not actually N3, and only then calls the real [`N3Executor::run`]. This is a real new call
//!    path -- exercised by this module's own `#[cfg(test)]` tests, which are new tests written
//!    and run this session -- but it lives in `multifractal-workflow`, not inside
//!    `praxis-graphlaw::chatman::engine`'s own admission pipeline; see gap (a) below.
//!
//! # What is still not wired (disclosed gaps, not fixed by this pass)
//!
//! (a) **N3Executor is still not called from `ChatmanEngine`'s own production admission/execution
//!     path.** [`route_and_execute_n3`] below is a real, tested call path, but it is a new
//!     composition living in this crate, not a retrofit of `engine.rs`'s S1-S6 pipeline itself.
//!     The survey's "full family... reachable from a real production entrypoint" exit bar refers
//!     to `ChatmanEngine`'s own pipeline; that retrofit is out of this module's scope (it would
//!     mean editing `praxis-graphlaw::chatman::engine`, a different crate's core admission path)
//!     and is not claimed here.
//! (b) **The real N3 parser/reasoner is not bridged to the quarantine boundary.** Verified this
//!     session: `grep -rln "n3rule_parser" crates/praxis-graphlaw/src/` finds 4 files --
//!     `parser/n3rule_parser.rs` itself, `parser/mod.rs`'s re-export, and two other
//!     parser-internal files that reference it for unrelated internal reasons
//!     (`parser/n3.pest`'s grammar comments, `parser/n3_terms.rs`'s scope-stack imports) -- none
//!     of which route through this crate's quarantine boundary. `N3Rule` above is a
//!     caller-declared classification struct (rule id + declared builtins + declared cost), not
//!     a parsed N3 rule -- anyone calling `parser::n3rule_parser::parse` directly still bypasses
//!     every gate in this module. This module does not close that gap; it is real hand-engineering
//!     work, tracked under this ticket (V12-006), not attempted here.
//! (c) **No L7 concurrency/chaos-recovery semantics** (duplicate events, engine restart mid-N3-run,
//!     stale receipt detection) exist for N3 execution anywhere in this module or in the code it
//!     wraps. Not implemented; not claimed.
//!
//! # Survey-cited paths for F06
//! - /Users/sac/Downloads/v26.7.12_mermaid_atlas/families/F06_n3-quarantine.md
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/quarantine.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/router.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/router_test.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/abi.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/chatman/engine.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/parser/n3rule_parser.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/src/parser/mod.rs
//! - /Users/sac/praxis/crates/praxis-graphlaw/tests/chatman_acceptance_routing.rs
//! - /Users/sac/praxis/docs/standing/SEMANTIC_PROFILE_DOCTRINE.md
//! - /Users/sac/praxis/justfile

// ---- Routing-level gate: ALREADY_BUILT, re-exported (not reimplemented). ----
pub use praxis_graphlaw::chatman::router::{
    Dialect, DialectRouter, ProfileGates, QueryShape, Route, RouteDecision,
};

// ---- Execution-level quarantine: REUSE_ADAPT, re-exported (not reimplemented). ----
pub use praxis_graphlaw::chatman::router::{
    N3ActuationBuiltin, N3Builtin, N3CostBound, N3ExecutionProfile, N3ExecutionReceipt, N3Executor,
    N3Rule, N3Ticks,
};

// ---- Cross-cutting ABI types the two layers above are expressed in terms of. ----
pub use praxis_graphlaw::chatman::abi::{Digest, ProfileId, Refusal};

/// Routes `shape` under `router`, then -- only if the router's own
/// least-expressive-dialect law actually lands on [`Dialect::N3`] -- runs `rules`
/// through [`N3Executor`] under `execution`.
///
/// This is the family's quarantine boundary made concrete as one call: it refuses to run the
/// N3 execution surface at all unless the routing gate independently agrees N3 was the dialect
/// this shape required, closing the trivial bypass of calling [`N3Executor::run`] directly with
/// an [`N3ExecutionProfile`] for a shape that never needed N3 in the first place. It does not
/// parse N3 syntax and does not change [`N3Executor`]'s own enforcement (whitelist, cost bound,
/// direct-actuation refusal); both remain exactly as they are in `praxis-graphlaw`.
///
/// # Errors
/// - Any [`Refusal`] [`DialectRouter::decide`] returns for `shape` (including
///   [`Refusal::N3UnavailableByProfile`] / [`Refusal::N3ActuationRefused`] when the profile has
///   not explicitly enabled N3).
/// - [`Refusal::UnsupportedDialect`] if `shape`'s least-expressive permitted dialect is not
///   [`Dialect::N3`] (i.e. some cheaper dialect already suffices -- N3 execution is refused as
///   unnecessary, not merely undesired).
/// - Any [`Refusal`] [`N3Executor::run`] returns for `rules` (builtin whitelist, cost bound,
///   direct-actuation refusal -- see `router.rs`'s doc comments for each variant).
///
/// # Complexity
/// O(1) for the routing decision (fixed 6-variant scan) plus O(R * (A + B)) for execution, where
/// R = `rules.len()`, A/B are bounded by [`N3ActuationBuiltin::ALL`]/[`N3Builtin::ALL`]'s fixed
/// sizes per rule -- identical bounds to [`DialectRouter::decide`] and [`N3Executor::run`]
/// themselves, since this function adds only O(1) glue around them.
pub fn route_and_execute_n3(
    router: &DialectRouter,
    shape: &QueryShape,
    execution: &N3ExecutionProfile,
    rules: &[N3Rule],
) -> Result<(RouteDecision, N3ExecutionReceipt), Refusal> {
    let decision = router.decide(shape)?;
    if decision.dialect != Dialect::N3 {
        return Err(Refusal::UnsupportedDialect(format!(
            "F06 quarantine boundary (route_and_execute_n3): shape's least expressive \
             permitted dialect is {}, not N3; refusing to force N3 execution for a shape \
             that does not require it",
            decision.dialect.name()
        )));
    }
    let executor = N3Executor::new(router.gates(), execution);
    let receipt = executor.run(rules)?;
    Ok((decision, receipt))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n3_enabled_gates() -> ProfileGates {
        // N3 bit (Dialect::N3.mask_bit() == 1 << 5 == 0b0010_0000) explicitly OR'd on top of
        // the default mask, matching router_test.rs's own `n3_enabled_gates` helper -- N3 is
        // never on by accident, only by explicit enablement (invariant 2).
        ProfileGates::new(
            ProfileId::new("f06-test-profile"),
            ProfileGates::DEFAULT_ENABLED_MASK | Dialect::N3.mask_bit(),
            ProfileGates::DEFAULT_ENABLED_MASK & !Dialect::N3.mask_bit(),
            8,
        )
        .expect("valid gates: actuation mask is a subset of enabled mask and excludes N3")
    }

    fn n3_shape() -> QueryShape {
        QueryShape {
            constraint_count: 1,
            requires_construct: false,
            requires_owl: false,
            requires_n3_builtins: true,
            wants_actuation: false,
        }
    }

    #[test]
    fn route_and_execute_n3_admits_a_pure_rule_under_an_n3_enabled_profile() {
        let gates = n3_enabled_gates();
        let router = DialectRouter::new(gates);
        let execution = N3ExecutionProfile {
            builtin_whitelist_mask: N3Builtin::MathSum.mask_bit(),
            cost_bound_ticks: N3Ticks(100),
        };
        let rules = vec![N3Rule {
            rule_id: "rule-1".to_string(),
            builtins: vec![N3Builtin::MathSum],
            declared_cost: N3Ticks(10),
            direct_actuation_builtins: vec![],
        }];

        let (decision, receipt) = route_and_execute_n3(&router, &n3_shape(), &execution, &rules)
            .expect("N3-enabled profile, whitelisted builtin, within-budget cost");

        assert_eq!(decision.dialect, Dialect::N3);
        assert_eq!(decision.route, Route::Cold);
        assert_eq!(receipt.rules_admitted, vec!["rule-1".to_string()]);
        assert_eq!(receipt.ticks_used, N3Ticks(10));
    }

    #[test]
    fn route_and_execute_n3_refuses_when_profile_does_not_enable_n3() {
        // Default mask excludes N3 (invariant 2) -- decide() itself refuses before this
        // function's own dialect check ever runs.
        let gates = ProfileGates::new(
            ProfileId::new("f06-test-profile-no-n3"),
            ProfileGates::DEFAULT_ENABLED_MASK,
            0,
            8,
        )
        .expect("valid gates");
        let router = DialectRouter::new(gates);
        let execution = N3ExecutionProfile {
            builtin_whitelist_mask: N3Builtin::MathSum.mask_bit(),
            cost_bound_ticks: N3Ticks(100),
        };
        let rules = vec![N3Rule {
            rule_id: "rule-1".to_string(),
            builtins: vec![N3Builtin::MathSum],
            declared_cost: N3Ticks(10),
            direct_actuation_builtins: vec![],
        }];

        let err = route_and_execute_n3(&router, &n3_shape(), &execution, &rules)
            .expect_err("N3 not enabled by profile");
        assert!(matches!(err, Refusal::N3UnavailableByProfile(_)));
    }

    #[test]
    fn route_and_execute_n3_refuses_direct_actuation_unconditionally() {
        let gates = n3_enabled_gates();
        let router = DialectRouter::new(gates);
        let execution = N3ExecutionProfile {
            builtin_whitelist_mask: N3Builtin::ALL.iter().fold(0u8, |m, b| m | b.mask_bit()),
            cost_bound_ticks: N3Ticks(1_000),
        };
        let rules = vec![N3Rule {
            rule_id: "rule-actuates".to_string(),
            builtins: vec![],
            declared_cost: N3Ticks(1),
            direct_actuation_builtins: vec![N3ActuationBuiltin::OsProcess],
        }];

        let err = route_and_execute_n3(&router, &n3_shape(), &execution, &rules)
            .expect_err("rule declares a direct-actuation builtin");
        assert!(matches!(err, Refusal::N3DirectActuationRefused(_)));
    }

    #[test]
    fn route_and_execute_n3_refuses_cost_bound_exceeded_before_admitting_the_rule() {
        let gates = n3_enabled_gates();
        let router = DialectRouter::new(gates);
        let execution = N3ExecutionProfile {
            builtin_whitelist_mask: N3Builtin::MathSum.mask_bit(),
            cost_bound_ticks: N3Ticks(5),
        };
        let rules = vec![N3Rule {
            rule_id: "over-budget".to_string(),
            builtins: vec![N3Builtin::MathSum],
            declared_cost: N3Ticks(6),
            direct_actuation_builtins: vec![],
        }];

        let err = route_and_execute_n3(&router, &n3_shape(), &execution, &rules)
            .expect_err("declared cost exceeds the bound");
        assert!(matches!(err, Refusal::N3CostBoundExceeded(_)));
    }

    #[test]
    fn route_and_execute_n3_refuses_non_whitelisted_builtin() {
        let gates = n3_enabled_gates();
        let router = DialectRouter::new(gates);
        let execution = N3ExecutionProfile {
            builtin_whitelist_mask: N3Builtin::LogEqualTo.mask_bit(), // MathSum NOT whitelisted
            cost_bound_ticks: N3Ticks(100),
        };
        let rules = vec![N3Rule {
            rule_id: "uses-mathsum".to_string(),
            builtins: vec![N3Builtin::MathSum],
            declared_cost: N3Ticks(1),
            direct_actuation_builtins: vec![],
        }];

        let err = route_and_execute_n3(&router, &n3_shape(), &execution, &rules)
            .expect_err("MathSum outside the declared whitelist");
        assert!(matches!(err, Refusal::N3BuiltinRefused(_)));
    }

    #[test]
    fn route_and_execute_n3_refuses_forcing_n3_for_a_shape_that_does_not_need_it() {
        // A shape with no N3/OWL/CONSTRUCT requirement floors at Triple8Pattern, so the
        // router's own least-expressive law never lands on N3 -- route_and_execute_n3 must
        // refuse rather than silently downgrading or silently running N3Executor anyway.
        let gates = n3_enabled_gates();
        let router = DialectRouter::new(gates);
        let plain_shape = QueryShape {
            constraint_count: 1,
            requires_construct: false,
            requires_owl: false,
            requires_n3_builtins: false,
            wants_actuation: false,
        };
        let execution = N3ExecutionProfile {
            builtin_whitelist_mask: N3Builtin::MathSum.mask_bit(),
            cost_bound_ticks: N3Ticks(100),
        };
        let rules = vec![N3Rule {
            rule_id: "irrelevant".to_string(),
            builtins: vec![],
            declared_cost: N3Ticks(0),
            direct_actuation_builtins: vec![],
        }];

        let err = route_and_execute_n3(&router, &plain_shape, &execution, &rules)
            .expect_err("shape does not require N3; forcing N3 execution must be refused");
        assert!(matches!(err, Refusal::UnsupportedDialect(_)));
        assert!(err.to_string().contains("Triple8Pattern"));
    }

    #[test]
    fn route_and_execute_n3_receipt_is_deterministic_across_repeated_runs() {
        // Mirrors router_test.rs's own determinism-style coverage: same (gates, shape,
        // execution, rules) must produce byte-identical decision_hash/execution_hash across
        // repeated calls -- no wall clock, no randomness, computed not asserted.
        let gates = n3_enabled_gates();
        let router = DialectRouter::new(gates);
        let execution = N3ExecutionProfile {
            builtin_whitelist_mask: N3Builtin::MathSum.mask_bit(),
            cost_bound_ticks: N3Ticks(100),
        };
        let rules = vec![N3Rule {
            rule_id: "rule-1".to_string(),
            builtins: vec![N3Builtin::MathSum],
            declared_cost: N3Ticks(10),
            direct_actuation_builtins: vec![],
        }];

        let (decision_a, receipt_a) =
            route_and_execute_n3(&router, &n3_shape(), &execution, &rules).expect("first run");
        let (decision_b, receipt_b) =
            route_and_execute_n3(&router, &n3_shape(), &execution, &rules).expect("second run");

        assert_eq!(decision_a.decision_hash, decision_b.decision_hash);
        assert_eq!(receipt_a.execution_hash, receipt_b.execution_hash);
    }
}
