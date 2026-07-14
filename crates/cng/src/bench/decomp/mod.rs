//! No-LLM multi-actor goal decomposition (v26.7.10-revised Track P,
//! PROJ-702..710): derive goal decompositions from admitted graph state,
//! manufacture helper/main PDDL subproblems via SPARQL CONSTRUCT, plan
//! classically on `bcinr_pddl::ground::IndexedGroundProblem::find_plan`
//! (PROJ-733: relaxed-reachability-pruned grounding — same BFS strategy
//! and identical plans as `bcinr_pddl::GroundProblem`, differential-tested
//! in `tests/indexed_grounding.rs`, but without materializing the
//! never-firing ground actions that an untyped multi-actor domain's
//! full-cross-product grounding produces), prove the interface state
//! s′ = E(s, π_h) by verified replay,
//! prove non-interference + resource-release closure, compose a nested
//! POWL partial order, and select deterministically. `LLM_CALLS = 0` is
//! structural: nothing in this module tree can call one.
//!
//! Authority partition:
//! - RDF is the ledger: the lifted pddl-strips graph (PROJ-702), the
//!   CONSTRUCT-manufactured subproblems (PROJ-706), and the emitted
//!   `decomp:DecompositionResult` receipt graph (PROJ-710) are the facts.
//! - Rust is the evaluator: grounding, BFS planning, replay, and scoring
//!   run over typed structs; every claim lands back in the receipt graph.
//! - Datalog derives the edges (PROJ-704, `rules/decomp.dl`); resource
//!   classification is admitted facts (`rules/decomp-resources.dl`,
//!   `pddl:resource` annotations), never Rust constants.
//!
//! The single-actor plan is ALWAYS candidate `0-single`;
//! `NoAdmissibleDecomposition` / `NoBeneficialDecomposition` are typed
//! SUCCESS outcomes carrying the single-actor POWL — never refusals, never
//! silent fallbacks. No wall clock anywhere in this tree.

mod compose;
pub mod dispatch_bridge;
mod interface;
mod interference;
mod lift;
mod manufacture;
mod render;
mod rules;
mod search;
mod select;

pub use compose::{compose_two, composed_to_turtle};
pub use interface::replay_to_interface_state;
pub use interference::{augmentation_atoms, check_interference, check_release_closure};
pub use lift::{
    action_iri, atom_iri, domain_iri, effect_iri, lift_ground, problem_iri, safe_token,
    PDDL_STRIPS_PREFIX,
};
pub use manufacture::{manufacture_helper, manufacture_main, manufacture_provenance, values_list};
pub use render::{
    atom_contents, render_domain, render_problem, strips_graph_to_surface, AtomContent,
};
pub use rules::{derive_edges, DerivedEdges};
pub use search::{
    candidate_id, enumerate_candidates, partition_goals, Candidate, DECOMP_MAX_CANDIDATES,
    DECOMP_MAX_COMPONENTS, SINGLE_ACTOR_CANDIDATE_ID,
};
pub use select::{
    longest_path_nodes, score_single, score_split, select, CandidateReceipt, CandidateStatus,
    Score, SelectionVerdict, DISPATCH_OVERHEAD_STEPS,
};

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use bcinr_pddl::parse::problem_from_pddl;
use bcinr_pddl::{Pddl8Domain, Pddl8GroundAtom, Pddl8Problem, Pddl8Tape};
// PROJ-733: relaxed-reachability-pruned grounding (see module doc above).
// `IndexedGroundProblem`'s `build`/`find_plan` signatures and its public
// `initial_state`/`goal`/`actions` fields mirror `bcinr_pddl::ground::
// GroundProblem` exactly, so every call site below needs no other change.
use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::store::Store;
use bcinr_pddl::ground::IndexedGroundProblem as GroundProblem;

use crate::bench::templates::QuerySet;
use crate::powl::{CngRefusal, Powl};

