//! Artifact pipeline for the cng CLI: import PDDL Turtle planning artifacts
//! from a directory, structurally merge the parsed fragments, and plan once.
//!
//! Artifact boundary: Turtle enters only from `.ttl` files on disk (parsed
//! with oxigraph; PDDL text is carried in `ceng:pddlDomain` /
//! `ceng:pddlProblem` literals) and leaves only as the exported POWL `.ttl`.
//! Source identity is content-addressed: every imported artifact gets a
//! `urn:blake3:<hex>` IRI, and every merged action remembers which artifact
//! contributed it, so the generated workflow can carry per-element
//! provenance. Merge law (mirrors `ChatmanEngine::plan_tape_for_snapshots`):
//! all domain fragments must declare the same domain name and every problem
//! fragment must target it; predicates, objects, init atoms, and goal atoms
//! are deduplicated unions in first-seen order; actions append in artifact
//! order with duplicate names refused; the merged goal is the conjunction of
//! every fragment goal. Artifacts are visited in lexicographic file order,
//! so the pipeline is deterministic for a fixed artifact set.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use bcinr_pddl::ground::GroundProblem;
use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};
use bcinr_pddl::{Pddl8Domain, Pddl8Problem, Pddl8Tape};
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::Term;
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::powl::CngRefusal;

const PDDL_DOMAIN_PREDICATE: &str = "urn:chatman:engine#pddlDomain";
const PDDL_PROBLEM_PREDICATE: &str = "urn:chatman:engine#pddlProblem";

/// One imported planning artifact: its canonical path, content-addressed
/// source IRI, and the PDDL literals it carries (either or both may be
/// absent).
pub struct ImportedArtifact {
    pub path: PathBuf,
    /// Content-addressed identity: `urn:blake3:<hex of file bytes>` —
    /// machine-independent, so provenance in the generated POWL is stable.
    pub source_iri: String,
    /// BLAKE3 hex digest of the artifact bytes.
    pub digest: String,
    pub domain_text: Option<String>,
    pub problem_text: Option<String>,
}

/// The admitted planning surface: the structurally merged domain/problem
/// pair plus, for each merged action name, the source IRI of the artifact
/// that contributed it.
pub struct AdmittedSurface {
    pub domain: Pddl8Domain,
    pub problem: Pddl8Problem,
    /// action name → contributing artifact's `urn:blake3:` IRI.
    pub action_sources: BTreeMap<String, String>,
}

/// Imports every `*.ttl` artifact under `dir` (non-recursive, lexicographic
/// order): parses each as Turtle and selects its deterministically smallest
/// `ceng:pddlDomain` / `ceng:pddlProblem` literal, mirroring the engine's
/// `select_literal` semantics.
///
/// # Errors
/// `CNG_R10 IoRefused` for unreadable directories/files; `CNG_R01
/// MalformedTtl` for Turtle parse failures or non-literal PDDL predicate
/// objects; `CNG_R02 MissingDomain` when the directory holds no `.ttl`
/// artifact at all.
///
/// # Complexity
/// O(files log files) for the ordering plus parser cost per artifact.
pub fn import_artifacts(dir: &Path) -> Result<Vec<ImportedArtifact>, CngRefusal> {
    let entries = fs::read_dir(dir).map_err(|e| {
        CngRefusal::IoRefused(format!("cannot read artifact dir {}: {e}", dir.display()))
    })?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|e| CngRefusal::IoRefused(format!("cannot list {}: {e}", dir.display())))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ttl") {
            paths.push(path);
        }
    }
    if paths.is_empty() {
        return Err(CngRefusal::MissingDomain(format!(
            "no .ttl planning artifacts found in {} (nothing to admit)",
            dir.display()
        )));
    }
    paths.sort();

    let mut artifacts = Vec::with_capacity(paths.len());
    for path in paths {
        let turtle = fs::read_to_string(&path)
            .map_err(|e| CngRefusal::IoRefused(format!("cannot read {}: {e}", path.display())))?;
        let digest = blake3::hash(turtle.as_bytes()).to_hex().to_string();
        let store = Store::new().map_err(|e| {
            CngRefusal::IoRefused(format!("oxigraph store construction failed: {e}"))
        })?;
        store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
            .map_err(|e| {
                CngRefusal::MalformedTtl(format!("{}: Turtle parse failed: {e}", path.display()))
            })?;
        let domain_text = select_literal(&store, PDDL_DOMAIN_PREDICATE, &path)?;
        let problem_text = select_literal(&store, PDDL_PROBLEM_PREDICATE, &path)?;
        let path = path.canonicalize().map_err(|e| {
            CngRefusal::IoRefused(format!("cannot canonicalize {}: {e}", path.display()))
        })?;
        artifacts.push(ImportedArtifact {
            path,
            source_iri: format!("urn:blake3:{digest}"),
            digest,
            domain_text,
            problem_text,
        });
    }
    Ok(artifacts)
}

