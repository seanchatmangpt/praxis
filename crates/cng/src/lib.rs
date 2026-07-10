//! cng library surface: the μ pipeline (import → merge → plan → project →
//! serialize) behind the noun-verb CLI, exposed so integration tests can
//! drive the exact code the binary runs.

#[cfg(feature = "bench")]
pub mod bench;
pub mod pipeline;
pub mod powl;
#[cfg(feature = "runner")]
pub mod runner;
pub mod shape;
