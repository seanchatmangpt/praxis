//! Task-spec ontology loader.
//!
//! Each task is authored once as a `.ttl` (Turtle) file using a small `tb:` vocabulary
//! layered directly on the `ggen:` prompt-manufacturing concepts (`Section`/`Block`/
//! `Instruction`/`Code`). Per the first-principles review (see
//! `~/.claude/plans/crispy-squishing-garden.md`, finding F1), both
//! `ggen_core::prompt_mfg::ir::PromptIR::from_construct` and `::from_store` are stubs
//! that discard triple content, so **this module does its own triple-walking** with
//! real oxigraph SPARQL SELECT queries and hands the fully-populated
//! [`ggen_core::prompt_mfg::ir::PromptIR`] to ggen's working emitter/validator/hash
//! path via `PromptCompiler::compile_from_ir` (see [`crate::prompt`]).
//!
//! # Vocabulary
//!
//! - `tb:` — `http://praxis.dev/ns/testbed#` — task-spec terms (`Task`, `id`,
//!   `taskType`, `difficulty`, `model`, `description`, `promptSection`, `fixture`,
//!   `targetPath` (optional), `expectedSteps`, `passCriteria`, `cargoTest`,
//!   `clippyDenyWarnings`, and the `taskType` object values `FunctionLevelBugfix` /
//!   `RepoLevelTranslation` / `UnsafeAudit` / `CryptoCodegen`, and the `expectedSteps`
//!   list member values `Build` / `Test` / `Clippy` / `SafetyAudit`).
//! - `ggen:` — `http://praxis.dev/ns/ggen-prompt#` — prompt-shape terms (`Section`,
//!   `role`, `block`, `Instruction`, `Code`, `text`, `lang`, `path`). This is a local
//!   vocabulary mirroring `ggen_core::prompt_mfg::ir` concept names for readability; it
//!   is not the same IRI space as any RDF ontology shipped by `ggen` itself, since the
//!   testbed builds `PromptIR` values directly in Rust rather than via a shared schema.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use ggen_core::prompt_mfg::ir::{
    BlockType, ContentBlock, PromptIR, PromptMetadata, Section, SectionType,
};
use oxigraph::io::RdfFormat;
use oxigraph::model::{Term, TermRef};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

/// `tb:` vocabulary namespace.
pub const TB_NS: &str = "http://praxis.dev/ns/testbed#";
/// `ggen:` vocabulary namespace (testbed-local; see module docs).
pub const GGEN_NS: &str = "http://praxis.dev/ns/ggen-prompt#";
const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// Errors specific to loading/parsing a task spec.
#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    /// The `.ttl` file could not be read from disk.
    #[error("failed to read task spec {path}: {source}")]
    Read {
        /// Path that failed to read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The Turtle content failed to parse or load into the store.
    #[error("failed to parse turtle in {path}: {message}")]
    Parse {
        /// Path whose content failed to parse.
        path: PathBuf,
        /// Human-readable parser error.
        message: String,
    },

    /// A SPARQL query failed to execute.
    #[error("sparql query failed: {0}")]
    Sparql(String),

    /// The task spec is missing a required field or has an unexpected shape.
    #[error("task spec {path} is missing or malformed: {message}")]
    Shape {
        /// Path of the offending task spec.
        path: PathBuf,
        /// What was expected/missing.
        message: String,
    },

    /// A `ggen:path`-referenced source file could not be read.
    #[error("failed to read referenced source {path}: {source}")]
    ReadReferenced {
        /// Path that failed to read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// Result alias scoped to spec loading.
pub type SpecResult<T> = std::result::Result<T, SpecError>;

/// The four task-type buckets identified by the Rust/Claude research corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskType {
    /// Fix a bug confined to a single function.
    FunctionLevelBugfix,
    /// Translate/port a whole repository or module between languages/idioms.
    RepoLevelTranslation,
    /// Audit `unsafe` usage for soundness.
    UnsafeAudit,
    /// Generate cryptographic code.
    CryptoCodegen,
}

impl TaskType {
    /// Parse from a `tb:` local name (e.g. `"FunctionLevelBugfix"`).
    fn from_local_name(name: &str) -> SpecResult<Self> {
        match name {
            "FunctionLevelBugfix" => Ok(Self::FunctionLevelBugfix),
            "RepoLevelTranslation" => Ok(Self::RepoLevelTranslation),
            "UnsafeAudit" => Ok(Self::UnsafeAudit),
            "CryptoCodegen" => Ok(Self::CryptoCodegen),
            other => Err(SpecError::Shape {
                path: PathBuf::new(),
                message: format!("unknown tb:taskType value '{other}'"),
            }),
        }
    }
}

/// The kind of a single prompt content block, plus its resolved textual content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptBlockKind {
    /// Free-form instruction text (`ggen:Instruction`).
    Instruction,
    /// A source-code excerpt (`ggen:Code`), tagged with its language.
    Code {
        /// Source language (e.g. `"rust"`).
        language: String,
    },
}