/// Selects the deterministically smallest literal value of `predicate` in
/// the store's default graph, or `None` when absent (mirrors the engine's
/// `select_literal`: collected and sorted so store iteration order never
/// leaks into output).
///
/// # Complexity
/// O(m log m) over matches.
fn select_literal(
    store: &Store,
    predicate: &str,
    path: &Path,
) -> Result<Option<String>, CngRefusal> {
    const Q_SELECT_LITERAL: &str = include_str!("queries/select-literal.rq");
    let query = Q_SELECT_LITERAL.replace("{PREDICATE}", predicate);
    let prepared = SparqlEvaluator::new().parse_query(&query).map_err(|e| {
        CngRefusal::MalformedTtl(format!("SELECT parse failed for <{predicate}>: {e}"))
    })?;
    let results = prepared
        .on_store(store)
        .execute()
        .map_err(|e| CngRefusal::MalformedTtl(format!("SELECT failed for <{predicate}>: {e}")))?;
    let mut values: Vec<String> = Vec::new();
    match results {
        QueryResults::Solutions(solutions) => {
            for solution in solutions {
                let solution = solution.map_err(|e| {
                    CngRefusal::MalformedTtl(format!(
                        "SELECT evaluation failed for <{predicate}>: {e}"
                    ))
                })?;
                match solution.get("v") {
                    Some(Term::Literal(literal)) => values.push(literal.value().to_string()),
                    Some(other) => {
                        return Err(CngRefusal::MalformedTtl(format!(
                            "{}: <{predicate}> must point at a text literal, found {other}",
                            path.display()
                        )))
                    }
                    None => {}
                }
            }
        }
        _ => {
            return Err(CngRefusal::MalformedTtl(format!(
                "SELECT for <{predicate}> did not yield solutions"
            )))
        }
    }
    values.sort();
    Ok(values.into_iter().next())
}

/// Structurally merges parsed fragments and plans once, returning the
/// combined plan tape plus the admitted surface it came from.
///
/// # Errors
/// See `merge_imported`; additionally `CNG_R05 UnsupportedConstruct` for
/// grounding failures and `CNG_R04 PlanUnsolvable` when no plan exists.
///
/// # Complexity
/// O(a log a) merge; grounding bounded by `PDDL8_MAX_GROUND`; bounded-BFS
/// plan search (`PDDL8_MAX_PLAN_DEPTH`).
pub fn generate_plan(
    artifacts: &[ImportedArtifact],
) -> Result<(Pddl8Tape, AdmittedSurface), CngRefusal> {
    let surface = merge_imported(artifacts)?;
    let ground = GroundProblem::build(&surface.domain, &surface.problem, None)
        .map_err(|e| CngRefusal::UnsupportedConstruct(format!("grounding failed: {e}")))?;
    let tape = ground
        .find_plan().into_result()
        .map_err(|e| CngRefusal::PlanUnsolvable(format!("no admitted plan: {e}")))?;
    Ok((tape, surface))
}

