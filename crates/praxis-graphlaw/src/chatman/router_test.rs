#![cfg(test)]

//! Chicago-style (state-based, real collaborators, Arrange-Act-Assert)
//! in-module tests: full N3 gate matrix, the LER ordering property over
//! all dialect pairs, and decision-hash determinism.

use super::*;

fn profile(enabled: u8, actuation: u8, max_hot: u8) -> ProfileGates {
    match ProfileGates::new(ProfileId::new("test-profile"), enabled, actuation, max_hot) {
        Ok(g) => g,
        Err(refusal) => unreachable!("test profile must validate: {refusal}"),
    }
}

fn shape(
    constraint_count: u8,
    requires_construct: bool,
    requires_owl: bool,
    requires_n3_builtins: bool,
    wants_actuation: bool,
) -> QueryShape {
    QueryShape {
        constraint_count,
        requires_construct,
        requires_owl,
        requires_n3_builtins,
        wants_actuation,
    }
}

fn ok_or_unreachable(result: Result<RouteDecision, Refusal>) -> RouteDecision {
    match result {
        Ok(d) => d,
        Err(refusal) => unreachable!("expected Ok decision: {refusal}"),
    }
}

const ALL_ENABLED: u8 = ProfileGates::DEFAULT_ENABLED_MASK | 1 << 5;

// ---- Dialect order / route / mask laws --------------------------------

#[test]
fn dialect_order_is_ascending_expressive_power() {
    for window in Dialect::ALL.windows(2) {
        assert!(window[0] < window[1], "{:?} !< {:?}", window[0], window[1]);
    }
}

#[test]
fn routes_map_hot_warm_cold() {
    assert_eq!(Dialect::Triple8Pattern.route(), Route::Hot);
    assert_eq!(Dialect::ShaclCore.route(), Route::Warm);
    assert_eq!(Dialect::SparqlSelect.route(), Route::Warm);
    assert_eq!(Dialect::SparqlConstruct.route(), Route::Warm);
    assert_eq!(Dialect::OwlRl.route(), Route::Warm);
    assert_eq!(Dialect::N3.route(), Route::Cold);
}

#[test]
fn mask_bits_are_distinct_powers_of_two() {
    let mut seen: u8 = 0;
    for d in Dialect::ALL {
        let bit = d.mask_bit();
        assert_eq!(bit.count_ones(), 1);
        assert_eq!(seen & bit, 0, "duplicate mask bit for {d:?}");
        seen |= bit;
    }
}

#[test]
fn default_enabled_mask_excludes_n3() {
    assert_eq!(
        ProfileGates::DEFAULT_ENABLED_MASK & Dialect::N3.mask_bit(),
        0
    );
    for d in Dialect::ALL {
        if d != Dialect::N3 {
            assert_ne!(ProfileGates::DEFAULT_ENABLED_MASK & d.mask_bit(), 0);
        }
    }
}

// ---- ProfileGates validation -------------------------------------------

#[test]
fn gates_refuse_n3_in_actuation_mask() {
    let err = ProfileGates::new(ProfileId::new("p"), ALL_ENABLED, Dialect::N3.mask_bit(), 8);
    assert!(
        matches!(err, Err(Refusal::N3ActuationRefused(_))),
        "{err:?}"
    );
}

#[test]
fn gates_refuse_actuation_not_subset_of_enabled() {
    let err = ProfileGates::new(
        ProfileId::new("p"),
        Dialect::Triple8Pattern.mask_bit(),
        Dialect::ShaclCore.mask_bit(),
        8,
    );
    assert!(matches!(err, Err(Refusal::ValidationFailed(_))), "{err:?}");
}

#[test]
fn gates_refuse_hot_budget_over_eight() {
    let err = ProfileGates::new(ProfileId::new("p"), ALL_ENABLED, 0, 9);
    assert!(matches!(err, Err(Refusal::ValidationFailed(_))), "{err:?}");
}

// ---- N3 gate matrix ------------------------------------------------------
// Axes: N3 enabled × requires_n3_builtins × wants_actuation (2×2×2 = 8).

#[test]
fn n3_gate_matrix_full() {
    for n3_on in [false, true] {
        for requires_n3 in [false, true] {
            for wants_actuation in [false, true] {
                let enabled = if n3_on {
                    ALL_ENABLED
                } else {
                    ProfileGates::DEFAULT_ENABLED_MASK
                };
                // Actuation permitted on every non-N3 dialect.
                let router =
                    DialectRouter::new(profile(enabled, ProfileGates::DEFAULT_ENABLED_MASK, 8));
                let s = shape(2, false, false, requires_n3, wants_actuation);
                let result = router.decide(&s);
                match (n3_on, requires_n3, wants_actuation) {
                    (_, false, _) => {
                        let decision = ok_or_unreachable(result);
                        assert_eq!(decision.dialect, Dialect::Triple8Pattern);
                        assert_eq!(decision.route, Route::Hot);
                    }
                    (false, true, _) => assert!(
                        matches!(result, Err(Refusal::N3UnavailableByProfile(_))),
                        "n3 disabled + required must refuse by profile: {result:?}"
                    ),
                    (true, true, true) => assert!(
                        matches!(result, Err(Refusal::N3ActuationRefused(_))),
                        "n3 + actuation must refuse: {result:?}"
                    ),
                    (true, true, false) => {
                        let decision = ok_or_unreachable(result);
                        assert_eq!(decision.dialect, Dialect::N3);
                        assert_eq!(decision.route, Route::Cold);
                    }
                }
            }
        }
    }
}

// ---- Least-expressive-route law ------------------------------------------

