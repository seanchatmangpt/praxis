//! PROJ-703 — Renderer: pddl-strips graphs → deterministic PDDL text via
//! the on-disk `templates/decomp-problem.template.pddl` /
//! `templates/decomp-domain.template.pddl` templates, feeding the unchanged
//! `bcinr_pddl` parse → `GroundProblem::build` → `find_plan` path.
//!
//! Determinism: every atom/object/action list is sorted lexicographically
//! before substitution; same graph → byte-identical PDDL. Round-trip law
//! (property test): lift ∘ render is identity on the (objects, init, goal)
//! atom sets modulo ordering.
//!
//! Zero inline PDDL/SPARQL: the `(define …)` skeletons live in the
//! templates; every query is a `queries/decomp/*.rq` file with `{KEY}`
//! placeholders substituted via `str::replace`.

use std::collections::BTreeMap;

use bcinr_pddl::parse::{domain_from_pddl, problem_from_pddl};
use bcinr_pddl::{Pddl8Domain, Pddl8Problem};
use oxigraph::store::Store;

use crate::bench::roles::select_rows;
use crate::bench::templates::QuerySet;
use crate::powl::CngRefusal;

/// One structured atom row assembled from `atom-args.rq`: predicate name
/// plus positional argument symbols.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AtomContent {
    pub pred: String,
    pub args: Vec<String>,
}

impl AtomContent {
    /// Canonical ground label `pred(a0,a1)` / bare `pred`.
    ///
    /// # Complexity
    /// O(arity).
    pub fn label(&self) -> String {
        if self.args.is_empty() {
            self.pred.clone()
        } else {
            format!("{}({})", self.pred, self.args.join(","))
        }
    }

    /// PDDL s-expression `(pred a0 a1)`.
    ///
    /// # Complexity
    /// O(arity).
    pub fn sexpr(&self) -> String {
        if self.args.is_empty() {
            format!("({})", self.pred)
        } else {
            format!("({} {})", self.pred, self.args.join(" "))
        }
    }
}

/// Reads every atom-bearing node (plain Atoms and Add/Del effect nodes) of
/// `store` into `IRI → AtomContent`, positions taken from `argIndex`.
///
/// # Errors
/// `CNG_R01 MalformedTtl` for malformed rows or index parse failures.
///
/// # Complexity
/// O(rows log rows) over the `atom-args.rq` result.
pub fn atom_contents(
    store: &Store,
    queries: &QuerySet,
) -> Result<BTreeMap<String, AtomContent>, CngRefusal> {
    let rows = select_rows(store, queries.get("atom-args")?)?;
    let mut preds: BTreeMap<String, String> = BTreeMap::new();
    let mut args: BTreeMap<String, BTreeMap<u64, String>> = BTreeMap::new();
    for row in rows {
        let atom = row
            .get("atom")
            .cloned()
            .ok_or_else(|| CngRefusal::MalformedTtl("atom-args row missing ?atom".to_string()))?;
        let pred = row
            .get("pred")
            .cloned()
            .ok_or_else(|| CngRefusal::MalformedTtl("atom-args row missing ?pred".to_string()))?;
        preds.insert(atom.clone(), pred);
        if let (Some(idx), Some(val)) = (row.get("idx"), row.get("val")) {
            let idx: u64 = idx.parse().map_err(|e| {
                CngRefusal::MalformedTtl(format!("atom-args argIndex parse ({idx}): {e}"))
            })?;
            args.entry(atom).or_default().insert(idx, val.clone());
        }
    }
    let mut out = BTreeMap::new();
    for (atom, pred) in preds {
        let ordered: Vec<String> = args
            .remove(&atom)
            .unwrap_or_default()
            .into_values()
            .collect();
        out.insert(
            atom,
            AtomContent {
                pred,
                args: ordered,
            },
        );
    }
    Ok(out)
}

