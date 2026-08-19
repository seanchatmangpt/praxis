//! `ecosystem` verb — inspect and structurally verify the Chatman ecosystem
//! composition contract without executing or crowning any sibling runtime.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use my_conforming_project::ecosystem;
use serde_json::{json, Value};

/// Return the complete admitted composition contract as JSON.
#[verb]
pub fn contract() -> Result<Value> {
    let contract =
        ecosystem::verify_contract().map_err(|e| NounVerbError::argument_error(e.to_string()))?;
    serde_json::to_value(contract).map_err(|e| NounVerbError::argument_error(e.to_string()))
}

/// Return the owner-wide project universe and the admission/authority rules
/// that apply to every current and future project in that scope.
#[verb]
pub fn projects() -> Result<Value> {
    let contract =
        ecosystem::verify_contract().map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    Ok(json!({
        "scope": contract.project_scope,
        "stages": contract.project_stages,
        "candidate_rule": "every repository observed under the authorized seanchatmangpt/* scope is eligible for SELECT; no repository is excluded by name",
        "construction_rule": "a discovered project requires explicit project admission before executable capability composition",
        "execution_refusal": "PROJECT_EXECUTION_UNADMITTED_REFUSED",
        "consequential_do": "BRCE_ONLY_WITH_RECEIPT",
        "observed": ["embedded owner-wide project-universe contract"],
        "executed": ["structural project-universe verification"],
        "not_executed": [
            "live GitHub repository discovery",
            "repository checkout/materialization",
            "project build or test",
            "project actuation"
        ],
        "standing": contract.project_scope.standing,
    }))
}

/// Verify the embedded public-ontology profile and report exactly what this
/// check did and did not execute.
#[verb]
pub fn verify() -> Result<Value> {
    let contract =
        ecosystem::verify_contract().map_err(|e| NounVerbError::argument_error(e.to_string()))?;

    Ok(json!({
        "contract": {
            "source": contract.source,
            "ontology_blake3": contract.ontology_blake3,
            "structurally_admitted": true,
        },
        "project_universe": {
            "scope": contract.project_scope.repository_glob,
            "disposition": contract.project_scope.planning,
            "standing": contract.project_scope.standing,
        },
        "observed": [
            "embedded Chatman ecosystem RDF profile",
            "owner-wide project-universe rule",
            "canonical component identities",
            "ordered project and lifecycle markers",
            "BRCE authority edge",
            "receipt and replay edges"
        ],
        "executed": ["structural contract verification"],
        "not_executed": [
            "live GitHub repository discovery",
            "project runtimes",
            "ggen",
            "Lean 4",
            "mfact",
            "BRCE",
            "GymAct"
        ],
        "external_standing": contract.external_standing,
        "actuation_law": contract.actuation_law,
    }))
}