/// Grounding cap for decomposition surfaces: untyped multi-actor domains
/// legitimately exceed the STRIPS8 default (`PDDL8_MAX_GROUND` = 4096, e.g.
/// a 4-parameter schema over 9 objects grounds to 6561 actions), so the
/// decomposition path passes an explicit, documented bound.
pub const DECOMP_MAX_GROUND: usize = 16_384;

/// Typed decomposition outcome — a RESULT serialized into the receipt
/// graph, never a refusal, never a silent fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecompositionOutcome {
    /// A split candidate won the selection law.
    Selected {
        candidate_id: String,
        subworkflows: usize,
    },
    /// Every split candidate failed a proof obligation (or no split was
    /// enumerable); the single-actor plan carries the goal.
    NoAdmissibleDecomposition { rejected: usize },
    /// At least one split was admissible but the single-actor candidate won
    /// the lexicographic argmin.
    NoBeneficialDecomposition { best_rejected_id: String },
}

impl DecompositionOutcome {
    /// Stable receipt string.
    pub fn as_str(&self) -> &'static str {
        match self {
            DecompositionOutcome::Selected { .. } => "Selected",
            DecompositionOutcome::NoAdmissibleDecomposition { .. } => "NoAdmissibleDecomposition",
            DecompositionOutcome::NoBeneficialDecomposition { .. } => "NoBeneficialDecomposition",
        }
    }
}

/// One subworkflow of the selected decomposition (the single-actor result
/// has exactly one, role `single`).
#[derive(Debug, Clone)]
pub struct SubworkflowPlan {
    /// Stable subworkflow id (candidate-scoped index).
    pub id: String,
    /// Structural decomposition role: `helper` | `main` | `single`.
    pub role: String,
    /// The planned tape (total order within the subworkflow).
    pub tape: Pddl8Tape,
    /// The subworkflow's own total-order POWL model.
    pub model: Powl,
    /// Rendered PDDL problem text provenance (empty for `single`, whose
    /// problem is the admitted source problem itself).
    pub problem_pddl: String,
    /// BLAKE3 digest of `problem_pddl` (`blake3:<hex>`).
    pub problem_digest: String,
}

/// The decomposition handoff artifact: per-subworkflow POWL + role, the
/// constraints surface (cross-workflow edges, interface atoms, release
/// obligations), the composed nested model, and the receipt trail. The
/// dispatch side consumes the RDF graph at `result_graph_path`
/// (`decomp:DecompositionResult`), never this Rust struct.
#[derive(Debug)]
pub struct DecompositionResult {
    pub outcome: DecompositionOutcome,
    pub subworkflows: Vec<SubworkflowPlan>,
    /// Cross-workflow mustPrecede edges (from ground label, to ground label).
    pub cross_edges: Vec<(String, String)>,
    /// Interface-state s′ atom labels (empty for the single-actor outcome).
    pub interface_atoms: Vec<String>,
    /// Discharged resource-release obligations (atom labels).
    pub release_obligations: Vec<String>,
    /// The composed model: nested `PartialOrder` (split) or the flat
    /// single-actor total order.
    pub composed_model: Powl,
    /// Where the emitted `decomp:DecompositionResult` Turtle graph lives.
    pub result_graph_path: PathBuf,
    /// Every examined candidate's receipt (selected + admissible +
    /// inadmissible).
    pub candidate_receipts: Vec<CandidateReceipt>,
}

/// On-disk decomposition templates (`templates/decomp-*.template.*`).
struct DecompTemplates {
    problem_pddl: String,
    receipt: String,
    result: String,
    subworkflow: String,
    interface_atom: String,
    order_edge: String,
    release_obligation: String,
}

