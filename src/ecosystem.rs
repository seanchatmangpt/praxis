//! Chatman ecosystem composition contract.
//!
//! The source of truth is the RDF profile in
//! `packs/chatman-ecosystem-pack/ontology.ttl`. This module does not execute
//! any ecosystem component. It verifies the embedded profile contains the
//! required identity, project-universe, stage, authority, receipt, and replay
//! boundaries and returns evidence about that bounded verification.

use serde::Serialize;
use thiserror::Error;

/// Public-ontology/ABox source embedded into the binary so contract drift is
/// visible at compile time and verification never depends on the working
/// directory.
pub const ECOSYSTEM_ONTOLOGY: &str = include_str!("../packs/chatman-ecosystem-pack/ontology.ttl");

/// Standing applied to external ecosystem components by this verifier.
/// Structural verification of the profile cannot crown a sibling runtime.
pub const EXTERNAL_STANDING: &str = "UNKNOWN";

/// One externally-owned Chatman ecosystem component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EcosystemSystem {
    /// Stable display identity.
    pub name: &'static str,
    /// Canonical source repository or upstream project identity.
    pub source: &'static str,
    /// Capability this component owns in the composed pipeline.
    pub role: &'static str,
    /// Standing visible from this contract alone.
    pub standing: &'static str,
}

/// Open project universe consumed by Praxis.
///
/// The scope is intentionally owner-wide instead of a frozen repository list:
/// every project visible under the authorized GitHub observation participates
/// in discovery and may become a SELECT candidate. Discovery never grants
/// construction or execution authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EcosystemProjectScope {
    /// GitHub owner whose complete visible repository graph is in scope.
    pub owner: &'static str,
    /// Stable open-world repository selector.
    pub repository_glob: &'static str,
    /// Canonical portfolio source.
    pub source: &'static str,
    /// Observation/discovery rule.
    pub discovery: &'static str,
    /// Admission rule before a project can contribute executable capability.
    pub admission: &'static str,
    /// Planning disposition for discovered projects.
    pub planning: &'static str,
    /// Construction disposition after admission.
    pub construction: &'static str,
    /// Consequential execution disposition.
    pub actuation: &'static str,
    /// Standing visible from this structural contract alone.
    pub standing: &'static str,
}

/// Evidence returned after the embedded profile passes its structural guard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EcosystemContract {
    /// Contract schema revision.
    pub schema_version: &'static str,
    /// Embedded ontology path relative to the repository root.
    pub source: &'static str,
    /// BLAKE3 identity of the exact embedded ontology bytes.
    pub ontology_blake3: String,
    /// Ordered lifecycle labels. Ordering is a contract, not an execution claim.
    pub stages: &'static [&'static str],
    /// Ordered project-universe labels. These govern portfolio composition.
    pub project_stages: &'static [&'static str],
    /// Open owner-wide project scope used by SELECT/planning.
    pub project_scope: EcosystemProjectScope,
    /// External component identities and ownership boundaries.
    pub systems: &'static [EcosystemSystem],
    /// Explicit authority invariant for consequential external/world changes.
    pub actuation_law: &'static str,
    /// Explicit standing caveat for sibling systems.
    pub external_standing: &'static str,
}

/// Structural contract refusal. A missing marker means the checked-in profile
/// no longer proves the boundary this binary advertises.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum EcosystemContractError {
    /// A required identity or lifecycle marker is absent from the profile.
    #[error("Chatman ecosystem contract missing `{marker}` ({meaning})")]
    MissingMarker {
        /// Literal marker required in the checked-in profile.
        marker: &'static str,
        /// Boundary the marker protects.
        meaning: &'static str,
    },
}

/// Complete owner-wide project universe. A newly created repository under the
/// owner enters discovery automatically without silently entering execution.
pub const PROJECT_SCOPE: EcosystemProjectScope = EcosystemProjectScope {
    owner: "seanchatmangpt",
    repository_glob: "seanchatmangpt/*",
    source: "https://github.com/seanchatmangpt",
    discovery: "ALL_VISIBLE_REPOSITORIES",
    admission: "PROJECT_CONTRACT_REQUIRED",
    planning: "ELIGIBLE_FOR_SELECT",
    construction: "ELIGIBLE_AFTER_ADMISSION",
    actuation: "PROJECT_EXECUTION_UNADMITTED_REFUSED; admitted consequential DO remains BRCE_ONLY",
    standing: EXTERNAL_STANDING,
};