/// Parses every imported fragment and structurally merges them into the one
/// admitted planning surface (the admission step, without planning).
///
/// # Errors
/// `CNG_R01 MalformedTtl` for PDDL parse failures; `CNG_R02/`CNG_R03` when
/// no domain/problem fragment exists; `CNG_R05 UnsupportedConstruct` for
/// mismatched domain names or duplicate action names.
///
/// # Complexity
/// Parser cost per fragment plus O(a log a) merge.
pub fn merge_imported(artifacts: &[ImportedArtifact]) -> Result<AdmittedSurface, CngRefusal> {
    let mut domains: Vec<(Pddl8Domain, String)> = Vec::new();
    let mut problems: Vec<Pddl8Problem> = Vec::new();
    for artifact in artifacts {
        if let Some(text) = &artifact.domain_text {
            let domain = domain_from_pddl(text).map_err(|e| {
                CngRefusal::MalformedTtl(format!(
                    "{}: PDDL domain parse failed: {e}",
                    artifact.path.display()
                ))
            })?;
            domains.push((domain, artifact.source_iri.clone()));
        }
        if let Some(text) = &artifact.problem_text {
            problems.push(problem_from_pddl(text).map_err(|e| {
                CngRefusal::MalformedTtl(format!(
                    "{}: PDDL problem parse failed: {e}",
                    artifact.path.display()
                ))
            })?);
        }
    }
    merge_fragments(domains, problems)
}

/// Increment-1 entry point for the hierarchical (8→8²) projection: thin
/// wrapper over `powl::project_tape_to_powl_hierarchical`, feeding it the
/// admitted surface's `action_sources` provenance map. Returns the nested
/// two-level POWL model plus one source IRI per phase for
/// `powl_to_turtle_with_phase_provenance`.
///
/// # Errors
/// See `project_tape_to_powl_hierarchical` (`CNG_R04` empty tape, `CNG_R09`
/// untracked action).
///
/// # Complexity
/// O(n log a) grouping over n ops and a admitted actions, plus O(n²) for
/// the closed order relations.
pub fn hierarchical_projection(
    tape: &Pddl8Tape,
    surface: &AdmittedSurface,
) -> Result<(crate::powl::Powl, Vec<String>), CngRefusal> {
    crate::powl::project_tape_to_powl_hierarchical(tape, &surface.action_sources)
}

/// BLAKE3 plan identifier over the ordered plan-step labels.
///
/// # Complexity
/// O(label bytes).
pub fn plan_id(tape: &Pddl8Tape) -> String {
    let labels: Vec<&str> = tape.ops.iter().map(|op| op.label.as_str()).collect();
    format!(
        "blake3:{}",
        blake3::hash(labels.join("\n").as_bytes()).to_hex()
    )
}

/// Per-leaf source IRIs for the projected model: leaf i (tape op i) maps to
/// the `urn:blake3:` IRI of the artifact whose domain fragment contributed
/// its action schema.
///
/// # Errors
/// `CNG_R09 HardcodingSuspicion` if a plan op's action is not found in the
/// admitted surface — the plan would be detached from its sources.
///
/// # Complexity
/// O(n log a) over n ops and a admitted actions.
pub fn leaf_sources(
    tape: &Pddl8Tape,
    surface: &AdmittedSurface,
) -> Result<Vec<String>, CngRefusal> {
    tape.ops
        .iter()
        .map(|op| {
            surface
                .action_sources
                .get(&op.action.schema_name)
                .cloned()
                .ok_or_else(|| {
                    CngRefusal::HardcodingSuspicion(format!(
                        "plan op {:?} has no contributing source artifact in the \
                         admitted surface; output would be detached from its inputs",
                        op.action.schema_name
                    ))
                })
        })
        .collect()
}