/// Loads the decomposition templates from the crate templates directory.
///
/// # Errors
/// `CNG_R10 IoRefused` for unreadable files.
///
/// # Complexity
/// O(files).
fn load_decomp_templates() -> Result<DecompTemplates, CngRefusal> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
    let read = |name: &str| -> Result<String, CngRefusal> {
        fs::read_to_string(dir.join(name))
            .map_err(|e| CngRefusal::IoRefused(format!("cannot read template {name}: {e}")))
    };
    Ok(DecompTemplates {
        problem_pddl: read("decomp-problem.template.pddl")?,
        receipt: read("decomp-candidate-receipt.template.ttl")?,
        result: read("decomp-result.template.ttl")?,
        subworkflow: read("decomp-subworkflow.template.ttl")?,
        interface_atom: read("decomp-interface-atom.template.ttl")?,
        order_edge: read("decomp-order-edge.template.ttl")?,
        release_obligation: read("decomp-release-obligation.template.ttl")?,
    })
}

/// Default decomposition query directory: `<crate>/queries/decomp`.
pub fn decomp_queries_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("queries")
        .join("decomp")
}

/// Reads `rules/decomp.dl` + `rules/decomp-resources.dl`.
///
/// # Errors
/// `CNG_R10 IoRefused`.
fn load_rules_texts() -> Result<(String, String), CngRefusal> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules");
    let read = |name: &str| -> Result<String, CngRefusal> {
        fs::read_to_string(dir.join(name))
            .map_err(|e| CngRefusal::IoRefused(format!("cannot read rules file {name}: {e}")))
    };
    Ok((read("decomp.dl")?, read("decomp-resources.dl")?))
}

/// Sorted N-Triples serialization of a store's default graph (oxigraph
/// term `Display` per component — serializer output, no inline Turtle).
///
/// # Complexity
/// O(t log t) over t triples.
fn store_to_sorted_ntriples(store: &Store) -> Result<String, CngRefusal> {
    let mut lines: Vec<String> = Vec::new();
    for quad in store.iter() {
        let quad = quad.map_err(|e| CngRefusal::MalformedTtl(format!("result graph iter: {e}")))?;
        lines.push(format!(
            "{} {} {} .",
            quad.subject, quad.predicate, quad.object
        ));
    }
    lines.sort();
    lines.dedup();
    let mut out = lines.join("\n");
    out.push('\n');
    Ok(out)
}

/// Short deterministic slug for a candidate id (receipt IRI local names).
///
/// # Complexity
/// O(len).
fn slug(id: &str) -> String {
    blake3::hash(id.as_bytes()).to_hex()[..16].to_string()
}

/// Everything a surviving split candidate carries into selection.
struct SplitArtifacts {
    helper_tape: Pddl8Tape,
    main_tape: Pddl8Tape,
    model: Powl,
    cross_edges: Vec<(String, String)>,
    s_prime: BTreeSet<Pddl8GroundAtom>,
    released: Vec<String>,
    helper_pddl: String,
    main_pddl: String,
    helper_iri: String,
    main_iri: String,
}

/// Plans a manufactured subproblem graph: render → parse → ground → BFS.
///
/// # Errors
/// The typed refusal of whichever stage failed.
///
/// # Complexity
/// Render + parser cost + bounded grounding/BFS.
fn plan_manufactured(
    problem_store: &Store,
    problem_iri_str: &str,
    domain: &Pddl8Domain,
    source_problem: &Pddl8Problem,
    queries: &QuerySet,
    problem_template: &str,
) -> Result<(Pddl8Tape, String), CngRefusal> {
    let pddl_text =
        render::render_problem(problem_store, problem_iri_str, queries, problem_template)?;
    let parsed = problem_from_pddl(&pddl_text).map_err(|e| {
        CngRefusal::MalformedTtl(format!("manufactured problem failed to parse: {e:?}"))
    })?;
    // Preserve the source problem's typing surface (untyped here, but kept
    // structurally faithful) while taking the manufactured objects/init/goal.
    let mut problem = source_problem.clone();
    problem.name = parsed.name;
    problem.objects = parsed.objects;
    problem.object_types = parsed.object_types;
    problem.init = parsed.init;
    problem.goal = parsed.goal;
    let ground = GroundProblem::build(domain, &problem, Some(DECOMP_MAX_GROUND)).map_err(|e| {
        CngRefusal::UnsupportedConstruct(format!("subproblem grounding failed: {e}"))
    })?;
    let tape = ground
        .find_plan().into_result()
        .map_err(|e| CngRefusal::PlanUnsolvable(format!("subproblem admits no plan: {e}")))?;
    Ok((tape, pddl_text))
}

