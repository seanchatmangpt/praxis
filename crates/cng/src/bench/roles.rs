//! SPARQL execution helpers (query text always from the on-disk `QuerySet`),
//! recursive-attachment derivation, observation emission (`ObsWriter`), and
//! the Datalog role layer (Phase 4): old-AI (Mycin) role inference plus
//! praxis-graphlaw Datalog materialization over the admitted roster graph.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{GraphName, LiteralRef, NamedNodeRef, Term, TermRef};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::bench::dispatch::write_atomic;
use crate::powl::CngRefusal;

use super::manufacture::{run_iri, term_value};
use super::templates::Templates;
use super::{fill_template, rwai_local, OBS_PER_PARTITION};

// Mycin/Datalog role-inference machinery (role_rules, infer_lawful_next_
// action, soc2_role_rules, infer_soc2_standing_role, RosterWorker,
// DatalogRoles, derive_roles_datalog) moved verbatim to `crate::roles`
// (unconditional module, gated by the `role-inference` feature — widened
// from bench-only so the live plan-admit path can reach the same engine;
// see that module's doc and `plan_approval.rs::derive_roster_roles`). Every
// bench call site below keeps working unchanged via this re-export.
pub(super) use crate::roles::{
    derive_roles_datalog, infer_lawful_next_action, infer_soc2_standing_role, role_rules,
    soc2_role_rules, DatalogRoles, RosterWorker,
};

/// Recursive-attachment derivation (G-D): pattern-scans the node's admitted
/// fragment graph for `ex:attachesWorkflow`, projects the pairs into
/// socket_attached observation facts (template-rendered), and reads them
/// back through the on-disk `attachments-with-parent.rq` SELECT over that
/// observation store — KEEPING the `?parentActivity` binding.
///
/// Returns `(parent activity IRI, child workflow IRI)` rows, ordered by
/// child IRI (the query's ORDER BY).
///
/// # Errors
/// Typed refusals for unreadable fragments, template/store failures, or a
/// query that does not yield solutions.
///
/// # Complexity
/// O(attachments) scan + one SELECT over O(attachments) facts.
pub(super) fn derive_attachments(
    set_dir: &Path,
    templates: &Templates,
    attach_query: &str,
) -> Result<Vec<(String, String)>, CngRefusal> {
    let path = set_dir.join("fragment-0.domain.ttl");
    let turtle = fs::read_to_string(&path)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", path.display())))?;
    let fragment_store =
        Store::new().map_err(|e| CngRefusal::IoRefused(format!("store construction: {e}")))?;
    fragment_store
        .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), turtle.as_bytes())
        .map_err(|e| CngRefusal::MalformedTtl(format!("fragment load {}: {e}", path.display())))?;

    let attaches_iri = format!("{}attachesWorkflow", super::RWAI_PREFIX);
    let attaches = NamedNodeRef::new(&attaches_iri)
        .map_err(|e| CngRefusal::MalformedTtl(format!("attachesWorkflow IRI: {e}")))?;
    let mut pairs: Vec<(String, String)> = Vec::new();
    for quad in fragment_store.quads_for_pattern(None, Some(attaches), None, None) {
        let quad =
            quad.map_err(|e| CngRefusal::MalformedTtl(format!("fragment pattern scan: {e}")))?;
        let subject = quad.subject.to_string();
        let subject = subject.trim_matches(|c| c == '<' || c == '>').to_string();
        if let Term::NamedNode(child) = quad.object {
            pairs.push((subject, child.as_str().to_string()));
        }
    }
    if pairs.is_empty() {
        return Ok(Vec::new());
    }

    // Project the pairs as socket_attached observation facts and read them
    // back through the on-disk query (preserves the parentActivity binding).
    let socket_template = templates.obs.get("socket-attached").ok_or_else(|| {
        CngRefusal::IoRefused("socket-attached observation template missing".to_string())
    })?;
    let obs_store =
        Store::new().map_err(|e| CngRefusal::IoRefused(format!("store construction: {e}")))?;
    let set_id = rwai_local(&run_iri(set_dir)).to_string();
    for (i, (parent, child)) in pairs.iter().enumerate() {
        let seq = i.to_string();
        let body = fill_template(
            socket_template,
            &[
                ("SUBJECT", format!("obs-attach-{set_id}-{i}").as_str()),
                ("SEQ", seq.as_str()),
                ("SET_ID", set_id.as_str()),
                ("WORKFLOW_ID", set_id.as_str()),
                ("PARENT_ACTIVITY", rwai_local(parent)),
                ("CHILD_WORKFLOW", rwai_local(child)),
            ],
        );
        obs_store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
            .map_err(|e| CngRefusal::MalformedTtl(format!("socket obs load: {e}")))?;
    }
    let rows = select_rows(&obs_store, attach_query)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let parent = row.get("parentActivity").cloned().ok_or_else(|| {
            CngRefusal::MalformedTtl(
                "attachments-with-parent.rq row missing ?parentActivity".to_string(),
            )
        })?;
        let child = row.get("childWorkflow").cloned().ok_or_else(|| {
            CngRefusal::MalformedTtl(
                "attachments-with-parent.rq row missing ?childWorkflow".to_string(),
            )
        })?;
        out.push((parent, child));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// SPARQL execution helpers (query text always from the on-disk QuerySet)
