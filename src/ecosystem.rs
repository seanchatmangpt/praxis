//! Chatman ecosystem composition contract.
//!
//! The source of truth is the RDF profile in
//! `packs/chatman-ecosystem-pack/ontology.ttl`. This module does not execute
//! any ecosystem component. It verifies the embedded profile contains the
//! required identity, stage, authority, receipt, and replay boundaries and
//! returns evidence about that bounded verification.

use serde::Serialize;
use thiserror::Error;

/// Public-ontology/ABox source embedded into the binary so contract drift is
/// visible at compile time and verification never depends on the working
/// directory.
pub const ECOSYSTEM_ONTOLOGY: &str =
    include_str!("../packs/chatman-ecosystem-pack/ontology.ttl");

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

/// Evidence returned after the embedded profile passes its structural guard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EcosystemContract {
    /// Contract schema revision.
    pub schema_version: &'static str,
    /// Embedded ontology path relative to the repository root.
    pub source: &'static str,
    /// BLAKE3 identity of the exact embedded ontology bytes.
    pub ontology_blake3: String,
    /// Ordered stage labels. Ordering is a contract, not an execution claim.
    pub stages: &'static [&'static str],
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

/// Ordered calculus. A preceding stage passing never implies a later stage
/// executed or changed state.
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
/// deliberately does not invoke ggen, Lean, mfact, BRCE, or GymAct, so those
/// external systems remain `UNKNOWN` until separately executed and receipted.
pub fn verify_contract() -> Result<EcosystemContract, EcosystemContractError> {
    for (marker, meaning) in REQUIRED_MARKERS {
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
        schema_version: "1.0.0",
        source: "packs/chatman-ecosystem-pack/ontology.ttl",
        ontology_blake3: blake3::hash(ECOSYSTEM_ONTOLOGY.as_bytes())
            .to_hex()
            .to_string(),
        stages: STAGES,
        systems: SYSTEMS,
        actuation_law: "external/world-changing DO requires BRCE authority; zero unreceipted actuation",
        external_standing: EXTERNAL_STANDING,
    })
}

#[cfg(test)]
mod tests {
    use super::{verify_contract, EcosystemContractError, EXTERNAL_STANDING, STAGES, SYSTEMS};

    #[test]
    fn embedded_profile_preserves_chatman_boundaries() -> Result<(), EcosystemContractError> {
        let contract = verify_contract()?;
        assert_eq!(contract.stages, STAGES);
        assert_eq!(contract.systems, SYSTEMS);
        assert_eq!(contract.external_standing, EXTERNAL_STANDING);
        assert_eq!(contract.ontology_blake3.len(), 64);
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
