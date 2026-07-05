//! The five-stage sync pipeline: Resolve → Enrich → Extract → Render → Write.
//!
//! [`sync`] loads `ggen.toml` and the ontology (Resolve), runs every
//! template `construct:` query once and inserts the produced triples
//! (Enrich — **single pass**: closure is not iterated to a fixed point yet;
//! constructs that depend on other constructs' output require a second
//! `sync` run), evaluates `when:` ASK guards and named `sparql:` SELECTs
//! (Extract), renders each template body via Tera (Render), and applies the
//! Hygen write semantics from [`crate::write`] (Write).
//!
//! After a non-dry-run sync a praxis-core [`ReceiptRecord`] is chained over
//! the payload `{ graph_hash, outputs: { path → BLAKE3 } }` and written to
//! `<root>/.ggen-v2/receipt.json`.
//!
//! Receipt determinism note: `ts_ns` is fixed to `0`. praxis-core's live
//! `LawObject::receipt` path falls back to the system wall clock when
//! `ReceiptMeta::ts_ns` is `None`; this crate forbids wall clocks, so the
//! record is built directly and chained through
//! [`ReceiptRecord::recompute_chain_hash`], which uses the exact same
//! `build_admission_frame`/`chain_from_frame` construction as the live
//! emission path.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use oxigraph::sparql::QueryResults;
use praxis_core::{
    receipt_record::{ReceiptRecord, RECEIPT_RECORD_VERSION},
    Andon,
};
use serde::Serialize;
use tera::Value;

use crate::{
    config::GgenConfig,
    error::{AppError, Result},
    graph::DeterministicGraph,
    template::{build_tera, sparql_to_value, Template},
    write::{plan_write, WriteOutcome},
};

/// Options controlling a [`sync`] run.
#[derive(Debug, Clone, Copy, Default)]
pub struct SyncOptions {
    /// Compute outcomes without writing any file (and without a receipt).
    pub dry_run: bool,
}

/// The outcome of one [`sync`] run.
#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    /// Files created, overwritten, or injected into (project-root relative).
    pub written: Vec<PathBuf>,
    /// Files not written, with the reason.
    pub skipped: Vec<(PathBuf, String)>,
    /// BLAKE3 hex of the post-Enrich canonical graph state.
    pub graph_hash_hex: String,
    /// Root-relative output path → write decision ("written", "injected",
    /// "skipped: <reason>").
    pub decisions: BTreeMap<String, String>,
    /// Pack name → BLAKE3 hex of the pack's content hash.
    pub packs: BTreeMap<String, String>,
}

/// Relative path of the sync receipt under the project root.
pub const RECEIPT_REL_PATH: &str = ".ggen-v2/receipt.json";

/// One fully-rendered template awaiting `apply` — the boundary between the
/// "may fail" render pass and the write pass, which by construction only
/// begins once every template in the run has rendered successfully.
struct PendingWrite<'a> {
    to: String,
    body: String,
    tpl: &'a Template,
}

/// Relative path of the append-only receipt history log (one JSON line per
/// non-dry-run sync) under the project root.
pub const RECEIPT_LOG_REL_PATH: &str = ".ggen-v2/receipt-log.jsonl";

/// On-disk shape of `<root>/.ggen-v2/receipt.json`.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SyncReceipt {
    /// The praxis-core chain record over `payload`.
    pub record: ReceiptRecord,
    /// The hashed payload: graph hash plus per-output BLAKE3 hashes.
    pub payload: ReceiptPayload,
}

/// The payload a sync receipt is chained over.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ReceiptPayload {
    /// BLAKE3 hex of the post-Enrich canonical graph state.
    pub graph_hash: String,
    /// Root-relative output path → BLAKE3 hex of the file bytes on disk.
    pub outputs: BTreeMap<String, String>,
    /// Pack name → BLAKE3 hex of the pack's content hash.
    #[serde(default)]
    pub packs: BTreeMap<String, String>,
    /// Root-relative output path → why the file landed (or did not).
    #[serde(default)]
    pub decisions: BTreeMap<String, String>,
}