/// One resolved content block within a [`PromptSectionSpec`].
///
/// `content` is fully resolved at [`load_task`] time: for `Instruction` blocks this is
/// the `ggen:text` literal; for `Code` blocks whose triple carries a `ggen:path`, the
/// referenced file (resolved relative to the `.ttl` file's parent directory) has
/// already been read into `content`, so [`task_to_prompt_ir`] itself needs no I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptBlockSpec {
    /// Block kind.
    pub kind: PromptBlockKind,
    /// Resolved textual content, ready to drop into a `ggen_core` `ContentBlock`.
    pub content: String,
    /// Original `ggen:path` value, if the block was file-backed (kept for diagnostics).
    pub source_path: Option<String>,
}

/// One `tb:promptSection` — a role (`system`/`user`/`assistant`/custom) plus its
/// ordered content blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptSectionSpec {
    /// Section role, e.g. `"system"`, `"user"`, `"assistant"`.
    pub role: String,
    /// Ordered content blocks.
    pub blocks: Vec<PromptBlockSpec>,
}

/// `tb:passCriteria` — machine-checkable pass conditions for a task.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PassCriteria {
    /// `tb:cargoTest` — the exact `cargo test` invocation that must succeed.
    pub cargo_test: Option<String>,
    /// `tb:clippyDenyWarnings` — whether clippy must be run with `-D warnings`.
    pub clippy_deny_warnings: bool,
}

/// A fully-loaded task specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSpec {
    /// `tb:id` — stable task identifier.
    pub id: String,
    /// `tb:taskType` — which research-corpus bucket this task exercises.
    pub task_type: TaskType,
    /// `tb:difficulty` — free-form difficulty label (e.g. `"medium"`).
    pub difficulty: String,
    /// `tb:model` — default model identifier to target (overridable by `--model`).
    pub model: String,
    /// `tb:description` — human-readable task description; doubles as `spec.md` intro.
    pub description: String,
    /// `tb:fixture` — path (relative to the `.ttl` file's directory, unless absolute)
    /// to the scratch Cargo project template this task operates on.
    pub fixture: PathBuf,
    /// `tb:targetPath` — the file (relative to the fixture's root, e.g. `"src/lib.rs"`
    /// or `"src/describe.rs"`) that the model's response should overwrite. `None` for
    /// tasks authored before this field existed; callers fall back to the single
    /// Code-block heuristic in that case (see [`crate::spec`] module docs and
    /// `bin/testbed.rs`'s `resolve_target_path`). Required for any task whose prompt
    /// includes more than one `ggen:Code` block (e.g. `RepoLevelTranslation` tasks that
    /// show read-only context files alongside the file to fix) since the multi-block
    /// case is otherwise ambiguous.
    pub target_path: Option<PathBuf>,
    /// `tb:expectedSteps` — ordered pipeline stage names (`Build`, `Test`, `Clippy`,
    /// `SafetyAudit`, ...), rendered as `tasks.md` rows.
    pub expected_steps: Vec<String>,
    /// `tb:passCriteria`.
    pub pass_criteria: PassCriteria,
    /// `tb:promptSection` values, in document order, ready to compile into a
    /// [`ggen_core::prompt_mfg::ir::PromptIR`] via [`task_to_prompt_ir`].
    pub prompt_sections: Vec<PromptSectionSpec>,
}

