#![doc = "Fused Law Object abstraction: obligation + lifecycle + receipt + OCEL."]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod default_law;
pub mod error;
pub mod law;
pub mod lifecycle;
pub mod quarantine;
pub mod verify;

#[cfg(feature = "ocel")]
pub mod ocel;

#[cfg(feature = "signed")]
pub mod signing;

pub use default_law::DefaultLaw;
pub use law::{Admit, Andon, Judge, LawObject, Obligation};
pub use quarantine::{BoundarySchema, JsonBoundarySchema, QuarantineError, RiceQuarantine};
