//! `mfg` verb dispatcher — pddl, facts, validate.
//!
//! Thin CLI wrappers over [`my_conforming_project::mfg`]: manufacture PDDL8
//! domain/problem text from a `pddl:` RDF ontology, project SPARQL facts as
//! JSON, and round-trip manufactured (or hand-written) PDDL8 text through
//! `bcinr-pddl`'s parser/grounder/planner.
//!
//! All subcommands read the ontology/PDDL text from disk; malformed paths or
//! I/O failures are a hard `Err`. A domain that fails to parse, ground, or
//! solve is `Ok(json)` with `parsed`/`solvable` fields describing why —
//! matching the rest of the CLI's "domain denial is `Ok`" convention.

use clap_noun_verb::error::{NounVerbError, Result};
use clap_noun_verb_macros::verb;
use my_conforming_project::mfg;
use serde_json::{json, Value};

fn arg_err<E: std::fmt::Display>(e: E) -> NounVerbError {
    NounVerbError::argument_error(e.to_string())
}

fn read_file(path: &str) -> std::result::Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))
}

/// Manufacture PDDL8 domain + problem text from a `pddl:` Turtle ontology.
///
/// Writes to `--domain-out`/`--problem-out` when given; otherwise returns
/// both texts (and the source graph's BLAKE3 hash) as JSON.
#[verb]
pub fn pddl(
    #[arg(help = "Path to the pddl: Turtle ontology file")] ontology: String,
    #[arg(help = "Write the domain PDDL text to this path instead of returning it")]
    domain_out: Option<String>,
    #[arg(help = "Write the problem PDDL text to this path instead of returning it")]
    problem_out: Option<String>,
) -> Result<Value> {
    let ttl = read_file(&ontology).map_err(arg_err)?;
    let manufactured = mfg::manufacture(&ttl, &ontology).map_err(|e| arg_err(e.to_string()))?;

    if let Some(path) = &domain_out {
        std::fs::write(path, &manufactured.project_domain_text()).map_err(arg_err)?;
    }
    if let Some(path) = &problem_out {
        std::fs::write(path, &manufactured.project_problem_text()).map_err(arg_err)?;
    }

    Ok(json!({
        "ontology": ontology,
        "graph_hash": manufactured.receipt.graph_hash,
        "domain": manufactured.project_domain_text(),
        "problem": manufactured.project_problem_text(),
        "domain_out": domain_out,
        "problem_out": problem_out,
    }))
}

/// Run a SPARQL `SELECT` query over a `pddl:` ontology and return the result
/// as a JSON array of objects (`ggen-core`'s `sparql_column`/`sparql_row`
/// row shape).
#[verb]
pub fn facts(
    #[arg(help = "Path to the pddl: Turtle ontology file")] ontology: String,
    #[arg(help = "SPARQL SELECT query text")] query: String,
) -> Result<Value> {
    let ttl = read_file(&ontology).map_err(arg_err)?;
    let graph = mfg::load_graph(&ttl).map_err(|e| arg_err(e.to_string()))?;
    let rows = mfg::facts_json(&graph, &query).map_err(|e| arg_err(e.to_string()))?;
    Ok(json!({ "ontology": ontology, "rows": rows }))
}

/// Round-trip PDDL8 domain/problem text through `bcinr-pddl`: parse, ground,
/// and attempt `find_plan`. Reports `parsed`/`solvable` rather than erroring
/// on a domain that fails to parse or ground — only bad file paths are
/// a hard `Err`.
#[verb]
pub fn validate(
    #[arg(help = "Path to the domain PDDL text file")] domain: String,
    #[arg(help = "Path to the problem PDDL text file")] problem: String,
) -> Result<Value> {
    let domain_text = read_file(&domain).map_err(arg_err)?;
    let problem_text = read_file(&problem).map_err(arg_err)?;
    let report = mfg::validate(&domain_text, &problem_text);
    serde_json::to_value(report).map_err(arg_err)
}