// ── Loading ─────────────────────────────────────────────────────────────────

/// Load and fully resolve a task spec from a `.ttl` file.
///
/// Parses the Turtle content into an in-memory oxigraph [`Store`], runs SPARQL SELECT
/// queries over the `tb:`/`ggen:` vocabulary to extract every field, and reads any
/// `ggen:path`-referenced source files (resolved relative to `ttl_path`'s parent
/// directory) into their block's `content`.
///
/// # Errors
///
/// Returns [`SpecError`] if the file can't be read, the Turtle can't be parsed, a
/// required triple is missing, or a referenced source file can't be read.
pub fn load_task(ttl_path: &Path) -> SpecResult<TaskSpec> {
    let turtle = std::fs::read_to_string(ttl_path).map_err(|source| SpecError::Read {
        path: ttl_path.to_path_buf(),
        source,
    })?;

    let store = Store::new().map_err(|e| SpecError::Parse {
        path: ttl_path.to_path_buf(),
        message: e.to_string(),
    })?;
    store
        .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
        .map_err(|e| SpecError::Parse {
            path: ttl_path.to_path_buf(),
            message: e.to_string(),
        })?;

    let base_dir = ttl_path.parent().unwrap_or_else(|| Path::new("."));

    let (task_iri, id, task_type_term, difficulty, model, description, fixture, target_path) =
        load_scalars(&store, ttl_path)?;

    let sections = load_prompt_sections(&store, &task_iri, base_dir, ttl_path)?;
    let expected_steps = load_expected_steps(&store, &task_iri, ttl_path)?;
    let pass_criteria = load_pass_criteria(&store, &task_iri, ttl_path)?;

    let task_type_name = iri_local_name(&task_type_term).ok_or_else(|| SpecError::Shape {
        path: ttl_path.to_path_buf(),
        message: "tb:taskType value must be an IRI".to_string(),
    })?;
    let task_type = TaskType::from_local_name(&task_type_name)?;

    Ok(TaskSpec {
        id,
        task_type,
        difficulty,
        model,
        description,
        fixture: PathBuf::from(fixture),
        target_path: target_path.map(PathBuf::from),
        expected_steps,
        pass_criteria,
        prompt_sections: sections,
    })
}

#[allow(clippy::type_complexity)]
fn load_scalars(
    store: &Store, ttl_path: &Path,
) -> SpecResult<(String, String, Term, String, String, String, String, Option<String>)> {
    let query = format!(
        r"PREFIX tb: <{TB_NS}>
SELECT ?task ?id ?taskType ?difficulty ?model ?description ?fixture ?targetPath WHERE {{
  ?task a tb:Task ;
        tb:id ?id ;
        tb:taskType ?taskType ;
        tb:difficulty ?difficulty ;
        tb:model ?model ;
        tb:description ?description ;
        tb:fixture ?fixture .
  OPTIONAL {{ ?task tb:targetPath ?targetPath }}
}}"
    );
    let mut rows = run_select(store, &query)?;
    let row = rows
        .next()
        .ok_or_else(|| SpecError::Shape {
            path: ttl_path.to_path_buf(),
            message: "no tb:Task with all required scalar fields found".to_string(),
        })?
        .map_err(|e| SpecError::Sparql(e.to_string()))?;

    let task = get_iri(&row, "task", ttl_path)?;
    let id = get_literal(&row, "id", ttl_path)?;
    let task_type = get_term(&row, "taskType", ttl_path)?;
    let difficulty = get_literal(&row, "difficulty", ttl_path)?;
    let model = get_literal(&row, "model", ttl_path)?;
    let description = get_literal(&row, "description", ttl_path)?;
    let fixture = get_literal(&row, "fixture", ttl_path)?;
    let target_path = get_optional_literal(&row, "targetPath");

    Ok((task, id, task_type, difficulty, model, description, fixture, target_path))
}

