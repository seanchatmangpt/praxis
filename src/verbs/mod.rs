//! Verb subcommands for the praxis CLI.
//!
//! Each module here defines one or more `#[verb]` functions that are auto-registered
//! by the `clap_noun_verb` crate via the `linkme::distributed_slice` mechanism.

pub mod config;
pub mod example;
pub mod law;
#[cfg(feature = "ggen")]
pub mod mfg;
pub mod plan;
#[cfg(feature = "proposer")]
pub mod propose;
pub mod receipt;
#[cfg(feature = "testbed")]
pub mod testbed;
pub mod verifier;
