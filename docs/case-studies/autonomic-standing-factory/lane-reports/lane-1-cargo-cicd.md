# Lane 1 — cargo-cicd Front Door and Standing Compiler

Status: DONE (with one out-of-scope, unfixed pre-existing defect noted).

## Concurrent-edit disclosure

Before touching anything, `git status`/`git diff` in `/Users/sac/cargo-cicd`
showed live uncommitted changes to exactly the files this ticket named as
at-risk (`src/main.rs`, `crates/cargo-cicd-core/src/standing/{emit,mod,model}.rs`,
`src/cicd_toml.rs`, `src/nouns/{claude_context,release_gate,standing}.rs`).
Read every diff before editing. Repo went from 32 to 40+ commits ahead of
`origin/main` during this session — another agent was actively committing
to the same non-worktree checkout in parallel, working the same ticket.
That session had already:

- fixed `cargo cicd` dispatch (argv-stripping) and `--version` identity
  (commit `6565004`)
- fixed TTL `generated_at_utc` non-determinism (commit `b7a7095`)
- renamed the schema id to `cicd-standing.v1` with a legacy alias (commit `a0f6605`)
- implemented `ingest_workspace_crates` with tests and wired it into
  `ingest_all` (commits `35978d7`, `81f0876`)

No destructive action was taken against any of this. Where their
implementation had a real bug (two failing tests in
`ingest_workspace_crates`, see below), it was fixed forward in a separate
commit with an explanation, not silently rewritten. `plugins/cargo-cicd-kit/standing-pack/ontology.ttl`
and `docs/convergence/FLEET_STANDING_CONVERGENCE.md` were left untouched —
generated/in-flight output owned by that other session, not this lane.

## Files inspected

- `/Users/sac/cargo-cicd/src/main.rs`, `src/cicd_toml.rs`,
  `src/nouns/{standing,claude_context,release_gate}.rs`
- `/Users/sac/cargo-cicd/crates/cargo-cicd-core/src/standing/{model,mod,emit,sources,glob,score}.rs`
- `/Users/sac/cargo-cicd/crates/cargo-cicd-core/src/workspace/snapshot.rs`
- `/Users/sac/cargo-cicd/Cargo.toml`, `crates/cargo-cicd-core/Cargo.toml`, `Cargo.lock`
- `/Users/sac/cargo-cicd/justfile`
- `/Users/sac/wasm4pm-compat/src/ocel.rs` (read-only, to match Shape-A field
  names/casing exactly — not edited, per the control ledger's non-goal list)
- `/Users/sac/praxis/justfile` (`standing` recipe), `/Users/sac/praxis/cicd.toml`

## Files changed

cargo-cicd repo (4 commits, `main` branch, not pushed):

1. `crates/cargo-cicd-core/src/standing/sources.rs` — fixed two failing
   tests in the (already-committed-by-the-other-session)
   `ingest_workspace_crates`: a test that clobbered its own root
   `Cargo.toml`'s `[workspace]` section by writing the `"."` member
   separately, and a test with an incorrect `UNSEEN` expectation where
   `DISCOVERED` is the ingestor's actual (and correct, per its own
   documented fallback contract) behavior.