/// Sorted s-expression list for a set of atom IRIs, resolved through the
/// atom-content map.
///
/// # Errors
/// `CNG_R01` when an IRI has no content rows.
///
/// # Complexity
/// O(n log n) over the atom IRIs.
fn sexpr_list(
    iris: &[String],
    contents: &BTreeMap<String, AtomContent>,
    what: &str,
) -> Result<String, CngRefusal> {
    let mut exprs = Vec::with_capacity(iris.len());
    for iri in iris {
        let content = contents.get(iri).ok_or_else(|| {
            CngRefusal::MalformedTtl(format!(
                "{what} atom <{iri}> has no predicateName/argument content"
            ))
        })?;
        exprs.push(content.sexpr());
    }
    exprs.sort();
    exprs.dedup();
    Ok(exprs.join(" "))
}

/// Single-column SELECT helper: runs a `{KEY}`-substituted query and
/// returns the named column, in query order.
///
/// # Complexity
/// O(rows).
fn column(store: &Store, query_text: &str, var: &str) -> Result<Vec<String>, CngRefusal> {
    let rows = select_rows(store, query_text)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(
            row.get(var)
                .cloned()
                .ok_or_else(|| CngRefusal::MalformedTtl(format!("query row missing ?{var}")))?,
        );
    }
    Ok(out)
}

/// Renders one `pddl:Problem` graph node to deterministic PDDL problem text
/// through the on-disk template.
///
/// # Errors
/// `CNG_R01` for graph/content failures; `CNG_R03 MissingProblem` when the
/// problem has no goal atoms (an empty goal renders no lawful PDDL).
///
/// # Complexity
/// O(n log n) over the problem's objects + init + goal atoms.
pub fn render_problem(
    store: &Store,
    problem_iri: &str,
    queries: &QuerySet,
    template: &str,
) -> Result<String, CngRefusal> {
    let meta = select_rows(
        store,
        &queries
            .get("problem-meta")?
            .replace("{PROBLEM}", problem_iri),
    )?;
    let meta = meta.first().ok_or_else(|| {
        CngRefusal::MissingProblem(format!("<{problem_iri}> has no problemName/fromDomain"))
    })?;
    let problem_name = meta.get("problemName").cloned().ok_or_else(|| {
        CngRefusal::MalformedTtl("problem-meta row missing ?problemName".to_string())
    })?;
    let domain_name = meta.get("domainName").cloned().ok_or_else(|| {
        CngRefusal::MalformedTtl("problem-meta row missing ?domainName".to_string())
    })?;

    let mut objects = column(
        store,
        &queries
            .get("problem-objects")?
            .replace("{PROBLEM}", problem_iri),
        "name",
    )?;
    objects.sort();
    objects.dedup();

    let contents = atom_contents(store, queries)?;
    let link_query = queries.get("problem-link-atoms")?;
    let init_iris = column(
        store,
        &link_query
            .replace("{PROBLEM}", problem_iri)
            .replace("{LINK}", "initAtom"),
        "atom",
    )?;
    let goal_iris = column(
        store,
        &link_query
            .replace("{PROBLEM}", problem_iri)
            .replace("{LINK}", "goalAtom"),
        "atom",
    )?;
    if goal_iris.is_empty() {
        return Err(CngRefusal::MissingProblem(format!(
            "<{problem_iri}> has no goal atoms; an empty goal is not renderable STRIPS"
        )));
    }
    let init = sexpr_list(&init_iris, &contents, "init")?;
    let goal = sexpr_list(&goal_iris, &contents, "goal")?;

    Ok(crate::bench::fill_template(
        template,
        &[
            ("PROBLEM_NAME", problem_name.as_str()),
            ("DOMAIN_NAME", domain_name.as_str()),
            ("OBJECTS", objects.join(" ").as_str()),
            ("INIT", init.as_str()),
            ("GOAL", goal.as_str()),
        ],
    ))
}

