//! Error types for chatman-common.

/// The central error type for this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A plain message error.
    #[error("{0}")]
    Message(String),

    /// An I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// A JSON serialization/deserialization error.
    #[cfg(feature = "serde")]
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl Error {
    /// Create an error from any string-like value.
    pub fn msg(m: impl Into<String>) -> Self {
        Self::Message(m.into())
    }
}

/// A specialized `Result` type for this crate.
pub type Result<T, E = Error> = core::result::Result<T, E>;
