# Rust Anti-Pattern Rules — praxis-graphlaw

Project-scoped rules derived from Knowledge Hooks implementation patterns observed across five repositories this session (unrdf, ofmf, dteam, knhk, praxis). Each rule cites its source and the invariant it upholds.

## Rules

1. **No `.unwrap()`/`.expect()` on external input parsing**
   - No `.unwrap()` or `.expect()` on `split()`, `parse()`, or other fallible operations on external/untrusted input (Turtle files, Datalog rule literals, hook pack contents).
   - Every such site must return/propagate a typed `Refusal` (invariant #1: no panics, every error is typed).
   - **Source**: `crates/praxis-graphlaw/src/reasoner/mod.rs:658,659,667,677` — `translate_datalog_program` uses `.split('(').nth(1).unwrap()` four times, panicking on malformed Datalog rule literals.
   - **Invariant**: #1 (no panics).

2. **No `.unwrap_or_default()` swallowing Result**
   - Never use `.unwrap_or_default()` or similar silent-default-substitution on a `Result` from a fallible operation (`materialize()`, hook pack loading, graph parsing).
   - Must propagate the error with `?` or surface via `Refusal`/logging path — never silently substitute an empty/default value.
   - **Source**: `crates/praxis-graphlaw/src/csprite.rs:148,157` and `dred.rs:157,166` — `.materialize(...).unwrap_or_default()` swallows refusal from the Reasoner.
   - **Invariant**: #1 (no silent defaults).

3. **No duplicate preprocessing/parsing implementations**
   - One canonical implementation per preprocessing or parsing function; others delegate.
   - **Source**: `crates/praxis-graphlaw/src/lib.rs:312-337` and `parser.rs:18-43` — two divergent `preprocess_turtle` functions with different `//`-stripping semantics and URI exemption logic. lib.rs handles inline `//` + URI scheme exemption; parser.rs only strips whole-line `//`.
   - **Invariant**: #6 (smallest diff, reuse first).

4. **No debug `println!` in shipped code paths**
   - No `println!`/`eprintln!` left in validation, parsing, or hot-path code.
   - If diagnostics are needed, use the crate's existing structured logging or diagnostic paths, not ad-hoc prints.
   - **Source**: `crates/praxis-graphlaw/src/shacl.rs:2634` — `println!("SHACL DEBUG: ...")` in `validate_shape_closed_and_targets_tail`.
   - **Invariant**: #1 (no unfinished instrumentation shipped).

5. **No non-cryptographic hashing in receipts**
   - No `DefaultHasher`, ad-hoc byte-layout chains, or custom canonicalization in any receipt, audit, or provenance path.
   - BLAKE3 only, canonical N-Quads serialization, per invariant #2.
   - **Source**: knhk `rust/genesis-etl/src/hash.rs` — uses `std::collections::hash_map::DefaultHasher` with a comment claiming "production-ready" despite zero cryptographic strength.
   - **Invariant**: #2 (BLAKE3-only receipts, computed not asserted).

6. **No status claims without verification**
   - Do not treat any status/milestone doc (PROJECT.md, `.agents/*/progress.md`, or similar) as authoritative truth.
   - Always verify against `git log`, passing tests, and actual code before treating a "done" claim as real.
   - **Source**: Recurring pattern across praxis `PROJECT.md:18-25` (all milestones marked `PLANNED` despite M1-M5 code existing), wasm4pm, knhk's optimization track.
   - **Invariant**: #6 (know your actual state).
   - **See**: `.claude/rules/no-overclaiming.md` for the required status vocabulary and
     forbidden-phrase list (applies to Rust and JS/TS alike).

7. **No stub functions returning success-shaped output**
   - Never return a success-shaped result (empty vec, `count = 0`, default object) when a dependency is missing or input is unsupported.
   - Must return a typed `Refusal` to make failure visible to the caller.
   - **Source**: knhk `c/src/rdf.c::knhk_rdf_load()` — returns `count = 0` with a warning when raptor2 is unavailable, masking the missing dependency from callers.
   - **Invariant**: #1 (no silent defaults).
   - **See**: `.claude/rules/no-overclaiming.md` for the repo-wide (Rust + JS/TS) no-silent-stub
     rule this generalizes to.