fn load_prompt_sections(
    store: &Store, task_iri: &str, base_dir: &Path, ttl_path: &Path,
) -> SpecResult<Vec<PromptSectionSpec>> {
    let section_query = format!(
        r"PREFIX tb: <{TB_NS}>
PREFIX ggen: <{GGEN_NS}>
SELECT ?section ?role WHERE {{
  <{task_iri}> tb:promptSection ?section .
  ?section ggen:role ?role .
}}"
    );
    let section_rows: Vec<(Term, String)> = run_select(store, &section_query)?
        .map(|row| {
            let row = row.map_err(|e| SpecError::Sparql(e.to_string()))?;
            let section = get_term(&row, "section", ttl_path)?;
            let role = get_literal(&row, "role", ttl_path)?;
            Ok::<_, SpecError>((section, role))
        })
        .collect::<SpecResult<Vec<_>>>()?;

    let block_query = format!(
        r"PREFIX tb: <{TB_NS}>
PREFIX ggen: <{GGEN_NS}>
PREFIX rdf: <{RDF_NS}>
SELECT ?section ?block ?blockType ?text ?lang ?path WHERE {{
  <{task_iri}> tb:promptSection ?section .
  ?section ggen:block ?block .
  ?block rdf:type ?blockType .
  OPTIONAL {{ ?block ggen:text ?text }}
  OPTIONAL {{ ?block ggen:lang ?lang }}
  OPTIONAL {{ ?block ggen:path ?path }}
}}"
    );

    let mut blocks_by_section: HashMap<Term, Vec<PromptBlockSpec>> = HashMap::new();
    for row in run_select(store, &block_query)? {
        let row = row.map_err(|e| SpecError::Sparql(e.to_string()))?;
        let section = get_term(&row, "section", ttl_path)?;
        let block_type_term = get_term(&row, "blockType", ttl_path)?;
        let block_type = iri_local_name(&block_type_term).ok_or_else(|| SpecError::Shape {
            path: ttl_path.to_path_buf(),
            message: "block rdf:type must be an IRI".to_string(),
        })?;
        let text = get_optional_literal(&row, "text");
        let lang = get_optional_literal(&row, "lang");
        let path = get_optional_literal(&row, "path");

        let (kind, content, source_path) = match block_type.as_str() {
            "Code" => {
                let language = lang.unwrap_or_else(|| "rust".to_string());
                let content = if let Some(rel) = path.clone() {
                    let full = base_dir.join(&rel);
                    std::fs::read_to_string(&full).map_err(|source| SpecError::ReadReferenced {
                        path: full,
                        source,
                    })?
                } else {
                    text.unwrap_or_default()
                };
                (PromptBlockKind::Code { language }, content, path)
            }
            _ => (PromptBlockKind::Instruction, text.unwrap_or_default(), path),
        };

        blocks_by_section
            .entry(section)
            .or_default()
            .push(PromptBlockSpec { kind, content, source_path });
    }

    let mut sections: Vec<PromptSectionSpec> = section_rows
        .into_iter()
        .map(|(section, role)| PromptSectionSpec {
            role,
            blocks: blocks_by_section.remove(&section).unwrap_or_default(),
        })
        .collect();

    // SPARQL SELECT row order for the (blank-node-keyed) sections isn't guaranteed to
    // match the .ttl document's declaration order, so impose a canonical role order
    // (system, then user, then assistant, then anything else) rather than relying on
    // incidental store iteration order. This is a stable sort: sections sharing a role
    // keep their relative SPARQL-returned order.
    sections.sort_by_key(|s| role_rank(&s.role));
    Ok(sections)
}

/// Canonical ordering key for prompt-section roles (see [`load_prompt_sections`]).
fn role_rank(role: &str) -> u8 {
    match role {
        "system" => 0,
        "user" => 1,
        "assistant" => 2,
        _ => 3,
    }
}

