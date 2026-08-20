## Summary

This PR closes the ecosystem-membership-tracking gap in `praxis-retrofit` by adding a real,
non-mocked OCEL 2.0 event log (`crates/praxis-retrofit/src/ocel_log.rs`) and a directly-follows
process-discovery module (`crates/praxis-retrofit/src/process_discovery.rs`), then wiring
emission into the existing `repo_registry.rs`, `fleet_audit.rs`, `fleet_validate.rs`, and
`fleet_apply.rs` verbs, plus a new `ecosystem-conformance` CLI verb
(`crates/praxis-retrofit/src/bin/praxis-retrofit.rs`). It replaces a previously hand-designed,
never-actually-unified merge of three disconnected ecosystem-membership subsystems with a
Van der Aalst-style process-mining approach: observe real lifecycle events, mine the
directly-follows graph, check conformance against a reference lifecycle — rather than hand-coding
a fourth merge policy on top of the three that already disagreed.

Scope of what is claimed ALIVE in this PR is narrower than "ecosystem conformance checking
works end-to-end" — see the ALIVE/PARTIAL/BLOCKED section below. The CLI-level, fully
brokered, live-repo demonstration is explicitly **BLOCKED** in this environment (no live
checkouts of the ecosystem-lock member repos exist on this disk); the underlying mining and
admission logic is ALIVE and proven via passing unit tests against real fixtures (the actual
`.chatmangpt/ecosystem.lock.toml` on disk, a real singleton OCEL log, real directly-follows
arcs mined from a real event sequence).

## Motivation / Problem Statement

### The ggen-legacy dead-end

An earlier attempt to unify ecosystem-membership tracking routed through ggen-generated
registry projections. That path dead-ended: ggen's generated registry surface has no admitted
notion of repo lifecycle (checkout → audit → validate → apply), only static frontmatter/schema
projection from `pack.toml`/`ontology.ttl`. Attempting to bolt lifecycle tracking onto a
generated projection would have meant hand-editing generated output (a documented anti-pattern
per this repo's ggen-pack doctrine: generated surfaces are extended through admitted sources
and generators, never hand edits) or minting a second, parallel, ungenerated tracking layer that
would silently diverge from the generated one on every regeneration. Neither option was taken
forward.

### Three disconnected ecosystem-membership subsystems found

Investigation (spike, prior to this PR) found three separate, non-communicating notions of
"is this repo part of the ecosystem, and in what state":

1. **`repo_registry.rs`** (pre-existing) — a static parse of `.chatmangpt/ecosystem.lock.toml`
   giving name/URL/SHA/standing per repo, with no lifecycle or event history at all; a repo's
   `standing` field was a hand-set string, not derived from any observed sequence of events.
2. **`fleet_audit.rs` / `fleet_validate.rs` / `fleet_apply.rs`** (pre-existing) — each ran an
   independent, in-process audit/validate/apply pass against a live checkout, with no shared
   event vocabulary between the three verbs and no persisted history connecting an audit run to
   a later validate or apply run on the same repo.
3. **CLI `audit scan`** (`src/bin/praxis-retrofit.rs`, pre-existing) — a standalone scoring path
   producing an ad hoc audit report, again disconnected from both (1) and (2).

These three subsystems each had their own idea of repo state, none of them fed each other, and
none of them left a durable, replayable, receipted trace. A fourth hand-written merge policy
(e.g. "if registry says X and audit says Y, resolve to Z") was considered and rejected: it would
have been an arbitrary tie-break table with no falsifier, violating this repo's semantic-runtime
discipline (no ad hoc reconciliation without an admitted, typed process).

### Why process mining (Van der Aalst) over a hand-designed merge

Instead of hand-coding reconciliation rules across the three subsystems, this PR treats every
lifecycle transition (registry load, audit run, validation, apply) as an **event** emitted to a
shared OCEL 2.0 log, and derives ecosystem state by **mining the directly-follows graph** over
those events (Van der Aalst's α-algorithm family: DFG construction, conformance-by-arc-presence
against a reference model) rather than asserting state transitions by fiat. This gives:

