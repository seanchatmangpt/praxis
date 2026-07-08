//! WASM bindings for Praxis GraphLaw law-state engine.
//!
//! This crate provides WebAssembly bindings to the praxis-graphlaw reasoning engine,
//! enabling semantic reasoning (N3, Datalog, SPARQL, SHACL, ShEx) in browser and
//! server-side JavaScript environments.

use wasm_bindgen::prelude::*;

pub mod core;
pub mod dto;

/// Initialize panic hook for better error messages in WASM environments.
///
/// Call this at startup to enable browser console panic traces.
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Simple version information endpoint.
///
/// Returns the GraphLaw engine version string.
#[wasm_bindgen]
pub fn graphlaw_version() -> String {
    format!("praxis-graphlaw v{}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphlaw_version() {
        let version = graphlaw_version();
        assert!(version.starts_with("praxis-graphlaw v"));
    }
}