/// Cross edges mapped to composed node indices (helper ops 0..h, main ops
/// h..h+m; first occurrence per label).
///
/// # Complexity
/// O(h + m + |cross| log n).
fn cross_index_edges(
    helper: &Pddl8Tape,
    main: &Pddl8Tape,
    cross: &[(String, String)],
) -> BTreeSet<(usize, usize)> {
    let mut helper_idx: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, op) in helper.ops.iter().enumerate() {
        helper_idx.entry(op.action.label.as_str()).or_insert(i);
    }
    let mut main_idx: BTreeMap<&str, usize> = BTreeMap::new();
    for (i, op) in main.ops.iter().enumerate() {
        main_idx.entry(op.action.label.as_str()).or_insert(i);
    }
    let h = helper.ops.len();
    let mut edges = BTreeSet::new();
    for (from, to) in cross {
        match (
            helper_idx.get(from.as_str()),
            main_idx.get(to.as_str()),
            main_idx.get(from.as_str()),
            helper_idx.get(to.as_str()),
        ) {
            (Some(&f), Some(&t), _, _) => {
                edges.insert((f, h + t));
            }
            (_, _, Some(&f), Some(&t)) => {
                edges.insert((h + f, t));
            }
            _ => {}
        }
    }
    edges
}

/// Derives the decomposition for an admitted (domain, problem) surface and
/// emits the `decomp:DecompositionResult` receipt graph under `out_dir`.
/// See [`decompose_with`] for the forced-candidate variant.
///
/// # Errors
/// `CNG_R04` when the single-actor surface admits no plan (the original
/// problem is unsolvable); `CNG_R21..R24` for selected-candidate proof
/// failures; `CNG_R01/R05/R10` for graph/IO failures.
///
/// # Complexity
/// Bounded by [`DECOMP_MAX_CANDIDATES`] candidate pipelines, each grounding
/// + BFS + O(h·m) proofs.
pub fn decompose(
    domain: &Pddl8Domain,
    problem: &Pddl8Problem,
    out_dir: &Path,
    base_iri: &str,
) -> Result<DecompositionResult, CngRefusal> {
    decompose_with(domain, problem, out_dir, base_iri, None)
}

