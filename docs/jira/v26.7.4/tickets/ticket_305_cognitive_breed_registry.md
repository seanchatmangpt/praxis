# Ticket: Compile-Checked Cognitive Breed Registry

## Title
Promote `COGNITIVE_BREED_MAPPING.md` prose into a checked const table (PROJ-305)

## Description
v26.7.3's PROJ-206 produced a documentation-only mapping of "cognitive breed" names (Guardian,
Detector, Tracker, Retriever, Planner, Herding, Recorder, Verifier, ...) onto the
praxis-synthesis module that already performs that function. Doc-only mappings drift silently
when modules are renamed or removed. This ticket closes that drift risk with a tiny, purely
descriptive registry — explicitly NOT a new runtime abstraction, trait, or dispatch mechanism:

```rust
/// (breed name, module path) — kept in sync with docs/v26.7.3/COGNITIVE_BREED_MAPPING.md
/// by a compile-time-checked test, not by convention.
pub const BREED_MODULE_MAP: &[(&str, &str)] = &[
    ("guardian", "quarantine"),
    ("detector", "hooks"),
    ("tracker", "firing"),
    ("retriever", "life"),
    ("planner", "ground"),
    ("recorder", "envelope"),
    ("verifier", "firing"),
    // ...
];
```

A test asserts every module path string in this table corresponds to an actual `pub mod`
declared in `lib.rs` (via a simple string search over the crate's own source, or — simpler and
more robust — by attempting `use crate::<module>;` for each in a macro/test harness). Any
breed with genuinely no code home stays undocumented here (matching PROJ-206's "NOT
IMPLEMENTED" markers) rather than being forced into a fake mapping.

## Acceptance Criteria
- `src/breeds.rs` with `BREED_MODULE_MAP`, no new trait, no new runtime behavior.
- A test failing if a listed module path does not exist as a `pub mod` in `lib.rs`.
- The table's content matches `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md` 1:1 (same breeds,
  same module citations) — read that doc via the Read tool first; do not invent new mappings
  here that weren't already vetted in PROJ-206.
- Zero new Cargo dependencies; zero new public traits.

## Dependencies
The v26.7.3 `docs/v26.7.3/COGNITIVE_BREED_MAPPING.md` doc (from PROJ-206) must exist and be
read before writing this table.

## Verification Mechanism
1. `cargo test -p praxis-synthesis --lib breeds::` — new test green.
2. `cargo clippy -p praxis-synthesis --all-targets -- -D warnings` — clean.
3. Manual diff between `BREED_MODULE_MAP` entries and the doc's table confirming 1:1 parity.