/// Run the five-stage pipeline rooted at `root` (the directory containing
/// `ggen.toml`).
///
/// # Errors
/// Fails closed on any resolve/parse/query/render/write failure; see the
/// FM-* codes on [`crate::config`], [`crate::graph`], [`crate::template`],
/// and [`crate::write`].
pub fn sync(root: &Path, opts: SyncOptions) -> Result<SyncReport> {
    // ── Stage 1: Resolve ────────────────────────────────────────────────
    let config = GgenConfig::load(&root.join("ggen.toml"))?;
    let ontology_path = root.join(&config.ontology.source);
    let ttl = std::fs::read_to_string(&ontology_path).map_err(|e| {
        AppError::fm_config(
            3,
            format!(
                "ontology `{}` unreadable: {e}. Remediation: fix [ontology].source.",
                ontology_path.display()
            ),
        )
    })?;
    let graph = DeterministicGraph::new()?;
    graph.insert_turtle(&ttl)?;

    // Resolve packs: union their ontologies into the same graph and append
    // their templates (packs sorted by name, then template path). Pack
    // content hashes are checked against ggen.lock before anything renders.
    let packs = crate::pack::resolve(&config, root)?;
    let lock_entries = crate::pack::lock_entries(&config, &packs)?;
    crate::pack::check_lock(root, &lock_entries)?;
    for pack in &packs {
        let pack_ttl = std::fs::read_to_string(&pack.ontology_path).map_err(|e| {
            AppError::fm_pack(
                4,
                format!(
                    "pack `{}`: ontology `{}` unreadable: {e}",
                    pack.name,
                    pack.ontology_path.display()
                ),
            )
        })?;
        graph.insert_turtle(&pack_ttl)?;
    }

    let templates = discover_templates(root, &config, &packs)?;

    // ── Stage 2: Enrich (single pass; see module docs) ──────────────────
    for (_, tpl) in &templates {
        if let Some(construct) = tpl.frontmatter.construct.as_deref() {
            insert_construct(&graph, construct)?;
        }
    }

    let graph_hash_hex = hex32(&graph.state_hash()?);
    let graph = Arc::new(graph);

    // ── Stages 3–4: Extract, Render every template into memory ──────────
    //
    // All rendering happens before any write. A render failure on template N
    // (a bad SPARQL query, a Tera error, a determinism mismatch) must leave
    // NO partial result on disk from templates 1..N-1 — so writes are
    // deferred to a second pass (below) that only starts once every
    // template has rendered successfully.
    let mut tera = build_tera(Arc::clone(&graph));
    let mut skipped: Vec<(PathBuf, String)> = Vec::new();
    let mut decisions: BTreeMap<String, String> = BTreeMap::new();
    let mut pending: Vec<PendingWrite<'_>> = Vec::new();

    for (tpl_path, tpl) in &templates {
        check_shape_files_exist(root, tpl_path, &tpl.frontmatter.shape)?;

        // Extract: ASK guard.
        if let Some(ask) = tpl.frontmatter.when.as_deref() {
            match graph.query(ask)? {
                QueryResults::Boolean(true) => {}
                QueryResults::Boolean(false) => {
                    let reason = format!("when guard false ({})", tpl_path.display());
                    decisions.insert(tpl.frontmatter.to.clone(), format!("skipped: {reason}"));
                    skipped.push((PathBuf::from(&tpl.frontmatter.to), reason));
                    continue;
                }
                _ => {
                    return Err(AppError::fm_tpl(
                        4,
                        format!(
                            "`when:` in {} must be an ASK query. \
                             Remediation: use ASK {{ … }}.",
                            tpl_path.display()
                        ),
                    ));
                }
            }
        }

        // Extract: named SELECTs → rows.
        let mut named: BTreeMap<String, Value> = BTreeMap::new();
        for (key, query) in &tpl.frontmatter.sparql {
            named.insert(key.clone(), sparql_to_value(&graph, query)?);
        }
        // Driving rows: the first named query (BTreeMap key order) with an
        // array result; empty when no SELECT produced rows.
        let results: Vec<Value> = named
            .values()
            .find_map(|v| v.as_array().cloned())
            .unwrap_or_default();

        let per_row = tpl.frontmatter.to.contains("{{");
        if per_row {
            for row in &results {
                let mut ctx = base_context(&named, &results);
                ctx.insert("row", row);
                if let Value::Object(map) = row {
                    for (k, v) in map {
                        ctx.insert(k, v);
                    }
                }
                let to = render_str(&mut tera, &tpl.frontmatter.to, &ctx, tpl_path)?;
                let body = render_str(&mut tera, &tpl.body, &ctx, tpl_path)?;
                check_determinism(
                    &mut tera,
                    &tpl.body,
                    &ctx,
                    tpl_path,
                    &tpl.frontmatter,
                    &body,
                )?;
                pending.push(PendingWrite { to, body, tpl });
            }
        } else {
            let ctx = base_context(&named, &results);
            let body = render_str(&mut tera, &tpl.body, &ctx, tpl_path)?;
            check_determinism(
                &mut tera,
                &tpl.body,
                &ctx,
                tpl_path,
                &tpl.frontmatter,
                &body,
            )?;
            pending.push(PendingWrite {
                to: tpl.frontmatter.to.clone(),
                body,
                tpl,
            });
        }
    }

    // ── Stage 5: Write every already-rendered template ───────────────────
    let mut written: Vec<PathBuf> = Vec::new();
    for pw in &pending {
        apply(
            root,
            &pw.to,
            &pw.body,
            pw.tpl,
            opts,
            &mut written,
            &mut skipped,
            &mut decisions,
        )?;
    }

    let pack_hashes: BTreeMap<String, String> = lock_entries
        .iter()
        .map(|e| {
            (
                e.name.clone(),
                e.content_hash.trim_start_matches("blake3:").to_string(),
            )
        })
        .collect();

    let report = SyncReport {
        written,
        skipped,
        graph_hash_hex,
        decisions,
        packs: pack_hashes,
    };

    if !opts.dry_run {
        write_receipt(root, &report)?;
        if !lock_entries.is_empty() {
            crate::pack::write_lock(root, &lock_entries)?;
        }
    }
    Ok(report)
}