- a single, closed, typed event vocabulary instead of three ad hoc ones;
- a receipted, replayable trace (the OCEL log itself) instead of transient console output;
- a conformance check that is a real graph computation (missing/present arcs) instead of a
  hand-tuned if/else chain.

This is consistent with `.claude/rules/cognition-contracts.md` §4 (process-mining algorithm
closure: registry/dispatcher reachability, positive/negative fixtures, no fabricated model) and
was chosen specifically because it gives a falsifier (an arc either is or isn't in the mined DFG)
that a hand-written merge policy would not.

## Design

### Step 0 — Baseline: identify the three disconnected subsystems

No code change; this is the spike finding restated above. Confirmed by reading
`crates/praxis-retrofit/src/repo_registry.rs` (pre-existing static parse),
`crates/praxis-retrofit/src/fleet_audit.rs`, `fleet_validate.rs`, `fleet_apply.rs`
(pre-existing independent verbs), and `crates/praxis-retrofit/src/bin/praxis-retrofit.rs`
(pre-existing `audit scan` path) to establish that none of the four shared an event vocabulary.

### Step 1 — Evaluate `chicago-tdd-tools`'s `ocel-generation` feature as a reuse candidate

Spiked (see Context above) against `chicago-tdd-tools-26.6.30`'s `ocel-generation` feature
(`src/observability/ocel/{mod,types,collector,wasm4pm,projections,discovery}.rs`,
`Cargo.toml:79,86,413`). Found it real and non-inert, but hard-scoped to the
**test-execution domain** — `TestActivity`/`TestObjectType` (types.rs:8-62) is a closed
vocabulary for assertions, fixtures, wave-orchestration phases, and run summaries, not for
praxis's repo/process/workflow domain. Decision (documented in the spike recommendation above,
carried into this PR): do **not** add `chicago-tdd-tools`'s `ocel-generation` as a runtime
dependency of the new module. Borrow only the *admission pattern* conceptually — typed refusals
(`MissingCaseId`, `NonMonotonicTimestamp`, `DanglingObjectReference`, wasm4pm.rs:9-49) sealed via
`Evidence<T, Admitted, Witness>` — and build directly on `wasm4pm_compat::ocel::OCEL` types
instead of chicago-tdd-tools' `TestOcelEvent`/`OcelLog`.

### Step 2 — New module: `ocel_log.rs`

Created `crates/praxis-retrofit/src/ocel_log.rs` (new file). Implements:

- A process-domain (not test-domain) OCEL 2.0 event log using `wasm4pm_compat::ocel::OCEL`
  wire types directly — avoiding the Step 1 dead-end of reusing a closed test-vocabulary enum.
- `enabled()` / `log_path()` — reflect the `PRAXIS_RETROFIT_OCEL_LOG` environment variable to
  determine whether emission is active and where the log is written.
- `emit()` — pushes an event with object relationships (`ocel:typing`/qualifier structure),
  registering the event type in the log's global declarations if not already present.
- `ensure_object()` — appends a new object and registers its object type; idempotent on repeat
  IDs (verified by `ensure_object_is_idempotent_on_repeat_id`, see Testing section).
- `global()` — singleton accessor (`OnceLock`/equivalent) so all call sites across
  `repo_registry.rs`/`fleet_*.rs` share one in-process log instance per run.
- `write_json()` — serializes to the real OCEL 2.0 wire format and round-trips through
  `wasm4pm_compat`'s own OCEL type, confirmed by `write_json_round_trips_through_real_ocel_wire_type`.

Unit tests: `crates/praxis-retrofit/src/ocel_log.rs` `#[cfg(test)] mod tests` — 6 tests, all
passing (enumerated in Testing section).

### Step 3 — New module: `process_discovery.rs`

Created `crates/praxis-retrofit/src/process_discovery.rs` (new file). Implements:

- `discover_lifecycle()` — mines the directly-follows graph (DFG) from a sequence of observed
  lifecycle events, in the Van der Aalst α-algorithm tradition: for each consecutive pair of
  events sharing a case/object identity, record a directly-follows arc.
