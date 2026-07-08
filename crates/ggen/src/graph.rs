//! Deterministic RDF graph over an in-memory oxigraph [`Store`].
//!
//! Provides [`DeterministicGraph`] — Turtle loading, SPARQL querying, and a
//! BLAKE3 `state_hash` computed over canonicalized, sorted N-Quads — plus a
//! [`Delta`] type describing additions/deletions between two graphs.
//!
//! Canonicalization: graphs without blank nodes are sorted lexicographically
//! by their N-Quads string form; graphs with blank nodes go through a bounded
//! color-refinement pass (5 iterations of BLAKE3 neighborhood signatures)
//! and blank nodes are relabelled `c14n{i}` in signature order so isomorphic
//! graphs hash identically regardless of blank-node labels.

use std::collections::{BTreeSet, HashMap, HashSet};

use oxigraph::{
    io::RdfFormat,
    model::{BlankNode, GraphName, NamedOrBlankNode, Quad, Term},
    sparql::{QueryResults, SparqlEvaluator},
    store::Store,
};

use crate::error::{AppError, Result};

/// Number of color-refinement iterations for blank-node canonicalization.
/// Five rounds are sufficient for the small ontology graphs ggen operates on.
const REFINEMENT_ITERATIONS: usize = 5;

/// An in-memory RDF store with deterministic state hashing.
pub struct DeterministicGraph {
    store: Store,
}

impl DeterministicGraph {
    /// Create an empty in-memory graph.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-001]` if the oxigraph store cannot be initialized.
    pub fn new() -> Result<Self> {
        let store = Store::new()
            .map_err(|e| AppError::fm_graph(1, format!("failed to create in-memory store: {e}")))?;
        Ok(Self { store })
    }

    /// Parse and insert Turtle content, returning the number of quads added.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-002]` on Turtle syntax or storage errors.
    pub fn insert_turtle(&self, ttl: &str) -> Result<usize> {
        let before = self
            .store
            .len()
            .map_err(|e| AppError::fm_graph(2, format!("store length unavailable: {e}")))?;
        self.store
            .load_from_slice(RdfFormat::Turtle, ttl)
            .map_err(|e| AppError::fm_graph(2, format!("turtle load failed: {e}")))?;
        let after = self
            .store
            .len()
            .map_err(|e| AppError::fm_graph(2, format!("store length unavailable: {e}")))?;
        Ok(after.saturating_sub(before))
    }

    /// Execute a SPARQL query against the graph.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-003]` on parse or evaluation errors.
    pub fn query(&self, sparql: &str) -> Result<QueryResults<'static>> {
        SparqlEvaluator::new()
            .parse_query(sparql)
            .map_err(|e| AppError::fm_graph(3, format!("SPARQL parse failed: {e}")))?
            .on_store(&self.store)
            .execute()
            .map_err(|e| AppError::fm_graph(3, format!("SPARQL evaluation failed: {e}")))
    }

    /// Return every quad currently in the store.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-004]` if iteration over the store fails.
    pub fn all_quads(&self) -> Result<Vec<Quad>> {
        self.store
            .iter()
            .collect::<std::result::Result<Vec<Quad>, _>>()
            .map_err(|e| AppError::fm_graph(4, format!("store iteration failed: {e}")))
    }

    /// BLAKE3 hash of the canonicalized graph state.
    ///
    /// The hash is computed over the sorted canonical N-Quads strings joined
    /// by `\n`. Isomorphic graphs (including blank-node relabelings) produce
    /// the same hash; insertion order never matters.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-004]`/`[FM-GRAPH-005]` on iteration or
    /// canonicalization failures.
    pub fn state_hash(&self) -> Result<[u8; 32]> {
        let quads = self.all_quads()?;
        let lines = canonical_nquad_lines(&quads)?;
        Ok(*blake3::hash(lines.join("\n").as_bytes()).as_bytes())
    }
}

// ---------------------------------------------------------------------------
// GraphEngine — the engine-neutral pipeline seam
// ---------------------------------------------------------------------------

/// One row of a SELECT result: bound variable name (bare, no `?`) → plain
/// value string (literals as their lexical form, IRIs as the bare IRI,
/// blank nodes in N-Triples form). Owned strings only — no engine model
/// types cross this seam (praxis-graphlaw is on oxrdf 0.3.x, this crate's
/// oxigraph is 0.5.9; the two must never mix).
pub type EngineRow = std::collections::BTreeMap<String, String>;

/// One triple of a CONSTRUCT/DESCRIBE result, pre-rendered both ways the
/// pipeline consumes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineTriple {
    /// Subject in N-Triples form (`<iri>` / `_:b`).
    pub subject: String,
    /// Predicate in N-Triples form.
    pub predicate: String,
    /// Object as a plain value string (literal lexical form / bare IRI).
    pub object_value: String,
    /// The whole triple in N-Triples form without the terminating ` .`,
    /// suitable for re-insertion via [`GraphEngine::insert_turtle`].
    pub ntriples: String,
}

