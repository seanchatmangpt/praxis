//! Noun-verb command modules.
//!
//! Each file's stem is a CLI noun; each `#[verb]` fn inside is a verb under
//! that noun. Modules must be declared here (and the lib linked from the
//! binary) or their `linkme` registrations are never included in the build.

pub mod example;
pub mod verifier;
