use thiserror::Error;

pub type Result<T> = std::result::Result<T, RetrofitError>;

#[derive(Debug, Error)]
pub enum RetrofitError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Invalid Cargo.toml: {0}")]
    InvalidCargoToml(String),

    #[error("Compliance check failed: {0}")]
    ComplianceFailed(String),

    #[error("Retrofit failed: {0}")]
    RetrofitFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    /// Refused per `.claude/rules/autonomous-escalation-policy.md`'s
    /// "genuinely underdetermined product law" class: `repos.toml` and
    /// `.chatmangpt/ecosystem.lock.toml` both name the same repository and
    /// this crate does not know a lawful merge/precedence rule between a
    /// hand-maintained fleet survey entry and a pinned Cargo-path lock
    /// entry. Refuse rather than silently overwrite either source.
    #[error("Repository name collision between repos.toml and ecosystem.lock.toml: {0}")]
    EcosystemNameCollision(String),
}
