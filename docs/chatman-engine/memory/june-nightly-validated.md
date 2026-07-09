# June Nightly Validated

**Summary**: All 4 constitutional crates pass `cargo check` and `cargo test` green on
nightly-2026-06-22 with zero fixes (verified 2026-07-09).

**Source evidence**: This session's toolchain validation run, 2026-07-09; the ConditionCell
feature obligation stops at the naming crate.

**Why it matters**: Pins a known-good nightly for the constitutional surface, so toolchain
drift can be distinguished from real regressions.

**Future instruction**: Treat nightly-2026-06-22 as the validated baseline. If a later nightly
breaks, bisect against this pin before changing code; ConditionCell feature work does not
propagate past the naming crate.