2. `Cargo.toml`, `Cargo.lock`, `crates/cargo-cicd-core/src/standing/emit.rs`,
   `src/nouns/standing.rs` — added `render_standing_ocel_shape_a` /
   `write_standing_ocel_shape_a` (Shape-A OCEL snapshot matching
   `wasm4pm_compat::ocel::OCEL`'s exact field names/casing), wired into
   `write_all_outputs` so `target/praxis-standing/standing.ocel.json` is
   produced on every refresh. Added `wasm4pm-compat` as a dev-dependency
   and a unit test parsing the emitted JSON with the real
   `wasm4pm_compat::ocel::OCEL` type.
3. `crates/cargo-cicd-lsp/src/analyzers/cicd_toml_schema.rs` — one-line
   pre-existing clippy fix (`Iterator::last` → `next_back`), unrelated to
   this ticket, fixed in passing since it blocked
   `cargo clippy --all-targets --all-features -D warnings`.
4. `crates/cargo-cicd-core/src/standing/emit.rs` — **real determinism bug
   found while dogfooding against praxis**: `EvidenceRef::Command`'s `utc`
   field (wall clock) was being serialized verbatim into `standing.ttl`'s
   `praxis:evidence` literals, so the TTL still changed on every refresh
   whenever any artifact carried Command evidence (praxis's
   `doctor_command` config triggers this). Added
   `ttl_safe_evidence_json` to strip `utc` from the TTL projection only
   (still recorded in full in `standing.json`), plus a regression test.

praxis repo (this commit + the one that follows this file):

- `docs/case-studies/autonomic-standing-factory/lane-reports/lane-1-cargo-cicd.md` (this file)
- `docs/case-studies/autonomic-standing-factory/CASE_STUDY_CONTROL.md` (phase rows 1–2 updated)
- Regenerated evidence from the dogfood proof: `ggen.lock`,
  `docs/standing/REALITY_INDEX.md`, `.ggen-v2/receipt-log.jsonl`,
  `.ggen-v2/receipt.json`, `.cargo-cicd/ocel/events.jsonl`

## Commands run (cargo-cicd repo, cwd `/Users/sac/cargo-cicd`)

| Command | Exit | Notes |
|---|---|---|
| `cargo-cicd --version` (pre-fix, installed binary) | 0 | already reported `cargo-cicd 26.6.30` — the other session's dispatch/version fix (`6565004`) was already installed before I started |
| `cargo cicd --version` (pre-fix) | 0 | same — cargo-subcommand form already worked |
| `cargo fmt` | 0 | clean both times run |
| `cargo build` | 0 | 114 pre-existing dead-code/unused warnings, no errors |
| `cargo test --workspace` | 0 | 300+ tests across all crates, 0 failed, after my two test-bug fixes (2 tests failed before the fix, see below) |
| `cargo clippy --all-targets --all-features -D warnings` | 1 (pre-existing, then 1 again) | see "Findings not fixed" below — one trivial lint fixed, one large pre-existing `--all-features` breakage left alone (out of scope) |
| `cargo cicd --help` | 0 | lists all nouns incl. `standing`, `release_gate` |
| `cargo cicd standing --help` | 0 | |
| `cargo cicd standing refresh` | 0 | `standing refresh: 10 artifact(s) -> ./target/praxis-standing/standing.json` (bare repo, no doctor_command configured) |
| `cargo cicd standing report` | 0 | table incl. 3 `RustCrate` rows: `crate:cargo-cicd`, `crate:cargo-cicd-core`, `crate:cargo-cicd-lsp` |
| `cargo cicd standing verify` | 0 | `standing verify: 0 drifted artifact(s)` |
| `cargo cicd claude_context show` | 0 | one line per artifact, incl. the 3 crate lines |
| `cargo install --path . --force` (x3, after each fix batch) | 0 each | `Replaced package cargo-cicd v26.6.30 ... with cargo-cicd v26.6.30` |

## Artifacts produced

- `/Users/sac/cargo-cicd/target/praxis-standing/{standing.json,standing.ttl,standing.ocel.json,CLAUDE_CODE_CONTEXT.md,benchmark-summary.json,receipt-summary.json,client-surface-summary.json,claim-index.json,LSP_DIAGNOSTICS.json}`
- `/Users/sac/praxis/target/praxis-standing/{standing.json,standing.ttl,standing.ocel.json,CLAUDE_CODE_CONTEXT.md,...}` (dogfood run)
- `/Users/sac/praxis/docs/standing/REALITY_INDEX.md` (regenerated, 28 artifacts, 12 `RustCrate`)

## Tests passed

- `cargo test --workspace` in cargo-cicd: all suites green after fixes
  (`cargo-cicd-core` lib: 59 tests; `cargo-cicd` lib + integration tests:
  ~15 test binaries, all `0 failed`).
- New tests added and passing: `ocel_shape_a_has_one_event_and_object_per_artifact`,
  `ocel_shape_a_event_relationship_points_at_matching_object`,
  `write_standing_ocel_shape_a_round_trips_as_json`,
  `standing_ocel_shape_a_parses_as_wasm4pm_compat_ocel` (parses the emitted
  JSON with the real `wasm4pm_compat::ocel::OCEL` type),
  `ttl_is_deterministic_with_command_evidence_carrying_wall_clock_utc`.

## Failures found

1. **Two failing tests in the already-committed `ingest_workspace_crates`**
   (`workspace_crates_literal_members_are_discovered_rust_crates`,
   `workspace_crates_empty_members_is_unseen`) — both test-authoring bugs,
   not implementation bugs. Fixed forward (see Files changed #1).
2. **Real TTL determinism gap**: `standing.ttl` was not actually
   byte-identical across runs once any artifact carried
   `EvidenceRef::Command` evidence (e.g. a configured `doctor_command`),
   because the evidence's wall-clock `utc` field was serialized verbatim
   into the TTL. Reproduced live: `just standing` in praxis failed on the
   *second* run with `[FM-PACK-008] pack standing-pack ... content hash
   mismatch` even though the repo state was unchanged between the two
   `cargo-cicd standing refresh` calls. Fixed forward (Files changed #4).
3. **`standing.ocel.json` never produced by refresh** (confirmed: only
   `.cargo-cicd/ocel/events.jsonl`, a different, append-only, non-Shape-A
   ledger, was written). Fixed forward (Files changed #2).
4. **Zero `RustCrate` artifacts in standing index** prior to my session's
   commits landing — confirmed via `cargo cicd standing report` showing 7
   artifacts, all `Doc`/`Workflow`/`Bench`/`Client` kind. Resolved (already
   fixed by the concurrent session's `ingest_workspace_crates` + wiring,
   modulo the two test bugs I fixed).
5. **Pre-existing, out-of-scope**: `cargo clippy --all-targets
   --all-features -D warnings` fails to even *compile* with 6 errors
   (`E0432`/`E0433`: unresolved crate `lsp_max_anti_cheat` referenced from
   `src/legacy_nouns/lsp.rs` behind the `anti-llm-cheat` feature, which is
   declared in `Cargo.toml` as `anti-llm-cheat = []` — an empty feature
   with no dependency ever wired to it) plus several unrelated
   unused-variable/mutability lints in `src/legacy_nouns/pipeline.rs` and
   `src/integrations/metrics_collector.rs`. This is unrelated to standing/
   dispatch, predates this session (confirmed via `git show HEAD` on the
   affected files before any of my edits), and `--all-features` is not
   exercised by plain `cargo build`/`cargo test` (which are green). Not
   fixed — wiring in a missing external crate is outside this ticket's
   named repair list and risks side effects in an unrelated subsystem.
   **Not required by PART C's explicit command list**, which does not
   include `clippy`/`verify-all`.

## Repairs made

See "Files changed" above — dispatch/version/TTL-header-determinism/schema-id
were already repaired by the concurrent session before I started; this
session added: workspace-crate ingestion test fixes, Shape-A OCEL emission,
one pre-existing clippy fix, and the Command-evidence TTL determinism fix
found by dogfooding.

## Determinism proof (PART D)

Two consecutive `cargo cicd standing refresh` runs in `/Users/sac/cargo-cicd`
(bare repo, no Command evidence), sha256 of `target/praxis-standing/standing.ttl`:

```
87ca7a85f497f855ded53672177db1e1c20c14d1f507770694f91b78bed1a3be
87ca7a85f497f855ded53672177db1e1c20c14d1f507770694f91b78bed1a3be
```

Identical. `standing.json`'s `generated_at_utc` sidecar timestamp is the
only field that changes between the two runs (allowed volatility,
documented in `emit.rs`'s module doc).

After the Command-evidence TTL fix, re-verified end-to-end against
praxis's real config (which does set `doctor_command`, exercising the
Command-evidence code path this fix targets) — see Praxis dogfood proof
below for the actual two-run hash match under that config.

## Praxis dogfood proof (PART E, folded in per instructions)

`just standing` in `/Users/sac/praxis` runs `cargo-cicd standing refresh`,
copies the resulting `standing.ttl` into cargo-cicd's own ggen pack, then
runs `ggen sync run` (which fails closed on any content-hash mismatch
against `ggen.lock` — a genuine determinism gate, not a rubber stamp).

- First run (after the OCEL/rust_crate fixes landed but before the
  Command-evidence TTL fix): failed with `[FM-PACK-008] ... content hash
  mismatch` — this is failure #2 above, caught live by this exact gate.
- Deleted `ggen.lock` once, intentionally, to accept the legitimate content
  change (new `RustCrate` artifacts + `standing.ocel.json` genuinely
  changed the pack's content — this is not the old "spurious mismatch on
  unchanged input" bug the header-timestamp fix already solved; the
  justfile's own comment documents that specific bug as already fixed and
  the `rm -f ggen.lock` workaround as no longer needed for *that* case).
- After fixing Command-evidence TTL leakage and reinstalling: ran
  `just standing` twice in a row with **no** `ggen.lock` deletion between
  runs. Both succeeded. sha256 of `target/praxis-standing/standing.ttl`
  both times:

```
4127bda98f880480c79c04aa336592449f12035ab05d3efc60019429bee31d28
4127bda98f880480c79c04aa336592449f12035ab05d3efc60019429bee31d28
```

Identical — this is the real proof: praxis's actual `doctor_command`
config exercises Command evidence, and the pipeline is now stable across
repeat runs without the `rm -f ggen.lock` workaround.

- `docs/standing/REALITY_INDEX.md` rendered: 28 artifacts total, grouped by
  kind, including 12 `RustCrate` rows (`crate:agent8`, `crate:chatman-common`,
  `crate:ggen`, `crate:pddl-index`, `crate:powl2-decompose`,
  `crate:praxis-core`, `crate:praxis-graphlaw`, `crate:praxis-lean`,
  `crate:praxis-proposer`, `crate:praxis-retrofit`, plus 2 more) — this is
  the schema doc's own worked example (`praxis-graphlaw`) actually showing
  up, which it did not before this lane's work.
- Praxis's `justfile` (`standing` recipe, lines ~8–24) already documents
  that it calls `cargo-cicd standing refresh` directly — confirmed: praxis
  has no duplicate/local standing-compiler implementation (`grep` for
  `standing_compiler`/`standing*.rs` outside `target/` found nothing in
  praxis beyond the consuming ggen pack).

## Remaining external side effects

None from this lane. `cargo install --path . --force` updated the local
`~/.cargo/bin/cargo-cicd` binary on this machine (expected, not pushed
anywhere). Nothing was pushed to any remote in either repo.

## Handoff to next lane

- `target/praxis-standing/standing.ocel.json` (both repos) is now real
  Shape-A OCEL, parseable by `wasm4pm_compat::ocel::OCEL` — Lane 4 can
  point wasm4pm process-conformance validation at it directly.
- `docs/standing/REALITY_INDEX.md` and `target/praxis-standing/standing.ttl`
  reflect the current, deterministic, dogfooded state — Lane 2 (GraphLaw)
  can treat `standing.ttl` as a stable input.
- The pre-existing `--all-features` clippy/compile breakage
  (`lsp_max_anti_cheat` unresolved crate, `anti-llm-cheat` empty feature)
  is unresolved and out of this lane's scope; flagging for whichever lane
  owns `anti-llm-cheat-lsp` policy integration, per the control ledger's
  "policy handoff notes only" note on that project.

## Evidence paths

- cargo-cicd commits: `2071fa7`, `2124607`, `c47c9ab`, `cbe0bee` (this
  session) on top of `81f0876` and earlier (concurrent session), branch
  `main`, 44 commits ahead of `origin/main`, not pushed.
- `/Users/sac/cargo-cicd/target/praxis-standing/standing.ocel.json`
- `/Users/sac/praxis/target/praxis-standing/standing.ocel.json`
- `/Users/sac/praxis/docs/standing/REALITY_INDEX.md`
- `/Users/sac/praxis/ggen.lock`
- This report: `docs/case-studies/autonomic-standing-factory/lane-reports/lane-1-cargo-cicd.md`