/// Walk an `rdf:List` reachable via `tb:expectedSteps`, returning member local names
/// in list order.
fn load_expected_steps(store: &Store, task_iri: &str, ttl_path: &Path) -> SpecResult<Vec<String>> {
    let head_query = format!(
        r"PREFIX tb: <{TB_NS}>
SELECT ?head WHERE {{ <{task_iri}> tb:expectedSteps ?head . }}"
    );
    let mut head_rows = run_select(store, &head_query)?;
    let Some(head_row) = head_rows.next() else {
        return Ok(Vec::new());
    };
    let head_row = head_row.map_err(|e| SpecError::Sparql(e.to_string()))?;
    let head = get_term(&head_row, "head", ttl_path)?;

    let nodes_query = format!(
        r"PREFIX tb: <{TB_NS}>
PREFIX rdf: <{RDF_NS}>
SELECT ?node ?item ?next WHERE {{
  <{task_iri}> tb:expectedSteps/rdf:rest* ?node .
  OPTIONAL {{ ?node rdf:first ?item }}
  OPTIONAL {{ ?node rdf:rest ?next }}
}}"
    );
    let mut chain: HashMap<Term, (Option<Term>, Option<Term>)> = HashMap::new();
    for row in run_select(store, &nodes_query)? {
        let row = row.map_err(|e| SpecError::Sparql(e.to_string()))?;
        let node = get_term(&row, "node", ttl_path)?;
        let item = get_optional_term(&row, "item");
        let next = get_optional_term(&row, "next");
        chain.insert(node, (item, next));
    }

    let mut steps = Vec::new();
    let mut current = Some(head);
    let mut guard = 0usize;
    while let Some(node) = current {
        guard += 1;
        if guard > 4096 {
            return Err(SpecError::Shape {
                path: ttl_path.to_path_buf(),
                message: "tb:expectedSteps rdf:List exceeds sanity limit (cycle?)".to_string(),
            });
        }
        let Some((item, next)) = chain.get(&node).cloned() else {
            break;
        };
        if let Some(item) = item {
            let name = iri_local_name(&item).unwrap_or_else(|| item.to_string());
            steps.push(name);
        }
        current = next;
    }
    Ok(steps)
}

fn load_pass_criteria(store: &Store, task_iri: &str, ttl_path: &Path) -> SpecResult<PassCriteria> {
    let query = format!(
        r"PREFIX tb: <{TB_NS}>
SELECT ?cargoTest ?clippyDenyWarnings WHERE {{
  <{task_iri}> tb:passCriteria ?pc .
  OPTIONAL {{ ?pc tb:cargoTest ?cargoTest }}
  OPTIONAL {{ ?pc tb:clippyDenyWarnings ?clippyDenyWarnings }}
}}"
    );
    let mut rows = run_select(store, &query)?;
    let Some(row) = rows.next() else {
        return Ok(PassCriteria::default());
    };
    let row = row.map_err(|e| SpecError::Sparql(e.to_string()))?;
    let cargo_test = get_optional_literal(&row, "cargoTest");
    let clippy_deny_warnings =
        get_optional_literal(&row, "clippyDenyWarnings").is_some_and(|v| v == "true");
    let _ = ttl_path;
    Ok(PassCriteria { cargo_test, clippy_deny_warnings })
}

// ── SPARQL helpers ────────────────────────────────────────────────────────

type Row = oxigraph::sparql::QuerySolution;

fn run_select<'a>(
    store: &'a Store, query: &str,
) -> SpecResult<Box<dyn Iterator<Item = std::result::Result<Row, oxigraph::sparql::QueryEvaluationError>> + 'a>>
{
    let results = SparqlEvaluator::new()
        .parse_query(query)
        .map_err(|e| SpecError::Sparql(e.to_string()))?
        .on_store(store)
        .execute()
        .map_err(|e| SpecError::Sparql(e.to_string()))?;
    match results {
        QueryResults::Solutions(solutions) => Ok(Box::new(solutions)),
        QueryResults::Boolean(_) | QueryResults::Graph(_) => Err(SpecError::Sparql(
            "expected SELECT results, got ASK/CONSTRUCT results".to_string(),
        )),
    }
}

fn get_term(row: &Row, var: &str, ttl_path: &Path) -> SpecResult<Term> {
    row.get(var).cloned().ok_or_else(|| SpecError::Shape {
        path: ttl_path.to_path_buf(),
        message: format!("missing binding for ?{var}"),
    })
}

fn get_optional_term(row: &Row, var: &str) -> Option<Term> {
    row.get(var).cloned()
}

fn get_iri(row: &Row, var: &str, ttl_path: &Path) -> SpecResult<String> {
    let term = get_term(row, var, ttl_path)?;
    iri_local_name_full(&term).ok_or_else(|| SpecError::Shape {
        path: ttl_path.to_path_buf(),
        message: format!("?{var} must be an IRI"),
    })
}

