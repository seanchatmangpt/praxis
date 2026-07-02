//! `config` verb — inspect and validate the effective `PraxisConfig`.
//!
//! Thin wrappers over [`my_conforming_project::config`]: all admission logic
//! (layered TOML sources, validation, the preventive-gate adapter) lives
//! there so it can be reused by any future non-CLI caller.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use my_conforming_project::config as cfg;
use serde_json::{json, Value};

/// Show the effective admitted configuration as JSON, alongside its witness
/// hash and a gate-verdict summary.
#[verb]
pub fn show() -> Result<Value> {
    let admitted = cfg::load_config().map_err(|e| NounVerbError::argument_error(e.to_string()))?;
    let value = serde_json::to_value(admitted.value())
        .map_err(|e| NounVerbError::argument_error(e.to_string()))?;
    Ok(json!({
        "config": value,
        "witness": admitted.witness().hash(),
    }))
}

/// Print only the `ConfigWitness` BLAKE3 hash of the effective configuration.
#[verb]
pub fn witness() -> Result<Value> {
    let admitted = cfg::load_config().map_err(|e| NounVerbError::argument_error(e.to_string()))?;
    Ok(json!({ "witness": admitted.witness().hash() }))
}

/// Run the full admission pipeline (layers + validation + preventive gate)
/// and report whether the configuration is admitted.
#[verb]
pub fn validate() -> Result<Value> {
    match cfg::load_config() {
        Ok(admitted) => Ok(json!({
            "status": "admitted",
            "witness": admitted.witness().hash(),
        })),
        Err(e) => Err(NounVerbError::argument_error(e.to_string())),
    }
}