/// Engine-neutral SPARQL results: what the template layer and the sync
/// pipeline consume instead of any engine's own result types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineQueryResults {
    /// ASK result.
    Boolean(bool),
    /// SELECT rows.
    Solutions(Vec<EngineRow>),
    /// CONSTRUCT/DESCRIBE triples.
    Graph(Vec<EngineTriple>),
}

/// Outcome of a SHACL validation run at the law gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaclOutcome {
    /// `sh:conforms` — true iff no validation result was produced.
    pub conforms: bool,
    /// One human-readable line per violation, each naming its focus node.
    pub violations: Vec<String>,
}

/// Outcome of a materialization pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterializeOutcome {
    /// Derived triples (N-Triples lines without the terminating ` .`)
    /// that were new relative to the pre-materialization state, sorted.
    pub derived: Vec<String>,
    /// Number of rules loaded into the reasoner for this pass.
    pub rules_loaded: usize,
}

/// The pipeline's graph surface: RDF facts + SPARQL plus the law-state
/// operations (N3/Datalog materialization, SHACL/ShEx gates, denials).
///
/// `state_hash` has a provided default so both engines hash identically:
/// BLAKE3 over the canonically sorted quads returned by `canonical_quads`,
/// joined by `\n` — computed ggen-side, never delegated to an engine.
pub trait GraphEngine: Send + Sync {
    /// Parse and insert Turtle content, returning the number of quads added.
    ///
    /// # Errors
    /// Fails closed on syntax or storage errors.
    fn insert_turtle(&self, ttl: &str) -> Result<usize>;

    /// Execute a SPARQL query, returning engine-neutral results.
    ///
    /// # Errors
    /// Fails closed on parse or evaluation errors.
    fn query(&self, sparql: &str) -> Result<EngineQueryResults>;

    /// Sorted canonical N-Quads lines of the current state (blank nodes
    /// relabelled `c14n{i}`; see module docs).
    ///
    /// # Errors
    /// Fails closed on iteration or canonicalization errors.
    fn canonical_quads(&self) -> Result<Vec<String>>;

    /// BLAKE3 hash of the canonicalized graph state (determinism invariant:
    /// identical for isomorphic graphs regardless of engine).
    ///
    /// # Errors
    /// Propagates [`GraphEngine::canonical_quads`] failures.
    fn state_hash(&self) -> Result<[u8; 32]> {
        let lines = self.canonical_quads()?;
        Ok(*blake3::hash(lines.join("\n").as_bytes()).as_bytes())
    }

    /// Load N3/Datalog rules (N3 rule syntax), returning how many rules
    /// the document contributed.
    ///
    /// # Errors
    /// Typed refusal when the engine has no rule support, or on parse errors.
    fn load_rules(&self, rules: &str) -> Result<usize>;

    /// Forward-chain all loaded rules to fixpoint, folding derived facts
    /// back into the queryable state.
    ///
    /// # Errors
    /// Typed refusal when the engine has no rule support.
    fn materialize(&self) -> Result<MaterializeOutcome>;

    /// Validate the current facts against a Turtle SHACL shapes graph.
    ///
    /// # Errors
    /// Typed refusal when the engine has no SHACL support, or on shape
    /// parse errors.
    fn validate_shacl(&self, shapes_turtle: &str) -> Result<ShaclOutcome>;

    /// Validate `(focus node, shape)` pairs against a ShExC schema.
    ///
    /// # Errors
    /// Typed refusal when the engine has no ShEx support, or on schema
    /// parse errors.
    fn validate_shex(
        &self,
        schema_shexc: &str,
        shape_map: &[(String, String)],
    ) -> Result<ShaclOutcome>;

    /// Evaluate every loaded denial rule (`{ body } => false.`) against the
    /// current (post-materialization) facts; one line per violated denial.
    ///
    /// # Errors
    /// Typed refusal when the engine has no rule support.
    fn check_denials(&self) -> Result<Vec<String>>;
}

/// Plain value string for an oxigraph term: literals as their lexical form,
/// IRIs as the bare IRI, everything else in N-Triples form. (Moved here from
/// `template.rs` so both the engine impls and the template layer share one
/// rendering.)
pub(crate) fn term_value(term: &Term) -> String {
    match term {
        Term::Literal(lit) => lit.value().to_string(),
        Term::NamedNode(n) => n.as_str().to_string(),
        other => other.to_string(),
    }
}

