//! Least-expressive dialect routing for the chatman engine.
//!
//! The routing law: a query is always answered by the **least expressive**
//! dialect that can express it and that the active profile permits. The
//! [`Dialect`] enum's derived `Ord` *is* that law — variants are declared in
//! ascending expressive power, so `a < b` means "`a` is strictly less
//! expressive than `b`". Any component claiming a more expressive route than
//! [`DialectRouter::decide`] would choose is refused with
//! [`Refusal::LeastExpressiveRouteViolation`].
//!
//! All hashing goes through [`wasm4pm_compat::hash::blake3_combined`] with
//! version- and field-tagged material; no wall clock, no randomness.

use serde::{Deserialize, Serialize};
use wasm4pm_compat::hash::blake3_combined;

use super::abi::{Digest, ProfileId, Refusal};

/// Domain-tag prefix for [`ProfileGates::hash`] material (versioned so a
/// future scheme change cannot collide with v1 digests).
const PROFILE_HASH_TAG: &str = "chatman/router/profile-gates/v1";

/// Domain-tag prefix for [`RouteDecision`] `decision_hash` material.
const DECISION_HASH_TAG: &str = "chatman/router/route-decision/v1";

/// Query dialects ordered **ascending by expressive power**.
///
/// The declaration order (and therefore the derived `Ord`) is the
/// least-expressive-route law: `Triple8Pattern < ShaclCore < SparqlSelect <
/// SparqlConstruct < OwlRl < N3`. Do not reorder variants — the router,
/// the mask bits, and the LER verification all depend on this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Dialect {
    /// Fixed-arity 8-constraint triple pattern matching (hot path).
    Triple8Pattern = 0,
    /// SHACL Core shape validation.
    ShaclCore = 1,
    /// SPARQL SELECT over the snapshot.
    SparqlSelect = 2,
    /// SPARQL CONSTRUCT (graph-producing queries).
    SparqlConstruct = 3,
    /// OWL RL materialization.
    OwlRl = 4,
    /// N3 rules with builtins (cold path; never actuation-capable).
    N3 = 5,
}

impl Dialect {
    /// Every dialect in ascending expressive order. Iteration over this array
    /// is the canonical least-expressive-first scan.
    pub const ALL: [Dialect; 6] = [
        Dialect::Triple8Pattern,
        Dialect::ShaclCore,
        Dialect::SparqlSelect,
        Dialect::SparqlConstruct,
        Dialect::OwlRl,
        Dialect::N3,
    ];

    /// Maps a dialect to its execution route. `Triple8Pattern` is the only
    /// hot dialect; `N3` is the only cold dialect; everything between is warm.
    ///
    /// # Complexity
    /// O(1) — exhaustive match on a fieldless enum.
    pub const fn route(self) -> Route {
        match self {
            Dialect::Triple8Pattern => Route::Hot,
            Dialect::ShaclCore
            | Dialect::SparqlSelect
            | Dialect::SparqlConstruct
            | Dialect::OwlRl => Route::Warm,
            Dialect::N3 => Route::Cold,
        }
    }

    /// The bit this dialect occupies in a profile mask (`1 << discriminant`).
    ///
    /// # Complexity
    /// O(1).
    pub const fn mask_bit(self) -> u8 {
        1 << (self as u8)
    }

    /// Stable name used as field-tagged hash material.
    ///
    /// # Complexity
    /// O(1).
    pub const fn name(self) -> &'static str {
        match self {
            Dialect::Triple8Pattern => "Triple8Pattern",
            Dialect::ShaclCore => "ShaclCore",
            Dialect::SparqlSelect => "SparqlSelect",
            Dialect::SparqlConstruct => "SparqlConstruct",
            Dialect::OwlRl => "OwlRl",
            Dialect::N3 => "N3",
        }
    }
}

/// Execution route tiers, ordered ascending by cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Route {
    /// Sub-microsecond fixed-pattern path (`Triple8Pattern` only).
    Hot,
    /// Bounded query/validation path.
    Warm,
    /// Unbounded-rule reasoning path (`N3` only).
    Cold,
}

impl Route {
    /// Stable name used as field-tagged hash material.
    ///
    /// # Complexity
    /// O(1).
    pub const fn name(self) -> &'static str {
        match self {
            Route::Hot => "Hot",
            Route::Warm => "Warm",
            Route::Cold => "Cold",
        }
    }
}

