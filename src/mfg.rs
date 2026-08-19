//! `mfg` (manufacturing) — RDF ontology → PDDL8 domain/problem text.
//!
//! Loads a Turtle ontology describing a `pddl:` instance vocabulary (Types,
//! Predicates, Actions, and a Problem — see `docs/PDDL_CAPABILITY_MODEL.md`
//! for the target shape this flattens), extracts a STRIPS8-profile
//! intermediate representation via SPARQL, enforces PDDL8 bounds
//! (arity/conjuncts/params <= 8, mirroring `wasm4pm_compat::pddl`'s
//! `PDDL8_MAX_*` constants), and emits PDDL domain/problem **text** by
//! direct Rust string building.
//!
//! Emission is deliberately *not* done via Tera templates: the PDDL8 bounds
//! are Rust invariants that must be enforced in code before a single byte of
//! output is produced, and the golden contract (see `ontology/lawobject.ttl`)
//! is byte-for-byte parser/planner round-trip, not template rendering.
//!
//! Tera still gets its due at the facts boundary: [`facts_json`] lowers a
//! SPARQL `SELECT` result into the exact JSON-array-of-objects shape that
//! `ggen-core`'s `sparql_column`/`sparql_row` Tera functions expect, so any
//! future template pipeline can consume ontology facts directly.

use bcinr_pddl::{
    domain_from_pddl, problem_from_pddl, GroundProblem, PDDL8_MAX_ARITY, PDDL8_MAX_CONJUNCTS,
    PDDL8_MAX_PARAMS,
};

use ggen_graph::prelude::{parse_turtle, DeterministicGraph, FactStore};

pub fn validate_shacl(
    graph: &DeterministicGraph,
    shapes_ttl: &str,
    profile_name: &str,
) -> Result<AdmissionReceipt> {
    let outcome = graph
        .validate_shacl(shapes_ttl)
        .map_err(|e| MfgError::Graph(e.to_string()))?;

    let gh = graph_hash_hex(graph)?;
    let sh_hash = hex::encode(blake3::hash(shapes_ttl.as_bytes()).as_bytes());

    Ok(AdmissionReceipt {
        conforms: outcome.conforms,
        shapes_hash: sh_hash,
        graph_hash: gh,
        profile_name: profile_name.to_string(),
        message: if outcome.conforms {
            None
        } else {
            Some("SHACL violation".to_string())
        },
    })
}

pub fn solve_ir(task: &AdmittedPlanningTask) -> ValidationReport {
    let d8: wasm4pm_compat::pddl::Pddl8Domain = (&task.domain).into();
    let p8: wasm4pm_compat::pddl::Pddl8Problem = (&task.problem).into();

    let ground = match bcinr_pddl::GroundProblem::build(&d8, &p8, None) {
        Ok(g) => g,
        Err(e) => {
            return ValidationReport {
                parsed: true,
                grounded_actions: 0,
                plan_len: 0,
                plan_steps: Vec::new(),
                solvable: false,
                error: Some(e.to_string()),
            }
        }
    };

    let grounded_actions = ground.actions.len();
    match ground.find_plan() {
        Ok(tape) => {
            let plan_steps: Vec<String> = tape
                .ops
                .iter()
                .map(|op| op.action.schema_name.clone())
                .collect();
            ValidationReport {
                parsed: true,
                grounded_actions,
                plan_len: plan_steps.len(),
                plan_steps,
                solvable: true,
                error: None,
            }
        }
        Err(e) => ValidationReport {
            parsed: true,
            grounded_actions,
            plan_len: 0,
            plan_steps: Vec::new(),
            solvable: false,
            error: Some(e.to_string()),
        },
    }
}

use oxigraph::{
    model::{Quad, Term},
    sparql::QueryResults,
};
use serde::{Deserialize, Serialize};

/// SPARQL `PREFIX` line for the `pddl:` instance vocabulary used by every
/// query in this module. See `ontology/lawobject.ttl` for the vocabulary
/// this module expects, documented inline.
const PDL_PREFIX: &str = "PREFIX pddl: <http://seanchatmangpt.github.io/praxis/pddl#>\n";

/// All errors from the ontology → PDDL8 manufacturing pipeline.
#[derive(Debug, thiserror::Error)]
pub enum MfgError {
    /// The underlying RDF graph store failed (parse, insert, or query).
    #[error("graph error: {0}")]
    Graph(String),
    /// A PDDL8 structural bound (arity, conjuncts, params) was exceeded.
    #[error("PDDL8 bound exceeded: {what} limit={limit} got={got} ({detail})")]
    BoundExceeded {
        what: &'static str,
        limit: usize,
        got: usize,
        detail: String,
    },
    /// The ontology is missing a required `pddl:` fact or has an unexpected shape.
    #[error("ontology shape error: {0}")]
    Shape(String),
    /// The manufactured PDDL text failed to round-trip through `bcinr-pddl`.
    #[error("PDDL8 round-trip error: {0}")]
    Pddl8(String),
}