/// Canonical externally-owned systems. Praxis composes them; it does not
/// silently absorb their authority into this crate.
pub const SYSTEMS: &[EcosystemSystem] = &[
    EcosystemSystem {
        name: "ggen",
        source: "https://github.com/seanchatmangpt/ggen",
        role: "deterministic graph-backed manufacture + generation receipt",
        standing: EXTERNAL_STANDING,
    },
    EcosystemSystem {
        name: "Lean 4",
        source: "https://github.com/leanprover/lean4",
        role: "kernel admission of formal declarations",
        standing: EXTERNAL_STANDING,
    },
    EcosystemSystem {
        name: "mfact",
        source: "https://github.com/seanchatmangpt/mfact",
        role: "certification of manufactured/admitted mathematical evidence",
        standing: EXTERNAL_STANDING,
    },
    EcosystemSystem {
        name: "BRCE",
        source: "https://chatmangpt.com/id/ecosystem/brce-policy",
        role: "exclusive external/world-changing DO authority",
        standing: EXTERNAL_STANDING,
    },
    EcosystemSystem {
        name: "GymAct",
        source: "https://github.com/seanchatmangpt/gymact",
        role: "bounded benchmark-world execution adapter behind authority",
        standing: EXTERNAL_STANDING,
    },
];

/// Ordered lifecycle calculus. A preceding stage passing never implies a later
/// stage executed or changed state.
pub const STAGES: &[&str] = &[
    "OBSERVE",
    "SELECT",
    "CONSTRUCT",
    "GGEN_RENDER",
    "LEAN_ADMIT",
    "MFACT_CERTIFY",
    "HOOK_INTENT",
    "BRCE_DO",
    "RECEIPT",
    "REPLAY",
];

/// Portfolio calculus used to turn the full repository graph into lawful
/// capability candidates without ambient authority.
pub const PROJECT_STAGES: &[&str] = &["DISCOVER_PROJECTS", "ADMIT_PROJECT", "COMPOSE_PROJECT"];

const REQUIRED_MARKERS: &[(&str, &str)] = &[
    (
        "http://www.w3.org/ns/prov#",
        "PROV-O is part of the public semantic profile",
    ),
    (
        "http://purl.org/net/p-plan#",
        "P-PLAN carries ordered process steps",
    ),
    (
        "http://www.w3.org/ns/odrl/2/",
        "ODRL carries the BRCE authority policy",
    ),
    (
        "dcterms:identifier \"seanchatmangpt/*\"",
        "the complete owner-wide project universe is explicit",
    ),
    (
        "dcterms:source <https://github.com/seanchatmangpt>",
        "the project universe has an explicit canonical source",
    ),
    (
        "rdfs:label \"DISCOVER_PROJECTS\"",
        "project discovery is separate from admission",
    ),
    (
        "rdfs:label \"ADMIT_PROJECT\"",
        "project admission is explicit",
    ),
    (
        "rdfs:label \"COMPOSE_PROJECT\"",
        "project composition is explicit",
    ),
    (
        "https://github.com/seanchatmangpt/ggen",
        "canonical ggen identity is externalized",
    ),
    (
        "https://github.com/leanprover/lean4",
        "Lean kernel admission authority is explicit",
    ),
    (
        "https://github.com/seanchatmangpt/mfact",
        "canonical mfact certification identity is externalized",
    ),
    (
        "https://github.com/seanchatmangpt/gymact",
        "canonical GymAct bounded execution identity is externalized",
    ),
    (
        "dcterms:requires chatman:brce-policy",
        "external/world-changing DO has an explicit BRCE authority edge",
    ),
    (
        "Hooks manufacture intents; they never actuate.",
        "hooks do not inherit ambient execution authority",
    ),
    (
        "pplan:isPrecededBy chatman:brce-do",
        "receipt is downstream of the DO boundary",
    ),
    (
        "pplan:isPrecededBy chatman:receipt",
        "replay is downstream of a receipt",
    ),
];

