//! `ggen.toml` configuration model, loaded via `star_toml`.
//!
//! Structural shape is closed at parse time (`deny_unknown_fields` on every
//! table, matching `schema/ggen-toml-schema.ttl` field-for-field — see
//! `tests/ggen_toml_schema_match.rs`). Semantic constraints beyond shape
//! (non-empty names, no path traversal in `[ontology].source`/`[templates].dir`)
//! are enforced by implementing `star_toml`'s [`star_toml::Validate`] trait
//! and running it via [`star_toml::Validate::validated`] after deserialization
//! — reusing `star_toml::Validator::check_path`'s existing traversal/null-byte
//! checks rather than reimplementing path safety here.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use star_toml::{Validate, Validator};

use crate::error::{AppError, Result};

/// Root model of a `ggen.toml` manifest.
///
/// `#[derive(JsonSchema)]` is load-bearing, not decorative: it is what lets
/// `tests/ggen_toml_schema_match.rs` compare this struct's *actual* field
/// set (via `schemars::schema_for!`) against `schema/ggen-toml-schema.ttl`,
/// instead of a hand-maintained mirror list that could itself drift — the
/// exact failure mode found in every sibling implementation's LSP/validator.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GgenConfig {
    /// `[project]` table.
    pub project: Project,
    /// `[ontology]` table.
    pub ontology: Ontology,
    /// `[packs]` table: pack name → source reference.
    #[serde(default)]
    pub packs: BTreeMap<String, PackRef>,
    /// `[templates]` table.
    pub templates: Templates,
    /// `[law]` table: N3/Datalog rule files and SHACL shapes gating a sync.
    /// Optional — an absent table means no law stage runs (existing
    /// projects unchanged).
    #[serde(default)]
    pub law: Law,
}

/// `[law]` — law-state inputs for the sync pipeline: rule files are
/// materialized into the graph after the Enrich stage; shapes files gate
/// rendering (violations are a typed refusal).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Law {
    /// N3/Datalog rule file paths, relative to the manifest, loaded in
    /// listed order.
    #[serde(default)]
    pub rules: Vec<PathBuf>,
    /// Turtle SHACL shapes file paths, relative to the manifest, each
    /// validated against the post-materialization graph.
    #[serde(default)]
    pub shapes: Vec<PathBuf>,
}

/// `[project]` — identity of the generating project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// Project name.
    pub name: String,
}

/// `[ontology]` — the RDF source of truth and its namespace prefixes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Ontology {
    /// Path to the ontology file (Turtle), relative to the manifest.
    pub source: PathBuf,
    /// Prefix → namespace IRI map.
    #[serde(default)]
    pub prefixes: BTreeMap<String, String>,
}

/// A pack source reference: either a local path or a git coordinate.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum PackRef {
    /// Local pack: `{ path = "…" }`.
    Path {
        /// Filesystem path to the pack directory.
        path: PathBuf,
    },
    /// Remote pack: `{ git = "…", version = "…" }`.
    Git {
        /// Git repository URL.
        git: String,
        /// Version requirement (tag or semver).
        version: String,
    },
}

/// `[templates]` — where Tera templates live.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Templates {
    /// Template directory, relative to the manifest.
    pub dir: PathBuf,
}

