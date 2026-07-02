//! `mission` verb — one mission language above the substrate, for every pack.
//!
//! RevTAC generalized (Genesis Day 6 phase 2). Where `propose mission`
//! compiled a *revenue* mission, this noun runs a mission for **any**
//! institution by name:
//!
//! ```text
//! mission run     --pack <revenue|church> --objective <path> --state <path>
//! mission ceiling --pack <revenue|church> --state <path>
//! ```
//!
//! `run` drives the full observation→proposal→plan→admission→receipt pipe via
//! the domain-independent [`my_conforming_project::mission::run_pipeline`] —
//! the *same* substrate functions for revenue and church, selected only by
//! `--pack`. `ceiling` computes the pack's Maximum Reachable objective (MRR
//! generalized) via [`my_conforming_project::mission::ceiling`].
//!
//! # AR-9 — output is proposal (O), not authority (O*)
//!
//! Everything these verbs emit is an untrusted observation. `run`'s compiled
//! goal and its receipt bind an admitted proposal's `proposal_hash`, but the
//! authority lives in the admission gate the pipe calls (`law judge`/
//! `law admit`), never in the mission document. `ceiling` is a physical bound
//! on lawful mission value, not an instruction to reach it.
//!
//! # No value discovery (Non-goal 1)
//!
//! The objective is authored data supplied by `--objective` (a file path).
//! Each pack validates it against *its own* fluent vocabulary; an objective
//! with unknown fluents or non-finite weights is a hard `Err`.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::{arg, verb};
use my_conforming_project::mission::{self, Pack};
use praxis_proposer::{ChurchDomain, ChurchState, RevenueDomain, RevenueState};
use serde_json::Value;

/// Read a required file path, mapping I/O failure to a labeled error.
fn read_file(flag: &str, path: &str) -> std::result::Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read --{flag} '{path}': {e}"))
}

/// Run the generic pipeline for the named pack. The `match` on `--pack` is the
/// only place the institution is named; both arms call the identical
/// `mission::run_pipeline::<P>` — one substrate, two packs.
fn run_for_pack(
    pack: &str,
    objective_path: &str,
    state_path: &str,
    mission_name: &str,
    ts_ns: u64,
) -> std::result::Result<Value, String> {
    let objective_text = read_file("objective", objective_path)?;
    let state_text = read_file("state", state_path)?;
    match pack {
        "revenue" => {
            let state: RevenueState = serde_json::from_str(&state_text)
                .map_err(|e| format!("invalid revenue state JSON: {e}"))?;
            let objective = RevenueDomain::load_objective(&objective_text)?;
            mission::run_pipeline::<RevenueDomain>(&state, &objective, mission_name, ts_ns)
        }
        "church" => {
            let state: ChurchState = serde_json::from_str(&state_text)
                .map_err(|e| format!("invalid church state JSON: {e}"))?;
            let objective = ChurchDomain::load_objective(&objective_text)?;
            mission::run_pipeline::<ChurchDomain>(&state, &objective, mission_name, ts_ns)
        }
        other => Err(format!("unknown pack '{other}': expected 'revenue' or 'church'")),
    }
}

/// Compute the Maximum Reachable objective for the named pack. Again the pack
/// name is the only difference; both arms call `mission::ceiling::<P>`.
fn ceiling_for_pack(pack: &str, state_path: &str) -> std::result::Result<Value, String> {
    let state_text = read_file("state", state_path)?;
    match pack {
        "revenue" => {
            let state: RevenueState = serde_json::from_str(&state_text)
                .map_err(|e| format!("invalid revenue state JSON: {e}"))?;
            Ok(mission::ceiling::<RevenueDomain>(&state))
        }
        "church" => {
            let state: ChurchState = serde_json::from_str(&state_text)
                .map_err(|e| format!("invalid church state JSON: {e}"))?;
            Ok(mission::ceiling::<ChurchDomain>(&state))
        }
        other => Err(format!("unknown pack '{other}': expected 'revenue' or 'church'")),
    }
}

/// Compile and run a mission for one pack: observe → propose → goal →
/// `plan solve` → `law judge`/`law admit` → `law receipt`. The pipe body is
/// pack-independent (see `mission::run_pipeline`); `--pack` only selects which
/// ontology + objective vocabulary to instantiate it at. Output is proposal
/// (O), not authority (O*): every step still passes admission (AR-9).
#[verb]
pub fn run(
    #[arg(help = "Pack to run: revenue or church")] pack: String,
    #[arg(help = "Path to the domain-authored objective JSON file")] objective: String,
    #[arg(help = "Path to the observed state JSON snapshot")] state: String,
    #[arg(
        default_value = "mission",
        help = "Free-form mission intent name bound into the receipt"
    )]
    mission: String,
    #[arg(
        default_value = "0",
        help = "Fixed receipt timestamp (ns) for a reproducible chain hash; 0 = wall clock"
    )]
    ts_ns: u64,
) -> Result<Value> {
    run_for_pack(&pack, &objective, &state, &mission, ts_ns).map_err(NounVerbError::argument_error)
}

/// Compute a pack's Maximum Reachable objective — MRR generalized. For revenue
/// this is Maximum Reachable Revenue; for church it is the ceiling of people
/// lawfully connectable and cared for. Objective-independent physics that
/// respects the pack's evidence gates. Output is observation (O): a bound, not
/// an instruction.
#[verb]
pub fn ceiling(
    #[arg(help = "Pack to compute the ceiling for: revenue or church")] pack: String,
    #[arg(help = "Path to the observed state JSON snapshot")] state: String,
) -> Result<Value> {
    ceiling_for_pack(&pack, &state).map_err(NounVerbError::argument_error)
}
