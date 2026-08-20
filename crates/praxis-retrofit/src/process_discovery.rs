//! Lifecycle process discovery and conformance checking over retrofit OCEL logs.
//!
//! ## What this module IS
//!
//! - A thin loader ([`load_ocel`]) for the JSON OCEL 2.0 logs written by
//!   [`crate::ocel_log`], and a thin wrapper ([`discover_lifecycle`]) over
//!   `wasm4pm_compat::dfg::discover_ocel_dfg` that mines a Directly-Follows
//!   Graph from this crate's own Discover/Audit/Apply/Validate/Admit event
//!   vocabulary (see [`crate::ocel_log`]).
//! - A conformance report ([`conformance_report`]) that combines
//!   `wasm4pm_compat::dfg::dfg_fitness`/`dfg_precision` with a plain
//!   set-difference against a caller-supplied reference arc list, so callers
//!   can see exactly which arcs are missing from, or unexpected in, the
//!   observed log.
//! - Two named reference-arc helpers:
//!   - [`reference_arcs_admission_lifecycle`] — the `DISCOVER_PROJECTS` →
//!     `ADMIT_PROJECT` → `COMPOSE_PROJECT` project-admission calculus
//!     declared in `packs/chatman-ecosystem-pack/ontology.ttl` lines 76-90
//!     (`chatman:discover-projects`, `chatman:admit-project`,
//!     `chatman:compose-project`), translated onto **this crate's own**
//!     event-type vocabulary (`Discover`, `Admit`, `Audit`) since that is
//!     what `discover_lifecycle`'s DFG actually contains. `COMPOSE_PROJECT`
//!     (exposing admitted capability to SELECT/CONSTRUCT) has no direct
//!     analogue in this crate's telemetry vocabulary; it is mapped onto
//!     `Audit`, the retrofit-domain step that follows admission.
//!   - [`reference_arcs_full_lifecycle`] — the full `OBSERVE` → ... →
//!     `REPLAY` BRCE pipeline declared in the same ontology file, lines
//!     96-145 (`chatman:observe` through `chatman:replay`). This is a
//!     **different layer** (the BRCE actuation pipeline, not the
//!     retrofit/ecosystem admission calculus) and its arcs use the
//!     ontology's own literal labels (`OBSERVE`, `SELECT`, ...), which do
//!     not appear anywhere in this crate's OCEL event-type vocabulary. It
//!     is included only as a **documented negative comparison**: running it
//!     through [`conformance_report`] against a retrofit OCEL log is
//!     expected to yield zero present arcs (fitness 0.0), because the two
//!     vocabularies never intersect.
//!
//! ## What this module is **NOT**
//!
//! - **Not** an OCEL discovery/mining engine of its own — all discovery and
//!   conformance math is delegated to `wasm4pm_compat::dfg`.
//! - **Not** an admission or receipt path. `load_ocel`/`discover_lifecycle`
//!   read and derive from a telemetry log; none of this feeds a BLAKE3
//!   receipt or a `Refusal` decision.

use std::path::Path;

use serde::{Deserialize, Serialize};
use wasm4pm_compat::dfg::{dfg_fitness, dfg_precision, discover_ocel_dfg};
use wasm4pm_compat::ocel::OCEL;

use crate::error::{Result, RetrofitError};

/// Load and parse a JSON OCEL 2.0 log previously written via [`crate::ocel_log`].
///
/// # Errors
///
/// Returns [`RetrofitError`] if the file cannot be read or does not parse as
/// a structurally valid `OCEL` document.
pub fn load_ocel(path: &Path) -> Result<OCEL> {
    let bytes = std::fs::read(path).map_err(|e| {
        RetrofitError::Io(std::io::Error::new(
            e.kind(),
            format!("failed to read OCEL log at {}: {e}", path.display()),
        ))
    })?;
    let ocel: OCEL = serde_json::from_slice(&bytes)?;
    Ok(ocel)
}

/// Discover the retrofit/ecosystem lifecycle Directly-Follows Graph from an
/// admitted OCEL log.
///
/// Thin wrapper over `wasm4pm_compat::dfg::discover_ocel_dfg`; see that
/// function's docs for the mining algorithm (group by shared object
/// membership, sort by timestamp, count directly-follows pairs).
pub fn discover_lifecycle(ocel: &OCEL) -> wasm4pm_compat::models::DFG {
    discover_ocel_dfg(ocel)
}