- `conformance_report()` — compares a mined or reference DFG against another DFG, reporting
  present and missing arcs (`hits` / `misses`), rather than a Boolean pass/fail — this
  satisfies `cognition-contracts.md`'s requirement that a formal-conformance claim carry a real
  witness, not a bare `true`.
- A deliberately non-conformant reference fixture
  (`full_lifecycle_reference_is_non_conformant_by_design`) exists specifically to prove the
  conformance checker can detect and report a missing arc, not just confirm happy-path presence
  — this is the negative/adversarial fixture required by `cognition-contracts.md` §4.

Unit tests: 3 tests, all passing (enumerated in Testing section).

### Step 4 — Wire emission into `repo_registry.rs`

Modified `crates/praxis-retrofit/src/repo_registry.rs` (+363 lines in this diff). Added:

- Emission of registry-load lifecycle events (repo discovered, ecosystem-lock parsed) via
  `ocel_log::emit()`/`ensure_object()`.
- New test coverage for ecosystem-lock loading edge cases: `test_load_ecosystem_lock_real_file`
  (loads the actual `.chatmangpt/ecosystem.lock.toml` on disk — not a synthetic fixture),
  `test_load_with_ecosystem_name_collision_refused`, `test_load_with_ecosystem_union_non_colliding`,
  `test_load_with_env_var_override`, `test_load_with_parent_directory_search` — 8 tests total in
  this module, all passing.

### Step 5 — Wire emission into `fleet_audit.rs` / `fleet_validate.rs` / `fleet_apply.rs`

Modified `crates/praxis-retrofit/src/fleet_audit.rs` (+44 lines), `fleet_validate.rs`
(+22 lines), `fleet_apply.rs` (+28 lines). Each verb now emits lifecycle events for its own
phase (audit-started/audit-completed, validate-started/validate-completed,
apply-started/apply-completed) through the shared `ocel_log::global()` singleton, so a single
run across all three verbs against the same repo produces one continuous, ordered event trace
instead of three disconnected console outputs.

### Step 6 — New CLI verb: `ecosystem-conformance`

Modified `crates/praxis-retrofit/src/bin/praxis-retrofit.rs` (+162 lines). Added the
`ecosystem-conformance --log <path> --reference <name>` verb: loads an OCEL log from disk,
mines its lifecycle DFG via `process_discovery::discover_lifecycle()`, and reports conformance
against a named reference lifecycle model via `process_discovery::conformance_report()`.

### Step 7 — Supporting plumbing

