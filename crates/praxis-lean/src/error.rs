//! Typed refusals for the Lean 4 admission gate.
//!
//! Every library-facing function in this crate returns `Result<T, LeanRefusal>`
//! rather than `anyhow::Result` — matching this workspace's convention (see
//! `praxis-synthesis::Refusal`) that every admission outcome is a named,
//! typed variant, never a panic or a silently-collapsed generic error.
//! `anyhow` is used only at the CLI binary boundary (`main.rs`) for
//! top-level error display.

use camino::Utf8PathBuf;
use thiserror::Error;

/// All refusal outcomes from the Lean 4 admission gate.
#[derive(Debug, Error)]
pub enum LeanRefusal {
    #[error("kernel rejected {file}: exit={exit_code:?}\n{stderr_preview}")]
    KernelRejected {
        file: Utf8PathBuf,
        exit_code: Option<i32>,
        stderr_preview: String,
    },

    #[error("{file}:{line}: forbidden `sorry`: {text}")]
    SorryFound {
        file: Utf8PathBuf,
        line: usize,
        text: String,
    },

    #[error("{file}:{line}: unauthorized axiom `{name}`: {text}")]
    UnauthorizedAxiom {
        file: Utf8PathBuf,
        line: usize,
        name: String,
        text: String,
    },

    #[error("statement label `{label}` has an index entry but no receipt line")]
    OrphanLabel { label: String },

    #[error("index entry `{label}` references file `{file}`, which does not exist")]
    OrphanFile { label: String, file: Utf8PathBuf },

    #[error("receipt line for `{label}` has no corresponding index entry")]
    OrphanReceipt { label: String },

    #[error("I/O error at {path}: {source}")]
    Io {
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("JSON error at {path}: {source}")]
    Json {
        path: Utf8PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("non-UTF-8 path: {0:?}")]
    NonUtf8Path(std::path::PathBuf),

    #[error("required external tool not installed: {tool} ({detail})")]
    ToolNotInstalled { tool: String, detail: String },
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, LeanRefusal>;
