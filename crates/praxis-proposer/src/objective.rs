//! The objective function is **authored data**, never invented values.
//!
//! Per Vision 2030 Non-goal 1 ("No value discovery") and PR-14's hard
//! requirement, this crate ships the *algebra* — a weighted linear scoring
//! over a fixed, documented set of numeric fluents — and the domain author
//! ships the *values* (the weights) as a JSON file. The default
//! `revenue_objective.json` beside this crate is a starting point the user
//! is expected to edit; the loader accepts any finite weights.
//!
//! # Schema (`revenue_objective.json`)
//!
//! ```json
//! {
//!   "name": "human-readable objective name",
//!   "version": "author-chosen version string",
//!   "weights": {
//!     "realized_revenue":       1.0,
//!     "pipeline_value_at_risk": -0.25,
//!     "time_penalty":           500.0,
//!     "stage_advance":          0.0
//!   }
//! }
//! ```
//!
//! - Unknown top-level fields and unknown fluent names are **rejected**
//!   (strict loading, matching PR-9 config-admission discipline).
//! - Missing fluent names default to weight `0.0` (the fluent is ignored).
//! - Non-finite weights (NaN/inf) are rejected: scores must be total-ordered.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::domain::{Account, Stage};

/// The fixed fluent vocabulary this crate knows how to compute, in the
/// canonical evaluation order. Scoring iterates this array (never the map)
/// so floating-point summation order is deterministic.
pub const FLUENT_NAMES: [&str; 4] = [
    "realized_revenue",
    "pipeline_value_at_risk",
    "time_penalty",
    "stage_advance",
];

/// Fluent semantics, given a candidate move of `account` to `target`:
///
/// | fluent                   | value                                                        |
/// |--------------------------|--------------------------------------------------------------|
/// | `realized_revenue`       | `amount_cents` if `target == ClosedWon`, else `0`            |
/// | `pipeline_value_at_risk` | `amount_cents` if `target < ClosedWon` (value still open), else `0` |
/// | `time_penalty`           | `days_in_stage` (staleness of the account being moved)       |
/// | `stage_advance`          | number of stages jumped (`target.index() - stage.index()`)   |
///
/// The *signs and magnitudes* attached to these are the author's judgment,
/// expressed in [`ObjectiveFunction::weights`]. This function only reports
/// facts about the candidate.
pub fn compute_fluents(account: &Account, target: Stage) -> [f64; 4] {
    let amount = account.amount_cents as f64;
    let realized = if target == Stage::ClosedWon { amount } else { 0.0 };
    let at_risk = if target < Stage::ClosedWon { amount } else { 0.0 };
    let staleness = account.days_in_stage as f64;
    // Subtract in f64, not u8: a backward candidate (target < current stage)
    // is a legitimate query for this public function and yields a *negative*
    // advance. Doing `target.index() - account.stage.index()` in u8 underflows
    // — a debug-build panic and a release-build wrap to a huge positive value.
    // (Surfaced by differential testing against a naive reimplementation.)
    let advance = f64::from(target.index()) - f64::from(account.stage.index());
    [realized, at_risk, staleness, advance]
}

/// Errors from loading or validating an authored objective.
#[derive(Debug)]
pub enum ObjectiveError {
    /// The file/string was not valid JSON matching the schema.
    Parse(String),
    /// A weight key is not in [`FLUENT_NAMES`].
    UnknownFluent(String),
    /// A weight is NaN or infinite.
    NonFiniteWeight(String),
    /// I/O failure reading the file.
    Io(String),
}

impl fmt::Display for ObjectiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectiveError::Parse(e) => write!(f, "objective parse error: {e}"),
            ObjectiveError::UnknownFluent(k) => {
                write!(f, "unknown fluent '{k}' (known: {FLUENT_NAMES:?})")
            }
            ObjectiveError::NonFiniteWeight(k) => {
                write!(f, "weight for '{k}' must be finite")
            }
            ObjectiveError::Io(e) => write!(f, "objective io error: {e}"),
        }
    }
}

impl std::error::Error for ObjectiveError {}

/// A domain-authored weighted scoring specification.
///
/// This type carries no defaults and computes nothing on its own initiative:
/// it is deserialized from a file the domain author wrote. The system never
/// invents these values (Non-goal 1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectiveFunction {
    /// Human-readable name, cited in every proposal rationale.
    pub name: String,
    /// Author-chosen version string, cited in every proposal rationale.
    pub version: String,
    /// Fluent name -> weight. BTreeMap for stable serialization; scoring
    /// iterates [`FLUENT_NAMES`], not this map.
    pub weights: BTreeMap<String, f64>,
}

impl ObjectiveFunction {
    /// Parse and validate a **revenue** objective from a JSON string (weights
    /// checked against the revenue [`FLUENT_NAMES`]). Domain packs with a
    /// different fluent vocabulary use [`ObjectiveFunction::from_json_str_for`].
    pub fn from_json_str(s: &str) -> Result<Self, ObjectiveError> {
        Self::from_json_str_for(s, &FLUENT_NAMES)
    }

    /// Load a **revenue** objective from a JSON file on disk.
    pub fn from_path(path: &std::path::Path) -> Result<Self, ObjectiveError> {
        Self::from_path_for(path, &FLUENT_NAMES)
    }