/// Reference arcs for the project-admission calculus
/// (`packs/chatman-ecosystem-pack/ontology.ttl` lines 76-90:
/// `chatman:discover-projects` / `chatman:admit-project` /
/// `chatman:compose-project`), expressed in this crate's own
/// Discover/Audit/Apply/Validate/Admit event-type vocabulary
/// (see [`crate::ocel_log`]) rather than the ontology's own step labels,
/// since it is that vocabulary that appears in this crate's OCEL logs and
/// therefore in [`discover_lifecycle`]'s DFG.
///
/// `DISCOVER_PROJECTS` -> `ADMIT_PROJECT` maps to `Discover` -> `Admit`.
/// `ADMIT_PROJECT` -> `COMPOSE_PROJECT` maps to `Admit` -> `Audit`, since
/// this crate has no distinct "composition/exposure" event type; `Audit` is
/// the step that follows admission in the retrofit domain.
pub fn reference_arcs_admission_lifecycle() -> Vec<(String, String)> {
    vec![
        ("Discover".to_string(), "Admit".to_string()),
        ("Admit".to_string(), "Audit".to_string()),
    ]
}

/// Reference arcs for the full BRCE pipeline
/// (`packs/chatman-ecosystem-pack/ontology.ttl` lines 96-145: `chatman:observe`
/// through `chatman:replay`), expressed as literal ontology labels
/// (`OBSERVE`, `SELECT`, `CONSTRUCT`, `GGEN_RENDER`, `LEAN_ADMIT`,
/// `MFACT_CERTIFY`, `HOOK_INTENT`, `BRCE_DO`, `RECEIPT`, `REPLAY`).
///
/// This is a **different layer** from the retrofit/ecosystem admission
/// calculus above (the BRCE actuation pipeline vs. project-admission), and
/// it is **expected to be non-conformant** against this crate's own DFG: no
/// retrofit OCEL log emits these labels as event types, so
/// [`conformance_report`] run against this reference set is included only
/// as a documented negative comparison, not a passing conformance target.
pub fn reference_arcs_full_lifecycle() -> Vec<(String, String)> {
    vec![
        ("OBSERVE".to_string(), "SELECT".to_string()),
        ("SELECT".to_string(), "CONSTRUCT".to_string()),
        ("CONSTRUCT".to_string(), "GGEN_RENDER".to_string()),
        ("GGEN_RENDER".to_string(), "LEAN_ADMIT".to_string()),
        ("LEAN_ADMIT".to_string(), "MFACT_CERTIFY".to_string()),
        ("MFACT_CERTIFY".to_string(), "HOOK_INTENT".to_string()),
        ("HOOK_INTENT".to_string(), "BRCE_DO".to_string()),
        ("BRCE_DO".to_string(), "RECEIPT".to_string()),
        ("RECEIPT".to_string(), "REPLAY".to_string()),
    ]
}

/// Conformance summary comparing an observed lifecycle DFG against a
/// reference arc set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConformanceSummary {
    /// Fraction of `reference_arcs` present in the observed DFG (`dfg_fitness`).
    pub fitness: f64,
    /// Fraction of observed DFG arcs present in `reference_arcs` (`dfg_precision`).
    pub precision: f64,
    /// All directly-follows arcs actually present in the observed DFG.
    pub observed_arcs: Vec<(String, String)>,
    /// Reference arcs that never occurred in the observed DFG.
    pub missing_from_log: Vec<(String, String)>,
    /// Observed arcs that are not part of the reference arc set.
    pub unexpected_in_log: Vec<(String, String)>,
}

