//! {{project-name}} — WASM library.
//!
//! Entry points are exposed via `#[wasm_bindgen]`. All Rust-side objects are
//! stored in a [`Store<T>`]; callers receive opaque string handles, never raw
//! pointers — this keeps the boundary `unsafe`-free and enables revocation.

use wasm_bindgen::prelude::*;

mod store;
pub use store::Store;

/// Initialise the WASM module. Call once from JavaScript before any other API.
///
/// Sets up `console_error_panic_hook` so Rust panics are readable in the
/// browser console rather than being swallowed silently.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Compute the BLAKE3 hex digest of `input`.
///
/// # Example (JavaScript)
/// ```js
/// import init, { blake3_hex } from './pkg/{{project-name}}.js';
/// await init();
/// console.log(blake3_hex("hello")); // deterministic 64-char hex
/// ```
#[wasm_bindgen]
pub fn blake3_hex(input: &str) -> String {
    blake3::hash(input.as_bytes()).to_hex().to_string()
}
