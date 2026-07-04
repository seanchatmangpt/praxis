//! `ggen.toml` configuration model, loaded via `star_toml`.
//!
//! `star_toml` is a full loader/validator for `*.toml` files (parse, layer,
//! env-expand, validate). Here we use its plain loading surface
//! ([`star_toml::load_file`] / [`star_toml::from_str`]) with serde
//! `deny_unknown_fields` on every table, so any unknown key is a hard error
//! (fail closed).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// Root model of a `ggen.toml` manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
}

/// `[project]` — identity of the generating project.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// Project name.
    pub name: String,
}

/// `[ontology]` — the RDF source of truth and its namespace prefixes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ontology {
    /// Path to the ontology file (Turtle), relative to the manifest.
    pub source: PathBuf,
    /// Prefix → namespace IRI map.
    #[serde(default)]
    pub prefixes: BTreeMap<String, String>,
}

/// A pack source reference: either a local path or a git coordinate.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Templates {
    /// Template directory, relative to the manifest.
    pub dir: PathBuf,
}

impl GgenConfig {
    /// Load and parse a `ggen.toml` file.
    ///
    /// # Errors
    /// Returns `[FM-CONFIG-001]` when the file is missing or unreadable and
    /// `[FM-CONFIG-002]` on TOML syntax errors or unknown keys (fail closed).
    pub fn load(path: &Path) -> Result<Self> {
        star_toml::load_file::<Self>(path).map_err(|e| match e {
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
        })
    }

    /// Parse a `ggen.toml` document from a string (env vars expanded by
    /// `star_toml` before parsing).
    ///
    /// # Errors
    /// Returns `[FM-CONFIG-002]` on TOML syntax errors or unknown keys.
    pub fn from_toml_str(toml: &str) -> Result<Self> {
        star_toml::from_str::<Self>(toml).map_err(|e| {
            AppError::fm_config(
                2,
                format!("invalid ggen.toml document: {e}. Remediation: fix the TOML syntax or remove unknown keys."),
            )
        })
    }
}