/// Compute a conformance report for `ocel`'s discovered lifecycle DFG against
/// `reference_arcs`.
///
/// `fitness`/`precision` delegate to `wasm4pm_compat::dfg::dfg_fitness` /
/// `dfg_precision`; `missing_from_log`/`unexpected_in_log` are a plain set
/// difference between the observed DFG's edges and `reference_arcs`.
pub fn conformance_report(ocel: &OCEL, reference_arcs: &[(String, String)]) -> ConformanceSummary {
    let dfg = discover_ocel_dfg(ocel);

    let fitness = dfg_fitness(&dfg, reference_arcs);
    let precision = dfg_precision(&dfg, reference_arcs);

    let observed_arcs: Vec<(String, String)> = dfg
        .edges
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();

    let missing_from_log: Vec<(String, String)> = reference_arcs
        .iter()
        .filter(|arc| !observed_arcs.contains(arc))
        .cloned()
        .collect();

    let unexpected_in_log: Vec<(String, String)> = observed_arcs
        .iter()
        .filter(|arc| !reference_arcs.contains(arc))
        .cloned()
        .collect();

    ConformanceSummary {
        fitness,
        precision,
        observed_arcs,
        missing_from_log,
        unexpected_in_log,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};
    use wasm4pm_compat::ocel::{OCELEvent, OCELObject, OCELRelationship, OCELType, OCEL};

    /// Build a tiny synthetic OCEL log: one object (`Repository` `repo-1`)
    /// touched, in order, by `Discover` -> `Admit` -> `Apply` events. This
    /// gives us a real observed arc set of `{Discover->Admit, Admit->Apply}`.
    fn synthetic_ocel() -> OCEL {
        let tz = FixedOffset::east_opt(0).expect("valid fixed offset");
        let t = |secs: i64| tz.timestamp_opt(1_700_000_000 + secs, 0).unwrap();

        let event = |id: &str, event_type: &str, secs: i64| OCELEvent {
            id: id.to_string(),
            event_type: event_type.to_string(),
            time: t(secs),
            attributes: vec![],
            relationships: vec![OCELRelationship {
                object_id: "repo-1".to_string(),
                qualifier: "acts_on".to_string(),
            }],
        };

        OCEL {
            event_types: vec![
                OCELType {
                    name: "Discover".to_string(),
                    attributes: vec![],
                },
                OCELType {
                    name: "Admit".to_string(),
                    attributes: vec![],
                },
                OCELType {
                    name: "Apply".to_string(),
                    attributes: vec![],
                },
            ],
            object_types: vec![OCELType {
                name: "Repository".to_string(),
                attributes: vec![],
            }],
            events: vec![
                event("e1", "Discover", 0),
                event("e2", "Admit", 10),
                event("e3", "Apply", 20),
            ],
            objects: vec![OCELObject {
                id: "repo-1".to_string(),
                object_type: "Repository".to_string(),
                attributes: vec![],
                relationships: vec![],
            }],
        }
    }

    #[test]
    fn discover_lifecycle_mines_real_directly_follows_arcs() {
        let ocel = synthetic_ocel();
        let dfg = discover_lifecycle(&ocel);

        let arcs: Vec<(String, String)> = dfg
            .edges
            .iter()
            .map(|e| (e.source.clone(), e.target.clone()))
            .collect();

        assert!(
            arcs.contains(&("Discover".to_string(), "Admit".to_string())),
            "expected Discover->Admit in {arcs:?}"
        );
        assert!(
            arcs.contains(&("Admit".to_string(), "Apply".to_string())),
            "expected Admit->Apply in {arcs:?}"
        );
        assert_eq!(dfg.start_activities, vec!["Discover".to_string()]);
        assert_eq!(dfg.end_activities, vec!["Apply".to_string()]);
    }

    #[test]
    fn conformance_report_hits_present_and_missing_arc_cases() {
        let ocel = synthetic_ocel();

        // Reference set: one arc the log actually contains (Discover->Admit),
        // and one arc the log does not contain (Admit->Validate) -- Validate
        // never occurs in this synthetic log at all.
        let reference_arcs = vec![
            ("Discover".to_string(), "Admit".to_string()),
            ("Admit".to_string(), "Validate".to_string()),
        ];

        let report = conformance_report(&ocel, &reference_arcs);

        // Present-arc case: exactly 1 of 2 reference arcs found -> fitness 0.5.
        assert_eq!(report.fitness, 0.5);

        // Observed DFG has 2 edges (Discover->Admit, Admit->Apply); exactly
        // one of them (Discover->Admit) is in the reference set -> precision 0.5.
        assert_eq!(report.precision, 0.5);

        assert!(report
            .observed_arcs
            .contains(&("Discover".to_string(), "Admit".to_string())));
        assert!(report
            .observed_arcs
            .contains(&("Admit".to_string(), "Apply".to_string())));

        // Missing-arc case: Admit->Validate never observed.
        assert_eq!(
            report.missing_from_log,
            vec![("Admit".to_string(), "Validate".to_string())]
        );

        // Unexpected-arc case: Admit->Apply is observed but not in reference_arcs.
        assert_eq!(
            report.unexpected_in_log,
            vec![("Admit".to_string(), "Apply".to_string())]
        );
    }

    #[test]
    fn full_lifecycle_reference_is_non_conformant_by_design() {
        // Documented negative comparison: the BRCE pipeline vocabulary never
        // appears in a retrofit OCEL log, so fitness against it is 0.0.
        let ocel = synthetic_ocel();
        let reference_arcs = reference_arcs_full_lifecycle();

        let report = conformance_report(&ocel, &reference_arcs);

        assert_eq!(report.fitness, 0.0);
        assert_eq!(report.missing_from_log.len(), reference_arcs.len());
    }
}