/// Verify the exact embedded ecosystem profile and return bounded evidence.
///
/// This is a structural admission gate for the composition contract only. It
/// deliberately does not invoke GitHub discovery, ggen, Lean, mfact, BRCE, or
/// GymAct, so those external systems remain `UNKNOWN` until separately
/// observed/executed and receipted.
pub fn verify_contract() -> Result<EcosystemContract, EcosystemContractError> {
    for &(marker, meaning) in REQUIRED_MARKERS {
        if !ECOSYSTEM_ONTOLOGY.contains(marker) {
            return Err(EcosystemContractError::MissingMarker { marker, meaning });
        }
    }

    for stage in STAGES {
        let marker = match *stage {
            "OBSERVE" => "rdfs:label \"OBSERVE\"",
            "SELECT" => "rdfs:label \"SELECT\"",
            "CONSTRUCT" => "rdfs:label \"CONSTRUCT\"",
            "GGEN_RENDER" => "rdfs:label \"GGEN_RENDER\"",
            "LEAN_ADMIT" => "rdfs:label \"LEAN_ADMIT\"",
            "MFACT_CERTIFY" => "rdfs:label \"MFACT_CERTIFY\"",
            "HOOK_INTENT" => "rdfs:label \"HOOK_INTENT\"",
            "BRCE_DO" => "rdfs:label \"BRCE_DO\"",
            "RECEIPT" => "rdfs:label \"RECEIPT\"",
            "REPLAY" => "rdfs:label \"REPLAY\"",
            _ => continue,
        };
        if !ECOSYSTEM_ONTOLOGY.contains(marker) {
            return Err(EcosystemContractError::MissingMarker {
                marker,
                meaning: "ordered lifecycle stage",
            });
        }
    }

    Ok(EcosystemContract {
        schema_version: "1.1.0",
        source: "packs/chatman-ecosystem-pack/ontology.ttl",
        ontology_blake3: blake3::hash(ECOSYSTEM_ONTOLOGY.as_bytes())
            .to_hex()
            .to_string(),
        stages: STAGES,
        project_stages: PROJECT_STAGES,
        project_scope: PROJECT_SCOPE,
        systems: SYSTEMS,
        actuation_law:
            "external/world-changing DO requires BRCE authority; zero unreceipted actuation",
        external_standing: EXTERNAL_STANDING,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        verify_contract, EcosystemContractError, EXTERNAL_STANDING, PROJECT_SCOPE, PROJECT_STAGES,
        STAGES, SYSTEMS,
    };

    #[test]
    fn embedded_profile_preserves_chatman_boundaries() -> Result<(), EcosystemContractError> {
        let contract = verify_contract()?;
        assert_eq!(contract.stages, STAGES);
        assert_eq!(contract.project_stages, PROJECT_STAGES);
        assert_eq!(contract.project_scope, PROJECT_SCOPE);
        assert_eq!(contract.systems, SYSTEMS);
        assert_eq!(contract.external_standing, EXTERNAL_STANDING);
        assert_eq!(contract.ontology_blake3.len(), 64);
        Ok(())
    }

    #[test]
    fn project_universe_is_open_owner_wide_and_non_actuating() -> Result<(), EcosystemContractError>
    {
        let contract = verify_contract()?;
        assert_eq!(contract.project_scope.repository_glob, "seanchatmangpt/*");
        assert_eq!(contract.project_scope.discovery, "ALL_VISIBLE_REPOSITORIES");
        assert_eq!(contract.project_scope.planning, "ELIGIBLE_FOR_SELECT");
        assert!(contract.project_scope.actuation.contains("REFUSED"));
        assert_eq!(contract.project_scope.standing, "UNKNOWN");
        Ok(())
    }

    #[test]
    fn structural_admission_never_crowns_external_systems() -> Result<(), EcosystemContractError> {
        let contract = verify_contract()?;
        assert!(contract
            .systems
            .iter()
            .all(|system| system.standing == "UNKNOWN"));
        Ok(())
    }

    #[test]
    fn do_receipt_replay_are_distinct_ordered_stages() -> Result<(), EcosystemContractError> {
        let contract = verify_contract()?;
        let do_index = contract.stages.iter().position(|stage| *stage == "BRCE_DO");
        let receipt_index = contract.stages.iter().position(|stage| *stage == "RECEIPT");
        let replay_index = contract.stages.iter().position(|stage| *stage == "REPLAY");
        assert!(matches!(
            (do_index, receipt_index, replay_index),
            (Some(do_i), Some(receipt_i), Some(replay_i))
                if do_i < receipt_i && receipt_i < replay_i
        ));
        Ok(())
    }
}