/// [`decompose`] with an optional demanded candidate id: a forced candidate
/// that is inadmissible refuses `CNG_R21` instead of falling back.
///
/// # Errors
/// See [`decompose`].
///
/// # Complexity
/// See [`decompose`].
pub fn decompose_with(
    domain: &Pddl8Domain,
    problem: &Pddl8Problem,
    out_dir: &Path,
    base_iri: &str,
    forced: Option<&str>,
) -> Result<DecompositionResult, CngRefusal> {
    let base_iri = base_iri.trim_end_matches('/');
    let templates = load_decomp_templates()?;
    let queries = QuerySet::load(&decomp_queries_dir())?;
    let (rules_text, resources_text) = load_rules_texts()?;

    // 1. Ground the admitted surface once (the unchanged planner path).
    let ground = GroundProblem::build(domain, problem, Some(DECOMP_MAX_GROUND))
        .map_err(|e| CngRefusal::UnsupportedConstruct(format!("grounding failed: {e}")))?;

    // 2. Lift into the working pddl-strips store (PROJ-702).
    let working = lift::lift_ground(
        &ground,
        &problem.objects,
        &domain.name,
        &problem.name,
        base_iri,
    )?;
    let source_problem_iri = lift::problem_iri(base_iri, &problem.name);

    // 3. Derive the decomposition edges (PROJ-704).
    let edges = rules::derive_edges(
        &working,
        &ground,
        base_iri,
        &queries,
        &rules_text,
        &resources_text,
    )?;

    // 4. Partition goals + enumerate bounded candidates (PROJ-705).
    let components = search::partition_goals(&ground.goal, &edges);
    let candidates = search::enumerate_candidates(&components);

    // 5. Candidate 0 — single actor. Unsolvable here means the admitted
    //    problem itself is unsolvable: propagate CNG_R04.
    let single_tape = ground
        .find_plan().into_result()
        .map_err(|e| CngRefusal::PlanUnsolvable(format!("no admitted plan: {e}")))?;
    let single_model = crate::powl::project_tape_to_powl(&single_tape)?;
    let mut receipts: Vec<CandidateReceipt> = vec![CandidateReceipt {
        candidate_id: SINGLE_ACTOR_CANDIDATE_ID.to_string(),
        status: CandidateStatus::Admissible,
        reason: "admissible".to_string(),
        score: select::score_single(single_tape.ops.len()),
    }];

    // 6. Split candidates: manufacture → plan → replay → proofs → compose
    //    → score. Failures are RECORDED receipts, never silent skips.
    //    O(candidates) bounded pipelines.
    let mut artifacts: BTreeMap<String, SplitArtifacts> = BTreeMap::new();
    let augmentation = interference::augmentation_atoms(&ground.initial_state, &edges);
    for candidate in candidates.iter().skip(1) {
        match run_split_candidate(
            candidate,
            &augmentation,
            &working,
            &queries,
            &templates,
            base_iri,
            &source_problem_iri,
            domain,
            problem,
            &ground,
            &edges,
        ) {
            Ok((score, split)) => {
                receipts.push(CandidateReceipt {
                    candidate_id: candidate.id.clone(),
                    status: CandidateStatus::Admissible,
                    reason: "admissible".to_string(),
                    score,
                });
                artifacts.insert(candidate.id.clone(), split);
            }
            Err(refusal) => {
                receipts.push(CandidateReceipt {
                    candidate_id: candidate.id.clone(),
                    status: CandidateStatus::Inadmissible,
                    reason: format!("{}: {}", refusal.code(), refusal.message()),
                    score: Score {
                        makespan: 0,
                        dispatch_cost: 0,
                        risk: 0,
                    },
                });
            }
        }
    }

    // 7. Selection law (PROJ-710).
    let verdict = select::select(&mut receipts, forced)?;

    // 8. Materialize the outcome.
    let result_iri = format!("{base_iri}/decomposition");
    let (
        outcome,
        subworkflows,
        cross_edges,
        interface_atoms,
        release_obligations,
        composed_model,
        provenance_pair,
    ) = if verdict.selected_id == SINGLE_ACTOR_CANDIDATE_ID {
        let outcome = match verdict.best_split_id.clone() {
            Some(best) => DecompositionOutcome::NoBeneficialDecomposition {
                best_rejected_id: best,
            },
            None => DecompositionOutcome::NoAdmissibleDecomposition {
                rejected: verdict.rejected.clone(),
            },
        };
        let sub = SubworkflowPlan {
            id: format!("{result_iri}/subworkflow/0"),
            role: "single".to_string(),
            tape: single_tape.clone(),
            model: single_model.clone(),
            problem_pddl: String::new(),
            problem_digest: "blake3:none".to_string(),
        };
        (
            outcome,
            vec![sub],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            single_model.clone(),
            None,
        )
    } else {
        let split = artifacts.remove(&verdict.selected_id).ok_or_else(|| {
            CngRefusal::DecompositionInadmissible {
                candidate: verdict.selected_id.clone(),
                reason: "selected candidate has no surviving artifacts".to_string(),
            }
        })?;
        // Belt-and-braces gates on the SELECTED candidate: these were
        // proven during search; a failure here is CNG_R22/CNG_R24.
        interference::check_interference(&split.helper_tape, &split.main_tape, &edges)?;
        interference::check_release_closure(
            &split.s_prime,
            &ground.initial_state,
            &split.helper_tape,
            &split.main_tape,
            &edges,
        )?;
        let helper_sub = SubworkflowPlan {
            id: format!("{result_iri}/subworkflow/0"),
            role: "helper".to_string(),
            tape: split.helper_tape.clone(),
            model: crate::powl::project_tape_to_powl(&split.helper_tape)?,
            problem_digest: format!(
                "blake3:{}",
                blake3::hash(split.helper_pddl.as_bytes()).to_hex()
            ),
            problem_pddl: split.helper_pddl.clone(),
        };
        let main_sub = SubworkflowPlan {
            id: format!("{result_iri}/subworkflow/1"),
            role: "main".to_string(),
            tape: split.main_tape.clone(),
            model: crate::powl::project_tape_to_powl(&split.main_tape)?,
            problem_digest: format!(
                "blake3:{}",
                blake3::hash(split.main_pddl.as_bytes()).to_hex()
            ),
            problem_pddl: split.main_pddl.clone(),
        };
        let interface: Vec<String> = split.s_prime.iter().map(|a| a.label()).collect();
        let outcome = DecompositionOutcome::Selected {
            candidate_id: verdict.selected_id.clone(),
            subworkflows: 2,
        };
        (
            outcome,
            vec![helper_sub, main_sub],
            split.cross_edges,
            interface,
            split.released,
            split.model,
            Some((split.helper_iri, split.main_iri)),
        )
    };

    // 9. Emit the result receipt graph (PROJ-710): templates + serializer
    //    output only.
    let result_path = emit_result_graph(
        out_dir,
        &result_iri,
        &source_problem_iri,
        &outcome,
        &verdict,
        &receipts,
        &subworkflows,
        &cross_edges,
        &interface_atoms,
        &release_obligations,
        &composed_model,
        provenance_pair.as_ref(),
        &working,
        &queries,
        &templates,
    )?;

    Ok(DecompositionResult {
        outcome,
        subworkflows,
        cross_edges,
        interface_atoms,
        release_obligations,
        composed_model,
        result_graph_path: result_path,
        candidate_receipts: receipts,
    })
}

