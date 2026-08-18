//! ALIVE gate: compile-fail + compile-pass fixture tests.
//!
//! These tests verify that the type system *accepts* the patterns it should
//! (`pass/`) and *rejects* the patterns it should (`fail/`).  The gate is
//! named "ALIVE" after the `wasm4pm-compat` convention: if the compiler stops
//! rejecting the right things (e.g., a refactoring accidentally makes a private
//! field public), this test suite turns red immediately.
//!
//! Run with:
//! ```bash
//! cargo test --test compile_tests
//! ```
//!
//! # Adding fixtures
//!
//! **Pass fixture:** create `tests/compile/pass/<name>.rs`.  It must compile
//! with no errors.  Add a new `t.pass(...)` line below.
//!
//! **Fail fixture:** create `tests/compile/fail/<name>.rs`.  The expected
//! compiler error must match the snapshot in `tests/compile/fail/<name>.stderr`.
//! To regenerate all `.stderr` snapshots after a Rust version upgrade run:
//!
//! ```bash
//! TRYBUILD=overwrite cargo test --test compile_tests
//! ```
//!
//! Add a new `t.compile_fail(...)` line below for each new fixture.
//!
//! # How trybuild works
//!
//! `trybuild` compiles each fixture as a standalone binary using the same
//! toolchain and dependencies as the current project.  For compile-fail tests
//! it captures `rustc`'s stderr output and diffs it against the `.stderr`
//! snapshot.  Mismatches (wrong error, missing error, unexpected compile
//! success) cause the test to fail.

#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();

    // ── compile-pass fixtures ─────────────────────────────────────────────
    // Each file should compile successfully.

    // Basic seal pattern: constructing a sealed type via its canonical builder.
    t.pass("tests/compile/pass/seal_via_builder.rs");

    // Typestate transition: Pending → Verified is legal.
    t.pass("tests/compile/pass/typestate_transition.rs");

    // ReceiptRefusal can be compared with assert_eq! (PartialEq).
    t.pass("tests/compile/pass/refusal_eq.rs");

    // assert_unique_ids at const time with a valid set.
    t.pass("tests/compile/pass/unique_ids_ok.rs");

    // ── compile-fail fixtures ─────────────────────────────────────────────
    // Each file must produce a compiler error.  The expected diagnostic is
    // stored in the matching `.stderr` file next to the `.rs` file.

    // Attempting to construct a sealed struct with a struct literal (E0451).
    t.compile_fail("tests/compile/fail/seal_forgery.rs");

    // Attempting to use State<Verified> where State<Pending> is expected (E0308).
    t.compile_fail("tests/compile/fail/typestate_wrong_state.rs");

    // Implementing the `Sealed` trait outside the crate (E0277 — cannot satisfy
    // private supertrait bound).
    t.compile_fail("tests/compile/fail/sealed_impl_forgery.rs");
}