fn get_literal(row: &Row, var: &str, ttl_path: &Path) -> SpecResult<String> {
    let term = get_term(row, var, ttl_path)?;
    match term {
        Term::Literal(lit) => Ok(lit.value().to_string()),
        other => Err(SpecError::Shape {
            path: ttl_path.to_path_buf(),
            message: format!("?{var} expected a literal, got {other}"),
        }),
    }
}

fn get_optional_literal(row: &Row, var: &str) -> Option<String> {
    match row.get(var)? {
        Term::Literal(lit) => Some(lit.value().to_string()),
        _ => None,
    }
}

/// Returns the full IRI string (without angle brackets) for a `NamedNode` term.
fn iri_local_name_full(term: &Term) -> Option<String> {
    match term {
        Term::NamedNode(n) => Some(n.as_str().to_string()),
        _ => None,
    }
}

/// Returns the fragment/last-path-segment local name for a `NamedNode` term, e.g.
/// `"FunctionLevelBugfix"` for `<http://praxis.dev/ns/testbed#FunctionLevelBugfix>`.
fn iri_local_name(term: &Term) -> Option<String> {
    let TermRef::NamedNode(n) = term.as_ref() else {
        return None;
    };
    let s = n.as_str();
    let local = s
        .rsplit_once('#')
        .map_or_else(|| s.rsplit_once('/').map_or(s, |(_, tail)| tail), |(_, tail)| tail);
    Some(local.to_string())
}

// ── PromptIR construction ────────────────────────────────────────────────────

/// Build a [`ggen_core::prompt_mfg::ir::PromptIR`] from a fully-loaded [`TaskSpec`].
///
/// Pure and I/O-free: every block's `content` was already resolved (including reading
/// any `ggen:path`-referenced source file) by [`load_task`]. Sections are keyed by role
/// name in a `BTreeMap` (as `PromptIR` requires for deterministic ordering); document
/// order among same-named sections is preserved via ascending `priority`.
#[must_use]
pub fn task_to_prompt_ir(task: &TaskSpec) -> PromptIR {
    let mut sections: BTreeMap<String, Section> = BTreeMap::new();
    for (idx, spec_section) in task.prompt_sections.iter().enumerate() {
        let section_type = match spec_section.role.as_str() {
            "system" => SectionType::System,
            "user" => SectionType::User,
            "assistant" => SectionType::Assistant,
            other => SectionType::Custom(other.to_string()),
        };
        let blocks: Vec<ContentBlock> = spec_section
            .blocks
            .iter()
            .map(|b| {
                let block_type = match &b.kind {
                    PromptBlockKind::Instruction => BlockType::Instruction,
                    PromptBlockKind::Code { language } => BlockType::Code { language: language.clone() },
                };
                let mut metadata = BTreeMap::new();
                if let Some(path) = &b.source_path {
                    metadata.insert("source_path".to_string(), path.clone());
                }
                ContentBlock { block_type, content: b.content.clone(), metadata }
            })
            .collect();

        // BTreeMap key: role name, disambiguated for repeats of the same role so no
        // section silently overwrites another (validator requires non-empty keys and
        // each standard section to carry at least one block).
        let key = if sections.contains_key(&spec_section.role) {
            format!("{}_{idx}", spec_section.role)
        } else {
            spec_section.role.clone()
        };
        sections.insert(
            key,
            Section { section_type, blocks, priority: i32::try_from(idx).unwrap_or(i32::MAX) },
        );
    }

    PromptMetadataBuilder::new(task).build_with(sections)
}

/// Small builder to keep [`task_to_prompt_ir`]'s metadata construction readable.
struct PromptMetadataBuilder<'a> {
    task: &'a TaskSpec,
}

impl<'a> PromptMetadataBuilder<'a> {
    fn new(task: &'a TaskSpec) -> Self {
        Self { task }
    }

