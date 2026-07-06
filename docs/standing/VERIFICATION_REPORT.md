# Standing Compiler Pipeline — Integration Verification Report

Read-and-verify pass across `/Users/sac/praxis`, `/Users/sac/cargo-cicd`, and
`/Users/sac/anti-llm-cheat-lsp`. Each section states ATTEMPTED, PASSED,
PARTIAL, or GAP with the command run and the exact evidence. No item was
skipped; no rounding up.

## 1. `cargo-cicd standing refresh` produces `standing.json`

**Verdict: PASSED, with one spot-check GAP.**

Command (per `justfile`'s own note that the binary must be invoked directly,
not via `cargo cicd`):

```
$ cargo-cicd standing refresh
standing refresh: 16 artifact(s) -> ./target/praxis-standing/standing.json
```

Exit 0. `target/praxis-standing/standing.json` parses as valid JSON
(`python3 -c "import json; json.load(open(...))"`) with top-level keys
`release_id` (`v26.6.30`), `generated_at_utc`, `generator`,
`standing_version`, `artifacts` (16 entries).

**Evidence coverage**: every artifact has `>=1` evidence entry except one:
`receipt-ledger:receipt-log` (path `.ggen-v2/receipt-log.jsonl`) carries
`"evidence": []` — the receipt-ledger ingestor records the artifact's
existence but does not yet populate an evidence entry for it. This is a
narrow, real gap in the ingestor, not a refusal or crash.

**Ladder spot-checks**:
- `autonomic-platform` (client): `ladder_level=1`, standing
  `[DISCOVERED, BUILDS]`, evidence = command `npm run build`, exit 0.
  Matches a live build I observed during refresh (`vite build`, 29 modules,
  built in ~370ms).
- `praxis-graphlaw`: **not present as its own artifact.** Only
  `bench:bench-graphlaw` (kind `bench`, ladder 0) exists, sourced from
  `docs/releases/v26.7.6/ocel/raw/bench-graphlaw.txt`. Cross-checked three
  ways:
  1. `standing.json`'s 16 artifact ids contain no `praxis-graphlaw` id.
  2. `cicd.toml`'s `[standing.release_gates]` names
     `{ artifact_id = "praxis-graphlaw", status = "PUBLISH_READY" }` for
     `v26.7.6`. Running `cargo-cicd release_gate check --release-id v26.7.6`
     gives:
     ```
     release-gate v26.7.6: 0/2 satisfied
       MISSING praxis-graphlaw PublishReady: artifact not found in standing.json
       MISSING autonomic-platform Tested: artifact standing is [Discovered, Builds], missing required status
     ```
     (The gate mechanism itself works correctly — it names the exact missing
     artifact/status rather than silently passing.)
  3. `docs/standing/PRODUCTION_READINESS.md`'s own worked walk-through
     narrates `crates/praxis-graphlaw` at rung 7 (`PUBLISH_READY`,
     `BUILDS, TESTED, RECEIPT_VERIFIED, OCEL_PROVEN, WASM4PM_PROVEN`) — that
     narration is not backed by the live `standing.json`; the crate is
     ingested nowhere except as a benchmark-output row. This is a gap in the
     standing compiler's crate/client ingestor (`praxis-graphlaw` needs a
     `[[standing.clients]]`-style or `rust_crate` ingestion entry in
     `cicd.toml`, which doesn't exist today — only `autonomic-platform` is
     registered under `[[standing.clients]]`), not a fabricated claim in the
     doc (the doc doesn't claim the number comes from a live index run).
- `dashboard.bak` reclassification (`docs/releases/v26.7.6/CLIENT_SURFACES.md`
  line 19/59): the doc records
  `dashboard.bak: RESOLVED_BY_SCOPE_RECLASSIFICATION` (Nuxt UI Pro is a paid
  external dependency, not release-critical) with `playground-web` promoted
  canonical (`NO_TERMINAL_BLOCKERS.md`). **Gap, not failure**: this
  reclassification is not yet ingested into `standing.json` at all —
  neither `dashboard.bak` nor `playground-web` appears as a standing
  artifact. There is no client/ingestion wiring for either path in
  `cicd.toml`'s `[[standing.clients]]` (only `autonomic-platform` is
  registered), so the CLIENT_SURFACES.md reclassification and the compiled
  standing index are currently two independent, unlinked sources of truth.