/// Per-profile dialect permissions. Constructed only through
/// [`ProfileGates::new`], which enforces the gate laws.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProfileGates {
    /// Identity of the profile these gates belong to.
    pub profile_id: ProfileId,
    /// Bitmask of dialects the profile may execute at all. The N3 bit is
    /// **default 0** (see [`ProfileGates::DEFAULT_ENABLED_MASK`]); enabling
    /// N3 is always an explicit act.
    pub enabled_dialects_mask: u8,
    /// Bitmask of dialects permitted to drive actuation (side effects).
    /// Must be a subset of `enabled_dialects_mask` and must never contain N3.
    pub actuation_dialects_mask: u8,
    /// Maximum constraint count the hot path accepts (≤ 8 by construction).
    pub max_hot_constraints: u8,
}

impl ProfileGates {
    /// Default enabled mask: every dialect except N3. N3 stays off unless a
    /// profile turns it on deliberately.
    pub const DEFAULT_ENABLED_MASK: u8 = Dialect::Triple8Pattern.mask_bit()
        | Dialect::ShaclCore.mask_bit()
        | Dialect::SparqlSelect.mask_bit()
        | Dialect::SparqlConstruct.mask_bit()
        | Dialect::OwlRl.mask_bit();

    /// Builds validated gates.
    ///
    /// # Errors
    /// - [`Refusal::N3ActuationRefused`] if the actuation mask contains the
    ///   N3 bit — N3 may never actuate, regardless of enablement.
    /// - [`Refusal::ValidationFailed`] if the actuation mask is not a subset
    ///   of the enabled mask, or if `max_hot_constraints > 8`.
    ///
    /// # Complexity
    /// O(1) — bit arithmetic only.
    pub fn new(
        profile_id: ProfileId,
        enabled_dialects_mask: u8,
        actuation_dialects_mask: u8,
        max_hot_constraints: u8,
    ) -> Result<Self, Refusal> {
        if actuation_dialects_mask & Dialect::N3.mask_bit() != 0 {
            return Err(Refusal::N3ActuationRefused(format!(
                "profile {profile_id}: actuation mask {actuation_dialects_mask:#010b} \
                 contains the N3 bit; N3 may never drive actuation"
            )));
        }
        if actuation_dialects_mask & !enabled_dialects_mask != 0 {
            return Err(Refusal::ValidationFailed(format!(
                "profile {profile_id}: actuation mask {actuation_dialects_mask:#010b} is not \
                 a subset of enabled mask {enabled_dialects_mask:#010b}"
            )));
        }
        if max_hot_constraints > 8 {
            return Err(Refusal::ValidationFailed(format!(
                "profile {profile_id}: max_hot_constraints {max_hot_constraints} exceeds the \
                 hot-path ceiling of 8"
            )));
        }
        Ok(Self {
            profile_id,
            enabled_dialects_mask,
            actuation_dialects_mask,
            max_hot_constraints,
        })
    }

    /// Whether the profile enables `dialect` at all.
    ///
    /// # Complexity
    /// O(1).
    pub const fn is_enabled(&self, dialect: Dialect) -> bool {
        self.enabled_dialects_mask & dialect.mask_bit() != 0
    }

    /// Whether the profile permits `dialect` to drive actuation.
    ///
    /// # Complexity
    /// O(1).
    pub const fn permits_actuation(&self, dialect: Dialect) -> bool {
        self.actuation_dialects_mask & dialect.mask_bit() != 0
    }

    /// Field-tagged BLAKE3 digest of the gates. Same gates → byte-identical
    /// digest; any field change → different digest. Hashing is delegated to
    /// [`blake3_combined`] (length-prefixed, injective); no wall clock.
    ///
    /// # Complexity
    /// O(|profile_id|) — dominated by hashing the identity string.
    pub fn hash(&self) -> Digest {
        let enabled = self.enabled_dialects_mask.to_string();
        let actuation = self.actuation_dialects_mask.to_string();
        let max_hot = self.max_hot_constraints.to_string();
        Digest::new(blake3_combined(&[
            PROFILE_HASH_TAG,
            "profile_id",
            self.profile_id.as_str(),
            "enabled_dialects_mask",
            &enabled,
            "actuation_dialects_mask",
            &actuation,
            "max_hot_constraints",
            &max_hot,
        ]))
    }
}

