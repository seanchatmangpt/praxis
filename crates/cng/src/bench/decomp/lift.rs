//! PROJ-702 — Lifter: grounded planning surface → pddl-strips triples in a
//! fresh oxigraph store (`ontologies/pddl-strips.ttl` contract).
//!
//! Direction of authority: the admitted PDDL string literals are parsed by
//! the existing bcinr parser and grounded by `GroundProblem::build`; this
//! module CONSTRUCTS the equivalent `pddl:` graph in Rust from those parsed
//! structs (data construction via the oxigraph model API — never inline
//! Turtle text). Atom identity is content-addressed: the same ground atom
//! always yields the same IRI, so SPARQL IRI equality is atom equality
//! (load-bearing for `facts-pre-noninit.rq`'s FILTER NOT EXISTS and for
//! s′ → `initAtom` VALUES injection).
//!
//! IRI scheme (deterministic, from `base_iri`):
//! - `<base>/domain/<name>`, `<base>/problem/<name>`, `<base>/object/<name>`
//! - plain atoms (preconditions, init, goal): `<base>/atom/<key>`
//! - effect atoms: `<base>/add/<key>` / `<base>/del/<key>` — separate
//!   namespaces because the shapes contract types effect nodes
//!   `pddl:AddEffect`/`pddl:DelEffect` ONLY (never also `pddl:Atom`), while
//!   the same ground atom may independently occur as a plain atom.
//! - ground actions: `<base>/action/<key>` (key = sanitized ground label;
//!   `pddl:actionName` carries the schema name, satisfying the canonical
//!   `^[a-z][a-z0-9-]*$` grammar — ground identity lives in the IRI).

use std::collections::BTreeSet;

use bcinr_pddl::Pddl8GroundAtom;
use oxigraph::model::{Literal, NamedNode, Quad, Term};
use oxigraph::store::Store;
// PROJ-733: bcinr_pddl::ground::lazy::IndexedGroundProblem (relaxed-reachability-
// pruned grounding) mirrors bcinr_pddl::ground::GroundProblem's public
// fields exactly — see crates/cng/src/bench/decomp/mod.rs's module doc.
use bcinr_pddl::ground::lazy::IndexedGroundProblem as GroundProblem;

use crate::powl::CngRefusal;

/// pddl-strips vocabulary namespace (`ontologies/pddl-strips.ttl`).
pub const PDDL_STRIPS_PREFIX: &str = "https://truex.io/ontology/pddl-strips#";

/// RDF `type` predicate IRI.
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// Sanitizes a ground label into an IRI-local / Datalog-safe token: ASCII
/// alphanumerics and `-` pass through (lowercased), every other character
/// maps 1:1 to `-`. Injective over the canonical symbol grammar
/// (`^[a-z][a-z0-9-]*$` names plus `(`, `)`, `,` structure characters).
///
/// # Complexity
/// O(len).
pub fn safe_token(label: &str) -> String {
    label
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' | '-' => c,
            'A'..='Z' => c.to_ascii_lowercase(),
            _ => '-',
        })
        .collect()
}

/// Content-addressed plain-atom IRI for a ground atom.
///
/// # Complexity
/// O(label len).
pub fn atom_iri(base_iri: &str, atom: &Pddl8GroundAtom) -> String {
    format!(
        "{}/atom/{}",
        base_iri.trim_end_matches('/'),
        safe_token(&atom.label())
    )
}

/// Content-addressed effect-node IRI (`kind` = "add" | "del").
///
/// # Complexity
/// O(label len).
pub fn effect_iri(base_iri: &str, kind: &str, atom: &Pddl8GroundAtom) -> String {
    format!(
        "{}/{kind}/{}",
        base_iri.trim_end_matches('/'),
        safe_token(&atom.label())
    )
}

/// Ground-action IRI.
///
/// # Complexity
/// O(label len).
pub fn action_iri(base_iri: &str, label: &str) -> String {
    format!(
        "{}/action/{}",
        base_iri.trim_end_matches('/'),
        safe_token(label)
    )
}

/// Problem IRI for the lifted source problem.
pub fn problem_iri(base_iri: &str, name: &str) -> String {
    format!(
        "{}/problem/{}",
        base_iri.trim_end_matches('/'),
        safe_token(name)
    )
}

