//! Praxis Retrofit: Automate standardization across Rust ecosystem
//!
//! Retrofits existing repositories with praxis house-style standards:
//! - Workspace [lints] configuration
//! - Dependency unification via [workspace.dependencies]
//! - Justfile standardization
//! - typos.toml spell-check configuration
//! - Compliance validation gates
//!
//! # Usage
//!
//! ```text
//! praxis-retrofit audit scan <repo-path>       # Audit repo compliance
//! praxis-retrofit audit report <repo-path>     # Generate compliance report
//! praxis-retrofit apply retrofit <repo-path>   # Apply retrofit changes
//! praxis-retrofit apply validate <repo-path>   # Validate retrofit success
//! praxis-retrofit generate templates            # Generate template files
//! praxis-retrofit validate compliance <repo-path> # CI compliance gate
//! ```

pub mod audit;
pub mod apply;
pub mod generate;
pub mod validate;
pub mod templates;
pub mod error;
pub mod models;

pub use error::{RetrofitError, Result};
pub use models::{ComplianceReport, RetrofitPlan, RepositoryMetadata};

/// Current praxis retrofit version (CalVer)
pub const VERSION: &str = "26.6.0";

/// Praxis standards specification
#[derive(Debug, Clone)]
pub struct PraxisSpec {
    pub edition: String,
    pub msrv: String,
    pub toolchain: String,
    pub lints_strict: bool,
    pub license_preferred: String,
    pub linters: Vec<String>,
}

impl Default for PraxisSpec {
    fn default() -> Self {
        Self {
            edition: "2021".to_string(),
            msrv: "1.82".to_string(),
            toolchain: "nightly-2026-04-15".to_string(),
            lints_strict: true,
            license_preferred: "MIT OR Apache-2.0".to_string(),
            linters: vec![
                "unsafe_code=forbid".to_string(),
                "clippy/all=warn".to_string(),
                "clippy/pedantic=warn".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_spec() {
        let spec = PraxisSpec::default();
        assert_eq!(spec.edition, "2021");
        assert_eq!(spec.msrv, "1.82");
        assert!(spec.lints_strict);
    }
}