/// Build the shared Tera context: `results` plus every named sparql key.
fn base_context(named: &BTreeMap<String, Value>, results: &[Value]) -> tera::Context {
    let mut ctx = tera::Context::new();
    ctx.insert("results", results);
    for (k, v) in named {
        ctx.insert(k, v);
    }
    ctx
}

/// Render a Tera string, mapping errors to `[FM-TPL-005]`.
fn render_str(
    tera: &mut tera::Tera,
    template: &str,
    ctx: &tera::Context,
    tpl_path: &Path,
) -> Result<String> {
    tera.render_str(template, ctx).map_err(|e| {
        AppError::fm_tpl(
            5,
            format!(
                "render failed for {}: {e}. Available top-level context keys: {}.",
                tpl_path.display(),
                context_key_summary(ctx)
            ),
        )
    })
}

/// Summarize a Tera context's top-level keys (and, for arrays, their
/// length) for a render-failure error message — enough to spot a typo'd
/// variable name (`row.nuon` vs `row.noun`) without dumping the full,
/// potentially large, context payload.
fn context_key_summary(ctx: &tera::Context) -> String {
    let Value::Object(map) = ctx.clone().into_json() else {
        return "(none)".to_string();
    };
    if map.is_empty() {
        return "(none)".to_string();
    }
    map.iter()
        .map(|(k, v)| match v {
            Value::Array(items) => format!("{k} (array, {} items)", items.len()),
            Value::Object(fields) => format!("{k} (object, {} fields)", fields.len()),
            other => format!("{k} ({})", value_type_name(other)),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Apply (or dry-run) one write and record the outcome and its decision.
#[allow(clippy::too_many_arguments)]
fn apply(
    root: &Path,
    rel_to: &str,
    body: &str,
    tpl: &Template,
    opts: SyncOptions,
    written: &mut Vec<PathBuf>,
    skipped: &mut Vec<(PathBuf, String)>,
    decisions: &mut BTreeMap<String, String>,
) -> Result<()> {
    if tpl.frontmatter.skip_empty && body.trim().is_empty() {
        let reason = "skip_empty: rendered body empty".to_string();
        decisions.insert(rel_to.to_string(), format!("skipped: {reason}"));
        skipped.push((PathBuf::from(rel_to), reason));
        return Ok(());
    }
    if opts.dry_run {
        // Dry run: classify without touching the filesystem or running
        // sh_before/sh_after (a dry run must have zero side effects).
        // Identical existing content reports Skipped(unchanged); everything
        // else is reported as a planned write.
        let target = root.join(rel_to);
        match std::fs::read_to_string(&target) {
            Ok(existing) if existing == body => {
                let reason = "unchanged: content identical".to_string();
                decisions.insert(rel_to.to_string(), format!("skipped: {reason}"));
                skipped.push((PathBuf::from(rel_to), reason));
            }
            _ => {
                decisions.insert(rel_to.to_string(), "planned: write (dry-run)".to_string());
                written.push(PathBuf::from(rel_to));
            }
        }
        return Ok(());
    }

    if let Some(cmd) = tpl.frontmatter.sh_before.as_deref() {
        run_shell_hook(root, cmd, "sh_before")?;
    }

    match plan_write(root, rel_to, body, &tpl.frontmatter)? {
        WriteOutcome::Written => {
            decisions.insert(rel_to.to_string(), "written".to_string());
            written.push(PathBuf::from(rel_to));
            if let Some(cmd) = tpl.frontmatter.sh_after.as_deref() {
                run_shell_hook(root, cmd, "sh_after")?;
            }
        }
        WriteOutcome::Injected => {
            decisions.insert(rel_to.to_string(), "injected".to_string());
            written.push(PathBuf::from(rel_to));
            if let Some(cmd) = tpl.frontmatter.sh_after.as_deref() {
                run_shell_hook(root, cmd, "sh_after")?;
            }
        }
        WriteOutcome::Skipped(reason) => {
            decisions.insert(rel_to.to_string(), format!("skipped: {reason}"));
            skipped.push((PathBuf::from(rel_to), reason));
        }
    }
    Ok(())
}

/// Refuse `cmd` against the shell-command denylist, then run it with `root`
/// as its working directory. Non-zero exit is a hard error.
fn run_shell_hook(root: &Path, cmd: &str, which: &str) -> Result<()> {
    crate::shell_safety::check_shell_command_safe(cmd)?;
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(root)
        .status()
        .map_err(|e| AppError::fm_shell(2, format!("{which} failed to launch: {e}")))?;
    if !status.success() {
        return Err(AppError::fm_shell(
            3,
            format!("{which} command `{cmd}` exited with {status}. Remediation: fix the command or remove it from frontmatter."),
        ));
    }
    Ok(())
}

/// Discover and parse every template a sync would process: project
/// templates (recursive under `[templates].dir`, sorted) followed by each
/// resolved pack's templates in pack order. Shared by [`sync`] and
/// `ggen graph validate` so both surfaces see the identical template set.
///
/// # Errors
/// Unreadable templates dir or any template parse failure (fail closed).
pub fn discover_templates(
    root: &Path,
    config: &GgenConfig,
    packs: &[crate::pack::Pack],
) -> Result<Vec<(PathBuf, Template)>> {
    let mut templates = load_templates(&root.join(&config.templates.dir))?;
    for pack in packs {
        for path in &pack.template_paths {
            templates.push((path.clone(), parse_template_file(path)?));
        }
    }
    Ok(templates)
}

/// Load and parse every `*.tmpl` under `dir` (recursive), in sorted path
/// order for deterministic processing.
fn load_templates(dir: &Path) -> Result<Vec<(PathBuf, Template)>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    collect_tmpl_paths(dir, &mut paths)?;
    paths.sort();
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        let tpl = parse_template_file(&path)?;
        out.push((path, tpl));
    }
    Ok(out)
}

/// Read and parse one `*.tmpl` file (identical frontmatter contract for
/// project and pack templates — no special casing). If the frontmatter sets
/// `from:`, the Tera body is replaced with the content of that path
/// (resolved relative to this template file's own directory); frontmatter
/// fields still come from `path` itself.
fn parse_template_file(path: &Path) -> Result<Template> {
    let content = std::fs::read_to_string(path)?;
    let mut tpl = Template::parse(&content)
        .map_err(|e| AppError::fm_tpl(6, format!("{}: {e}", path.display())))?;
    if let Some(from) = tpl.frontmatter.from.clone() {
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let from_path = crate::write::resolve_target(base, &from).map_err(|e| {
            AppError::fm_tpl(
                8,
                format!(
                    "{}: `from: {from}` is not a safe path (must stay inside the \
                     template's own directory): {e}",
                    path.display()
                ),
            )
        })?;
        tpl.body = std::fs::read_to_string(&from_path).map_err(|e| {
            AppError::fm_tpl(
                7,
                format!(
                    "{}: `from: {from}` unreadable at `{}`: {e}. \
                     Remediation: fix the `from:` path (relative to the template's own directory).",
                    path.display(),
                    from_path.display()
                ),
            )
        })?;
    }
    Ok(tpl)
}

/// Refuse if any `shape:` file listed in `frontmatter` does not exist.
/// **Existence-checked only** — no SHACL engine runs in this crate yet, so
/// this does not evaluate the shapes against rendered output; see
/// `docs/v26.7.4/GGEN_TOML_SCHEMA_MAPPING.md`.
fn check_shape_files_exist(root: &Path, tpl_path: &Path, shapes: &[String]) -> Result<()> {
    for shape in shapes {
        let shape_path = root.join(shape);
        if !shape_path.exists() {
            return Err(AppError::fm_tpl(
                8,
                format!(
                    "{}: `shape:` entry `{shape}` does not exist at `{}`. \
                     Remediation: fix the path or remove the entry.",
                    tpl_path.display(),
                    shape_path.display()
                ),
            ));
        }
    }
    Ok(())
}

/// When `frontmatter.determinism == Some(true)`, render the same template
/// body a second time with the identical context and refuse if the bytes
/// differ from `first_render` — a real, enforced assertion rather than a
/// declared-but-unchecked claim.
fn check_determinism(
    tera: &mut tera::Tera,
    body_template: &str,
    ctx: &tera::Context,
    tpl_path: &Path,
    frontmatter: &crate::template::Frontmatter,
    first_render: &str,
) -> Result<()> {
    if frontmatter.determinism != Some(true) {
        return Ok(());
    }
    let second_render = render_str(tera, body_template, ctx, tpl_path)?;
    if second_render != first_render {
        return Err(AppError::fm_tpl(
            9,
            format!(
                "{}: `determinism: true` violated — re-rendering the same template with the \
                 identical context produced different output. \
                 Remediation: remove non-deterministic Tera functions/filters from this template.",
                tpl_path.display()
            ),
        ));
    }
    Ok(())
}

/// Recursively collect `*.tmpl` files. A missing templates dir fails closed.
fn collect_tmpl_paths(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        AppError::fm_config(
            4,
            format!(
                "templates dir `{}` unreadable: {e}. Remediation: fix [templates].dir.",
                dir.display()
            ),
        )
    })?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_tmpl_paths(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "tmpl") {
            out.push(path);
        }
    }
    Ok(())
}