/// The expressiveness demands of one query, as classified by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QueryShape {
    /// Number of triple constraints in the query.
    pub constraint_count: u8,
    /// Whether the query produces a graph (CONSTRUCT semantics).
    pub requires_construct: bool,
    /// Whether the query needs OWL RL entailment.
    pub requires_owl: bool,
    /// Whether the query needs N3 builtins.
    pub requires_n3_builtins: bool,
    /// Whether the query intends to drive actuation (side effects).
    pub wants_actuation: bool,
}

impl QueryShape {
    /// The least expressive dialect capable of expressing this shape,
    /// ignoring profile gates and the hot-path constraint budget.
    /// Capabilities are monotone in `Dialect`'s order: every dialect ≥ the
    /// returned floor can also express the shape.
    ///
    /// # Complexity
    /// O(1).
    const fn minimum_dialect(&self) -> Dialect {
        if self.requires_n3_builtins {
            Dialect::N3
        } else if self.requires_owl {
            Dialect::OwlRl
        } else if self.requires_construct {
            Dialect::SparqlConstruct
        } else {
            Dialect::Triple8Pattern
        }
    }

    /// Field-tagged hash material for this shape (bools encoded as 0/1;
    /// fixed field order and separators, so the encoding is injective over
    /// `QueryShape`).
    ///
    /// # Complexity
    /// O(1) — five bounded fields.
    fn hash_material(&self) -> String {
        format!(
            "constraint_count={};construct={};owl={};n3={};actuation={}",
            self.constraint_count,
            self.requires_construct as u8,
            self.requires_owl as u8,
            self.requires_n3_builtins as u8,
            self.wants_actuation as u8
        )
    }
}

/// The router's binding answer for one query shape under one profile.
///
/// Derives only what upstream [`Digest`] supports (no serde: `Digest` in
/// `wasm4pm_compat` does not derive `Serialize`/`Deserialize`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RouteDecision {
    /// The least expressive permitted dialect.
    pub dialect: Dialect,
    /// The execution route implied by `dialect`.
    pub route: Route,
    /// Digest of the [`ProfileGates`] the decision was made under.
    pub profile_hash: Digest,
    /// Digest binding (profile, shape, dialect, route) together.
    pub decision_hash: Digest,
}

/// Routes query shapes to the least expressive permitted dialect under a
/// fixed [`ProfileGates`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialectRouter {
    gates: ProfileGates,
}

impl DialectRouter {
    /// Builds a router over validated gates.
    pub fn new(gates: ProfileGates) -> Self {
        Self { gates }
    }

    /// Borrows the gates this router decides under.
    pub fn gates(&self) -> &ProfileGates {
        &self.gates
    }