impl GraphEngine for DeterministicGraph {
    fn insert_turtle(&self, ttl: &str) -> Result<usize> {
        DeterministicGraph::insert_turtle(self, ttl)
    }

    fn query(&self, sparql: &str) -> Result<EngineQueryResults> {
        match DeterministicGraph::query(self, sparql)? {
            QueryResults::Boolean(b) => Ok(EngineQueryResults::Boolean(b)),
            QueryResults::Solutions(solutions) => {
                let mut rows = Vec::new();
                for solution in solutions {
                    let solution = solution.map_err(|e| {
                        AppError::fm_graph(3, format!("SELECT solution iteration failed: {e}"))
                    })?;
                    let mut row = EngineRow::new();
                    for (var, term) in solution.iter() {
                        row.insert(var.as_str().to_string(), term_value(term));
                    }
                    rows.push(row);
                }
                Ok(EngineQueryResults::Solutions(rows))
            }
            QueryResults::Graph(triples) => {
                let mut out = Vec::new();
                for triple in triples {
                    let triple = triple.map_err(|e| {
                        AppError::fm_graph(3, format!("CONSTRUCT triple iteration failed: {e}"))
                    })?;
                    out.push(EngineTriple {
                        subject: triple.subject.to_string(),
                        predicate: triple.predicate.to_string(),
                        object_value: term_value(&triple.object),
                        ntriples: triple.to_string(),
                    });
                }
                Ok(EngineQueryResults::Graph(out))
            }
        }
    }

    fn canonical_quads(&self) -> Result<Vec<String>> {
        canonical_nquad_lines(&self.all_quads()?)
    }

    fn load_rules(&self, _rules: &str) -> Result<usize> {
        Err(AppError::fm_law(
            1,
            "the oxigraph engine has no N3/Datalog rule support. \
             Remediation: run with the default GraphLaw engine.",
        ))
    }

    fn materialize(&self) -> Result<MaterializeOutcome> {
        Err(AppError::fm_law(
            1,
            "the oxigraph engine cannot materialize rules. \
             Remediation: run with the default GraphLaw engine.",
        ))
    }

    fn validate_shacl(&self, _shapes_turtle: &str) -> Result<ShaclOutcome> {
        Err(AppError::fm_law(
            2,
            "the oxigraph engine has no SHACL support. \
             Remediation: run with the default GraphLaw engine.",
        ))
    }

    fn validate_shex(
        &self,
        _schema_shexc: &str,
        _shape_map: &[(String, String)],
    ) -> Result<ShaclOutcome> {
        Err(AppError::fm_law(
            3,
            "the oxigraph engine has no ShEx support. \
             Remediation: run with the default GraphLaw engine.",
        ))
    }

    fn check_denials(&self) -> Result<Vec<String>> {
        Err(AppError::fm_law(
            1,
            "the oxigraph engine has no denial-rule support. \
             Remediation: run with the default GraphLaw engine.",
        ))
    }
}

// ---------------------------------------------------------------------------
// GraphLawStore — praxis-graphlaw as the live law-state engine
// ---------------------------------------------------------------------------

/// The default [`GraphEngine`]: praxis-graphlaw ("GraphLaw", the roxi fork)
/// as the law-state engine — N3/Datalog rule materialization, SHACL/ShEx
/// gates, denial checks — layered over a [`DeterministicGraph`] mirror that
/// answers SPARQL 1.1 and provides the canonical BLAKE3 state hash.
///
/// Division of labor (and why): praxis-graphlaw is on oxrdf 0.3.x while
/// this crate's oxigraph is 0.5.9, so model types never cross the seam —
/// facts flow between the two sides only as N-Triples strings. The mirror
/// is the *queryable* state; the GraphLaw reasoner is the *law* state.
/// Every derived fact enters the mirror exclusively through
/// [`GraphEngine::materialize`], i.e. through the GraphLaw reasoner — a
/// `when:` ASK that only passes after materialization is proof the
/// reasoner is in the loop (see `tests/graphlaw_e2e.rs`).
pub struct GraphLawStore {
    mirror: DeterministicGraph,
    law: std::sync::Mutex<LawState>,
}

/// Interior law-side state. `TripleStore` is rebuilt from the mirror's
/// canonical N-Triples at each `materialize()` so rules always see the
/// full fact state; it is kept afterwards for `check_denials`/SHACL/ShEx.
#[derive(Default)]
struct LawState {
    /// Raw N3 rule documents, in load order.
    rules_src: Vec<String>,
    /// Reasoner state from the last `materialize()` (None before the first).
    store: Option<praxis_graphlaw::TripleStore>,
    /// Number of rules loaded into the reasoner.
    rules_loaded: usize,
}

