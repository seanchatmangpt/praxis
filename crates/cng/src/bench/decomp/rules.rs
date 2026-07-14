//! PROJ-704 — Datalog decomposition-edge layer: EDB facts are projected out
//! of the lifted pddl-strips graph by the on-disk `queries/decomp/facts-*.rq`
//! SELECTs, the rule set lives in `rules/decomp.dl` (+ admitted resource
//! facts in `rules/decomp-resources.dl`), and the praxis-graphlaw semi-naive
//! materializer derives achieves/mutex/dependsOn/mustPrecede/
//! custodyConflict/releasesResource — same loader precedent as
//! `bench/roles.rs::derive_roles_datalog`.
//!
//! Tokens: actions are `:a-<safe(ground label)>`, atoms are
//! `:p-<safe(ground label)>` (`lift::safe_token`), so the Datalog layer and
//! the RDF layer are two projections of the same content-addressed
//! identities; the derived edges are decoded back to ground labels.

use std::collections::{BTreeMap, BTreeSet};

use oxigraph::store::Store;
// PROJ-733: bcinr_pddl::ground::IndexedGroundProblem (relaxed-reachability-
// pruned grounding) mirrors bcinr_pddl::ground::GroundProblem's public
// fields exactly — see crates/cng/src/bench/decomp/mod.rs's module doc.
use bcinr_pddl::ground::IndexedGroundProblem as GroundProblem;

use crate::bench::roles::select_rows;
use crate::bench::templates::QuerySet;
use crate::powl::CngRefusal;

use super::lift::{action_iri, atom_iri, effect_iri, safe_token};

/// Edges and classifications derived by `rules/decomp.dl`, decoded to
/// ground labels.
#[derive(Debug, Default)]
pub struct DerivedEdges {
    /// atom label → achiever ground-action labels.
    pub achievers: BTreeMap<String, BTreeSet<String>>,
    /// symmetric mutex pairs over ground-action labels.
    pub mutex: BTreeSet<(String, String)>,
    /// symmetric custody-conflict pairs over ground-action labels.
    pub custody: BTreeSet<(String, String)>,
    /// transitively closed mustPrecede over ground-action labels.
    pub must_precede: BTreeSet<(String, String)>,
    /// atom labels classified as resource atoms.
    pub resource_atoms: BTreeSet<String>,
}

/// (action IRI → label, atom-node IRI → atom label) inversion maps,
/// recomputed from the ground surface with the same deterministic IRI
/// scheme the lifter used.
///
/// # Complexity
/// O(A · c log n) over A ground actions with ≤ c conjuncts.
fn inversion_maps(
    ground: &GroundProblem,
    base_iri: &str,
) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let mut actions: BTreeMap<String, String> = BTreeMap::new();
    let mut atoms: BTreeMap<String, String> = BTreeMap::new();
    for atom in ground.initial_state.iter().chain(ground.goal.iter()) {
        atoms.insert(atom_iri(base_iri, atom), atom.label());
    }
    for action in &ground.actions {
        actions.insert(action_iri(base_iri, &action.label), action.label.clone());
        for atom in &action.preconditions {
            atoms.insert(atom_iri(base_iri, atom), atom.label());
        }
        for atom in &action.add_effects {
            atoms.insert(atom_iri(base_iri, atom), atom.label());
            atoms.insert(effect_iri(base_iri, "add", atom), atom.label());
        }
        for atom in &action.del_effects {
            atoms.insert(atom_iri(base_iri, atom), atom.label());
            atoms.insert(effect_iri(base_iri, "del", atom), atom.label());
        }
    }
    (actions, atoms)
}

/// Appends `:a-<action> :<pred> :p-<atom>.` fact lines for one EDB SELECT.
///
/// # Complexity
/// O(rows).
fn append_pair_facts(
    doc: &mut String,
    store: &Store,
    query: &str,
    dl_pred: &str,
    action_labels: &BTreeMap<String, String>,
    atom_labels: &BTreeMap<String, String>,
) -> Result<(), CngRefusal> {
    for row in select_rows(store, query)? {
        let action_iri = row.get("action").ok_or_else(|| {
            CngRefusal::MalformedTtl("facts query row missing ?action".to_string())
        })?;
        let atom_iri = row
            .get("atom")
            .ok_or_else(|| CngRefusal::MalformedTtl("facts query row missing ?atom".to_string()))?;
        let action = action_labels.get(action_iri).ok_or_else(|| {
            CngRefusal::HardcodingSuspicion(format!(
                "graph action <{action_iri}> has no ground-surface counterpart; the lifted \
                 graph is detached from the admitted surface"
            ))
        })?;
        let atom = atom_labels.get(atom_iri).ok_or_else(|| {
            CngRefusal::HardcodingSuspicion(format!(
                "graph atom <{atom_iri}> has no ground-surface counterpart; the lifted \
                 graph is detached from the admitted surface"
            ))
        })?;
        doc.push_str(&format!(
            ":a-{} :{dl_pred} :p-{}.\n",
            safe_token(action),
            safe_token(atom)
        ));
    }
    Ok(())
}