/// Run a CONSTRUCT query and insert its triples back into the graph.
fn insert_construct(graph: &DeterministicGraph, construct: &str) -> Result<()> {
    use std::fmt::Write as _;
    let QueryResults::Graph(triples) = graph.query(construct)? else {
        return Err(AppError::fm_graph(
            7,
            "`construct:` frontmatter must be a CONSTRUCT/DESCRIBE query. \
             Remediation: use CONSTRUCT { … } WHERE { … }.",
        ));
    };
    let mut doc = String::new();
    for triple in triples {
        let triple = triple
            .map_err(|e| AppError::fm_graph(7, format!("CONSTRUCT iteration failed: {e}")))?;
        let _ = writeln!(doc, "{triple} .");
    }
    if !doc.is_empty() {
        // N-Triples is a syntactic subset of Turtle.
        graph.insert_turtle(&doc)?;
    }
    Ok(())
}

/// Chain a praxis-core [`ReceiptRecord`] over `{ graph_hash, outputs }` and
/// write it to [`RECEIPT_REL_PATH`]. `ts_ns` is fixed to 0 (no wall clock;
/// see module docs).
fn write_receipt(root: &Path, report: &SyncReport) -> Result<()> {
    use std::io::Write as _;

    // Bind every decision target that exists on disk (written this run or
    // skipped-unchanged), so a no-op re-sync produces the identical payload.
    let mut outputs = BTreeMap::new();
    for rel in report.decisions.keys() {
        let target = root.join(rel);
        if let Ok(bytes) = std::fs::read(&target) {
            outputs.insert(rel.clone(), blake3::hash(&bytes).to_hex().to_string());
        }
    }
    let payload = ReceiptPayload {
        graph_hash: report.graph_hash_hex.clone(),
        outputs,
        packs: report.packs.clone(),
        decisions: report.decisions.clone(),
    };
    let payload_bytes = serde_json::to_vec(&payload)?;
    let payload_hash_hex = blake3::hash(&payload_bytes).to_hex().to_string();

    // Chain onto the previous sync's receipt when one exists; a genesis
    // receipt (no prior `.ggen-v2/receipt.json`) chains from all-zeros.
    let receipt_path = root.join(RECEIPT_REL_PATH);
    let prev_chain_hash_hex = match std::fs::read_to_string(&receipt_path) {
        Ok(raw) => {
            let prev: SyncReceipt = serde_json::from_str(&raw).map_err(|e| {
                AppError::fm_chain(
                    3,
                    format!(
                        "previous receipt `{}` malformed: {e}. \
                         Remediation: verify or remove the stale receipt.",
                        receipt_path.display()
                    ),
                )
            })?;
            prev.record.chain_hash_hex
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => "0".repeat(64),
        Err(e) => {
            return Err(AppError::fm_chain(
                3,
                format!(
                    "previous receipt `{}` unreadable: {e}",
                    receipt_path.display()
                ),
            ))
        }
    };

    let mut record = ReceiptRecord {
        version: RECEIPT_RECORD_VERSION,
        instruction_id: 0,
        activity_idx: 0,
        activity: Some("ggen.sync".to_string()),
        node_kind: 0,
        ts_ns: 0,
        duration_ms: None,
        object_ids: vec![format!("law:{}", &payload_hash_hex[..16])],
        payload_hash_hex,
        prev_chain_hash_hex,
        chain_hash_hex: String::new(),
        andon: Andon::Green,
        obligation_count: 0,
    };
    let chain = record
        .recompute_chain_hash()
        .map_err(|e| AppError::fm_chain(2, format!("receipt chain computation failed: {e}")))?;
    record.chain_hash_hex = hex32(&chain);

    let receipt = SyncReceipt { record, payload };
    if let Some(parent) = receipt_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt)?)?;

    // Append-only history: one compact JSON line per non-dry-run sync.
    let log_path = root.join(RECEIPT_LOG_REL_PATH);
    let mut line = serde_json::to_vec(&receipt)?;
    line.push(b'\n');
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut f| f.write_all(&line))
        .map_err(|e| {
            AppError::fm_chain(
                4,
                format!("receipt log `{}` append failed: {e}", log_path.display()),
            )
        })?;
    Ok(())
}

/// Lowercase hex of a 32-byte hash.
pub(crate) fn hex32(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}