impl GraphLawStore {
    /// Create an empty GraphLaw engine.
    ///
    /// # Errors
    /// Propagates mirror-store initialization failures (`[FM-GRAPH-001]`).
    pub fn new() -> Result<Self> {
        Ok(Self {
            mirror: DeterministicGraph::new()?,
            law: std::sync::Mutex::new(LawState::default()),
        })
    }

    /// Lock the law state, converting a poisoned lock into a typed refusal
    /// (never a panic).
    fn law_state(&self) -> Result<std::sync::MutexGuard<'_, LawState>> {
        self.law.lock().map_err(|_| {
            AppError::fm_law(
                4,
                "law-state lock poisoned by a prior panic; refusing to continue",
            )
        })
    }

    /// The current fact state as one N-Triples document (canonical order).
    fn mirror_ntriples(&self) -> Result<String> {
        let lines = canonical_nquad_lines(&self.mirror.all_quads()?)?;
        Ok(lines.iter().map(|l| format!("{l} .\n")).collect())
    }

    /// Build a fresh GraphLaw `TripleStore` over the mirror's facts and the
    /// loaded rule documents.
    fn build_law_store(
        &self,
        rules_src: &[String],
    ) -> Result<(praxis_graphlaw::TripleStore, usize)> {
        use praxis_graphlaw::parser::Syntax;
        let nt = self.mirror_ntriples()?;
        let mut ts = praxis_graphlaw::TripleStore::new();
        ts.load_triples(&nt, Syntax::NTriples).map_err(|e| {
            AppError::fm_law(5, format!("GraphLaw fact load (N-Triples) refused: {e}"))
        })?;
        for src in rules_src {
            ts.load_rules(src)
                .map_err(|e| AppError::fm_law(6, format!("GraphLaw rule load refused: {e}")))?;
        }
        let rules_loaded = ts.rules.len();
        Ok((ts, rules_loaded))
    }
}

impl GraphEngine for GraphLawStore {
    fn insert_turtle(&self, ttl: &str) -> Result<usize> {
        self.mirror.insert_turtle(ttl)
    }

    fn query(&self, sparql: &str) -> Result<EngineQueryResults> {
        GraphEngine::query(&self.mirror, sparql)
    }

    fn canonical_quads(&self) -> Result<Vec<String>> {
        GraphEngine::canonical_quads(&self.mirror)
    }

    fn load_rules(&self, rules: &str) -> Result<usize> {
        // Parse eagerly so a bad rule document is refused at load time,
        // not at the later materialize call.
        let mut probe = praxis_graphlaw::TripleStore::new();
        probe
            .load_rules(rules)
            .map_err(|e| AppError::fm_law(6, format!("GraphLaw rule load refused: {e}")))?;
        let n = probe.rules.len();
        self.law_state()?.rules_src.push(rules.to_string());
        Ok(n)
    }

    fn materialize(&self) -> Result<MaterializeOutcome> {
        let mut state = self.law_state()?;
        let (mut ts, rules_loaded) = self.build_law_store(&state.rules_src)?;
        let derived = ts.materialize();
        if let Some(refused_verdict) = ts.verdicts.iter().find(|v| v.effect == praxis_graphlaw::hooks::EffectKind::Refuse && v.verdict == praxis_graphlaw::hooks::HookVerdict::Fired) {
            let reason = refused_verdict.diagnostics.as_ref()
                .and_then(|d| d.details.first())
                .map(|det| det.message.clone())
                .unwrap_or_else(|| "refused by hook".to_string());
            return Err(AppError::fm_law(9, format!("Reasoner materialize failed: {}", reason)));
        }
        let derived_doc = praxis_graphlaw::TripleStore::decode_triples(&derived);

        // Fold derived facts back into the mirror through the one canonical
        // door (Turtle/N-Triples parse) — an underived/duplicate line is
        // idempotent; an unparseable derived term is a typed refusal, never
        // silently dropped.
        let before: std::collections::BTreeSet<String> =
            self.canonical_quads()?.into_iter().collect();
        if !derived_doc.is_empty() {
            self.mirror.insert_turtle(&derived_doc).map_err(|e| {
                AppError::fm_law(
                    7,
                    format!(
                        "derived facts from the GraphLaw reasoner are not valid \
                         N-Triples/Turtle: {e}. Derived document:\n{derived_doc}"
                    ),
                )
            })?;
        }
        let after = self.canonical_quads()?;
        let derived_new: Vec<String> = after.into_iter().filter(|l| !before.contains(l)).collect();

        state.store = Some(ts);
        state.rules_loaded = rules_loaded;
        Ok(MaterializeOutcome {
            derived: derived_new,
            rules_loaded,
        })
    }