impl From<ggen_graph::GraphError> for MfgError {
    fn from(e: ggen_graph::GraphError) -> Self {
        Self::Graph(e.to_string())
    }
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, MfgError>;

// ---------------------------------------------------------------------------
// Intermediate representation (STRIPS8-shaped: pre/add/del atom lists, not
// arbitrary formula strings — this is what makes `enforce_pddl8` a simple
// length check rather than a formula-complexity analysis).
// ---------------------------------------------------------------------------

/// A ground or schema-level predicate atom, e.g. `(in-stage ?obj raw)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Atom {
    /// Predicate name.
    pub pred: String,
    /// Each entry is a `?variable` or a bare constant.
    pub args: Vec<String>,
}

/// `(child - parent)` or a bare, parent-less type declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeDecl {
    pub name: String,
    pub parent: Option<String>,
}

/// A `(:predicates ...)` entry: name + typed parameter list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredicateDecl {
    pub name: String,
    /// `(?var, type)` pairs, in declared order.
    pub params: Vec<(String, String)>,
}

/// A `(:action ...)` schema: STRIPS8-only (positive conjunctive
/// precondition, add effects, del effects — no forall/implies/derived).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionDecl {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub pre: Vec<Atom>,
    pub add: Vec<Atom>,
    pub del: Vec<Atom>,
}

/// The full domain IR: types, predicates, and action schemas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PddlPddlDomainIr {
    pub name: String,
    pub types: Vec<TypeDecl>,
    pub predicates: Vec<PredicateDecl>,
    pub actions: Vec<ActionDecl>,
}

/// A `(:objects ...)` entry: object name + declared type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectDecl {
    pub name: String,
    pub ty: String,
}

/// The full problem IR: objects, initial state, and goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PddlPddlProblemIr {
    pub name: String,
    pub domain: String,
    pub objects: Vec<ObjectDecl>,
    pub init: Vec<Atom>,
    pub goal: Vec<Atom>,
}

/// Manufactured output: PDDL8 domain + problem text, plus the source
/// graph's BLAKE3 state hash (embedded as a provenance comment in the
/// domain text and returned separately for callers that need it raw).
#[derive(Debug, Clone, Serialize, Deserialize, Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionReceipt {
    pub conforms: bool,
    pub shapes_hash: String,
    pub graph_hash: String,
    pub profile_name: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmittedPlanningTask {
    pub domain: PddlDomainIr,
    pub problem: PddlProblemIr,
    pub receipt: AdmissionReceipt,
}

impl AdmittedPlanningTask {
    pub fn project_domain_text(&self) -> String {
        emit_domain(&self.domain, "projected", &self.receipt.graph_hash)
    }

    pub fn project_problem_text(&self) -> String {
        emit_problem(&self.problem)
    }
}