impl GgenConfig {
    /// Load and parse a `ggen.toml` file, then run semantic validation.
    ///
    /// # Errors
    /// Returns `[FM-CONFIG-001]` when the file is missing or unreadable,
    /// `[FM-CONFIG-002]` on TOML syntax errors or unknown keys, and
    /// `[FM-CONFIG-003]` when parsing succeeds but a semantic invariant
    /// fails (empty name, unsafe path) — fail closed at every stage.
    pub fn load(path: &Path) -> Result<Self> {
        let parsed = star_toml::load_file::<Self>(path).map_err(|e| match e {
            star_toml::Error::FileNotFound(p) => AppError::fm_config(
                1,
                format!(
                    "ggen.toml not found at `{}`. Remediation: create the manifest or fix the path.",
                    p.display()
                ),
            ),
            star_toml::Error::Io { .. } => AppError::fm_config(
                1,
                format!("cannot read `{}`: {e}. Remediation: check file permissions.", path.display()),
            ),
            other => AppError::fm_config(
                2,
                format!(
                    "invalid ggen.toml at `{}`: {other}. Remediation: fix the TOML syntax or remove unknown keys.",
                    path.display()
                ),
            ),
        })?;
        parsed.validated().map_err(|errs| {
            AppError::fm_config(
                3,
                format!(
                    "ggen.toml at `{}` failed semantic validation: {errs}. \
                     Remediation: fix the reported field(s).",
                    path.display()
                ),
            )
        })
    }

    /// Parse a `ggen.toml` document from a string (env vars expanded by
    /// `star_toml` before parsing), then run semantic validation.
    ///
    /// # Errors
    /// Returns `[FM-CONFIG-002]` on TOML syntax errors or unknown keys, and
    /// `[FM-CONFIG-003]` when a semantic invariant fails.
    pub fn from_toml_str(toml: &str) -> Result<Self> {
        let parsed = star_toml::from_str::<Self>(toml).map_err(|e| {
            AppError::fm_config(
                2,
                format!("invalid ggen.toml document: {e}. Remediation: fix the TOML syntax or remove unknown keys."),
            )
        })?;
        parsed.validated().map_err(|errs| {
            AppError::fm_config(
                3,
                format!(
                    "ggen.toml failed semantic validation: {errs}. \
                     Remediation: fix the reported field(s)."
                ),
            )
        })
    }
}

impl Validate for Project {
    fn validate(&self, v: &mut Validator) {
        v.check_non_empty("name", &self.name);
    }
}

impl Validate for Ontology {
    fn validate(&self, v: &mut Validator) {
        // Declared "relative to the manifest" in schema/ggen-toml-schema.ttl;
        // `must_be_absolute: Some(false)` matches that contract.
        v.check_path("source", &self.source.to_string_lossy(), Some(false));
    }
}

impl Validate for Templates {
    fn validate(&self, v: &mut Validator) {
        v.check_path("dir", &self.dir.to_string_lossy(), Some(false));
    }
}

impl Validate for PackRef {
    fn validate(&self, v: &mut Validator) {
        match self {
            // Pack paths legitimately reference sibling directories
            // (`../foo-pack`, per tests/cross_pack_matrix.rs and the pack
            // layout convention found across every real consumer pack
            // surveyed for this ticket) — `check_path`'s traversal rejection
            // does not apply here, unlike `ontology.source`/`templates.dir`
            // which must stay within the project. Only non-emptiness is
            // required.
            Self::Path { path } => {
                v.check_non_empty("path", &path.to_string_lossy());
            }
            Self::Git { git, version } => {
                v.check_non_empty("git", git);
                v.check_non_empty("version", version);
            }
        }
    }
}

impl Validate for Law {
    fn validate(&self, v: &mut Validator) {
        for (i, rule) in self.rules.iter().enumerate() {
            v.check_path(&format!("rules[{i}]"), &rule.to_string_lossy(), Some(false));
        }
        for (i, shape) in self.shapes.iter().enumerate() {
            v.check_path(
                &format!("shapes[{i}]"),
                &shape.to_string_lossy(),
                Some(false),
            );
        }
    }
}

impl Validate for GgenConfig {
    fn validate(&self, v: &mut Validator) {
        v.field("project", |v| self.project.validate(v));
        v.field("ontology", |v| self.ontology.validate(v));
        v.field("templates", |v| self.templates.validate(v));
        v.field("law", |v| self.law.validate(v));
        for (name, pack_ref) in &self.packs {
            v.field(&format!("packs.{name}"), |v| pack_ref.validate(v));
        }
    }
}