    fn validate_shacl(&self, shapes_turtle: &str) -> Result<ShaclOutcome> {
        // Validate against the full current fact state (post-materialization
        // if materialize ran), by building a law store over the mirror.
        let state = self.law_state()?;
        let (ts, _) = self.build_law_store(&state.rules_src)?;
        let report = ts
            .validate_shacl(shapes_turtle)
            .map_err(|e| AppError::fm_law(8, format!("SHACL shapes graph refused: {e}")))?;
        let violations = report
            .results
            .iter()
            .map(|r| {
                let msg = r.message.as_deref().unwrap_or("constraint violated");
                format!(
                    "focus node {}: {msg} (source shape {})",
                    r.focus_node, r.source_shape
                )
            })
            .collect();
        Ok(ShaclOutcome {
            conforms: report.conforms,
            violations,
        })
    }

    fn validate_shex(
        &self,
        schema_shexc: &str,
        shape_map: &[(String, String)],
    ) -> Result<ShaclOutcome> {
        let state = self.law_state()?;
        let (ts, _) = self.build_law_store(&state.rules_src)?;
        let report = ts
            .validate_shex_c(schema_shexc, shape_map)
            .map_err(|e| AppError::fm_law(9, format!("ShExC schema refused: {e}")))?;
        let violations = report
            .failures
            .iter()
            .map(|f| {
                format!(
                    "focus node {}: does not conform to shape {}: {}",
                    f.node, f.shape, f.reason
                )
            })
            .collect::<Vec<_>>();
        Ok(ShaclOutcome {
            conforms: report.conforms,
            violations,
        })
    }

    fn check_denials(&self) -> Result<Vec<String>> {
        let state = self.law_state()?;
        match &state.store {
            Some(ts) => Ok(ts.check_denials()),
            None => {
                // Denials are rules; without a materialize pass there is no
                // reasoner state. Build one so `law validate` can be called
                // without an explicit prior derive.
                let (mut ts, _) = self.build_law_store(&state.rules_src)?;
                ts.materialize();
                Ok(ts.check_denials())
            }
        }
    }
}

/// A set difference between two graph states, expressed as canonical
/// N-Quads strings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Delta {
    /// Canonical N-Quads present in the target but not the baseline (sorted).
    pub additions: Vec<String>,
    /// Canonical N-Quads present in the baseline but not the target (sorted).
    pub deletions: Vec<String>,
}

impl Delta {
    /// Compute the delta that transforms `baseline` into `target`.
    ///
    /// # Errors
    /// Propagates canonicalization/iteration failures from either graph.
    pub fn compute(baseline: &DeterministicGraph, target: &DeterministicGraph) -> Result<Self> {
        let base: BTreeSet<String> = canonical_nquad_lines(&baseline.all_quads()?)?
            .into_iter()
            .collect();
        let tgt: BTreeSet<String> = canonical_nquad_lines(&target.all_quads()?)?
            .into_iter()
            .collect();
        Ok(Self {
            additions: tgt.difference(&base).cloned().collect(),
            deletions: base.difference(&tgt).cloned().collect(),
        })
    }

    /// Apply this delta to `graph`: remove the deletions, insert the additions.
    ///
    /// Deletions are matched by canonical N-Quads string against the graph's
    /// current canonicalization; additions are parsed as N-Quads and inserted.
    ///
    /// # Errors
    /// Returns `[FM-GRAPH-006]` if a deletion does not match any quad in the
    /// graph (fail closed), or on parse/storage errors while applying.
    pub fn apply(&self, graph: &DeterministicGraph) -> Result<()> {
        if !self.deletions.is_empty() {
            let quads = graph.all_quads()?;
            let pairs = canonical_pairs(&quads)?;
            let by_canonical: HashMap<&str, &Quad> =
                pairs.iter().map(|(s, q)| (s.as_str(), q)).collect();
            for del in &self.deletions {
                let quad = by_canonical.get(del.as_str()).ok_or_else(|| {
                    AppError::fm_graph(
                        6,
                        format!("deletion not present in graph (refusing partial apply): {del}"),
                    )
                })?;
                graph.store.remove(quad.as_ref()).map_err(|e| {
                    AppError::fm_graph(6, format!("failed to remove quad `{del}`: {e}"))
                })?;
            }
        }
        if !self.additions.is_empty() {
            // `Quad`'s Display form omits the terminating ` .` required by
            // the N-Quads grammar; add it per line before parsing.
            let doc: String = self.additions.iter().map(|a| format!("{a} .\n")).collect();
            graph
                .store
                .load_from_slice(RdfFormat::NQuads, doc.as_str())
                .map_err(|e| {
                    AppError::fm_graph(6, format!("failed to insert delta additions: {e}"))
                })?;
        }
        Ok(())
    }