    /// Parse and validate an authored objective against an **arbitrary** domain
    /// fluent vocabulary. This is the reuse seam (Genesis Day 6): the loader,
    /// the `deny_unknown_fields` discipline, and the finite-weight rule are
    /// identical across domains; only the set of allowed fluent names changes.
    /// The church pack calls this with its own vocabulary.
    pub fn from_json_str_for(s: &str, allowed_fluents: &[&str]) -> Result<Self, ObjectiveError> {
        let obj: ObjectiveFunction =
            serde_json::from_str(s).map_err(|e| ObjectiveError::Parse(e.to_string()))?;
        obj.validate_fluents(allowed_fluents)?;
        Ok(obj)
    }

    /// Load an authored objective from disk against an arbitrary domain fluent
    /// vocabulary. See [`ObjectiveFunction::from_json_str_for`].
    pub fn from_path_for(
        path: &std::path::Path,
        allowed_fluents: &[&str],
    ) -> Result<Self, ObjectiveError> {
        let s = std::fs::read_to_string(path).map_err(|e| ObjectiveError::Io(e.to_string()))?;
        Self::from_json_str_for(&s, allowed_fluents)
    }

    /// Strict validation against the revenue vocabulary. Kept for revenue
    /// callers; delegates to [`ObjectiveFunction::validate_fluents`].
    pub fn validate(&self) -> Result<(), ObjectiveError> {
        self.validate_fluents(&FLUENT_NAMES)
    }

    /// Strict validation against an arbitrary fluent vocabulary: every weight
    /// key must be a known fluent, every weight finite. This one rule serves
    /// every domain pack.
    pub fn validate_fluents(&self, allowed_fluents: &[&str]) -> Result<(), ObjectiveError> {
        for (k, v) in &self.weights {
            if !allowed_fluents.contains(&k.as_str()) {
                return Err(ObjectiveError::UnknownFluent(k.clone()));
            }
            if !v.is_finite() {
                return Err(ObjectiveError::NonFiniteWeight(k.clone()));
            }
        }
        Ok(())
    }

    /// Weight for a fluent; absent fluents weigh `0.0` (ignored).
    pub fn weight(&self, fluent: &str) -> f64 {
        self.weights.get(fluent).copied().unwrap_or(0.0)
    }

    /// Score a candidate and explain it.
    ///
    /// Returns `(score, rationale_lines)`. The score is the dot product of
    /// the authored weights with [`compute_fluents`], summed in the fixed
    /// [`FLUENT_NAMES`] order (deterministic f64 evaluation). The rationale
    /// cites, for every fluent with a nonzero authored weight, the fluent
    /// value, the weight, and the contribution — auditable judgment, not
    /// vibes. Zero-weight fluents are noted as ignored.
    pub fn score(&self, account: &Account, target: Stage) -> (f64, Vec<String>) {
        // Reuse the domain-independent scorer (Genesis Day 6): revenue and
        // church produce their rationales through the exact same code. Only
        // the fluent vocabulary and the per-candidate description differ.
        let fluents = compute_fluents(account, target);
        let candidate_desc = format!(
            "candidate: account {} {} -> {} (amount_cents={}, days_in_stage={})",
            account.id,
            account.stage.pddl_name(),
            target.pddl_name(),
            account.amount_cents,
            account.days_in_stage
        );
        crate::engine::score(self, &FLUENT_NAMES, &candidate_desc, &fluents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_objective_file_loads() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("revenue_objective.json");
        let obj = ObjectiveFunction::from_path(&path).expect("default objective must load");
        assert!(obj.weights.contains_key("realized_revenue"));
    }

    #[test]
    fn stage_advance_is_negative_for_backward_target_no_underflow() {
        // Regression (differential-verification finding): compute_fluents must
        // not underflow when target < current stage. It should report a
        // negative `stage_advance`, not panic (debug) or wrap (release).
        let acct = Account {
            id: "a".into(),
            stage: Stage::ClosedWon,
            amount_cents: 1000,
            security_review_done: true,
            legal_approved: true,
            exec_sponsor: true,
            days_in_stage: 0,
        };
        let fluents = compute_fluents(&acct, Stage::Lead);
        // stage_advance is index 3 in FLUENT_NAMES.
        assert_eq!(fluents[3], -4.0);
    }

    #[test]
    fn unknown_fluent_rejected() {
        let s = r#"{"name":"x","version":"1","weights":{"vibes":1.0}}"#;
        assert!(matches!(
            ObjectiveFunction::from_json_str(s),
            Err(ObjectiveError::UnknownFluent(_))
        ));
    }

    #[test]
    fn non_finite_weight_rejected() {
        // JSON can't express NaN, but validate() guards programmatic construction.
        let mut weights = BTreeMap::new();
        weights.insert("time_penalty".to_string(), f64::NAN);
        let obj = ObjectiveFunction {
            name: "x".into(),
            version: "1".into(),
            weights,
        };
        assert!(matches!(
            obj.validate(),
            Err(ObjectiveError::NonFiniteWeight(_))
        ));
    }

    #[test]
    fn missing_weights_default_to_zero() {
        let s = r#"{"name":"x","version":"1","weights":{"realized_revenue":1.0}}"#;
        let obj = ObjectiveFunction::from_json_str(s).unwrap();
        assert_eq!(obj.weight("time_penalty"), 0.0);
    }
}