/// Structural fragment merge — admitted composition at the parsed-artifact
/// level, never text concatenation. Deterministic in fragment order.
///
/// # Errors
/// See `merge_imported`.
///
/// # Complexity
/// O(a log a) over total actions, predicates, objects, and atoms.
fn merge_fragments(
    domains: Vec<(Pddl8Domain, String)>,
    problems: Vec<Pddl8Problem>,
) -> Result<AdmittedSurface, CngRefusal> {
    let Some((first_domain, _)) = domains.first() else {
        return Err(CngRefusal::MissingDomain(format!(
            "no PDDL domain fragment literal at <{PDDL_DOMAIN_PREDICATE}> in any artifact"
        )));
    };
    if problems.is_empty() {
        return Err(CngRefusal::MissingProblem(format!(
            "no PDDL problem fragment literal at <{PDDL_PROBLEM_PREDICATE}> in any artifact"
        )));
    }
    let domain_name = first_domain.name.clone();

    let mut merged_domain = Pddl8Domain {
        name: domain_name.clone(),
        predicates: Vec::new(),
        actions: Vec::new(),
        types: Vec::new(),
        functions: Vec::new(),
        durative_actions: Vec::new(),
        derived: Vec::new(),
        constraints: Vec::new(),
        events: Vec::new(),
        processes: Vec::new(),
    };
    let mut action_sources: BTreeMap<String, String> = BTreeMap::new();
    let mut seen_predicates: BTreeSet<(String, u8)> = BTreeSet::new();
    for (fragment, source_iri) in domains {
        if fragment.name != domain_name {
            return Err(CngRefusal::UnsupportedConstruct(format!(
                "domain fragment name mismatch: expected {domain_name:?}, found {:?}",
                fragment.name
            )));
        }
        for predicate in fragment.predicates {
            if seen_predicates.insert(predicate.clone()) {
                merged_domain.predicates.push(predicate);
            }
        }
        for action in fragment.actions {
            if action_sources
                .insert(action.name.clone(), source_iri.clone())
                .is_some()
            {
                return Err(CngRefusal::UnsupportedConstruct(format!(
                    "duplicate PDDL action name {:?} across domain fragments",
                    action.name
                )));
            }
            merged_domain.actions.push(action);
        }
        merged_domain.types.extend(fragment.types);
        merged_domain.functions.extend(fragment.functions);
        merged_domain
            .durative_actions
            .extend(fragment.durative_actions);
        merged_domain.derived.extend(fragment.derived);
        merged_domain.constraints.extend(fragment.constraints);
        merged_domain.events.extend(fragment.events);
        merged_domain.processes.extend(fragment.processes);
    }

    let mut merged_problem = Pddl8Problem {
        name: format!("{domain_name}-combined"),
        domain: domain_name.clone(),
        objects: Vec::new(),
        init: Vec::new(),
        goal: Vec::new(),
        object_types: Vec::new(),
        fn_values: Vec::new(),
        metric: None,
        timed_inits: Vec::new(),
        preferences: Vec::new(),
    };
    let mut seen_objects: BTreeSet<String> = BTreeSet::new();
    let mut seen_init: BTreeSet<String> = BTreeSet::new();
    let mut seen_goal: BTreeSet<String> = BTreeSet::new();
    for fragment in problems {
        if fragment.domain != domain_name {
            return Err(CngRefusal::UnsupportedConstruct(format!(
                "problem fragment {:?} targets domain {:?}, expected {domain_name:?}",
                fragment.name, fragment.domain
            )));
        }
        for object in fragment.objects {
            if seen_objects.insert(object.clone()) {
                merged_problem.objects.push(object);
            }
        }
        // Atoms keyed by debug rendering: total and injective over
        // (predicate, args).
        for atom in fragment.init {
            if seen_init.insert(format!("{atom:?}")) {
                merged_problem.init.push(atom);
            }
        }
        for atom in fragment.goal {
            if seen_goal.insert(format!("{atom:?}")) {
                merged_problem.goal.push(atom);
            }
        }
        merged_problem.object_types.extend(fragment.object_types);
        merged_problem.fn_values.extend(fragment.fn_values);
        merged_problem.timed_inits.extend(fragment.timed_inits);
        merged_problem.preferences.extend(fragment.preferences);
    }
    Ok(AdmittedSurface {
        domain: merged_domain,
        problem: merged_problem,
        action_sources,
    })
}