    fn build_with(self, sections: BTreeMap<String, Section>) -> PromptIR {
        PromptIR {
            sections,
            metadata: PromptMetadata {
                id: self.task.id.clone(),
                version: "0.1.0".to_string(),
                schema_version: "1.0.0".to_string(),
                source_ontology: format!("{TB_NS}{}", self.task.id),
                construct_query: "n/a: IR built manually from parsed .ttl triples \
                    (ggen PromptIR::from_construct/from_store are stubs; see spec.rs docs)"
                    .to_string(),
            },
            variables: BTreeMap::new(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp_task(dir: &Path) -> PathBuf {
        let fixture_rs = dir.join("snippet.rs");
        std::fs::write(&fixture_rs, "fn add(a: i32, b: i32) -> i32 { a + b }\n")
            .expect("write fixture snippet");

        let ttl = format!(
            r#"@prefix tb: <{TB_NS}> .
@prefix ggen: <{GGEN_NS}> .

tb:function_bugfix_001 a tb:Task ;
    tb:id "function_bugfix_001" ;
    tb:taskType tb:FunctionLevelBugfix ;
    tb:difficulty "medium" ;
    tb:model "claude-opus-4-8" ;
    tb:description "Fix an off-by-one in a binary search fn." ;
    tb:promptSection [ a ggen:Section ; ggen:role "system" ;
        ggen:block [ a ggen:Instruction ; ggen:text "You are a careful Rust engineer." ] ] ;
    tb:promptSection [ a ggen:Section ; ggen:role "user" ;
        ggen:block [ a ggen:Code ; ggen:lang "rust" ; ggen:path "snippet.rs" ] ;
        ggen:block [ a ggen:Instruction ; ggen:text "Function returns wrong index on duplicates. Fix it." ] ] ;
    tb:fixture "fixtures/function_bugfix_001/" ;
    tb:expectedSteps ( tb:Build tb:Test tb:Clippy tb:SafetyAudit ) ;
    tb:passCriteria [ tb:cargoTest "cargo test" ; tb:clippyDenyWarnings true ] .
"#
        );
        let ttl_path = dir.join("function_bugfix_001.ttl");
        std::fs::write(&ttl_path, ttl).expect("write ttl");
        ttl_path
    }

    #[test]
    fn loads_full_task_spec_from_turtle() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ttl_path = write_temp_task(dir.path());

        let task = load_task(&ttl_path).expect("load_task should succeed");

        assert_eq!(task.id, "function_bugfix_001");
        assert_eq!(task.task_type, TaskType::FunctionLevelBugfix);
        assert_eq!(task.difficulty, "medium");
        assert_eq!(task.model, "claude-opus-4-8");
        assert!(task.description.contains("off-by-one"));
        assert_eq!(task.fixture, PathBuf::from("fixtures/function_bugfix_001/"));
        assert_eq!(task.expected_steps, vec!["Build", "Test", "Clippy", "SafetyAudit"]);
        assert_eq!(task.pass_criteria.cargo_test.as_deref(), Some("cargo test"));
        assert!(task.pass_criteria.clippy_deny_warnings);

        assert_eq!(task.prompt_sections.len(), 2);
        let system = &task.prompt_sections[0];
        assert_eq!(system.role, "system");
        assert_eq!(system.blocks.len(), 1);
        assert_eq!(system.blocks[0].kind, PromptBlockKind::Instruction);

        let user = &task.prompt_sections[1];
        assert_eq!(user.role, "user");
        assert_eq!(user.blocks.len(), 2);
        let code_block = user.blocks.iter().find(|b| matches!(b.kind, PromptBlockKind::Code { .. }));
        let code_block = code_block.expect("expected a Code block");
        assert!(code_block.content.contains("fn add"));
    }

    #[test]
    fn task_to_prompt_ir_builds_valid_ir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ttl_path = write_temp_task(dir.path());
        let task = load_task(&ttl_path).expect("load_task should succeed");

        let ir = task_to_prompt_ir(&task);
        assert_eq!(ir.metadata.id, "function_bugfix_001");
        assert!(ir.sections.contains_key("system"));
        assert!(ir.sections.contains_key("user"));

        let compiler = ggen_core::prompt_mfg::PromptCompiler::new().expect("compiler init");
        let compiled = compiler.compile_from_ir(ir).expect("compile_from_ir should succeed");
        assert!(!compiled.content().is_empty());
    }
}
