//! Verb subcommands for the praxis CLI.
//!
//! Each module here defines one or more `#[verb]` functions that are auto-registered
//! by the `clap_noun_verb` crate via the `linkme::distributed_slice` mechanism.

pub mod config;
pub mod doctor;
pub mod dod;
pub mod example;
pub mod frontier;
pub mod law;
#[cfg(feature = "ggen")]
pub mod mfg;
#[cfg(feature = "proposer")]
pub mod mission;
pub mod plan;
#[cfg(feature = "proposer")]
pub mod propose;
pub mod receipt;
pub mod synth;
#[cfg(feature = "testbed")]
pub mod testbed;
pub mod verifier;