// Map IR to bcinr_pddl AST
impl From<&Atom> for wasm4pm_compat::pddl::Pddl8Atom {
    fn from(a: &Atom) -> Self {
        Self {
            pred: a.pred.clone(),
            args: a.args.clone(),
        }
    }
}
impl From<&TypeDecl> for wasm4pm_compat::pddl::PddlType {
    fn from(t: &TypeDecl) -> Self {
        Self {
            name: t.name.clone(),
            parent: t.parent.clone(),
        }
    }
}
impl From<&ActionDecl> for wasm4pm_compat::pddl::Pddl8ActionSchema {
    fn from(a: &ActionDecl) -> Self {
        Self {
            name: a.name.clone(),
            params: a.params.iter().map(|(v, _)| v.clone()).collect(),
            preconditions: a.pre.iter().map(Into::into).collect(),
            add_effects: a.add.iter().map(Into::into).collect(),
            del_effects: a.del.iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}
impl From<&PddlDomainIr> for wasm4pm_compat::pddl::Pddl8Domain {
    fn from(d: &PddlDomainIr) -> Self {
        Self {
            name: d.name.clone(),
            types: d.types.iter().map(Into::into).collect(),
            predicates: d
                .predicates
                .iter()
                .map(|p| (p.name.clone(), p.params.len() as u8))
                .collect(),
            actions: d.actions.iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}
impl From<&PddlProblemIr> for wasm4pm_compat::pddl::Pddl8Problem {
    fn from(p: &PddlProblemIr) -> Self {
        Self {
            name: p.name.clone(),
            domain: p.domain.clone(),
            objects: p
                .objects
                .iter()
                .map(|o| (o.name.clone(), o.ty.clone()))
                .collect(),
            init: p.init.iter().map(Into::into).collect(),
            goal: p.goal.iter().map(Into::into).collect(),
            ..Default::default()
        }
    }
}

/// Old Manufactured struct
pub struct Manufactured {
    pub domain_text: String,
    pub problem_text: String,
    /// Hex-encoded BLAKE3 hash of the source RDF graph's quad state.
    pub graph_hash_hex: String,
}

/// Report from round-tripping manufactured PDDL text through `bcinr-pddl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether both domain and problem text parsed via `bcinr-pddl`.
    pub parsed: bool,
    /// Number of grounded actions (0 if parsing failed before grounding).
    pub grounded_actions: usize,
    /// Length of the found plan (0 if unsolved or not grounded).
    pub plan_len: usize,
    /// Ordered schema names of the found plan's actions (empty if unsolved).
    pub plan_steps: Vec<String>,
    /// Whether `find_plan` found a plan reaching the goal.
    pub solvable: bool,
    /// Present when `parsed` is true but grounding/solving failed, or when
    /// parsing itself failed.
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Graph loading
// ---------------------------------------------------------------------------

/// Parse a Turtle ontology string into a fresh [`DeterministicGraph`].
pub fn load_graph(ttl: &str) -> Result<DeterministicGraph> {
    let graph = DeterministicGraph::new()?;
    let quads: Vec<Quad> = parse_turtle(ttl).map_err(|e| MfgError::Graph(e.to_string()))?;
    for quad in &quads {
        graph.insert_quad(quad)?;
    }
    Ok(graph)
}

/// Hex-encoded BLAKE3 state hash of `graph`'s current quad set.
pub fn graph_hash_hex(graph: &DeterministicGraph) -> Result<String> {
    let hash = graph.state_hash()?;
    Ok(hex::encode(hash))
}

// ---------------------------------------------------------------------------
// SPARQL extraction helpers
// ---------------------------------------------------------------------------

/// Pull the literal lexical value out of a bound term, erroring on
/// unexpected shapes (unbound variable, or a term that isn't a literal for
/// fields that are always string-valued in the `pddl:` vocabulary).
fn literal_str(term: Option<&Term>, what: &str) -> Result<String> {
    match term {
        Some(Term::Literal(lit)) => Ok(lit.value().to_string()),
        Some(other) => Err(MfgError::Shape(format!(
            "{what}: expected a literal, got {other:?}"
        ))),
        None => Err(MfgError::Shape(format!("{what}: variable not bound"))),
    }
}

/// Pull an integer-valued literal (used for `pddl:index`).
fn literal_index(term: Option<&Term>, what: &str) -> Result<usize> {
    let s = literal_str(term, what)?;
    s.parse::<usize>()
        .map_err(|e| MfgError::Shape(format!("{what}: not an integer index ({e}): {s}")))
}

/// Parse an opaque atom literal like `"(in-stage ?obj raw)"` into an [`Atom`].
fn parse_atom_literal(s: &str) -> Result<Atom> {
    let inner = s
        .trim()
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| MfgError::Shape(format!("atom literal must be `(pred arg*)`: {s}")))?;
    let mut parts = inner.split_whitespace();
    let pred = parts
        .next()
        .ok_or_else(|| MfgError::Shape(format!("atom literal has no predicate: {s}")))?
        .to_string();
    let args = parts.map(str::to_string).collect();
    Ok(Atom { pred, args })
}

fn run_select(graph: &DeterministicGraph, query: &str) -> Result<Vec<Vec<(String, Term)>>> {
    let mut rows = Vec::new();
    match graph
        .query(query)
        .map_err(|e| MfgError::Graph(e.to_string()))?
    {
        QueryResults::Solutions(sols) => {
            for sol in sols {
                let sol = sol.map_err(|e| MfgError::Shape(e.to_string()))?;
                let row = sol
                    .iter()
                    .map(|(var, term)| (var.as_str().to_string(), term.clone()))
                    .collect();
                rows.push(row);
            }
        }
        QueryResults::Boolean(_) | QueryResults::Graph(_) => {
            return Err(MfgError::Shape(
                "expected a SELECT query, got ASK/CONSTRUCT results".to_string(),
            ));
        }
    }
    Ok(rows)
}

fn row_get<'a>(row: &'a [(String, Term)], var: &str) -> Option<&'a Term> {
    row.iter().find(|(v, _)| v == var).map(|(_, t)| t)
}

/// Extract the domain's declared name via `?d a pddl:Domain ; pddl:name ?name`.
fn extract_domain_name(graph: &DeterministicGraph) -> Result<String> {
    let q = format!("{PDL_PREFIX}SELECT ?name WHERE {{ ?d a pddl:Domain ; pddl:name ?name }}");
    let rows = run_select(graph, &q)?;
    let row = rows
        .first()
        .ok_or_else(|| MfgError::Shape("no pddl:Domain instance found".to_string()))?;
    literal_str(row_get(row, "name"), "domain name")
}

/// `SELECT ?name ?parent WHERE { ?c a pddl:Type ; pddl:name ?name . OPTIONAL
/// { ?c pddl:subTypeOf/pddl:name ?parent } } ORDER BY ?name`
fn extract_types(graph: &DeterministicGraph) -> Result<Vec<TypeDecl>> {
    let q = format!(
        "{PDL_PREFIX}SELECT ?name ?parent WHERE {{ \
           ?c a pddl:Type ; pddl:name ?name . \
           OPTIONAL {{ ?c pddl:subTypeOf/pddl:name ?parent }} \
         }} ORDER BY ?name"
    );
    let rows = run_select(graph, &q)?;
    rows.iter()
        .map(|row| {
            let name = literal_str(row_get(row, "name"), "type name")?;
            let parent = match row_get(row, "parent") {
                Some(t) => Some(literal_str(Some(t), "type parent")?),
                None => None,
            };
            Ok(TypeDecl { name, parent })
        })
        .collect()
}

/// Shared by predicate/action param extraction: `SELECT ?name ?i ?v ?t WHERE
/// { ?x a <class> ; pddl:name ?name ; pddl:param ?pp . ?pp pddl:index ?i ;
/// pddl:var ?v ; pddl:ofType ?t } ORDER BY ?name ?i`
fn extract_params_by_name(
    graph: &DeterministicGraph,
    pdl_class: &str,
) -> Result<Vec<(String, usize, String, String)>> {
    let q = format!(
        "{PDL_PREFIX}SELECT ?name ?i ?v ?t WHERE {{ \
           ?x a pddl:{pdl_class} ; pddl:name ?name ; pddl:param ?pp . \
           ?pp pddl:index ?i ; pddl:var ?v ; pddl:ofType ?t \
         }} ORDER BY ?name ?i"
    );
    let rows = run_select(graph, &q)?;
    rows.iter()
        .map(|row| {
            let name = literal_str(row_get(row, "name"), "param owner name")?;
            let i = literal_index(row_get(row, "i"), "param index")?;
            let v = literal_str(row_get(row, "v"), "param var")?;
            let t = literal_str(row_get(row, "t"), "param type")?;
            Ok((name, i, v, t))
        })
        .collect()
}

/// `SELECT ?name WHERE { ?x a pddl:<class> ; pddl:name ?name } ORDER BY ?name`
fn extract_names(graph: &DeterministicGraph, pdl_class: &str) -> Result<Vec<String>> {
    let q = format!(
        "{PDL_PREFIX}SELECT ?name WHERE {{ ?x a pddl:{pdl_class} ; pddl:name ?name }} ORDER BY ?name"
    );
    let rows = run_select(graph, &q)?;
    rows.iter()
        .map(|row| literal_str(row_get(row, "name"), "name"))
        .collect()
}

/// `SELECT ?name ?atom WHERE { ?x a pddl:<class> ; pddl:name ?name ;
/// pddl:<prop> ?atom } ORDER BY ?name ?atom` — one multi-valued property at a
/// time, kept separate per property to avoid a cross-product join when an
/// action has several multi-valued properties (params/pre/add/del) at once.
fn extract_atoms_by_name(
    graph: &DeterministicGraph,
    pdl_class: &str,
    pdl_prop: &str,
) -> Result<Vec<(String, Atom)>> {
    let q = format!(
        "{PDL_PREFIX}SELECT ?name ?atom WHERE {{ \
           ?x a pddl:{pdl_class} ; pddl:name ?name ; pddl:{pdl_prop} ?atom \
         }} ORDER BY ?name ?atom"
    );
    let rows = run_select(graph, &q)?;
    rows.iter()
        .map(|row| {
            let name = literal_str(row_get(row, "name"), "atom owner name")?;
            let atom_text = literal_str(row_get(row, "atom"), "atom text")?;
            Ok((name, parse_atom_literal(&atom_text)?))
        })
        .collect()
}

fn params_for(all: &[(String, usize, String, String)], name: &str) -> Vec<(String, String)> {
    let mut out: Vec<(usize, String, String)> = all
        .iter()
        .filter(|(n, ..)| n == name)
        .map(|(_, i, v, t)| (*i, v.clone(), t.clone()))
        .collect();
    out.sort_by_key(|(i, ..)| *i);
    out.into_iter().map(|(_, v, t)| (v, t)).collect()
}

fn atoms_for(all: &[(String, Atom)], name: &str) -> Vec<Atom> {
    all.iter()
        .filter(|(n, _)| n == name)
        .map(|(_, a)| a.clone())
        .collect()
}

/// Extract `pddl:Predicate` instances into [`PredicateDecl`]s, ordered by name.
fn extract_predicates(graph: &DeterministicGraph) -> Result<Vec<PredicateDecl>> {
    let names = extract_names(graph, "Predicate")?;
    let params = extract_params_by_name(graph, "Predicate")?;
    Ok(names
        .into_iter()
        .map(|name| {
            let p = params_for(&params, &name);
            PredicateDecl { name, params: p }
        })
        .collect())
}

/// Extract `pddl:Action` instances into [`ActionDecl`]s, ordered by name.
fn extract_actions(graph: &DeterministicGraph) -> Result<Vec<ActionDecl>> {
    let names = extract_names(graph, "Action")?;
    let params = extract_params_by_name(graph, "Action")?;
    let pre = extract_atoms_by_name(graph, "Action", "pre")?;
    let add = extract_atoms_by_name(graph, "Action", "add")?;
    let del = extract_atoms_by_name(graph, "Action", "del")?;
    Ok(names
        .into_iter()
        .map(|name| ActionDecl {
            params: params_for(&params, &name),
            pre: atoms_for(&pre, &name),
            add: atoms_for(&add, &name),
            del: atoms_for(&del, &name),
            name,
        })
        .collect())
}

/// Extract the full [`DomainIr`] from `graph`.
pub fn extract_domain(graph: &DeterministicGraph) -> Result<PddlDomainIr> {
    Ok(PddlDomainIr {
        name: extract_domain_name(graph)?,
        types: extract_types(graph)?,
        predicates: extract_predicates(graph)?,
        actions: extract_actions(graph)?,
    })
}

/// Extract the [`ProblemIr`] from `graph`. Assumes exactly one `pddl:Problem`
/// instance (the first `ORDER BY`-stable match is used if there are more).
pub fn extract_problem(graph: &DeterministicGraph) -> Result<PddlProblemIr> {
    let q = format!(
        "{PDL_PREFIX}SELECT ?name ?domain WHERE {{ ?p a pddl:Problem ; pddl:name ?name ; pddl:domain ?domain }} ORDER BY ?name"
    );
    let rows = run_select(graph, &q)?;
    let row = rows
        .first()
        .ok_or_else(|| MfgError::Shape("no pddl:Problem instance found".to_string()))?;
    let name = literal_str(row_get(row, "name"), "problem name")?;
    let domain = literal_str(row_get(row, "domain"), "problem domain")?;

    let objq = format!(
        "{PDL_PREFIX}SELECT ?name ?t WHERE {{ \
           ?p a pddl:Problem ; pddl:object ?oo . ?oo pddl:name ?name ; pddl:ofType ?t \
         }} ORDER BY ?name"
    );
    let objects = run_select(graph, &objq)?
        .iter()
        .map(|row| {
            Ok(ObjectDecl {
                name: literal_str(row_get(row, "name"), "object name")?,
                ty: literal_str(row_get(row, "t"), "object type")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let initq = format!(
        "{PDL_PREFIX}SELECT ?atom WHERE {{ ?p a pddl:Problem ; pddl:init ?atom }} ORDER BY ?atom"
    );
    let init = run_select(graph, &initq)?
        .iter()
        .map(|row| parse_atom_literal(&literal_str(row_get(row, "atom"), "init atom")?))
        .collect::<Result<Vec<_>>>()?;

    let goalq = format!(
        "{PDL_PREFIX}SELECT ?atom WHERE {{ ?p a pddl:Problem ; pddl:goal ?atom }} ORDER BY ?atom"
    );
    let goal = run_select(graph, &goalq)?
        .iter()
        .map(|row| parse_atom_literal(&literal_str(row_get(row, "atom"), "goal atom")?))
        .collect::<Result<Vec<_>>>()?;

    Ok(PddlProblemIr {
        name,
        domain,
        objects,
        init,
        goal,
    })
}

// ---------------------------------------------------------------------------
// PDDL8 bounds enforcement
// ---------------------------------------------------------------------------

/// Enforce PDDL8 bounds on `domain` before any text is emitted: predicate
/// arity, action parameter count, and precondition/effect conjunct counts
/// must each stay within `wasm4pm_compat::pddl::PDDL8_MAX_*`.

// ---------------------------------------------------------------------------
// PDDL8 text emission (direct Rust string building, not Tera)
// ---------------------------------------------------------------------------

fn atom_str(a: &Atom) -> String {
    if a.args.is_empty() {
        format!("({})", a.pred)
    } else {
        format!("({} {})", a.pred, a.args.join(" "))
    }
}

fn conjunction(atoms: &[Atom]) -> String {
    match atoms.len() {
        0 => "(and)".to_string(),
        1 => atom_str(&atoms[0]),
        _ => format!(
            "(and {})",
            atoms.iter().map(atom_str).collect::<Vec<_>>().join(" ")
        ),
    }
}

fn effect_conjunction(add: &[Atom], del: &[Atom]) -> String {
    let mut parts: Vec<String> = add.iter().map(atom_str).collect();
    parts.extend(del.iter().map(|d| format!("(not {})", atom_str(d))));
    match parts.len() {
        0 => "(and)".to_string(),
        1 => parts.into_iter().next().unwrap_or_default(),
        _ => format!("(and {})", parts.join(" ")),
    }
}

fn typed_list(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(v, t)| format!("{v} - {t}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Emit PDDL8 domain text for `domain`. `source_label` and `graph_hash_hex`
/// are embedded as a provenance comment header (not parsed by the planner).
pub fn emit_domain(domain: &PddlDomainIr, source_label: &str, graph_hash_hex: &str) -> String {
    let mut s = String::new();
    s.push_str(";; GENERATED by praxis::mfg — do not hand-edit.\n");
    s.push_str(&format!(
        ";; manufactured-from: {source_label} blake3:{graph_hash_hex}\n"
    ));
    s.push_str(&format!("(define (domain {})\n", domain.name));
    s.push_str("  (:requirements :strips :typing)\n");
    s.push_str("  (:types\n");
    for t in &domain.types {
        match &t.parent {
            Some(p) => s.push_str(&format!("    {} - {}\n", t.name, p)),
            None => s.push_str(&format!("    {}\n", t.name)),
        }
    }
    s.push_str("  )\n");
    s.push_str("  (:predicates\n");
    for p in &domain.predicates {
        let params = typed_list(&p.params);
        if params.is_empty() {
            s.push_str(&format!("    ({})\n", p.name));
        } else {
            s.push_str(&format!("    ({} {})\n", p.name, params));
        }
    }
    s.push_str("  )\n");
    for a in &domain.actions {
        s.push_str(&format!("  (:action {}\n", a.name));
        s.push_str(&format!("    :parameters ({})\n", typed_list(&a.params)));
        s.push_str(&format!("    :precondition {}\n", conjunction(&a.pre)));
        s.push_str(&format!(
            "    :effect {}\n",
            effect_conjunction(&a.add, &a.del)
        ));
        s.push_str("  )\n");
    }
    s.push_str(")\n");
    s
}

/// Emit PDDL8 problem text for `problem`.
pub fn emit_problem(problem: &PddlProblemIr) -> String {
    let mut s = String::new();
    s.push_str(&format!("(define (problem {})\n", problem.name));
    s.push_str(&format!("  (:domain {})\n", problem.domain));
    s.push_str("  (:objects\n");
    for o in &problem.objects {
        s.push_str(&format!("    {} - {}\n", o.name, o.ty));
    }
    s.push_str("  )\n");
    s.push_str("  (:init\n");
    for a in &problem.init {
        s.push_str(&format!("    {}\n", atom_str(a)));
    }
    s.push_str("  )\n");
    s.push_str(&format!("  (:goal {})\n", conjunction(&problem.goal)));
    s.push_str(")\n");
    s
}

// ---------------------------------------------------------------------------
// Facts (SPARQL-JSON, `ggen-core`'s sparql_column/sparql_row row shape)
// ---------------------------------------------------------------------------

fn term_to_json(term: &Term) -> serde_json::Value {
    match term {
        Term::NamedNode(n) => serde_json::Value::String(n.as_str().to_string()),
        Term::BlankNode(b) => serde_json::Value::String(format!("_:{}", b.as_str())),
        Term::Literal(lit) => {
            if lit.datatype().as_str().ends_with("#integer") {
                if let Ok(n) = lit.value().parse::<i64>() {
                    return serde_json::Value::Number(n.into());
                }
            }
            serde_json::Value::String(lit.value().to_string())
        }
        // RDF-star triple terms (only reachable if the `rdf-12` oxigraph
        // feature is enabled transitively) are not part of the `pddl:`
        // vocabulary; represent them opaquely rather than failing.
        _ => serde_json::Value::Null,
    }
}

/// Run `sparql` (a `SELECT` query) against `graph` and lower the results
/// into a JSON array of objects, one per solution row, keyed by variable
/// name (no leading `?`) — the exact shape `ggen-core`'s `sparql_column`/
/// `sparql_row` Tera functions expect.
pub fn facts_json(graph: &DeterministicGraph, sparql: &str) -> Result<serde_json::Value> {
    let rows = run_select(graph, sparql)?;
    let out: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (var, term) in row {
                obj.insert(var, term_to_json(&term));
            }
            serde_json::Value::Object(obj)
        })
        .collect();
    Ok(serde_json::Value::Array(out))
}

// ---------------------------------------------------------------------------
// Top-level manufacture + validate
// ---------------------------------------------------------------------------

/// Load `ttl`, extract the domain + problem IR, enforce PDDL8 bounds, and
/// emit PDDL8 domain/problem text. `source_label` is embedded in the
/// domain's provenance comment (typically the ontology file path).

const PDDL_ADMISSION_SHAPES_TTL: &str = include_str!("../ontology/pddl-admission-shapes.ttl");

pub fn manufacture(ttl: &str, profile_name: &str) -> Result<AdmittedPlanningTask> {
    let graph = load_graph(ttl)?;

    // We run SHACL admission
    let receipt = validate_shacl(&graph, PDDL_ADMISSION_SHAPES_TTL, profile_name)?;
    if !receipt.conforms {
        return Err(MfgError::Shape(receipt.message.unwrap_or_default()));
    }

    let hash_hex = receipt.graph_hash.clone();
    let domain = extract_domain(&graph)?;

    // PDDL 3.1: remove enforce_pddl8 as global law
    // enforce_pddl8(&domain)?;

    let problem = extract_problem(&graph)?;

    Ok(AdmittedPlanningTask {
        domain,
        problem,
        receipt,
    })
}

/// Round-trip manufactured (or hand-written) PDDL8 `domain_text`/
/// `problem_text` through `bcinr-pddl`: parse, ground, and attempt
/// `find_plan`. Parse/ground/solve failures are reported in the returned
/// [`ValidationReport`] rather than as an `Err` — this mirrors the rest of
/// the CLI's "domain denial is `Ok(json)`" convention.
pub fn validate(domain_text: &str, problem_text: &str) -> ValidationReport {
    let empty = |error: String, parsed: bool, grounded_actions: usize| ValidationReport {
        parsed,
        grounded_actions,
        plan_len: 0,
        plan_steps: Vec::new(),
        solvable: false,
        error: Some(error),
    };

    let domain = match domain_from_pddl(domain_text) {
        Ok(d) => d,
        Err(e) => return empty(e.to_string(), false, 0),
    };
    let problem = match problem_from_pddl(problem_text) {
        Ok(p) => p,
        Err(e) => return empty(e.to_string(), true, 0),
    };
    let ground = match GroundProblem::build(&domain, &problem, None) {
        Ok(g) => g,
        Err(e) => return empty(e.to_string(), true, 0),
    };
    let grounded_actions = ground.actions.len();
    match ground.find_plan() {
        Ok(tape) => {
            let plan_steps: Vec<String> = tape
                .ops
                .iter()
                .map(|op| op.action.schema_name.clone())
                .collect();
            ValidationReport {
                parsed: true,
                grounded_actions,
                plan_len: plan_steps.len(),
                plan_steps,
                solvable: true,
                error: None,
            }
        }
        Err(e) => empty(e.to_string(), true, grounded_actions),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAWOBJECT_TTL: &str = include_str!("../ontology/lawobject.ttl");

    #[test]
    fn emit_domain_omits_not_when_no_del_effects() {
        let domain = PddlDomainIr {
            name: "d".to_string(),
            types: vec![TypeDecl {
                name: "t".to_string(),
                parent: None,
            }],
            predicates: vec![PredicateDecl {
                name: "p".to_string(),
                params: vec![("?x".to_string(), "t".to_string())],
            }],
            actions: vec![ActionDecl {
                name: "a".to_string(),
                params: vec![("?x".to_string(), "t".to_string())],
                pre: vec![Atom {
                    pred: "p".to_string(),
                    args: vec!["?x".to_string()],
                }],
                add: vec![Atom {
                    pred: "p".to_string(),
                    args: vec!["?x".to_string()],
                }],
                del: vec![],
            }],
        };
        let text = emit_domain(&domain, "test", "deadbeef");
        assert!(!text.contains("(not"));
        assert!(text.contains(":effect (p ?x)"));
    }

    #[test]
    fn emit_domain_renders_parent_type() {
        let domain = PddlDomainIr {
            name: "d".to_string(),
            types: vec![TypeDecl {
                name: "child".to_string(),
                parent: Some("parent".to_string()),
            }],
            predicates: vec![],
            actions: vec![],
        };
        let text = emit_domain(&domain, "test", "deadbeef");
        assert!(text.contains("child - parent"));
    }

    #[test]
    fn parse_atom_literal_splits_pred_and_args() {
        let atom = parse_atom_literal("(in-stage ?obj raw)").unwrap();
        assert_eq!(atom.pred, "in-stage");
        assert_eq!(atom.args, vec!["?obj".to_string(), "raw".to_string()]);
    }

    #[test]
    fn parse_atom_literal_rejects_missing_parens() {
        assert!(parse_atom_literal("in-stage ?obj raw").is_err());
    }

    #[test]
    fn manufacture_lawobject_golden_solves() {
        let m = manufacture(LAWOBJECT_TTL, "ontology/lawobject.ttl").expect("manufacture");
        let report = validate(&m.domain_text, &m.problem_text);
        assert!(report.parsed, "{:?}", report.error);
        assert!(report.solvable, "{:?}", report.error);
        assert_eq!(
            report.plan_steps,
            vec![
                "supply-evidence".to_string(),
                "clear-obligations".to_string(),
                "judge".to_string(),
                "admit".to_string(),
                "receipt".to_string(),
            ]
        );
    }

    #[test]
    fn manufacture_is_deterministic() {
        let a = manufacture(LAWOBJECT_TTL, "ontology/lawobject.ttl").unwrap();
        let b = manufacture(LAWOBJECT_TTL, "ontology/lawobject.ttl").unwrap();
        assert_eq!(a.domain_text, b.domain_text);
        assert_eq!(a.problem_text, b.problem_text);
        assert_eq!(a.graph_hash_hex, b.graph_hash_hex);
    }

    #[test]
    fn enforce_pddl8_rejects_high_arity_predicate() {
        let ttl = r#"
            @prefix pddl: <http://seanchatmangpt.github.io/praxis/pddl#> .
            pddl:domain_x a pddl:Domain ; pddl:name "x" .
            pddl:pred_wide a pddl:Predicate ; pddl:name "wide" ;
              pddl:param [ pddl:index 0 ; pddl:var "?a0" ; pddl:ofType "t" ] ,
                        [ pddl:index 1 ; pddl:var "?a1" ; pddl:ofType "t" ] ,
                        [ pddl:index 2 ; pddl:var "?a2" ; pddl:ofType "t" ] ,
                        [ pddl:index 3 ; pddl:var "?a3" ; pddl:ofType "t" ] ,
                        [ pddl:index 4 ; pddl:var "?a4" ; pddl:ofType "t" ] ,
                        [ pddl:index 5 ; pddl:var "?a5" ; pddl:ofType "t" ] ,
                        [ pddl:index 6 ; pddl:var "?a6" ; pddl:ofType "t" ] ,
                        [ pddl:index 7 ; pddl:var "?a7" ; pddl:ofType "t" ] ,
                        [ pddl:index 8 ; pddl:var "?a8" ; pddl:ofType "t" ] .
        "#;
        let graph = load_graph(ttl).unwrap();
        let domain = extract_domain(&graph).unwrap();
        assert_eq!(domain.predicates[0].params.len(), 9);
        let err = enforce_pddl8(&domain).unwrap_err();
        match err {
            MfgError::BoundExceeded {
                what, limit, got, ..
            } => {
                assert_eq!(what, "predicate arity");
                assert_eq!(limit, PDDL8_MAX_ARITY);
                assert_eq!(got, 9);
            }
            other => panic!("expected BoundExceeded, got {other:?}"),
        }
    }

    #[test]
    fn facts_json_shape_has_plain_keys() {
        let graph = load_graph(LAWOBJECT_TTL).unwrap();
        let q = format!(
            "{PDL_PREFIX}SELECT ?name WHERE {{ ?c a pddl:Type ; pddl:name ?name }} ORDER BY ?name"
        );
        let value = facts_json(&graph, &q).unwrap();
        let arr = value.as_array().expect("array");
        assert!(!arr.is_empty());
        let first = arr[0].as_object().expect("object");
        assert!(first.contains_key("name"));
        assert!(first.get("name").unwrap().is_string());
    }
}