/// Domain IRI for the lifted domain.
pub fn domain_iri(base_iri: &str, name: &str) -> String {
    format!(
        "{}/domain/{}",
        base_iri.trim_end_matches('/'),
        safe_token(name)
    )
}

fn named(iri: &str) -> Result<NamedNode, CngRefusal> {
    NamedNode::new(iri).map_err(|e| CngRefusal::MalformedTtl(format!("lift IRI {iri}: {e}")))
}

fn insert(store: &Store, s: &NamedNode, p: &str, o: Term) -> Result<(), CngRefusal> {
    let pred = named(p)?;
    let quad = Quad::new(s.clone(), pred, o, oxigraph::model::GraphName::DefaultGraph);
    store
        .insert(&quad)
        .map_err(|e| CngRefusal::IoRefused(format!("lift insert: {e}")))?;
    Ok(())
}

/// Emits `predicateName` + reified ordered `Argument` nodes for `node`
/// describing `atom` (contract: argIndex 0-based, argValue string symbol).
///
/// # Complexity
/// O(arity) per atom.
fn emit_atom_content(
    store: &Store,
    node: &NamedNode,
    node_iri: &str,
    atom: &Pddl8GroundAtom,
) -> Result<(), CngRefusal> {
    let pddl = PDDL_STRIPS_PREFIX;
    insert(
        store,
        node,
        &format!("{pddl}predicateName"),
        Term::Literal(Literal::new_simple_literal(&atom.pred)),
    )?;
    for (i, value) in atom.args.iter().enumerate() {
        let arg_node = named(&format!("{node_iri}/arg/{i}"))?;
        insert(
            store,
            node,
            &format!("{pddl}argument"),
            Term::NamedNode(arg_node.clone()),
        )?;
        insert(
            store,
            &arg_node,
            RDF_TYPE,
            Term::NamedNode(named(&format!("{pddl}Argument"))?),
        )?;
        insert(
            store,
            &arg_node,
            &format!("{pddl}argIndex"),
            Term::Literal(Literal::from(i as i64)),
        )?;
        insert(
            store,
            &arg_node,
            &format!("{pddl}argValue"),
            Term::Literal(Literal::new_simple_literal(value)),
        )?;
    }
    Ok(())
}