/// Renders one `pddl:Domain` graph node (SCHEMA-level: parameters and
/// ?-variable atoms, e.g. `examples/pddl-strips-potato.ttl`) to
/// deterministic PDDL domain text through the on-disk template. Not
/// intended for lifted-ground graphs (whose actions share schema names).
///
/// # Errors
/// `CNG_R01` for graph failures; `CNG_R05 UnsupportedConstruct` for
/// duplicate action names or predicate arity conflicts.
///
/// # Complexity
/// O(A · c log c) over A actions with ≤ c conjuncts, plus O(P log P)
/// predicate collection.
pub fn render_domain(
    store: &Store,
    domain_iri: &str,
    queries: &QuerySet,
    template: &str,
) -> Result<String, CngRefusal> {
    let name_rows = select_rows(
        store,
        &queries
            .get("domain-actions")?
            .replace("{DOMAIN}", domain_iri),
    )?;
    let contents = atom_contents(store, queries)?;

    // Predicate declarations from every atom's (name, arity); conflict on
    // arity is refused. O(atoms log atoms).
    let mut arities: BTreeMap<String, usize> = BTreeMap::new();
    for content in contents.values() {
        match arities.get(&content.pred) {
            Some(&a) if a != content.args.len() => {
                return Err(CngRefusal::UnsupportedConstruct(format!(
                    "predicate {} used with arities {a} and {}; STRIPS predicates have one arity",
                    content.pred,
                    content.args.len()
                )));
            }
            _ => {
                arities.insert(content.pred.clone(), content.args.len());
            }
        }
    }
    let mut predicates = Vec::with_capacity(arities.len());
    for (pred, arity) in &arities {
        let vars: Vec<String> = (0..*arity).map(|i| format!("?x{i}")).collect();
        if vars.is_empty() {
            predicates.push(format!("({pred})"));
        } else {
            predicates.push(format!("({pred} {})", vars.join(" ")));
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut action_blocks = Vec::with_capacity(name_rows.len());
    for row in &name_rows {
        let action = row.get("action").cloned().ok_or_else(|| {
            CngRefusal::MalformedTtl("domain-actions row missing ?action".to_string())
        })?;
        let name = row.get("name").cloned().ok_or_else(|| {
            CngRefusal::MalformedTtl("domain-actions row missing ?name".to_string())
        })?;
        if !seen.insert(name.clone()) {
            return Err(CngRefusal::UnsupportedConstruct(format!(
                "duplicate actionName {name:?} in domain <{domain_iri}>; the domain renderer \
                 covers schema-level graphs only"
            )));
        }
        let param_rows = select_rows(
            store,
            &queries.get("action-params")?.replace("{ACTION}", &action),
        )?;
        let mut params = Vec::with_capacity(param_rows.len());
        for p in &param_rows {
            params.push(p.get("name").cloned().ok_or_else(|| {
                CngRefusal::MalformedTtl("action-params row missing ?name".to_string())
            })?);
        }
        let pre_iris = column(
            store,
            &queries
                .get("action-pre-atoms")?
                .replace("{ACTION}", &action),
            "atom",
        )?;
        let pre = sexpr_list(&pre_iris, &contents, "precondition")?;
        let eff_rows = select_rows(
            store,
            &queries
                .get("action-effect-atoms")?
                .replace("{ACTION}", &action),
        )?;
        let mut effects = Vec::with_capacity(eff_rows.len());
        for e in &eff_rows {
            let atom = e.get("atom").cloned().ok_or_else(|| {
                CngRefusal::MalformedTtl("action-effect-atoms row missing ?atom".to_string())
            })?;
            let kind = e.get("kind").cloned().ok_or_else(|| {
                CngRefusal::MalformedTtl("action-effect-atoms row missing ?kind".to_string())
            })?;
            let content = contents.get(&atom).ok_or_else(|| {
                CngRefusal::MalformedTtl(format!("effect atom <{atom}> has no content"))
            })?;
            if kind.ends_with("DelEffect") {
                effects.push(format!("(not {})", content.sexpr()));
            } else {
                effects.push(content.sexpr());
            }
        }
        effects.sort();
        let mut block = String::new();
        block.push_str("  (:action ");
        block.push_str(&name);
        block.push_str("\n    :parameters (");
        block.push_str(&params.join(" "));
        block.push_str(")\n    :precondition (and ");
        block.push_str(&pre);
        block.push_str(")\n    :effect (and ");
        block.push_str(&effects.join(" "));
        block.push_str("))");
        action_blocks.push(block);
    }
    action_blocks.sort();

    // The domainName literal is fetched with a typed pattern scan (no
    // inline SPARQL).
    let domain_name = domain_name_of(store, domain_iri)?;

    Ok(crate::bench::fill_template(
        template,
        &[
            ("DOMAIN_NAME", domain_name.as_str()),
            ("PREDICATES", predicates.join(" ").as_str()),
            ("ACTIONS", action_blocks.join("\n").as_str()),
        ],
    ))
}

/// `pddl:domainName` of a Domain node via a typed pattern scan.
///
/// # Complexity
/// O(matches).
fn domain_name_of(store: &Store, domain_iri: &str) -> Result<String, CngRefusal> {
    use oxigraph::model::{NamedNodeRef, Term};
    let subject = NamedNodeRef::new(domain_iri)
        .map_err(|e| CngRefusal::MalformedTtl(format!("domain IRI {domain_iri}: {e}")))?;
    let pred_iri = format!("{}domainName", super::lift::PDDL_STRIPS_PREFIX);
    let pred = NamedNodeRef::new(&pred_iri)
        .map_err(|e| CngRefusal::MalformedTtl(format!("domainName IRI: {e}")))?;
    let mut values: Vec<String> = Vec::new();
    for quad in store.quads_for_pattern(Some(subject.into()), Some(pred), None, None) {
        let quad = quad.map_err(|e| CngRefusal::MalformedTtl(format!("domainName scan: {e}")))?;
        if let Term::Literal(lit) = quad.object {
            values.push(lit.value().to_string());
        }
    }
    values.sort();
    values.into_iter().next().ok_or_else(|| {
        CngRefusal::MissingDomain(format!("<{domain_iri}> carries no pddl:domainName"))
    })
}

/// Graph → parsed-surface bridge: renders the (single) Problem and its
/// Domain out of a pddl-strips graph and parses both through the unchanged
/// bcinr parser. This is how a hand-authored pddl-strips instance (e.g.
/// `examples/pddl-strips-potato.ttl`) enters the proven planner path.
///
/// # Errors
/// `CNG_R03 MissingProblem` when the graph holds no Problem; `CNG_R05` when
/// it holds more than one; `CNG_R01` for render/parse failures.
///
/// # Complexity
/// Render cost (see `render_domain`/`render_problem`) + parser cost.
pub fn strips_graph_to_surface(
    store: &Store,
    queries: &QuerySet,
    domain_template: &str,
    problem_template: &str,
) -> Result<(Pddl8Domain, Pddl8Problem), CngRefusal> {
    let rows = select_rows(store, queries.get("list-problems")?)?;
    if rows.is_empty() {
        return Err(CngRefusal::MissingProblem(
            "pddl-strips graph holds no pddl:Problem".to_string(),
        ));
    }
    if rows.len() > 1 {
        return Err(CngRefusal::UnsupportedConstruct(format!(
            "pddl-strips graph holds {} Problems; the graph→surface bridge covers exactly one",
            rows.len()
        )));
    }
    let problem_iri = rows[0].get("problem").cloned().ok_or_else(|| {
        CngRefusal::MalformedTtl("list-problems row missing ?problem".to_string())
    })?;
    let domain_iri = rows[0]
        .get("domain")
        .cloned()
        .ok_or_else(|| CngRefusal::MalformedTtl("list-problems row missing ?domain".to_string()))?;

    let domain_text = render_domain(store, &domain_iri, queries, domain_template)?;
    let problem_text = render_problem(store, &problem_iri, queries, problem_template)?;
    let domain = domain_from_pddl(&domain_text)
        .map_err(|e| CngRefusal::MalformedTtl(format!("rendered domain failed to parse: {e:?}")))?;
    let problem = problem_from_pddl(&problem_text).map_err(|e| {
        CngRefusal::MalformedTtl(format!("rendered problem failed to parse: {e:?}"))
    })?;
    Ok((domain, problem))
}