// ---------------------------------------------------------------------------

/// Executes a SELECT and returns rows as `variable name → plain value`
/// maps, in the engine's (query-ordered) row order.
pub(super) fn select_rows(
    store: &Store,
    query: &str,
) -> Result<Vec<BTreeMap<String, String>>, CngRefusal> {
    let prepared = SparqlEvaluator::new()
        .parse_query(query)
        .map_err(|e| CngRefusal::MalformedTtl(format!("query parse: {e}")))?;
    match prepared.on_store(store).execute() {
        Ok(QueryResults::Solutions(solutions)) => {
            let mut rows = Vec::new();
            for solution in solutions {
                let solution =
                    solution.map_err(|e| CngRefusal::MalformedTtl(format!("query eval: {e}")))?;
                let mut row = BTreeMap::new();
                for (var, term) in solution.iter() {
                    row.insert(var.as_str().to_string(), term_value(&term.clone()));
                }
                rows.push(row);
            }
            Ok(rows)
        }
        Ok(_) => Err(CngRefusal::MalformedTtl(
            "query did not yield solutions".to_string(),
        )),
        Err(e) => Err(CngRefusal::MalformedTtl(format!(
            "query execution failed: {e}"
        ))),
    }
}

/// Executes a single-`?count` metric SELECT.
pub(super) fn metric_count(store: &Store, query: &str, name: &str) -> Result<u64, CngRefusal> {
    let rows = select_rows(store, query)?;
    let row = rows
        .first()
        .ok_or_else(|| CngRefusal::MalformedTtl(format!("{name}: metric query yielded no rows")))?;
    let value = row.get("count").ok_or_else(|| {
        CngRefusal::MalformedTtl(format!("{name}: metric query row has no ?count binding"))
    })?;
    value
        .parse::<u64>()
        .map_err(|e| CngRefusal::MalformedTtl(format!("{name}: count parse: {e}")))
}

