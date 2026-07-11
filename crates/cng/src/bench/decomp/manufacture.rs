//! PROJ-706 — Subproblem manufacture: helper/main `pddl:Problem` graphs are
//! CONSTRUCTed (never hand-assembled as triples in Rust) from the lifted
//! working store via the on-disk `queries/decomp/construct-*.rq` files,
//! with `{KEY}` placeholders substituted via `str::replace` and atom sets
//! injected as `VALUES` IRI lists (content-addressed atom identity, so
//! every s′ / goal atom already has a node in the working store). PROV-O
//! decomposition provenance follows the
//! `templates/dispatch-consequence.template.ttl` precedent.

use bcinr_pddl::Pddl8GroundAtom;
use oxigraph::store::Store;

use crate::bench::roles::run_construct;
use crate::bench::templates::QuerySet;
use crate::powl::CngRefusal;

use super::lift::atom_iri;

/// Space-separated `<iri>` list for `VALUES` injection.
///
/// # Complexity
/// O(n) over the IRIs.
pub fn values_list(base_iri: &str, atoms: &[Pddl8GroundAtom]) -> String {
    let mut iris: Vec<String> = atoms
        .iter()
        .map(|a| format!("<{}>", atom_iri(base_iri, a)))
        .collect();
    iris.sort();
    iris.dedup();
    iris.join(" ")
}

/// Runs one filled CONSTRUCT over `working` into a fresh store, then pulls
/// the manufactured problem's atom/object/domain content closure so the
/// result graph is self-contained for rendering. The manufactured problem
/// triples are ALSO inserted into `working` (needed for the closure query's
/// joins; the working store is the per-decomposition scratch graph).
///
/// # Errors
/// `CNG_R01` for construct failures; `CNG_R05` when the CONSTRUCT produced
/// an empty graph (the manufacture law would otherwise pass silence along).
///
/// # Complexity
/// Query-engine cost + O(produced triples).
fn manufacture(
    working: &Store,
    queries: &QuerySet,
    skeleton_query: &str,
    problem_iri: &str,
) -> Result<Store, CngRefusal> {
    let out = Store::new()
        .map_err(|e| CngRefusal::IoRefused(format!("manufacture store construction: {e}")))?;
    let produced = run_construct(working, skeleton_query, &out)?;
    if produced == 0 {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "subproblem manufacture CONSTRUCT for <{problem_iri}> produced zero triples"
        )));
    }
    // Mirror the skeleton into the working store so the closure query can
    // join the new problem against the lifted content. O(produced).
    for quad in out.iter() {
        let quad = quad.map_err(|e| CngRefusal::MalformedTtl(format!("manufacture iter: {e}")))?;
        working
            .insert(&quad)
            .map_err(|e| CngRefusal::IoRefused(format!("manufacture mirror insert: {e}")))?;
    }
    let closure = queries
        .get("construct-atom-closure")?
        .replace("{PROBLEM}", problem_iri);
    run_construct(working, &closure, &out)?;
    Ok(out)
}

/// Manufactures the helper subproblem graph: full source init, candidate
/// helper goal atoms (+ release-closure augmentation atoms) as goal.
///
/// # Complexity
/// See `manufacture`.
#[allow(clippy::too_many_arguments)]
pub fn manufacture_helper(
    working: &Store,
    queries: &QuerySet,
    base_iri: &str,
    source_problem_iri: &str,
    helper_iri: &str,
    helper_name: &str,
    goal_atoms: &[Pddl8GroundAtom],
) -> Result<Store, CngRefusal> {
    let query = queries
        .get("construct-helper-problem")?
        .replace("{HELPER_IRI}", helper_iri)
        .replace("{HELPER_NAME}", helper_name)
        .replace("{SOURCE_PROBLEM}", source_problem_iri)
        .replace("{GOAL_ATOM_VALUES}", &values_list(base_iri, goal_atoms));
    manufacture(working, queries, &query, helper_iri)
}

/// Manufactures the main subproblem graph: init = the proven interface
/// state s′, goal = the full original goal set.
///
/// # Complexity
/// See `manufacture`.
#[allow(clippy::too_many_arguments)]
pub fn manufacture_main(
    working: &Store,
    queries: &QuerySet,
    base_iri: &str,
    source_problem_iri: &str,
    main_iri: &str,
    main_name: &str,
    init_atoms: &[Pddl8GroundAtom],
    goal_atoms: &[Pddl8GroundAtom],
) -> Result<Store, CngRefusal> {
    let query = queries
        .get("construct-main-problem")?
        .replace("{MAIN_IRI}", main_iri)
        .replace("{MAIN_NAME}", main_name)
        .replace("{SOURCE_PROBLEM}", source_problem_iri)
        .replace("{INIT_ATOM_VALUES}", &values_list(base_iri, init_atoms))
        .replace("{GOAL_ATOM_VALUES}", &values_list(base_iri, goal_atoms));
    manufacture(working, queries, &query, main_iri)
}

/// CONSTRUCTs the PROV-O decomposition provenance triples for one candidate
/// into `sink`.
///
/// # Complexity
/// Query-engine cost; O(produced triples).
pub fn manufacture_provenance(
    working: &Store,
    queries: &QuerySet,
    candidate_iri: &str,
    candidate_id: &str,
    helper_iri: &str,
    main_iri: &str,
    source_problem_iri: &str,
    sink: &Store,
) -> Result<usize, CngRefusal> {
    let query = queries
        .get("construct-provenance")?
        .replace("{CANDIDATE_IRI}", candidate_iri)
        .replace("{CANDIDATE_ID}", candidate_id)
        .replace("{HELPER_IRI}", helper_iri)
        .replace("{MAIN_IRI}", main_iri)
        .replace("{SOURCE_PROBLEM}", source_problem_iri);
    run_construct(working, &query, sink)
}