## 2. `cargo-cicd standing verify`

**Verdict: PASSED for the "right after refresh" case; PARTIAL for drift
detection.**

```
$ cargo-cicd standing verify
standing verify: 0 drifted artifact(s)
```
Exit 0, immediately after refresh, as expected.

Drift-detection probe: touched `.ggen-v2/receipt.json`'s mtime
(`touch .ggen-v2/receipt.json`), re-ran verify:
```
$ cargo-cicd standing verify
standing verify: 0 drifted artifact(s)
```
Still exit 0, no drift reported. Read `cargo-cicd/src/nouns/standing.rs`
(`compute_drift`, `drift_entry_for`, lines 218-251) to understand why this is
correct-as-implemented rather than broken: `compute_drift` diffs only two
fields per artifact — `path` and `standing` (the ladder status list) —
between the persisted and freshly-ingested artifact lists. It does not
hash-diff evidence content. Two consequences, reported honestly:
- `.ggen-v2/receipt.json` (the file the checklist named) isn't even an
  evidence path for any artifact — `receipt-ledger:receipt-log` points at
  the sibling `.ggen-v2/receipt-log.jsonl`, and that artifact's evidence
  array is empty (see §1), so it has nothing to drift on regardless.
- More generally: verify's drift detector is **added/removed/path-changed/
  standing-changed** only. A content edit to a source artifact that doesn't
  change its computed standing (e.g. editing `CLAIM_PROMOTION_TABLE.md` text
  without altering discoverability) would also **not** be flagged as drift
  today, because there is no per-evidence content-hash comparison in
  `compute_drift`. This is a real, un-implemented layer of the
  verify/drift-ingestor, not a bug in the code that exists — the existing
  added/removed/changed diff is correctly implemented and passing, it's just
  narrower than "any evidence content changed."

## 3. `standing.ocel.json` / `ocel_process_validate`

**Verdict: GAP.**

`target/praxis-standing/standing.ocel.json` does not exist after refresh
(`ls target/praxis-standing/*.ocel.json` → no matches). The standing
compiler does emit an OCEL-adjacent artifact, but under a different name and
shape: `emit_standing_ocel` (`cargo-cicd/src/nouns/standing.rs:130`) appends
one `standing_compiled` event per artifact via `crate::ocel::append_ocel_event`
to `.cargo-cicd/ocel/events.jsonl` — a flat, hash-chained JSONL log
(`{event_id, event_type, timestamp, objects, git_delta, prev_hash,
event_hash}` per line), not an OCEL 2.0 Shape-A document (no top-level
`objects`/`eventTypes`/`objectTypes` arrays).