/// One split candidate's full proof pipeline (PROJ-706..709). Any typed
/// refusal is the candidate's recorded inadmissibility reason.
///
/// # Errors
/// `CNG_R04` (subproblem unsolvable), `CNG_R23`, `CNG_R22`, `CNG_R24`,
/// `CNG_R21` (cyclic composition), plus graph/render refusals.
///
/// # Complexity
/// Two bounded ground/BFS passes + O(h·m) proofs.
#[allow(clippy::too_many_arguments)]
fn run_split_candidate(
    candidate: &Candidate,
    augmentation: &[Pddl8GroundAtom],
    working: &Store,
    queries: &QuerySet,
    templates: &DecompTemplates,
    base_iri: &str,
    source_problem_iri: &str,
    domain: &Pddl8Domain,
    problem: &Pddl8Problem,
    ground: &GroundProblem,
    edges: &DerivedEdges,
) -> Result<(Score, SplitArtifacts), CngRefusal> {
    let candidate_slug = slug(&candidate.id);

    // Helper goal = candidate atoms ∪ release-closure augmentation (the
    // helper must restore the initial custody surface). O(n log n).
    let mut helper_goal: Vec<Pddl8GroundAtom> = candidate.helper_goal.clone();
    helper_goal.extend(augmentation.iter().cloned());
    helper_goal.sort();
    helper_goal.dedup();

    let helper_name = format!("helper-{candidate_slug}");
    let helper_iri = lift::problem_iri(base_iri, &helper_name);
    let helper_store = manufacture::manufacture_helper(
        working,
        queries,
        base_iri,
        source_problem_iri,
        &helper_iri,
        &helper_name,
        &helper_goal,
    )?;
    let (helper_tape, helper_pddl) = plan_manufactured(
        &helper_store,
        &helper_iri,
        domain,
        problem,
        queries,
        &templates.problem_pddl,
    )?;

    // Interface state s′ by verified replay (PROJ-707).
    let s_prime = interface::replay_to_interface_state(&ground.initial_state, &helper_tape)?;

    let main_name = format!("main-{candidate_slug}");
    let main_iri = lift::problem_iri(base_iri, &main_name);
    let s_prime_vec: Vec<Pddl8GroundAtom> = s_prime.iter().cloned().collect();
    let main_store = manufacture::manufacture_main(
        working,
        queries,
        base_iri,
        source_problem_iri,
        &main_iri,
        &main_name,
        &s_prime_vec,
        &ground.goal,
    )?;
    let (main_tape, main_pddl) = plan_manufactured(
        &main_store,
        &main_iri,
        domain,
        problem,
        queries,
        &templates.problem_pddl,
    )?;

    // Proof obligations (PROJ-708).
    interference::check_interference(&helper_tape, &main_tape, edges)?;
    let released = interference::check_release_closure(
        &s_prime,
        &ground.initial_state,
        &helper_tape,
        &main_tape,
        edges,
    )?;

    // Composition + score (PROJ-709/710).
    let (model, cross) = compose::compose_two(&candidate.id, &helper_tape, &main_tape, edges)?;
    let index_edges = cross_index_edges(&helper_tape, &main_tape, &cross);
    let score = select::score_split(
        &candidate.id,
        helper_tape.ops.len(),
        main_tape.ops.len(),
        &index_edges,
    )?;

    Ok((
        score,
        SplitArtifacts {
            helper_tape,
            main_tape,
            model,
            cross_edges: cross,
            s_prime,
            released,
            helper_pddl,
            main_pddl,
            helper_iri,
            main_iri,
        },
    ))
}

