//! `agent8` — an 8-bit agent projection plus the 64-byte wire ABI it rides on.
//!
//! Three layers, smallest to largest:
//!
//! 1. [`AgentByte`] — a `#[repr(transparent)]` `u8` newtype projecting one
//!    agent's governance posture into eight named bits, with a documented
//!    [`AgentByte::GRANT_REQUIRED`] mask and const `with`/`carries`/`select`.
//!    Hand-authored sibling of the *generated* `Status8Field` in `semantic_bit`
//!    (prior art cited in [`byte`]).
//! 2. [`Env64`] + [`Pulse64`] — `#[repr(C, align(64))]` ports of the bytestar
//!    ABI (`env64_t` / `pulse64_t`), each exactly 64 bytes (compile-time
//!    asserted, `OcelCausalFrame` idiom), plus
//!    [`pulse64_from_receipt_record`] bridging a praxis
//!    [`praxis_core::ReceiptRecord`] onto the wire. The [`Env64`] pattern byte
//!    is the [`AgentByte`] wire slot.
//! 3. [`Fleet`] — a SWAR kernel packing 8 agents per word, with
//!    [`sweep_admit`] (ported `unibit` `admit`/denial-polarity primitive) and
//!    popcount [`FleetStats`].
//!
//! # Denial polarity
//!
//! Throughout the fleet layer, **zero means admitted, nonzero means denied**,
//! carried over verbatim from `unibit-kernel`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod abi;
pub mod byte;
pub mod fleet;

pub use abi::{
    pulse64_from_receipt_record, Env64, Pulse64, ABI_VERSION, ENV_MAGIC, MAX_PRIORITY, MAX_STEP,
    PULSE_FLAG_ERROR, PULSE_FLAG_FINAL, PULSE_FLAG_VALID, PULSE_MAGIC,
};
pub use byte::{AgentByte, AgentSelect};
pub use fleet::{sweep_admit, Fleet, FleetStats, LANES_PER_WORD};
