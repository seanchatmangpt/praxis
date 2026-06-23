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
}