Modified `crates/praxis-retrofit/src/error.rs` (+9 lines) — new typed error variants for OCEL
log I/O and reference-lifecycle lookup failures (no silent defaults; consistent with the
repo's typed-`Refusal` discipline). Modified `crates/praxis-retrofit/src/lib.rs` (+2 lines) —
`pub mod ocel_log;` and `pub mod process_discovery;` declarations. Modified
`crates/praxis-retrofit/Cargo.toml` (+1 line) and `Cargo.lock` (+1 line) — added the
`wasm4pm-compat` dependency used directly by `ocel_log.rs` (per the Step 1 decision to build on
its OCEL types rather than chicago-tdd-tools').

## What is ALIVE vs PARTIAL vs deliberately deferred/UNVERIFIED

**ALIVE** (observed, this session, real execution, no mocks):

- `ocel_log.rs` — all 6 unit tests pass against real in-memory `OnceLock` singleton state and a
  real round-trip through `wasm4pm_compat`'s actual OCEL wire type (not a hand-asserted JSON
  shape).
- `process_discovery.rs` — all 3 unit tests pass, including a real DFG mined from a real event
  sequence and a real, deliberately-non-conformant reference fixture that the conformance
  checker correctly flags as non-conformant.
- `repo_registry.rs` — all 8 tests pass, including a real parse of the actual
  `/Users/sac/praxis/.chatmangpt/ecosystem.lock.toml` file on disk (not a synthetic copy).
- Compilation of `praxis-retrofit` itself — confirmed clean (nextest had to fully compile the
  crate to run the 80+2+3 tests below; zero compile errors attributable to this crate).
- Existing test suite regression-free: 80 unit tests + 2 property tests
  (`test_ocel_tracing_integration`, `test_parsing_resilience`) + 3 doc-tests, all passing,
  0 failures.

**PARTIAL**:

- Workspace-wide `just check` (`cargo check --all-features`) fails, but on three crates
  unrelated to this PR: `cng` (unresolved `bcinr_pddl::ground::IndexedGroundProblem`),
  `praxis-synthesis` (missing `pddl_index` crate), and `affidavit` (a mismatched-types error at
  `/Users/sac/affidavit/src/model_mining.rs:175`, an external path dependency outside this
  repo). None of these errors reference `praxis-retrofit`. No scoped `check-pkg` recipe exists
  in the justfile to isolate a single crate's `cargo check`, so this PR's own compile
  correctness is evidenced indirectly (via the full nextest compile in the scoped test run)
  rather than via a directly isolated `cargo check`.

**BLOCKED** (named exactly, per the verify stage):

- **End-to-end CLI demonstration of `ecosystem-conformance` against a live, OCEL-log-backed
  run is BLOCKED.** Exact reason: `grep -rln "ocel_log::" crates/praxis-retrofit/src/` shows
  emission wired only into `repo_registry.rs`, `fleet_audit.rs`, `fleet_validate.rs`,
  `fleet_apply.rs` — the CLI's pre-existing `audit scan` verb
  (`src/bin/praxis-retrofit.rs`) is **not** wired to `ocel_log`, so running
  `PRAXIS_RETROFIT_OCEL_LOG=/tmp/retrofit-verify.ocel.json ./target/debug/praxis-retrofit audit
  scan /Users/sac/praxis/crates/praxis-retrofit` produces an audit report but writes no OCEL
  log file at all (`ls /tmp/retrofit-verify.ocel.json` → "No such file or directory"). The
  verbs that *do* emit OCEL events (`fleet_audit`/`fleet_apply`/`fleet_validate`/
  `repo_registry`) require live, on-disk checkouts of the ecosystem-lock member repos (`bcinr`,
  `wasm4pm`, `ggen`, `wasm4pm-compat`, `lsp-max`, `chicago-tdd-tools`), which are not present in
  this environment — `.chatmangpt/ecosystem.lock.toml` holds only remote URL/SHA references with
  `standing = "UNKNOWN"`, not local clones. Attempting `ecosystem-conformance --log
  /tmp/retrofit-verify.ocel.json --reference admission` against a log that was never written
  correctly fails with `Failed to load OCEL log ... No such file or directory`, which is the
  expected typed-refusal behavior given no upstream event producer ran, not a bug in the new
  verb itself.
- Correctness of the underlying mining/admission logic that this CLI path would exercise is
  instead demonstrated via the passing unit tests in the ALIVE section (real DFG mining, real
  OCEL round-trip, real ecosystem-lock parse) — but the full CLI-to-disk-to-CLI loop itself
  remains UNVERIFIED in this environment pending live repo checkouts.

**UNSUPPORTED / not attempted**: no attempt was made to wire OCEL emission into the pre-existing
`audit scan` verb in this PR — see Follow-up tickets below.

## Files changed

**New:**
- `crates/praxis-retrofit/src/ocel_log.rs` — singleton OCEL 2.0 event/object log built on
  `wasm4pm_compat::ocel::OCEL`, env-var-gated, with idempotent object registration.
- `crates/praxis-retrofit/src/process_discovery.rs` — directly-follows-graph mining and
  conformance reporting (present/missing arcs) over lifecycle event sequences.

**Modified:**
- `Cargo.lock` — lockfile update for the new `wasm4pm-compat` dependency edge.
- `crates/praxis-retrofit/Cargo.toml` — add `wasm4pm-compat` dependency.
- `crates/praxis-retrofit/src/bin/praxis-retrofit.rs` — add `ecosystem-conformance` CLI verb
  (load log, mine DFG, report conformance against a named reference).
- `crates/praxis-retrofit/src/error.rs` — new typed error variants for OCEL I/O and
  reference-lifecycle lookup failures.
- `crates/praxis-retrofit/src/fleet_apply.rs` — emit apply-phase lifecycle events.
- `crates/praxis-retrofit/src/fleet_audit.rs` — emit audit-phase lifecycle events.
- `crates/praxis-retrofit/src/fleet_validate.rs` — emit validate-phase lifecycle events.
- `crates/praxis-retrofit/src/lib.rs` — declare the two new modules.
- `crates/praxis-retrofit/src/repo_registry.rs` — emit registry-load events; add 5 new tests
  for ecosystem-lock loading edge cases (real file, collision-refused, union-non-colliding,
  env-var override, parent-directory search).

## Testing performed

Exact commands and exact output, as run by the verify stage this session:

```
$ timeout 300 just check 2>&1 | tail -100
error: could not compile `cng` (lib) due to 5 previous errors ...
error: could not compile `praxis-synthesis` (lib) due to 2 previous errors ...
error: could not compile `affidavit` (lib) due to 1 previous error ...
error: recipe `check` failed on line 257 with exit code 101
```
→ PARTIAL, all three failures pre-existing/unrelated to praxis-retrofit (confirmed by errors
naming only `cng`, `praxis-synthesis`, `affidavit`).

```
$ timeout 600 just test-pkg praxis-retrofit 2>&1 | tail -200
running 80 tests
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running unittests src/bin/praxis-retrofit.rs — 0 tests, ok
Running tests/property_tests.rs — 2 passed (test_ocel_tracing_integration, test_parsing_resilience)
Doc-tests praxis_retrofit — 3 passed, 2 ignored (compile-only doctests)
```
→ PASS.

New-suite breakdown from the same run:
- `ocel_log::tests` — 6/6 pass.
- `process_discovery::tests` — 3/3 pass.
- `repo_registry::tests` — 8/8 pass, including `test_load_ecosystem_lock_real_file` against the
  real `.chatmangpt/ecosystem.lock.toml`.

CLI demonstration attempt:
```
$ PRAXIS_RETROFIT_OCEL_LOG=/tmp/retrofit-verify.ocel.json ./target/debug/praxis-retrofit ecosystem-conformance --log /tmp/retrofit-verify.ocel.json --reference admission
Failed to load OCEL log at /tmp/retrofit-verify.ocel.json: IO error: failed to read OCEL log at /tmp/retrofit-verify.ocel.json: No such file or directory (os error 2)

$ PRAXIS_RETROFIT_OCEL_LOG=/tmp/retrofit-verify.ocel.json ./target/debug/praxis-retrofit audit scan /Users/sac/praxis/crates/praxis-retrofit
Audit Report: { ...6 checks, score 0.0... }
$ ls /tmp/retrofit-verify.ocel.json
No such file or directory
```
→ BLOCKED, exact reason given in the ALIVE/PARTIAL/BLOCKED section above.

## Explicit non-goals / deferred items

- **Object-centric conformance via token replay**: this PR's `conformance_report()` reports
  present/missing directly-follows arcs, not a full Petri-net/object-centric token-replay
  conformance metric (fitness/precision in the Van der Aalst sense). A token-replay-based
  conformance check is deferred; the current arc-presence check is a coarser, cheaper witness
  and is documented as such, not overclaimed as full conformance checking.
- **BRCE pipeline integration**: per `.claude/rules/semantic-runtime-contracts.md` §1, BRCE is
  the authority root and the only `DO` path in this repo. This PR's OCEL emission is
  observation/proposal only (`emit()`/`ensure_object()` construct log entries; they do not
  actuate machine state or grant standing). Routing ecosystem-conformance results through BRCE
  as an admitted, brokered actuation was evaluated and deferred — the mined conformance report
  is a diagnostic artifact, not (yet) an admitted `O*` feeding a brokered decision. This is a
  poor near-term fit because BRCE admission would require a closed-vocabulary mapping from
  `process_discovery`'s arc-presence findings to a typed intent, which has not been designed.
- **Name-collision merge policy across the three original subsystems**: this PR does not define
  a reconciliation policy for a repo name that appears with conflicting state across
  `repo_registry`, `fleet_*`, and a hand-scanned `audit scan` result — by design, per the
  Motivation section: the whole point of moving to process mining was to avoid inventing such a
  policy. `repo_registry.rs`'s `test_load_with_ecosystem_name_collision_refused` covers only the
  narrower case of a name collision within the ecosystem-lock file itself (refused), not a
  cross-subsystem state collision.
- **`chicago-tdd-tools`'s `ocel-generation` feature**: explicitly not adopted as a dependency,
  per the Step 1 design rationale — its `TestActivity`/`TestObjectType` vocabulary
  (chicago-tdd-tools `src/observability/ocel/types.rs:8-62`) is scoped to test-suite execution
  events and would require either abusing that closed vocabulary to represent unrelated
  process-domain events or forking the enum, both rejected. Only the admission-pattern shape
  (`Evidence<T, Admitted, Witness>`, typed refusals) was borrowed conceptually.

## Risk assessment

- **Low risk to existing behavior**: all changes to pre-existing files (`repo_registry.rs`,
  `fleet_audit.rs`, `fleet_validate.rs`, `fleet_apply.rs`) are additive (new emission calls,
  new tests); no existing function signature or return type was changed per the diff stat
  (363/44/22/28 lines added across those four files, 2 lines subtracted total across the whole
  PR). Existing 80-test suite passes unchanged.
- **Medium risk: untested CLI integration path**. The `ecosystem-conformance` verb
  (`src/bin/praxis-retrofit.rs`, +162 lines) has not been exercised end-to-end in this
  environment (BLOCKED, see above) — its correctness rests on the unit-level guarantees of
  `ocel_log.rs`/`process_discovery.rs` plus manual code inspection, not an observed full-loop
  run. A live-repo-checkout environment should re-run the exact BLOCKED command before this
  path is claimed ALIVE.
- **New dependency surface**: `wasm4pm-compat` is now a direct dependency of `praxis-retrofit`
  (`Cargo.toml` +1 line, `Cargo.lock` +1 line). This is a pre-existing, in-workspace crate
  already depended on elsewhere in the repo (per the Chatman toolchain memory note on canonical
  type owners across wasm4pm/bcinr crates), so it does not introduce a new external/third-party
  dependency, only a new internal edge.
- **No receipt-chain or hash-path changes**: this PR does not touch
  `crates/praxis-graphlaw/src/chatman/` or any BLAKE3 receipt path; the 8-invariant Rust
  core-team discipline items (determinism, no wall-clock in hash paths, receipts computed not
  asserted) are not applicable to this PR's scope, which is test-execution-domain-adjacent
  OCEL logging for repo lifecycle observation, not the sealed Chatman receipt envelope.

## Follow-up tickets suggested

1. Wire OCEL emission into the CLI's pre-existing `audit scan` verb
   (`src/bin/praxis-retrofit.rs`) so a standalone, no-live-checkout-required run can produce a
   real OCEL log end-to-end, unblocking the BLOCKED CLI demonstration above without requiring
   live repo checkouts.
2. Re-run the exact BLOCKED `ecosystem-conformance` command in an environment with live
   checkouts of the `.chatmangpt/ecosystem.lock.toml` member repos (`bcinr`, `wasm4pm`, `ggen`,
   `wasm4pm-compat`, `lsp-max`, `chicago-tdd-tools`) and promote this PR's CLI-path claim from
   BLOCKED to ALIVE or PARTIAL based on the real result.
3. Design a closed-vocabulary mapping from `process_discovery::conformance_report()` output to
   a typed BRCE-admitted intent, per `semantic-runtime-contracts.md` §1/§5, if ecosystem
   conformance is meant to eventually drive brokered actuation (e.g. auto-quarantine a
   non-conformant repo) rather than remain a read-only diagnostic.
4. Evaluate object-centric token-replay conformance (fitness/precision) as a stronger
   replacement for the current arc-presence conformance report, if the coarser check proves
   insufficient in practice.
5. Add a scoped `check-pkg <crate>` justfile recipe (mirroring the existing `test-pkg`) so a
   single crate's `cargo check` can be isolated from unrelated workspace-wide compile failures
   (this PR's own verification had to route around three unrelated broken crates: `cng`,
   `praxis-synthesis`, `affidavit`).