    /// The inverse delta: additions and deletions swapped, so that applying
    /// `self` then `self.inverse()` is a net no-op.
    #[must_use]
    pub fn inverse(&self) -> Self {
        Self {
            additions: self.deletions.clone(),
            deletions: self.additions.clone(),
        }
    }

    /// Compose two deltas: the result of applying `self` first, then `other`.
    ///
    /// Cancellation semantics: an addition in `self` removed again by a
    /// deletion in `other` vanishes from the composite (and vice versa).
    /// Output vectors are sorted and deduplicated (deterministic).
    #[must_use]
    pub fn compose(&self, other: &Delta) -> Self {
        let a1: BTreeSet<&String> = self.additions.iter().collect();
        let d1: BTreeSet<&String> = self.deletions.iter().collect();
        let a2: BTreeSet<&String> = other.additions.iter().collect();
        let d2: BTreeSet<&String> = other.deletions.iter().collect();

        // Net additions: self's additions that survive other's deletions,
        // plus other's additions that are not merely undoing self's deletions.
        let additions: BTreeSet<String> = a1
            .iter()
            .filter(|s| !d2.contains(**s))
            .chain(a2.iter().filter(|s| !d1.contains(**s)))
            .map(|s| (**s).clone())
            .collect();
        // Net deletions, symmetrically.
        let deletions: BTreeSet<String> = d1
            .iter()
            .filter(|s| !a2.contains(**s))
            .chain(d2.iter().filter(|s| !a1.contains(**s)))
            .map(|s| (**s).clone())
            .collect();

        Self {
            additions: additions.into_iter().collect(),
            deletions: deletions.into_iter().collect(),
        }
    }

    /// True when the delta contains no additions and no deletions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.additions.is_empty() && self.deletions.is_empty()
    }

    /// BLAKE3 hash of the delta: sorted additions prefixed `+`, then sorted
    /// deletions prefixed `-`.
    #[must_use]
    pub fn hash(&self) -> [u8; 32] {
        let mut additions = self.additions.clone();
        additions.sort();
        let mut deletions = self.deletions.clone();
        deletions.sort();
        let mut hasher = blake3::Hasher::new();
        for a in &additions {
            hasher.update(b"+");
            hasher.update(a.as_bytes());
        }
        for d in &deletions {
            hasher.update(b"-");
            hasher.update(d.as_bytes());
        }
        *hasher.finalize().as_bytes()
    }
}

/// Sorted canonical N-Quads lines for a quad slice.
fn canonical_nquad_lines(quads: &[Quad]) -> Result<Vec<String>> {
    Ok(canonical_pairs(quads)?
        .into_iter()
        .map(|(s, _)| s)
        .collect())
}

/// Canonicalize quads, returning `(canonical N-Quads string, original quad)`
/// pairs sorted by the canonical string.
///
/// Blank nodes are relabelled `c14n{i}` after bounded color refinement so
/// the canonical strings are stable across blank-node renamings.
fn canonical_pairs(quads: &[Quad]) -> Result<Vec<(String, Quad)>> {
    let blank_nodes = collect_blank_nodes(quads);

    let relabel: HashMap<BlankNode, BlankNode> = if blank_nodes.is_empty() {
        HashMap::new()
    } else {
        canonical_blank_node_map(quads, &blank_nodes)?
    };

    let mut pairs: Vec<(String, Quad)> = quads
        .iter()
        .map(|q| (relabel_quad(q, &relabel).to_string(), q.clone()))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(pairs)
}

/// Collect every blank node appearing in subject, object, or graph position.
fn collect_blank_nodes(quads: &[Quad]) -> HashSet<BlankNode> {
    let mut blanks = HashSet::new();
    for q in quads {
        if let NamedOrBlankNode::BlankNode(b) = &q.subject {
            blanks.insert(b.clone());
        }
        if let Term::BlankNode(b) = &q.object {
            blanks.insert(b.clone());
        }
        if let GraphName::BlankNode(b) = &q.graph_name {
            blanks.insert(b.clone());
        }
    }
    blanks
}

