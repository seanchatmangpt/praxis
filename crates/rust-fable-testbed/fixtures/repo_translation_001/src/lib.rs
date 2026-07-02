//! Fixture for `repo_translation_001`: a small multi-file "mini-repo" exercising
//! repo-level (cross-module) translation rather than a single-function bugfix.
//!
//! `Shape` (this file) recently gained a new `label: String` field, shared across two
//! sibling modules:
//!
//! - [`area`] (`src/area.rs`) — already updated: its `describe_area` function was
//!   written *with* the new field in mind and compiles/passes today.
//! - [`describe`] (`src/describe.rs`) — **not yet updated**: `describe_shape` still
//!   builds its summary string as if `Shape` only had `width`/`height`, silently
//!   dropping `label` from the output. It compiles (the field exists and is public,
//!   it's just unused in that one function) but fails the
//!   `describe_shape_includes_the_label` test in `describe.rs`.
//!
//! This mirrors the real-world "add a field to a shared struct, then every module that
//! constructs/uses it must be updated" shape called out in
//! `RUST_CLAUDE_COMPREHENSIVE_RESEARCH.md`'s repo-level-translation section, while
//! keeping the mini-repo small (3 files, 2 modules) and, per the v1 scope cut in
//! `src/sandbox.rs` (`apply_model_output` replaces exactly one target file from one
//! fenced code block), solvable by fixing a single file: `describe.rs`. `lib.rs` and
//! `area.rs` are both already correct and are given to the model as read-only context.

pub mod area;
pub mod describe;

/// A labeled rectangle.
///
/// `label` was added after `area.rs` and `describe.rs` were first written; `area.rs`
/// was updated to match but `describe.rs` (the task's target file) was not.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Width in arbitrary units.
    pub width: f64,
    /// Height in arbitrary units.
    pub height: f64,
    /// Human-readable label, e.g. `"tile"` or `"panel"`.
    pub label: String,
}

impl Shape {
    /// Construct a new labeled `Shape`.
    #[must_use]
    pub fn new(width: f64, height: f64, label: impl Into<String>) -> Self {
        Self { width, height, label: label.into() }
    }
}