Confirmed structurally by running the validator against it directly:
```
$ cargo run --bin ocel_process_validate -- .cargo-cicd/ocel/events.jsonl
[ocel_process_validate] refusal: OCEL parse refusal on .cargo-cicd/ocel/events.jsonl: invalid type: map, expected a sequence at line 1 column 117
```
Exit 2 (typed `Refusal`, no panic — consistent with invariant #1). This
confirms `events.jsonl` is not Shape-A OCEL and the validator correctly
refuses it by structure rather than silently accepting it.

`ocel_process_validate` does accept an arbitrary input path as `argv[1]`
(falls back to `DEFAULT_LOG =
"docs/releases/v26.7.6/ocel/playwright-wasm4pm-validation.ocel.json"` only
when no arg is given — `src/bin/ocel_process_validate.rs:712-715`), so the
CLI is not the limiting factor here. There is simply no
`standing.ocel.json` artifact produced by the standing compiler to point it
at, and no other candidate file at that name exists anywhere under
`target/praxis-standing/`. Nothing to spot-check "by hand" — the gap is the
absence of the export, not a malformed export.

## 4. `docs/standing/REALITY_INDEX.md`

**Verdict: PASSED.**

The file exists with the required header:
```
GENERATED by `ggen sync` from `packs/standing-pack/ontology.ttl` ...
Do not edit by hand: this report is a pure SPARQL projection of the compiled
`praxis-standing.v1` index.
```

Receipt presence: `.ggen-v2/receipt-log.jsonl` carries entries whose
`payload.outputs` and `payload.decisions` maps key on
`docs/standing/REALITY_INDEX.md` (e.g. `"docs/standing/REALITY_INDEX.md":
"written"`), with `ts_ns: 0` on the receipt record itself (genesis-folded,
no wall clock in the hash path — invariant #3 compliant).

Determinism: ran `just standing` twice back-to-back and diffed
`docs/standing/REALITY_INDEX.md` between the two runs —
**byte-identical** (`diff` returns no output). The sibling file
`target/praxis-standing/CLAUDE_CODE_CONTEXT.md` *does* embed a live
`generated <UTC timestamp>` header that differs run-to-run — but that is a
separate, human-facing display doc, not `REALITY_INDEX.md`, and this
convention (real RFC3339 time on display docs derived from `ts_ns`, never a
raw wall-clock stamp inside a receipt/hash path) matches the precedent in
`crates/praxis-core/src/ocel_export.rs`'s `ts_ns_to_rfc3339`, which
explicitly derives display time from `ts_ns` rather than `SystemTime::now()`.
No determinism gap on the artifact the checklist named.

`git status`/`git diff` on `docs/standing/REALITY_INDEX.md` after all of the
above: clean — the file the two `just standing` runs regenerated matches
what's already committed at `HEAD`, corroborating the determinism finding
independently.

## 5. `anti-llm-cheat-lsp` scan — fixtures and real-repo run

**Verdict: PARTIAL on fixtures (mechanism works, but not through the literal
CLI invocation named in the checklist); ATTEMPTED and reported for the real
run.**

All 6 `standing_*` negative-control fixtures exist under
`fixtures/negative_controls/`. Ran:
```
$ cargo run --bin anti-llm-cheat-lsp -- server scan --dir \
    /Users/sac/anti-llm-cheat-lsp/fixtures/negative_controls --format json
```
(`scan` is a verb under the `server` noun, not a top-level subcommand — the
checklist's literal `cargo run -- scan --dir ...` doesn't resolve; `server
scan --dir <DIR>` is the actual invocation, confirmed via `--help`.)

Result: only `standing_unscoped_claim.md` produced a diagnostic
(`ANTI-LLM-STANDING-001`, x2). The other 5 fixtures
(`standing_alive_without_receipt`, `standing_benchmark_without_artifact`,
`standing_claim_outruns_index`, `standing_dry_run_as_publish`,
`standing_stale_index`) produced **zero** diagnostics under this invocation.
Root cause, read from `src/rules/standing.rs`'s own doc comment and
confirmed experimentally: `ANTI-LLM-STANDING-000/002/003/004/005/006` are
gated by a `[standing]` table in the *scanned directory's* `anti.toml`
(`AntiLlmConfig::load_from_dir(dirpath)` loads config from `--dir`, not from
the repo root); `fixtures/negative_controls/` has no `anti.toml` of its own,
so the subsystem stays off (only `-001`, the purely textual/config-free
check, always runs). I confirmed this by temporarily adding a `[standing]`
table to the repo-root `anti.toml` and re-running — no change, because that
file isn't in the scanned directory. I reverted this test edit immediately
(`git diff anti.toml` is clean).

The mechanism itself is not broken: `cargo test --test dogfood standing`
(the tests that construct a `StandingConfig` in-process, pointing at
`fixtures/standing/standing.json` / `standing_stale.json`, one config per
fixture, exactly matching what each fixture needs) — **all 7 pass**
(`detects_standing_unscoped_claim`, `detects_standing_claim_outruns_index`,
`detects_standing_dry_run_as_publish`, `detects_standing_alive_without_receipt`,
`detects_standing_benchmark_without_artifact`, `detects_standing_stale_index`,
`standing_positive_control_stays_clean`). The gap is specifically that the
bare CLI `server scan --dir <fixtures dir>` invocation the checklist
describes does not, by itself, wire up per-fixture standing config the way
the test harness does — there's no `anti.toml` co-located with the fixtures
today.

Real-repo run:
```
$ cargo run --bin anti-llm-cheat-lsp -- server scan --dir /Users/sac/praxis --format json
```
Exit 0. **3,347 total diagnostics** across 31 distinct codes. Top codes:
`ANTI-LLM-METRIC-003` (782), `ANTI-LLM-STRANGE-008` (653),
`ANTI-LLM-CLAIM-004` (366), `ANTI-LLM-TEST-005` (228),
`ANTI-LLM-STRANGE-009` (210). `ANTI-LLM-STANDING-001` fired 25 times (again,
only the config-free unscoped-claim check — praxis's root `anti.toml` has no
`[standing]` table either, so 002-006 don't run on this scan). Three
concrete examples:
```
[ANTI-LLM-METRIC-003] /Users/sac/praxis/backup_template/benches/bench_main.rs:39: Function 'bench_throughput' max nesting depth 6 (threshold 4)
[ANTI-LLM-STANDING-001] /Users/sac/praxis/CLAUDE.md:24: 'production-ready' claim has no captured scope phrase ('for <scope>' / 'scoped to <scope>').
[ANTI-LLM-STANDING-001] /Users/sac/praxis/COMPLIANCE_DASHBOARD_MANIFEST.md:18: 'production-ready' claim has no captured scope phrase ('for <scope>' / 'scoped to <scope>').
```
No fixes attempted against the real-repo findings — that's explicitly
follow-up work per the checklist, not this pass.

## 6. `cargo-cicd claude_context show`

**Verdict: PASSED.**

```
$ cargo-cicd claude_context show
```
(verb is `claude_context show`, not `claude-context show` — confirmed via
the `justfile`'s own `standing` recipe, which invokes
`cargo-cicd claude_context show`.)

Output is a Markdown listing of all 16 artifacts with the same
`standing`/`ladder`/`evidence`/`next` fields as `standing.json`. Cross-checked
programmatically: `standing.json` has 16 artifact ids; `claude_context show`
prints exactly 16 `- ` bullet lines, one per id, with matching standing
lists (e.g. `autonomic-platform: standing=[Discovered,Builds], ladder 1,
scope=none, evidence: command: npm run build` matches the JSON's
`"standing": ["DISCOVERED","BUILDS"]`, `"ladder_level": 1`, evidence command
`npm run build`). Fully consistent.

## 7. Test suites

**Verdict: ATTEMPTED, all green after one fix-forward correction (see
below).**

| Suite | Command | Result |
|---|---|---|
| cargo-cicd workspace | `cargo test --workspace --no-fail-fast` | **250 passed, 0 failed** across 47 test binaries |
| anti-llm-cheat-lsp dogfood | `cargo test --test dogfood` | **70 passed, 0 failed** |
| praxis DoD gate | `just verify-all` (check + test + clippy + doctor) | see below |

**One real, small bug found and fixed (fix-forward, smallest diff) in
`cargo-cicd`**: the first `cargo test --workspace` run (without
`--no-fail-fast`) stopped after the *first* failing test binary:
```
test cicd_toml::tests::valid_cicd_toml_admits ... FAILED
thread '...' panicked at src/cicd_toml.rs:352:24:
valid config should pass validation: ValidationErrors { ... code: "invalid_toolchain" ...
  msg: "must be stable, beta, nightly, or a version string starting with a digit" }
```
Root cause: `CicdToml::default()` calls `detect_toolchain()`, which reads
this repo's own `rust-toolchain.toml` (`channel = "nightly-2026-06-22"`, a
dated rustup nightly — this project is pinned to it deliberately, per that
file's own comment, for `wasm4pm-compat`'s `generic_const_exprs` build). The
`valid_toolchain` predicate in `Validate for CicdToml` only accepted the
literal strings `"stable"`/`"beta"`/`"nightly"` or a string starting with a
digit — it rejected the exact dated-nightly form this repo pins itself to.
So `CicdToml::default()`, round-tripped through `write`/`from_file`/`check`,
failed its own validator — a self-referential dogfooding bug (the tool
couldn't validate its own default config in its own repo).

Fix applied (`/Users/sac/cargo-cicd/src/cicd_toml.rs`, `Validate for
CicdToml::validate`): added `|| tc.starts_with("nightly-")` to the
`valid_toolchain` predicate, and updated the rejection message to mention
the dated-nightly form. Re-ran `cargo test --lib cicd_toml`: 4 passed, 0
failed. Full re-run with `--no-fail-fast`: 250 passed, 0 failed, 0 gated
early exits. **This fix is applied in the working tree at
`/Users/sac/cargo-cicd/src/cicd_toml.rs` but not committed** — per this
task's instructions, the only commit this pass makes is the praxis
documentation commit below; the cargo-cicd fix is left for the user to
review and commit in that repo.

`just verify-all` in praxis (`check` → `test` → `clippy -D warnings` →
`doctor check`, stopping at first failure):
```
$ just verify-all
...
verify-all: check + test + clippy + doctor all passed
```
Exit 0. Aggregate test count across all 153 `test result:` lines in the
run: **1,566 passed, 0 failed** (`grep -oE "([0-9]+) passed; ([0-9]+)
failed"` summed over the full log). `clippy --all-targets --all-features -- -D
warnings` passed (the chain would have stopped at `clippy` otherwise — it
did not). `doctor check` reported `Overall: HEALTHY`: config admitted
(witness `9db286ed...`), Frontier `pass_rate=1.00 coverage=0.10
evaluated=30/286 failures=0`, Receipts `0 records`, required tools (`git`,
`cicd-evidence-gen`) on `PATH`, and all 12 feature flags compiled in
(`typestate`, `repl`, `otel`, `discovery`, `lsp`, `andon`, `mcp`, `ggen`,
`law-signed`, `law-ocel`, `testbed`, `proposer`).

## Summary table

| # | Item | Verdict |
|---|---|---|
| 1 | `standing refresh` -> standing.json, evidence coverage, ladder spot-checks | PASSED (1 evidence gap: receipt-ledger; `praxis-graphlaw` and `dashboard.bak`/`playground-web` not ingested — GAP, documented) |
| 2 | `standing verify` exit-0-after-refresh + drift probe | PASSED (verify itself) / PARTIAL (drift detector is add/remove/path/standing-only, no evidence-content hash diff) |
| 3 | `standing.ocel.json` Shape-A validity | GAP (file never produced; compiler emits a different, non-OCEL hash-chained log instead) |
| 4 | `REALITY_INDEX.md` generation, receipt, determinism | PASSED (byte-identical across 2 runs, receipt-linked, correct header) |
| 5 | anti-llm-cheat-lsp fixtures + real-repo scan | PARTIAL (5/6 fixtures need per-fixture config the bare CLI scan doesn't wire; mechanism verified working via `cargo test --test dogfood`) / ATTEMPTED (real scan: 3,347 diagnostics, 31 codes, examples given) |
| 6 | `claude_context show` vs `standing.json` consistency | PASSED (16/16 artifacts match) |
| 7 | Test suites (3 repos) | PASSED — cargo-cicd 250/250, anti-llm-cheat-lsp dogfood 70/70, praxis verify-all 1,566/1,566 + clippy + doctor HEALTHY (one small fix-forward bug found and fixed in cargo-cicd's toolchain-string validator; left uncommitted per this task's commit scope) |