/// Bounded color refinement: compute stable BLAKE3 neighborhood signatures
/// for every blank node, then map each to a `c14n{i}` label in signature
/// order (ties broken by original label for determinism within this graph).
fn canonical_blank_node_map(
    quads: &[Quad],
    blank_nodes: &HashSet<BlankNode>,
) -> Result<HashMap<BlankNode, BlankNode>> {
    let mut labels: HashMap<BlankNode, String> = blank_nodes
        .iter()
        .map(|b| (b.clone(), "bnode".to_string()))
        .collect();

    for _ in 0..REFINEMENT_ITERATIONS {
        let mut next: HashMap<BlankNode, String> = HashMap::new();
        for bnode in blank_nodes {
            let mut neighborhood: Vec<String> = quads
                .iter()
                .filter(|q| quad_touches(q, bnode))
                .map(|q| neighborhood_line(q, bnode, &labels))
                .collect();
            neighborhood.sort();
            let signature = blake3::hash(neighborhood.join("\n").as_bytes())
                .to_hex()
                .to_string();
            next.insert(bnode.clone(), signature);
        }
        labels = next;
    }

    let mut ordered: Vec<&BlankNode> = blank_nodes.iter().collect();
    ordered.sort_by(|a, b| {
        let la = labels.get(*a).map_or("", String::as_str);
        let lb = labels.get(*b).map_or("", String::as_str);
        la.cmp(lb).then_with(|| a.to_string().cmp(&b.to_string()))
    });

    let mut map = HashMap::new();
    for (idx, bnode) in ordered.into_iter().enumerate() {
        let canonical = BlankNode::new(format!("c14n{idx}")).map_err(|e| {
            AppError::fm_graph(5, format!("canonical blank node label rejected: {e}"))
        })?;
        map.insert(bnode.clone(), canonical);
    }
    Ok(map)
}

/// Does `quad` mention `bnode` in any position?
fn quad_touches(quad: &Quad, bnode: &BlankNode) -> bool {
    matches!(&quad.subject, NamedOrBlankNode::BlankNode(b) if b == bnode)
        || matches!(&quad.object, Term::BlankNode(b) if b == bnode)
        || matches!(&quad.graph_name, GraphName::BlankNode(b) if b == bnode)
}

/// Render one neighborhood line for the signature of `bnode`, substituting
/// `_:self` for the node itself and current labels for other blank nodes.
fn neighborhood_line(
    quad: &Quad,
    bnode: &BlankNode,
    labels: &HashMap<BlankNode, String>,
) -> String {
    let blank_repr = |b: &BlankNode| -> String {
        if b == bnode {
            "_:self".to_string()
        } else {
            format!("_:{}", labels.get(b).map_or("", String::as_str))
        }
    };
    let s = match &quad.subject {
        NamedOrBlankNode::BlankNode(b) => blank_repr(b),
        NamedOrBlankNode::NamedNode(n) => n.to_string(),
    };
    let o = match &quad.object {
        Term::BlankNode(b) => blank_repr(b),
        other => other.to_string(),
    };
    let g = match &quad.graph_name {
        GraphName::DefaultGraph => String::new(),
        GraphName::NamedNode(n) => n.to_string(),
        GraphName::BlankNode(b) => blank_repr(b),
    };
    format!("{s} {} {o} {g}", quad.predicate)
}