/// Assembles and writes the `decomp:DecompositionResult` Turtle graph:
/// template-rendered core + per-candidate receipts + per-fact constraint
/// triples + the powl2 serialization + CONSTRUCTed PROV-O provenance. The
/// assembled text is parsed back through oxigraph (proof it is lawful RDF)
/// and the embedded POWL graph is shape-validated.
///
/// # Errors
/// `CNG_R10` write failures, `CNG_R01` parse-back failures, `CNG_R06`
/// POWL shape violations.
///
/// # Complexity
/// O(receipts + facts + model nodes) template fills + one parse pass.
#[allow(clippy::too_many_arguments)]
fn emit_result_graph(
    out_dir: &Path,
    result_iri: &str,
    source_problem_iri: &str,
    outcome: &DecompositionOutcome,
    verdict: &SelectionVerdict,
    receipts: &[CandidateReceipt],
    subworkflows: &[SubworkflowPlan],
    cross_edges: &[(String, String)],
    interface_atoms: &[String],
    release_obligations: &[String],
    composed_model: &Powl,
    provenance_pair: Option<&(String, String)>,
    working: &Store,
    queries: &QuerySet,
    templates: &DecompTemplates,
) -> Result<PathBuf, CngRefusal> {
    fs::create_dir_all(out_dir)
        .map_err(|e| CngRefusal::IoRefused(format!("mkdir {}: {e}", out_dir.display())))?;

    let mut body = String::new();
    body.push_str(&crate::bench::fill_template(
        &templates.result,
        &[
            ("RESULT_IRI", result_iri),
            ("OUTCOME", outcome.as_str()),
            ("SELECTED_CANDIDATE_ID", verdict.selected_id.as_str()),
            ("SUBWORKFLOW_COUNT", subworkflows.len().to_string().as_str()),
            ("REJECTED_COUNT", verdict.rejected.to_string().as_str()),
            ("SOURCE_PROBLEM", source_problem_iri),
        ],
    ));

    // Per-candidate receipts (accepted AND rejected). O(receipts).
    for receipt in receipts {
        body.push('\n');
        body.push_str(&crate::bench::fill_template(
            &templates.receipt,
            &[
                ("RESULT_IRI", result_iri),
                ("CANDIDATE_SLUG", slug(&receipt.candidate_id).as_str()),
                ("CANDIDATE_ID", receipt.candidate_id.as_str()),
                ("STATUS", receipt.status.as_str()),
                ("REASON", receipt.reason.as_str()),
                ("MAKESPAN", receipt.score.makespan.to_string().as_str()),
                (
                    "DISPATCH_COST",
                    receipt.score.dispatch_cost.to_string().as_str(),
                ),
                ("RISK", receipt.score.risk.to_string().as_str()),
            ],
        ));
    }

    // Subworkflow facts. O(subworkflows).
    let powl_base = format!("{result_iri}/powl");
    for (i, sub) in subworkflows.iter().enumerate() {
        let powl_root = if subworkflows.len() == 1 {
            format!("{powl_base}/n0")
        } else {
            format!("{powl_base}/n0/c{i}")
        };
        body.push('\n');
        body.push_str(&crate::bench::fill_template(
            &templates.subworkflow,
            &[
                ("RESULT_IRI", result_iri),
                ("INDEX", i.to_string().as_str()),
                ("ROLE", sub.role.as_str()),
                ("POWL_ROOT_IRI", powl_root.as_str()),
                ("PROBLEM_DIGEST", sub.problem_digest.as_str()),
            ],
        ));
    }

    // Constraint surface: interface atoms, cross edges, obligations.
    // O(facts) template instances (templates cannot iterate).
    for atom in interface_atoms {
        body.push('\n');
        body.push_str(&crate::bench::fill_template(
            &templates.interface_atom,
            &[("RESULT_IRI", result_iri), ("ATOM_LABEL", atom.as_str())],
        ));
    }
    for (i, (from, to)) in cross_edges.iter().enumerate() {
        body.push('\n');
        body.push_str(&crate::bench::fill_template(
            &templates.order_edge,
            &[
                ("RESULT_IRI", result_iri),
                ("INDEX", i.to_string().as_str()),
                ("FROM_LABEL", from.as_str()),
                ("TO_LABEL", to.as_str()),
            ],
        ));
    }
    for resource in release_obligations {
        body.push('\n');
        body.push_str(&crate::bench::fill_template(
            &templates.release_obligation,
            &[
                ("RESULT_IRI", result_iri),
                ("RESOURCE_LABEL", resource.as_str()),
            ],
        ));
    }

    // Composed POWL graph (powl2 serializer output).
    body.push('\n');
    let powl_turtle = compose::composed_to_turtle(composed_model, &powl_base, source_problem_iri);
    body.push_str(&powl_turtle);

    // PROV-O provenance for the selected split (CONSTRUCT output,
    // serialized as sorted N-Triples). Skipped for single-actor outcomes.
    if let Some((helper_iri, main_iri)) = provenance_pair {
        let prov_store = Store::new()
            .map_err(|e| CngRefusal::IoRefused(format!("provenance store construction: {e}")))?;
        manufacture::manufacture_provenance(
            working,
            queries,
            &format!("{result_iri}/candidate/{}", slug(&verdict.selected_id)),
            &verdict.selected_id,
            helper_iri,
            main_iri,
            source_problem_iri,
            &prov_store,
        )?;
        body.push('\n');
        body.push_str(&store_to_sorted_ntriples(&prov_store)?);
    }

    // Prove the assembled graph parses; shape-validate the POWL portion.
    let full_store = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("result store construction: {e}")))?;
    full_store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
        .map_err(|e| CngRefusal::MalformedTtl(format!("result graph does not parse: {e}")))?;
    let powl_store =
        Store::new().map_err(|e| CngRefusal::IoRefused(format!("powl store construction: {e}")))?;
    powl_store
        .load_from_slice(
            RdfParser::from_format(RdfFormat::Turtle),
            powl_turtle.as_bytes(),
        )
        .map_err(|e| CngRefusal::MalformedTtl(format!("powl graph does not parse: {e}")))?;
    crate::shape::validate_powl_store(&powl_store, true)?;

    let path = out_dir.join("decomposition-result.ttl");
    fs::write(&path, &body)
        .map_err(|e| CngRefusal::IoRefused(format!("write {}: {e}", path.display())))?;
    Ok(path)
}

#[cfg(test)]
#[path = "decomp_test.rs"]
mod decomp_test;
