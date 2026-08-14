//! `ecosystem` verb — inspect and structurally verify the Chatman ecosystem
//! composition contract without executing or crowning any sibling runtime.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use my_conforming_project::ecosystem;
use serde_json::{json, Value};

/// Return the complete admitted composition contract as JSON.
#[verb]
pub fn contract() -> Result<Value> {
    let contract = ecosystem::verify_contract()
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;
    serde_json::to_value(contract).map_err(|e| NounVerbError::argument_error(e.to_string()))
}

/// Verify the embedded public-ontology profile and report exactly what this
/// check did and did not execute.
#[verb]
pub fn verify() -> Result<Value> {
    let contract = ecosystem::verify_contract()
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    Ok(json!({
        "contract": {
            "source": contract.source,
            "ontology_blake3": contract.ontology_blake3,
            "structurally_admitted": true,
        },
        "observed": [
            "embedded Chatman ecosystem RDF profile",
            "canonical component identities",
            "ordered lifecycle markers",
            "BRCE authority edge",
            "receipt and replay edges"
        ],
        "executed": ["structural contract verification"],
        "not_executed": ["ggen", "Lean 4", "mfact", "BRCE", "GymAct"],
        "external_standing": contract.external_standing,
        "actuation_law": contract.actuation_law,
    }))
}