    /// Decides the least expressive permitted dialect for `shape`.
    ///
    /// Scans [`Dialect::ALL`] ascending from the shape's capability floor and
    /// returns the first dialect that is (a) capable, (b) enabled by the
    /// profile, (c) within the hot constraint budget when hot, and (d)
    /// actuation-permitted when the shape wants actuation.
    ///
    /// # Errors
    /// - [`Refusal::N3UnavailableByProfile`] — the shape needs N3 builtins
    ///   but the profile does not enable N3.
    /// - [`Refusal::N3ActuationRefused`] — the shape needs N3 builtins *and*
    ///   wants actuation; N3 never actuates.
    /// - [`Refusal::WarmPathRequired`] — the shape exceeds the hot constraint
    ///   budget, so a warm dialect is required, but the profile enables none
    ///   that qualifies.
    /// - [`Refusal::UnsupportedDialect`] — no enabled dialect satisfies the
    ///   shape for any other reason.
    ///
    /// # Complexity
    /// O(1) — the scan covers the fixed 6-variant [`Dialect::ALL`]; each
    /// candidate check is bit arithmetic. Sealing the decision hashes the
    /// profile identity, O(|profile_id|).
    pub fn decide(&self, shape: &QueryShape) -> Result<RouteDecision, Refusal> {
        let floor = shape.minimum_dialect();

        if floor == Dialect::N3 {
            if !self.gates.is_enabled(Dialect::N3) {
                return Err(Refusal::N3UnavailableByProfile(format!(
                    "profile {}: shape requires N3 builtins but the profile does not \
                     enable N3 (enabled mask {:#010b})",
                    self.gates.profile_id, self.gates.enabled_dialects_mask
                )));
            }
            if shape.wants_actuation {
                return Err(Refusal::N3ActuationRefused(format!(
                    "profile {}: shape requires N3 builtins and wants actuation; \
                     N3 may never drive actuation",
                    self.gates.profile_id
                )));
            }
        }

        // Whether the shape was hot-capable except for the constraint budget:
        // distinguishes WarmPathRequired from UnsupportedDialect on miss.
        let mut hot_blocked_by_budget = false;

        // O(1): fixed 6-iteration scan, ascending expressive power (LER law).
        for dialect in Dialect::ALL {
            if dialect < floor {
                continue;
            }
            if dialect == Dialect::Triple8Pattern
                && shape.constraint_count > self.gates.max_hot_constraints
            {
                hot_blocked_by_budget = self.gates.is_enabled(Dialect::Triple8Pattern);
                continue;
            }
            if !self.gates.is_enabled(dialect) {
                continue;
            }
            if shape.wants_actuation && !self.gates.permits_actuation(dialect) {
                if dialect == Dialect::N3 {
                    // N3 can never be in the actuation mask (gate law), so an
                    // actuation-wanting shape landing here is a hard refusal.
                    return Err(Refusal::N3ActuationRefused(format!(
                        "profile {}: only N3 could express the shape but N3 may never \
                         drive actuation",
                        self.gates.profile_id
                    )));
                }
                continue;
            }
            return Ok(self.seal(shape, dialect));
        }

        if hot_blocked_by_budget {
            return Err(Refusal::WarmPathRequired(format!(
                "profile {}: shape has {} constraints, exceeding the hot budget of {}; \
                 a warm path is required but no enabled warm dialect qualifies",
                self.gates.profile_id, shape.constraint_count, self.gates.max_hot_constraints
            )));
        }
        Err(Refusal::UnsupportedDialect(format!(
            "profile {}: no enabled dialect satisfies the shape (enabled mask {:#010b}, \
             actuation mask {:#010b}, wants_actuation={})",
            self.gates.profile_id,
            self.gates.enabled_dialects_mask,
            self.gates.actuation_dialects_mask,
            shape.wants_actuation
        )))
    }

    /// Verifies a claimed decision against the router's own decision.
    ///
    /// # Errors
    /// - [`Refusal::LeastExpressiveRouteViolation`] — the claim names a
    ///   dialect strictly more expressive than the router would choose.
    /// - [`Refusal::RouteDecisionMismatch`] — any other drift (dialect,
    ///   route, profile hash, or decision hash differs).
    /// - Any refusal [`DialectRouter::decide`] itself returns for the shape.
    ///
    /// # Complexity
    /// O(1) — one `decide` plus constant-size comparisons.
    pub fn verify_claim(&self, shape: &QueryShape, claimed: &RouteDecision) -> Result<(), Refusal> {
        let computed = self.decide(shape)?;
        if claimed.dialect > computed.dialect {
            return Err(Refusal::LeastExpressiveRouteViolation(format!(
                "profile {}: claimed dialect {} is more expressive than the least \
                 expressive permitted dialect {}",
                self.gates.profile_id,
                claimed.dialect.name(),
                computed.dialect.name()
            )));
        }
        if claimed != &computed {
            return Err(Refusal::RouteDecisionMismatch(format!(
                "profile {}: claimed decision drifts from computed decision \
                 (claimed dialect {} route {} decision_hash {}; computed dialect {} \
                 route {} decision_hash {})",
                self.gates.profile_id,
                claimed.dialect.name(),
                claimed.route.name(),
                claimed.decision_hash.0,
                computed.dialect.name(),
                computed.route.name(),
                computed.decision_hash.0
            )));
        }
        Ok(())
    }

    /// Builds the sealed decision for a chosen dialect. Material is version-
    /// and field-tagged; same (gates, shape, dialect) → byte-identical digest.
    ///
    /// # Complexity
    /// O(|profile_id|) — dominated by the profile-identity hash.
    fn seal(&self, shape: &QueryShape, dialect: Dialect) -> RouteDecision {
        let route = dialect.route();
        let profile_hash = self.gates.hash();
        let shape_material = shape.hash_material();
        let decision_hash = Digest::new(blake3_combined(&[
            DECISION_HASH_TAG,
            "profile_hash",
            &profile_hash.0,
            "shape",
            &shape_material,
            "dialect",
            dialect.name(),
            "route",
            route.name(),
        ]));
        RouteDecision {
            dialect,
            route,
            profile_hash,
            decision_hash,
        }
    }
}

#[cfg(test)]
#[path = "router_test.rs"]
mod tests;