/// Runs the full Datalog derivation: EDB projection from `store` via the
/// on-disk queries, rules from `rules_text` (`rules/decomp.dl`) + admitted
/// resource facts (`resources_text`, `rules/decomp-resources.dl`),
/// materialization via praxis-graphlaw, decode back to ground labels.
///
/// # Errors
/// `CNG_R05 UnsupportedConstruct` for rule parse/materialization refusals;
/// `CNG_R01` for query failures; `CNG_R09` when the graph names an
/// action/atom absent from the ground surface.
///
/// # Complexity
/// O(rows) EDB assembly + semi-naive materialization over the rule set
/// (worst case O(|rules| · |facts|²) joins, bounded by the STRIPS8 caps).
pub fn derive_edges(
    store: &Store,
    ground: &GroundProblem,
    base_iri: &str,
    queries: &QuerySet,
    rules_text: &str,
    resources_text: &str,
) -> Result<DerivedEdges, CngRefusal> {
    use praxis_graphlaw::parser::Parser;
    use praxis_graphlaw::TripleStore;

    let (action_labels, atom_labels) = inversion_maps(ground, base_iri);

    let mut doc = String::new();
    append_pair_facts(
        &mut doc,
        store,
        queries.get("facts-pre")?,
        "pre",
        &action_labels,
        &atom_labels,
    )?;
    append_pair_facts(
        &mut doc,
        store,
        queries.get("facts-pre-noninit")?,
        "preOpen",
        &action_labels,
        &atom_labels,
    )?;
    append_pair_facts(
        &mut doc,
        store,
        queries.get("facts-add")?,
        "addEff",
        &action_labels,
        &atom_labels,
    )?;
    append_pair_facts(
        &mut doc,
        store,
        queries.get("facts-del")?,
        "delEff",
        &action_labels,
        &atom_labels,
    )?;

    // Atom content facts: predicate symbol + argument symbols. Derived from
    // the ground structs (the graph carries the same content; the labels
    // are the shared canonical form). O(atoms · arity).
    let mut seen_atoms: BTreeSet<String> = BTreeSet::new();
    let mut all_atoms: Vec<&bcinr_pddl::Pddl8GroundAtom> = Vec::new();
    for atom in ground.initial_state.iter().chain(ground.goal.iter()) {
        all_atoms.push(atom);
    }
    for action in &ground.actions {
        all_atoms.extend(action.preconditions.iter());
        all_atoms.extend(action.add_effects.iter());
        all_atoms.extend(action.del_effects.iter());
    }
    for atom in all_atoms {
        let token = safe_token(&atom.label());
        if !seen_atoms.insert(token.clone()) {
            continue;
        }
        doc.push_str(&format!(
            ":p-{token} :predName :{}.\n",
            safe_token(&atom.pred)
        ));
        for arg in &atom.args {
            doc.push_str(&format!(":p-{token} :hasArgVal :{}.\n", safe_token(arg)));
        }
    }

    // Admitted object-level resource annotations from the graph. O(rows).
    for row in select_rows(store, queries.get("resource-objects")?)? {
        if let Some(name) = row.get("name") {
            doc.push_str(&format!(":{} :isResourceObject :true.\n", safe_token(name)));
        }
    }

    // Admitted predicate-level resource facts + the rule set. O(lines).
    for line in resources_text.lines().chain(rules_text.lines()) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        doc.push_str(trimmed);
        doc.push('\n');
    }

    let (facts, rules) = Parser::parse(doc);
    if rules.is_empty() {
        return Err(CngRefusal::UnsupportedConstruct(
            "decomp.dl yielded zero parsed Datalog rules".to_string(),
        ));
    }
    let mut dl_store = TripleStore::new();
    for fact in facts {
        dl_store.add(fact);
    }
    dl_store.add_rules(rules).map_err(|e| {
        CngRefusal::UnsupportedConstruct(format!("decomp.dl rule validation refused: {e}"))
    })?;
    let inferred = dl_store.materialize().map_err(|e| {
        CngRefusal::UnsupportedConstruct(format!("decomp.dl materialization refused: {e}"))
    })?;

    // Token → label decode maps. O(n log n).
    let mut action_by_token: BTreeMap<String, String> = BTreeMap::new();
    for label in action_labels.values() {
        action_by_token.insert(format!(":a-{}", safe_token(label)), label.clone());
    }
    let mut atom_by_token: BTreeMap<String, String> = BTreeMap::new();
    for label in atom_labels.values() {
        atom_by_token.insert(format!(":p-{}", safe_token(label)), label.clone());
    }

    let decode = |encoded: usize| -> Result<String, CngRefusal> {
        praxis_graphlaw::encoding::Encoder::decode(&encoded)
            .ok_or_else(|| CngRefusal::MalformedTtl("Datalog term failed to decode".to_string()))
    };

    let mut edges = DerivedEdges::default();
    // O(inferred facts).
    for triple in &inferred {
        let predicate = decode(triple.p.to_encoded())?;
        let s = decode(triple.s.to_encoded())?;
        let o = decode(triple.o.to_encoded())?;
        match predicate.as_str() {
            ":achieves" => {
                if let (Some(a), Some(p)) = (action_by_token.get(&s), atom_by_token.get(&o)) {
                    edges
                        .achievers
                        .entry(p.clone())
                        .or_default()
                        .insert(a.clone());
                }
            }
            ":mutex" => {
                if let (Some(a), Some(b)) = (action_by_token.get(&s), action_by_token.get(&o)) {
                    if a != b {
                        edges.mutex.insert((a.clone(), b.clone()));
                    }
                }
            }
            ":custodyConflict" => {
                if let (Some(a), Some(b)) = (action_by_token.get(&s), action_by_token.get(&o)) {
                    if a != b {
                        edges.custody.insert((a.clone(), b.clone()));
                    }
                }
            }
            ":mustPrecede" => {
                if let (Some(a), Some(b)) = (action_by_token.get(&s), action_by_token.get(&o)) {
                    if a != b {
                        edges.must_precede.insert((a.clone(), b.clone()));
                    }
                }
            }
            ":resourceAtom" => {
                if let Some(p) = atom_by_token.get(&s) {
                    edges.resource_atoms.insert(p.clone());
                }
            }
            _ => {}
        }
    }
    Ok(edges)
}
