//! Shared house crate for the seanchatmangpt Rust fleet.
//!
//! Provides common utilities for error handling, telemetry, CLI helpers,
//! provenance/content-addressing, and test infrastructure.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub use error::{Error, Result};

#[cfg(feature = "telemetry")]
pub mod telemetry;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "provenance")]
pub mod provenance;

pub mod chain;
#[cfg(feature = "provenance")]
pub use chain::RollingChain;

#[cfg(feature = "testkit")]
pub mod testkit;
