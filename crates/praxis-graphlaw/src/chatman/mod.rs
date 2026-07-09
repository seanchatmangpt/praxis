//! Chatman engine: invocation ABI, Triple8 universe, admission tables, and routing.
//!
//! Module layout (one lane owns each file; `abi` is the cross-lane contract):
//! - [`abi`] — invocation envelopes, receipts, and the refusal taxonomy.
//! - [`triple8`] — bounded Triple8 term universe.
//! - [`admission8`] — admission tables for hook patterns and OCEL events.
//! - [`router`] — least-expressive dialect routing.
//! - [`engine`] — the engine loop over admitted invocations.
//! - [`bridge`] — boundary bridge to external process substrates.

pub mod abi;
pub mod admission8;
pub mod bridge;
pub mod engine;
pub mod router;
pub mod triple8;

pub use abi::Refusal;