/// Executes a CONSTRUCT over `source` and inserts the produced triples
/// into `sink` (default graph). Returns the triple count produced.
pub(super) fn run_construct(
    source: &Store,
    query: &str,
    sink: &Store,
) -> Result<usize, CngRefusal> {
    let prepared = SparqlEvaluator::new()
        .parse_query(query)
        .map_err(|e| CngRefusal::MalformedTtl(format!("construct parse: {e}")))?;
    match prepared.on_store(source).execute() {
        Ok(QueryResults::Graph(triples)) => {
            let mut n = 0usize;
            for triple in triples {
                let triple =
                    triple.map_err(|e| CngRefusal::MalformedTtl(format!("construct eval: {e}")))?;
                let quad = triple.in_graph(GraphName::DefaultGraph);
                sink.insert(&quad)
                    .map_err(|e| CngRefusal::IoRefused(format!("evidence insert: {e}")))?;
                n += 1;
            }
            Ok(n)
        }
        Ok(_) => Err(CngRefusal::MalformedTtl(
            "construct query did not yield a graph".to_string(),
        )),
        Err(e) => Err(CngRefusal::MalformedTtl(format!(
            "construct execution failed: {e}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Observation emission (G-A)
// ---------------------------------------------------------------------------

/// Appends template-rendered observation facts into partitioned `.ttl`
/// artifacts (replay/audit input) and mirrors every fact into the given
/// observation store. `obs:obsSeq` is the writer's monotone logical
/// counter — deterministic, no wall clock.
pub(super) struct ObsWriter<'a> {
    templates: &'a Templates,
    store: &'a Store,
    dir: PathBuf,
    prefix: &'static str,
    seq: u64,
    buf: String,
    in_buf: usize,
    part_idx: usize,
    /// Emissions buffered before an automatic partition flush (PROJ-721):
    /// defaults to [`OBS_PER_PARTITION`]; durable serve loops set 1 (eager
    /// per-emit flush) via [`Self::with_flush_threshold`].
    flush_threshold: usize,
}

impl<'a> ObsWriter<'a> {
    pub(super) fn new(
        templates: &'a Templates,
        store: &'a Store,
        dir: &Path,
        prefix: &'static str,
    ) -> Result<Self, CngRefusal> {
        let mut attempts = 0;
        loop {
            if let Err(e) = fs::create_dir_all(dir) {
                if attempts < 10 {
                    attempts += 1;
                    std::thread::yield_now();
                    continue;
                }
                return Err(CngRefusal::IoRefused(format!(
                    "mkdir {}: {e}",
                    dir.display()
                )));
            }
            if dir.exists() {
                break;
            }
            if attempts < 10 {
                attempts += 1;
                std::thread::yield_now();
                continue;
            }
            return Err(CngRefusal::IoRefused(format!(
                "mkdir {}: returned Ok but does not exist",
                dir.display()
            )));
        }
        // Resume-safe partition numbering (swarm audit wnl2yhbgm finding #8): a fresh
        // `ObsWriter` always started `part_idx` at 0 regardless of what's already on disk, so
        // `engine_resume`'s `run_serve_loop` call -- which constructs a NEW `ObsWriter` over
        // the SAME `bundle.ticks_dir()` a prior `engine_serve` pass already wrote into --
        // would silently overwrite that prior pass's own `<prefix>-part-00000.ttl` (and
        // onward) with the resumed session's own first flush, destroying already-durable
        // observation partitions in exactly the crash-resume scenario `engine_resume` exists
        // for. Scan for the highest already-written `<prefix>-part-<N>.ttl` and continue one
        // past it; never overwrite.
        let next_part_idx = fs::read_dir(dir)
            .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", dir.display())))?
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name();
                let name = name.to_str()?.to_string();
                name.strip_prefix(prefix)?
                    .strip_prefix("-part-")?
                    .strip_suffix(".ttl")?
                    .parse::<usize>()
                    .ok()
            })
            .max()
            .map_or(0, |max_idx| max_idx + 1);

        Ok(ObsWriter {
            templates,
            store,
            dir: dir.to_path_buf(),
            prefix,
            seq: 0,
            buf: String::new(),
            in_buf: 0,
            part_idx: next_part_idx,
            flush_threshold: OBS_PER_PARTITION,
        })
    }

    /// Overrides the automatic flush threshold (PROJ-721 eager-flush
    /// option; a threshold of 1 flushes every emission durably). O(1).
    pub(super) fn with_flush_threshold(mut self, flush_threshold: usize) -> Self {
        self.flush_threshold = flush_threshold.max(1);
        self
    }

    /// Emits one observation of `kind` with the extra placeholder pairs;
    /// SUBJECT and SEQ are supplied by the writer's monotone counter.
    pub(super) fn emit(
        &mut self,
        kind: &'static str,
        extra: &[(&str, &str)],
    ) -> Result<(), CngRefusal> {
        let template = self.templates.obs.get(kind).ok_or_else(|| {
            CngRefusal::IoRefused(format!(
                "observation template {kind} missing from loaded set"
            ))
        })?;
        let seq = self.seq;
        self.seq += 1;
        let subject = format!("obs-{}-{seq}", self.prefix);
        let seq_text = seq.to_string();
        let mut pairs: Vec<(&str, &str)> =
            vec![("SUBJECT", subject.as_str()), ("SEQ", seq_text.as_str())];
        pairs.extend_from_slice(extra);
        let body = fill_template(template, &pairs);
        self.store
            .load_from_slice(RdfParser::from_format(RdfFormat::Turtle), body.as_bytes())
            .map_err(|e| CngRefusal::MalformedTtl(format!("observation load ({kind}): {e}")))?;
        self.buf.push_str(&body);
        self.buf.push('\n');
        self.in_buf += 1;
        if self.in_buf >= self.flush_threshold {
            self.flush()?;
        }
        Ok(())
    }

    pub(super) fn flush(&mut self) -> Result<(), CngRefusal> {
        if self.in_buf == 0 {
            return Ok(());
        }
        let path = self
            .dir
            .join(format!("{}-part-{:05}.ttl", self.prefix, self.part_idx));
        // write_atomic (tmp + fs::rename), not plain fs::write: a concurrent reader scanning
        // ticks_dir() (e.g. engine_collect_remote's evidence-materialization pass) must never
        // observe a torn/partially-written partition file (swarm audit wnl2yhbgm finding #8).
        write_atomic(&path, &self.buf)?;
        self.part_idx += 1;
        self.buf.clear();
        self.in_buf = 0;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Datalog role layer (Phase 4) — RosterWorker/DatalogRoles/
// derive_roles_datalog now live in `crate::roles` (re-exported above);
// `roster_workers` below stays here since it reads bench's own
// ObsWriter-emitted observation-store shape specifically.
// ---------------------------------------------------------------------------

/// Reads every roster_admitted observation back out of the observation
/// store via pattern scans (worker id, declared role, department), sorted
/// by worker id.
pub(super) fn roster_workers(obs_store: &Store) -> Result<Vec<RosterWorker>, CngRefusal> {
    const OBS_PREFIX: &str = "https://ggen.io/ontology/bench-obs#";
    let kind_iri = format!("{OBS_PREFIX}obsKind");
    let worker_iri = format!("{OBS_PREFIX}obsWorkerId");
    let role_iri = format!("{OBS_PREFIX}obsRole");
    let dept_iri = format!("{OBS_PREFIX}obsDepartment");
    fn pred(iri: &str) -> Result<NamedNodeRef<'_>, CngRefusal> {
        NamedNodeRef::new(iri).map_err(|e| CngRefusal::MalformedTtl(format!("{iri}: {e}")))
    }
    let kind_pred = pred(&kind_iri)?;
    let kind_lit = LiteralRef::new_simple_literal("roster_admitted");
    let worker_pred = pred(&worker_iri)?;
    let role_pred = pred(&role_iri)?;
    let dept_pred = pred(&dept_iri)?;

    let mut workers = Vec::new();
    for quad in
        obs_store.quads_for_pattern(None, Some(kind_pred), Some(TermRef::from(kind_lit)), None)
    {
        let quad = quad.map_err(|e| CngRefusal::MalformedTtl(format!("roster scan: {e}")))?;
        let subject = quad.subject;
        let read = |pred: NamedNodeRef<'_>, what: &str| -> Result<String, CngRefusal> {
            obs_store
                .quads_for_pattern(Some(subject.as_ref()), Some(pred), None, None)
                .next()
                .transpose()
                .map_err(|e| CngRefusal::MalformedTtl(format!("roster scan ({what}): {e}")))?
                .map(|q| term_value(&q.object))
                .ok_or_else(|| {
                    CngRefusal::MalformedTtl(format!(
                        "roster_admitted observation {subject} has no {what}"
                    ))
                })
        };
        workers.push(RosterWorker {
            worker_id: read(worker_pred, "obsWorkerId")?,
            role: read(role_pred, "obsRole")?,
            department: read(dept_pred, "obsDepartment")?,
        });
    }
    workers.sort_by(|a, b| a.worker_id.cmp(&b.worker_id));
    Ok(workers)
}

/// Recursively collects `.ttl` file paths under `dir`, appending to `out`.
///
/// # Complexity
/// O(entries under dir).
pub(super) fn collect_ttl_paths_recursive(
    dir: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), CngRefusal> {
    let entries = fs::read_dir(dir)
        .map_err(|e| CngRefusal::IoRefused(format!("read {}: {e}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| CngRefusal::IoRefused(format!("read dir entry: {e}")))?;
        let path = entry.path();
        if path.is_dir() {
            collect_ttl_paths_recursive(&path, out)?;
        } else if path.extension().and_then(|x| x.to_str()) == Some("ttl") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "roles_test.rs"]
mod roles_test;