/// Rewrite a quad's blank nodes through the canonical relabeling map.
/// Nodes absent from the map (i.e. non-blank positions) pass through.
fn relabel_quad(quad: &Quad, map: &HashMap<BlankNode, BlankNode>) -> Quad {
    let subject = match &quad.subject {
        NamedOrBlankNode::BlankNode(b) => {
            NamedOrBlankNode::BlankNode(map.get(b).cloned().unwrap_or_else(|| b.clone()))
        }
        other => other.clone(),
    };
    let object = match &quad.object {
        Term::BlankNode(b) => Term::BlankNode(map.get(b).cloned().unwrap_or_else(|| b.clone())),
        other => other.clone(),
    };
    let graph_name = match &quad.graph_name {
        GraphName::BlankNode(b) => {
            GraphName::BlankNode(map.get(b).cloned().unwrap_or_else(|| b.clone()))
        }
        other => other.clone(),
    };
    Quad::new(subject, quad.predicate.clone(), object, graph_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delta(adds: &[&str], dels: &[&str]) -> Delta {
        Delta {
            additions: adds.iter().map(ToString::to_string).collect(),
            deletions: dels.iter().map(ToString::to_string).collect(),
        }
    }

    #[test]
    fn compose_with_inverse_is_empty_and_hashes_as_empty() {
        let d = delta(
            &["<http://e/a> <http://e/p> \"1\""],
            &["<http://e/b> <http://e/p> \"2\""],
        );
        let net = d.compose(&d.inverse());
        assert!(net.is_empty(), "delta ∘ delta⁻¹ must cancel: {net:?}");
        assert_eq!(
            net.hash(),
            Delta::default().hash(),
            "empty-composite hash must equal empty delta hash"
        );
    }

    #[test]
    fn inverse_is_an_involution() {
        let d = delta(&["a", "b"], &["c"]);
        assert_eq!(d.inverse().inverse(), d);
    }

    #[test]
    fn compose_cancels_crosswise_and_keeps_survivors_sorted() {
        // self adds x, deletes y; other deletes x (cancel), adds y (cancel),
        // and adds z (survives).
        let d1 = delta(&["x"], &["y"]);
        let d2 = delta(&["z", "y"], &["x"]);
        let net = d1.compose(&d2);
        assert_eq!(net.additions, vec!["z".to_string()]);
        assert!(net.deletions.is_empty());
    }

    #[test]
    fn empty_delta_is_empty() {
        assert!(Delta::default().is_empty());
        assert!(!delta(&["a"], &[]).is_empty());
    }

    // -- GraphLawStore ------------------------------------------------------

    const DOG_TTL: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:rex a ex:Dog .
    "#;

    const DOG_RULE: &str = "@prefix ex: <http://example.org/>. {?s a ex:Dog} => {?s a ex:Animal}.";

    #[test]
    fn graphlaw_materialize_derives_facts_visible_to_sparql() {
        let store = GraphLawStore::new().expect("store");
        GraphEngine::insert_turtle(&store, DOG_TTL).expect("ttl");
        // Before materialization the ASK is false.
        let ask = "ASK { <http://example.org/rex> a <http://example.org/Animal> }";
        assert_eq!(
            store.query(ask).expect("ask"),
            EngineQueryResults::Boolean(false)
        );
        let rules = store.load_rules(DOG_RULE).expect("rules");
        assert_eq!(rules, 1, "one rule in the document");
        let outcome = store.materialize().expect("materialize");
        assert_eq!(outcome.rules_loaded, 1);
        assert_eq!(
            outcome.derived.len(),
            1,
            "one derived fact: {:?}",
            outcome.derived
        );
        assert_eq!(
            store.query(ask).expect("ask"),
            EngineQueryResults::Boolean(true),
            "derived fact must be visible to SPARQL after materialization"
        );
    }

    #[test]
    fn graphlaw_state_hash_matches_oxigraph_for_same_facts() {
        // Determinism invariant: both engines hash identical fact states
        // identically (BLAKE3 over canonical quads, computed ggen-side).
        let law = GraphLawStore::new().expect("law");
        GraphEngine::insert_turtle(&law, DOG_TTL).expect("ttl");
        let oxi = DeterministicGraph::new().expect("oxi");
        oxi.insert_turtle(DOG_TTL).expect("ttl");
        assert_eq!(
            GraphEngine::state_hash(&law).expect("law hash"),
            GraphEngine::state_hash(&oxi).expect("oxi hash"),
        );
    }

    #[test]
    fn graphlaw_check_denials_reports_violation() {
        let store = GraphLawStore::new().expect("store");
        GraphEngine::insert_turtle(
            &store,
            "@prefix ex: <http://example.org/> . ex:x a ex:Forbidden .",
        )
        .expect("ttl");
        store
            .load_rules("@prefix ex: <http://example.org/>. {?s a ex:Forbidden} => false.")
            .expect("denial rule");
        store.materialize().expect("materialize");
        let denials = store.check_denials().expect("denials");
        assert_eq!(denials.len(), 1, "one violated denial: {denials:?}");
        assert!(denials[0].contains("DENIED"), "{denials:?}");
    }

    #[test]
    fn graphlaw_validate_shacl_flags_focus_node() {
        let store = GraphLawStore::new().expect("store");
        GraphEngine::insert_turtle(
            &store,
            "@prefix ex: <http://example.org/> . ex:rex a ex:Dog .",
        )
        .expect("ttl");
        let shapes = r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            ex:DogShape a sh:NodeShape ;
                sh:targetClass ex:Dog ;
                sh:property [ sh:path ex:name ; sh:minCount 1 ] .
        "#;
        let outcome = store.validate_shacl(shapes).expect("shacl");
        assert!(!outcome.conforms, "rex has no ex:name");
        assert!(
            outcome.violations.iter().any(|v| v.contains("rex")),
            "violation must name the focus node: {:?}",
            outcome.violations
        );
        // Conforming after the fix.
        GraphEngine::insert_turtle(
            &store,
            "@prefix ex: <http://example.org/> . ex:rex ex:name \"Rex\" .",
        )
        .expect("ttl");
        let outcome = store.validate_shacl(shapes).expect("shacl");
        assert!(outcome.conforms, "{:?}", outcome.violations);
    }

    #[test]
    fn oxigraph_engine_refuses_law_ops_with_typed_fm_law() {
        let g = DeterministicGraph::new().expect("graph");
        let err =
            GraphEngine::load_rules(&g, "{?s ?p ?o} => {?s ?p ?o}.").expect_err("must refuse");
        assert!(err.to_string().contains("FM-LAW-001"), "{err}");
        let err = GraphEngine::materialize(&g).expect_err("must refuse");
        assert!(err.to_string().contains("FM-LAW-001"), "{err}");
        let err = GraphEngine::validate_shacl(&g, "").expect_err("must refuse");
        assert!(err.to_string().contains("FM-LAW-002"), "{err}");
    }
}