/// Lifts a grounded STRIPS surface into a fresh pddl-strips store: Domain,
/// Problem (init/goal), Objects, one `pddl:Action` per ground action with
/// plain-atom preconditions and typed Add/Del effect nodes. Every distinct
/// ground atom additionally gets one plain `pddl:Atom` node (content
/// addressed) so init/goal/precondition identity is IRI identity.
///
/// # Errors
/// `CNG_R01 MalformedTtl` for IRI construction failures, `CNG_R10 IoRefused`
/// for store failures.
///
/// # Complexity
/// O(A · c + F) inserts over A ground actions with ≤ c conjuncts each and
/// F init/goal atoms; all iteration over sorted (`BTreeSet` / sorted `Vec`)
/// collections for deterministic insertion (store semantics are set-based,
/// but downstream serialization sorts, so order never leaks).
pub fn lift_ground(
    ground: &GroundProblem,
    objects: &[String],
    domain_name: &str,
    problem_name: &str,
    base_iri: &str,
) -> Result<Store, CngRefusal> {
    let store =
        Store::new().map_err(|e| CngRefusal::IoRefused(format!("lift store construction: {e}")))?;
    let pddl = PDDL_STRIPS_PREFIX;

    let dom_iri = domain_iri(base_iri, domain_name);
    let dom = named(&dom_iri)?;
    insert(
        &store,
        &dom,
        RDF_TYPE,
        Term::NamedNode(named(&format!("{pddl}Domain"))?),
    )?;
    insert(
        &store,
        &dom,
        &format!("{pddl}domainName"),
        Term::Literal(Literal::new_simple_literal(domain_name)),
    )?;

    let prob_iri = problem_iri(base_iri, problem_name);
    let prob = named(&prob_iri)?;
    insert(
        &store,
        &prob,
        RDF_TYPE,
        Term::NamedNode(named(&format!("{pddl}Problem"))?),
    )?;
    insert(
        &store,
        &prob,
        &format!("{pddl}problemName"),
        Term::Literal(Literal::new_simple_literal(problem_name)),
    )?;
    insert(
        &store,
        &prob,
        &format!("{pddl}fromDomain"),
        Term::NamedNode(dom.clone()),
    )?;

    // Objects, sorted. O(|objects| log |objects|).
    let mut sorted_objects: Vec<&String> = objects.iter().collect();
    sorted_objects.sort();
    for name in sorted_objects {
        let obj = named(&format!(
            "{}/object/{}",
            base_iri.trim_end_matches('/'),
            safe_token(name)
        ))?;
        insert(
            &store,
            &obj,
            RDF_TYPE,
            Term::NamedNode(named(&format!("{pddl}Object"))?),
        )?;
        insert(
            &store,
            &obj,
            &format!("{pddl}objectName"),
            Term::Literal(Literal::new_simple_literal(name)),
        )?;
        insert(
            &store,
            &prob,
            &format!("{pddl}hasObject"),
            Term::NamedNode(obj),
        )?;
    }

    // Every distinct ground atom (init ∪ goal ∪ preconditions ∪ effects)
    // gets one plain content-addressed pddl:Atom node. O(total atoms).
    let mut all_atoms: BTreeSet<Pddl8GroundAtom> = ground.initial_state.iter().cloned().collect();
    all_atoms.extend(ground.goal.iter().cloned());
    for action in &ground.actions {
        all_atoms.extend(action.preconditions.iter().cloned());
        all_atoms.extend(action.add_effects.iter().cloned());
        all_atoms.extend(action.del_effects.iter().cloned());
    }
    for atom in &all_atoms {
        let iri = atom_iri(base_iri, atom);
        let node = named(&iri)?;
        insert(
            &store,
            &node,
            RDF_TYPE,
            Term::NamedNode(named(&format!("{pddl}Atom"))?),
        )?;
        emit_atom_content(&store, &node, &iri, atom)?;
    }

    // Init and goal links to the plain atom nodes. O(F).
    for atom in ground.initial_state.iter() {
        let node = named(&atom_iri(base_iri, atom))?;
        insert(
            &store,
            &prob,
            &format!("{pddl}initAtom"),
            Term::NamedNode(node),
        )?;
    }
    let mut goal_sorted: Vec<&Pddl8GroundAtom> = ground.goal.iter().collect();
    goal_sorted.sort();
    for atom in goal_sorted {
        let node = named(&atom_iri(base_iri, atom))?;
        insert(
            &store,
            &prob,
            &format!("{pddl}goalAtom"),
            Term::NamedNode(node),
        )?;
    }

    // Ground actions, sorted by label for determinism. O(A log A + A·c).
    let mut actions: Vec<&bcinr_pddl::Pddl8GroundAction> = ground.actions.iter().collect();
    actions.sort_by(|a, b| a.label.cmp(&b.label));
    for action in actions {
        let act_iri = action_iri(base_iri, &action.label);
        let act = named(&act_iri)?;
        insert(
            &store,
            &act,
            RDF_TYPE,
            Term::NamedNode(named(&format!("{pddl}Action"))?),
        )?;
        insert(
            &store,
            &act,
            &format!("{pddl}actionName"),
            Term::Literal(Literal::new_simple_literal(&action.schema_name)),
        )?;
        insert(
            &store,
            &dom,
            &format!("{pddl}hasAction"),
            Term::NamedNode(act.clone()),
        )?;
        for atom in &action.preconditions {
            let node = named(&atom_iri(base_iri, atom))?;
            insert(
                &store,
                &act,
                &format!("{pddl}precondition"),
                Term::NamedNode(node),
            )?;
        }
        for atom in &action.add_effects {
            let iri = effect_iri(base_iri, "add", atom);
            let node = named(&iri)?;
            insert(
                &store,
                &node,
                RDF_TYPE,
                Term::NamedNode(named(&format!("{pddl}AddEffect"))?),
            )?;
            emit_atom_content(&store, &node, &iri, atom)?;
            insert(
                &store,
                &act,
                &format!("{pddl}effect"),
                Term::NamedNode(node),
            )?;
        }
        for atom in &action.del_effects {
            let iri = effect_iri(base_iri, "del", atom);
            let node = named(&iri)?;
            insert(
                &store,
                &node,
                RDF_TYPE,
                Term::NamedNode(named(&format!("{pddl}DelEffect"))?),
            )?;
            emit_atom_content(&store, &node, &iri, atom)?;
            insert(
                &store,
                &act,
                &format!("{pddl}effect"),
                Term::NamedNode(node),
            )?;
        }
    }

    Ok(store)
}