#[test]
fn ler_ordering_property_over_all_dialect_pairs() {
    // For every ordered pair (a, b) with a < b: when both are enabled and
    // the shape's floor is ≤ a, decide never returns anything more
    // expressive than a.
    for a in Dialect::ALL {
        for b in Dialect::ALL {
            if a >= b {
                continue;
            }
            let router = DialectRouter::new(profile(a.mask_bit() | b.mask_bit(), 0, 8));
            // Shape whose capability floor is exactly `a`.
            let s = shape(
                1,
                a >= Dialect::SparqlConstruct,
                a >= Dialect::OwlRl,
                a == Dialect::N3,
                false,
            );
            assert!(s.minimum_dialect() <= a, "floor must not exceed a");
            let decision = ok_or_unreachable(router.decide(&s));
            assert!(
                decision.dialect <= a,
                "LER violated: chose {:?} over enabled {:?}",
                decision.dialect,
                a
            );
        }
    }
}

#[test]
fn verify_claim_refuses_more_expressive_claim() {
    let router = DialectRouter::new(profile(ALL_ENABLED, 0, 8));
    let s = shape(1, false, false, false, false);
    let computed = ok_or_unreachable(router.decide(&s));
    assert_eq!(computed.dialect, Dialect::Triple8Pattern);
    let inflated = RouteDecision {
        dialect: Dialect::SparqlSelect,
        route: Route::Warm,
        profile_hash: computed.profile_hash.clone(),
        decision_hash: computed.decision_hash.clone(),
    };
    let err = router.verify_claim(&s, &inflated);
    assert!(
        matches!(err, Err(Refusal::LeastExpressiveRouteViolation(_))),
        "{err:?}"
    );
}

#[test]
fn verify_claim_refuses_hash_drift() {
    let router = DialectRouter::new(profile(ALL_ENABLED, 0, 8));
    let s = shape(1, false, false, false, false);
    let mut claimed = ok_or_unreachable(router.decide(&s));
    claimed.decision_hash = Digest::new("0000".to_string());
    let err = router.verify_claim(&s, &claimed);
    assert!(
        matches!(err, Err(Refusal::RouteDecisionMismatch(_))),
        "{err:?}"
    );
}

#[test]
fn verify_claim_accepts_exact_decision() {
    let router = DialectRouter::new(profile(ALL_ENABLED, 0, 8));
    let s = shape(3, true, false, false, false);
    let claimed = ok_or_unreachable(router.decide(&s));
    assert_eq!(claimed.dialect, Dialect::SparqlConstruct);
    assert!(router.verify_claim(&s, &claimed).is_ok());
}

// ---- Hot budget / refusal coverage ----------------------------------------

#[test]
fn over_budget_falls_to_warm_when_available() {
    let router = DialectRouter::new(profile(ProfileGates::DEFAULT_ENABLED_MASK, 0, 4));
    let s = shape(5, false, false, false, false);
    let decision = ok_or_unreachable(router.decide(&s));
    assert_eq!(decision.dialect, Dialect::ShaclCore);
    assert_eq!(decision.route, Route::Warm);
}

#[test]
fn over_budget_with_no_warm_dialect_is_warm_path_required() {
    let router = DialectRouter::new(profile(Dialect::Triple8Pattern.mask_bit(), 0, 4));
    let s = shape(5, false, false, false, false);
    let err = router.decide(&s);
    assert!(matches!(err, Err(Refusal::WarmPathRequired(_))), "{err:?}");
}

#[test]
fn nothing_enabled_is_unsupported_dialect() {
    let router = DialectRouter::new(profile(0, 0, 8));
    let s = shape(1, false, false, false, false);
    let err = router.decide(&s);
    assert!(
        matches!(err, Err(Refusal::UnsupportedDialect(_))),
        "{err:?}"
    );
}

#[test]
fn actuation_skips_non_permitted_dialects() {
    // Actuation only permitted on ShaclCore; a hot-capable shape wanting
    // actuation must land on ShaclCore, not Triple8Pattern.
    let router = DialectRouter::new(profile(
        ProfileGates::DEFAULT_ENABLED_MASK,
        Dialect::ShaclCore.mask_bit(),
        8,
    ));
    let s = shape(1, false, false, false, true);
    let decision = ok_or_unreachable(router.decide(&s));
    assert_eq!(decision.dialect, Dialect::ShaclCore);
}

// ---- Determinism ------------------------------------------------------------

#[test]
fn decision_hash_is_deterministic() {
    let router = DialectRouter::new(profile(ALL_ENABLED, 0, 8));
    let s = shape(2, true, false, false, false);
    let d1 = ok_or_unreachable(router.decide(&s));
    let d2 = ok_or_unreachable(router.decide(&s));
    assert_eq!(d1, d2);
    assert_eq!(d1.decision_hash, d2.decision_hash);
    assert_eq!(d1.profile_hash, d2.profile_hash);
}

#[test]
fn decision_hash_distinguishes_shapes_and_profiles() {
    let router = DialectRouter::new(profile(ALL_ENABLED, 0, 8));
    let d1 = ok_or_unreachable(router.decide(&shape(2, false, false, false, false)));
    let d2 = ok_or_unreachable(router.decide(&shape(3, false, false, false, false)));
    assert_ne!(d1.decision_hash, d2.decision_hash);

    let other = DialectRouter::new(profile(ALL_ENABLED, 0, 7));
    let d3 = ok_or_unreachable(other.decide(&shape(2, false, false, false, false)));
    assert_ne!(d1.decision_hash, d3.decision_hash);
    assert_ne!(d1.profile_hash, d3.profile_hash);
}
